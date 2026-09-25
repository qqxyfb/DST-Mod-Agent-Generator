# projects/ —— 用户 Mod 项目默认输出目录

每个子目录是一个独立 Mod 项目（`project.json` 记录元数据、阶段产物版本与配置快照）：

```
projects/<ModName>/
├── project.json    元数据 + 各阶段产物版本 + 配置快照
├── characters/     人设定稿 JSON（vN）+ Markdown 文档
├── reference/      用户上传参考图（原图）
├── artwork/        部件提示词 JSON、生成原图、后处理 PNG
├── mod/            mod 源码（modmain/modinfo/prefabs/speech 等）
├── mod_compiled/   编译产物（导出缓存，默认不直接写 DST mods）
└── logs/           各阶段日志、LLM 对话记录（chat.jsonl）
```

> 打包版（无仓库开发目录）时输出到 `%APPDATA%/dst-mod-agent-generator/projects`。