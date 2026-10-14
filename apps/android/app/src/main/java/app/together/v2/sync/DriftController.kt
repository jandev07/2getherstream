// SPDX-License-Identifier: AGPL-3.0-only
// Adapted from Synkplay nudgeRate and Syncplay's correction ladder.
// Copyright retained for the respective upstream contributors.
// See THIRD-PARTY-NOTICES.md for pinned sources, licenses and changes.
package app.together.v2.sync

import kotlin.math.pow

/** Seek requests an authoritative prepare barrier; never apply it directly to mpv. */
class DriftController {
    private var generation: Long? = null
    private var level = 0
    private var smoothed: Double? = null
    private var slowed = false
    private var behindSinceMs: Long? = null
    private var lastSampleMs: Long? = null
    fun reset() {
        generation = null; level = 0; smoothed = null; slowed = false; behindSinceMs = null; lastSampleMs = null
    }
    fun update(targetMs: Long, actualMs: Long, nowMs: Long, activeGeneration: Long,
               steady: Boolean, clockFresh: Boolean): Correction {
        require(targetMs >= 0 && actualMs >= 0 && nowMs >= 0 && activeGeneration >= 0)
        if (!steady || !clockFresh || (generation != null && generation != activeGeneration)) {
            reset(); generation = activeGeneration
            return Correction.Rate(1.0)
        }
        generation = activeGeneration
        val previous=lastSampleMs
        if(previous!=null && nowMs<=previous) return Correction.Hold
        val elapsed=previous?.let {nowMs-it} ?: 100L
        lastSampleMs=nowMs
        val diff = (actualMs.toDouble() - targetMs.toDouble()) / 1000.0
        if (diff > 4.0) {
            reset(); generation = activeGeneration; lastSampleMs=nowMs; return Correction.Seek(targetMs)
        }
        if (diff < -1.75) {
            val since = behindSinceMs ?: nowMs.also { behindSinceMs = it }
            if (nowMs - since > 3250 && diff < -5.0) {
                reset(); generation = activeGeneration; lastSampleMs=nowMs; return Correction.Seek(targetMs)
            }
        } else behindSinceMs = null
        if (!slowed && diff > 1.5) {
            slowed = true; level = 0; smoothed = null; return Correction.Rate(0.95)
        } else if (slowed && diff < 0.1) {
            slowed = false; level = 0; smoothed = null; return Correction.Rate(1.0)
        }
        if (slowed) return Correction.Hold
        val alpha=1.0-(1.0-NUDGE_SMOOTHING).pow(elapsed.toDouble()/100.0)
        val filtered = smoothed?.let { it + alpha * (diff - it) } ?: diff
        val nextLevel = when (level) {
            0 -> when { filtered > NUDGE_START -> -1; filtered < -NUDGE_START -> 1; else -> 0 }
            -1 -> if (filtered < NUDGE_STOP) 0 else -1
            else -> if (filtered > -NUDGE_STOP) 0 else 1
        }
        val changed = nextLevel != level
        level = nextLevel; smoothed = filtered
        return if (changed) Correction.Rate(1.0 + level * NUDGE_RATE) else Correction.Hold
    }
    companion object {
        const val NUDGE_RATE = 0.005
        // Together experiment: tighter entry/exit band, same 0.5% rate.
        const val NUDGE_START = 0.075
        const val NUDGE_STOP = 0.02
        const val NUDGE_SMOOTHING = 0.3
    }
}
