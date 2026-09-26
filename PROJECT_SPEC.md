# DST-Mod-Agent-Generator 项目设计文档（v2.0 实现对齐版）

- 版本：v2.0
- 日期：2026-09-26
- 状态：**M1 MVP 已实现并编译/打包验证通过**，源码已推送 GitHub
- 仓库：<https://github.com/qqxyfb/DST-Mod-Agent-Generator>（分支 main）
- 说明：本文档由 v1.1「M0 设计定稿版」升级而来。v1.1 中属于"规划态"的内容已按**实际落地实现**逐条校正（UI 结构、目录结构、project.json 结构、部件规格、环境检测、阶段状态机等），并补充了多轮迭代中产生的决策与变更记录。**凡与实现不一致的规划内容，本文以实现为准重新表述；尚未实现的部分统一标注为"V1 预留"。**

## 目录

1. 版本与变更记录
2. 文档评审结论（问题清单 + 落地状态）
3. 可行性分析
4. 产品目标与范围
5. 需求规格（实现对齐）
6. 流水线设计（7 阶段 + 双管线 + 可重跑）
7. 项目目录结构（实现对齐）
8. 架构与核心规格
9. 错误处理与人工兜底
10. 安全与隐私
11. 版本更新机制
12. 内置预制资源与许可
13. 硬性限制（固化，不可突破）
14. 里程碑与 Roadmap
15. 验收标准
16. 构建、打包与日志
17. V1 待办清单

## 1. 版本与变更记录

### 1.1 文档版本

| 版本 | 日期 | 状态 | 说明 |
|---|---|---|---|
| v1.0 | 2026-09-23 | 原始需求 | 用户提交的需求文档 |
| v1.1 | 2026-09-23 | M0 设计定稿 | 合并评审问题清单、可行性结论、两项流程补充需求 |
| **v2.0** | **2026-09-26** | **实现对齐** | 按已落地的 MVP 实现重写；新增变更记录、打包/日志章节、V1 待办清单 |

### 1.2 迭代变更记录（按提交顺序）

| # | 提交 | 变更 |
|---|---|---|
| 1 | `6f2adc0` | MVP 首版：Tauri 2 + Vue 3 骨架、配置/项目/人设对话/双管线状态机/日志调试 |
| 2 | `2e93ba2` | README 补充打包指南 |
| 3 | `93c11e2` | python/requirements*.txt 增加 PEP263 编码声明（修复 zh-CN Windows 下 pip GBK 报错） |
| 4 | `6d3f9c4` | 新增根目录 `build.bat` 一键打包脚本 |
| 5 | `eb28231` | 修复 build.bat 提权时空参数导致的参数绑定异常 |
| 6 | `59f25b8` | 修复 MSI 打包图标空数组问题；新增 `log/` 目录记录运行日志与打包日志 |
| 7 | `410029b` | 环境面板安装结果反馈（成功/失败原因回显） |
| 8 | `dcd46ac` | autocompiler 自动探测（Steam 注册表 + libraryfolders.vdf）；依赖按功能组安装 |
| 9 | `18de2e2` | API 连通性测试语义化报错；图像接口走真实 `images/generations` |
| 10 | `f7d7656` | Tauri 参数改 camelCase，修复 `import_reference` 缺 `projectPath` 报错 |
| 11 | `afb297f` | 项目切换回填；创建项目改为独立按钮；输入框不再横向溢出 |
| 12 | `83f6d6e` | **全局配置页移除**改为右上角设置弹窗；Tab2 标签改多选；对话区铺满；新增「一键生成人物概设」 |
| 13 | `b32dae9` | 标签改为折叠下拉多选（搜索/全选/卡内滚动）；阶段重跑支持提示词定向调整 |
| 14 | `7f08fe4` | 人设对话持久化到 `logs/chat.jsonl` 并自动回填；消除双滚动条、初始化按钮常驻可见 |
| 15 | `92645f3` | 一键初始化后对话区滚动到概设开头（长文本不跳结尾） |

### 1.3 本版相对 v1.1 的主要差异

1. **UI 结构变了**：v1.1 的「Tab1 全局配置」已取消，改为**右上角 `⚙ 设置` 弹窗**；主界面收敛为 3 个 Tab。
2. **滚动模型明确**：主容器 `overflow:hidden`，各视图内部独立滚动 —— 避免双滚动条，窗口默认 `1280x900`（最小 `1024x640`）。
3. **对话持久化落地**：人设对话写入 `<项目>/logs/chat.jsonl` 并自动回填，解决"每次打开程序都要重新生成"。
4. **阶段重跑支持提示词**：阶段状态新增 `hint` 字段随项目保存，"重跑"可带定向调整要求。
5. **project.json 结构定型**：以 `ProjectInfo` 为准（不是 v1.1 §7.4 的 `schema_version/mod/dst_dir/api` 草案结构）。
6. **环境面板按功能组就绪检测**：Python / 基础依赖（Pillow、numpy）/ rembg / SAM2 / autocompiler.exe / ESC 模板 共 6 项。
7. **不落地项标注**：`templates/esc/*.scml` 目前仍为占位 README，图像线（Stage2~4）与资源编译（Stage6）为 V1 预留。

## 2. 文档评审结论（问题清单 + 落地状态）

### 2.1 总体评价

原始需求结构完整、目标明确，优点：

- 面向普通用户的一站式体验，Tab 划分清晰，MVP 范围（配置、人设对话、代码生成）合理。
- 明确固化"不自动生成骨骼动画"等边界，防止实现阶段无限扩展。
- 错误处理规则具体（重试、跳过、LLM 修复），API 直连用户配置地址，安全约束好。
- 预留图像模块与编译模块接口，符合渐进式开发。

主要不足集中在技术细节缺定义（部件清单、尺寸命名、日志路径、ModTools 来源）、流程模型与"并行/重跑"补充需求冲突、若干实现层坑（Windows 文件锁、seed 兼容、mods 目录写入权限）。这些在 v1.1 已给出修订决策，v2.0 记录其落地状态。

### 2.2 评审问题清单与落地状态

级别说明：P0 = 必须修订（影响正确性或必然失败）；P1 = 影响体验/健壮性；P2 = 优化。

