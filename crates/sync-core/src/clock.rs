use std::collections::VecDeque;

#[derive(Debug, Clone, Copy)]
pub struct Sample { pub offset_ms: f64, pub rtt_ms: f64, pub received_ms: f64 }
/// Together's interval estimate; not an upstream Syncplay/Synkplay algorithm.
/// Bounds assume nonnegative transit delay and constant relative clock offset
/// during the fresh window. They are not measured synchronization accuracy.
#[derive(Debug, Clone, Copy)]
pub struct IntervalEstimate {
    pub sample: Sample,
    pub lowest_rtt_offset_ms: f64,
    pub lower_offset_ms: Option<f64>,
    pub upper_offset_ms: Option<f64>,
    pub support_samples: usize,
    pub oldest_received_ms: f64,
}
#[derive(Default)]
pub struct ClockEstimator { samples: VecDeque<Sample> }
impl ClockEstimator {
    /// t0/t3: client monotonic clock; t1/t2: server monotonic clock.
    pub fn observe(&mut self, t0: f64, t1: f64, t2: f64, t3: f64) -> Result<Sample, &'static str> {
        if [t0, t1, t2, t3].iter().any(|v| !v.is_finite() || *v < 0.0) || t3 < t0 || t2 < t1 { return Err("invalid clock sample"); }
        let rtt = (t3 - t0) - (t2 - t1);
        if !(0.0..=1500.0).contains(&rtt) { return Err("invalid RTT"); }
        let offset_ms = ((t1 - t0) + (t2 - t3)) / 2.0;
        // Finite inputs can still overflow the derived offset. Never publish an
        // unusable calibration or let it displace the last good sample.
        if !offset_ms.is_finite() { return Err("invalid clock offset"); }
        let sample = Sample { offset_ms, rtt_ms: rtt, received_ms: t3 };
        self.samples.push_back(sample);
        // At the client's 500 ms exchange cadence, eight samples discard a
        // still-fresh low-RTT calibration after ~4 s. Retain the advertised
        // 30 s freshness window with bounded space (64 samples ~32 s).
        while self.samples.len() > 64 { self.samples.pop_front(); }
        Ok(sample)
    }
    pub fn estimate(&self, now_client_ms: f64) -> Option<Sample> {
        self.samples.iter().filter(|s| now_client_ms >= s.received_ms && now_client_ms - s.received_ms <= 30_000.0)
            .min_by(|a,b| a.rtt_ms.total_cmp(&b.rtt_ms)).copied()
    }
    pub fn estimate_interval(&self, now_client_ms: f64) -> Option<IntervalEstimate> {
        let best = self.estimate(now_client_ms)?;
        let mut lower = f64::NEG_INFINITY;
        let mut upper = f64::INFINITY;
        let mut support_samples = 0;
        let mut oldest_received_ms = f64::INFINITY;
        for sample in self.samples.iter().filter(|s| now_client_ms >= s.received_ms && now_client_ms - s.received_ms <= 30_000.0) {
            // Equivalent to [t2-t3, t1-t0]. Each exchange constrains the
            // offset without assuming equal forward and return transit time.
            lower = lower.max(sample.offset_ms - sample.rtt_ms / 2.0);
            upper = upper.min(sample.offset_ms + sample.rtt_ms / 2.0);
            support_samples += 1;
            oldest_received_ms = oldest_received_ms.min(sample.received_ms);
        }
        let consistent = lower.is_finite() && upper.is_finite() && lower <= upper;
        let offset_ms = if consistent { lower / 2.0 + upper / 2.0 } else { best.offset_ms };
        Some(IntervalEstimate {
            sample: Sample { offset_ms, ..best }, lowest_rtt_offset_ms: best.offset_ms,
            lower_offset_ms: consistent.then_some(lower),
            upper_offset_ms: consistent.then_some(upper), support_samples, oldest_received_ms,
        })
    }
    pub fn reset(&mut self) { self.samples.clear(); }
}

#[cfg(test)]
mod tests {
    use super::ClockEstimator;

    #[test]
    fn derived_offset_overflow_preserves_good_calibration() {
        for (t0, t1, t2, t3) in [
            (0.0, f64::MAX, f64::MAX, 0.0),
            (f64::MAX, 0.0, 0.0, f64::MAX),
        ] {
            let mut clock = ClockEstimator::default();
            clock.observe(100.0, 620.0, 622.0, 142.0).unwrap();
            assert_eq!(clock.observe(t0, t1, t2, t3).unwrap_err(), "invalid clock offset");
            let sample = clock.estimate(142.0).unwrap();
            assert_eq!(sample.offset_ms, 500.0);
            assert_eq!(sample.rtt_ms, 40.0);
            assert_eq!(clock.samples.len(), 1);
        }
    }
}
