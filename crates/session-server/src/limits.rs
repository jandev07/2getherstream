//! Bounded monotonic token buckets. No caller-supplied timestamps or IP headers.
pub(crate) struct Bucket {tokens:f64,last_ms:u64,capacity:f64,per_second:f64}
impl Bucket {
    pub(crate) fn new(capacity:u32,per_second:u32,now:u64)->Self {Self {tokens:capacity as f64,last_ms:now,capacity:capacity as f64,per_second:per_second as f64}}
    pub(crate) fn take(&mut self,now:u64)->bool {
        self.tokens=(self.tokens+now.saturating_sub(self.last_ms) as f64*self.per_second/1000.0).min(self.capacity);
        self.last_ms=self.last_ms.max(now);
        if self.tokens<1.0 {false} else {self.tokens-=1.0;true}
    }
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn bursts_are_bounded_and_monotonic_refill_cannot_be_replayed() {
        let mut b=Bucket::new(3,2,0);for _ in 0..3 {assert!(b.take(0));}
        assert!(!b.take(0));assert!(!b.take(499));assert!(b.take(500));assert!(!b.take(500));
        assert!(!b.take(100));assert!(b.take(1000));assert!(!b.take(1000));
        for _ in 0..3 {assert!(b.take(1_000_000));}assert!(!b.take(1_000_000));
    }
    #[test] fn reconnect_pacing_respects_sustained_upgrade_budget() {
        let mut bucket=Bucket::new(60,1,0);
        for cycle in 0..1800 {assert!(bucket.take(cycle*1000));}
    }
    #[test] fn faster_reconnects_eventually_exhaust_burst_allowance() {
        let mut bucket=Bucket::new(60,1,0);
        assert!((0..1800).any(|cycle|!bucket.take(cycle*900)));
    }
}