| # | 级别 | 问题 | 修订决策 | 落地状态 |
|---|---|---|---|---|
| C1 | P0 | 部件清单自相矛盾（9 项却写"8 个"），且缺 ghost/portrait/modicon | 以内置模板 scml 实际引用名为唯一事实源，工具解析校验 | **V1**：清单已在 §6.7 与 `templates/esc/README.md` 固化，实际解析待模板入包 |
| C2 | P0 | "resize 到 2 的幂"未定义精确尺寸与锚点，粗暴拉伸破坏比例 | 增加部件-尺寸-命名对照；等比缩放 + 透明留白居中，标记待人工复核 | **V1**：规则已写入 §6.7 |
| C3 | P0 | 自动写入 DST mods 目录：游戏运行中文件被锁 / 无权限 | 默认输出到项目 `mod_compiled/` 导出缓存；写入 DST 前检测进程与锁 | **V1**：目录已预留（`mod_compiled/`） |
| C4 | P0 | 自动更新"自动替换程序文件"：运行中 exe 无法覆盖 | 下载到临时目录 + 退出后替换 + 重启，失败回滚 | **已落地到设计**：见 §11；当前 `check_update` 只做版本比对与提示 |
| C5 | P0 | autocompiler 来源与授权未定义 | 用户经 Steam 自装，工具不捆绑；ESC 模板需注明来源与许可 | **已落地**：`detect_modtools` 自动探测 + 手动选路径；`templates/esc/README.md` 已注明 |
| C6 | P0 | 人设 JSON 无 Schema，LLM 可能输出非法数值 | 固化 Schema + 数值范围 + 格式白名单，失败带错误重试 | **已落地**：`core/schema.rs` 实现校验（§8.5） |
| C7 | P1 | "固定 seed 保证画风统一"对不支持 seed 的 API 无效 | 区分支持/不支持 seed，不支持时降级为固定风格前缀 + 负面词 + 参考图一致 | **V1**：规则保留在 §6.7 / §17 |
| C8 | P1 | DST 日志路径未定义，读取方式未说明 | 明确默认路径 + 递归探测 + 手动选择兜底 | **已落地**：`core/logreader.rs` 递归扫描 `Documents/Klei` |
| C9 | P1 | "一键安装依赖自动拉取 Python/rembg/SAM2"范围含糊 | 按功能组拆分，每组独立就绪检测；MVP 仅装基础依赖 | **已落地**：`python_env_status` / `python_env_setup(group)` |
| C10 | P1 | "读取 scripts 原版代码作参考"无粒度定义，全量注入超 token 预算 | 白名单范围 + 分片控制预算 | **已落地**：`core/dst.rs` 9 文件白名单，单文件上限 60000 字符、总上限 8000 |
| C11 | P1 | Tab3 有 7 个 Skill，内部 Skill 规则只列 5 个，两套编号不一致 | 统一为 7 阶段（前端卡片）+ 5 内部 Skill 域的映射 | **已落地**：见 §6.3 |
| C12 | P1 | 原"按顺序执行"与补充"可并行切换、单独重跑"冲突 | 视觉线/代码线双管线 + 阶段状态机 | **已落地**：`core/pipeline.rs` + `StageCard.vue` |
| C13 | P2 | 未提中文语言文件（schinese）与 modicon | 代码生成增加可选语言包与 modicon | **部分**：`speech_<char>.lua` 已生成；schinese/modicon 列 V1 |
| C14 | P2 | bigportrait/smallportrait/avatar 语义未区分 | 提示词阶段分别给规格 | **V1**（提示词阶段未接入） |
| C15 | P2 | "打包 anim zip 动画包"表述不准确（运行时不需要 zip） | 修订为输出 anim/（bin+tex）；创意工坊 zip 列入 V1 | **已修订**：见 §6.7、§17 |
| C16 | P2 | Stage1"人设定稿校验"与 Tab2"人设对话定稿"职责重叠 | 合并：Tab2 产出定稿；Stage1 只做校验 + 冻结版本 | **已落地**：`pipeline::stage1` 有草稿则冻结、有定稿则复校 |

### 2.3 用户补充需求（已固化）

- 补充 1：制作流程分阶段展示为卡片；流程上模型（视觉）与代码逻辑阶段可并行、可切换 → **已落地**（§6）。
- 补充 2：AI 结果不可能一次达标，需提供特定阶段重新生成/修改能力 → **已落地并增强**：支持阶段级重跑 + **重跑提示词**（定向调整生成方向，`StageState.hint`）。
- 补充 3（迭代新增）：设置项从占位 Tab 改为右上角弹窗；创意工坊标签改多选折叠下拉；一键初始化生成人物概设；对话持久化。→ **已落地**（§5）。

## 3. 可行性分析

### 3.1 总体结论

**可行，且 MVP 已验证。** 技术栈成熟（Tauri 2 + Vue 3 + Rust 主程序 + Python 子进程）；DST 人物 mod 本质是"Lua 脚本 + Spriter 骨骼（复用 ESC 模板）+ TEX 贴图"，各环节均有成熟本地工具链。主要不确定性集中在图像自动化管线（AI 出图 → 抠图 → 分割 → 模板对齐），必须设计人工兜底，不承诺"一次全自动"。

### 3.2 分模块可行性

| 模块 | 方案 | 可行性 | 关键风险 | 现状 |
|---|---|---|---|---|
| GUI | Tauri 2 + Vue 3，Windows 优先，NSIS/MSI 打包 | 高 | 依赖 WebView2（Win10/11 自带） | 已完成 |
| 后端主程序 | Rust（Tauri commands），配置/项目/状态机 | 高 | 无 | 已完成 |
| Python 子进程桥 | `std::process::Command` + 就绪探测 | 高 | 依赖检测与 UTF-8 编码 | 检测已实现；JSONL 图像桥 V1 |
| LLM 接入 | OpenAI 兼容 chat/completions（ollama/deepseek/qwen） | 高 | 结构化输出可靠度 → Schema 校验兜底 | 已完成 |
| 图像生成 | OpenAI 兼容 images API；SD WebUI/ComfyUI 适配器（预留） | 中高 | seed 兼容性、部件一致性 | 连通性测试已实现；批量生成 V1 |
| 抠图 rembg | `pip install rembg` + onnxruntime 本地推理，CPU 可跑 | 中高 | 出图质量差时失败，走人工兜底 | V1（安装组已预留） |
| 分割 SAM2 | pip 包 + 首次下载权重（数百 MB，可选 GPU） | 中 | 权重获取、分割失败率，可跳过/手动上传 | V1（安装组已预留） |
| 代码生成 | 确定性模板 + LLM 填充 + 原版参考注入 + 必生成项断言 | 中高 | 规范符合性，用断言 + 日志闭环修复 | 已完成（Stage5） |
| 资源编译 | 用户自装 Klei Mod Tools，调用 autocompiler.exe | 中高 | 工具路径、scml 版本兼容 | V1（探测已完成） |
| 自动更新 | GitHub Release API + zip + 重启替换 | 高 | Windows 文件锁（已设计规避） | 版本比对已实现 |

### 3.3 环境依赖"能否自动安装"的分析结论

这是迭代过程中用户明确提出过的问题，结论固化如下：

