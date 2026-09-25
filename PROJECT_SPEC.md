# DST-Mod-Agent-Generator 项目设计文档（评审修订版）

- 版本：v1.1（评审修订版）
- 日期：2026-09-23
- 状态：M0 设计定稿，待进入 MVP 开发（M1）
- 说明：本文档基于原始需求重新生成，已合并：评审问题清单与修订决策、可行性分析结论、用户补充的两项流程要求（阶段卡片化 + 视觉/代码双管线可并行、单阶段/单部件可重新生成）。

## 目录

1. 文档评审结论
2. 可行性分析
3. 产品目标与范围
4. 需求规格（修订后）
5. 流水线设计（阶段卡片 + 双管线 + 可重跑）
6. 项目目录结构（标准）
7. 架构与核心规格
8. 错误处理与人工兜底
9. 安全与隐私
10. 版本更新机制
11. 内置预制资源与许可说明
12. 硬性限制（固化，不可突破）
13. 里程碑与 Roadmap
14. 验收标准（MVP）

## 1. 文档评审结论

### 1.1 总体评价

原始需求结构完整、目标明确，优点：

- 面向普通用户的一站式体验，Tab 划分清晰，MVP 范围（配置页、人设对话、代码生成）合理。
- 明确固化"不自动生成骨骼动画"等边界，防止实现阶段无限扩展。
- 错误处理规则具体（重试、跳过、LLM 修复），API 直连用户配置地址，安全约束好。
- 预留图像模块与编译模块接口，符合渐进式开发。

主要不足集中在：技术细节缺定义（部件清单、尺寸命名、日志路径、ModTools 来源）、流程模型与"并行/重跑"补充需求冲突、若干实现层坑（Windows 文件锁、seed 兼容、mods 目录写入权限）。

### 1.2 评审问题清单与修订决策

级别说明：P0 = 必须修订（影响正确性或必然失败）；P1 = 影响体验/健壮性；P2 = 优化。

| # | 级别 | 问题 | 修订决策 |
|---|---|---|---|
| C1 | P0 | 部件清单自相矛盾：列举了 head/body/arm_left/arm_right/leg_left/leg_right/bigportrait/avatar/item_icon 共 9 项却写"8 个"；且缺 DST 必需的 ghost 变体、smallportrait、modicon | 统一为"部件清单"（7.6 节），以内置模板 scml 实际引用名为唯一事实源，工具自动解析校验 |
| C2 | P0 | "自动 resize 到 2 的幂尺寸"未说明模板要求的精确尺寸与锚点，粗暴拉伸会破坏角色比例 | 增加部件-尺寸-命名对照（7.6 节）；宽高比与模板不符时采用"等比缩放 + 透明留白居中"保证锚点不变，并标记待人工复核 |
| C3 | P0 | 自动写入 DST mods 目录：游戏运行中文件被锁、或目录只读/无权限时失败 | 增加"导出缓存目录"为默认输出；写入 DST mods 前检测游戏进程与文件锁，失败提示手动复制（8 节） |
| C4 | P0 | 自动更新"自动替换程序文件"：Windows 上运行中的 exe 无法被覆盖 | 改为"下载到临时目录 + 退出后由 updater 替换 + 重启主程序"，失败回滚（10 节） |
| C5 | P0 | autocompiler 来源未定义；工具与模板的授权未说明 | 用户经 Steam 安装 Don't Starve Mod Tools 并手动配置路径，工具不捆绑分发；ESC 模板随包附带需注明来源与许可（11 节） |
| C6 | P0 | 人设 JSON 无 Schema：LLM 可能输出非法数值/技能名，生成非法 prefab | 固化人设 JSON Schema（枚举白名单 + 数值范围），LLM 输出强制校验，失败带错误重试（7.5 节） |
| C7 | P1 | "固定 seed 保证画风统一"对不支持 seed 参数的图像 API 无效 | 图像适配层区分支持/不支持 seed；不支持时降级为"固定风格前缀 + 固定负面提示词 + 参考图一致"（7.7 节） |
| C8 | P1 | DST 日志路径未定义，读取方式未说明 | 明确 Windows 日志默认路径与递归探测策略，提供手动选择兜底（4.4、8.4 节） |
| C9 | P1 | "一键安装依赖自动拉取 Python/rembg/SAM2"范围含糊 | 按功能组拆分安装项，每组有就绪状态检测；MVP 仅安装基础 Python 运行环境（4.1.3 节） |
| C10 | P1 | "读取 scripts 原版代码作参考"无粒度定义，全量注入会超 token 预算 | 定义白名单范围（prefabs 关键人物、components 白名单、strings 关键文件）+ 本地索引缓存（7.8 节） |
| C11 | P1 | Tab3 有 7 个 Skill，而"程序内部 Skill 调用规则"只列 5 个，两套编号不一致 | 统一为 7 个流程阶段（前端卡片）+ 5 个内部 Skill 域的映射（5.3 节） |
| C12 | P1 | 原"按顺序执行"流程与用户补充"可并行切换、单独重跑"冲突 | 引入视觉线/代码线双管线 + 阶段状态机 + 产物版本化（5 节） |
| C13 | P2 | 未提中文语言文件（schinese）与 modicon（创意工坊图标） | 代码生成增加可选语言包与 modicon 生成（7.8 节） |
| C14 | P2 | bigportrait/smallportrait/avatar 语义未区分（全身立绘 vs 小头像） | 提示词阶段分别给出规格（7.6 节） |
| C15 | P2 | "打包 anim zip 动画包"表述不准确：DST mod 运行时不需要 zip（zip 仅为创意工坊上传格式） | 修订为：输出 anim/（bin+tex）；创意工坊 zip 打包列入 V2 |
| C16 | P2 | Tab3 Skill1"人设定稿校验"与 Tab2"人设对话定稿"职责重叠 | 合并：Tab2 产出定稿 JSON；Tab3 Stage1 只做校验 + 冻结版本（5 节） |

