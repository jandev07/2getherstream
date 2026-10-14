package app.together.v2.session

import kotlinx.coroutines.*
import org.junit.Assert.*
import org.junit.Test

/** Real queued authority token; ACK/PC updates are synthetic, no socket/JNI. */
class QueuedControlTest {
    @Test fun currentAuthorityRetainsTheExactQueuedAction() {
        val action=Any();val queued=QueuedControl("room-a",7L,action)
        assertSame(action,queued.forAuthority("room-a",7L))
    }
    @Test fun generationAndEpochChangesDiscardTheQueuedAction() {
        val queued=QueuedControl("room-a",7L,"pause")
        assertNull(queued.forAuthority("room-a",8L))
        assertNull(queued.forAuthority("room-b",7L))
        assertNull(queued.forAuthority(null,-1L))
    }
    @Test fun pendingAckCannotRebaseEarlierIntentAfterAuthorityReplacement() = runBlocking {
        for(replaceEpoch in listOf(false,true)) {
            var epoch="room-a";var generation=7L
            val queued=QueuedControl(epoch,generation,"seek:1000")
            val waiting=CompletableDeferred<Unit>();val ack=CompletableDeferred<Unit>()
            val sent=mutableListOf<String>()
            val flush=launch {
                waiting.complete(Unit);ack.await()
                queued.forAuthority(epoch,generation)?.let(sent::add)
            }
            waiting.await()
            if(replaceEpoch)epoch="room-b" else generation=8L
            ack.complete(Unit);flush.join();assertTrue(sent.isEmpty())
            val current=QueuedControl(epoch,generation,"seek:2000")
            assertEquals("seek:2000",current.forAuthority(epoch,generation))
        }
    }
    @Test fun successfulMatchedCommandKeepsRapidNextControlThroughItsOwnAdvance() {
        val sent=QueuedControl("room-a",7L,"play")
        val pending=QueuedControl("room-a",7L,"pause")
        val carried=requireNotNull(pending.afterSuccessfulCommand(sent,"room-a",8L))
        assertEquals("pause",carried.forAuthority("room-a",8L))
        assertSame(pending,pending.afterSuccessfulCommand(sent,"room-a",7L))
        assertNull(pending.afterSuccessfulCommand(sent,"room-a",9L))
        assertNull(pending.afterSuccessfulCommand(sent,"room-b",8L))
        assertNull(pending.afterSuccessfulCommand(QueuedControl("room-a",6L,"play"),"room-a",8L))
        val newer=QueuedControl("room-a",8L,"seek:2000")
        assertSame(newer,newer.afterSuccessfulCommand(sent,"room-a",8L))
        val maximum=QueuedControl("room-a",Long.MAX_VALUE,"pause")
        assertNull(maximum.afterSuccessfulCommand(QueuedControl("room-a",Long.MAX_VALUE,"play"),"room-a",Long.MIN_VALUE))
    }
    @Test fun terminalFeedbackDoesNotPromoteEarlierPlayIntoRecoveryOrReplay() {
        val pending=QueuedControl("room-a",7L,"play")
        for(kind in listOf("player_failed","ended")) {
            val sent=QueuedControl("room-a",7L,kind)
            assertNull(pending.afterSuccessfulCommand(sent,"room-a",8L))
            // A fresh intent made against already-recorded terminal authority is valid.
            val fresh=QueuedControl("room-a",8L,"play")
            assertSame(fresh,fresh.afterSuccessfulCommand(sent,"room-a",8L))
            assertSame(pending,pending.afterSuccessfulCommand(sent,"room-a",7L))
        }
        for(kind in listOf("play","pause","seek","buffering","player_recovered")) {
            assertNotNull(pending.afterSuccessfulCommand(QueuedControl("room-a",7L,kind),"room-a",8L))
        }
    }
    @Test fun reportsThatCannotAdvanceGenerationCannotCarryAnOlderIntent() {
        val pending=QueuedControl("room-a",7L,"pause")
        for(kind in listOf("ready","executed","unknown")) {
            val sent=QueuedControl("room-a",7L,kind)
            assertNull(pending.afterSuccessfulCommand(sent,"room-a",8L))
            assertSame(pending,pending.afterSuccessfulCommand(sent,"room-a",7L))
            val current=QueuedControl("room-a",8L,"pause")
            assertSame(current,current.afterSuccessfulCommand(sent,"room-a",8L))
        }
    }
}
