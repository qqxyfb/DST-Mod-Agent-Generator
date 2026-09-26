<script setup lang="ts">
import { onMounted, ref } from "vue";
import { api } from "../api/tauri";
import type { AgentReply, ChatMsg, CharacterSheet, ModMeta, ProjectInfo } from "../types";

// ——— 表单状态：Mod 基础信息 + 角色描述 ———
const meta = ref<ModMeta>({ name: "", author: "", version: "1.0.0", description: "", tags: [] });
const description = ref("");
/** 创意工坊标签输入框的原始文本（逗号分隔），提交时解析为数组写入 meta.tags */
const tagsText = ref("");
const refNames = ref<string[]>([]);
const refMsg = ref("");

// ——— 项目状态 ———
const projects = ref<ProjectInfo[]>([]);
const project = ref<ProjectInfo | null>(null);
const projectPath = ref("");
/** true = 表单为「新建」态（尚未落盘）；false = 正在编辑已存在项目 */
const isNew = ref(true);
const projectMsg = ref("");

// ——— 人设对话状态 ———
const messages = ref<ChatMsg[]>([]);
const input = ref("");
const busy = ref(false);
const confirmMsg = ref("");
const preview = ref<CharacterSheet | null>(null);
const baseSent = ref(false);

onMounted(async () => {
  projects.value = await api.listProjects();
  if (projects.value.length) await openProject(projects.value[0].path);
});

/** 标签文本 → 数组（支持中英文逗号） */
function parseTags(text: string): string[] {
  return text
    .split(/[,，]/)
    .map((t) => t.trim())
    .filter(Boolean);
}

/** 重置为「新建」态：清空表单与对话，等用户点「创建项目」再落盘 */
function newProject() {
  isNew.value = true;
  project.value = null;
  projectPath.value = "";
  meta.value = { name: "", author: "", version: "1.0.0", description: "", tags: [] };
  tagsText.value = "";
  description.value = "";
  refNames.value = [];
  refMsg.value = "";
  messages.value = [];
  preview.value = null;
  confirmMsg.value = "";
  baseSent.value = false;
  projectMsg.value = "已切换到新建模式：填写下方信息后点「创建项目」。";
}

/** 独立的「创建项目」：把当前表单落盘为新项目 */
async function createProject() {
  if (!meta.value.name.trim()) {
    projectMsg.value = "请填写 Mod 名称（英文小写，将作为项目目录名）";
    return;
  }
  try {
    meta.value.tags = parseTags(tagsText.value);
    const created = await api.createProject(meta.value, description.value);
    project.value = created;
    projectPath.value = created.path;
    isNew.value = false;
    baseSent.value = false;
    refNames.value = [];
    refMsg.value = "";
    messages.value = [
      {
        role: "assistant",
        content: `项目 ${created.name} 已创建。上传参考图、填写描述后即可与我对话迭代人设。`,
      },
    ];
    projects.value = await api.listProjects();
    projectMsg.value = `✓ 项目 ${created.name} 已创建`;
  } catch (e) {
    projectMsg.value = `创建失败：${String(e)}`;
  }
}

/** 「保存修改」：写回当前项目的 Mod 基础信息与角色描述 */
async function saveProject() {
  if (!project.value) {
    projectMsg.value = "请先选择或创建项目";
    return;
  }
  try {
    meta.value.tags = parseTags(tagsText.value);
    project.value = await api.updateProject(project.value.path, meta.value, description.value);
    projects.value = await api.listProjects();
    projectMsg.value = "✓ 已保存 Mod 基础信息与角色描述";
  } catch (e) {
    projectMsg.value = `保存失败：${String(e)}`;
  }
}