### 1.3 用户补充需求（已固化）

- 补充 1：制作流程分阶段展示为卡片；流程上模型（视觉）与代码逻辑阶段可并行、可切换。
- 补充 2：AI 结果不可能一次达标，需提供"特定阶段重新生成 / 修改"能力，例如对不满意部件单独重新生成。

### 1.4 修订摘要

- 流程模型：严格串行 Skill → 双管线阶段状态机（第 5 节）。
- 新增规格：人设 JSON Schema（7.5）、部件对照（7.6）、API 适配层（7.7）、代码生成规则（7.8）。
- 新增设计：导出缓存目录、更新重启替换、ModTools 路径配置、日志自动定位。

## 2. 可行性分析

### 2.1 总体结论

**可行。** 技术栈成熟（Tauri 2 + Vue 3 + Rust 主程序 + Python 子进程）；DST 人物 mod 本质是"Lua 脚本 + Spriter 骨骼（复用 ESC 模板）+ TEX 贴图"，各环节均有成熟本地工具链。主要不确定性集中在图像自动化管线（AI 出图 → 抠图 → 分割 → 模板对齐），必须设计人工兜底，不承诺"一次全自动"。

### 2.2 分模块可行性

| 模块 | 方案 | 可行性 | 关键风险 |
|---|---|---|---|
| GUI | Tauri 2 + Vue 3，Windows 优先，NSIS 打包 | 高 | 依赖 WebView2（Win10/11 自带） |
| 后端主程序 | Rust（Tauri commands），配置/项目/状态机 | 高 | 无 |
| Python 子进程桥 | std::process::Command + stdio JSONL，取消 = terminate | 高 | 子进程生命周期、UTF-8 编码 |
| LLM 接入 | OpenAI 兼容 chat/completions（ollama/deepseek/qwen） | 高 | 结构化输出可靠度，用 Schema 校验 + 重试兜底 |
| 图像生成 | OpenAI 兼容 images API；SD WebUI/ComfyUI 适配器（预留） | 中高 | seed 兼容性、部件一致性 |
| 抠图 rembg | onnxruntime 本地推理，CPU 可跑 | 中高 | 出图质量差时失败，走人工兜底 |
| 分割 SAM2 | 模型权重首次下载（数百 MB，可选 GPU） | 中 | 权重获取、分割失败率，可跳过/手动上传 |
| 代码生成 | LLM + 原版参考注入 + 必生成项断言 | 中高 | 规范符合性，用断言 + 游戏日志闭环修复 |
| 资源编译 | 用户自装 Klei Mod Tools，调用 autocompiler.exe；Python 改写 scml 贴图引用 | 中高 | 工具路径、scml 版本兼容 |
| 自动更新 | GitHub Release API + zip + 重启替换 | 高 | Windows 文件锁（已设计规避） |

