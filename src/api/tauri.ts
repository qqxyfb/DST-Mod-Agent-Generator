import type {
  AgentReply,
  AppConfig,
  CharacterSheet,
  ChatMsg,
  CodegenReport,
  DstInfo,
  EnvItem,
  EnvSetupResult,
  FixResult,
  LogContent,
  LogFile,
  ModReport,
  ModMeta,
  ProgressEvent,
  ProjectInfo,
  TestResult,
  UpdateInfo,
  ValidationResult,
} from "../types";

// 浏览器调试 fallback：非 Tauri 环境返回静态桩数据，保证 `npm run dev` 可以预览 UI
const isTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
/** 暴露给视图层判断当前是否运行在 Tauri 桌面环境 */
export const isTauriRuntime = isTauri;

const MOCK_CONFIG: AppConfig = {
  dst_dir: "",
  modtools_path: "",
  llm: { base_url: "http://127.0.0.1:11434/v1", api_key: "", model: "qwen2.5:7b" },
  image: { base_url: "", api_key: "", model: "" },
  update: { enabled: false, repo: "" },
};

const MOCK_DRAFT: CharacterSheet = {
  char_name: "demo",
  display_name: "示例",
  description: "（浏览器预览占位人设）",
  stats: { health: 150, hunger: 150, sanity: 150 },
  passive_skills: [],
  active_skills: [],
  starter_items: ["cutgrass"],
  special_items: [],
  speech: { spawn: "你好，我是示例角色" },
  pros_cons: { pros: [], cons: [] },
};

const MOCK_PROJECT: ProjectInfo = {
  path: "mock://project/demo",
  name: "demo",
  notes: "",
  meta: { name: "demo", author: "预览", version: "1.0.0", description: "", tags: [] },
  created_at: new Date().toISOString(),
  character: null,
  pipeline: {},
  report: null,
};

function genMock(cmd: string, args: Record<string, unknown>): unknown {
  switch (cmd) {
    case "get_config":
      return MOCK_CONFIG;
    case "list_projects":
      return [MOCK_PROJECT];
    case "create_project": {
      const meta = args.meta as ModMeta | null;
      return {
        ...MOCK_PROJECT,
        name: meta?.name || "demo",
        meta: meta ?? MOCK_PROJECT.meta,
        notes: String(args.notes ?? ""),
      };
    }
    case "update_project": {
      const meta = args.meta as ModMeta | null;
      return { ...MOCK_PROJECT, meta: meta ?? MOCK_PROJECT.meta, notes: String(args.notes ?? "") };
    }
    case "list_references":
      return [] as string[];
    case "open_project":
      return MOCK_PROJECT;
    case "validate_dst_dir":
      return { valid: false, errors: ["（浏览器预览）请选择 DST 目录"], scripts_count: 0, sample_scripts: [] } as DstInfo;
    case "llm_test":
      return { ok: false, message: "（浏览器预览）未连接后端" } as TestResult;
    case "image_test":
      return { ok: false, message: "图像接口 V1 预留，未接入" } as TestResult;
    case "python_env_status":
      return [
        { name: "Python 环境", ready: false, detail: "（浏览器预览）Tauri 运行后探测" },
        { name: "基础依赖（Pillow / numpy）", ready: false, detail: "（浏览器预览）安装 Pillow / numpy" },
        { name: "rembg 抠图", ready: false, detail: "V1 接入" },
        { name: "SAM2 分割", ready: false, detail: "V1 接入" },
        { name: "autocompiler.exe", ready: false, detail: "用户自装" },
        { name: "ESC 模板", ready: true, detail: "随包附带" },
      ] as EnvItem[];
    case "python_env_setup":
      return { ok: false, message: "（浏览器预览）依赖安装需在 Tauri 中运行" } as EnvSetupResult;
    case "detect_modtools":
      return null;
    case "import_reference":
      return ["mock_ref.png"] as string[];
    case "check_update":
      return { current_version: "0.1.0", latest_version: null, update_available: false, message: "（浏览器预览）" } as UpdateInfo;
    case "agent_chat":
      return { reply: "（浏览器预览）请通过 Tauri 运行以调用 LLM。以下是占位草稿。", draft: MOCK_DRAFT } as AgentReply;
    case "agent_init":
      return {
        reply: "（浏览器预览）一键初始化需在 Tauri 中运行：将按固定模板读取 Mod 信息 + 角色描述 + 参考图，自动发起第一轮请求。",
        draft: MOCK_DRAFT,
      } as AgentReply;
    case "confirm_character":
      return { ok: true, errors: ["（浏览器预览）已冻结 v1"] };
    case "pipeline_state":
      return {};
    case "get_stage_log":
      return "（浏览器预览）阶段日志需在 Tauri 中运行";
    case "get_report":
      return {
        mod_dir: "mock://mod",
        files: ["modinfo.lua", "modmain.lua", "characters/demo.lua"],
        notes: ["（浏览器预览）报告占位"],
      };
    case "fix_from_log":
      return { applied: ["modmain.lua"], ok: true, message: "（浏览器预览）修复占位" };
    case "gen_console_cmds":
      return ["DebugSpawn('demo')", "c_despawn(AllPlayers[1])"];
    default:
      return null;
  }
}

