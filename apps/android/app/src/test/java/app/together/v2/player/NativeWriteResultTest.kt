package app.together.v2.player

import io.github.yuroyami.libmpvkt.*
import org.junit.Assert.*
import org.junit.Test

/** Pinned wrapper result and Kotlin assignment semantics; no native handle/JNI. */
class NativeWriteResultTest {
    private class RejectedWriter {
        @Suppress("UNUSED_PARAMETER")
        operator fun set(property:String,value:String):MpvResult<Unit> =
            MpvResult.Fail(MpvError.PROPERTY_ERROR,"synthetic private native detail")
    }
    @Test fun operatorAssignmentDiscardsActualFailedWrapperResult() {
        val writer=RejectedWriter()
        // The old native-boundary syntax has no exception for the queue to catch.
        writer["pause"]="yes"
        assertTrue(writer.set("pause","yes") is MpvResult.Fail)
    }
    @Test fun explicitConsumptionRejectsWritesBeforeLaterSourceWork() {
        for(property in listOf("pause","speed","android-surface-size")) {
            val writer=RejectedWriter();var laterSourceWork=false
            val error=runCatching {
                writer.set(property,"synthetic").getOrThrow()
                laterSourceWork=true
            }.exceptionOrNull()
            assertNotNull(error);assertFalse(laterSourceWork)
            // Actual application's existing fixed status transform does not expose detail.
            val failed=PlayerStatus(sourceReady=true).withCommandFailure()
            assertEquals("command_failed",failed.failure);assertFalse(failed.sourceReady)
            assertEquals("Player operation failed",failed.error)
        }
    }
    @Test fun successfulWriteAllowsNextOperationAndFailureDoesNotReplacePendingFault() {
        assertEquals(Unit,MpvResult.Ok(Unit).getOrThrow())
        val pending=PlayerStatus(failure="unexpected_end",error="Original fixed failure",sourceReady=false)
        try {
            RejectedWriter().set("pause","yes").getOrThrow()
            fail("Failed wrapper result was accepted")
        } catch(_:Exception) {
            assertSame(pending,pending.withCommandFailure())
        }
    }
}