### 2.3 关键风险与缓解

| 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|
| 图像分割 / 模板对齐失败 | 高 | 中 | 模板占位合成兜底、部件级重跑、手动上传替换、失败信息反馈 LLM 重新生成 |
| LLM 生成 Lua 不合 DST 规范 | 中 | 中 | 原版参考注入、必生成项断言、游戏日志闭环自动修复 |
| 环境缺失（ModTools 未装等） | 中 | 中 | 就绪检测 + 引导配置，缺失时禁止进入对应阶段 |
| DST 版本更新导致接口变化 | 中 | 低 | 每次生成前读原版脚本作参考，规则库版本化 |
| 长任务阻塞 UI | 中 | 中 | 重活全部在子进程/异步任务，进度事件流推送 |
| Windows 中文路径 / 非 ASCII | 低 | 中 | 全程 UTF-8，路径处理用测试用例覆盖 |

### 2.4 工作量参考（单人估算，非承诺）

- MVP（M1）：配置页 + 人设对话 + 代码生成 + 状态机骨架 + 日志读取：约 2~3 周。
- V1（M2）：图像线 + 资源编译 + 一键修复闭环：累计约 4~6 周。
- 不含图像质量调优的反复迭代时间。

## 3. 产品目标与范围

### 3.1 目标用户与价值

普通玩家无需懂 Lua、无需手动配置 ComfyUI/SAM2，上传参考图 + 文字描述即可生成 DST 人物 Mod；全程本地生成，API 直连用户自配服务。

### 3.2 名词定义

- 阶段（Stage）：流程中的一个可执行节点，有明确的输入/输出产物。
- 产物：人设 JSON vN、部件提示词集、部件 PNG、mod 源码、编译后的 mod 文件夹。
- 视觉线 / 代码线：Tab3 中两条并行管线，共享 Stage1 定稿人设。
- 定稿：用户确认后冻结的人设 JSON 版本，后续生成/编译都以它为输入快照。
- 部件：head/body 等可单独重生成的图像单元。
- 模板：内置 ESC（Extended Sample Character）scml 骨骼模板，只替换贴图不生成骨骼。

### 3.3 版本范围

| 版本 | 范围 |
|---|---|
| MVP（M1） | Tab1：DST 目录校验、LLM API 配置 + 测试、Python 基础环境一键安装、更新设置 UI；Tab2：项目创建 + 人设对话 + 定稿；Tab3：阶段卡片状态机 + Stage1 校验 + Stage5 代码生成（图像线显示"未接入"）；Tab4：日志读取 + 控制台命令 |
| V1（M2） | 图像线（提示词/生成/抠图/分割/模板对齐/命名）、Stage6 资源编译、一键修复闭环、自动更新全量 |
| V2（M3） | SD WebUI / ComfyUI 适配器、多角色批量、创意工坊发布辅助 |
## 4. 需求规格（修订后）

### 4.1 Tab1 全局配置

#### 4.1.1 DST 游戏目录配置

- 文件选择框选择 Don't Starve Together 根目录。
- 校验：目录下必须存在 mods/ 与 scripts/ 文件夹；失败弹窗提示并禁止进入生成流程（Tab3 置灰）。
- 参考上下文：按白名单读取 scripts 目录（见 7.8），生成本地索引缓存 cache/dst_scripts_index.json（记录文件路径、大小、mtime），供代码生成注入。

#### 4.1.2 API 配置面板

- LLM API：Base URL / API Key / 模型名称，OpenAI 兼容接口（ollama、deepseek、qwen 等）。
- 图像 API：Base URL / Key / 模型名称，OpenAI 兼容图像接口；预留 SD WebUI / ComfyUI 适配器槽位。
- 测试按钮：LLM 发最小 chat 请求；图像发最小生成请求（1 张、最小尺寸）；错误分类提示（网络/鉴权/限流/模型不存在）。
- Key 本地存储；可选提示使用系统凭据管理器（Windows Credential Manager 预留）。

#### 4.1.3 工具环境管理

