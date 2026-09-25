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
}

export interface ProjectInfo {
  path: string;
  name: string;
  meta: ModMeta;
  created_at: string;
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
