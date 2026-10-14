package app.together.v2.player

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.update
import org.junit.Assert.*
import org.junit.Test
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicInteger

/** Actual status/CAS transform; no Android fixture IO, worker or JNI invocation. */
class LocalPreparationStatusTest {
    @Test fun localNoticeDoesNotInventFaultOrDiscardNativeObservation() {
        val current=PlayerStatus(state="Paused",sourceRevision=6,acceptedSourceRevision=6,
            sourceReady=true,positionMs=500,sampledAtMs=123)
        val after=current.withLocalPreparationFailure()
        assertNull(after.failure);assertEquals(current.copy(error=after.error),after)
        assertEquals("Could not prepare the sample video. Try loading it again.",after.error)
    }
    @Test fun pendingNativeFaultAndItsErrorHavePrecedence() {
        for(code in listOf("load_failed","decoder_failed","command_failed","unexpected_end")) {
            val current=PlayerStatus(state="Failed",failure=code,error="Original fixed native error",sourceReady=false)
            assertSame(current,current.withLocalPreparationFailure())
        }
    }
    @Test fun atomicRetryKeepsFaultArrivingDuringNoticeUpdate() {
        val status=MutableStateFlow(PlayerStatus(sourceRevision=7,state="Playing",sourceReady=true))
        val entered=CountDownLatch(1);val release=CountDownLatch(1);val finished=CountDownLatch(1)
        val attempts=AtomicInteger();var problem:Throwable?=null
        val writer=Thread {
            try {
                status.update {current ->
                    if(attempts.incrementAndGet()==1) {entered.countDown();check(release.await(5,TimeUnit.SECONDS))}
                    current.withLocalPreparationFailure()
                }
            } catch(error:Throwable) {problem=error} finally {finished.countDown()}
        }
        writer.start()
        try {
            assertTrue(entered.await(5,TimeUnit.SECONDS))
            status.update {it.withCommandFailure()};release.countDown()
            assertTrue(finished.await(5,TimeUnit.SECONDS));assertNull(problem)
            assertTrue(attempts.get()>=2)
            assertEquals("command_failed",status.value.failure);assertEquals("Failed",status.value.state)
            assertEquals("Player operation failed",status.value.error);assertFalse(status.value.sourceReady)
        } finally {release.countDown();writer.join(5000)}
    }
}
