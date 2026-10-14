package app.together.v2.update

import java.io.File
import java.io.IOException
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicReference
import kotlinx.coroutines.CancellationException
import org.junit.Assert.*
import org.junit.Test

class UpdateArtifactTest {
    private fun isolated(test:(File)->Unit) {
        val directory=kotlin.io.path.createTempDirectory("update-artifact-test-").toFile()
        try {test(File(directory,"isolated-payload.bin"))}finally {directory.deleteRecursively()}
    }
    @Test fun cancelledWaiterCannotLeaveFileCreatedLaterByWorker()=isolated {file ->
        val active=AtomicBoolean(true);val entered=CountDownLatch(1);val release=CountDownLatch(1)
        val failure=AtomicReference<Throwable?>()
        val worker=Thread {
            try {writeUpdateArtifact(file,{active.get()}) {
                entered.countDown();check(release.await(3,TimeUnit.SECONDS))
                file.writeText("isolated partial bytes");file
            }} catch(error:Throwable) {failure.set(error)}
        }.apply {isDaemon=true;start()}
        try {
            assertTrue(entered.await(3,TimeUnit.SECONDS))
            active.set(false);assertFalse(file.delete()) // Waiter cleanup ran before creation.
            release.countDown();worker.join(3000);assertFalse(worker.isAlive)
            assertTrue(failure.get() is CancellationException)
            assertFalse("Late worker file must be removed",file.exists())
        } finally {release.countDown();worker.join(3000)}
    }
    @Test fun alreadyCancelledOwnerDoesNotBeginWrite()=isolated {file ->
        var began=false
        try {writeUpdateArtifact(file,{false}) {began=true;file.writeText("partial")};fail("Expected cancellation")}
        catch(_:CancellationException) {}
        assertFalse(began);assertFalse(file.exists())
    }
    @Test fun cancellationAfterVerificationDiscardsResultAndFile()=isolated {file ->
        var active=true
        try {writeUpdateArtifact(file,{active}) {file.writeText("synthetic verified bytes");active=false;file};fail("Expected cancellation")}
        catch(_:CancellationException) {}
        assertFalse(file.exists())
    }
    @Test fun writeOrVerificationFailureRemovesPartialArtifact()=isolated {file ->
        try {writeUpdateArtifact(file,{true}) {file.writeText("partial");throw IOException("synthetic verification failure")};fail("Expected failure")}
        catch(_:IOException) {}
        assertFalse(file.exists())
    }
    @Test fun currentCompletedArtifactIsRetainedForCaller()=isolated {file ->
        val result=writeUpdateArtifact(file,{true}) {file.writeText("synthetic verified bytes");file}
        assertEquals(file,result);assertEquals("synthetic verified bytes",file.readText())
    }
}
