package app.together.v2.player

import kotlinx.coroutines.*
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.update
import org.junit.Assert.*
import org.junit.Test

/** Real status/CAS policy, synthetic command-completion barriers; no JNI calls. */
class NativeSourceLoadFenceTest {
    @Test fun pendingFaultBeforeSubmissionRejectsOlderLoad() {
        assertTrue(NativeSourceLoadFence.canSubmit(2,2,null,null))
        assertFalse(NativeSourceLoadFence.canSubmit(2,2,null,"unexpected_end"))
        assertFalse(NativeSourceLoadFence.canSubmit(2,2,"decoder_failed","command_failed"))
        assertTrue(NativeSourceLoadFence.canSubmit(2,2,"decoder_failed","decoder_failed"))
    }
    @Test fun newerSourceRejectsOlderSubmissionAndPreservesItsFaultAtCompletion() {
        assertFalse(NativeSourceLoadFence.canSubmit(2,3,null,null))
        val current=PlayerStatus(sourceRevision=3,failure="decoder_failed",error="fixed failure",sourceReady=false)
        val observed=NativeSourceLoadFence.complete(current,2,null)
        assertEquals(2L,observed.acceptedSourceRevision)
        assertEquals(3L,observed.sourceRevision);assertEquals(current.failure,observed.failure)
        assertEquals(current.error,observed.error);assertFalse(observed.sourceReady)
    }
    @Test fun nativeCompletionPreservesFaultPublishedWhileCommandWasPending()=runBlocking {
        val status=MutableStateFlow(PlayerStatus(sourceRevision=2))
        val entered=CompletableDeferred<Unit>();val release=CompletableDeferred<Unit>()
        val completion=launch(start=CoroutineStart.UNDISPATCHED) {
            entered.complete(Unit);release.await()
            status.update {NativeSourceLoadFence.complete(it,2,null)}
        }
        entered.await();status.update {it.copy(failure="unexpected_end",error="fixed failure",sourceReady=false)}
        release.complete(Unit);completion.join()
        assertEquals("unexpected_end",status.value.failure);assertEquals("fixed failure",status.value.error)
        assertEquals(2L,status.value.acceptedSourceRevision);assertFalse(status.value.sourceReady)
    }
    @Test fun explicitMatchingRecoveryClearsOnlyItsAdmittedFault() {
        val failed=PlayerStatus(sourceRevision=4,failure="decoder_failed",error="fixed failure",sourceReady=false)
        val recovered=NativeSourceLoadFence.complete(failed,4,"decoder_failed")
        assertNull(recovered.failure);assertNull(recovered.error);assertEquals(4L,recovered.acceptedSourceRevision)
        assertFalse(recovered.sourceReady)
        val changed=NativeSourceLoadFence.complete(failed.copy(failure="command_failed"),4,"decoder_failed")
        assertEquals("command_failed",changed.failure);assertEquals("fixed failure",changed.error)
    }
}
