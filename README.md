# 这是什么？

这是湖南大学 2026 春计科拔尖班的算法设计与分析课程的代码仓库。

```
├─benches               算法的基准测试
├─reports               课程作业的 PPT / 报告
│  └─discussion1        第一次小班讨论：PageRank
├─src
│  ├─algorithms         算法的代码实现
│  ├─bin
|  |  ├─bmssp.rs        BMSSP 算法的测试（写成 bin 主要为了方便跑火焰图）
|  |  ├─text_rank.rs    PageRank 算法扩展：TextRank
|  |  └─textbook_1_2.rs 课本 1.2 题目实现
│  ├─dataset            测试/基准测试用到的测试数据
│  ├─ds                 数据结构的代码实现
│  └─utils              其他的工具函数
└─tests                 集成测试（一般是在大数据上的测试）
```

# 任务

## 1. 第一次小班讨论

> 第六组：数学之美：P98 第 10 章 PageRank-Google 的民主表决式网页排名技术+实现至少一种 PageRank 算法+大规模数据集效果测试（建议阅读 第 10 章、网上搜索更多资料）

* PageRank
  * 代码：`src/algorithms/pagerank.rs`
  * 测试：`tests/pagerank.rs`
  * 基准测试：无
* CSCMatrix（按列压缩的稀疏矩阵）
  * 代码：`src/algorithms/matrix.rs`
  * 测试：单元测试
  * 基准测试：无
  * 参考：<https://www.caiwen.work/post/mit6172-1#1-5-sparsity>
  * 用于实现在稀疏图上的 PageRank 算法
* TextRank
  * 代码：`src/bin/text_rank.rs`
  * 测试：无
  * 基准测试：无

课堂展示PPT （pdf + latex 源码）位于：`reports/discussion1`

## 2. 第一次前沿阅读

> STOC2025-单源最短路径新算法(要求：word 版阅读报告+代码复现（阅读报告里整理实际运行结果和分析） +ppt 分享报告 ) ， 建议作业周期 2026.3.9-2026.3.30，3 月 30 号晚 8：00 前提交，4 月 1 号课堂分享
>
> 论文：https://arxiv.org/abs/2504.17033

* BMSSP（论文中的最短路算法）
  * 代码：`src/algorithms/bmssp/*`、`src/bin/bmssp.rs`
  * 测试：单元测试、`tests/bmssp.rs`
  * 基准测试：`benches/bmssp.rs`
* Dijkstra
  * 代码：`src/algorithms/ssp.rs`
  * 测试：文档测试、单元测试、`tests/ssp.rs`
  * 基准测试：`benches/ssp.rs`

* Spfa
  * 代码：`src/algorithms/ssp.rs`
  * 测试：文档测试、单元测试、`tests/ssp.rs`
  * 基准测试：`benches/ssp.rs`

阅读报告：<https://www.caiwen.work/post/bmssp>

课堂展示PPT：TODO

## 3. 第一次实验

> **（离线题** **3** **题）**
>
> 经典案例+课后算法实现题 
>
> 共 3 题（离线准备，每人 3 道题，每人完成 3 个实验并撰写实验报告，【第 5 周周六验收】
>
> 1. 分治法查找最大最小值
> 2. 分治法实现合并排序
> 3. 实现题 1-2 字典序问题
>
> **第二次实验（离线题** **3** **题）**【第 5 周周六验收】
>
> 1. 用动态规划算法求解 0-1 背包问题
> 2. 用贪心算法求解背包问题
> 3. 实现题 2-6 排列的字典序问题

* find_min_max（分治法查找最大最小值）
  * 代码：`src/algorithms/divide_conquer.rs`
  * 测试：文档测试、单元测试、`tests/divide_conquer.rs`
  * 基准测试：`benches/divide_conquer_min_max.rs`
* sort（归并排序）
  * 代码：`src/algorithms/divide_conquer.rs`
  * 测试：文档测试、单元测试、`tests/divide_conquer.rs`
  * 基准测试：`benches/divide_conquer_sort.rs`
* 课本 1.2 字典序问题
  * 代码：`src/bin/textbook_1_2.rs`
  * 测试：单元测试（暴力和计数 DP 做法对拍）
  * 基准测试：无
* SimpleKnapsack（01 背包）
  * 代码：`src/algorithms/dp.rs`
  * 测试：文档测试、`tests/dp.rs`
  * 基准测试：`benches/simple_knapsack.rs`
* 康托展开/逆康托展开
  * 代码：`src/algorithms/misc.rs`
  * 测试：文档测试、`tests/misc.rs`
  * 基准测试：无
* 树状数组
  * 代码：`src/ds/bit.rs`
  * 测试：单元测试
  * 基准测试：无
  * 用于优化康托展开和逆康托展开

实验报告：TODO
