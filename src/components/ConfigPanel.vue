<template>
  <div class="config-panel">
    <div class="tabs">
      <button
        v-for="t in tabs"
        :key="t.key"
        class="tab"
        :class="{ active: tab === t.key }"
        @click="tab = t.key"
      >
        {{ t.label }}
      </button>
    </div>

    <!-- 基础 -->
    <div v-if="tab === 'base'" class="tab-body">
      <div class="grid-2">
        <label class="switch-row">
          <input type="checkbox" v-model="s.debugError" />
          <span>错误日志 <code>debug.error</code></span>
        </label>
        <label class="switch-row">
          <input type="checkbox" v-model="s.echolog" />
          <span>回显日志 <code>debug.echolog</code></span>
        </label>
      </div>
      <div>
        <label class="field-label">运行时工作目录 <code>workdir</code></label>
        <div class="row">
          <input type="text" v-model="s.workdir" placeholder="留空使用默认" />
          <button class="btn btn-ghost" @click="browseDir('workdir')">浏览...</button>
          <button class="btn btn-ghost" @click="openDir(s.workdir)" :disabled="!s.workdir">打开</button>
        </div>
      </div>
      <div class="grid-2">
        <div>
          <label class="field-label">API 协议</label>
          <select v-model="s.apiProtocol">
            <option value="http">http</option>
            <option value="https">https</option>
          </select>
        </div>
        <div>
          <label class="field-label">API 监听地址</label>
          <input type="text" v-model="s.apiHostname" />
        </div>
      </div>
      <div>
        <label class="field-label">API 端口</label>
        <input type="number" v-model.number="s.apiPort" min="1" max="65535" />
      </div>
      <div class="grid-2">
        <label class="switch-row">
          <input type="checkbox" v-model="s.mcpEnabled" />
          <span>启用 MCP 服务 <code>mcp.enabled</code></span>
        </label>
        <div>
          <label class="field-label">抓取任务保留数 <code>scraper.retainedJobs</code></label>
          <input type="number" v-model.number="s.scraperRetainedJobs" min="1" />
        </div>
      </div>
    </div>

    <!-- 下载 -->
    <div v-else-if="tab === 'download'" class="tab-body">
      <div>
        <label class="field-label">文件名模板</label>
        <input type="text" v-model="s.filenameTemplate" />
      </div>
      <div>
        <label class="field-label">下载保存目录</label>
        <div class="row">
          <input type="text" v-model="s.downloadDir" />
          <button class="btn btn-ghost" @click="browseDir('downloadDir')">浏览...</button>
          <button class="btn btn-ghost" @click="openDir(s.downloadDir)" :disabled="!s.downloadDir">打开</button>
          <button class="btn btn-ghost" @click="s.downloadDir = '%UserDownloads%'">默认</button>
        </div>
      </div>
      <label class="switch-row">
        <input type="checkbox" v-model="s.playDoneAudio" />
        <span>下载完成播放提示音</span>
      </label>
      <div class="grid-2">
        <div>
          <label class="field-label">文件已存在时</label>
          <select v-model="s.defaultAction">
            <option value="">（留空 / 询问）</option>
            <option value="overwrite">覆盖</option>
            <option value="skip">跳过</option>
            <option value="rename">重命名</option>
          </select>
        </div>
        <div>
          <label class="field-label">最大并行任务数</label>
          <input type="number" v-model.number="s.maxRunning" min="1" />
        </div>
      </div>
      <div class="grid-2">
        <div>
          <label class="field-label">资源并发数</label>
          <input type="number" v-model.number="s.resourceConcurrency" min="1" />
        </div>
        <div>
          <label class="field-label">分片并发数</label>
          <input type="number" v-model.number="s.segmentConcurrency" min="1" />
        </div>
      </div>
      <div>
        <label class="field-label">连接并发数 (0=自动)</label>
        <input type="number" v-model.number="s.connectionConcurrency" min="0" />
      </div>
      <div class="divider"></div>
      <label class="switch-row">
        <input type="checkbox" v-model="s.remoteEnabled" />
        <span>启用远程下载服务 <code>download.remoteServer</code></span>
      </label>
      <template v-if="s.remoteEnabled">
        <div class="grid-2">
          <div>
            <label class="field-label">远程服务协议</label>
            <select v-model="s.remoteProtocol">
              <option value="http">http</option>
              <option value="https">https</option>
            </select>
          </div>
          <div>
            <label class="field-label">远程服务地址</label>
            <input type="text" v-model="s.remoteHostname" placeholder="192.168.1.2" />
          </div>
        </div>
        <div>
          <label class="field-label">远程服务端口</label>
          <input type="number" v-model.number="s.remotePort" min="1" max="65535" />
        </div>
      </template>
    </div>

    <!-- 代理 -->
    <div v-else-if="tab === 'proxy'" class="tab-body">
      <label class="switch-row">
        <input type="checkbox" v-model="s.proxySystem" />
        <span>设置系统代理 <code>proxy.system</code>（需管理员权限）</span>
      </label>
      <div class="grid-2">
        <div>
          <label class="field-label">代理监听地址</label>
          <input type="text" v-model="s.proxyHostname" />
        </div>
        <div>
          <label class="field-label">代理端口</label>
          <input type="number" v-model.number="s.proxyPort" min="1" max="65535" />
        </div>
      </div>
      <div class="divider"></div>
      <label class="switch-row">
        <input type="checkbox" v-model="s.tcpRelayEnabled" />
        <span>启用 TCP 中继 <code>proxy.tcpRelay</code></span>
      </label>
      <template v-if="s.tcpRelayEnabled">
        <div class="grid-2">
          <div>
            <label class="field-label">TCP 中继地址</label>
            <input type="text" v-model="s.tcpRelayHostname" />
          </div>
          <div>
            <label class="field-label">TCP 中继端口</label>
            <input type="number" v-model.number="s.tcpRelayPort" min="1" max="65535" />
          </div>
        </div>
      </template>
      <div class="grid-2">
        <label class="switch-row">
          <input type="checkbox" v-model="s.tun" />
          <span>TUN 模式 <code>proxy.tun</code></span>
        </label>
        <div>
          <label class="field-label">默认网络接口</label>
          <input type="text" v-model="s.defaultInterface" placeholder="留空自动" />
        </div>
      </div>
      <label class="switch-row">
        <input type="checkbox" v-model="s.skipInstallRootCert" />
        <span>跳过安装根证书 <code>proxy.skipInstallRootCert</code></span>
      </label>
      <div>
        <label class="field-label">上游代理 <code>proxy.upstreamProxy</code></label>
        <input type="text" v-model="s.upstreamProxy" placeholder="留空不使用，例如 http://127.0.0.1:7890" />
      </div>
    </div>

    <!-- 视频号 -->
    <div v-else-if="tab === 'channels'" class="tab-body">
      <label class="switch-row">
        <input type="checkbox" v-model="s.disableLocationToHome" />
        <span>禁用定位到首页 <code>channels.disableLocationToHome</code></span>
      </label>
      <label class="switch-row">
        <input type="checkbox" v-model="s.defaultHighest" />
        <span>默认下载最高清晰度 <code>channels.download.defaultHighest</code></span>
      </label>
      <label class="switch-row">
        <input type="checkbox" v-model="s.frontend" />
        <span>前端下载模式 <code>channels.download.frontend</code></span>
      </label>
      <label class="switch-row">
        <input type="checkbox" v-model="s.cover" />
        <span>同时下载封面 <code>channels.download.cover</code></span>
      </label>
      <label class="switch-row">
        <input type="checkbox" v-model="s.pauseWhenDownload" />
        <span>下载时暂停其他任务 <code>channels.download.pauseWhenDownload</code></span>
      </label>
      <label class="switch-row">
        <input type="checkbox" v-model="s.forceCheckAllFeeds" />
        <span>强制检查全部订阅 <code>channels.download.forceCheckAllFeeds</code></span>
      </label>
      <div class="divider"></div>
      <div class="grid-2">
        <div>
          <label class="field-label">数据库类型 <code>db.type</code></label>
          <select v-model="s.dbType">
            <option value="sqlite">sqlite</option>
            <option value="mysql">mysql</option>
            <option value="postgres">postgres</option>
          </select>
        </div>
      </div>
      <div>
        <label class="field-label">数据库文件路径 <code>db.filepath</code></label>
        <div class="row">
          <input type="text" v-model="s.dbFilepath" />
          <button class="btn btn-ghost" @click="saveDbTo">保存到...</button>
          <button class="btn btn-ghost" @click="openDir(s.dbFilepath)" :disabled="!s.dbFilepath">打开</button>
        </div>
      </div>
    </div>

    <!-- YAML 原文 -->
    <div v-else class="tab-body yaml-body">
      <div class="yaml-head">
        <span class="field-label" style="margin: 0">{{ configPath || "配置文件" }}</span>
        <button class="btn btn-ghost" @click="$emit('save-yaml')">保存 YAML</button>
        <button class="btn btn-ghost" @click="$emit('reset-yaml')">按设置重新生成</button>
      </div>
      <textarea class="yaml-editor" v-model="yamlText" spellcheck="false"></textarea>
      <p class="hint">直接编辑 YAML 原文并保存，可配置此界面未覆盖的所有选项（cookie、cloudflare、bridge 等）。「按设置重新生成」会用界面配置覆盖此文件。</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { open, save } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import type { Settings } from "../types";

