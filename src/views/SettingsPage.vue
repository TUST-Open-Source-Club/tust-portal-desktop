<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { loadCredentials, saveCredentials } from "@/native/store";
import { refreshStatus, setAutoLoginPaused, setIgnoreSsid } from "@/native/network_state";
import { tryLogin } from "@/native/login";
import { getPlatform } from "@/native/platform";
import { getAutoStartEnabled, setAutoStartEnabled } from "@/native/auto_start";
import type { NetworkStatus, LoginResult } from "@/native/types";

const username = ref("");
const password = ref("");
const networkType = ref("校园网");
const saved = ref(false);
const paused = ref(false);
const ignoreSsid = ref(false);
const loading = ref(false);
const networkStatus = ref<NetworkStatus | null>(null);
const loginResult = ref<LoginResult | null>(null);
const platform = ref("");
const autoStartEnabled = ref(false);
const showFirstRunModal = ref(false);

const activeTab = ref<"account" | "connection">("account");

let pollTimer: ReturnType<typeof setInterval> | null = null;

async function refreshStatusLoop() {
  const status = await refreshStatus();
  paused.value = status.paused;
  ignoreSsid.value = status.ignoreSsid;
  networkStatus.value = status.networkStatus;
}

async function initCredentials() {
  const creds = await loadCredentials();
  if (creds) {
    username.value = creds.username;
    password.value = creds.password;
    networkType.value = creds.network_type || "校园网";
    saved.value = true;
  }
}

async function handleSave() {
  await saveCredentials(username.value, password.value, networkType.value);
  saved.value = true;
}

async function handleLogin() {
  if (!username.value || !password.value) {
    loginResult.value = {
      success: false,
      message: "请先输入用户名和密码",
    };
    return;
  }
  loading.value = true;
  loginResult.value = null;
  try {
    loginResult.value = await tryLogin(
      username.value,
      password.value,
      networkType.value,
    );
  } catch (e: any) {
    loginResult.value = {
      success: false,
      message: typeof e === "string" ? e : "登录请求异常",
    };
  } finally {
    loading.value = false;
  }
}

async function togglePause() {
  paused.value = !paused.value;
  await setAutoLoginPaused(paused.value);
}

async function toggleIgnoreSsid() {
  ignoreSsid.value = !ignoreSsid.value;
  await setIgnoreSsid(ignoreSsid.value);
}

async function handleAutoStartToggle() {
  autoStartEnabled.value = !autoStartEnabled.value;
  try {
    await setAutoStartEnabled(autoStartEnabled.value);
  } catch (e: any) {
    autoStartEnabled.value = !autoStartEnabled.value;
    alert(typeof e === "string" ? e : "设置开机自启失败");
  }
}

async function dismissFirstRun(enable: boolean) {
  showFirstRunModal.value = false;
  autoStartEnabled.value = enable;
  try {
    await setAutoStartEnabled(enable);
  } catch (e) {
    // ignore
  }
}

onMounted(async () => {
  await initCredentials();
  await refreshStatusLoop();

  platform.value = await getPlatform();
  if (platform.value === "windows") {
    try {
      autoStartEnabled.value = await getAutoStartEnabled();
    } catch (e) {
      autoStartEnabled.value = false;
    }
    if (!saved.value) {
      showFirstRunModal.value = true;
    }
  }

  pollTimer = setInterval(async () => {
    await refreshStatusLoop();
  }, 3000);
});

onUnmounted(() => {
  if (pollTimer) clearInterval(pollTimer);
});
</script>

