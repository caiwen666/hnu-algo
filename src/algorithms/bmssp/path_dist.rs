use std::cmp::Ordering;
use std::fmt;

/// 终点为 `end` 的一条最短路在算法中的全序关键字：距离、跳数、终点、前驱。
///
/// 比较顺序：`dis` → `hop` → `end` → `pred`，全部相同则相等。
///
/// 编码为两个 u64（共 16 字节）：
/// - `hi`: dis（64 位，主比较键）
/// - `lo`: (hop:22 | end:21 | pred:21)
///
/// 限制：hop < 4194304, end/pred < 2097152
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct PathDist {
    pub hi: u64,
    pub lo: u64,
}

const END_BITS: u32 = 21;
const PRED_BITS: u32 = 21;
const END_MASK: u64 = (1u64 << END_BITS) - 1;
const PRED_MASK: u64 = (1u64 << PRED_BITS) - 1;

impl PathDist {
    pub const MAX: Self = Self {
        hi: u64::MAX,
        lo: u64::MAX,
    };

    #[inline(always)]
    pub const fn new(dis: u64, hop: u32, end: u32, pred: u32) -> Self {
        Self {
            hi: dis,
            lo: ((hop as u64) << (END_BITS + PRED_BITS))
                | ((end as u64) << PRED_BITS)
                | (pred as u64),
        }
    }

    #[inline(always)]
    pub const fn dis(self) -> u64 {
        self.hi
    }

    #[inline(always)]
    pub const fn hop(self) -> u32 {
        (self.lo >> (END_BITS + PRED_BITS)) as u32
    }

    #[inline(always)]
    pub const fn end(self) -> u32 {
        ((self.lo >> PRED_BITS) & END_MASK) as u32
    }

    #[inline(always)]
    pub const fn pred(self) -> u32 {
        (self.lo & PRED_MASK) as u32
    }

    #[inline]
    pub fn scalar_upper(dis: u64) -> Self {
        Self {
            hi: dis,
            lo: u64::MAX,
        }
    }

    #[inline]
    pub fn from_dis(dis: u64, end: usize) -> Self {
        debug_assert!(end <= 0xFF_FFFF);
        Self::new(dis, 0, end as u32, 0)
    }
}

impl Ord for PathDist {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> Ordering {
        match self.hi.cmp(&other.hi) {
            Ordering::Equal => self.lo.cmp(&other.lo),
            o => o,
        }
    }
}

impl PartialOrd for PathDist {
    #[inline(always)]
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
            .field("pred", &self.pred())
            .finish()
    }
}