/** 打开已有项目：回填 Mod 信息 / 角色描述 / 已导入参考图，并恢复对话区 */
async function openProject(path: string) {
  if (!path) return;
  projectPath.value = path;
  try {
    const info = await api.openProject(path);
    project.value = info;
    isNew.value = false;
    projectMsg.value = "";
    baseSent.value = false;
    // 回填 Mod 基础信息（tags 复制一份，避免与 project.json 数据共享引用）
    meta.value = {
      name: info.meta.name,
      author: info.meta.author,
      version: info.meta.version || "1.0.0",
      description: info.meta.description,
      tags: [...(info.meta.tags ?? [])],
    };
    tagsText.value = (info.meta.tags ?? []).join(", ");
    // 回填 Tab2 角色描述文本与已导入的参考图文件名
    description.value = info.notes ?? "";
    refNames.value = await api.listReferences(path);
    refMsg.value = "";
    if (info.character) {
      preview.value = info.character.json;
      const frozen = !!info.character.frozen;
      const ver = info.character.version;
      messages.value = [
        {
          role: "assistant",
          content: frozen
            ? `已加载人设（已定稿 ${ver}，可继续对话修改或“重跑”重新生成）。`
            : "已加载历史人设草稿。",
        },
      ];
      confirmMsg.value = frozen ? "" : "存在未定稿草稿，确认后才会冻结进入流水线。";
    } else {
      messages.value = [];
      preview.value = null;
      confirmMsg.value = "";
    }
  } catch (e) {
    projectMsg.value = `打开项目失败：${String(e)}`;
  }
}

// 选择参考图：Tauri 下用 dialog 插件拿真实路径，再复制进项目 reference/ 目录
async function pickReference() {
  if (!project.value) {
    refMsg.value = "请先创建 / 选择项目，再上传参考图。";
    return;
  }
  const picked = await api.pickReferenceFiles();
  if (picked === null) {
    refMsg.value = "浏览器预览模式无法读取真实文件；请在 Tauri 桌面窗口中运行以支持上传。";
    return;
  }
  if (!picked.length) return;
  let names = refNames.value;
  for (const p of picked) {
    try {
      // 复制进项目 reference/ 目录，返回目录内全部参考图文件名
      names = await api.importReference(project.value.path, p);
      refMsg.value = "";
    } catch (e) {
      refMsg.value = `导入失败：${String(e)}`;
    }
  }
  refNames.value = names;
}

async function send() {
  const text = input.value.trim();
  if (!text || !project.value || busy.value) return;
  messages.value.push({ role: "user", content: text });
  input.value = "";
  busy.value = true;
  try {
    // 首条消息携带项目基础信息（角色描述 + 参考图文件名），避免逐条重复注入
    const ctxNote = [
      description.value.trim() ? `[角色描述]\n${description.value.trim()}` : "",
      refNames.value.length ? `[参考图文件]\n${refNames.value.join("、")}（图像内容由 V1 视觉模型理解）` : "",
    ]
      .filter(Boolean)
      .join("\n\n");
    const payload = !baseSent.value && ctxNote ? `${text}\n\n${ctxNote}` : text;
    baseSent.value = true;
    const convo: ChatMsg[] = [...messages.value, { role: "user", content: payload } as ChatMsg];
    const reply: AgentReply = await api.agentChat(project.value.path, convo);
    messages.value.push({ role: "assistant", content: reply.reply });
    if (reply.draft) {
      preview.value = reply.draft;
      confirmMsg.value = "Agent 已产出人设草稿，请确认或继续修改。";
    }
  } finally {
    busy.value = false;
  }
}

async function confirmDraft() {
  if (!project.value) return;
  const r = await api.confirmCharacter(project.value.path);
  confirmMsg.value = r.ok
    ? `✓ 人设定稿已冻结（${r.errors.join("；")}），可前往 Tab3 生成。`
    : `✗ 校验未通过：${r.errors.join("；")}`;
  project.value = await api.openProject(project.value.path);
}
</script>

