<script setup lang="ts">
import { onMounted, reactive, ref } from "vue";
import { api } from "../api/tauri";
import type { AppConfig, DstInfo, EnvItem } from "../types";

const cfg = reactive<AppConfig>({
  dst_dir: "",
  modtools_path: "",
  llm: { base_url: "", api_key: "", model: "" },
  image: { base_url: "", api_key: "", model: "" },
  update: { enabled: false, repo: "" },
});

const dstInfo = ref<DstInfo | null>(null);
const llmTestMsg = ref("");
const imgTestMsg = ref("");
const envItems = ref<EnvItem[]>([]);
const installMsg = ref("");
const installing = ref(""); // 当前安装分组：base / rembg / sam2
const pipMirror = ref("https://mirrors.aliyun.com/pypi/simple/");
const updateMsg = ref("");
const saving = ref(false);

onMounted(async () => {
  const c = await api.getConfig();
  Object.assign(cfg, c);
  refreshEnv();
  if (cfg.dst_dir) validateDst();
});

async function save() {
  saving.value = true;
  try {
    const c = await api.saveConfig(cfg);
    Object.assign(cfg, c);
  } finally {
    saving.value = false;
  }
}

async function validateDst() {
  dstInfo.value = await api.validateDstDir(cfg.dst_dir);
}

async function testLlm() {
  llmTestMsg.value = "测试中…";
  const r = await api.llmTest(cfg.llm);
  llmTestMsg.value = r.ok ? `✓ ${r.message}` : `✗ ${r.message}`;
}

async function testImage() {
  imgTestMsg.value = "测试中…";
  const r = await api.imageTest(cfg.image);
  imgTestMsg.value = r.ok ? `✓ ${r.message}` : `✗ ${r.message}`;
}

async function refreshEnv() {
  envItems.value = await api.pythonEnvStatus();
}

// 按功能组一键安装依赖：base（Pillow/numpy）/ rembg（抠图）/ sam2（CPU 版分割）
const INSTALL_LABELS: Record<string, string> = {
  base: "基础依赖（Pillow / numpy）",
  rembg: "rembg 抠图",
  sam2: "SAM2 分割（CPU，体积大）",
};
async function installEnv(group: string) {
  installing.value = group;
  installMsg.value = `安装中…（${INSTALL_LABELS[group]}，下载较大，视网络可能需要几分钟）`;
  try {
    const r = await api.pythonEnvSetup(group, pipMirror.value);
    installMsg.value = r.ok ? `✓ ${r.message}` : `✗ ${r.message}`;
  } finally {
    installing.value = "";
    await refreshEnv();
  }
}

// 自动检测 autocompiler.exe（Steam 注册表 + libraryfolders.vdf）
async function detectModtools() {
  const p = await api.detectModtools();
  if (p) {
    cfg.modtools_path = p;
    installMsg.value = `✓ 已自动检测到 autocompiler.exe：${p}（请点击“保存配置”生效）`;
  } else {
    installMsg.value = "✗ 未找到 autocompiler.exe，请确认已通过 Steam 安装 Don't Starve Mod Tools";
  }
}

async function checkUpdate() {
  const r = await api.checkUpdate();
  updateMsg.value = `${r.message}${r.latest_version ? ` 最新版本: ${r.latest_version}` : ""}`;
}

function pickDstDir() {
  // 目录选择：使用 webkitdirectory 文件选择器（WebView2 支持），选中后取第一项的目录前缀
  const input = document.createElement("input");
  input.type = "file";
  input.setAttribute("webkitdirectory", "");
  input.onchange = () => {
    const f = input.files && input.files[0];
    if (f) {
      // webkitRelativePath 形如 "Don't Starve Together/xxx/yyy"
      const root = f.webkitRelativePath.split("/")[0];
      cfg.dst_dir = (f as unknown as { path?: string }).path
        ? (f as unknown as { path: string }).path.split("\\").slice(0, -1).join("\\")
        : root;
      validateDst();
    }
  };
  input.click();
}
</script>