- 一键安装按功能组拆分：基础 Python 运行环境（内嵌 venv + 依赖清单）→ V1 追加 rembg → V1 追加 SAM2 权重。
- ModTools：不自动安装；提供路径选择（Steam 的 Don't Starve Mod Tools）。
- 环境状态展示：Python 版本、rembg、SAM2、autocompiler.exe 路径、ESC 模板完整性；状态 = 就绪 / 缺失 / 版本过旧。

#### 4.1.4 版本更新设置

- 开关"自动更新" + GitHub 仓库地址输入 + "立即检查"按钮；机制见第 10 节。

### 4.2 Tab2 Mod 项目创建与人设对话

- 基础信息：Mod 名称、作者、版本、简介、创意工坊标签（多选）。
- 图片上传：1~N 张角色参考图（本地复制进项目 reference/）。
- 文本描述：人设、性格、技能想法自由文本。
- 对话聊天框：与内置 Agent 迭代修改设定；Agent 输出完整角色设定文档（三围、被动/主动技能、开局物品、专属道具、台词、优缺点），等待用户确认。
- 定稿：用户确认后冻结人设 JSON 版本（v1；每次基于新对话再做修改生成新版本 vN，不覆盖旧版本），进入 Tab3。
- 产物同步保存 Markdown 人设文档与 JSON 到项目 characters/ 目录。

### 4.3 Tab3 资源生成面板（阶段卡片 + 双管线）

- 全局展示 7 张阶段卡片（编号、名称、状态、操作按钮）。
- 双管线：
  - 视觉线：Stage1 人设定稿校验 → Stage2 部件提示词 → Stage3 部件图像生成 → Stage4 图像后处理。
  - 代码线：Stage1 人设定稿校验 → Stage5 代码生成 → Stage6 资源编译。
  - 共享 Stage1 产物；左右分栏展示，可自由切换、可并行执行；Stage6 开始前检查 Stage4 与 Stage5 产物齐备。
- 阶段卡片操作：开始/继续、重跑（重新执行当前阶段）、局部重生成（部件级）、手动覆盖（上传替换产物）、跳过（标记风险）、查看日志。
- 进度：分阶段进度条 + 当前阶段子步骤日志流；全局暂停/停止，停止点保留产出，可恢复。
- Stage7：输出 Mod 报告（文件路径列表、待人工复核项、启动游戏测试指引）。

### 4.4 Tab4 测试与调试面板

- 日志读取：自动定位 DST 日志（路径见 8.4），文件选择 + 自动刷新，错误行高亮，捕获 Lua 报错。
- 一键修复：游戏日志报错块 + 当前 mod 源码 + 相关上下文 → LLM 输出修复补丁 → 应用 → 仅重写受影响 Lua 文件 → 提示重新测试（最多 N 轮，见 8.3）。
- 控制台命令生成：针对当前角色输出 DST 控制台调试命令（c_give/c_spawn/属性查看等），一键复制。

## 5. 流水线设计（阶段卡片 + 双管线 + 可重跑）

### 5.1 全局原则

1. 每个阶段是独立可执行、可重跑、可中断的节点，有输入快照与输出产物。
2. 重跑 = 基于当前输入快照重新执行该阶段，生成新版本产物；旧版本保留 1 份用于回滚。
3. 校验失败或用户不满意 → 只在该阶段内重试/局部重生成，不强制级联重跑上游。
4. 上游产物变更（如人设出新版本）→ 下游标记"需重跑"，提示用户确认，不自动级联。

### 5.2 双管线与依赖（DAG）

```
                    ┌─ Stage2 提示词 ─ Stage3 图像生成 ─ Stage4 后处理 ─┐
Stage1 人设定稿校验 ─┤                                            ├─ Stage6 资源编译 → Stage7 报告
                    └─ Stage5 代码生成 ───────────────────────────┘
```

- Stage1 → Stage2 / Stage5：并行分支（视觉线 / 代码线）。
- Stage4 与 Stage5 均为 Stage6 的前置。
- Stage7 依赖 Stage6（及全部产物）。

### 5.3 内部 Skill 映射（统一两套编号）

