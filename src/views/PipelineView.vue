<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { api } from "../api/tauri";
import { STAGES, type ProgressEvent, type ProjectInfo, type StageState } from "../types";
import StageCard from "../components/pipeline/StageCard.vue";

const project = ref<ProjectInfo | null>(null);
const projectPath = ref("");
const pipeline = ref<Record<string, StageState>>({});
const activeStage = ref("stage1");
const stageLog = ref("");
const progress = ref<ProgressEvent | null>(null);
const report = ref<string>("");

const projects = ref<ProjectInfo[]>([]);
let unlisten: (() => void) | null = null;

/** 各阶段「重跑提示词」：随阶段保存在 project.json，重跑时带给后端 → AI 按预期方向调整 */
const hints = ref<Record<string, string>>({});
/** 快捷填充：常用调整方向（仅作输入辅助，用户可自由编辑） */
const HINT_PRESETS = [
  "台词更口语化、贴合角色性格",
  "三围数值更平衡，接近原版手感",
  "技能描述更详细，写清数值与冷却",
  "只用原版已有 API，避免自定义全局函数",
];
const running = computed(() => Object.values(pipeline.value).some((s) => s.status === "running"));

onMounted(async () => {
  projects.value = await api.listProjects();
  if (projects.value.length) await selectProject(projects.value[0].path);
  unlisten = api.onProgress((e) => {
    progress.value = e;
    if (e.stage === activeStage.value) stageLog.value = stageLog.value + e.message + "\n";
  });
});
onBeforeUnmount(() => unlisten?.());

async function selectProject(path: string) {
  projectPath.value = path;
  project.value = await api.openProject(path);
  pipeline.value = (project.value?.pipeline ?? {}) as Record<string, StageState>;
  loadHints();
  await loadStageLog(activeStage.value);
}

/** 从阶段状态回填已保存的提示词（切换项目时全量回填） */
function loadHints() {
  const h: Record<string, string> = {};
  for (const s of STAGES) h[s.id] = pipeline.value[s.id]?.hint ?? "";
  hints.value = h;
}

/** 快捷填充：追加到当前阶段的提示词（用「；」分隔多条目） */
function appendHint(text: string) {
  const cur = hints.value[activeStage.value] ?? "";
  hints.value[activeStage.value] = cur ? `${cur.replace(/[；;]\s*$/, "")}；${text}` : text;
}

const lanes = computed(() => ({
  visual: STAGES.filter((s) => s.lane !== "code"),
  code: STAGES.filter((s) => s.lane !== "visual"),
}));

async function loadStageLog(id: string) {
  activeStage.value = id;
  if (!project.value) return;
  stageLog.value = await api.getStageLog(project.value.path, id);
}

async function runAction(action: "start" | "rerun" | "skip", id: string) {
  if (!project.value) return;
  // 带当前阶段的重跑提示词（可空）；为空时后端会复用该阶段上次保存的提示词
  const hint = hints.value[id] ?? "";
  pipeline.value =
    ((await (action === "start"
      ? api.startStage(project.value.path, id, hint)
      : action === "rerun"
        ? api.rerunStage(project.value.path, id, hint)
        : api.skipStage(project.value.path, id))) as unknown as Record<string, StageState>) ?? pipeline.value;
  project.value = await api.openProject(project.value.path);
  // 只回填本次执行的阶段：不覆盖用户在其他阶段尚未提交的输入
  const saved = pipeline.value[id]?.hint;
  if (saved !== undefined) hints.value[id] = saved;
  await loadStageLog(id);
}

async function genReport() {
  if (!project.value) return;
  const r = await api.getReport(project.value.path);
  report.value = ["【Mod 文件清单】", ...r.files.map((f) => `- ${f}`), "", "【注意事项】", ...r.notes.map((n) => `- ${n}`)].join("\n");
}
</script>