| 组件 | 能否自动安装 | 结论与依据 | 工具当前行为 |
|---|---|---|---|
| rembg 抠图 | **可以** | 纯 pip 依赖（`rembg` + `onnxruntime`），无外部安装器、无许可限制；模型权重在首次推理时自动下载（受网络影响，可配镜像或离线导入） | 安装分组 `rembg` 已预留，MVP 未启用（`python_env_setup(group="rembg")`） |
| SAM2 分割 | **可以（但权重需下载）** | 包本身可 `pip install sam2`（或 git+github 源）；真正的不确定性是权重文件（数百 MB，官方托管在 Meta 的 CDN），国内网络可能失败。因此必须提供"镜像 / 手动放置权重 / 离线包"三条路 | 安装分组 `sam2` 已预留，MVP 未启用 |
| autocompiler.exe | **不可以** | Klei Mod Tools 是 Steam 上独立分发的应用（AppID 322330 相关工具包），没有官方公开直链；第三方镜像分发存在许可与版本风险 | 不做自动安装，改为**自动探测**：Steam 注册表安装路径 + `libraryfolders.vdf` 扫描全部库；失败则引导用户手动选择路径 |
| ESC 模板 | **可以（随包附带）** | 属文件分发而非安装；前提是先确认再分发许可 | `templates/esc/` 已建占位 + 许可说明；实际 scml 于 V1 入包 |

> 设计取舍：**能装的一定能自动装（rembg/SAM2），不能装的不做规避性自动化（autocompiler）**。拒绝捆绑 Klei 工具既是许可要求，也避免版本错配导致编译产物异常。

### 3.4 关键风险与缓解

| 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|
| 图像分割 / 模板对齐失败 | 高 | 中 | 模板占位合成兜底、部件级重跑、手动上传替换、失败信息反馈 LLM 重新生成 |
| LLM 生成 Lua 不合 DST 规范 | 中 | 中 | 原版参考注入、必生成项断言、游戏日志闭环自动修复 |
| 环境缺失（ModTools 未装等） | 中 | 中 | 就绪检测 + 引导配置，缺失时禁止进入对应阶段 |
| DST 版本更新导致接口变化 | 中 | 低 | 每次生成前读原版脚本作参考，规则库版本化 |
| 长任务阻塞 UI | 中 | 中 | 重活走异步 command，进度经 `pipeline://progress` 事件推送 |
| Windows 中文路径 / 非 ASCII | 低 | 中 | 全程 UTF-8（含 pip 依赖文件的 PEP263 声明） |

### 3.5 工作量参考（单人估算，非承诺）

- MVP（M1）：配置 + 人设对话 + 代码生成 + 状态机骨架 + 日志读取：约 2~3 周（**实际已完成**）。
- V1（M2）：图像线 + 资源编译 + 一键修复闭环 + 更新全量：累计约 4~6 周。
- 不含图像质量调优的反复迭代时间。

## 4. 产品目标与范围

### 4.1 目标用户与价值

普通玩家无需懂 Lua、无需手动配置 ComfyUI/SAM2，上传参考图 + 文字描述即可生成 DST 人物 Mod；全程本地生成，API 直连用户自配服务。

### 4.2 名词定义

- **阶段（Stage）**：流程中的一个可执行节点，有明确的输入/输出产物，共 7 个（stage1~stage7）。
- **产物**：人设 JSON vN、部件提示词集、部件 PNG、mod 源码、编译后的 mod 文件夹。
- **视觉线 / 代码线**：Tab「资源生成」中两条 lane，共享 Stage1 定稿人设。
- **定稿（frozen）**：用户确认后冻结的人设 JSON 版本，后续生成/编译都以它为输入快照。
- **部件**：head/body 等可单独重生成的图像单元。
- **模板**：内置 ESC（Extended Sample Character）scml 骨骼模板，只替换贴图、不生成骨骼。
- **重跑提示词（hint）**：用户在重跑阶段时填写的自然语言调整要求，追加进 LLM 提示词并随项目保存。

### 4.3 版本范围

| 版本 | 范围 | 状态 |
|---|---|---|
| MVP（M1） | 右上角设置弹窗（DST 目录校验、LLM/图像 API 配置与连通测试、Python 基础依赖一键安装、更新设置）；项目创建 + 人设对话 + 一键初始化概设 + 定稿；资源生成页阶段卡片状态机 + Stage1 校验 + Stage5 代码生成（图像线显示"V1 未接入"并标记 skipped）；测试与调试页日志读取 + 一键修复 + 控制台命令 | **已完成并打包验证** |
| V1（M2） | 图像线（提示词/生成/抠图/分割/模板对齐/命名）、Stage6 资源编译、ESC 模板实际入包、schinese 语言包与 modicon、自动更新全量 | 未开始 |
| V2（M3） | SD WebUI / ComfyUI 适配器、多角色批量、创意工坊发布辅助 | 未开始 |

## 5. 需求规格（实现对齐）

> 与 v1.1 的关键差异：**不再有"Tab1 全局配置页"**。全局配置收敛为右上角 `⚙ 设置` 弹窗；主界面 3 个 Tab。
> 全局滚动模型：主容器 `overflow:hidden`，由各视图内部滚动条承担滚动（`.view-scroll`，项目页左列独立滚动），保证任意分辨率下**只有一条滚动条**。

### 5.1 右上角「设置」弹窗（原 Tab1 全局配置）

入口：标题栏右上角 `⚙ 设置` 按钮，点击弹出模态（`.modal-mask` / `.modal-panel`），内容复用 `ConfigView.vue`。分为 4 组：

#### 5.1.1 DST 游戏目录配置

- 文件选择框选择 Don't Starve Together 根目录（`tauri-plugin-dialog`）。
- 校验（`validate_dst_dir`）：目录必须存在且包含 `mods/` 与 `scripts/`；失败返回 `errors[]` 并在界面提示，同时阻止进入生成流程。
- 校验通过后回显 `scripts_count` 与命中白名单的样例行，作为"参考上下文已就绪"的证据。
- 参考上下文：按白名单读取 `scripts/`（见 §8.6），单文件与总量均有预算上限。

#### 5.1.2 API 配置面板

- **LLM API**：Base URL / API Key / 模型名称，OpenAI 兼容 `chat/completions`（ollama、deepseek、qwen 等）。
- **图像 API**：Base URL / Key / 模型名称，OpenAI 兼容图像接口。
- **测试按钮**（`llm_test` / `image_test`）：
  - `llm_test` 发最小 chat 请求；`image_test` 走**真实** `images/generations` 请求（不是桩）。
  - 错误**分类**提示，而非裸抛：网络不可达 / DNS 失败 / 鉴权失败（401/403）/ 限流（429）/ 模型不存在（404）/ 服务端错误（5xx）/ 响应体非预期 JSON。
  - 排障约定：报错信息中同时回显 **HTTP 状态码 + 请求 URL + 响应体前 200 字符**，便于用户对照服务商文档自查。
    > 实践提示：部分服务商的"连通性失败"并非网络问题，而是**请求体字段与该服务商约定不一致**。例如样例 `{"model","prompt","size","ratio"}` 说明其图像接口使用 `size` + `ratio`（而非 OpenAI 标准的 `size: "1024x1024"`）。图像测试请求应允许携带该服务商的扩展字段，并用其返回的错误体定位原因。
- API Key 本地存储，不上传（§10）。

#### 5.1.3 工具环境管理面板（6 项，按功能组就绪检测）

