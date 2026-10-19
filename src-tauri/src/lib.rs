use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppSettings {
    pub exe_path: String,
    pub download_dir: String,
    pub max_running: u32,
    pub api_port: u32,
    pub proxy_port: u32,
    pub proxy_system: bool,
    pub play_done_audio: bool,
    pub echolog: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            exe_path: String::new(),
            download_dir: "%UserDownloads%".into(),
            max_running: 3,
            api_port: 2022,
            proxy_port: 2023,
            proxy_system: true,
            play_done_audio: true,
            echolog: false,
        }
    }
}

struct AppData {
    settings: AppSettings,
    child: Option<Child>,
}

impl AppData {
    fn settings_path(app: &AppHandle) -> Option<PathBuf> {
        app.path().app_config_dir().ok().map(|d| d.join("settings.json"))
    }

    fn load(app: &AppHandle) -> Self {
        let settings = Self::settings_path(app)
            .and_then(|p| fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        Self {
            settings,
            child: None,
        }
    }

    fn save_settings(&self, app: &AppHandle) -> Result<(), String> {
        let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let path = dir.join("settings.json");
        let json = serde_json::to_string_pretty(&self.settings).map_err(|e| e.to_string())?;
        fs::write(path, json).map_err(|e| e.to_string())
    }

    /// 在核心程序所在目录旁生成 config.yaml（跟随 exe，便于用户查看）
    fn write_config(&self, exe_dir: &std::path::Path) -> PathBuf {
        let exe_dir_str = exe_dir.display().to_string().replace('\\', "/");
        let yaml = format!(
            r#"# 由 wx-dl-ui 自动生成，保存设置时会覆盖
debug:
  error: false
  echolog: {echolog}

workdir: "{exe_dir}"

download:
  filenameTemplate: "{{{{filename}}}}_{{{{spec}}}}"
  dir: "{download_dir}"
  playDoneAudio: {play_done_audio}
  defaultActionWhenExisting: ""
  maxRunning: {max_running}
  resourceConcurrency: 100
  segmentConcurrency: 10
  connectionConcurrency: 0

api:
  protocol: "http"
  hostname: "127.0.0.1"
  port: {api_port}

mcp:
  enabled: false

scraper:
  retainedJobs: 20

db:
  type: "sqlite"
  host: ""
  port: ""
  username: ""
  password: ""
  filename: ""
  filepath: "{exe_dir}/data.db"

proxy:
  enabled: true
  system: {proxy_system}
  hostname: "127.0.0.1"
  port: {proxy_port}
  tcpRelay:
    enabled: false
    hostname: "127.0.0.1"
    port: 9900
  tun: false
  defaultInterface: ""
  skipInstallRootCert: false
  upstreamProxy: ""

cookie:
  uuid: ""
  password: ""
  key: ""

update:
  sources: []
  proxy: ""
  mirror: ""

channels:
  disableLocationToHome: false
  download:
    defaultHighest: false
    frontend: false
    cover: false
    pauseWhenDownload: false
    forceCheckAllFeeds: false
"#,
            echolog = self.settings.echolog,
            exe_dir = exe_dir_str,
            download_dir = self.settings.download_dir,
            play_done_audio = self.settings.play_done_audio,
            max_running = self.settings.max_running,
            api_port = self.settings.api_port,
            proxy_system = self.settings.proxy_system,
            proxy_port = self.settings.proxy_port,
        );
        let path = exe_dir.join("config.yaml");
        let _ = fs::write(&path, yaml);
        path
    }

    fn is_running(&mut self) -> bool {
        match &mut self.child {
            Some(c) => match c.try_wait() {
                Ok(Some(_)) => {
                    self.child = None;
                    false
                }
                Ok(None) => true,
                Err(_) => {
                    self.child = None;
                    false
                }
            },
            None => false,
        }
    }
}

fn spawn_child(exe_path: &PathBuf, config_path: &PathBuf) -> std::io::Result<Child> {
    let mut cmd = Command::new(exe_path);
    cmd.arg("--config").arg(config_path);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    #[cfg(windows)]
    {
        // CREATE_NO_WINDOW，避免弹出黑窗
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }

    cmd.spawn()
}

fn pipe_logs(app: AppHandle, stream: impl std::io::Read + Send + 'static, name: &'static str) {
    let reader = BufReader::new(stream);
    std::thread::spawn(move || {
        for line in reader.lines() {
            match line {
                Ok(l) => {
                    let _ = app.emit("service-log", format!("[{}] {}", name, l));
                }
                Err(_) => break,
            }
        }
    });
}

#[tauri::command]
fn get_settings(state: State<'_, Mutex<AppData>>) -> AppSettings {
    state.lock().unwrap().settings.clone()
}

#[tauri::command]
fn save_settings(
    app: AppHandle,
    state: State<'_, Mutex<AppData>>,
    settings: AppSettings,
) -> Result<(), String> {
    let mut data = state.lock().unwrap();
    data.settings = settings;
    data.save_settings(&app)
}

#[tauri::command]
fn start_service(app: AppHandle, state: State<'_, Mutex<AppData>>) -> Result<(), String> {
    let mut data = state.lock().unwrap();
    if data.is_running() {
        return Err("服务已在运行".into());
    }
    let exe = data.settings.exe_path.clone();
    if exe.is_empty() {
        return Err("请先在设置中选择 wx_video_download 可执行文件".into());
    }
    let exe_path = PathBuf::from(&exe);
    if !exe_path.is_file() {
        return Err(format!("可执行文件不存在：{}", exe));
    }
    let exe_dir = exe_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    let config_path = data.write_config(&exe_dir);

    let mut child = spawn_child(&exe_path, &config_path)
        .map_err(|e| format!("启动失败：{}", e))?;

    if let Some(stdout) = child.stdout.take() {
        pipe_logs(app.clone(), stdout, "out");
    }
    if let Some(stderr) = child.stderr.take() {
        pipe_logs(app.clone(), stderr, "err");
    }

    let _ = app.emit(
        "service-log",
        format!(
            "[info] 已启动 {}（管理页 http://127.0.0.1:{}，代理端口 {}）",
            exe, data.settings.api_port, data.settings.proxy_port
        ),
    );
    data.child = Some(child);
    Ok(())
}

#[tauri::command]
fn stop_service(state: State<'_, Mutex<AppData>>) -> Result<(), String> {
    let mut data = state.lock().unwrap();
    if let Some(mut child) = data.child.take() {
        #[cfg(unix)]
        {
            // 先 SIGTERM 优雅退出（让代理正确清理系统代理设置），超时后 SIGKILL
            unsafe {
                libc::kill(child.id() as i32, libc::SIGTERM);
            }
            for _ in 0..30 {
                match child.try_wait() {
                    Ok(Some(_)) => return Ok(()),
                    Ok(None) => std::thread::sleep(std::time::Duration::from_millis(100)),
                    Err(_) => break,
                }
            }
            unsafe {
                libc::kill(child.id() as i32, libc::SIGKILL);
            }
        }
        #[cfg(not(unix))]
        {
            let _ = child.kill();
        }
        let _ = child.wait();
    }
    Ok(())
}

#[tauri::command]
fn service_status(state: State<'_, Mutex<AppData>>) -> bool {
    let mut data = state.lock().unwrap();
    data.is_running()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data = AppData::load(app.handle());
            app.manage(Mutex::new(data));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            start_service,
            stop_service,
            service_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
