<script setup lang="ts">
import { ref } from "vue";
import ConfigView from "./views/ConfigView.vue";
import ProjectView from "./views/ProjectView.vue";
import PipelineView from "./views/PipelineView.vue";
import DebugView from "./views/DebugView.vue";

const tabs = [
  { id: "config", label: "Tab1 全局配置" },
  { id: "project", label: "Tab2 项目创建与人设对话" },
  { id: "pipeline", label: "Tab3 资源生成" },
  { id: "debug", label: "Tab4 测试与调试" },
] as const;

const active = ref<(typeof tabs)[number]["id"]>("config");
</script>

<template>
  <div class="app-shell">
    <header class="app-header">
      <h1>DST Mod Agent Generator</h1>
      <span class="app-version">v0.1.0 MVP</span>
    </header>
    <nav class="app-tabs">
      <button
        v-for="t in tabs"
        :key="t.id"
        class="tab-btn"
        :class="{ active: active === t.id }"
        @click="active = t.id"
      >
        {{ t.label }}
      </button>
    </nav>
    <main class="app-main">
      <ConfigView v-if="active === 'config'" />
      <ProjectView v-else-if="active === 'project'" />
      <PipelineView v-else-if="active === 'pipeline'" />
      <DebugView v-else />
    </main>
  </div>
</template>

<style>
* { box-sizing: border-box; }
html, body, #app { height: 100%; margin: 0; }
body {
  font-family: "Segoe UI", "Microsoft YaHei", system-ui, sans-serif;
  background: #171a21;
  color: #e6e6e6;
  font-size: 14px;
}
.app-shell { display: flex; flex-direction: column; height: 100%; }
.app-header {
  display: flex; align-items: baseline; gap: 12px;
  padding: 10px 20px; background: #1d232d; border-bottom: 1px solid #2c3542;
}
.app-header h1 { font-size: 17px; margin: 0; }
.app-version { color: #8b96a5; font-size: 12px; }
.app-tabs { display: flex; gap: 4px; padding: 8px 20px 0; background: #1d232d; border-bottom: 1px solid #2c3542; }
.tab-btn {
  border: 1px solid transparent; border-bottom: none; border-radius: 6px 6px 0 0;
  background: transparent; color: #9fb0c3; padding: 8px 16px; cursor: pointer; font-size: 13px;
}
.tab-btn.active { background: #171a21; color: #fff; border-color: #2c3542; }
.app-main { flex: 1; overflow: auto; padding: 16px 20px; }
button { cursor: pointer; }
button:disabled { cursor: not-allowed; opacity: 0.55; }
input[type="text"], input[type="password"], textarea, select {
  background: #0f141b; color: #e6e6e6; border: 1px solid #2c3542; border-radius: 4px; padding: 6px 8px;
}
label { display: block; margin: 8px 0 4px; color: #9fb0c3; font-size: 12px; }
.btn-primary { background: #4d8cff; color: #fff; border: none; border-radius: 4px; padding: 8px 14px; }
.btn-secondary { background: #2c3542; color: #e6e6e6; border: 1px solid #3c4a5a; border-radius: 4px; padding: 6px 12px; }
.card {
  background: #1d232d; border: 1px solid #2c3542; border-radius: 8px; padding: 14px 16px; margin-bottom: 12px;
}
.card-title { font-weight: 600; margin: 0 0 10px; color: #fff; }
.error-text { color: #ff6b6b; }
.ok-text { color: #5cd68a; }
.muted { color: #8b96a5; font-size: 12px; }
pre.code-block {
  background: #0f141b; border: 1px solid #2c3542; border-radius: 4px; padding: 10px;
  overflow: auto; max-height: 340px; white-space: pre-wrap; word-break: break-all;
}
</style>