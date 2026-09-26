<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { api } from "../api/tauri";
import { WORKSHOP_TAG_LABELS, WORKSHOP_TAGS } from "../types";
import type { AgentReply, ChatMsg, CharacterSheet, ModMeta, ProjectInfo } from "../types";

// ——— 表单状态：Mod 基础信息 + 角色描述 ———
const meta = ref<ModMeta>({ name: "", author: "", version: "1.0.0", description: "", tags: [] });
const description = ref("");
/** 自定义标签输入框（不在预置标签表里的值从这里添加） */
const customTag = ref("");
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
/** 结构化草稿默认展开；收起后对话区可以独占右列全部高度 */
const showPreview = ref(true);
/** 首轮上下文（描述 + 参考图）是否已注入，避免后续每条消息重复拼接 */
const baseSent = ref(false);

/** 标签下拉面板开关 + 搜索关键字（面板内滚动 / 搜索 / 全选，避免标签表常驻撑高卡片） */
const showTagPanel = ref(false);
const tagFilter = ref("");
/** 搜索过滤：中文名与英文值都参与匹配 */
const filteredTags = computed(() => {
  const q = tagFilter.value.trim().toLowerCase();
  if (!q) return WORKSHOP_TAGS;
  return WORKSHOP_TAGS.filter(
    (t) => t.toLowerCase().includes(q) || (WORKSHOP_TAG_LABELS[t] ?? "").toLowerCase().includes(q),
  );
});

onMounted(async () => {
  projects.value = await api.listProjects();
  if (projects.value.length) await openProject(projects.value[0].path);
});

/** 添加自定义标签（去重、去空白） */
function addCustomTag() {
  const t = customTag.value.trim();
  if (!t) return;
  if (!meta.value.tags.includes(t)) meta.value.tags = [...meta.value.tags, t];
  customTag.value = "";
}

/** 勾选 / 取消勾选单个标签 */
function toggleTag(t: string) {
  meta.value.tags = meta.value.tags.includes(t)
    ? meta.value.tags.filter((x) => x !== t)
    : [...meta.value.tags, t];
}

/** 全选：一次性选中全部预置标签（自定义标签保留） */
function selectAllTags() {
  meta.value.tags = [...new Set([...meta.value.tags, ...WORKSHOP_TAGS])];
}

/** 清空：移除全部标签（含自定义） */
function clearTags() {
  meta.value.tags = [];
}

/** 移除单个标签 */
function removeTag(t: string) {
  meta.value.tags = meta.value.tags.filter((x) => x !== t);
}