| 组件 | 检测方式 | 就绪判据 | MVP 行为 |
|---|---|---|---|
| Python 环境 | 查找 `python` / `py -3` | 命令可执行且版本可解析 | 就绪则显示版本 |
| 基础依赖（Pillow / numpy） | `python -c "import PIL,numpy"` | 两者均可导入 | **支持一键安装**（分组 `base`） |
| rembg 抠图 | 同上探测 `import rembg` | 可导入 | 未安装则提示"V1 一键安装自动安装" |
| SAM2 分割 | 同上探测 | 可导入且权重存在 | 标注"V1" |
| autocompiler.exe | 配置路径 → Steam 注册表 → `libraryfolders.vdf` 全库扫描 | 文件存在 | 用户自装；支持自动探测与手动选择 |
| ESC 模板 | `templates/esc/` 目录 + README | 目录与说明存在 | 随包附带（实际 scml 待 V1） |

- 一键安装按功能组：`python_env_setup(group)`，`group ∈ {base, rembg, sam2}`；MVP 界面只暴露 `base`。
- 安装结果**必须回显**成功/失败原因（含 pip 原始报错尾部），避免"提示安装完成但刷新仍显示缺失"。
- 依赖清单：`python/requirements.txt`（Pillow、numpy，MVP）与 `python/requirements-image.txt`（rembg、onnxruntime、sam2，V1）。两个 files 首行均含 `# -*- coding: utf-8 -*-`，修复 zh-CN Windows 下 pip 读取 GBK 报错。

#### 5.1.4 版本更新设置

- 开关「自动更新」+ GitHub 仓库地址输入框 + 「立即检查」按钮（`check_update`）。
- 版本常量：`CURRENT_VERSION = "0.1.0"`；与 GitHub Releases 的 `tag_name` 做语义化比较。
- 当前仅实现"比对 + 提示"；下载替换流程见 §11（V1 落地）。

### 5.2 Tab「项目创建与人设对话」

布局：左列（表单，可独立滚动）+ 右列（对话，铺满剩余空间）。

- **项目切换与按钮**：顶部项目下拉（`list_projects` / `open_project`）+ 三个独立按钮「新建项目 / 创建项目 / 保存修改」。
  - 切换已有项目时**必须回填** Mod 信息、参考图列表、角色描述与历史对话（修复"重新选择项目无法回填"）。
- **Mod 基础信息**：Mod 名称、作者、版本、简介。
- **创意工坊标签**：折叠下拉多选（`WORKSHOP_TAGS` 27 个标签 + 中文映射），支持搜索、全选、清空、卡片内滚动（避免整页被拉高）、自定义回车添加；结果写入 modinfo 的 `server_filter_tags`。
- **参考图**：1~N 张，经 `import_reference(projectPath, filePath)` 复制进项目 `reference/`；参数名统一 **camelCase**（历史 bug：下划线命名导致 `missing required key projectPath`）。
- **角色描述**：人设 / 性格 / 技能想法自由文本；输入框**不得横向溢出卡片**（`min-width:0` + `overflow-x:hidden`）。
- **一键生成人物概设（初始化）**：按钮常驻左列底部（sticky，滚动时始终可见）。点击后以固定 prompt 组合"参考图 + 角色描述 + Mod 基础信息"发起**第一次 agent 请求**（`agent_init`），产出人物特点作为对话首条内容。
  - 触发后右列对话区自动滚动到**概设开头**（长文本不跳结尾）；首条消息闪烁高亮并配 `chatNotice` 提示条，让用户明确知道已触发。
  - 已有对话时再次点击会弹 `window.confirm`，确认后清空 `chat.jsonl` 重开一轮。
- **对话 Agent**：多轮迭代修改设定；Agent 输出完整角色设定文档（三围、被动/主动技能、开局物品、专属道具、台词、优缺点）。超长输出模板以 `<details>` 折叠展示。
- **对话持久化（本轮新增）**：每轮写入 `<项目>/logs/chat.jsonl`，每行 `{role, content, ts}`。
  - 打开/切换项目自动回填（`load_chat_history`），并提示"已恢复 N 条历史对话"。
  - 读取时逐行解析、坏行跳过、只保留最近 200 条，保证损坏文件不阻塞 UI。
  - `reset_chat_log()` 由 `agent_init` 调用，实现"重开一轮"。
- **定稿**：`confirm_character` → Schema 校验 → 冻结人设为版本 vN（写入 `characters/`，含 JSON 与 Markdown）；后续基于新对话再生成新版本，不覆盖旧版本。

### 5.3 Tab「资源生成」

- 顶部：阶段卡片区（`StageCard.vue`），按 lane 分栏展示（视觉线 / 代码线），每卡含编号、名称、状态、操作按钮。
- 中部：当前阶段进度条（`pipeline://progress` 事件流：message + percent）。
- 下部：阶段日志、Mod 报告（`get_report`）。
- **重跑提示词**：阶段卡片提供输入框 + 快捷预设词；`start_stage / rerun_stage` 接受 `hint`。
  - 非空 → 覆盖保存到 `StageState.hint` 并写日志；为空 → 复用该阶段上次保存的 hint（点「重跑」即复用）。
  - 生效范围：**Stage5 代码生成实际生效**（作为最高优先级要求追加进每条 LLM 提示词）；Stage2 待 V1。
- 全局操作：开始/继续、重跑、跳过（标记风险）、查看日志；V1 预留：部件级重生成、手动覆盖产物、暂停/中断。

### 5.4 Tab「测试与调试」

- **日志读取**（`locate_logs` / `read_log`）：递归扫描 `%USERPROFILE%\Documents\Klei\**`，匹配含 `log` 的 `.txt`，按修改时间倒序取最近 10 个；读取后高亮 `error_indices`（错误行）。
- **一键修复**（`fix_from_log`）：日志报错块 + 当前 mod 源码 → LLM 输出修复结果 → 写回受影响的 Lua 文件 → 提示重新测试；多轮修复受上限约束（§9.3）。
- **控制台命令生成**（`gen_console_cmds`）：按当前定稿人设输出 `c_give`（开局物品 / 专属道具）与三围打印命令，一键复制到游戏控制台。

## 6. 流水线设计（7 阶段 + 双管线 + 可重跑）

### 6.1 全局原则

1. 每个阶段是独立可执行、可重跑、可跳过的节点，有输入快照与输出产物。
2. 重跑 = 基于当前输入快照重新执行该阶段，生成新版本产物；旧版本保留用于回滚。
3. 校验失败或用户不满意 → 只在该阶段内重试/局部重生成，不强制级联重跑上游。
4. 上游产物变更（如人设出新版本）→ 下游标记"需重跑"，提示用户确认，不自动级联。

### 6.2 阶段清单与双管线

```
                    ┌─ Stage2 部件提示词 ─ Stage3 部件图像生成 ─ Stage4 图像后处理 ─┐
Stage1 人设定稿校验 ─┤                                                                  ├─ Stage6 资源编译 → Stage7 Mod 报告
                    └─ Stage5 代码生成 ─────────────────────────────────────────────────┘
```

