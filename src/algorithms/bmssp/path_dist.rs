use std::cmp::Ordering;
use std::fmt;

/// 终点为 `end` 的一条最短路在算法中的全序关键字：距离、跳数、终点、前驱。
///
/// 比较顺序：`dis` → `hop` → `end` → `pred`，全部相同则相等。
///
/// 存储：`dis`(64) · `hop`(32) · `end`(32) 压入 [`PathDist::packed`]，`pred` 为 [`PathDist::pred`]。
/// 这样 [`Ord`] 可先比较一个 `u128` 再比较 `pred`。
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PathDist {
    /// 高 64：`dis`；次 32：`hop`；低 32：`end`。
    pub packed: u128,
    pub pred: u32,
}

impl PathDist {
    pub const MAX: Self = Self {
        packed: u128::MAX,
        pred: u32::MAX,
    };

    #[inline]
    pub const fn new(dis: u64, hop: u32, end: u32, pred: u32) -> Self {
        Self {
            packed: ((dis as u128) << 64) | ((hop as u128) << 32) | (end as u128),
            pred,
        }
    }

    #[inline]
    pub const fn dis(self) -> u64 {
        (self.packed >> 64) as u64
    }

    #[inline]
    pub const fn hop(self) -> u32 {
        ((self.packed >> 32) & 0xFFFF_FFFF) as u32
    }

    #[inline]
    pub const fn end(self) -> u32 {
        (self.packed & 0xFFFF_FFFF) as u32
    }

    /// 由「标量」上界 `B` 得到四元组上界：在 `dis == B` 时对 `hop/end/pred` 取最大，
    /// 使得所有满足 `dis <= B` 的路径均 < 该上界
    #[inline]
    pub fn scalar_upper(dis: u64) -> Self {
        Self::new(dis, u32::MAX, u32::MAX, u32::MAX)
    }

    /// `hop = 0`, `pred = 0`。与旧实现里按 `(dis, key)` 排序一致（`key` 即 `end`）。
    #[inline]
    pub fn from_dis(dis: u64, end: usize) -> Self {
        debug_assert!(end <= u32::MAX as usize);
        Self::new(dis, 0, end as u32, 0)
    }
}

impl Ord for PathDist {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        match self.packed.cmp(&other.packed) {
            Ordering::Equal => self.pred.cmp(&other.pred),
            o => o,
        }
    }
}

impl PartialOrd for PathDist {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Debug for PathDist {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PathDist")
            .field("dis", &self.dis())
            .field("hop", &self.hop())
            .field("end", &self.end())
            .field("pred", &self.pred)
            .finish()
    }
}
