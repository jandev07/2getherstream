package app.together.v2.diagnostics

import kotlinx.coroutines.*
import org.junit.Assert.*
import org.junit.Test
import java.util.concurrent.atomic.AtomicInteger

class VisibleJournalSamplerTest {
    @Test fun hiddenAndClosedOwnersDoNotKeepWritingOverPlaybackHistory()=runBlocking {
        val scope=CoroutineScope(SupervisorJob()+Dispatchers.Unconfined);val records=AtomicInteger(0)
        val sampler=VisibleJournalSampler(scope,10) {records.incrementAndGet();Unit}
        try {
            sampler.setActive(true)
            withTimeout(2000) {while(records.get()<2) delay(5)}
            sampler.setActive(false);delay(30);val hidden=records.get();delay(50)
            assertEquals(hidden,records.get())
            sampler.setActive(true);withTimeout(2000) {while(records.get()<=hidden) delay(5)}
            sampler.close();delay(30);val closed=records.get();sampler.setActive(true);delay(50)
            assertEquals(closed,records.get())
        } finally {sampler.close();scope.cancel()}
    }
    @Test fun hidingCancelsAnInflightCancellableRecordingWithoutWaitingOnTheUi()=runBlocking {
        val scope=CoroutineScope(SupervisorJob()+Dispatchers.Unconfined)
        val entered=CompletableDeferred<Unit>();val cancelled=CompletableDeferred<Unit>()
        val sampler=VisibleJournalSampler(scope) {entered.complete(Unit);try {awaitCancellation()}finally{cancelled.complete(Unit)}}
        try {sampler.setActive(true);withTimeout(2000) {entered.await()};sampler.setActive(false);withTimeout(2000) {cancelled.await()}}
        finally {sampler.close();scope.cancel()}
    }
}