const props = defineProps<{
  s: Settings;
  configPath: string;
}>();

defineEmits(["save-yaml", "reset-yaml"]);

const yamlText = defineModel<string>("yamlText", { required: true });

const tab = ref<"base" | "download" | "proxy" | "channels" | "yaml">("base");
const tabs = [
  { key: "base", label: "基础" },
  { key: "download", label: "下载" },
  { key: "proxy", label: "代理" },
  { key: "channels", label: "视频号" },
  { key: "yaml", label: "YAML 原文" },
] as const;

const s = props.s;

async function browseDir(field: "workdir" | "downloadDir") {
  const path = await open({ multiple: false, directory: true, title: "选择目录" });
  if (typeof path === "string") {
    s[field] = path;
  }
}

async function openDir(path: string) {
  if (!path) return;
  await invoke("open_dir_of", { path });
}

async function saveDbTo() {
  const path = await save({ title: "数据库文件保存到", defaultPath: "data.db" });
  if (typeof path === "string") {
    s.dbFilepath = path;
  }
}
</script>

<style scoped>
.config-panel {
  display: flex;
  flex-direction: column;
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  overflow: hidden;
  flex: 1;
  min-height: 0;
}
.tabs {
  display: flex;
  gap: 2px;
  padding: 8px 10px 0;
  border-bottom: 1px solid var(--border);
  background: rgba(255, 255, 255, 0.02);
}
.tab {
  padding: 8px 16px;
  font-size: 13px;
  background: transparent;
  color: var(--text-dim);
  border-bottom: 2px solid transparent;
  transition: all 0.15s;
}
.tab:hover {
  color: var(--text);
}
.tab.active {
  color: var(--accent);
  border-bottom-color: var(--accent);
  font-weight: 600;
}
.tab-body {
  padding: 16px 18px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 14px;
  flex: 1;
  min-height: 0;
}
.grid-2 {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
}
.row {
  display: flex;
  gap: 8px;
}
.row input {
  flex: 1;
}
.row .btn {
  flex-shrink: 0;
}
.divider {
  height: 1px;
  background: var(--border);
  margin: 2px 0;
}
code {
  font-size: 11px;
  color: var(--text-dim);
  background: var(--bg-soft);
  padding: 1px 5px;
  border-radius: 4px;
}
select {
  width: 100%;
  background: var(--bg-soft);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 8px 12px;
  color: var(--text);
  font-size: 13px;
  outline: none;
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
.yaml-body {
  gap: 10px;
}
.yaml-head {
  display: flex;
  align-items: center;
  gap: 10px;
}
.yaml-editor {
  flex: 1;
  min-height: 260px;
  background: var(--bg-soft);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 12px;
  color: #c4cbd9;
  font-family: "SF Mono", Menlo, Consolas, monospace;
  font-size: 12px;
  line-height: 1.6;
  outline: none;
  resize: none;
  user-select: text;
}
.yaml-editor:focus {
  border-color: var(--accent);
}
.hint {
  font-size: 12px;
  color: var(--text-dim);
}
</style>
