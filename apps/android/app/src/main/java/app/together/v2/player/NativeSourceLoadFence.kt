package app.together.v2.player

/** A queued load owns its source revision and the fault state it was admitted with. */
internal object NativeSourceLoadFence {
    fun canSubmit(revision:Long,currentRevision:Long,submittedFault:String?,pendingFault:String?):Boolean =
        revision==currentRevision && submittedFault==pendingFault

    fun complete(current:PlayerStatus,revision:Long,submittedFault:String?):PlayerStatus {
        // Native acceptance is an observation even if authority/failure changed.
        val accepted=current.copy(acceptedSourceRevision=revision)
        return if(canSubmit(revision,current.sourceRevision,submittedFault,current.failure))
            accepted.copy(error=null,failure=null) else accepted
    }
}
