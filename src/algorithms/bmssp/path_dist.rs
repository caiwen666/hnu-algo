use std::cmp::Ordering;

/// 终点为 `end` 的一条最短路在算法中的全序关键字：距离、跳数、终点、前驱。
///
/// 比较顺序：`dis` → `hop` → `end` → `pred`，全部相同则相等。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PathDist {
    pub dis: u64,
    pub hop: u32,
    pub end: usize,
    pub pred: usize,
}

impl PathDist {
    pub const MAX: Self = Self {
        dis: u64::MAX,
        hop: u32::MAX,
        end: usize::MAX,
        pred: usize::MAX,
    };

    #[inline]
    pub const fn new(dis: u64, hop: u32, end: usize, pred: usize) -> Self {
        Self {
            dis,
            hop,
            end,
            pred,
        }
    }

    /// 由「标量」上界 `B` 得到四元组上界：在 `dis == B` 时对 `hop/end/pred` 取最大，
    /// 使得所有满足 `dis <= B` 的路径均 < 该上界
    #[inline]
    pub fn scalar_upper(dis: u64) -> Self {
        Self {
            dis,
            hop: u32::MAX,
            end: usize::MAX,
            pred: usize::MAX,
        }
    }

    /// `hop = 0`, `pred = 0`。与旧实现里按 `(dis, key)` 排序一致（`key` 即 `end`）。
    #[inline]
    pub fn from_dis(dis: u64, end: usize) -> Self {
        Self {
            dis,
            hop: 0,
            end,
            pred: 0,
        }
    }
}

impl Ord for PathDist {
    fn cmp(&self, other: &Self) -> Ordering {
        self.dis
            .cmp(&other.dis)
            .then_with(|| self.hop.cmp(&other.hop))
            .then_with(|| self.end.cmp(&other.end))
            .then_with(|| self.pred.cmp(&other.pred))
    }
}

impl PartialOrd for PathDist {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