| 内部 Skill | 对应阶段 | 输入 | 输出 |
|---|---|---|---|
| 人设 Skill（S1） | Stage1（含 Tab2 对话产出） | 图片 + 描述 + Mod 基础信息 | 结构化 JSON + Markdown 设定文档；用户确认定稿 |
| 图像生成 Skill（S2） | Stage2 ~ Stage4（子步骤） | 人设 JSON vN + 参考图 | 全部部件 PNG（透明 RGBA、POT、模板命名） |
| 代码生成 Skill（S3） | Stage5 | 人设 JSON vN + 原版脚本参考 | 全套 mod Lua 源码 + modinfo + strings |
| 资源编译 Skill（S4） | Stage6 | 部件 PNG + ESC 模板 + 源码 | 编译完成的 mod 文件夹（导出缓存，可选写入 DST） |
| 调试修复 Skill（S5） | Stage7 / Tab4 | 游戏日志 + mod 源码 | 修复后的 Lua 代码 |

### 5.4 阶段状态机

状态与操作矩阵：

| 状态 | 允许操作 |
|---|---|
| 未开始 | 开始 / 跳过 |
| 进行中 | 暂停 / 中断（保留进度） |
| 待确认 | 确认 / 修改并重跑 / 放弃 |
| 成功 | 重跑 / 局部重生成 / 手动覆盖 / 查看产物 |
| 失败 | 重试 / 局部重生成 / 跳过（标记风险）/ 查看日志 |
| 已跳过 | 恢复执行 |

- 卡片颜色与图标对应状态；当前运行阶段高亮并展示进度。
- 双线并行时每个阶段有独立任务句柄，互不阻塞；共享 Stage1 产物（读多写少，重跑 Stage1 需先停下游并提示确认）。

### 5.5 产物版本与增量重跑

- 项目内 artifacts/ 按产物类型 + 版本号存放；manifest.json 记录各阶段产物版本、输入快照哈希、LLM 模型、耗时。
- 部件级重生成：仅重跑指定部件 ID（如只有 head 不满意），其余部件沿用原产物。
- 手动覆盖：导入用户文件 → 标记来源 = 人工，跳过自动生成校验（尺寸/命名仍强制校验）。
## 6. 项目目录结构（标准）

顶层：

```
DST-Mod-Agent-Generator/
├── README.md
├── PROJECT_SPEC.md
├── src-tauri/                    # Rust 主程序（Tauri 2）
│   ├── src/
│   │   ├── main.rs / lib.rs
│   │   ├── commands/             # Tauri command 层（GUI 调用的后端接口）
│   │   ├── core/
│   │   │   ├── config/           # 全局配置（游戏目录/API/ModTools/更新）
│   │   │   ├── project/          # 项目模型、project.json 读写
│   │   │   ├── pipeline/         # 阶段状态机、产物版本管理（MVP 骨架）
│   │   │   ├── llm/              # OpenAI 兼容客户端 + JSON Schema 校验
│   │   │   ├── image/            # 图像 API 适配层（预留本地管线接口）
│   │   │   ├── codegen/          # 代码生成（S3，MVP 核心）
│   │   │   ├── compile/          # 资源编译（S4，预留）
│   │   │   ├── updater/          # GitHub 版本更新
│   │   │   └── dst/              # DST 目录校验、脚本读取、日志解析
│   │   ├── py/                   # Rust → Python 子进程桥（JSONL 协议）
│   │   └── ...
│   └── Cargo.toml / tauri.conf.json / capabilities
├── python/                       # Python 图像管线（V1 启用）
│   ├── pyproject.toml / requirements.txt
│   ├── bridge.py                 # stdio JSONL 入口（请求/进度/结果/取消）
│   └── pipeline/
│       ├── background_remover.py # rembg 抠图（预留）
│       ├── segmenter.py          # SAM2 部件分割（预留）
│       ├── postprocess.py        # resize/合并/命名
│       ├── escmapper.py          # 模板 scml 解析、部件对齐、尺寸表
│       └── scml_editor.py        # 改写 scml 贴图引用（编译用）
├── src/                          # Vue3 前端
│   ├── views/
│   │   ├── ConfigView.vue        # Tab1
│   │   ├── ProjectView.vue       # Tab2
│   │   ├── PipelineView.vue      # Tab3（阶段卡片）
│   │   └── DebugView.vue         # Tab4
│   ├── components/pipeline/      # StageCard / ProgressBar / LaneSwitcher
│   └── api/                      # Tauri invoke 封装 + 事件订阅
├── templates/esc/                # 内置 ESC 模板（scml + 骨骼 + 授权说明）
├── resources/
│   ├── schemas/character.schema.json  # 人设 JSON Schema
│   └── rules/codegen_rules.md         # DST 代码生成规则库（版本化）
├── cache/                        # 运行缓存（原版脚本索引等）
├── projects/                     # 用户 Mod 项目默认输出目录
└── docs/                         # 附加文档（授权、FAQ）
```

