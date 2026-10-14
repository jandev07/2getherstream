package app.together.v2.session

/** No URL/credential retention: binds an accepted native file to PC authority. */
internal data class MediaSourceBinding(val epoch:String,val providerRevision:Long,val nativeRevision:Long) {
    fun reusable(currentEpoch:String,currentProviderRevision:Long,currentNativeRevision:Long,
                 acceptedNativeRevision:Long,sourceReady:Boolean,failed:Boolean):Boolean =
        providerRevision>0 && nativeRevision>0 && epoch==currentEpoch && providerRevision==currentProviderRevision &&
            nativeRevision==currentNativeRevision && nativeRevision==acceptedNativeRevision && sourceReady && !failed
}
