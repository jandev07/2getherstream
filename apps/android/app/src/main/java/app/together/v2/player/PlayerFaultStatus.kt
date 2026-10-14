package app.together.v2.player

/** EOF interpretation must not overwrite a terminal result published by the worker. */
internal fun PlayerStatus.withUnexpectedEnd():PlayerStatus =
    if(failure==null) copy(state="Failed",failure="unexpected_end",sourceReady=false) else this

/** A secondary control failure cannot replace the fault awaiting PC recovery. */
internal fun PlayerStatus.withCommandFailure():PlayerStatus =
    if(failure==null) copy(state="Failed",error="Player operation failed",failure="command_failed",sourceReady=false) else this

/** An entry-fenced native event still cannot replace a fault already awaiting recovery. */
internal fun PlayerStatus.withDecoderFailure():PlayerStatus =
    if(failure==null) copy(state="Failed",failure="decoder_failed",sourceReady=false) else this

/** Classify a fresh native observation without hiding the pending terminal fault. */
internal fun nativePlaybackState(failure:String?,durationMs:Long?,buffering:Boolean,paused:Boolean):String =
    if(failure!=null) "Failed" else if(durationMs==null || durationMs==0L) "Idle" else if(buffering) "Buffering" else if(paused) "Paused" else "Playing"

/** A recoverable local IO notice must not mask an already pending native result. */
internal fun PlayerStatus.withLocalPreparationFailure():PlayerStatus =
    if(failure==null) copy(error="Could not prepare the sample video. Try loading it again.") else this