单个 Mod 项目文件夹（projects/<ModName>/）：

```
projects/<ModName>/
├── project.json           # 元数据 + 各阶段产物版本 + 配置快照
├── characters/            # 人设定稿 JSON（vN）+ Markdown 文档
├── reference/             # 用户上传参考图（原图）
├── artwork/               # 部件提示词 JSON、生成原图、后处理 PNG（按部件）
├── mod/                   # mod 源码（modmain/modinfo/prefabs/speech 等）
├── mod_compiled/          # 编译产物（导出缓存，默认不直接写 DST mods）
└── logs/                  # 各阶段日志、LLM 对话记录
```

## 7. 架构与核心规格

### 7.1 总体架构

- 前端 Vue 3（四个 Tab）→ Tauri invoke 调用 Rust。
- Rust 主进程：配置、项目、阶段状态机、LLM 调用、更新、日志解析。
- Python 子进程：图像重活（V1）；协议 = stdio JSONL（request/progress/result/cancel），UTF-8。
- 进度事件：Rust → 前端 event 推送（阶段/子步骤/进度百分比/日志行）。

### 7.2 模块职责

- commands/：薄封装，只做参数校验与调用 core。
- core/pipeline：状态机核心，阶段节点注册表（StageId → handler），产物 manifest 管理。
- core/llm：OpenAI 兼容客户端；结构化输出（response_format 或引导 + 解析 + Schema 校验 + 重试）。
- core/codegen：注入参考上下文（分片控制 token 预算）、调用 LLM、必生成项断言、产物落盘。
- core/dst：目录校验、脚本索引、日志解析（error 行提取）。
- core/image：ImageAdapter trait（OpenAI 兼容实现 + 预留 SD/ComfyUI）；MVP 仅有桩（返回未接入）。

### 7.3 进程模型

- Rust 负责 Python 生命周期（spawn/terminate/超时）；请求带 request_id 关联响应；取消 = terminate + 清理。
- Python 启动前执行版本与依赖就绪探测（python --version、import 检查）。
- 所有跨进程文本 UTF-8。

### 7.4 数据流与 project.json

- 用户输入 → project.json（配置快照）→ 各阶段产物写入 → manifest.json。
- project.json 示例（骨架）：

```json
{
  "schema_version": 1,
  "mod": { "name": "wanda_cn", "author": "player", "version": "1.0.0", "description": "", "tags": [] },
  "dst_dir": "D:/Steam/steamapps/common/Don't Starve Together",
  "api": { "llm": { "base_url": "", "model": "" }, "image": { "base_url": "", "model": "" } },
  "pipeline": {
    "stage1": { "status": "success", "artifact_version": "character_v3", "input_hash": "..." },
    "stage5": { "status": "pending", "artifact_version": null }
  }
}
```

### 7.5 人设 JSON Schema（关键字段与校验规则）

- 必填：char_name（小写 snake，格式白名单校验）、display_name、stats、passive_skills、active_skills、starter_items、speech、pros_cons。
- stats：health/hunger/sanity，整数 50~300（建议默认 100~200），超出范围视为校验失败。
- passive_skills：id ∈ 规则库白名单（如 hunger 变化、san 光环、背包加成等），params 为数值字典。
- active_skills：id、实现载体（专属物品 item id）、cooldown；item 名 ∈ 原生 prefab 白名单或本项目自定义（自动生成 prefab）。
- starter_items：数组，最多 6 项，名称 ∈ 白名单或自定义。
- speech：键 ∈ DST 台词键白名单（ANNOUNCE / DESCRIBE / ACTIONFAIL 子集），值为中文（生成时配中文/英文语言包）。
- 校验失败 → 将校验错误回传 LLM 修正（最多 3 次）；仍失败转人工编辑。