| 阶段 | 名称 | lane | MVP 实现状态 |
|---|---|---|---|
| stage1 | 人设定稿校验 | shared | 已实现（有草稿则冻结，有定稿则复校） |
| stage2 | 部件提示词 | visual | V1 预留：调用即标记 `skipped` 并记录原因与重跑提示词 |
| stage3 | 部件图像生成 | visual | V1 预留（同上） |
| stage4 | 图像后处理 | visual | V1 预留（同上） |
| stage5 | 代码生成 | code | 已实现（模板 + LLM + 必生成项断言） |
| stage6 | 资源编译 | code | V1 预留（同上） |
| stage7 | 输出 Mod 报告 | code | 已实现（收集 `mod/**/*.lua` + 测试指引） |

- stage1 与 stage7 归 `shared` / 收尾；stage2~4 视觉线，stage5~7 代码线。
- stage6 的前置是 stage4（贴图）与 stage5（源码）双齐备。
- V1 阶段的"执行"当前行为：写入 `status = "skipped"` + 日志说明，**不报错、不阻塞**，保证 MVP 流程可走通。

### 6.3 内部 Skill 映射（统一两套编号）

| 内部 Skill | 对应阶段 | 输入 | 输出 |
|---|---|---|---|
| 人设 Skill（S1） | Stage1（含 Tab「项目创建与人设对话」产出） | 图片 + 描述 + Mod 基础信息 | 结构化 JSON + Markdown 设定文档；用户确认定稿 |
| 图像生成 Skill（S2） | Stage2 ~ Stage4 | 人设 JSON vN + 参考图 | 全部部件 PNG（透明 RGBA、POT、模板命名） |
| 代码生成 Skill（S3） | Stage5 | 人设 JSON vN + 原版脚本参考 + 重跑提示词 | 全套 mod Lua 源码 + modinfo + strings |
| 资源编译 Skill（S4） | Stage6 | 部件 PNG + ESC 模板 + 源码 | 编译完成的 mod 文件夹（先导出缓存，可选写入 DST） |
| 调试修复 Skill（S5） | Stage7 / Tab 测试与调试 | 游戏日志 + mod 源码 | 修复后的 Lua 代码 |

### 6.4 阶段状态机

`StageState = { status, artifact_version, input_hash, log[], updated_at, hint }`，随 `project.json` 持久化。

| 状态 | 允许操作 |
|---|---|
| `pending`（未开始） | 开始 / 跳过 |
| `running`（进行中） | 查看实时日志（V1：暂停 / 中断） |
| `success`（成功） | 重跑（可带提示词）/ 查看产物 / V1：局部重生成、手动覆盖 |
| `failed`（失败） | 重试 / 跳过（标记风险）/ 查看日志 |
| `skipped`（已跳过） | 恢复执行（重新调用即可） |

- 卡片颜色与图标对应状态；当前运行阶段高亮并展示进度。
- 双线可并行：两 lane 的阶段各自独立调用，互不阻塞。

### 6.5 产物版本与增量重跑

- 项目内按产物类型保存：人设 `characters/`（vN），图像 `artwork/`（按部件），源码 `mod/`，编译产物 `mod_compiled/`。
- 人设版本冻结在 `character.version` / `character.frozen`；重跑 Stage1 时校验 `input_hash` 提示上游是否已变更。
- 部件级重生成与手动覆盖属 V1（见 §17）。

### 6.6 进度事件

`pipeline://progress` → `{ project, stage, message, percent }`。Rust 在每个阶段的关键节点 emit（如 stage5：读取参考 10% → modinfo 20% → modmain 35% → prefab 55% → speech 75% → 道具 88% → 校验 95% → 完成 100%）。

### 6.7 图像与编译规格（V1 落地，规则先行固化）

- **唯一事实源**：内置 `templates/esc` 的 scml 中实际引用的纹理名；由 escmapper 解析并输出"真实对照表"到项目报告。
- **部件类别**：
  - build 部件（动画面片）：head、body、arm_left、arm_right、leg_left、leg_right
  - ghost 变体：视模板而定（如 ghost_head），供死亡幽灵状态使用
  - 头像部件：bigportrait（全身立绘）、smallportrait（小头像）、avatar（头像图）
  - 图标：item_icon（专属道具图标）、modicon（创意工坊图标，可选）
- **尺寸**：全部 2 的幂（POT），具体数值以模板为准；宽高比与模板不符 → 等比缩放 + 透明留白居中（锚点不变），并标记"待人工复核"。
- **命名**：snake_case，与 scml 引用一致；编译前做一致性校验，缺失/多余文件名直接列错。
- **动画产物修订（C15）**：输出 `anim/`（bin + tex）；**DST 运行时不需要 zip**，zip 仅为创意工坊上传格式，列 V1。
- **seed 一致性（C7）**：优先请求 `seed` 并校验响应；不支持时降级为"固定风格前缀 + 固定负面词 + 参考图一致"。

## 7. 项目目录结构（实现对齐）

### 7.1 仓库顶层

