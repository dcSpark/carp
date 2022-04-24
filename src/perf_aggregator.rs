use std::time::Duration;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PerfAggregator {
    pub block_parse: Duration,
    pub block_insertion: Duration,
    pub transaction_insert: Duration,
    pub transaction_input_insert: Duration,
    pub transaction_output_insert: Duration,
    pub addr_cred_relation_insert: Duration,
    pub stake_cred_insert: Duration,
    pub addr_insert: Duration,
    pub tx_credential_relation: Duration,
    pub block_fetch: Duration,
    pub rollback: Duration,
    pub overhead: Duration,
}
impl PerfAggregator {
    pub fn new() -> Self {
        Self {
            block_parse: Duration::new(0, 0),
            block_insertion: Duration::new(0, 0),
            transaction_insert: Duration::new(0, 0),
            transaction_input_insert: Duration::new(0, 0),
            transaction_output_insert: Duration::new(0, 0),
            addr_cred_relation_insert: Duration::new(0, 0),
            stake_cred_insert: Duration::new(0, 0),
            addr_insert: Duration::new(0, 0),
            tx_credential_relation: Duration::new(0, 0),
            block_fetch: Duration::new(0, 0),
            rollback: Duration::new(0, 0),
            overhead: Duration::new(0, 0),
        }
    }
    pub fn set_overhead(&mut self, total_duration: &Duration) {
        let non_duration_sum = self.block_parse
            + self.block_insertion
            + self.transaction_insert
            + self.transaction_input_insert
            + self.transaction_output_insert
            + self.addr_cred_relation_insert
            + self.stake_cred_insert
            + self.addr_insert
            + self.tx_credential_relation
            + self.block_fetch
            + self.rollback;
        self.overhead = *total_duration - non_duration_sum
    }
}
impl std::ops::Add for PerfAggregator {
    type Output = PerfAggregator;

    fn add(self, other: Self) -> Self {
        Self {
            block_parse: self.block_parse + other.block_parse,
            block_insertion: self.block_insertion + other.block_insertion,
            transaction_insert: self.transaction_insert + other.transaction_insert,
            transaction_input_insert: self.transaction_input_insert
                + other.transaction_input_insert,
            transaction_output_insert: self.transaction_output_insert
                + other.transaction_output_insert,
            addr_cred_relation_insert: self.addr_cred_relation_insert
                + other.addr_cred_relation_insert,
            stake_cred_insert: self.stake_cred_insert + other.stake_cred_insert,
            addr_insert: self.addr_insert + other.addr_insert,
            tx_credential_relation: self.tx_credential_relation + other.tx_credential_relation,
            block_fetch: self.block_fetch + other.block_fetch,
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
