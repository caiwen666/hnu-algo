#!/usr/bin/env python3
"""Task5 无分隔符字典：生成 `dataset/lab2/sfd_{1,2,3}.in/.out`（小/中/大）。

须满足 n^k <= 2^22（与 Rust `MAX_SEPARATOR_SEARCH_STATES` 一致）。
DFS 标答与 `hnu_algo::algorithms::dfs::sfd::max_sfd_size` 对齐。

仓库根目录执行：`python scripts/gen_lab2_task5_sfd.py`
"""

from __future__ import annotations

import os

ROOT = os.path.join(os.path.dirname(__file__), "..")
OUT_DIR = os.path.join(ROOT, "dataset", "lab2")

MAX_SFD_STATES = 1 << 22


def pack_word(symbols: list[int], alphabet_size: int) -> int:
    wid = 0
    for x in symbols:
        wid = wid * alphabet_size + x
    return wid


def unpack_word(wid: int, alphabet_size: int, str_len: int) -> list[int]:
    v = [0] * str_len
    for i in range(str_len - 1, -1, -1):
        v[i] = wid % alphabet_size
        wid //= alphabet_size
    return v


def overlap_ids(a: int, b: int, alphabet_size: int, str_len: int) -> list[int]:
    xa = unpack_word(a, alphabet_size, str_len)
    xb = unpack_word(b, alphabet_size, str_len)
    k = str_len
    out: list[int] = []
    for r in range(1, k):
        sym = xa[r:k] + xb[0:r]
        out.append(pack_word(sym, alphabet_size))
    return out


def max_sfd_size_py(n: int, k: int) -> int:
    if n == 0 or k == 0:
        return 0
    total = n**k
    if total > MAX_SFD_STATES:
        raise ValueError(f"n^k={total} exceeds MAX_SFD_STATES")

    exclude_count = [0] * total
    in_set = [False] * total
    cur: list[int] = []
    best = [0]

    def dfs(idx: int) -> None:
        if len(cur) + (total - idx) <= best[0]:
            return
        if idx == total:
            best[0] = max(best[0], len(cur))
            return
        if exclude_count[idx] > 0:
            dfs(idx + 1)
            return

        dfs(idx + 1)

        cur.append(idx)
        in_set[idx] = True
        valid = True
        for y in cur:
            for o in overlap_ids(idx, y, n, k):
                if in_set[o]:
                    valid = False
                    break
            if not valid:
                break
            for o in overlap_ids(y, idx, n, k):
                if in_set[o]:
                    valid = False
                    break
            if not valid:
                break
        if valid:
            for x in cur:
                if x == idx:
                    continue
                for y in cur:
                    if y == idx:
                        continue
                    for o in overlap_ids(x, y, n, k):
                        if o == idx:
                            valid = False
                            break
                    if not valid:
                        break
                if not valid:
                    break

        if valid:
            stack: list[int] = []
            for y in cur:
                for o in overlap_ids(idx, y, n, k):
                    if o > idx:
                        exclude_count[o] += 1
                        stack.append(o)
            for x in cur:
                if x == idx:
                    continue
                for o in overlap_ids(x, idx, n, k):
                    if o > idx:
                        exclude_count[o] += 1
                        stack.append(o)
            dfs(idx + 1)
            while stack:
                o = stack.pop()
                exclude_count[o] -= 1

        in_set[idx] = False
        cur.pop()

    dfs(0)
    return best[0]


def write_one(case: int, n: int, k: int) -> None:
    ans = max_sfd_size_py(n, k)
    os.makedirs(OUT_DIR, exist_ok=True)
    open(os.path.join(OUT_DIR, f"sfd_{case}.in"), "w", encoding="utf-8").write(f"{n} {k}\n")
    open(os.path.join(OUT_DIR, f"sfd_{case}.out"), "w", encoding="utf-8").write(f"{ans}\n")


def main() -> None:
    assert max_sfd_size_py(3, 2) == 3
    assert max_sfd_size_py(2, 2) == 1
    # 控制 n^k 与搜索规模，保证脚本在纯 Python 下可快速完成
    write_one(1, 2, 5)  # 32
    write_one(2, 3, 3)  # 27
    write_one(3, 2, 6)  # 64


if __name__ == "__main__":
    main()
