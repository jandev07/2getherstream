package app.together.v2.player

import kotlinx.coroutines.*
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.update
import org.junit.Assert.*
import org.junit.Test

/** Actual status and recovery policies; deferred control failure is synthetic, not JNI. */
class NativeCommandFaultTest {
    @Test fun firstCommandFailureInvalidatesReadinessAndPreservesSourceObservation() {
        val before=PlayerStatus(initialized=true,sourceRevision=7,acceptedSourceRevision=7,
            sourceReady=true,positionMs=900,sampledAtMs=123)
        val after=before.withCommandFailure()
        assertEquals("command_failed",after.failure);assertEquals("Player operation failed",after.error)
        assertFalse(after.sourceReady)
        assertEquals(before.sourceRevision,after.sourceRevision)
        assertEquals(before.acceptedSourceRevision,after.acceptedSourceRevision)
        assertEquals(before.positionMs,after.positionMs);assertEquals(before.sampledAtMs,after.sampledAtMs)
    }
    @Test fun pendingFaultAndItsErrorSurviveSecondaryControlFailure() {
        for(code in listOf("load_failed","decoder_failed","unexpected_end","command_failed")) {
            val before=PlayerStatus(failure=code,error="Original fixed failure",sourceReady=false)
            assertSame(before,before.withCommandFailure())
        }
    }
    @Test fun delayedControlFailureConsultsCurrentStatusAndDoesNotAuthorizeWrongRecovery()=runBlocking {
        val status=MutableStateFlow(PlayerStatus(sourceRevision=8,sourceReady=true))
        val entered=CompletableDeferred<Unit>();val release=CompletableDeferred<Unit>()
        val command=launch(start=CoroutineStart.UNDISPATCHED) {
            assertNull(status.value.failure);entered.complete(Unit);release.await()
            status.update {it.withCommandFailure()}
        }
        entered.await();status.update {it.withUnexpectedEnd()}
        release.complete(Unit);command.join()
        assertEquals("unexpected_end",status.value.failure)
        assertFalse(NativeSourceLoadFence.canSubmit(8,8,"command_failed",status.value.failure))
        assertTrue(NativeSourceLoadFence.canSubmit(8,8,"unexpected_end",status.value.failure))
    }
    @Test fun failedRecoveryRetainsFaultUntilMatchingLoadCompletes() {
        val failed=PlayerStatus(sourceRevision=9,failure="decoder_failed",error="Original fixed failure")
        val retryFailed=failed.withCommandFailure()
        assertEquals("decoder_failed",retryFailed.failure)
        val stillFailed=NativeSourceLoadFence.complete(retryFailed,9,"command_failed")
        assertEquals("decoder_failed",stillFailed.failure)
        val recovered=NativeSourceLoadFence.complete(retryFailed,9,"decoder_failed")
        assertNull(recovered.failure);assertNull(recovered.error);assertFalse(recovered.sourceReady)
        assertEquals("command_failed",recovered.withCommandFailure().failure)
    }
    @Test fun secondaryDecoderEventPreservesPendingFaultAndError() {
        for(code in listOf("load_failed","decoder_failed","unexpected_end","command_failed")) {
            val before=PlayerStatus(failure=code,error="Original fixed failure",sourceReady=false)
            assertSame(before,before.withDecoderFailure())
        }
    }
    @Test fun firstDecoderEventStillInvalidatesReadiness() {
        val before=PlayerStatus(sourceRevision=10,sourceReady=true)
        val after=before.withDecoderFailure()
        assertEquals("decoder_failed",after.failure);assertFalse(after.sourceReady)
        assertEquals("Native media playback failed",after.error)
        assertEquals(before.sourceRevision,after.sourceRevision)
    }
    @Test fun delayedDecoderEventCannotReplaceMainClassifiedEnd()=runBlocking {
        val status=MutableStateFlow(PlayerStatus(sourceRevision=11,sourceReady=true))
        val entered=CompletableDeferred<Unit>();val release=CompletableDeferred<Unit>()
        val event=launch(start=CoroutineStart.UNDISPATCHED) {
            assertNull(status.value.failure);entered.complete(Unit);release.await()
            status.update {it.withDecoderFailure()}
        }
        entered.await();status.update {it.withUnexpectedEnd()}
        release.complete(Unit);event.join()
        assertEquals("unexpected_end",status.value.failure);assertFalse(status.value.sourceReady)
    }
}
