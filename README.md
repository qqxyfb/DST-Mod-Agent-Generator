# DST-Mod-Agent-Generator

仓库：<https://github.com/qqxyfb/DST-Mod-Agent-Generator>

饥荒联机版（Don't Starve Together）自定义人物 Mod 生成桌面工具：普通用户上传**参考图片 + 文字描述**，即可自动生成 DST 人物 Mod（Lua 代码 + 复用 ESC 模板动画贴图）。用户不需要懂 Lua，不需要手动配置 ComfyUI/SAM2 等底层工具。

## 当前状态

- 阶段：**M1 MVP 已完成并编译验证**（含账号校验、人设对话、代码生成、双管线状态机、日志调试）；图像线与资源编译为 V1 预留接口。
- 完整设计文档见 [PROJECT_SPEC.md](./PROJECT_SPEC.md)：评审结论、可行性分析、修订后的需求规格、双管线阶段状态机、MVP 范围与 Roadmap。
- 用户补充需求已固化：① 阶段卡片化 + 视觉线/代码线可并行切换；② 单阶段/单部件可单独重新生成。
- M1 已动作化：Tab1 “一键安装基础依赖”真实执行 `pip install -r python/requirements.txt`；Tab2 参考图经 `tauri-plugin-dialog` 选择并复制进项目 `reference/` 目录。

## 技术栈

- 前端：Vue 3 + TypeScript + Vite（四个 Tab）
- 主程序：Rust（Tauri 2），包含配置 / 项目 / 人设对话 / 流水线状态机 / 代码生成 / 日志调试
- Python 子进程：图像管线（rembg / SAM2 / 后处理），**V1 预留**，见 [python/README.md](./python/README.md)
- 参考图选择依赖 `@tauri-apps/plugin-dialog`（前端）与 `tauri-plugin-dialog`（Rust），`npm install` 时一并安装

## 目录结构（摘要）

```
DST-Mod-Agent-Generator/
├── PROJECT_SPEC.md         # 设计文档（评审 + 可行性 + 需求规格）
├── src/                    # Vue3 前端（views / api / components/pipeline）
├── src-tauri/              # Rust 主程序
│   ├── src/commands/       # Tauri command 层（GUI 调用的后端接口）
│   └── src/core/           # 业务逻辑：config/dst/llm/schema/project/pipeline/codegen/logreader/console/py
├── python/                 # Python 图像管线桥（V1 预留）
├── templates/esc/          # ESC 模板占位目录（V2 随包附带，授权确认后放入）
├── resources/
│   ├── schemas/character.schema.json   # 人设 JSON Schema
│   └── rules/codegen_rules.md          # DST 代码生成规则库
├── cache/                  # 运行缓存（原版脚本索引等）
└── projects/               # 用户 Mod 项目默认输出目录
```

## 编译与运行（Windows 优先）

### 前置依赖

| 组件 | 版本 | 说明 |
|---|---|---|
| Node.js + npm | ≥ 18 | 前端构建 |
| Rust 工具链 | stable（MSVC） | 后端编译，需安装 VS Build Tools C++ |
| WebView2 | Win10/11 自带 | Tauri 2 运行时 |
| Python（可选） | 3.10+ | 仅环境检测用；图像管线 V1 才需要 |

### 开发模式

```powershell
# 1. 安装前端依赖
npm install

# 2. 启动 Tauri 开发（自动拉起 Vite + Rust，窗口打开）
npm run tauri dev
```

> 仅预览 UI（无后端真实功能）：`npm run dev`，浏览器打开 http://127.0.0.1:1420，走前端 mock 数据。

### 生产打包

```powershell
# 方式一：根目录一键脚本（双击 build.bat 即可，自动提权 + 国内镜像）
.\build.bat -NoBundle            # 仅编译应用 exe（快速调试）
.\build.bat -Bundles nsis        # 生成 NSIS 安装包（推荐）
.\build.bat -Bundles msi         # 生成 MSI 安装包（需要 WiX，首次自动下载）
.\build.bat                      # 完整打包（NSIS + MSI）
.\build.bat -SkipNpmInstall      # 跳过 npm install 与图标生成

# 方式二：scripts\build.ps1 脚本打包（推荐高级用法）
.\scripts\build.ps1 -NoBundle            # 仅编译应用 exe（快速调试）
.\scripts\build.ps1 -Bundles nsis        # 生成 NSIS 安装包
.\scripts\build.ps1 -Bundles msi         # 生成 MSI 安装包（需要 WiX，首次自动下载）
.\scripts\build.ps1                      # 完整打包（NSIS + MSI）
.\scripts\build.ps1 -SkipNpmInstall      # 跳过 npm install 与图标生成

# 方式三：直接调用 tauri
npm run tauri build -- --bundles nsis
```

产物：
- 应用 exe：`src-tauri/target/release/dst-mod-agent.exe`
- NSIS 安装包：`src-tauri/target/release/bundle/nsis/*-setup.exe`
- MSI 安装包：`src-tauri/target/release/bundle/msi/*.msi`

> 国内网络提示
> - npm 建议使用镜像源：`npm install --registry=https://registry.npmmirror.com`
> - 首次打包时 tauri-bundler 会从 GitHub 下载 NSIS / WiX 工具；若下载超时，可设置镜像环境变量：
>   `$env:TAURI_BUNDLER_TOOLS_GITHUB_MIRROR = "https://gh-proxy.com"`
> - 也可离线预置：把官方 `nsis-3.11.zip` 解压到 `%LOCALAPPDATA%\tauri\NSIS`，并把 `nsis_tauri_utils.dll` 放到
>   `%LOCALAPPDATA%\tauri\NSIS\Plugins\x86-unicode\additional\`，即可跳过 NSIS 下载（哈希校验通过后直接使用）。

> `autocompiler.exe`（Klei Mod Tools）由用户经 Steam 自装，工具不捆绑分发。

### 目录校验与 LLM 配置

1. Tab1 选择 DST 根目录（需含 `mods/` 与 `scripts/`）并点击"校验"。
2. 填写 LLM API（OpenAI 兼容：ollama / deepseek / qwen 等）并测试连通。
3. （可选）Tab1 “一键安装基础依赖”安装 Python 的 Pillow / numpy（图像线 V1 前置）。
4. Tab2 创建项目 → 上传参考图（复制进 `reference/`）→ 人设对话 → 确认定稿 → Tab3 执行 Stage1 / Stage5 生成代码。

> Tab2 的人设对话按项目持久化到 `<项目>/logs/chat.jsonl`；重新打开程序或切换项目会自动回填历史对话，
> 不需要重新「一键生成人物概设」。再次点击「一键生成人物概设」会清空该项目的对话记录并重开一轮。

## 里程碑

| 里程碑 | 内容 | 状态 |
|---|---|---|
| M0 设计 | PROJECT_SPEC.md 定稿 | 完成 |
| M1 MVP | 配置页 + 人设对话 + 代码生成 + 双管线状态机 + 日志调试 | 已编译验证（含 NSIS 安装包） |
| M2 V1 | 图像线（提示词/生成/rembg/SAM2/对齐）+ 资源编译 + 一键修复闭环 + 更新全量 | 未开始 |
| M3 V2 | SD WebUI/ComfyUI 适配、多角色、创意工坊发布辅助 | 未开始 |

## 硬性限制

- 不自动生成/编辑 Spriter 骨骼关键帧动画，仅复用 ESC 模板骨骼动作（`src-tauri/src/core/py/mod.rs`、`python/bridge.py` 均有注释标记）。
- API Key 与项目文件仅本地存储；所有 API 请求直连用户配置地址，不中转第三方服务器。
