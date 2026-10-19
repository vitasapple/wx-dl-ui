<template>
  <div class="layout">
    <aside class="sidebar">
      <div class="brand">
        <div class="brand-icon">
          <Download :size="20" :stroke-width="2.2" />
        </div>
        <div class="brand-text">
          <div class="brand-name">视频号下载器</div>
          <div class="brand-sub">wx-dl-ui</div>
        </div>
      </div>
      <nav class="nav">
        <button
          class="nav-item"
          :class="{ active: tab === 'home' }"
          @click="tab = 'home'"
        >
          <MonitorPlay :size="17" />
          <span>控制台</span>
        </button>
        <button
          class="nav-item"
          :class="{ active: tab === 'settings' }"
          @click="tab = 'settings'"
        >
          <Settings :size="17" />
          <span>设置</span>
        </button>
      </nav>
      <div class="sidebar-foot">
        <div class="foot-dot" :class="statusClass"></div>
        <span>{{ running ? "服务运行中" : "服务未运行" }}</span>
      </div>
    </aside>

    <main class="main">
      <!-- 控制台 -->
      <section v-if="tab === 'home'" class="view">
        <div class="status-card">
          <div class="status-left">
            <div class="status-ring" :class="statusClass">
              <Power :size="26" :stroke-width="1.8" />
            </div>
            <div class="status-info">
              <div class="status-title">
                {{ running ? "服务运行中" : "服务已停止" }}
              </div>
              <div class="status-desc">
                {{
                  running
                    ? `管理页地址 http://127.0.0.1:${settings.apiPort}`
                    : "选择好可执行文件后，点击启动即可开始服务"
                }}
              </div>
            </div>
          </div>
          <div class="status-actions">
            <button
              v-if="!running"
              class="btn btn-primary"
              :disabled="starting || !settings.exePath"
              @click="start"
            >
              <Play :size="16" />
              {{ starting ? "启动中..." : "启动服务" }}
            </button>
            <button v-else class="btn btn-danger" @click="stop">
              <Square :size="16" />
              停止服务
            </button>
            <button class="btn btn-ghost" :disabled="!running" @click="openAdmin">
              <ExternalLink :size="15" />
              打开管理页
            </button>
          </div>
        </div>

        <div class="exe-line">
          <FolderOpen :size="15" class="dim" />
          <span class="dim">核心程序：</span>
          <span class="exe-path">{{ settings.exePath || "尚未选择 wx_video_download 可执行文件" }}</span>
          <button class="btn btn-ghost btn-mini" @click="goSettings">去选择</button>
        </div>

        <div class="log-card">
          <div class="log-head">
            <Terminal :size="15" />
            <span>运行日志</span>
            <button class="btn btn-ghost btn-mini log-clear" @click="logs = []">
              清空
            </button>
          </div>
          <div class="log-body" ref="logBox">
            <div v-if="logs.length === 0" class="log-empty">
              暂无日志，启动服务后将在此显示输出
            </div>
            <div v-for="(l, i) in logs" :key="i" class="log-line">{{ l }}</div>
          </div>
        </div>
      </section>

      <!-- 设置 -->
      <section v-else class="view">
        <div class="panel">
          <div class="panel-title">核心程序</div>
          <div class="panel-body">
            <label class="field-label">wx_video_download 可执行文件路径（外置，不随软件打包，可随时替换新版本）</label>
            <div class="row">
              <input type="text" v-model="settings.exePath" placeholder="例如 /Users/xxx/Downloads/wx_video_download" readonly />
              <button class="btn btn-ghost" @click="pickExe">
                <FolderOpen :size="15" />
                选择文件
              </button>
            </div>
            <p class="hint">首次使用请先到 ltaoo/wx_channels_download 的 Releases 下载对应平台的可执行文件。</p>
          </div>
        </div>

        <div class="panel">
          <div class="panel-title">下载设置</div>
          <div class="panel-body grid-2">
            <div>
              <label class="field-label">下载目录</label>
              <input type="text" v-model="settings.downloadDir" placeholder="%UserDownloads%" />
            </div>
            <div>
              <label class="field-label">同时下载任务数上限</label>
              <input type="number" v-model.number="settings.maxRunning" min="1" max="10" />
            </div>
          </div>
        </div>

        <div class="panel">
          <div class="panel-title">网络设置</div>
          <div class="panel-body grid-2">
            <div>
              <label class="field-label">API 服务端口</label>
              <input type="number" v-model.number="settings.apiPort" min="1" max="65535" />
            </div>
            <div>
              <label class="field-label">代理端口</label>
              <input type="number" v-model.number="settings.proxyPort" min="1" max="65535" />
            </div>
          </div>
        </div>

        <div class="panel">
          <div class="panel-title">其他</div>
          <div class="panel-body">
            <label class="switch-row">
              <input type="checkbox" v-model="settings.proxySystem" />
              <span>启动时设置系统代理（需要管理员权限，首次使用会安装证书）</span>
            </label>
            <label class="switch-row">
              <input type="checkbox" v-model="settings.playDoneAudio" />
              <span>下载完成时播放提示音</span>
            </label>
            <label class="switch-row">
              <input type="checkbox" v-model="settings.echolog" />
              <span>显示调试日志</span>
            </label>
          </div>
        </div>

        <div class="save-bar">
          <button class="btn btn-primary" :disabled="saving" @click="save">
            <Save :size="15" />
            {{ saving ? "保存中..." : "保存设置" }}
          </button>
          <span class="hint">{{ saveMsg }}</span>
        </div>
      </section>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onBeforeUnmount, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
  Download, MonitorPlay, Settings, Power, Play, Square,
  ExternalLink, FolderOpen, Terminal, Save,
} from "lucide-vue-next";