```
DST-Mod-Agent-Generator/
├── README.md                     # 编译/运行/打包说明
├── PROJECT_SPEC.md               # 本文档
├── build.bat                     # 一键打包（提权 + 国内镜像，支持 -NoBundle / -Bundles / -SkipNpmInstall）
├── index.html / vite.config.ts / tsconfig.json / package.json
├── app-icon.png                  # 图标源图（由 scripts/generate-icons.ps1 生成多尺寸）
├── src-tauri/                    # Rust 主程序（Tauri 2）
│   ├── src/
│   │   ├── main.rs / lib.rs      # 入口 + command 注册（29 个）
│   │   ├── commands/             # Tauri command 层（薄封装）
│   │   │   ├── config.rs         # 配置 / 校验 / API 测试 / 环境 / 更新
│   │   │   ├── project.rs        # 项目 / 参考图 / 人设对话 / 定稿
│   │   │   ├── pipeline.rs       # 阶段状态机
│   │   │   └── debug.rs          # 日志 / 一键修复 / 控制台命令
│   │   └── core/
│   │       ├── config.rs         # AppConfig 读写（%APPDATA%）
│   │       ├── project.rs        # ProjectInfo / project.json / chat.jsonl
│   │       ├── pipeline.rs       # 阶段定义、执行、进度事件
│   │       ├── schema.rs         # CharacterSheet + 校验 + Markdown 渲染
│   │       ├── llm.rs            # OpenAI 兼容客户端 + extract_json
│   │       ├── codegen.rs        # Stage5：模板 + LLM 生成
│   │       ├── dst.rs            # 目录校验 / 白名单读取 / ModTools 探测
│   │       ├── logreader.rs      # DST 日志定位与错误行提取
│   │       ├── console.rs        # 控制台调试命令生成
│   │       ├── runtime_log.rs    # 运行时日志（log/runtime.log）
│   │       └── py/mod.rs         # Python 就绪探测 / 依赖安装
│   └── tauri.conf.json           # 窗口 1280x900、minWidth 1024、minHeight 640
├── src/                          # Vue3 前端
│   ├── App.vue                   # 顶栏（标题 + ⚙ 设置）+ 3 Tab + 设置模态
│   ├── views/
│   │   ├── ConfigView.vue        # 设置弹窗内容（DST / API / 环境 / 更新）
│   │   ├── ProjectView.vue       # 项目创建与人设对话
│   │   ├── PipelineView.vue      # 资源生成（阶段卡片）
│   │   └── DebugView.vue         # 测试与调试
│   ├── components/pipeline/StageCard.vue
│   ├── api/tauri.ts              # invoke 封装 + 事件订阅（camelCase 参数）
│   └── types/index.ts            # STAGES(7) / WORKSHOP_TAGS(27) / 类型定义
├── python/
│   ├── bridge.py                 # stdio JSONL 桥（V1 图像管线入口，含限制注释）
│   ├── requirements.txt          # MVP：Pillow、numpy
│   ├── requirements-image.txt    # V1：rembg、onnxruntime、sam2
│   └── README.md
├── templates/esc/                # ESC 模板（当前仅 README.md 占位 + 许可说明）
├── resources/
│   ├── schemas/character.schema.json   # 人设 JSON Schema
│   └── rules/codegen_rules.md          # DST 代码生成规则库（版本化）
├── scripts/
│   ├── build.ps1                 # 打包主体（被 build.bat 调用）
│   └── generate-icons.ps1        # 由 app-icon.png 生成各尺寸图标
├── log/                          # 运行日志 + 打包日志（已 gitignore，保留 .gitkeep）
├── cache/                        # 打包/运行缓存（已 gitignore）
├── projects/                     # 用户 Mod 项目默认输出目录（内容 gitignore）
├── dist/                         # 前端构建产物（已 gitignore）
└── node_modules/                 #（已 gitignore）
```

### 7.2 单个 Mod 项目文件夹

```
projects/<ModName>/
├── project.json      # ProjectInfo：meta + notes + character + pipeline + report
├── characters/       # 人设定稿 JSON（vN）+ Markdown 文档
├── reference/        # 用户上传参考图（原图副本）
├── artwork/          # 部件提示词 JSON、生成原图、后处理 PNG（V1）
├── mod/              # mod 源码：modinfo.lua / modmain.lua / prefabs/*.lua / speech_*.lua
├── mod_compiled/     # 编译产物导出缓存（V1）
└── logs/             # 各阶段日志、chat.jsonl（Agent 对话记录）
```

> `projects/` 内容默认被 gitignore（仅保留 `.gitkeep` 与 README），属用户数据，不随仓库分发。

## 8. 架构与核心规格

### 8.1 总体架构

- 前端 Vue 3（3 Tab + 设置弹窗）→ `invoke` 调用 Rust command。
- Rust 主进程：配置、项目、阶段状态机、LLM 调用、更新、日志解析。
- Python 子进程：就绪探测 + 依赖安装（MVP）；图像重活（V1），协议 = stdio JSONL（request/progress/result/cancel），UTF-8。
- 进度事件：Rust → 前端事件推送（阶段 / 消息 / 百分比）。

### 8.2 模块职责

- `commands/`：薄封装，只做参数校验与转发 core，并写 `runtime_log`。
- `core/pipeline`：状态机核心，阶段定义表 + 执行分派 + 进度 emit。
- `core/llm`：OpenAI 兼容客户端；`extract_json()` 从回复抽 JSON；失败降级为"⚠ LLM 调用失败"文本回复，不中断对话。
- `core/codegen`：注入参考上下文（预算受控）、调用 LLM、必生成项断言、产物落盘。
- `core/dst`：目录校验、白名单读取、ModTools 自动探测。
- `core/schema`：人设 Schema、校验、Markdown 渲染、`json_schema_hint()`（给 LLM 的结构提示）。
- `core/project`：`project.json` 读写、人设冻结、`chat.jsonl` 读写与重置。
- `core/py`：Python 解释器探测、pip 安装（分组）。

### 8.3 进程模型

- Rust 负责 Python 生命周期（spawn / 超时 / 终止）；请求带 `request_id` 关联响应；取消 = terminate + 清理。
- Python 启动前执行版本与依赖就绪探测（`python --version`、`import` 检查）。
- 所有跨进程文本 UTF-8。

### 8.4 配置存储

- 路径：`%APPDATA%\dst-mod-agent-generator\config.json`（`config_dir()`）。
- 结构：

```json
{
  "dst_dir": "D:/Steam/steamapps/common/Don't Starve Together",
  "modtools_path": "",
  "llm":   { "base_url": "", "api_key": "", "model": "" },
  "image": { "base_url": "", "api_key": "", "model": "" },
  "update": { "enabled": true, "repo": "qqxyfb/DST-Mod-Agent-Generator" }
}
```

### 8.5 project.json（真实结构）

```json
{
  "path": "H:/agent/DST-Mod-Agent-Generator/projects/my_char",
  "name": "my_char",
  "meta": { "name": "my_char", "author": "player", "version": "1.0.0", "description": "", "tags": [] },
  "notes": "角色描述自由文本",
  "created_at": "2026-09-26T00:00:00Z",
  "character": {
    "version": 1,
    "json": { },
    "markdown": "# 角色设定文档 ...",
    "frozen": false,
    "draft": true
  },
  "pipeline": {
    "stage1": { "status": "success", "artifact_version": "character_v1", "input_hash": "…", "log": [], "updated_at": "…", "hint": "" }
  },
  "report": { "mod_dir": "…", "files": [], "notes": [] }
}
```

### 8.6 人设 Schema（`core/schema.rs`）

`CharacterSheet` 实际字段：

```json
{
  "char_name": "wanda_cn",
  "display_name": "Wanda",
  "description": "一句话简介",
  "stats": { "health": 150, "hunger": 150, "sanity": 150 },
  "passive_skills": [ { "id": "night_vision", "params": { "radius": 4 } } ],
  "active_skills": [ { "id": "spell_gift", "item": "wanda_gift", "cooldown": 120 } ],
  "starter_items": [ "axe", "torch" ],
  "special_items": [ { "id": "wanda_gift", "kind": "tool", "stats": { "damage": 20 } } ],
  "speech": { "ANNOUNCE_ACCOMPLISHMENT": "这是我的新成就！" },
  "pros_cons": { "pros": ["移动快"], "cons": ["怕黑"] }
}
```

校验规则（失败即拒绝定稿）：

- `char_name`：仅小写字母 / 数字 / 下划线，非空。
- `stats.health / hunger / sanity`：整数且落在 **50~300**。
- `starter_items`：**非空且不超过 6 项**。
- `display_name`、`pros_cons` 等必填字段非空。
- 校验错误会原样回传 LLM 修正（最多 3 次），仍失败转人工编辑。
- `json_schema_hint()` 把上述结构提示注入 LLM 提示词，降低输出偏差。

