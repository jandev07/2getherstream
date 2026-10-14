package app.together.v2.sync

/** One-use echoes for this connection's clock exchanges; no history replay.
 * Scheduling and socket callbacks use the controller's Main dispatcher.
 */
internal class ClockRequests {
    private val pending = LinkedHashSet<Long>()
    fun issue(nowMs: Long): Boolean {
        if (nowMs < 0) return false
        expire(nowMs)
        if (nowMs in pending) return false
        pending.add(nowMs)
        while (pending.size > 16) pending.remove(pending.first())
        return true
    }
    fun consume(echoMs: Double, receivedMs: Long): Boolean {
        if (!echoMs.isFinite() || echoMs < 0 || echoMs.toLong().toDouble() != echoMs) return false
        expire(receivedMs)
        val stamp = echoMs.toLong()
        return pending.remove(stamp) && receivedMs >= stamp && receivedMs - stamp <= 5000
    }
    private fun expire(nowMs: Long) { pending.removeAll { nowMs >= it && nowMs - it > 5000 } }
    fun reset() = pending.clear()
}