async function call<T>(cmd: string, args: Record<string, unknown> = {}): Promise<T> {
  if (!isTauri) {
    return genMock(cmd, args) as T;
  }
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
}

export const api = {
  // Tab1 配置
  getConfig: () => call<AppConfig>("get_config"),
  saveConfig: (cfg: AppConfig) => call<AppConfig>("save_config", { cfg }),
  validateDstDir: (path: string) => call<DstInfo>("validate_dst_dir", { path }),
  llmTest: (cfg: AppConfig["llm"]) => call<TestResult>("llm_test", { cfg }),
  imageTest: (cfg: AppConfig["image"]) => call<TestResult>("image_test", { cfg }),
  pythonEnvStatus: () => call<EnvItem[]>("python_env_status"),
  pythonEnvSetup: (group = "base", mirror = "") =>
    call<EnvSetupResult>("python_env_setup", { group, mirror }),
  /** 自动检测 autocompiler.exe（Steam 注册表 + libraryfolders.vdf），未找到返回 null */
  detectModtools: () => call<string | null>("detect_modtools"),
  checkUpdate: () => call<UpdateInfo>("check_update"),

  // Tab2 项目与人设
  createProject: (meta: ModMeta, notes = "") => call<ProjectInfo>("create_project", { meta, notes }),
  /** 保存当前项目的 Mod 基础信息与角色描述文本（Tab2「保存修改」） */
  updateProject: (path: string, meta: ModMeta, notes = "") =>
    call<ProjectInfo>("update_project", { path, meta, notes }),
  /** 列出项目 reference/ 目录内已导入的参考图文件名（Tab2 切换项目时回填） */
  listReferences: (projectPath: string) => call<string[]>("list_references", { projectPath }),
  listProjects: () => call<ProjectInfo[]>("list_projects"),
  openProject: (path: string) => call<ProjectInfo>("open_project", { path }),
  agentChat: (projectPath: string, messages: ChatMsg[]) =>
    call<AgentReply>("agent_chat", { projectPath, messages }),
  /** 一键初始化：按内置固定模板自动发起第一轮 Agent 请求，产出游戏内人物概设 */
  agentInit: (projectPath: string) => call<AgentReply>("agent_init", { projectPath }),
  confirmCharacter: (projectPath: string) => call<ValidationResult>("confirm_character", { projectPath }),
  /** 选择参考图（Tauri dialog 插件；浏览器预览返回 null） */
  pickReferenceFiles: async (): Promise<string[] | null> => {
    if (!isTauri) return null;
    const { open } = await import("@tauri-apps/plugin-dialog");
    const sel = await open({
      multiple: true,
      directory: false,
      filters: [{ name: "图片", extensions: ["png", "jpg", "jpeg", "webp", "bmp"] }],
    });
    return Array.isArray(sel) ? sel : sel ? [sel] : [];
  },
  /** 复制参考图到项目 reference/ 目录，返回目录内全部参考图文件名 */
  importReference: (projectPath: string, filePath: string) =>
    call<string[]>("import_reference", { projectPath, filePath }),

  // Tab3 流水线
  pipelineState: (projectPath: string) => call<Record<string, unknown>>("pipeline_state", { projectPath }),
  /** hint = 可选重跑提示词：让 AI 按指定方向调整（LLM 阶段生效） */
  startStage: (projectPath: string, stageId: string, hint = "") =>
    call<Record<string, unknown>>("start_stage", { projectPath, stageId, hint }),
  rerunStage: (projectPath: string, stageId: string, hint = "") =>
    call<Record<string, unknown>>("rerun_stage", { projectPath, stageId, hint }),
  skipStage: (projectPath: string, stageId: string) =>
    call<Record<string, unknown>>("skip_stage", { projectPath, stageId }),
  getStageLog: (projectPath: string, stageId: string) => call<string>("get_stage_log", { projectPath, stageId }),
  getReport: (projectPath: string) => call<ModReport>("get_report", { projectPath }),

  // Tab4 调试
  locateLogs: () => call<LogFile[]>("locate_logs"),
  readLog: (path: string) => call<LogContent>("read_log", { path }),
  fixFromLog: (projectPath: string, logPath: string) =>
    call<FixResult>("fix_from_log", { projectPath, logPath }),
  genConsoleCmds: (projectPath: string) => call<string[]>("gen_console_cmds", { projectPath }),

  // 进度事件订阅（Rust 侧 Emitter 推送）
  onProgress(cb: (e: ProgressEvent) => void): (() => void) | null {
    if (!isTauri) return null;
    let stop: (() => void) | undefined;
    import("@tauri-apps/api/event").then(({ listen }) => {
      listen<ProgressEvent>("pipeline://progress", (ev) => cb(ev.payload)).then((un) => {
        stop = un;
      });
    });
    return () => stop?.();
  },
};