### 8.7 DST 原版参考白名单（`core/dst.rs`）

固定 9 个文件（控制 token 预算）：

```
scripts/prefabs/wilson.lua
scripts/prefabs/wolfgang.lua
scripts/prefabs/wickerbottom.lua
scripts/prefabs/wes.lua
scripts/components/health.lua
scripts/components/hunger.lua
scripts/components/sanity.lua
scripts/components/inventory.lua
scripts/components/combat.lua
```

`whitelist_snippets(dir, single_limit = 60000, total_limit = 8000)`：单文件截断 60000 字符，注入总量上限 8000 tokens 量级，避免超预算。

### 8.8 代码生成规则（Stage5 产物）

- **确定性模板**（不依赖 LLM，保证格式稳定）：
  - `modinfo.lua`：`name / description / author / version / api_version = 10 / dst_compatible = true / dont_starve_compatible = false / all_clients_require_mod = true / server_filter_tags = { 标签多选结果 } / configuration_options = {}`。
  - `modmain.lua`：`PrefabFiles`（人物 + 专属道具）、`GLOBAL.STRINGS.CHARACTERS[CHAR_NAME] = require("speech_<char>")`、`AddModCharacter(CHAR_NAME, "NEUTRAL")`。
- **LLM 生成**（system prompt 注入白名单参考 + 重跑提示词，输出前剥离 ```lua 围栏）：
  - `prefabs/<char_name>.lua`（人物 prefab）
  - `speech_<char_name>.lua`（台词表）
  - `prefabs/<item>.lua`（每个专属道具一个）
- **必生成项断言**：`modinfo.lua`、`modmain.lua`、`prefabs/<char_name>.lua`、`speech_<char_name>.lua` 缺一即阶段失败。
- **写入规范**（system prompt 固化）：服务器逻辑位于 `if not TheWorld.ismastersim then return end` 之后；客户端访问 `ThePlayer` 前判空；不污染全局；只用已知 DST API；只输出 Lua 代码。
- 规则库文件：`resources/rules/codegen_rules.md`（版本化，跟随 DST 更新）。

### 8.9 Tauri command 清单（29 个）

| 分组 | commands |
|---|---|
| 配置（9） | `get_config` `save_config` `validate_dst_dir` `llm_test` `image_test` `python_env_status` `python_env_setup` `detect_modtools` `check_update` |
| 项目与人设（10） | `create_project` `list_projects` `open_project` `update_project` `list_references` `import_reference` `load_chat_history` `agent_chat` `agent_init` `confirm_character` |
| 流水线（6） | `pipeline_state` `start_stage` `rerun_stage` `skip_stage` `get_stage_log` `get_report` |
| 调试（4） | `locate_logs` `read_log` `fix_from_log` `gen_console_cmds` |

> 前端 `src/api/tauri.ts` 统一封装 invoke；参数一律 **camelCase**（Rust 侧由 serde 重命名），避免 `missing required key` 类错误。

## 9. 错误处理与人工兜底

### 9.1 全局错误处理

- API 调用失败：界面提示 + 保存当前项目 + 允许重试当前步骤（阶段可重跑）。
- 阶段产物校验失败：进入该阶段修复模式，不动其他阶段产物。
- DST 目录校验失败：禁止继续生成 mod（Stage5 直接返回错误并列出缺失项）。
- 设置弹窗内 API 测试失败：按 §5.1.2 分类展示状态码 + URL + 响应体片段。

### 9.2 图像失败退回流程（V1）

- 部件分割失败（rembg/SAM2）提供 3 个选项：
  1. 用原图重新生成该部件（LLM 收到失败原因调整提示词）；
  2. 跳过 SAM2，手动上传部件图（仍强制模板尺寸/命名校验）；
  3. 模板占位合成（原图直接贴到模板对应位置，报告标记待人工复核）。
- 批量多部件失败：支持整体重试或逐部件处理。

### 9.3 Lua 修复循环

- 输入：日志报错块（含文件/行号）+ 相关源码 + 规则库摘要。
- LLM 输出修复结果（整文件重写），应用后标记"待测"。
- 每轮带上次修复结果；最多 3 轮；仍失败保留现场（日志 + 源码副本）供人工排查。
- 修复仅触碰 Lua，不涉及图像/骨骼。

### 9.4 DST 日志定位

- 递归扫描 `%USERPROFILE%\Documents\Klei\**`，匹配文件名含 `log` 的 `.txt`（覆盖 `DoNotStarveTogether`、分支子目录、KleiUserID 子目录），按修改时间倒序取前 10。
- 读取策略：只读 + 错误行索引提取（`error_indices`），供界面高亮。

## 10. 安全与隐私

- 所有 API 请求直连用户配置地址，**不中转第三方服务器**。
- API Key 仅本地存储（`config.json`，目录受用户账号保护）；不上传项目文件、不上传 Key。
- 自动更新只从用户配置的 GitHub 仓库拉取。
- 报告的打包日志可能含本机路径，`log/` 默认 gitignore，不进版本库。
- 生成的 Mod 内容与发布责任由用户自行承担。

## 11. 版本更新机制

1. 启动时（受开关控制）请求 GitHub Releases API（仓库来自配置），与 `CURRENT_VERSION` 做语义化比较。
2. 发现新版本 → 提示用户 → 下载 zip 到临时目录 → 校验（版本 / 可选哈希）。
3. 替换流程（V1 落地）：退出主程序 → 独立 updater 进程移动新版本文件 → 重启主程序；失败自动回滚旧版本（旧包保留）。
4. 更新期间不写入 mods 目录，不影响用户 Mod 项目数据。

> 设计要点（C4）：Windows 上运行中的 exe 不可覆盖，因此**必须**走"退出后替换 + 重启"，不能原地覆盖。

## 12. 内置预制资源与许可

- **ESC 模板**：来源 = 社区 Extended Sample Character 模板（Klei 官方示例角色模板衍生）。随包附带前必须确认其再分发许可；当前 `templates/esc/` 仅有 README 占位与许可说明，实际文件于 V1 入包。
  - 工具只生成副本并替换贴图引用，**不修改骨骼关键帧**。
  - 预期布局：`esc.xml / *.scml`（骨骼动画）、`tex/`（将被替换的贴图）、`sizes.json`（由 escmapper 解析生成的对照表）。
- **Klei Mod Tools（autocompiler.exe）**：由用户自行安装（Steam）并配置路径，工具**不捆绑分发**；提供自动探测 + 手动选择。
- **rembg / SAM2 权重**：首次使用时下载至本地模型目录（可配置镜像或离线导入）。

## 13. 硬性限制（固化，不可突破）

1. **不自动生成/编辑 Spriter 骨骼关键帧动画**，仅复用 ESC 模板骨骼动作。
2. 图像分割存在失败概率：工具必须检测异常并走 §9.2 兜底，不承诺全自动成功。
3. 不做流程外的自定义 LLM 工作流编排（ComfyUI 工作流由 V2 适配器按固定预设调用）。
4. 不收集/上传用户数据（§10）。
5. ModTools 与 DST 游戏本体不随工具分发。
6. 不自动安装 autocompiler.exe（Klei 工具无公开直链，避免许可与版本风险）。

（代码内相应位置均加注释标记上述限制，防止后续实现越界：`src-tauri/src/core/py/mod.rs`、`python/bridge.py`、`core/pipeline.rs`、`templates/esc/README.md`。）

## 14. 里程碑与 Roadmap

| 里程碑 | 内容 | 状态 |
|---|---|---|
| M0 设计 | PROJECT_SPEC 定稿（v1.1） | 完成 |
| M1 MVP | 设置弹窗（配置/校验/API 测试/环境/更新）；项目 + 人设对话 + 一键概设 + 持久化 + 定稿；状态机 + Stage1 + Stage5 + Stage7；日志 + 一键修复 + 控制台命令；打包脚本 + 日志目录 | **完成（已编译验证，NSIS/MSI 打包通路打通，代码已推送）** |
| M2 V1 | 图像线（提示词/生成/rembg/SAM2/对齐）、Stage6 资源编译、ESC 模板入包、自动更新全量、schinese/modicon | 未开始 |
| M3 V2 | SD WebUI/ComfyUI 适配、多角色、创意工坊发布辅助 | 未开始 |

## 15. 验收标准

### 15.1 MVP（已达成）

- Win10/11 上按 README 指引从源码编译并运行（`npm run tauri dev`）。
- `build.bat -NoBundle` 产出可运行 exe；`build.bat -Bundles nsis|msi` 产出安装包。
- DST 目录校验：错误路径给出缺失项提示、正确路径通过并回显 scripts 统计。
- LLM / 图像 API 测试连通，失败时给出可定位原因（状态码 + URL + 响应片段）。
- 环境面板 6 项状态正确；一键安装基础依赖后状态可刷新为就绪。
- 完整走通：建项目 → 传图 → 描述 → 一键概设 → 多轮对话 → 定稿 → Stage1 → Stage5 → Stage7 报告。
- 生成的 mod 放入 DST mods 后无基础 Lua 报错、可在游戏中启用。
- 对话在重启程序后自动回填（不丢失）。
- 调试页能定位并读取日志、高亮错误行；控制台命令可复制。

### 15.2 V1（待验收）

- 生成含贴图 / 动画的可运行 mod（复用 ESC 骨骼）。
- 资源编译产出 `anim/`（bin + tex）并写入导出缓存目录。
- 一键修复闭环：日志报错 → LLM → 修复 → 复测通过。

## 16. 构建、打包与日志

### 16.1 打包入口

| 方式 | 命令 | 说明 |
|---|---|---|
| 一键脚本 | `.\build.bat -NoBundle` | 仅编译 exe（快速调试） |
| 一键脚本 | `.\build.bat -Bundles nsis` / `-Bundles msi` | 生成安装包 |
| 一键脚本 | `.\build.bat` | 完整打包（NSIS + MSI） |
| 参数 | `-SkipNpmInstall` | 跳过 `npm install` 与图标生成 |
| 脚本主体 | `.\scripts\build.ps1 [同参数]` | 高级用法 |
| 直调 | `npm run tauri build -- --bundles nsis` | 绕过脚本 |

- `build.bat` 逻辑：申请管理员权限（UAC 提权，用于写 tauri bundler 工具目录）→ 调 `scripts/build.ps1` → 设置国内镜像（npm registry、`TAURI_BUNDLER_TOOLS_GITHUB_MIRROR`）。
- **提权修复**：提权分支必须处理"无额外参数"的情况（空 `-ArgumentList` 会触发 PowerShell 参数绑定异常），已修复。
- 环境变量 `DST_NO_ELEVATE=1` 可跳过提权（用于 CI / 自动化调试）。

### 16.2 图标

- 源图 `app-icon.png` → `scripts/generate-icons.ps1` 生成各尺寸到 `src-tauri/icons/`。
- **打包陷阱（已修复）**：`tauri.conf.json` 的 `bundle.icon` 不能为空数组，且必须存在 `.ico`，否则 bundler 报 `Couldn't find a .ico icon`。

