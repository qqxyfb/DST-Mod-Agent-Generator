# -*- coding: utf-8 -*-
"""Rust -> Python 子进程桥（stdio JSONL 协议，UTF-8）。

协议（与 src-tauri/src/core/py 对应，V1 启用）：
  请求:   {"request_id": "...", "cmd": "...", "args": {...}}
  进度:   {"type": "progress", "request_id": "...", "percent": 0..100, "message": "..."}
  完成:   {"type": "result", "request_id": "...", "ok": true/false, "data": {...}, "error": "..."}
  取消:   {"type": "cancel", "request_id": "..."}（当前单请求模型：标记后结束当前命令）

MVP 仅提供：ping / env_status；rembg / sam2 / postprocess / escmapper / scml_editor
为 V1 预留桩，返回明确"未接入"错误。

限制固化（PROJECT_SPEC.md §12）：
- 本桥只承载图像处理（抠图/分割/后处理），不实现骨骼动画生成。
- 不做流程外的 ComfyUI 工作流编排。
"""

import json
import sys


def emit(obj):
    sys.stdout.write(json.dumps(obj, ensure_ascii=False) + "\n")
    sys.stdout.flush()


def cmd_ping(args):
    return {"ok": True, "data": {"pong": True}}


def cmd_env_status(args):
    import importlib.util

    def has(mod):
        return importlib.util.find_spec(mod) is not None

    return {
        "ok": True,
        "data": {
            "python": sys.version.split()[0],
            "modules": {
                "PIL": has("PIL"),
                "numpy": has("numpy"),
                "rembg": has("rembg"),
                "sam2": has("sam2"),
            },
        },
    }


# V1 预留桩：接口签名已定义，返回固定错误
V1_STUBS = ("rembg", "sam2", "postprocess", "escmapper", "scml_editor")


def dispatch(cmd, args):
    if cmd == "ping":
        return cmd_ping(args)
    if cmd == "env_status":
        return cmd_env_status(args)
    if cmd in V1_STUBS:
        return {
            "ok": False,
            "data": None,
            "error": "V1 未接入（预留接口）：%s" % cmd,
        }
    return {"ok": False, "data": None, "error": "未知命令: %s" % cmd}


def main():
    cur = None
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            req = json.loads(line)
        except Exception as e:  # noqa: BLE001
            emit({"type": "result", "request_id": cur, "ok": False, "data": None, "error": "JSON 解析失败: %s" % e})
            continue

        cur = req.get("request_id")
        req_type = req.get("type")
        cmd = req.get("cmd", "")

        if req_type == "cancel":
            emit({"type": "result", "request_id": cur, "ok": False, "data": None, "error": "已取消"})
            continue

        out = dispatch(cmd, req.get("args") or {})
        emit({
            "type": "result",
            "request_id": cur,
            "ok": out.get("ok", True),
            "data": out.get("data"),
            "error": out.get("error"),
        })


if __name__ == "__main__":
    main()