```json
{
  "char_name": "wanda_cn",
  "stats": { "health": 150, "hunger": 150, "sanity": 150 },
  "passive_skills": [ { "id": "night_vision", "params": { "radius": 4 } } ],
  "active_skills": [ { "id": "spell_gift", "item": "wanda_gift", "cooldown": 120 } ],
  "starter_items": [ "axe", "torch", "wanda_gift" ],
  "speech": { "ANNOUNCE_ACCOMPLISHMENT": "这是我的新成就！" },
  "pros_cons": { "pros": ["移动快"], "cons": ["怕黑"] }
}
```

### 7.6 ESC 部件-尺寸-命名对照

- 唯一事实源：内置 templates/esc 的 scml 中实际引用的纹理名；构建时 escmapper 解析并输出"真实对照表"到项目报告。
- 部件类别：
  - build 部件（动画面片）：head、body、arm_left、arm_right、leg_left、leg_right。
  - ghost 变体：视模板而定（如 ghost_head 等），供死亡幽灵状态使用。
  - 头像部件：bigportrait（全身立绘）、smallportrait（小头像）、avatar（头像图）。
  - 图标：item_icon（专属道具图标）、modicon（创意工坊图标，可选）。
- 尺寸：全部 2 的幂（POT），具体数值以模板为准；宽高比与模板不符 → 等比缩放 + 透明留白居中（锚点不变），报告标记"待人工复核"。
- 命名：snake_case，与 scml 引用一致；编译前做一致性校验，缺失/多余文件名直接报错列出。

### 7.7 API 适配层

- LLMAdapter：OpenAI 兼容 chat/completions；trait 定义；支持 response_format=json_object 探测，失败降级引导式解析。
- ImageAdapter（trait，预留扩展）：
  - OpenAIImageAdapter：images/generations（文生图）；图生图用参考图（URL/base64）作为引导输入。
  - seed 支持探测：请求带 seed 参数并校验响应；不支持 → 降级 = 固定风格前缀 + 固定负面词 + 参考图一致。
  - 预留 StableDiffusionWebUIAdapter / ComfyUIAdapter 接口位（V2）。
- 通用：超时、重试（指数退避）、取消；错误分类（网络/鉴权/限流/模型不存在）映射为 GUI 提示。

### 7.8 代码生成规则（S3）

- 参考上下文：白名单读取 scripts/prefabs（wilson 等关键人物）、scripts/components（白名单）、scripts/strings（关键键）；注入前分片，控制单次 token 预算；用 cache 索引避免重复全量读取。
- 必生成文件（生成后断言存在）：
  - modinfo.lua：DST 规范字段（api_version 取规则库常量、dst_compatible = true、dont_starve_compatible = false、all_clients_require_mod = true、icon 字段）。
  - modmain.lua：注册 prefab/strings/语言包；服务器侧守卫。
  - prefabs/<char>.lua：三围、被动/主动技能、开局物品、专属道具注册。
  - prefabs/<item>.lua：专属道具（如有）。
  - speech_<char>.lua：台词表（ANNOUNCE/DESCRIBE/ACTIONFAIL 子集）。
  - strings/schinese.lua 等：可选本地化。
  - modicon.png：可选生成。
- 代码规范固化项：
  - 服务器逻辑：if not TheWorld.ismastersim then return end 守卫。
  - 客户端逻辑：访问 ThePlayer 前判空。
  - 不污染全局作用域；V1 前避免自定义复杂 RPC。
  - 组件/方法名使用 DST 已知白名单，规则库版本化（跟随 DST 更新）。
- 输出校验：必生成项存在 + Lua 语法检查（luac -p 若可用）+ 关键模式断言；失败 → 错误 + 上下文回传 LLM 修复（8.3 循环）。

## 8. 错误处理与人工兜底

### 8.1 全局错误处理

- API 调用失败：GUI 弹窗提示 + 保存当前项目 + 允许重试当前步骤（阶段断点续跑）。
- 阶段产物校验失败：进入该阶段修复模式，不动其他阶段产物。
- DST 目录校验失败：禁止继续生成 mod，提示路径问题并列出缺失项。

### 8.2 图像失败退回流程

- 部件分割失败（rembg/SAM2）：提供 3 个选项：
  1. 用原图重新生成该部件（LLM 收到失败原因调整提示词）；
  2. 跳过 SAM2，手动上传部件图（仍强制模板尺寸/命名校验）；
  3. 模板占位合成（原图直接贴到模板对应位置，报告标记待人工复核）。
