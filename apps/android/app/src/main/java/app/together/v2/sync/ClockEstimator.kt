package app.together.v2.sync

data class ClockSample(val offsetMs: Double, val rttMs: Double, val receivedMs: Double)
/** Together interval estimator; bounds assume constant offset/nonnegative transit
 * within the fresh window. They do not measure actual sync accuracy. */
data class ClockIntervalEstimate(val sample:ClockSample, val lowestRttOffsetMs:Double,
    val lowerOffsetMs:Double?, val upperOffsetMs:Double?, val supportSamples:Int,
    val oldestReceivedMs:Double)
class ClockEstimator {
    private val samples = ArrayDeque<ClockSample>()
    val sampleCount: Int get() = samples.size
    fun observe(t0: Double, t1: Double, t2: Double, t3: Double): ClockSample? {
        // A delayed sample is an estimator outlier, not a broken control connection.
        if(!listOf(t0,t1,t2,t3).all {it.isFinite() && it>=0} || t3<t0 || t2<t1) return null
        val rtt = (t3 - t0) - (t2 - t1)
        if(rtt !in 0.0..1500.0) return null
        val offset = ((t1 - t0) + (t2 - t3)) / 2
        // Finite timestamps can overflow their derived offset; preserve history.
        if(!offset.isFinite()) return null
        val sample = ClockSample(offset, rtt, t3)
        samples.addLast(sample)
        // 64 samples cover the 30 s freshness window at our 500 ms cadence.
        // Eight samples evicted good calibration after only ~4 seconds.
        while (samples.size > 64) samples.removeFirst()
        return sample
    }
    fun estimate(nowClientMs: Double): ClockSample? = samples.filter {
        nowClientMs >= it.receivedMs && nowClientMs - it.receivedMs <= 30_000
    }.minByOrNull { it.rttMs }
    fun estimateInterval(nowClientMs:Double):ClockIntervalEstimate? {
        val best=estimate(nowClientMs) ?: return null
        var lower=Double.NEGATIVE_INFINITY
        var upper=Double.POSITIVE_INFINITY
        var count=0
        var oldest=Double.POSITIVE_INFINITY
        for(sample in samples) {
            if(nowClientMs<sample.receivedMs || nowClientMs-sample.receivedMs>30_000) continue
            // Equivalent to [t2-t3, t1-t0]; no equal-transit assumption.
            lower=maxOf(lower,sample.offsetMs-sample.rttMs/2)
            upper=minOf(upper,sample.offsetMs+sample.rttMs/2)
            count++
            oldest=minOf(oldest,sample.receivedMs)
        }
        val consistent=lower.isFinite() && upper.isFinite() && lower<=upper
        val offset=if(consistent) lower/2+upper/2 else best.offsetMs
        return ClockIntervalEstimate(best.copy(offsetMs=offset),best.offsetMs,
            if(consistent) lower else null,if(consistent) upper else null,count,oldest)
    }
    fun reset() = samples.clear()
}
sealed interface Correction {
    data object Hold : Correction
    data class Rate(val value: Double) : Correction
    data class Seek(val positionMs: Long) : Correction
}
