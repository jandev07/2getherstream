package app.together.v2.player

import kotlinx.coroutines.*
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.update
import org.junit.Assert.*
import org.junit.Test

/** Actual probe classification/status policy with synthetic observations; no JNI. */
class NativePlaybackStateTest {
    @Test fun pendingFaultHasPrecedenceOverEveryObservedPlaybackState() {
        for(failure in listOf("load_failed","decoder_failed","command_failed","unexpected_end")) {
            for(duration in listOf<Long?>(null,0,60000))for(cache in listOf(false,true))for(paused in listOf(false,true))
                assertEquals("Failed",nativePlaybackState(failure,duration,cache,paused))
        }
    }
    @Test fun normalNativeStatesAndExplicitRecoveryRemainVisible() {
        assertEquals("Idle",nativePlaybackState(null,null,false,false))
        assertEquals("Idle",nativePlaybackState(null,0,true,false))
        assertEquals("Buffering",nativePlaybackState(null,60000,true,true))
        assertEquals("Paused",nativePlaybackState(null,60000,false,true))
        assertEquals("Playing",nativePlaybackState(null,60000,false,false))
        val pending=PlayerStatus(sourceRevision=3,failure="decoder_failed",state="Failed")
        val recovered=NativeSourceLoadFence.complete(pending,3,"decoder_failed")
        assertNull(recovered.failure)
        assertEquals("Paused",nativePlaybackState(recovered.failure,60000,false,true))
    }
    @Test fun publicationMarksFailureBeforeNextProbe() {
        val playing=PlayerStatus(state="Playing",sourceReady=true)
        for(failed in listOf(playing.withUnexpectedEnd(),playing.withCommandFailure(),playing.withDecoderFailure())) {
            assertEquals("Failed",failed.state);assertFalse(failed.sourceReady)
        }
    }
    @Test fun delayedObservationUsesFaultPublishedAfterNativeRead()=runBlocking {
        val status=MutableStateFlow(PlayerStatus(state="Playing",sourceReady=true))
        val entered=CompletableDeferred<Unit>();val release=CompletableDeferred<Unit>()
        val probe=launch(start=CoroutineStart.UNDISPATCHED) {
            assertNull(status.value.failure);entered.complete(Unit);release.await()
            status.update {it.copy(state=nativePlaybackState(it.failure,60000,false,false))}
        }
        entered.await();status.update {it.withDecoderFailure()};release.complete(Unit);probe.join()
        assertEquals("decoder_failed",status.value.failure);assertEquals("Failed",status.value.state)
    }
}