- 批量多部件失败：支持整体重试或逐部件处理。

### 8.3 Lua 修复循环

- 输入：日志报错块（含文件/行号）+ 相关源码文件 + 规则库摘要。
- LLM 输出修复补丁（diff 或整文件），应用后标记"待测"。
- 每轮带上次修复结果；最多 3 轮；仍失败保留现场（日志 + 源码副本）供人工排查。
- 修复仅触碰 Lua，不涉及图像/骨骼。

### 8.4 DST 日志定位

- Windows 默认路径（递归探测 + 手动选择兜底）：
  - %USERPROFILE%\\Documents\\Klei\\DoNotStarveTogether\\client_log.txt
  - %USERPROFILE%\\Documents\\Klei\\DoNotStarveTogether\\server_log.txt
- 新版本可能按 KleiUserID/分支子目录存放：递归查找最近修改的 client_log/server_log。
- 读取策略：只读、文件锁捕获重试、UTF-8/GBK 编码探测。

## 9. 安全与隐私

- 所有 API 请求直连用户配置地址，不中转第三方服务器。
- API Key 仅本地存储（可选系统凭据管理器）；不上传项目文件、不上传 Key。
- 自动更新只从用户配置的 GitHub 仓库拉取。
- 生成的 Mod 内容与发布责任由用户自行承担。

## 10. 版本更新机制

- 启动时（受开关控制）请求 GitHub Releases API（仓库地址来自配置），语义化版本比较。
- 发现新版本 → 提示 → 下载 zip 到临时目录 → 校验（版本/可选哈希）→ 生成 updater 计划。
- 替换流程：退出主程序 → updater（独立小进程）移动新版本文件 → 重启主程序；失败自动回滚旧版本（旧包保留）。
- 更新期间不写入 mods 目录，不影响用户 Mod 项目数据。

## 11. 内置预制资源与许可说明

- ESC 模板：来源标注（社区 Extended Sample Character 模板），随包附带前确认其再分发许可；工具只生成副本并替换贴图，不修改骨骼关键帧。
- Klei Mod Tools（autocompiler.exe）：由用户自行安装（Steam: Don't Starve Mod Tools）并配置路径，工具不捆绑分发。
- rembg / SAM2 权重：首次使用时下载至本地模型目录（可配置镜像或离线导入）。

## 12. 硬性限制（固化，不可突破）

1. 不自动生成/编辑 Spriter 骨骼关键帧动画，仅复用 ESC 模板骨骼动作。
2. 图像分割存在失败概率：工具必须检测异常并走 8.2 兜底，不承诺全自动成功。
3. 不做流程外的自定义 LLM 工作流编排（ComfyUI 工作流由 V2 适配器按固定预设调用）。
4. 不收集/上传用户数据（第 9 节）。
5. ModTools 与 DST 游戏本体不随工具分发。

（代码中相应位置需加注释标记上述限制，防止后续实现越界。）

## 13. 里程碑与 Roadmap

| 里程碑 | 内容 | 验收 |
|---|---|---|
| M0 设计 | 本文档 | 评审通过 |
| M1 MVP | Tab1 配置/校验/API 测试；Tab2 项目 + 人设对话 + 定稿；Tab3 状态机 + Stage1 + Stage5；Tab4 日志 + 控制台命令 | 用示例人设生成基础 mod 代码，放入游戏无基础报错 |
| M2 V1 | 图像线（提示词/生成/抠图/分割/对齐）；Stage6 资源编译；一键修复闭环；更新全量 | 生成含贴图/动画的可运行 mod |
| M3 V2 | SD WebUI/ComfyUI 适配、多角色、创意工坊发布辅助 | 内部验收通过 |

## 14. 验收标准（MVP）

- Win10/11 上按 README 指引从源码编译运行。
- DST 目录校验：错误路径提示、正确路径通过；LLM API 测试连通。
- 完整走通：建项目 → 传图 → 对话 → 定稿 → 生成代码 → 导出 mod 文件夹。
- 生成 mod 放入 DST mods 后无基础 Lua 报错（日志无 error）。
- Tab4 能定位并读取日志、高亮错误；控制台命令可复制。