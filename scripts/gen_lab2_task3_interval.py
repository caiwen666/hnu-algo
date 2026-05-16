#!/usr/bin/env python3
"""Task3 区间覆盖：生成 `dataset/lab2/interval_{1,2,3}.in/.out`（小/中/大）。

标答为排序后贪心。仓库根目录执行：`python scripts/gen_lab2_task3_interval.py`
"""

from __future__ import annotations

import os
import random

try:
    from cyaron import Vector  # type: ignore
except ImportError:

    class Vector:
        @staticmethod
        def random(n: int, ranges):  # noqa: ANN001
            (lo, hi), *_ = ranges
            return [random.randint(lo, hi) for _ in range(n)]


ROOT = os.path.join(os.path.dirname(__file__), "..")
OUT_DIR = os.path.join(ROOT, "dataset", "lab2")


def greedy_count(xs: list[float], k: float) -> int:
    ans = 0
    i = 0
    n = len(xs)
    while i < n:
        left = xs[i]
        right = left + k
        ans += 1
        i += 1
        while i < n and xs[i] <= right:
            i += 1
    return ans


def write_one(case: int, n: int, k: float) -> None:
    random.seed(4242 + case)
    pts = Vector.random(n, [(0, 1_000_000)])
    xs = sorted(float(x) for x in pts)
    ans = greedy_count(xs, k)
    os.makedirs(OUT_DIR, exist_ok=True)
    inp = f"{n} {k}\n" + " ".join(str(x) for x in xs) + "\n"
    open(os.path.join(OUT_DIR, f"interval_{case}.in"), "w", encoding="utf-8").write(inp)
    open(os.path.join(OUT_DIR, f"interval_{case}.out"), "w", encoding="utf-8").write(str(ans) + "\n")


def main() -> None:
    write_one(1, 200, 5.0)
    write_one(2, 5000, 12.5)
    write_one(3, 80_000, 12.5)


if __name__ == "__main__":
    main()
