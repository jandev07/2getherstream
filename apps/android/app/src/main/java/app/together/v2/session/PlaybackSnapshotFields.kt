package app.together.v2.session

/** Validate playback fields once at acceptance, before native/loading effects.
 * Bounds follow the existing nonnegative signed-millisecond wire representation. */
internal object PlaybackSnapshotFields {
    fun epoch(value:Any?):String? = value as? String
    fun terminal(ended:Any?,fault:Any?):Boolean = ended is Boolean && (fault==null || fault is String)
    private fun nonnegative(value: Any?): Boolean = MediaGrantNumbers.integer(value)?.let { it >= 0 } == true
    fun timeline(position: Any?, anchor: Any?, playing: Any?, desiredPlaying: Any?): Boolean =
        nonnegative(position) && nonnegative(anchor) && playing is Boolean && desiredPlaying is Boolean
    fun preparation(generation: Any?, position: Any?, deadline: Any?, expectedGeneration: Long): Boolean =
        expectedGeneration >= 0 && MediaGrantNumbers.integer(generation) == expectedGeneration &&
            nonnegative(position) && nonnegative(deadline)
    fun participant(connected:Any?,buffering:Any?,rtt:Any?,fault:Any?):Boolean =
        connected is Boolean && buffering is Boolean && nonnegative(rtt) &&
            (fault==null || PlayerFaultAuthority.known(fault))
    fun execution(generation:Any?,playing:Any?,position:Any?,observed:Any?,received:Any?):Boolean =
        nonnegative(generation) && playing is Boolean && nonnegative(position) &&
            nonnegative(observed) && nonnegative(received)
}
