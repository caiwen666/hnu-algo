#!/usr/bin/env python3
"""在 `dataset/lab2/` 下生成 Lab2 Task2/3/5/6 数据（各任务小/中/大三档）。

本入口依次调用四个子脚本。仓库根目录执行：`python scripts/gen_lab2_datasets.py`

可选：`pip install cyaron`（各子脚本在无 cyaron 时退回 stdlib random）。
"""

from __future__ import annotations

import os
import subprocess
import sys

ROOT = os.path.join(os.path.dirname(__file__), "..")
OUT_DIR = os.path.join(ROOT, "dataset", "lab2")

SUB_SCRIPTS = [
    "gen_lab2_task2_batch.py",
    "gen_lab2_task3_interval.py",
    "gen_lab2_task5_sfd.py",
    "gen_lab2_task6_vertex_cover.py",
]


def ensure_dir() -> None:
    os.makedirs(OUT_DIR, exist_ok=True)


def run_sub_scripts() -> None:
    scripts_dir = os.path.join(ROOT, "scripts")
    exe = sys.executable
    for name in SUB_SCRIPTS:
        path = os.path.join(scripts_dir, name)
        r = subprocess.run([exe, path], cwd=ROOT, check=False)
        if r.returncode != 0:
            raise SystemExit(f"{name} failed with code {r.returncode}")


def main() -> None:
    ensure_dir()
    run_sub_scripts()


if __name__ == "__main__":
    main()
