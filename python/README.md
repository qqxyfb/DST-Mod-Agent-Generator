# python/ —— Python 图像管线（V1）

- `bridge.py`：stdio JSONL 服务端，供 Rust 主程序以子进程方式调用。
- `requirements.txt`：MVP 基础依赖（仅用于环境就绪检测）。
- `requirements-image.txt`：V1 图像管线完整依赖（rembg / onnxruntime / SAM2）。

## 协议

```
请求   {"request_id":"r1","cmd":"ping","args":{}}
进度   {"type":"progress","request_id":"r1","percent":50,"message":"..."}
完成   {"type":"result","request_id":"r1","ok":true,"data":{...},"error":null}
```

## 本地测试

```powershell
Get-Content req.json | python python/bridge.py
```

## V1 预留模块（接口位）

| 模块 | 命令 | 职责 |
|---|---|---|
| background_remover.py | rembg | 抠透明背景 |
| segmenter.py | sam2 | SAM2 部件分割 |
| postprocess.py | postprocess | resize 到 2 的幂、命名、输出 |
| escmapper.py | escmapper | 解析模板 scml、部件-尺寸-命名对照 |
| scml_editor.py | scml_editor | 改写 scml 贴图引用（编译用） |

硬性限制：本管线只做图像处理，不生成/修改骨骼动画；分割失败必须上报错误走人工兜底（PROJECT_SPEC.md §8.2）。