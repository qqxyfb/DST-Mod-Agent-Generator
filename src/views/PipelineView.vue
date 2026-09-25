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
  await loadStageLog(activeStage.value);
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
  pipeline.value = ((await (action === "start"
    ? api.startStage(project.value.path, id)
    : action === "rerun"
      ? api.rerunStage(project.value.path, id)
      : api.skipStage(project.value.path, id))) as unknown as Record<string, StageState>) ?? pipeline.value;
  project.value = await api.openProject(project.value.path);
  await loadStageLog(id);
}

async function genReport() {
  if (!project.value) return;
  const r = await api.getReport(project.value.path);
  report.value = ["【Mod 文件清单】", ...r.files.map((f) => `- ${f}`), "", "【注意事项】", ...r.notes.map((n) => `- ${n}`)].join("\n");
}
</script>

<template>
  <div>
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
      <pre class="code-block" style="max-height:200px">{{ stageLog || "（空）" }}</pre>
    </div>

    <div v-if="report" class="card">
      <h3 class="card-title">Mod 报告</h3>
      <pre class="code-block">{{ report }}</pre>
    </div>
  </div>
</template>

<style scoped>
.lane { margin-top: 14px; }
.lane-title { font-weight: 600; margin-bottom: 8px; color: #9fb0c3; }
.lane-cards { display: flex; gap: 10px; flex-wrap: wrap; }
</style>
