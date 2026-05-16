#!/usr/bin/env python3
"""Task6 最小权顶点覆盖：生成 `dataset/lab2/vcover_{1,2,3}.in/.out`（小/中/大）。

输入文件中顶点编号为 1..n（与常见 OJ 一致）；标答为暴力 2^n 最优权值和。
仓库根目录执行：`python scripts/gen_lab2_task6_vertex_cover.py`
"""

from __future__ import annotations

import os
import random
from itertools import product

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


def brute_opt(n: int, edges: list[tuple[int, int]], w: list[int]) -> int:
    best = 10**40
    for mask in product([0, 1], repeat=n):
        ok = True
        for u, v in edges:
            if mask[u - 1] == 0 and mask[v - 1] == 0:
                ok = False
                break
        if not ok:
            continue
        cost = sum(int(w[i]) for i in range(n) if mask[i] == 1)
        best = min(best, cost)
    return int(best)


def write_one(case: int, n: int, m: int) -> None:
    rng = random.Random(7 + case)
    edges = []
    seen = set()
    while len(edges) < m:
        u = rng.randint(0, n - 1)
        v = rng.randint(0, n - 1)
        if u == v:
            continue
        a, b = (u, v) if u < v else (v, u)
        if (a, b) in seen:
            continue
        seen.add((a, b))
        edges.append((a + 1, b + 1))

    w = Vector.random(n, [(1, 10)])
    best = brute_opt(n, edges, [int(x) for x in w])

    lines = [f"{n} {len(edges)}"]
    for u, v in edges:
        lines.append(f"{u} {v}")
    lines.append(" ".join(str(int(x)) for x in w))
    os.makedirs(OUT_DIR, exist_ok=True)
    open(os.path.join(OUT_DIR, f"vcover_{case}.in"), "w", encoding="utf-8").write(
        "\n".join(lines) + "\n"
    )
    open(os.path.join(OUT_DIR, f"vcover_{case}.out"), "w", encoding="utf-8").write(str(best) + "\n")


def main() -> None:
    write_one(1, 8, 14)
    write_one(2, 14, 28)
    write_one(3, 20, 48)


if __name__ == "__main__":
    main()
