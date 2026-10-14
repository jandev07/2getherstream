package app.together.v2.player

import kotlinx.coroutines.*
import org.junit.Assert.*
import org.junit.Test
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit

class PlayerCleanupTest {
    @Test fun cancelledPlaybackDoesNotDiscardQueuedSurfaceRelease() = runBlocking {
        val worker = Dispatchers.IO.limitedParallelism(1)
        val entered = CountDownLatch(1)
        val release = CountDownLatch(1)
        val playback = CoroutineScope(SupervisorJob() + worker)
        val blocker = playback.launch {
            entered.countDown()
            check(release.await(5, TimeUnit.SECONDS))
        }
        val cleanup = PlayerCleanup(worker) { fail("Unexpected cleanup failure") }
        try {
            assertTrue(entered.await(5, TimeUnit.SECONDS))
            val events = mutableListOf<String>()
            val surface = cleanup.enqueue { events.add("surface released") }
            playback.cancel()
            val native = cleanup.finish { events.add("native closed") }
            // Cancellation happened before either release body could execute.
            assertFalse(surface.isCompleted)
            release.countDown()
            withTimeout(5_000) { surface.join(); native.join(); blocker.join() }
            assertEquals(listOf("surface released", "native closed"), events)
            assertFalse(surface.isCancelled)
            assertFalse(native.isCancelled)
        } finally {
            release.countDown()
            playback.cancel()
        }
    }

    @Test fun finishClosesAdmissionAfterFinalRelease() = runBlocking {
        val cleanup = PlayerCleanup(Dispatchers.Unconfined) { fail("Unexpected cleanup failure") }
        var releases = 0
        cleanup.enqueue { releases++ }.join()
        cleanup.finish { releases++ }.join()
        val late = cleanup.enqueue { releases++ }
        late.join()
        assertTrue(late.isCancelled)
        assertEquals(2, releases)
    }

    @Test fun failedSurfaceCleanupIsReportedWithoutCancellingNativeCleanup() = runBlocking {
        var failures = 0
        var nativeClosed = false
        val cleanup = PlayerCleanup(Dispatchers.Unconfined) { failures++ }
        val surface = cleanup.enqueue { throw IllegalStateException("private native detail") }
        val native = cleanup.finish { nativeClosed = true }
        surface.join(); native.join()
        assertTrue(surface.isCancelled)
        assertFalse(native.isCancelled)
        assertTrue(nativeClosed)
        assertEquals(1, failures)
    }
}