interface Settings {
  exePath: string;
  downloadDir: string;
  maxRunning: number;
  apiPort: number;
  proxyPort: number;
  proxySystem: boolean;
  playDoneAudio: boolean;
  echolog: boolean;
}

const tab = ref<"home" | "settings">("home");
const running = ref(false);
const starting = ref(false);
const saving = ref(false);
const saveMsg = ref("");
const logs = ref<string[]>([]);
const logBox = ref<HTMLElement | null>(null);

const settings = reactive<Settings>({
  exePath: "",
  downloadDir: "%UserDownloads%",
  maxRunning: 3,
  apiPort: 2022,
  proxyPort: 2023,
  proxySystem: true,
  playDoneAudio: true,
  echolog: false,
});

const statusClass = computed(() => (running.value ? "on" : "off"));

let unlisten: (() => void) | null = null;
let pollTimer: number | null = null;

async function refreshStatus() {
  try {
    running.value = await invoke<boolean>("service_status");
  } catch {
    running.value = false;
  }
}

async function loadSettings() {
  try {
    const s = await invoke<Partial<Settings>>("get_settings");
    Object.assign(settings, s);
  } catch {
    /* 首次使用无配置 */
  }
}

async function pickExe() {
  const path = await open({
    multiple: false,
    directory: false,
    title: "选择 wx_video_download 可执行文件",
  });
  if (typeof path === "string") {
    settings.exePath = path;
    await save();
  }
}

async function save() {
  saving.value = true;
  saveMsg.value = "";
  try {
    await invoke("save_settings", { settings: { ...settings } });
    saveMsg.value = "已保存";
  } catch (e) {
    saveMsg.value = `保存失败：${e}`;
  } finally {
    saving.value = false;
    setTimeout(() => (saveMsg.value = ""), 2500);
  }
}

async function start() {
  starting.value = true;
  try {
    await invoke("start_service");
  } catch (e) {
    logs.value.push(`[启动失败] ${e}`);
  } finally {
    starting.value = false;
    refreshStatus();
  }
}

async function stop() {
  try {
    await invoke("stop_service");
  } catch (e) {
    logs.value.push(`[停止失败] ${e}`);
  }
  refreshStatus();
}

function openAdmin() {
  openUrl(`http://127.0.0.1:${settings.apiPort}`);
}

function goSettings() {
  tab.value = "settings";
}

onMounted(async () => {
  await loadSettings();
  await refreshStatus();
  unlisten = await listen<string>("service-log", (e) => {
    logs.value.push(e.payload);
    if (logs.value.length > 500) logs.value.splice(0, logs.value.length - 500);
    nextTick(() => {
      if (logBox.value) logBox.value.scrollTop = logBox.value.scrollHeight;
    });
  });
  pollTimer = window.setInterval(refreshStatus, 2000);
});

onBeforeUnmount(() => {
  unlisten?.();
  if (pollTimer) clearInterval(pollTimer);
});
</script>

