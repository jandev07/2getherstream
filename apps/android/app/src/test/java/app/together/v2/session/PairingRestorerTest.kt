package app.together.v2.session

import kotlinx.coroutines.*
import org.junit.Assert.*
import org.junit.Test

class PairingRestorerTest {
    @Test fun userChoiceRejectsLateNonCancellableSavedConnection()=runBlocking {
        val scope=CoroutineScope(coroutineContext+Dispatchers.Unconfined)
        val entered=CompletableDeferred<Unit>();val release=CompletableDeferred<Unit>()
        val connections=mutableListOf<String>();var failures=0
        val restore=PairingRestorer(scope,load={entered.complete(Unit);withContext(NonCancellable) {release.await();"old-pc"}},
            deliver={connections.add(it)},failed={failures++})
        restore.start();entered.await();restore.cancel();connections.add("new-pc")
        release.complete(Unit);yield()
        assertEquals(listOf("new-pc"),connections);assertEquals(0,failures)
        restore.close()
    }
    @Test fun obsoleteFailureDoesNotReplaceNewConnectionFeedback()=runBlocking {
        val scope=CoroutineScope(coroutineContext+Dispatchers.Unconfined)
        val release=CompletableDeferred<Unit>();var failures=0;var deliveries=0
        val restore=PairingRestorer<String>(scope,load={withContext(NonCancellable) {release.await();error("obsolete-store-error")}},
            deliver={deliveries++},failed={failures++})
        restore.start();restore.cancel();release.complete(Unit);yield()
        assertEquals(0,failures);assertEquals(0,deliveries);restore.close()
    }
    @Test fun latestRestoreDeliversOnceAndClosedOwnerCannotRestart()=runBlocking {
        val scope=CoroutineScope(coroutineContext+Dispatchers.Unconfined)
        val release=CompletableDeferred<Unit>();val connections=mutableListOf<String>();var requests=0
        val restore=PairingRestorer(scope,load={if(++requests==1) withContext(NonCancellable) {release.await();"old"} else "current"},
            deliver={connections.add(it)},failed={fail("unexpected failure")})
        restore.start();restore.start();release.complete(Unit);yield()
        assertEquals(listOf("current"),connections)
        restore.close();restore.start();assertEquals(2,requests)
    }
    @Test fun currentFailureIsReportedButMissingPairingIsNormal()=runBlocking {
        val scope=CoroutineScope(coroutineContext+Dispatchers.Unconfined);var failures=0
        val missing=PairingRestorer<String>(scope,load={null},deliver={fail("unexpected connection")},failed={failures++})
        missing.start();assertEquals(0,failures);missing.close()
        val broken=PairingRestorer<String>(scope,load={error("unreadable")},deliver={fail("unexpected connection")},failed={failures++})
        broken.start();assertEquals(1,failures);broken.close()
    }
}
