package app.together.v2.player

import kotlinx.coroutines.*
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.update
import org.junit.Assert.*
import org.junit.Test

class PlayerFaultStatusTest {
    @Test fun prematureEndWithoutPendingFailureMarksSourceUnavailable() {
        val before=PlayerStatus(sourceRevision=3,acceptedSourceRevision=3,sourceReady=true,positionMs=500)
        val after=before.withUnexpectedEnd()
        assertEquals("unexpected_end",after.failure);assertFalse(after.sourceReady)
        assertEquals(before.sourceRevision,after.sourceRevision);assertEquals(before.positionMs,after.positionMs)
    }
    @Test fun everyExistingNativeCodeAndItsPrivateSafeErrorRemainOwned() {
        for(code in listOf("load_failed","decoder_failed","command_failed","unexpected_end")) {
            val before=PlayerStatus(failure=code,error="fixed failure",sourceReady=false)
            assertSame(before,before.withUnexpectedEnd())
        }
    }
    @Test fun delayedEndClassificationUsesWorkerPublishedFailureRatherThanEarlierSnapshot()=runBlocking {
        val status=MutableStateFlow(PlayerStatus(sourceReady=true))
        val entered=CompletableDeferred<Unit>();val release=CompletableDeferred<Unit>()
        val classification=launch(start=CoroutineStart.UNDISPATCHED) {
            val earlier=status.value;assertNull(earlier.failure)
            entered.complete(Unit);release.await()
            status.update {it.withUnexpectedEnd()}
        }
        entered.await();status.update {it.copy(failure="command_failed",error="fixed failure",sourceReady=false)}
        release.complete(Unit);classification.join()
        assertEquals("command_failed",status.value.failure);assertEquals("fixed failure",status.value.error)
    }
}
