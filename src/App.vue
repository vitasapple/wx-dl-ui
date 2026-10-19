<template>
  <div class="layout">
    <header class="topbar">
      <div class="brand">
        <div class="brand-icon">
          <Download :size="20" :stroke-width="2.2" />
        </div>
        <div class="brand-text">
          <div class="brand-name">微下载</div>
          <div class="brand-sub">wx-dl-ui · Tauri 版</div>
        </div>
      </div>
      <div class="topbar-right">
        <div class="status-pill">
          <span class="dot" :class="running ? 'on' : 'off'"></span>
          <span>{{ starting ? "启动中..." : running ? "运行中" : "已停止" }}</span>
        </div>
        <label class="switch-row">
          <input type="checkbox" v-model="settings.admin" />
          <span>管理员权限</span>
        </label>
        <button class="btn btn-ghost" :disabled="saving || running" @click="save">
          {{ saving ? "保存中..." : "保存配置" }}
        </button>
        <button
          v-if="!running"
          class="btn btn-primary"
          :disabled="starting || !settings.exePath"
          @click="start"
        >
          <Play :size="15" />
          启动 dl
        </button>
        <button v-else class="btn btn-danger" @click="stop">
          <Square :size="14" />
          停止
        </button>
      </div>
    </header>

    <div class="path-area">
      <div class="path-row">
        <span class="path-label">主程序</span>
        <span class="path-value" :title="settings.exePath">
          {{ settings.exePath || "尚未选择 wx_video_download 可执行文件" }}
        </span>
        <button class="btn btn-ghost btn-mini" @click="pickExe">选择...</button>
        <button class="btn btn-ghost btn-mini" @click="openDir(settings.exePath)" :disabled="!settings.exePath">打开目录</button>
      </div>
      <div class="path-row">
        <span class="path-label">配置文件</span>
        <span class="path-value" :title="configPath">{{ configPath || "..." }}</span>
        <button class="btn btn-ghost btn-mini" @click="pickConfig">切换...</button>
        <button class="btn btn-ghost btn-mini" @click="openDir(configPath)" :disabled="!configPath">打开目录</button>
      </div>
    </div>

    <div class="content">
      <ConfigPanel
        :s="settings"
        v-model:yaml-text="yamlText"
        :configPath="configPath"
        @save-yaml="saveYaml"
        @reset-yaml="resetYaml"
      />
      <div class="log-card">
        <div class="log-head">
          <Terminal :size="14" />
          <span>运行日志</span>
          <button class="btn btn-ghost btn-mini log-clear" @click="logs = []">清空</button>
        </div>
        <div class="log-body" ref="logBox">
          <div v-if="logs.length === 0" class="log-empty">
            -- 暂无输出，点击「启动 dl」开始 --
          </div>
          <div v-for="(l, i) in logs" :key="i" class="log-line">{{ l }}</div>
        </div>
      </div>
    </div>

    <transition name="fade">
      <div v-if="toast" class="toast" :class="toastType">{{ toast }}</div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, onBeforeUnmount, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { Download, Play, Square, Terminal } from "lucide-vue-next";
import ConfigPanel from "./components/ConfigPanel.vue";
import { defaultSettings, type Settings } from "./types";

const running = ref(false);
const starting = ref(false);
const saving = ref(false);
const toast = ref("");
const toastType = ref<"ok" | "err">("ok");
const logs = ref<string[]>([]);
const logBox = ref<HTMLElement | null>(null);
const yamlText = ref("");
const configPath = ref("");

const settings = reactive<Settings>({ ...defaultSettings });

let unlisten: (() => void) | null = null;
let pollTimer: number | null = null;

function showToast(msg: string, type: "ok" | "err" = "ok") {
  toast.value = msg;
  toastType.value = type;
  setTimeout(() => (toast.value = ""), 2500);
}

async function refreshStatus() {
  try {
    running.value = await invoke<boolean>("service_status");
  } catch {
    running.value = false;
  }
}

async function loadAll() {
  try {
    const s = await invoke<Partial<Settings>>("get_settings");
    Object.assign(settings, s);
  } catch {
    /* 首次使用无配置 */
  }
  try {
    configPath.value = await invoke<string>("get_config_path");
    yamlText.value = await invoke<string>("read_config_yaml");
  } catch {
    /* 忽略 */
  }
}