<template>
  <!-- view-scroll：本视图自己滚动，外层 .app-main 不再滚动 -->
  <div class="view-scroll">
    <div class="card">
      <h3 class="card-title">流水线总览（阶段卡片 — 视觉线 / 代码线可并行切换）</h3>
      <label>项目</label>
      <select v-model="projectPath" style="width: 360px" @change="selectProject(projectPath)">
        <option v-for="p in projects" :key="p.path" :value="p.path">{{ p.name }}</option>
      </select>
      <div v-if="!project" class="muted" style="margin-top: 8px">请先在 Tab2 创建项目并确认人设定稿。</div>

      <template v-if="project">
        <div class="lane">
          <div class="lane-title">视觉线（模型）</div>
          <div class="lane-cards">
            <StageCard
              v-for="s in lanes.visual"
              :key="s.id"
              :def="s"
              :state="pipeline[s.id]"
              :active="activeStage === s.id"
              @select="loadStageLog"
              @start="runAction('start', $event)"
              @rerun="runAction('rerun', $event)"
              @skip="runAction('skip', $event)"
            />
          </div>
        </div>
        <div class="lane">
          <div class="lane-title">代码线（代码逻辑）</div>
          <div class="lane-cards">
            <StageCard
              v-for="s in lanes.code"
              :key="s.id"
              :def="s"
              :state="pipeline[s.id]"
              :active="activeStage === s.id"
              @select="loadStageLog"
              @start="runAction('start', $event)"
              @rerun="runAction('rerun', $event)"
              @skip="runAction('skip', $event)"
            />
          </div>
        </div>
      </template>
    </div>

    <div v-if="progress" class="card">
      <h3 class="card-title">当前进度</h3>
      <div style="display:flex; align-items:center; gap:10px">
        <div style="flex:1; background:#0f141b; border-radius:4px; height:10px; overflow:hidden">
          <div style="background:#4d8cff; height:100%; width:0%" :style="{ width: progress.percent + '%' }"></div>
        </div>
        <span class="muted">{{ progress.percent }}%</span>
      </div>
      <div class="muted" style="margin-top:6px">{{ progress.message }}</div>
    </div>

    <div class="card">
      <div style="display: flex; justify-content: space-between; align-items: center">
        <h3 class="card-title" style="margin:0">阶段日志 — {{ activeStage }}</h3>
        <div style="display:flex; gap:8px">
          <button class="btn-secondary" @click="genReport">生成 Mod 报告（Stage7）</button>
        </div>
      </div>

      <!-- 重跑提示词：重跑时给 AI 一个调整方向（如「只改台词，数值别动」），随阶段保存 -->
      <div class="hint-box">
        <label>重跑提示词（可选，作用于当前阶段 {{ activeStage }}）</label>
        <textarea
          v-model="hints[activeStage]"
          rows="2"
          style="width: 100%"
          placeholder="例如：保留三围数值，只重写台词风格；技能描述更详细；只用原版已有 API…"
        ></textarea>
        <div class="hint-actions">
          <button class="btn-primary" :disabled="!project || running" @click="runAction('rerun', activeStage)">
            带提示词重跑本阶段
          </button>
          <button class="btn-secondary" @click="hints[activeStage] = ''">清空提示词</button>
          <span class="muted">快捷填充</span>
          <button v-for="p in HINT_PRESETS" :key="p" class="hint-preset" @click="appendHint(p)">{{ p }}</button>
        </div>
        <div class="muted">
          阶段卡片上的「重跑」同样会带上这里填写的提示词；提示词随阶段保存。
          当前实际生效的是 LLM 阶段（Stage5 代码生成），Stage2 部件提示词待 V1 接入后生效。
        </div>
      </div>

      <pre class="code-block" style="max-height:200px">{{ stageLog || "（空）" }}</pre>
    </div>

    <div v-if="report" class="card">
      <h3 class="card-title">Mod 报告</h3>
      <pre class="code-block">{{ report }}</pre>
    </div>
  </div>
</template>

<style scoped>
.hint-box {
  background: #0f141b; border: 1px solid #2c3542; border-radius: 6px;
  padding: 10px; margin: 10px 0 12px;
}
.hint-actions { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; margin-top: 8px; }
.hint-preset {
  background: #232c3a; color: #9fb0c3; border: 1px solid #3c4a5a; border-radius: 999px;
  padding: 3px 10px; font-size: 11px;
}
.hint-preset:hover { color: #dbe7ff; border-color: #4d8cff; }
.lane { margin-top: 14px; }
.lane-title { font-weight: 600; margin-bottom: 8px; color: #9fb0c3; }
.lane-cards { display: flex; gap: 10px; flex-wrap: wrap; }
</style>