### 16.3 日志目录（`log/`）

| 文件 | 内容 |
|---|---|
| `log/runtime.log` | 程序运行时日志（command 调用、阶段执行、错误） |
| `log/build-*.log` | 打包脚本日志（`build.ps1` 每次执行的完整输出） |

- `log/` 已加入 `.gitignore`（保留 `.gitkeep`），只记本机日志，不入版本库。
- 排查约定：打包失败先看 `log/build-*.log` 尾部；程序行为异常先看 `log/runtime.log`。

### 16.4 网络与镜像（国内环境）

- npm：`npm install --registry=https://registry.npmmirror.com`
- Tauri bundler 工具（NSIS / WiX）：`$env:TAURI_BUNDLER_TOOLS_GITHUB_MIRROR = "https://gh-proxy.com"`
- 离线预置：把 `nsis-3.11.zip` 解压到 `%LOCALAPPDATA%\tauri\NSIS`，并把 `nsis_tauri_utils.dll` 放到 `%LOCALAPPDATA%\tauri\NSIS\Plugins\x86-unicode\additional\`，哈希校验通过后可直接使用。

## 17. V1 待办清单

| # | 待办 | 对应章节 |
|---|---|---|
| 1 | ESC 模板实际文件入包（确认再分发许可）+ escmapper 解析 `sizes.json` | §6.7 / §12 |
| 2 | Stage2 部件提示词生成（固定 seed / 降级策略，消费重跑提示词） | §6.2 / §6.7 |
| 3 | Stage3 批量图像生成（OpenAI 兼容 + 服务商扩展字段适配） | §5.1.2 / §6.7 |
| 4 | Stage4 rembg 抠图 + SAM2 分割 + POT 尺寸 + 模板命名对齐 + 异常检测 | §6.7 / §9.2 |
| 5 | 部件级重生成 / 手动覆盖产物 / 阶段暂停与中断 | §6.4 / §6.5 |
| 6 | Stage6 资源编译（scml 贴图引用改写 + autocompiler 调用 + 导出缓存） | §6.2 / §3.3 |
| 7 | 自动更新全量（下载 → 校验 → 退出替换 → 重启 → 回滚） | §11 |
| 8 | `strings/schinese.lua` 语言包与 `modicon.png` 生成 | §2.2 C13 |
| 9 | SD WebUI / ComfyUI 适配器 | §4.3 V2 |

---

### 附：本文档维护约定

- 代码内注释以 `PROJECT_SPEC.md §X` 形式回指本文档章节；改动实现时同步修订对应章节。
- 新增"硬性限制"必须同时落到：本文档 §13、相关代码注释、README「硬性限制」三段。
- 每个里程碑完成需更新：§1.1 版本表、§1.2 变更记录、§14 状态列。