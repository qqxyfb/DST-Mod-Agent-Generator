<script setup lang="ts">
import type { StageDef, StageState } from "../../types";

const props = defineProps<{
  def: StageDef;
  state: StageState | undefined;
  active: boolean;
}>();

const emit = defineEmits<{
  select: [stageId: string];
  start: [stageId: string];
  rerun: [stageId: string];
  skip: [stageId: string];
}>();

function statusLabel(): string {
  const s = props.state?.status ?? "pending";
  const map: Record<string, string> = {
    pending: "未开始",
    running: "进行中",
    awaiting: "待确认",
    success: "成功",
    failed: "失败",
    skipped: "已跳过",
  };
  return map[s] ?? s;
}
</script>

<template>
  <div
    class="stage-card"
    :class="[state?.status ?? 'pending', { active }]"
    @click="emit('select', def.id)"
  >
    <div class="stage-head">
      <span class="stage-no">{{ def.order }}</span>
      <span class="stage-status">{{ statusLabel() }}</span>
    </div>
    <div class="stage-name">{{ def.name }}</div>
    <div class="stage-lane">{{ def.lane === "shared" ? "共享" : def.lane === "visual" ? "视觉线" : "代码线" }}</div>
    <div v-if="state?.artifact_version" class="stage-ver">产物: {{ state.artifact_version }}</div>
    <div v-if="state?.hint" class="stage-hint" :title="state.hint">✎ 已设重跑提示词</div>
    <div class="stage-ops" @click.stop>
      <button class="btn-secondary" @click="emit('start', def.id)" :disabled="state?.status === 'running'">开始</button>
      <button class="btn-secondary" @click="emit('rerun', def.id)" :disabled="!state || state.status === 'pending'">重跑</button>
      <button class="btn-secondary" @click="emit('skip', def.id)" :disabled="state?.status === 'running'">跳过</button>
    </div>
  </div>
</template>

<style scoped>
.stage-card {
  background: #1d232d; border: 1px solid #2c3542; border-radius: 8px;
  padding: 10px; min-width: 150px; cursor: pointer; display: flex; flex-direction: column; gap: 4px;
}
.stage-card.active { border-color: #4d8cff; box-shadow: 0 0 0 1px #4d8cff; }
.stage-card.success { border-color: #5cd68a; }
.stage-card.failed { border-color: #ff6b6b; }
.stage-card.skipped { opacity: 0.6; }
.stage-head { display: flex; justify-content: space-between; align-items: center; }
.stage-no {
  background: #2c3542; border-radius: 4px; padding: 1px 7px; font-size: 12px; font-weight: 700;
}
.stage-status { font-size: 11px; color: #8b96a5; }
.stage-name { font-weight: 600; font-size: 13px; }
.stage-lane { font-size: 11px; color: #8b96a5; }
.stage-ver { font-size: 11px; color: #5cd68a; }
.stage-hint { font-size: 11px; color: #e0b341; }
.stage-ops { display: flex; gap: 4px; margin-top: 4px; }
.stage-ops .btn-secondary { font-size: 11px; padding: 3px 8px; }
</style>