<style scoped>
.layout {
  display: flex;
  height: 100%;
}

.sidebar {
  width: 200px;
  flex-shrink: 0;
  background: var(--bg-soft);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  padding: 18px 12px;
}
.brand {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 4px 8px 18px;
}
.brand-icon {
  width: 38px;
  height: 38px;
  border-radius: 10px;
  background: linear-gradient(135deg, #4f8cff, #22d3a5);
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
}
.brand-name {
  font-size: 15px;
  font-weight: 600;
}
.brand-sub {
  font-size: 11px;
  color: var(--text-dim);
}
.nav {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 10px;
  background: transparent;
  color: var(--text-dim);
  font-size: 14px;
  transition: all 0.15s;
}
.nav-item:hover {
  color: var(--text);
  background: var(--panel-2);
}
.nav-item.active {
  color: #fff;
  background: linear-gradient(135deg, rgba(79, 140, 255, 0.25), rgba(106, 92, 255, 0.2));
}
.sidebar-foot {
  margin-top: auto;
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: var(--text-dim);
  padding: 8px;
}
.foot-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}
.foot-dot.on {
  background: var(--accent-2);
  box-shadow: 0 0 6px var(--accent-2);
}
.foot-dot.off {
  background: #4b5468;
}

.main {
  flex: 1;
  overflow-y: auto;
  padding: 22px 26px;
}
.view {
  display: flex;
  flex-direction: column;
  gap: 16px;
  max-width: 860px;
}

.status-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  background: linear-gradient(135deg, var(--panel), var(--panel-2));
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 20px 22px;
}
.status-left {
  display: flex;
  align-items: center;
  gap: 16px;
}
.status-ring {
  width: 58px;
  height: 58px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.status-ring.on {
  color: var(--accent-2);
  background: rgba(34, 211, 165, 0.12);
  border: 1.5px solid rgba(34, 211, 165, 0.5);
  animation: pulse 2s infinite;
}
.status-ring.off {
  color: #5a6378;
  background: var(--bg-soft);
  border: 1.5px solid var(--border);
}
@keyframes pulse {
  0%, 100% { box-shadow: 0 0 0 0 rgba(34, 211, 165, 0.25); }
  50% { box-shadow: 0 0 0 10px rgba(34, 211, 165, 0); }
}
.status-title {
  font-size: 17px;
  font-weight: 600;
}
.status-desc {
  font-size: 12px;
  color: var(--text-dim);
  margin-top: 4px;
}
.status-actions {
  display: flex;
  gap: 10px;
}

.exe-line {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  padding: 0 4px;
}
.dim {
  color: var(--text-dim);
}
.exe-path {
  color: var(--text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}
.btn-mini {
  padding: 4px 10px;
  font-size: 12px;
  border-radius: 8px;
}

.log-card {
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  overflow: hidden;
  display: flex;
  flex-direction: column;
  min-height: 320px;
  flex: 1;
}
.log-head {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  border-bottom: 1px solid var(--border);
  font-size: 13px;
  color: var(--text-dim);
}
.log-clear {
  margin-left: auto;
}
.log-body {
  flex: 1;
  overflow-y: auto;
  padding: 12px 14px;
  font-family: "SF Mono", Menlo, Consolas, monospace;
  font-size: 12px;
  line-height: 1.7;
  user-select: text;
}
.log-empty {
  color: #5a6378;
  text-align: center;
  padding-top: 110px;
  font-family: inherit;
}
.log-line {
  white-space: pre-wrap;
  word-break: break-all;
  color: #c4cbd9;
}

.panel {
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  overflow: hidden;
}
.panel-title {
  padding: 12px 18px;
  font-size: 14px;
  font-weight: 600;
  border-bottom: 1px solid var(--border);
  background: rgba(255, 255, 255, 0.02);
}
.panel-body {
  padding: 16px 18px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.grid-2 {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
}
.row {
  display: flex;
  gap: 10px;
}
.hint {
  font-size: 12px;
  color: var(--text-dim);
}
.switch-row {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 13px;
  cursor: pointer;
}
.switch-row input[type="checkbox"] {
  width: 16px;
  height: 16px;
  accent-color: var(--accent);
  cursor: pointer;
}
.save-bar {
  display: flex;
  align-items: center;
  gap: 14px;
}
</style>
