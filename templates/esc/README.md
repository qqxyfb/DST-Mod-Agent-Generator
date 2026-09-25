# ESC（Extended Sample Character）模板

此目录为内置 ESC scml 骨骼模板的占位目录。**MVP 不含实际模板文件**，V2 随包附带。

## 说明

- 来源：社区 Extended Sample Character 模板（Klei 官方示例角色模板衍生）。
- 授权：随包附带前必须确认其再分发许可；仅在确认后把模板文件放入本目录。
- 工具行为：只替换贴图引用，**不生成/修改骨骼关键帧动画**（PROJECT_SPEC.md §12 硬性限制，代码内亦有注释标记）。

## 预期布局（V2）

```
templates/esc/
├── esc.xml / *.scml      骨骼与动画文件（Spriter 工程）
├── tex/                  模板贴图（将被生成部件替换）
└── sizes.json            部件-尺寸-命名对照表（由 escmapper 解析 scml 生成，唯一事实源）
```

## 部件类别（供图像管线参考）

- build 部件：head / body / arm_left / arm_right / leg_left / leg_right
- ghost 变体：视模板而定（如 ghost_head）
- 头像部件：bigportrait / smallportrait / avatar
- 图标：item_icon（专属道具）、modicon（创意工坊，可选）

> 精确清单以实际 scml 引用的纹理名为准（7.6 节）。