<template>
  <div class="app">
    <!-- First Run Modal -->
    <div v-if="showFirstRunModal" class="modal-overlay">
      <div class="modal">
        <h2 class="modal-title">欢迎使用 TustPortal</h2>
        <p class="modal-body">是否开启开机自动启动？</p>
        <div class="modal-actions">
          <button class="btn primary" @click="dismissFirstRun(true)">是</button>
          <button class="btn" @click="dismissFirstRun(false)">否</button>
        </div>
      </div>
    </div>

    <h1 class="title">天科大校园网自动登录</h1>

    <!-- Network Status Bar -->
    <div class="status-bar" v-if="networkStatus">
      <span class="status-dot" :class="{ online: networkStatus.is_tust_network }"></span>
      <span v-if="networkStatus.wifi_ssid">{{ networkStatus.wifi_ssid }}</span>
      <span v-else class="dim">未检测到WiFi</span>
      <span class="sep">|</span>
      <span v-if="networkStatus.local_ipv4">{{ networkStatus.local_ipv4 }}</span>
      <span v-else class="dim">无IP</span>
      <span v-if="networkStatus.is_tust_network" class="tag tust">校园网</span>
    </div>

    <!-- Tab Bar -->
    <div class="tab-bar">
      <button
        class="tab-btn"
        :class="{ active: activeTab === 'account' }"
        @click="activeTab = 'account'"
      >
        账号
      </button>
      <button
        class="tab-btn"
        :class="{ active: activeTab === 'connection' }"
        @click="activeTab = 'connection'"
      >
        连接
      </button>
    </div>

    <!-- Tab: Account -->
    <div v-if="activeTab === 'account'" class="section">
      <label class="label">用户名</label>
      <input
        v-model="username"
        type="text"
        class="input"
        placeholder="学号/工号"
        autocomplete="username"
      />

      <label class="label">密码</label>
      <input
        v-model="password"
        type="password"
        class="input"
        placeholder="校园网密码"
        autocomplete="current-password"
      />

      <label class="label">运营商</label>
      <select v-model="networkType" class="input">
        <option value="校园网">校园网</option>
        <option value="中国联通">中国联通</option>
      </select>

      <div class="btn-row">
        <button class="btn primary" @click="handleSave">保存凭据</button>
        <span v-if="saved" class="saved-hint">已保存</span>
      </div>

      <div class="btn-row" style="margin-top: 12px">
        <button
          class="btn"
          :class="{ primary: !loading }"
          @click="handleLogin"
          :disabled="loading"
        >
          {{ loading ? "登录中..." : "手动登录" }}
        </button>
      </div>

      <!-- Login Result -->
      <div
        v-if="loginResult"
        class="result"
        :class="{ success: loginResult.success, error: !loginResult.success }"
      >
        {{ loginResult.message }}
      </div>
    </div>

    <!-- Tab: Connection -->
    <div v-if="activeTab === 'connection'" class="section">
      <div class="toggle-group">
        <div class="toggle-row">
          <div class="toggle-info">
            <span class="toggle-label">自动登录</span>
            <span class="toggle-desc">{{ paused ? "已暂停" : "运行中" }}</span>
          </div>
          <button
            class="toggle-switch"
            :class="{ on: !paused }"
            @click="togglePause"
          >
            <span class="toggle-knob"></span>
          </button>
        </div>

        <div class="toggle-row" v-if="platform === 'windows'">
          <div class="toggle-info">
            <span class="toggle-label">开机自动启动</span>
            <span class="toggle-desc">
              {{ autoStartEnabled ? "登录 Windows 时自动启动" : "手动启动应用" }}
            </span>
          </div>
          <button
            class="toggle-switch"
            :class="{ on: autoStartEnabled }"
            @click="handleAutoStartToggle"
          >
            <span class="toggle-knob"></span>
          </button>
        </div>

        <div class="toggle-row">
          <div class="toggle-info">
            <span class="toggle-label">忽略 SSID 检测</span>
            <span class="toggle-desc">
              {{ ignoreSsid ? "任意网络下尝试登录" : "仅 TUST / 10.x 网络下登录" }}
            </span>
          </div>
          <button
            class="toggle-switch"
            :class="{ on: ignoreSsid }"
            @click="toggleIgnoreSsid"
          >
            <span class="toggle-knob"></span>
          </button>
        </div>
      </div>

      <div class="status-details" v-if="networkStatus">
        <div class="detail-row">
          <span class="detail-key">WiFi SSID</span>
          <span class="detail-val">{{ networkStatus.wifi_ssid || "—" }}</span>
        </div>
        <div class="detail-row">
          <span class="detail-key">IPv4</span>
          <span class="detail-val">{{ networkStatus.local_ipv4 || "—" }}</span>
        </div>
        <div class="detail-row">
          <span class="detail-key">IPv6</span>
          <span class="detail-val">{{ networkStatus.local_ipv6 || "—" }}</span>
        </div>
        <div class="detail-row">
          <span class="detail-key">校园网</span>
          <span class="detail-val">
            <span class="status-dot" :class="{ online: networkStatus.is_tust_network }"></span>
            {{ networkStatus.is_tust_network ? "已连接" : "未连接" }}
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.app {
  padding: 20px 24px;
  max-width: 520px;
  margin: 0 auto;
}

.title {
  font-size: 18px;
  font-weight: 600;
  margin: 0 0 16px 0;
  text-align: center;
}

/* -- Status Bar -- */

.status-bar {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  background: #f0f0f0;
  border-radius: 6px;
  font-size: 13px;
  margin-bottom: 16px;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #ccc;
  flex-shrink: 0;
}

.status-dot.online {
  background: #4caf50;
}

.sep {
  color: #ccc;
}

.tag {
  font-size: 11px;
  padding: 1px 6px;
  border-radius: 4px;
  margin-left: auto;
}

.tag.tust {
  background: #e3f2fd;
  color: #1976d2;
}

.dim {
  color: #999;
}

/* -- Tab Bar -- */

.tab-bar {
  display: flex;
  border-bottom: 2px solid #eee;
  margin-bottom: 16px;
}

.tab-btn {
  flex: 1;
  padding: 8px 0;
  border: none;
  background: none;
  font-size: 14px;
  font-weight: 500;
  color: #999;
  cursor: pointer;
  border-bottom: 2px solid transparent;
  margin-bottom: -2px;
  transition: all 0.15s;
}

.tab-btn:hover {
  color: #555;
}

