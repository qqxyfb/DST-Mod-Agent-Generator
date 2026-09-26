// 前后端共享类型：与 src-tauri 侧 serde 结构保持一致

export interface LlmConfig {
  base_url: string;
  api_key: string;
  model: string;
}

export interface ImageConfig {
  base_url: string;
  api_key: string;
  model: string;
}

export interface UpdateConfig {
  enabled: boolean;
  repo: string;
}

export interface AppConfig {
  dst_dir: string;
  modtools_path: string;
  llm: LlmConfig;
  image: ImageConfig;
  update: UpdateConfig;
}

export interface DstInfo {
  valid: boolean;
  errors: string[];
  scripts_count: number;
  sample_scripts: string[];
}

export interface TestResult {
  ok: boolean;
  message: string;
}

export interface EnvItem {
  name: string;
  ready: boolean;
  detail: string;
}

export interface EnvSetupResult {
  ok: boolean;
  message: string;
}

export interface UpdateInfo {
  current_version: string;
  latest_version: string | null;
  update_available: boolean;
  message: string;
}

/** DST 创意工坊筛选标签（写入 modinfo.lua 的 server_filter_tags），Tab2 多选使用 */
export const WORKSHOP_TAGS: string[] = [
  "Character",
  "Creature",
  "Item",
  "Weapon",
  "Armor",
  "Hat",
  "Tool",
  "Food",
  "Structure",
  "World",
  "Tweak",
  "Tuning",
  "Map",
  "UI",
  "Sound",
  "Server",
  "Client",
  "Language",
  "Art",
  "Cosmetic",
  "Balance",
  "Difficulty",
  "Quality of Life",
  "Utility",
  "Fun",
  "Bug Fix",
  "Library",
];

/** 标签英文值 → 界面中文名（modinfo.lua 里仍写英文值，界面展示中文更友好） */
export const WORKSHOP_TAG_LABELS: Record<string, string> = {
  Character: "角色",
  Creature: "生物",
  Item: "物品",
  Weapon: "武器",
  Armor: "护甲",
  Hat: "帽子",
  Tool: "工具",
  Food: "食物",
  Structure: "建筑",
  World: "世界",
  Tweak: "机制调整",
  Tuning: "数值调整",
  Map: "地图",
  UI: "界面",
  Sound: "音效",
  Server: "服务器",
  Client: "客户端",
  Language: "语言 / 翻译",
  Art: "美术",
  Cosmetic: "外观",
  Balance: "平衡性",
  Difficulty: "难度",
  "Quality of Life": "便捷优化",
  Utility: "实用工具",
  Fun: "趣味",
  "Bug Fix": "Bug 修复",
  Library: "前置库",
};

export interface ModMeta {
  name: string;
  author: string;
  version: string;
  description: string;
  tags: string[];
}

export interface Stats {
  health: number;
  hunger: number;
  sanity: number;
}

export interface Skill {
  id: string;
  params: Record<string, number | string | boolean>;
  item?: string;
  cooldown?: number;
}

export interface ItemDef {
  id: string;
  kind: string;
  stats?: Partial<Stats>;
}

export interface ProsCons {
  pros: string[];
  cons: string[];
}

export interface CharacterSheet {
  char_name: string;
  display_name: string;
  description?: string;
  stats: Stats;
  passive_skills: Skill[];
  active_skills: Skill[];
  starter_items: string[];
  special_items: ItemDef[];
  speech: Record<string, string>;
  pros_cons: ProsCons;
}

export interface CharacterArtifact {
  version: string;
  json: CharacterSheet;
  markdown: string;
  frozen: boolean;
  draft: boolean;
}

export interface StageState {
  status: "pending" | "running" | "awaiting" | "success" | "failed" | "skipped";
  artifact_version: string | null;
  input_hash: string | null;
  log: string[];
  updated_at: string;
  /** 最近一次重跑提示词（Tab3 定向调整用；LLM 阶段会优先遵循） */
  hint?: string;
}

export interface ProjectInfo {
  path: string;
  name: string;
  meta: ModMeta;
  created_at: string;
  /** Tab2「角色人设 / 性格 / 技能想法」自由文本；随项目保存，切换项目时回填 */
  notes: string;
  character: CharacterArtifact | null;
  pipeline: Record<string, StageState>;
  report: Record<string, unknown> | null;
}

export interface ChatMsg {
  role: "user" | "assistant" | "system";
  content: string;
}

export interface AgentReply {
  reply: string;
  draft: CharacterSheet | null;
}

export interface ValidationResult {
  ok: boolean;
  errors: string[];
}

export interface LogFile {
  path: string;
  modified: string;
}

export interface LogContent {
  path: string;
  lines: string[];
  error_indices: number[];
}

export interface FixResult {
  applied: string[];
  ok: boolean;
  message: string;
}

export interface CodegenReport {
  files: Array<{ path: string; ok: boolean; size: number }>;
  errors: string[];
}

export interface ModReport {
  mod_dir: string;
  files: string[];
  notes: string[];
}

// 流水线阶段定义（与 Rust STAGES 常量一致）
export interface StageDef {
  id: string;
  name: string;
  lane: "visual" | "code" | "shared";
  order: number;
}

export const STAGES: StageDef[] = [
  { id: "stage1", name: "人设定稿校验", lane: "shared", order: 1 },
  { id: "stage2", name: "部件提示词", lane: "visual", order: 2 },
  { id: "stage3", name: "部件图像生成", lane: "visual", order: 3 },
  { id: "stage4", name: "图像后处理", lane: "visual", order: 4 },
  { id: "stage5", name: "代码生成", lane: "code", order: 5 },
  { id: "stage6", name: "资源编译", lane: "code", order: 6 },
  { id: "stage7", name: "输出 Mod 报告", lane: "code", order: 7 },
];

export interface ProgressEvent {
  project: string;
  stage: string;
  message: string;
  percent: number;
}
