package app.together.v2.session

import kotlin.random.Random

/** Monotonic time only. A momentary successful handshake must not reset backoff. */
internal class ReconnectPolicy {
    private var failures=0
    private var healthySince:Long?=null
    fun validatedSnapshot(nowMs:Long) {if(healthySince==null) healthySince=nowMs}
    fun nextDelayMs(nowMs:Long,jitter:Double=Random.nextDouble()):Long {
        require(jitter.isFinite() && jitter in 0.0..1.0)
        if(healthySince?.let {nowMs-it>=30000}==true) failures=0
        healthySince=null
        val base=(1000L shl failures.coerceAtMost(5)).coerceAtMost(30000)
        failures=(failures+1).coerceAtMost(5)
        return (base+(base*0.25*jitter).toLong()).coerceAtMost(30000)
    }
    fun reset() {failures=0;healthySince=null}
}