.tab-btn.active {
  color: #1976d2;
  border-bottom-color: #1976d2;
}

/* -- Section -- */

.section {
  margin-bottom: 16px;
}

.label {
  display: block;
  font-size: 13px;
  font-weight: 500;
  margin-bottom: 4px;
  color: #555;
}

.input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 14px;
  margin-bottom: 10px;
  box-sizing: border-box;
  outline: none;
  transition: border-color 0.2s;
}

.input:focus {
  border-color: #1976d2;
}

select.input {
  appearance: none;
  -webkit-appearance: none;
  background: #fff url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='8'%3E%3Cpath d='M1 1l5 5 5-5' stroke='currentColor' stroke-width='1.5' fill='none'/%3E%3C/svg%3E") no-repeat right 10px center;
  padding-right: 30px;
  cursor: pointer;
  color: #333;
}

@media (prefers-color-scheme: dark) {
  select.input {
    background-color: #333;
    color: #ccc;
    border-color: #555;
  }
}

/* -- Buttons -- */

.btn-row {
  display: flex;
  gap: 8px;
  align-items: center;
}

.btn {
  padding: 8px 16px;
  border: 1px solid #ddd;
  border-radius: 6px;
  background: #fff;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.15s;
}

.btn:hover {
  background: #f5f5f5;
}

.btn.primary {
  background: #1976d2;
  color: #fff;
  border-color: #1976d2;
}

.btn.primary:hover {
  background: #1565c0;
}

.btn:disabled {
  opacity: 0.6;
  cursor: default;
}

.saved-hint {
  font-size: 12px;
  color: #4caf50;
}

/* -- Login Result -- */

.result {
  padding: 10px 14px;
  border-radius: 6px;
  font-size: 13px;
  margin-top: 12px;
}

.result.success {
  background: #e8f5e9;
  color: #2e7d32;
}

.result.error {
  background: #ffebee;
  color: #c62828;
}

/* -- Toggle Switch -- */

.toggle-group {
  display: flex;
  flex-direction: column;
  gap: 0;
}

.toggle-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 0;
  border-bottom: 1px solid #f0f0f0;
}

.toggle-row:first-child {
  padding-top: 0;
}

.toggle-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.toggle-label {
  font-size: 14px;
  font-weight: 500;
}

.toggle-desc {
  font-size: 12px;
  color: #999;
}

.toggle-switch {
  position: relative;
  width: 44px;
  height: 26px;
  border: none;
  border-radius: 13px;
  background: #ccc;
  cursor: pointer;
  transition: background 0.2s;
  flex-shrink: 0;
  padding: 0;
}

.toggle-switch.on {
  background: #1976d2;
}

.toggle-knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 22px;
  height: 22px;
  border-radius: 50%;
  background: #fff;
  transition: transform 0.2s;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}

.toggle-switch.on .toggle-knob {
  transform: translateX(18px);
}

/* -- Status Details -- */

.status-details {
  margin-top: 20px;
  padding: 12px;
  background: #f9f9f9;
  border-radius: 6px;
}

.detail-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 0;
  font-size: 13px;
}

.detail-row + .detail-row {
  border-top: 1px solid #eee;
}

.detail-key {
  color: #888;
}

.detail-val {
  display: flex;
  align-items: center;
  gap: 6px;
  color: #333;
}

/* -- Dark Mode -- */

@media (prefers-color-scheme: dark) {
  .status-bar {
    background: #2a2a2a;
  }

  .tab-bar {
    border-bottom-color: #333;
  }

  .tab-btn {
    color: #888;
  }

  .tab-btn:hover {
    color: #bbb;
  }

  .tab-btn.active {
    color: #64b5f6;
    border-bottom-color: #64b5f6;
  }

  .label {
    color: #aaa;
  }

  .input {
    background: #333;
    color: #ccc;
    border-color: #555;
  }

  .input:focus {
    border-color: #64b5f6;
  }

  .btn {
    background: #333;
    color: #ccc;
    border-color: #555;
  }

  .btn:hover {
    background: #444;
  }

  .btn.primary {
    background: #1976d2;
    color: #fff;
    border-color: #1976d2;
  }

  .toggle-row {
    border-bottom-color: #2a2a2a;
  }

  .toggle-label {
    color: #ddd;
  }

  .status-details {
    background: #2a2a2a;
  }

  .detail-row + .detail-row {
    border-top-color: #333;
  }

  .detail-val {
    color: #ccc;
  }
}

/* -- Modal -- */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.modal {
  background: #fff;
  padding: 24px;
  border-radius: 8px;
  max-width: 320px;
  width: 100%;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
}

.modal-title {
  font-size: 16px;
  font-weight: 600;
  margin: 0 0 8px 0;
}

.modal-body {
  font-size: 14px;
  color: #555;
  margin: 0 0 16px 0;
}

.modal-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}

@media (prefers-color-scheme: dark) {
  .modal {
    background: #2a2a2a;
    color: #ccc;
  }
  .modal-body {
    color: #aaa;
  }
}
</style>
