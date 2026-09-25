# DST 代码生成规则库（版本化，跟随 DST 更新）

> 用途：作为 LLM 代码生成的系统提示与人工审查清单（PROJECT_SPEC.md §7.8）。
> 规则版本：v1（对应 DST 联机版现行 API）。

## 1. 文件清单（生成后必须断言存在）

- `modinfo.lua`：`api_version = 10`、`dst_compatible = true`、`dont_starve_compatible = false`、`all_clients_require_mod = true`、`icon` 字段。
- `modmain.lua`：注册 `PrefabFiles`、台词表、`AddModCharacter(name, "NEUTRAL")`。
- `prefabs/<char_name>.lua`：三围、被动/主动技能、开局物品、专属道具注册。
- `prefabs/<item>.lua`：专属道具（如有人设特殊道具）。
- `speech_<char_name>.lua`：ANNOUNCE_ / DESCRIBE_ / ACTIONFAIL_ 台词子集。
- 可选：`strings/schinese.lua` 语言包、`modicon.png`。

## 2. 代码规范（硬性）

1. 服务器逻辑必须放在 `if not TheWorld.ismastersim then return end` 之后。
2. 客户端逻辑访问 `ThePlayer` 前判空。
3. 不污染全局作用域；`GLOBAL.` 前缀明确时才能触碰引擎全局。
4. 组件/方法名只使用 DST 已知 API（白名单见原版 scripts 参考注入）。
5. V1 前避免自定义复杂 RPC；不修改预制组件源码。

## 3. 三围填值

- health / hunger / sanity 均取人设 JSON `stats` 字段（整数 50~300）。

## 4. 被动技能常见载体（DEMO 白名单，可按项目扩展）

- 饥饿/精神速率调整：`hunger`、`sanity` 组件 rate 修改。
- 夜间视野：`playerlight`、`nightvision`。
- 光环：`sanityaura`、`burning`。
- 背包加成：`inventory` / 自定义 `container`。
- 由 LLM 依据人设 JSON 的 `passive_skills[].id` 映射，未知名一律走校验失败回传修正。

## 5. 主动技能载体

- 以专属道具 prefab（`active_skills[].item`）+ 点击交互实现，冷却用组件 `rechargeable` 或自定义计时。
- 不生成骨骼动画；技能特效仅复用 DST 原生 prefab 特效（若有）。

## 6. 输出校验

- 必生成项断言 + Lua 语法检查（`luac -p` 若可用）+ 关键模式断言；
- 失败 → 错误 + 上下文回传 LLM 修复（最多 3 轮，PROJECT_SPEC.md §8.3）。

## 7. 限制固化（不可突破）

- 不自动生成/编辑 Spriter 骨骼关键帧动画，仅复用 ESC 模板骨骼动作。
- 不收集用户数据；代码生成请求只发往用户配置的 API 地址。