/** 重置为「新建」态：清空表单与对话，等用户点「创建项目」再落盘 */
function newProject() {
  isNew.value = true;
  project.value = null;
  projectPath.value = "";
  meta.value = { name: "", author: "", version: "1.0.0", description: "", tags: [] };
  customTag.value = "";
  showTagPanel.value = false;
  tagFilter.value = "";
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
    // 标签直接来自多选框 / 自定义标签，无需再做文本解析
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
        content: `项目 ${created.name} 已创建。上传参考图、填写角色描述后，点右上角「一键生成人物概设」即可开始第一轮人设推演。`,
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
    customTag.value = "";
    showTagPanel.value = false;
    tagFilter.value = "";
    // 回填 Mod 基础信息（tags 复制一份，避免与 project.json 数据共享引用）
    meta.value = {
      name: info.meta.name,
      author: info.meta.author,
      version: info.meta.version || "1.0.0",
      description: info.meta.description,
      tags: [...(info.meta.tags ?? [])],
    };
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

/**
 * 一键初始化：不用用户手写第一句话。
 * 后端按固定模板把 Mod 基础信息 + 角色描述 + 参考图清单组装成首轮提示词，产出「游戏内人物概设」，
 * 直接作为对话区的第一条消息。
 */
async function initFromAgent() {
  if (busy.value) return;
  if (!project.value) {
    projectMsg.value = "请先创建 / 选择项目，再执行一键初始化。";
    return;
  }
  if (!description.value.trim() && !refNames.value.length) {
    confirmMsg.value = "请先填写「角色人设 / 性格 / 技能想法」或上传参考图，再点一键生成。";
    return;
  }
  busy.value = true;
  confirmMsg.value = "";
  try {
    // 先落盘当前表单：描述可能刚改过，后端 agent_init 是从 project.json 读取的
    project.value = await api.updateProject(project.value.path, meta.value, description.value);
    const reply = await api.agentInit(project.value.path);
    // 概设作为对话第一条消息（初始化即视为首轮，后续消息不再重复注入上下文）
    messages.value = [{ role: "assistant", content: reply.reply }];
    baseSent.value = true;
    if (reply.draft) {
      preview.value = reply.draft;
      showPreview.value = true;
      confirmMsg.value = "已生成人物概设与人设草稿，可继续对话修改，或确认定稿。";
    }
  } catch (e) {
    confirmMsg.value = `一键初始化失败：${String(e)}`;
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

        <label>创意工坊标签（多选，写入 modinfo 的 server_filter_tags）</label>
        <!-- 折叠式多选：默认只占一行高度，面板内可搜索 / 全选 / 滚动挑选 -->
        <div class="tag-select">
          <button type="button" class="tag-trigger" @click="showTagPanel = !showTagPanel">
            <span :class="{ placeholder: !meta.tags.length }">
              {{
                meta.tags.length
                  ? `已选 ${meta.tags.length} 个：${meta.tags.join("、")}`
                  : "点击选择标签（可多选 / 全选）"
              }}
            </span>
            <span class="tag-arrow">{{ showTagPanel ? "▲" : "▼" }}</span>
          </button>

          <!-- 点击面板外任意位置收起 -->
          <div v-if="showTagPanel" class="tag-backdrop" @click="showTagPanel = false"></div>
          <div v-if="showTagPanel" class="tag-panel">
            <div class="tag-panel-head">
              <input type="text" v-model="tagFilter" placeholder="搜索标签…" style="flex: 1" />
              <button class="btn-secondary" @click="selectAllTags">全选</button>
              <button class="btn-secondary" @click="clearTags">清空</button>
            </div>
            <div class="tag-panel-list">
              <label
                v-for="t in filteredTags"
                :key="t"
                class="tag-option"
                :class="{ on: meta.tags.includes(t) }"
                :title="t"
              >
                <input type="checkbox" :checked="meta.tags.includes(t)" @change="toggleTag(t)" />
                <span>{{ WORKSHOP_TAG_LABELS[t] ?? t }}</span>
              </label>
              <div v-if="!filteredTags.length" class="muted">没有匹配的标签，可在下方添加自定义标签。</div>
            </div>
            <div class="tag-add">
              <input
                type="text"
                v-model="customTag"
                placeholder="自定义标签，回车添加"
                style="flex: 1"
                @keydown.enter.prevent="addCustomTag"
              />
              <button class="btn-secondary" @click="addCustomTag">添加</button>
            </div>
          </div>
        </div>

        <!-- 已选标签：超过 3 行时卡内滚动，不再无限撑高 -->
        <div v-if="meta.tags.length" class="tag-picker selected-list">
          <span v-for="t in meta.tags" :key="t" class="tag-chip on">
            <span>{{ WORKSHOP_TAG_LABELS[t] ?? t }}</span>
            <button class="tag-del" title="移除标签" @click="removeTag(t)">×</button>
          </span>
        </div>
        <div class="muted" style="margin-top: 6px">已选 {{ meta.tags.length }} 个标签</div>

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
        <button
          class="btn-primary"
          style="margin-top: 10px; width: 100%"
          :disabled="!project || busy"
          @click="initFromAgent"
        >
          ✨ 一键生成人物概设（初始化）
        </button>
        <div class="muted" style="margin-top: 6px">
          按上方描述 + 参考图清单，自动发起第一次 Agent 请求，产出游戏内人物概设并写入右侧对话区。
        </div>
      </div>
    </div>

    <div class="tab2-col right">
      <div class="card chat-card">
        <div class="card-head">
          <h3 class="card-title">人设 Agent 对话</h3>
          <button class="btn-primary" :disabled="!project || busy" @click="initFromAgent">
            ✨ 一键生成人物概设
          </button>
        </div>
        <div class="chat-box">
          <div v-if="!messages.length" class="muted">
            还没有对话。填写左侧描述 / 上传参考图后点「一键生成人物概设」，或直接在下方输入你的想法。
          </div>
          <div v-for="(m, i) in messages" :key="i" class="chat-msg" :class="m.role">
            <div class="chat-bubble">{{ m.content }}</div>
          </div>
          <div v-if="busy" class="muted">Agent 思考中…</div>
        </div>
        <div class="chat-input">
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

      <div v-if="preview" class="card preview-card">
        <div class="card-head">
          <h3 class="card-title">人设结构化草稿预览</h3>
          <button class="btn-secondary" @click="showPreview = !showPreview">
            {{ showPreview ? "收起" : "展开" }}
          </button>
        </div>
        <pre v-if="showPreview" class="code-block">{{ JSON.stringify(preview, null, 2) }}</pre>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 左列固定 340px，右列 minmax(0,1fr) 允许收缩，避免聊天/预览把整行撑宽 */
.tab2-grid { display: grid; grid-template-columns: 340px minmax(0, 1fr); gap: 16px; height: 100%; min-height: 0; }
/* min-width:0 是关键：否则网格子项按内容最小宽度撑开，多行输入框会溢出卡片 */
.tab2-col { min-width: 0; }
/* 右列铺满：纵向 flex + 高度 100%，聊天卡片 flex:1 吃掉全部剩余高度 */
.tab2-col.right { display: flex; flex-direction: column; height: 100%; min-height: 0; }
.card-head { display: flex; align-items: center; justify-content: space-between; gap: 10px; margin-bottom: 10px; }
.card-head .card-title { margin: 0; }
.chat-card { display: flex; flex-direction: column; flex: 1; min-height: 360px; }
.btn-row { display: flex; gap: 8px; margin-top: 10px; }
.btn-row > button { flex: 1; }
.chat-box {
  background: #0f141b; border: 1px solid #2c3542; border-radius: 6px;
  padding: 10px; flex: 1; min-height: 120px; overflow: auto;
  display: flex; flex-direction: column; gap: 8px;
}
.chat-input { display: flex; gap: 8px; margin-top: 10px; }
.chat-msg { display: flex; }
.chat-msg.user { justify-content: flex-end; }
.chat-bubble {
  max-width: 80%; padding: 8px 10px; border-radius: 8px; white-space: pre-wrap; word-break: break-word;
}
.chat-msg.user .chat-bubble { background: #24406e; }
.chat-msg.assistant .chat-bubble { background: #232c3a; }
/* 草稿预览卡不抢高度：展开时上限 260px，收起后只剩标题条 */
.preview-card { margin-bottom: 0; }
.preview-card .code-block { max-height: 260px; }

/* ——— 创意工坊标签多选（折叠面板） ——— */
.tag-select { position: relative; }
.tag-trigger {
  width: 100%; display: flex; align-items: center; gap: 8px; text-align: left;
  background: #0f141b; color: #e6e6e6; border: 1px solid #2c3542; border-radius: 4px;
  padding: 6px 8px; font-size: 13px;
}
/* 已选内容过长时省略号截断，保证触发器始终只有一行高 */
.tag-trigger > span:first-child { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.tag-trigger .placeholder { color: #6b7684; }
.tag-arrow { color: #8b96a5; font-size: 11px; }
.tag-backdrop { position: fixed; inset: 0; z-index: 10; }
.tag-panel {
  position: absolute; left: 0; right: 0; top: calc(100% + 4px); z-index: 11;
  background: #1d232d; border: 1px solid #3c4a5a; border-radius: 6px; padding: 8px;
  box-shadow: 0 8px 20px rgba(0, 0, 0, 0.45);
}
.tag-panel-head { display: flex; align-items: center; gap: 6px; }
.tag-panel-list {
  margin-top: 8px; max-height: 190px; overflow: auto;
  display: grid; grid-template-columns: 1fr 1fr; gap: 1px 6px;
}
.tag-option {
  display: flex; align-items: center; gap: 6px; margin: 0; padding: 3px 5px; border-radius: 4px;
  font-size: 12px; color: #9fb0c3; cursor: pointer; min-width: 0;
}
.tag-option:hover { background: #232c3a; }
.tag-option.on { background: #1d3350; color: #dbe7ff; }
.tag-option > span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.tag-option input[type="checkbox"] { margin: 0; flex: none; }
.tag-picker { display: flex; flex-wrap: wrap; gap: 6px; }
/* 已选标签最多 3 行，超出后卡内滚动 */
.selected-list { max-height: 76px; overflow-y: auto; margin-top: 8px; }
/* 覆写全局 label 的 block / margin，让标签变成小胶囊 */
.tag-chip {
  display: inline-flex; align-items: center; gap: 4px; margin: 0; cursor: pointer;
  padding: 3px 8px; border: 1px solid #3c4a5a; border-radius: 999px;
  background: #0f141b; color: #9fb0c3; font-size: 12px; user-select: none;
}
.tag-chip.on { border-color: #4d8cff; background: #1d3350; color: #dbe7ff; }
.tag-chip input[type="checkbox"] { margin: 0; }
.tag-del {
  background: transparent; border: none; color: #9fb0c3; font-size: 13px; line-height: 1;
  padding: 0 2px; cursor: pointer;
}
.tag-del:hover { color: #ff6b6b; }
.tag-add { display: flex; gap: 8px; margin-top: 8px; }
</style>