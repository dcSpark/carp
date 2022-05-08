use std::time::Duration;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PerfAggregator {
    pub block_fetch: Duration,
    pub block_parse: Duration,
    pub block_insertion: Duration,
    pub rollback: Duration,
    pub overhead: Duration,
}
impl PerfAggregator {
    pub fn new() -> Self {
        Self {
            block_fetch: Duration::new(0, 0),
            block_parse: Duration::new(0, 0),
            block_insertion: Duration::new(0, 0),
            rollback: Duration::new(0, 0),
            overhead: Duration::new(0, 0),
        }
    }
    pub fn set_overhead(&mut self, total_duration: &Duration, tasks: Duration) {
        let non_duration_sum =
            self.block_fetch + self.block_parse + self.block_insertion + self.rollback + tasks;
        self.overhead = *total_duration - non_duration_sum
    }
}
impl std::ops::Add for PerfAggregator {
    type Output = PerfAggregator;

    fn add(self, other: Self) -> Self {
        Self {
            block_fetch: self.block_fetch + other.block_fetch,
            block_parse: self.block_parse + other.block_parse,
            block_insertion: self.block_insertion + other.block_insertion,
            rollback: self.rollback + other.rollback,
            overhead: self.overhead + other.overhead,
        }
    }
}
impl std::ops::AddAssign for PerfAggregator {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other
    }
}