<template>
  <div class="tab2-grid">
    <div class="tab2-col">
      <div class="card">
        <h3 class="card-title">项目</h3>
        <label>切换已有项目（选择后自动回填下方信息）</label>
        <select v-model="projectPath" style="width: 100%" @change="openProject(projectPath)">
          <option value="">（新建中 / 未选择）</option>
          <option v-for="p in projects" :key="p.path" :value="p.path">{{ p.name }}（{{ p.meta.author }}）</option>
        </select>
        <button
          class="btn-secondary"
          style="margin-top: 10px; width: 100%"
          :disabled="!project"
          @click="saveProject"
        >
          保存修改
        </button>
        <div v-if="projectMsg" class="muted" style="margin-top: 6px">{{ projectMsg }}</div>
      </div>

      <div class="card">
        <h3 class="card-title">Mod 基础信息</h3>
        <label>Mod 名称（目录名，英文小写）</label>
        <input type="text" v-model="meta.name" placeholder="wanda_cn" style="width: 100%" />
        <label>作者</label>
        <input type="text" v-model="meta.author" style="width: 100%" />
        <label>版本</label>
        <input type="text" v-model="meta.version" style="width: 100%" />
        <label>简介</label>
        <textarea v-model="meta.description" rows="2" style="width: 100%"></textarea>
        <label>创意工坊标签（逗号分隔）</label>
        <input type="text" v-model="tagsText" placeholder="角色, 生存" style="width: 100%" />
        <div class="btn-row">
          <button class="btn-secondary" @click="newProject">新建项目</button>
          <button class="btn-primary" :disabled="!isNew" @click="createProject">创建项目</button>
        </div>
        <div class="muted" style="margin-top: 6px">
          「创建项目」把当前表单落盘为新项目；已选中已有项目时请用上方「保存修改」。
        </div>
      </div>

      <div class="card">
        <h3 class="card-title">参考图与描述</h3>
        <label>角色参考图（1~N 张，复制到项目 reference/ 目录）</label>
        <div style="display: flex; gap: 8px; align-items: center">
          <button class="btn-secondary" @click="pickReference" :disabled="!project">
            {{ project ? "选择参考图…" : "（先创建项目）" }}
          </button>
          <span class="muted" v-if="refNames.length">{{ refNames.length }} 张已导入</span>
        </div>
        <div v-if="refNames.length" class="muted" style="margin-top: 6px">已导入：{{ refNames.join("、") }}</div>
        <div v-if="refMsg" class="error-text" style="margin-top: 6px">{{ refMsg }}</div>
        <label>角色人设 / 性格 / 技能想法</label>
        <textarea
          v-model="description"
          rows="6"
          placeholder="例如：一个来自东方的占卜师，怕黑但夜晚视野更远，喜欢吃蓝莓……"
          style="width: 100%"
        ></textarea>
      </div>
    </div>

    <div class="tab2-col">
      <div class="card">
        <h3 class="card-title">人设 Agent 对话</h3>
        <div class="chat-box">
          <div v-for="(m, i) in messages" :key="i" class="chat-msg" :class="m.role">
            <div class="chat-bubble">{{ m.content }}</div>
          </div>
          <div v-if="busy" class="muted">Agent 思考中…</div>
        </div>
        <div style="display: flex; gap: 8px; margin-top: 10px">
          <input
            type="text"
            v-model="input"
            placeholder="描述角色设定，或要求修改某个数值 / 技能…"
            style="flex: 1"
            @keydown.enter="send"
            :disabled="!project"
          />
          <button class="btn-primary" @click="send" :disabled="!project || busy">发送</button>
        </div>
        <div v-if="confirmMsg" class="muted" style="margin-top: 8px">{{ confirmMsg }}</div>
        <button
          v-if="preview"
          class="btn-primary"
          style="margin-top: 10px"
          :disabled="!!project?.character?.frozen"
          @click="confirmDraft"
        >
          确认人设定稿（冻结为输入快照）
        </button>
      </div>

      <div v-if="preview" class="card">
        <h3 class="card-title">人设结构化草稿预览</h3>
        <pre class="code-block">{{ JSON.stringify(preview, null, 2) }}</pre>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 左列固定 340px，右列 minmax(0,1fr) 允许收缩，避免聊天/预览把整行撑宽 */
.tab2-grid { display: grid; grid-template-columns: 340px minmax(0, 1fr); gap: 16px; align-items: start; }
/* min-width:0 是关键：否则网格子项按内容最小宽度撑开，多行输入框会溢出卡片 */
.tab2-col { min-width: 0; }
.btn-row { display: flex; gap: 8px; margin-top: 10px; }
.btn-row > button { flex: 1; }
.chat-box {
  background: #0f141b; border: 1px solid #2c3542; border-radius: 6px;
  padding: 10px; height: 380px; overflow: auto; display: flex; flex-direction: column; gap: 8px;
}
.chat-msg { display: flex; }
.chat-msg.user { justify-content: flex-end; }
.chat-bubble {
  max-width: 80%; padding: 8px 10px; border-radius: 8px; white-space: pre-wrap; word-break: break-word;
}
.chat-msg.user .chat-bubble { background: #24406e; }
.chat-msg.assistant .chat-bubble { background: #232c3a; }
</style>
