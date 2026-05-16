#!/usr/bin/env python3
"""Task2 最优批处理：生成 `dataset/lab2/batch_{1,2,3}.in/.out`（小/中/大）。

标答与 Rust `BatchScheduling::solve` 的 O(n^2) DP 一致。仓库根目录执行：
`python scripts/gen_lab2_task2_batch.py`
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


def solve_batch(n: int, s: int, t: list[int], f: list[int]) -> int:
    prefix_t = [0] * (n + 1)
    prefix_f = [0] * (n + 1)
    for i in range(n):
        prefix_t[i + 1] = prefix_t[i] + int(t[i])
        prefix_f[i + 1] = prefix_f[i] + int(f[i])
    f_n = prefix_f[n]
    inf = 10**40
    dp = [inf] * (n + 1)
    dp[0] = 0
    for i in range(1, n + 1):
        ti = prefix_t[i]
        fi = prefix_f[i]
        for j in range(i):
            cand = dp[j] + ti * (fi - prefix_f[j]) + s * (f_n - prefix_f[j])
            dp[i] = min(dp[i], cand)
    return int(dp[n])


def write_one(case: int, n: int, s: int) -> None:
    random.seed(1337 + case)
    t = Vector.random(n, [(1, 50)])
    f = Vector.random(n, [(1, 20)])
    ans = solve_batch(n, s, [int(x) for x in t], [int(x) for x in f])
    body = f"{s}\n{n}\n" + " ".join(str(int(x)) for x in t) + "\n"
    body += " ".join(str(int(x)) for x in f) + "\n"
    os.makedirs(OUT_DIR, exist_ok=True)
    open(os.path.join(OUT_DIR, f"batch_{case}.in"), "w", encoding="utf-8").write(body)
    open(os.path.join(OUT_DIR, f"batch_{case}.out"), "w", encoding="utf-8").write(str(ans) + "\n")


def main() -> None:
    # 小 / 中 / 大：与 `BatchScheduling::solve` 的 n^2 规模匹配
    write_one(1, 50, 4)
    write_one(2, 800, 9)
    write_one(3, 6000, 11)


if __name__ == "__main__":
    main()
