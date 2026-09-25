<script setup lang="ts">
import { onMounted, ref } from "vue";
import { api } from "../api/tauri";
import type { LogContent, LogFile, ProjectInfo } from "../types";

const projects = ref<ProjectInfo[]>([]);
const projectPath = ref("");
const logs = ref<LogFile[]>([]);
const currentLog = ref<LogContent | null>(null);
const errorLines = ref<Map<number, string>>(new Map());
const fixMsg = ref("");
const cmds = ref<string[]>([]);
const busy = ref(false);

onMounted(async () => {
  projects.value = await api.listProjects();
  if (projects.value.length) projectPath.value = projects.value[0].path;
  logs.value = await api.locateLogs();
});

async function loadLog(path: string) {
  currentLog.value = await api.readLog(path);
  errorLines.value = new Map();
  for (const i of currentLog.value.error_indices) {
    errorLines.value.set(i, currentLog.value.lines[i] ?? "");
  }
}

async function fixLog() {
  if (!projectPath.value || !currentLog.value) return;
  busy.value = true;
  fixMsg.value = "修复中…";
  try {
    const r = await api.fixFromLog(projectPath.value, currentLog.value.path);
    fixMsg.value = r.ok
      ? `✓ 已应用修复：${r.applied.join("、") || "（无改动）"}。请重新启动游戏测试。`
      : `✗ ${r.message}`;
  } finally {
    busy.value = false;
  }
}

async function genCmds() {
  if (!projectPath.value) return;
  cmds.value = await api.genConsoleCmds(projectPath.value);
}

async function copyCmds() {
  await navigator.clipboard.writeText(cmds.value.join("\n"));
}
</script>

<template>
  <div style="display: grid; grid-template-columns: 320px 1fr; gap: 16px; align-items: start">
    <div>
      <div class="card">
        <h3 class="card-title">项目</h3>
        <select v-model="projectPath" style="width: 100%">
          <option v-for="p in projects" :key="p.path" :value="p.path">{{ p.name }}</option>
        </select>
      </div>
      <div class="card">
        <h3 class="card-title">DST 游戏日志（自动定位）</h3>
        <div v-if="!logs.length" class="muted">未找到日志文件（通常位于 Documents\Klei\DoNotStarveTogether）</div>
        <div v-for="l in logs" :key="l.path" class="log-item" @click="loadLog(l.path)">
          <div style="word-break: break-all">{{ l.path }}</div>
          <div class="muted">{{ l.modified }}</div>
        </div>
      </div>
    </div>

    <div>
      <div class="card">
        <div style="display: flex; justify-content: space-between; align-items: center">
          <h3 class="card-title" style="margin: 0">日志内容（错误高亮）</h3>
          <div style="display: flex; gap: 8px">
            <button class="btn-primary" @click="fixLog" :disabled="busy || !currentLog">一键修复（LLM）</button>
          </div>
        </div>
        <div v-if="fixMsg" :class="fixMsg.startsWith('✓') ? 'ok-text' : 'error-text'" style="margin: 6px 0">{{ fixMsg }}</div>
        <pre v-if="currentLog" class="code-block" style="max-height: 420px">
          <template v-for="(line, i) in currentLog.lines" :key="i">
<span :class="errorLines.has(i) ? 'err' : ''">{{ line }}</span>
          </template>
        </pre>
        <div v-else class="muted">选择左侧日志文件查看；Lua 报错行将高亮。</div>
      </div>

      <div class="card">
        <div style="display: flex; justify-content: space-between; align-items: center">
          <h3 class="card-title" style="margin: 0">控制台调试命令</h3>
          <div style="display: flex; gap: 8px">
            <button class="btn-secondary" @click="genCmds">生成当前角色命令</button>
            <button class="btn-secondary" v-if="cmds.length" @click="copyCmds">复制</button>
          </div>
        </div>
        <pre class="code-block">{{ cmds.length ? cmds.join("\n") : "（点击生成）" }}</pre>
      </div>
    </div>
  </div>
</template>

<style scoped>
.log-item { padding: 6px 8px; border-radius: 4px; cursor: pointer; font-size: 12px; }
.log-item:hover { background: #232c3a; }
.err { color: #ff6b6b; background: rgba(255, 107, 107, 0.1); display: inline-block; width: 100%; }
</style>