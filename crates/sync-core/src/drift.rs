// SPDX-License-Identifier: AGPL-3.0-only
// Adapted from Synkplay nudgeRate and Syncplay's correction ladder.
// Copyright retained for the respective upstream contributors.
// See THIRD-PARTY-NOTICES.md for pinned sources, licenses and changes.

#[derive(Debug, PartialEq)]
pub enum Correction { Hold, Rate(f64), Seek(u64) }

/// Seek requests a server barrier; it must never directly seek a player.
#[derive(Default)]
pub struct DriftController {
    generation: Option<u64>,
    level: i8,
    smoothed: Option<f64>,
    slowed: bool,
    behind_since_ms: Option<u64>,
    last_sample_ms: Option<u64>,
}
impl DriftController {
    pub fn reset(&mut self) { *self = Self::default(); }
    pub fn update(&mut self, target_ms: u64, actual_ms: u64, now_ms: u64, generation: u64,
                  steady: bool, clock_fresh: bool) -> Correction {
        if !steady || !clock_fresh || self.generation.is_some_and(|g| g != generation) {
            self.reset(); self.generation = Some(generation);
            return Correction::Rate(1.0);
        }
        self.generation = Some(generation);
        // Same physical filter response at different native probe cadences.
        // Never integrate a duplicate or reordered observation twice.
        let elapsed = match self.last_sample_ms {
            Some(previous) if now_ms <= previous => return Correction::Hold,
            Some(previous) => now_ms - previous,
            None => 100,
        };
        self.last_sample_ms = Some(now_ms);
        // Positive means this player is ahead of the authoritative timeline.
        let diff = (actual_ms as f64 - target_ms as f64) / 1000.0;
        if diff > 4.0 {
            self.reset(); self.generation = Some(generation); self.last_sample_ms = Some(now_ms); return Correction::Seek(target_ms);
        }
        if diff < -1.75 {
            let since = *self.behind_since_ms.get_or_insert(now_ms);
            if now_ms.saturating_sub(since) > 3250 && diff < -5.0 {
                self.reset(); self.generation = Some(generation); self.last_sample_ms = Some(now_ms); return Correction::Seek(target_ms);
            }
        } else { self.behind_since_ms = None; }
        // Syncplay's slowdown entry/exit hysteresis.
        if !self.slowed && diff > 1.5 {
            self.slowed = true; self.level = 0; self.smoothed = None;
            return Correction::Rate(0.95);
        } else if self.slowed && diff < 0.1 {
            self.slowed = false; self.level = 0; self.smoothed = None;
            return Correction::Rate(1.0);
        }
        if self.slowed { return Correction::Hold; }
        // Synkplay nudgeRate: filtered drift, separate start/stop thresholds,
        // and a speed command only when the nudge level changes.
        // Together experiment: 75 ms entry / 20 ms exit limits opposing
        // sub-150 ms deviations; retain the subtle 0.5% rate adjustment.
        let alpha = 1.0 - 0.7_f64.powf(elapsed as f64 / 100.0);
        let smoothed = self.smoothed.map(|old| old + alpha * (diff - old)).unwrap_or(diff);
        let level = match self.level {
            0 => if smoothed > 0.075 { -1 } else if smoothed < -0.075 { 1 } else { 0 },
            -1 => if smoothed < 0.02 { 0 } else { -1 },
            _ => if smoothed > -0.02 { 0 } else { 1 },
        };
        self.smoothed = Some(smoothed);
        let changed = level != self.level;
        self.level = level;
        if changed { Correction::Rate(1.0 + level as f64 * 0.005) } else { Correction::Hold }
    }
}
