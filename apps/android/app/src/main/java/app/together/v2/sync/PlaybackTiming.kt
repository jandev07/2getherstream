package app.together.v2.sync

/** Compare native position with the timeline at the observation time, not at
 * the later control-loop tick. Never extrapolate an old native position. */
object PlaybackTiming {
    private fun validSample(sampledAtClientMs:Long,nowClientMs:Long,offsetMs:Double):Boolean =
        sampledAtClientMs>0 && nowClientMs>=sampledAtClientMs && nowClientMs-sampledAtClientMs<=1000 && offsetMs.isFinite()
    fun serverAtSample(sampledAtClientMs:Long,nowClientMs:Long,offsetMs:Double):Long? {
        if(!validSample(sampledAtClientMs,nowClientMs,offsetMs)) return null
        val observed=sampledAtClientMs+offsetMs
        // Double.toLong saturates oversized finite values instead of failing.
        if(!observed.isFinite() || observed>=Long.MAX_VALUE.toDouble()) return null
        return observed.toLong().coerceAtLeast(0)
    }
    fun targetAtSample(positionMs:Long, anchorServerMs:Long, playing:Boolean,
                       sampledAtClientMs:Long, nowClientMs:Long, offsetMs:Double):Long? {
        if(positionMs<0 || anchorServerMs<0 || !validSample(sampledAtClientMs,nowClientMs,offsetMs)) return null
        if(!playing) return positionMs
        val observedServer=serverAtSample(sampledAtClientMs,nowClientMs,offsetMs) ?: return null
        val elapsed=(observedServer-anchorServerMs).coerceAtLeast(0)
        if(elapsed>Long.MAX_VALUE-positionMs) return null
        return positionMs+elapsed
    }
}
