use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppSettings {
    pub exe_path: String,
    pub config_path: String,
    pub admin: bool,
    pub debug_error: bool,
    pub echolog: bool,
    pub workdir: String,
    pub api_protocol: String,
    pub api_hostname: String,
    pub api_port: u32,
    pub mcp_enabled: bool,
    pub scraper_retained_jobs: u32,
    pub filename_template: String,
    pub download_dir: String,
    pub play_done_audio: bool,
    pub default_action: String,
    pub max_running: u32,
    pub resource_concurrency: u32,
    pub segment_concurrency: u32,
    pub connection_concurrency: u32,
    pub remote_enabled: bool,
    pub remote_protocol: String,
    pub remote_hostname: String,
    pub remote_port: u32,
    pub proxy_system: bool,
    pub proxy_hostname: String,
    pub proxy_port: u32,
    pub tcp_relay_enabled: bool,
    pub tcp_relay_hostname: String,
    pub tcp_relay_port: u32,
    pub tun: bool,
    pub default_interface: String,
    pub skip_install_root_cert: bool,
    pub upstream_proxy: String,
    pub disable_location_to_home: bool,
    pub default_highest: bool,
    pub frontend: bool,
    pub cover: bool,
    pub pause_when_download: bool,
    pub force_check_all_feeds: bool,
    pub db_type: String,
    pub db_filepath: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            exe_path: String::new(),
            config_path: String::new(),
            admin: false,
            debug_error: false,
            echolog: false,
            workdir: String::new(),
            api_protocol: "http".into(),
            api_hostname: "127.0.0.1".into(),
            api_port: 2022,
            mcp_enabled: false,
            scraper_retained_jobs: 20,
            filename_template: "{{filename}}_{{spec}}".into(),
            download_dir: "%UserDownloads%".into(),
            play_done_audio: true,
            default_action: String::new(),
            max_running: 3,
            resource_concurrency: 100,
            segment_concurrency: 10,
            connection_concurrency: 0,
            remote_enabled: false,
            remote_protocol: "http".into(),
            remote_hostname: "192.168.1.2".into(),
            remote_port: 2022,
            proxy_system: true,
            proxy_hostname: "127.0.0.1".into(),
            proxy_port: 2023,
            tcp_relay_enabled: false,
            tcp_relay_hostname: "127.0.0.1".into(),
            tcp_relay_port: 9900,
            tun: false,
            default_interface: String::new(),
            skip_install_root_cert: false,
            upstream_proxy: String::new(),
            disable_location_to_home: false,
            default_highest: false,
            frontend: false,
            cover: false,
            pause_when_download: false,
            force_check_all_feeds: false,
            db_type: "sqlite".into(),
            db_filepath: "%CWD%/data.db".into(),
        }
    }
}

struct AppData {
    settings: AppSettings,
    child: Option<Child>,
    admin_pid: Option<u32>,
}

impl AppData {
    fn config_dir(app: &AppHandle) -> Result<PathBuf, String> {
        let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        Ok(dir)
    }

    /// 默认配置文件路径：app_config_dir/wx-dl/config.yaml
    fn default_config_path(app: &AppHandle) -> Result<PathBuf, String> {
        Ok(Self::config_dir(app)?.join("wx-dl").join("config.yaml"))
    }

