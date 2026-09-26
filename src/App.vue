<script setup lang="ts">
import { ref } from "vue";
import ConfigView from "./views/ConfigView.vue";
import ProjectView from "./views/ProjectView.vue";
import PipelineView from "./views/PipelineView.vue";
import DebugView from "./views/DebugView.vue";

// 全局配置已从 Tab 栏移出：改为右上角「设置」按钮打开的弹窗（原 Tab1 内容不变）
const tabs = [
  { id: "project", label: "项目创建与人设对话" },
  { id: "pipeline", label: "资源生成" },
  { id: "debug", label: "测试与调试" },
] as const;

const active = ref<(typeof tabs)[number]["id"]>("project");
const showSettings = ref(false);
</script>

<template>
  <div class="app-shell">
    <header class="app-header">
      <h1>DST Mod Agent Generator</h1>
      <span class="app-version">v0.1.0 MVP</span>
      <button class="btn-secondary settings-btn" @click="showSettings = true">⚙ 设置</button>
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
      <ProjectView v-if="active === 'project'" />
      <PipelineView v-else-if="active === 'pipeline'" />
      <DebugView v-else />
    </main>

    <!-- 设置弹窗：DST 目录 / API / 工具环境 / 版本更新 -->
    <div v-if="showSettings" class="modal-mask" @click.self="showSettings = false">
      <div class="modal-panel">
        <div class="modal-head">
          <h2>设置 · 全局配置</h2>
          <button class="btn-secondary" @click="showSettings = false">关闭</button>
        </div>
        <div class="modal-body">
          <ConfigView />
        </div>
      </div>
    </div>
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
  display: flex; align-items: center; gap: 12px;
  padding: 10px 20px; background: #1d232d; border-bottom: 1px solid #2c3542;
}
.app-header h1 { font-size: 17px; margin: 0; }
.app-version { color: #8b96a5; font-size: 12px; }
/* 右上角设置入口（替代原 Tab1 全局配置页） */
.settings-btn { margin-left: auto; }
.app-tabs { display: flex; gap: 4px; padding: 8px 20px 0; background: #1d232d; border-bottom: 1px solid #2c3542; }
.tab-btn {
  border: 1px solid transparent; border-bottom: none; border-radius: 6px 6px 0 0;
  background: transparent; color: #9fb0c3; padding: 8px 16px; cursor: pointer; font-size: 13px;
}
.tab-btn.active { background: #171a21; color: #fff; border-color: #2c3542; }
/* .app-main 只负责占位，不自己滚动：避免与子视图滚动容器叠加成「双滚动条」。
   各视图用 .view-scroll（或自己的列）承担滚动，整页永远只有一条滚动条。 */
.app-main { flex: 1; min-height: 0; overflow: hidden; padding: 16px 20px; }
.view-scroll { height: 100%; overflow-y: auto; overflow-x: hidden; padding-right: 6px; }
button { cursor: pointer; }
button:disabled { cursor: not-allowed; opacity: 0.55; }
input[type="text"], input[type="password"], textarea, select {
  background: #0f141b; color: #e6e6e6; border: 1px solid #2c3542; border-radius: 4px; padding: 6px 8px;
}
/* 多行输入框只允许纵向拉伸：横向拖拽会把输入框撑出卡片 */
textarea { resize: vertical; max-width: 100%; }
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
/* 设置弹窗 */
.modal-mask {
  position: fixed; inset: 0; z-index: 100; padding: 24px;
  background: rgba(8, 11, 16, 0.72);
  display: flex; align-items: center; justify-content: center;
}
.modal-panel {
  width: min(1080px, 100%); max-height: 100%; display: flex; flex-direction: column;
  background: #171a21; border: 1px solid #2c3542; border-radius: 10px; overflow: hidden;
}
.modal-head {
  display: flex; align-items: center; justify-content: space-between;
  padding: 12px 16px; background: #1d232d; border-bottom: 1px solid #2c3542;
}
.modal-head h2 { font-size: 15px; margin: 0; }
.modal-body { overflow: auto; padding: 14px 16px; }
</style>
