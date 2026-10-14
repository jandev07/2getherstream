package app.together.v2.session

/** A recorded different failure cannot authorize discarding the pending native result. */
internal object PlayerFaultAuthority {
    fun known(value:Any?):Boolean = when(value) {
        "load_failed","decoder_failed","command_failed","unexpected_end" -> true
        else -> false
    }
    fun canRecover(nativeFault:String?,recordedFault:Any?,prepare:Boolean,roomFaultAbsent:Boolean,
                   freshAttempt:Boolean):Boolean =
        known(nativeFault) && nativeFault==recordedFault && prepare && roomFaultAbsent && freshAttempt
}