    /// 当前生效的配置文件路径
    fn active_config_path(&self, app: &AppHandle) -> Result<PathBuf, String> {
        if self.settings.config_path.is_empty() {
            Self::default_config_path(app)
        } else {
            Ok(PathBuf::from(&self.settings.config_path))
        }
    }

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
            admin_pid: None,
        }
    }

    fn save_settings(&self, app: &AppHandle) -> Result<(), String> {
        let dir = Self::config_dir(app)?;
        let path = dir.join("settings.json");
        let json = serde_json::to_string_pretty(&self.settings).map_err(|e| e.to_string())?;
        fs::write(path, json).map_err(|e| e.to_string())
    }

    /// 依据设置生成完整 config.yaml
    fn build_config_yaml(&self) -> String {
        let s = &self.settings;
        format!(
            r#"# 由 wx-dl-ui 自动生成，保存设置时会覆盖
debug:
  error: {debug_error}
  echolog: {echolog}

workdir: "{workdir}"

download:
  filenameTemplate: "{filename_template}"
  dir: "{download_dir}"
  playDoneAudio: {play_done_audio}
  defaultActionWhenExisting: "{default_action}"
  maxRunning: {max_running}
  resourceConcurrency: {resource_concurrency}
  segmentConcurrency: {segment_concurrency}
  connectionConcurrency: {connection_concurrency}
  remoteServer:
    enabled: {remote_enabled}
    protocol: "{remote_protocol}"
    hostname: "{remote_hostname}"
    port: {remote_port}

api:
  protocol: "{api_protocol}"
  hostname: "{api_hostname}"
  port: {api_port}

mcp:
  enabled: {mcp_enabled}

scraper:
  retainedJobs: {scraper_retained_jobs}

db:
  type: "{db_type}"
  host: ""
  port: ""
  username: ""
  password: ""
  filename: ""
  filepath: "{db_filepath}"

proxy:
  enabled: true
  system: {proxy_system}
  hostname: "{proxy_hostname}"
  port: {proxy_port}
  tcpRelay:
    enabled: {tcp_relay_enabled}
    hostname: "{tcp_relay_hostname}"
    port: {tcp_relay_port}
  tun: {tun}
  defaultInterface: "{default_interface}"
  skipInstallRootCert: {skip_install_root_cert}
  upstreamProxy: "{upstream_proxy}"

cookie:
  uuid: ""
  password: ""
  key: ""

update:
  sources: []
  proxy: ""
  mirror: ""

channels:
  disableLocationToHome: {disable_location_to_home}
  download:
    defaultHighest: {default_highest}
    frontend: {frontend}
    cover: {cover}
    pauseWhenDownload: {pause_when_download}
    forceCheckAllFeeds: {force_check_all_feeds}
"#,
            debug_error = s.debug_error,
            echolog = s.echolog,
            workdir = s.workdir,
            filename_template = s.filename_template,
            download_dir = s.download_dir,
            play_done_audio = s.play_done_audio,
            default_action = s.default_action,
            max_running = s.max_running,
            resource_concurrency = s.resource_concurrency,
            segment_concurrency = s.segment_concurrency,
            connection_concurrency = s.connection_concurrency,
            remote_enabled = s.remote_enabled,
            remote_protocol = s.remote_protocol,
            remote_hostname = s.remote_hostname,
            remote_port = s.remote_port,
            api_protocol = s.api_protocol,
            api_hostname = s.api_hostname,
            api_port = s.api_port,
            mcp_enabled = s.mcp_enabled,
            scraper_retained_jobs = s.scraper_retained_jobs,
            db_type = s.db_type,
            db_filepath = s.db_filepath,
            proxy_system = s.proxy_system,
            proxy_hostname = s.proxy_hostname,
            proxy_port = s.proxy_port,
            tcp_relay_enabled = s.tcp_relay_enabled,
            tcp_relay_hostname = s.tcp_relay_hostname,
            tcp_relay_port = s.tcp_relay_port,
            tun = s.tun,
            default_interface = s.default_interface,
            skip_install_root_cert = s.skip_install_root_cert,
            upstream_proxy = s.upstream_proxy,
            disable_location_to_home = s.disable_location_to_home,
            default_highest = s.default_highest,
            frontend = s.frontend,
            cover = s.cover,
            pause_when_download = s.pause_when_download,
            force_check_all_feeds = s.force_check_all_feeds,
        )
    }

    /// 将设置写入配置文件（自定义配置文件路径时不覆盖）
    fn write_config(&self, app: &AppHandle) -> Result<PathBuf, String> {
        let path = self.active_config_path(app)?;
        if !self.settings.config_path.is_empty() {
            return Ok(path); // 用户切换了自定义配置文件，原样使用
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(&path, self.build_config_yaml()).map_err(|e| e.to_string())?;
        Ok(path)
    }

    fn is_running(&mut self) -> bool {
        match &mut self.child {
            Some(c) => match c.try_wait() {
                Ok(Some(_)) | Err(_) => {
                    self.child = None;
                    false
                }
                Ok(None) => true,
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
    #[cfg(unix)]
    {
        // 独立进程组，停止时可整体清理
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }

    cmd.spawn()
}

fn sh_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

fn pipe_logs(app: AppHandle, stream: impl std::io::Read + Send + 'static) {
    let reader = BufReader::new(stream);
    std::thread::spawn(move || {
        for line in reader.lines() {
            match line {
                Ok(l) => {
                    let _ = app.emit("service-log", l);
                }
                Err(_) => break,
            }
        }
    });
}

#[cfg(target_os = "macos")]
fn tail_log_file(app: AppHandle, path: PathBuf, pid: u32) {
    std::thread::spawn(move || {
        let mut pos: u64 = 0;
        loop {
            unsafe {
                if libc::kill(pid as i32, 0) != 0 {
                    break;
                }
            }
            if let Ok(mut f) = fs::File::open(&path) {
                if f.seek(SeekFrom::Start(pos)).is_ok() {
                    let mut buf = String::new();
                    if f.read_to_string(&mut buf).is_ok() && !buf.is_empty() {
                        pos += buf.len() as u64;
                        for line in buf.lines() {
                            let _ = app.emit("service-log", line.to_string());
                        }
                    }
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(300));
        }
    });
}

#[cfg(target_os = "macos")]
fn start_admin(exe: &str, cfg: &str, log_path: &PathBuf) -> Result<u32, String> {
    let script = format!(
        "do shell script \"{} --config {} > {} 2>&1 & echo $!\" with administrator privileges with prompt \"wx-dl-ui 需要管理员权限以安装证书并设置系统代理\"",
        sh_quote(exe),
        sh_quote(cfg),
        sh_quote(&log_path.display().to_string()),
    );
    let out = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| format!("osascript 启动失败：{}", e))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    String::from_utf8_lossy(&out.stdout)
        .trim()
        .parse::<u32>()
        .map_err(|_| "无法解析服务进程 PID".into())
}

#[cfg(target_os = "macos")]
fn stop_admin(pid: u32) {
    let script = format!(
        "do shell script \"kill -TERM {}\" with administrator privileges with prompt \"停止服务需要管理员权限\"",
        pid
    );
    let _ = Command::new("osascript").arg("-e").arg(&script).output();
    for _ in 0..30 {
        unsafe {
            if libc::kill(pid as i32, 0) != 0 {
                return;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    let script = format!(
        "do shell script \"kill -9 {}\" with administrator privileges with prompt \"停止服务需要管理员权限\"",
        pid
    );
    let _ = Command::new("osascript").arg("-e").arg(&script).output();
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
fn get_config_path(app: AppHandle, state: State<'_, Mutex<AppData>>) -> String {
    state
        .lock()
        .unwrap()
        .active_config_path(&app)
        .map(|p| p.display().to_string())
        .unwrap_or_default()
}

#[tauri::command]
fn read_config_yaml(app: AppHandle, state: State<'_, Mutex<AppData>>) -> Result<String, String> {
    let data = state.lock().unwrap();
    let path = data.active_config_path(&app)?;
    if !path.is_file() {
        fs::write(&path, data.build_config_yaml()).map_err(|e| e.to_string())?;
    }
    fs::read_to_string(&path).map_err(|e| e.to_string())
}

#[tauri::command]
fn write_config_yaml_raw(
    app: AppHandle,
    state: State<'_, Mutex<AppData>>,
    content: String,
) -> Result<(), String> {
    let data = state.lock().unwrap();
    let path = data.active_config_path(&app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&path, content).map_err(|e| e.to_string())
}

#[tauri::command]
fn open_dir_of(path: String) -> Result<(), String> {
    let p = PathBuf::from(&path);
    let dir = if p.is_dir() { p } else { p.parent().map(|d| d.to_path_buf()).unwrap_or(p) };
    if dir.is_dir() {
        tauri_plugin_opener::open_path(dir.display().to_string(), None::<&str>)
            .map_err(|e| e.to_string())
    } else {
        Err("路径不存在".into())
    }
}

#[tauri::command]
fn start_service(app: AppHandle, state: State<'_, Mutex<AppData>>) -> Result<(), String> {
    let mut data = state.lock().unwrap();
    let running = data.is_running();
    if running {
        return Err("服务已在运行".into());
    }
    let exe = data.settings.exe_path.clone();
    if exe.is_empty() {
        return Err("请先选择 wx_video_download 可执行文件".into());
    }
    let exe_path = PathBuf::from(&exe);
    if !exe_path.is_file() {
        return Err(format!("可执行文件不存在：{}", exe));
    }
    let config_path = data.write_config(&app)?;

    let want_admin = data.settings.admin;
    #[allow(unused_variables)]
    let cfg_str = config_path.display().to_string();

    #[cfg(target_os = "macos")]
    if want_admin {
        let log_path = AppData::config_dir(&app)?.join("service.log");
        let _ = fs::write(&log_path, "");
        let pid = start_admin(&exe, &cfg_str, &log_path)?;
        data.admin_pid = Some(pid);
        tail_log_file(app.clone(), log_path, pid);
        let _ = app.emit(
            "service-log",
            format!(
                "[info] 已以管理员权限启动 {}（PID {}，管理页 http://{}:{}）",
                exe, pid, data.settings.api_hostname, data.settings.api_port
            ),
        );
        return Ok(());
    }
    #[cfg(not(target_os = "macos"))]
    if want_admin {
        return Err("Windows 下请右键以管理员身份运行本软件后再启动服务".into());
    }

    let mut child =
        spawn_child(&exe_path, &config_path).map_err(|e| format!("启动失败：{}", e))?;
    if let Some(stdout) = child.stdout.take() {
        pipe_logs(app.clone(), stdout);
    }
    if let Some(stderr) = child.stderr.take() {
        pipe_logs(app.clone(), stderr);
    }
    let _ = app.emit(
        "service-log",
        format!(
            "[info] 已启动 {}（管理页 http://{}:{}，代理端口 {}）",
            exe, data.settings.api_hostname, data.settings.api_port, data.settings.proxy_port
        ),
    );
    data.child = Some(child);
    Ok(())
}

#[tauri::command]
fn stop_service(state: State<'_, Mutex<AppData>>) -> Result<(), String> {
    let mut data = state.lock().unwrap();

    #[cfg(target_os = "macos")]
    if let Some(pid) = data.admin_pid.take() {
        stop_admin(pid);
        return Ok(());
    }

    if let Some(mut child) = data.child.take() {
        #[cfg(unix)]
        {
            // 先 SIGTERM 优雅退出（让代理正确清理系统代理设置），超时后 SIGKILL
            let pid = child.id() as i32;
            unsafe {
                libc::kill(-pid, libc::SIGTERM);
            }
            for _ in 0..30 {
                match child.try_wait() {
                    Ok(Some(_)) => return Ok(()),
                    Ok(None) => std::thread::sleep(std::time::Duration::from_millis(100)),
                    Err(_) => break,
                }
            }
            unsafe {
                libc::kill(-pid, libc::SIGKILL);
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

    #[cfg(target_os = "macos")]
    if let Some(pid) = data.admin_pid {
        unsafe {
            return libc::kill(pid as i32, 0) == 0;
        }
    }

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
            get_config_path,
            read_config_yaml,
            write_config_yaml_raw,
            open_dir_of,
            start_service,
            stop_service,
            service_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