<template>
  <div>
    <div class="card">
      <h3 class="card-title">DST 游戏目录配置</h3>
      <div style="display: flex; gap: 8px; align-items: center">
        <input
          type="text"
          v-model="cfg.dst_dir"
          placeholder="选择 Don't Starve Together 根目录，例如 D:\Steam\steamapps\common\Don't Starve Together"
          style="flex: 1"
        />
        <button class="btn-secondary" @click="pickDstDir">浏览</button>
        <button class="btn-primary" @click="validateDst">校验</button>
      </div>
      <div v-if="dstInfo" style="margin-top: 8px" :class="dstInfo.valid ? 'ok-text' : 'error-text'">
        {{ dstInfo.valid ? `✓ 校验通过，找到 ${dstInfo.scripts_count} 个参考脚本` : "✗ 校验失败：" }}
        <ul v-if="!dstInfo.valid" style="margin: 4px 0 0; padding-left: 18px">
          <li v-for="(e, i) in dstInfo.errors" :key="i">{{ e }}</li>
        </ul>
      </div>
    </div>

    <div class="card">
      <h3 class="card-title">API 配置（OpenAI 兼容接口：ollama / deepseek / qwen …）</h3>
      <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 16px">
        <div>
          <h4 style="margin: 0 0 4px">LLM API</h4>
          <label>Base URL</label>
          <input type="text" v-model="cfg.llm.base_url" placeholder="http://127.0.0.1:11434/v1" style="width: 100%" />
          <label>API Key</label>
          <input type="password" v-model="cfg.llm.api_key" placeholder="留空表示无需鉴权" style="width: 100%" />
          <label>模型名称</label>
          <input type="text" v-model="cfg.llm.model" placeholder="qwen2.5:7b / deepseek-chat" style="width: 100%" />
          <button class="btn-secondary" style="margin-top: 8px" @click="testLlm">测试连通性</button>
          <div v-if="llmTestMsg" class="muted" style="margin-top: 4px">{{ llmTestMsg }}</div>
        </div>
        <div>
          <h4 style="margin: 0 0 4px">图像生成 API（图生图 / 文生图）</h4>
          <label>Base URL</label>
          <input type="text" v-model="cfg.image.base_url" placeholder="OpenAI 兼容图像接口（V1 接入）" style="width: 100%" />
          <label>API Key</label>
          <input type="password" v-model="cfg.image.api_key" style="width: 100%" />
          <label>模型名称</label>
          <input type="text" v-model="cfg.image.model" style="width: 100%" />
          <button class="btn-secondary" style="margin-top: 8px" @click="testImage">测试连通性</button>
          <div v-if="imgTestMsg" class="muted" style="margin-top: 4px">{{ imgTestMsg }}</div>
        </div>
      </div>
    </div>

    <div class="card">
      <h3 class="card-title">工具环境管理（按功能组就绪检测；图像依赖 V1 接入）</h3>
      <table style="width: 100%; border-collapse: collapse">
        <tr style="text-align: left">
          <th style="padding: 4px 8px">组件</th><th>状态</th><th>说明</th>
        </tr>
        <tr v-for="it in envItems" :key="it.name">
          <td style="padding: 4px 8px">{{ it.name }}</td>
          <td :class="it.ready ? 'ok-text' : 'error-text'">{{ it.ready ? "✓ 就绪" : "✗ 缺失" }}</td>
          <td class="muted">{{ it.detail }}</td>
        </tr>
        <tr v-if="!envItems.length"><td colspan="3" class="muted">（浏览器预览模式无数据；Tauri 运行后展示）</td></tr>
      </table>
      <div style="margin-top: 10px; display: flex; gap: 8px; align-items: center; flex-wrap: wrap">
        <button class="btn-secondary" @click="refreshEnv">刷新状态</button>
        <button class="btn-secondary" @click="installEnv('base')" :disabled="installing !== ''">{{ installing === 'base' ? "安装中…" : "一键安装基础依赖" }}</button>
        <button class="btn-secondary" @click="installEnv('rembg')" :disabled="installing !== ''">{{ installing === 'rembg' ? "安装中…" : "安装 rembg（抠图）" }}</button>
        <button class="btn-secondary" @click="installEnv('sam2')" :disabled="installing !== ''">{{ installing === 'sam2' ? "安装中…" : "安装 SAM2（CPU，体积大）" }}</button>
      </div>
      <div style="margin-top: 8px; display: flex; gap: 8px; align-items: center">
        <label style="white-space: nowrap">pip 镜像（https）</label>
        <input type="text" v-model="pipMirror" style="flex: 1" placeholder="https://mirrors.aliyun.com/pypi/simple/（留空用默认源）" />
      </div>
      <div v-if="installMsg" style="margin-top: 8px" :class="installMsg.startsWith('✓') ? 'ok-text' : 'error-text'">{{ installMsg }}</div>
      <div style="margin-top: 12px">
        <label>Klei Mod Tools（autocompiler.exe）路径（V1 编译用）</label>
        <div style="display: flex; gap: 8px; margin-top: 4px">
          <input type="text" v-model="cfg.modtools_path" style="flex: 1" placeholder="D:\Steam\steamapps\common\Don't Starve Mod Tools\mod_tools\autocompiler.exe" />
          <button class="btn-secondary" @click="detectModtools">自动检测</button>
        </div>
        <span class="muted">可点击“自动检测”扫描 Steam 注册表与 libraryfolders.vdf 自动定位；改完后点“保存配置”生效</span>
      </div>
    </div>

    <div class="card">
      <h3 class="card-title">版本更新设置</h3>
      <div style="display: flex; gap: 8px; align-items: center; flex-wrap: wrap">
        <label style="margin: 0">自动更新 <input type="checkbox" v-model="cfg.update.enabled" /></label>
        <input type="text" v-model="cfg.update.repo" placeholder="GitHub 仓库地址，如 owner/dst-mod-agent-generator" style="flex: 1" />
        <button class="btn-secondary" @click="checkUpdate">立即检查</button>
      </div>
      <div v-if="updateMsg" class="muted" style="margin-top: 6px">{{ updateMsg }}</div>
    </div>

    <div style="display: flex; gap: 8px; margin-top: 12px">
      <button class="btn-primary" @click="save" :disabled="saving">{{ saving ? "保存中…" : "保存配置" }}</button>
      <span class="muted" style="align-self: center">配置与 API Key 仅保存在本机，不上传任何第三方服务器</span>
    </div>
  </div>
</template>