async function refreshYaml() {
  try {
    configPath.value = await invoke<string>("get_config_path");
    yamlText.value = await invoke<string>("read_config_yaml");
  } catch (e) {
    showToast(`读取配置失败：${e}`, "err");
  }
}

async function save() {
  saving.value = true;
  try {
    await invoke("save_settings", { settings: { ...settings } });
    showToast("配置已保存");
    await refreshYaml();
  } catch (e) {
    showToast(`保存失败：${e}`, "err");
  } finally {
    saving.value = false;
  }
}

async function saveYaml() {
  try {
    await invoke("write_config_yaml_raw", { content: yamlText.value });
    showToast("YAML 已保存");
  } catch (e) {
    showToast(`保存失败：${e}`, "err");
  }
}

async function resetYaml() {
  await save();
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

async function pickConfig() {
  const path = await open({
    multiple: false,
    directory: false,
    title: "选择 config.yaml 配置文件",
    filters: [{ name: "YAML", extensions: ["yaml", "yml"] }],
  });
  if (typeof path === "string") {
    settings.configPath = path;
    await save();
    await refreshYaml();
  }
}

async function openDir(path: string) {
  if (!path) return;
  try {
    await invoke("open_dir_of", { path });
  } catch (e) {
    showToast(`${e}`, "err");
  }
}

async function start() {
  starting.value = true;
  try {
    await invoke("start_service");
    showToast("服务启动中...");
  } catch (e) {
    logs.value.push(`[启动失败] ${e}`);
    showToast(`${e}`, "err");
  } finally {
    starting.value = false;
    refreshStatus();
  }
}

async function stop() {
  try {
    await invoke("stop_service");
    showToast("服务已停止");
  } catch (e) {
    logs.value.push(`[停止失败] ${e}`);
    showToast(`${e}`, "err");
  }
  refreshStatus();
}

onMounted(async () => {
  await loadAll();
  await refreshStatus();
  unlisten = await listen<string>("service-log", (e) => {
    logs.value.push(e.payload);
    if (logs.value.length > 800) logs.value.splice(0, logs.value.length - 800);
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
  flex-direction: column;
  height: 100%;
}

.topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  padding: 14px 20px;
  border-bottom: 1px solid var(--border);
  background: linear-gradient(135deg, var(--bg-soft), var(--panel));
  flex-shrink: 0;
}
.brand {
  display: flex;
  align-items: center;
  gap: 11px;
}
.brand-icon {
  width: 38px;
  height: 38px;
  min-width: 38px;
  border-radius: 10px;
  background: linear-gradient(135deg, #4f8cff, #22d3a5);
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  overflow: hidden;
}
.brand-name {
  font-size: 15px;
  font-weight: 600;
}
.brand-sub {
  font-size: 11px;
  color: var(--text-dim);
}
.topbar-right {
  display: flex;
  align-items: center;
  gap: 12px;
}
.status-pill {
  display: flex;
  align-items: center;
  gap: 7px;
  font-size: 12px;
  color: var(--text-dim);
}
.dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}
.dot.on {
  background: var(--accent-2);
  box-shadow: 0 0 6px var(--accent-2);
}
.dot.off {
  background: #4b5468;
}
.switch-row {
  display: flex;
  align-items: center;
  gap: 7px;
  font-size: 13px;
  cursor: pointer;
}
.switch-row input[type="checkbox"] {
  width: 15px;
  height: 15px;
  accent-color: var(--accent);
  cursor: pointer;
}

.path-area {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 12px 20px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}
.path-row {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 12px;
}
.path-label {
  color: var(--text-dim);
  min-width: 56px;
  flex-shrink: 0;
}
.path-value {
  flex: 1;
  color: var(--text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  user-select: text;
}
.btn-mini {
  padding: 4px 10px;
  font-size: 12px;
  border-radius: 8px;
}

.content {
  flex: 1;
  display: flex;
  gap: 14px;
  padding: 14px 20px;
  min-height: 0;
}

.log-card {
  width: 40%;
  min-width: 320px;
  flex-shrink: 0;
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  overflow: hidden;
  display: flex;
  flex-direction: column;
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
  padding-top: 40px;
  font-family: inherit;
}
.log-line {
  white-space: pre-wrap;
  word-break: break-all;
  color: #c4cbd9;
}

.toast {
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  padding: 10px 20px;
  border-radius: 10px;
  font-size: 13px;
  color: #fff;
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.4);
  z-index: 100;
}
.toast.ok {
  background: #2a9d6f;
}
.toast.err {
  background: #c74a55;
}
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
