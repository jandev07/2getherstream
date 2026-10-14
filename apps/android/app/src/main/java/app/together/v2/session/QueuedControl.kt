package app.together.v2.session

/** A waiting user intention must not be rebound to a later PC authority. */
internal data class QueuedControl<T>(val epoch:String,val generation:Long,val action:T) {
    fun forAuthority(currentEpoch:String?,currentGeneration:Long):T? =
        action.takeIf {epoch==currentEpoch && generation==currentGeneration}

    /** A matching successful ACK contains the atomic result of our own command.
     * Preserve rapid user controls through that single causal advance, never
     * through a rejected command, terminal feedback or later authority replacement. */
    fun afterSuccessfulCommand(sent:QueuedControl<String>,currentEpoch:String?,currentGeneration:Long):QueuedControl<T>? = when {
        forAuthority(currentEpoch,currentGeneration)!=null -> this
        sent.action!="player_failed" && sent.action!="ended" &&
            epoch==sent.epoch && generation==sent.generation && currentEpoch==epoch &&
            generation<Long.MAX_VALUE && currentGeneration==generation+1 -> copy(generation=currentGeneration)
        else -> null
    }
}
