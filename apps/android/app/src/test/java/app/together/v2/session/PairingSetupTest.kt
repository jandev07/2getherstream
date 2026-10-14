package app.together.v2.session

import kotlinx.coroutines.*
import org.junit.Assert.*
import org.junit.Test

class PairingSetupTest {
    @Test fun pendingClearPreventsNewClaimAndSaveUntilCompletion()=runBlocking {
        val setup=PairingSetup();val entered=CompletableDeferred<Unit>();val release=CompletableDeferred<Unit>()
        val effects=mutableListOf<String>()
        val clearing=launch(start=CoroutineStart.UNDISPATCHED) {setup.run {
            entered.complete(Unit);release.await();effects.add("clear");effects.add("disconnect")
        }}
        entered.await();assertTrue(setup.busy.value)
        setup.run {effects.add("claim");effects.add("save")}
        assertTrue(effects.isEmpty());release.complete(Unit);clearing.join()
        assertFalse(setup.busy.value)
        setup.run {effects.add("claim");effects.add("save")}
        assertEquals(listOf("clear","disconnect","claim","save"),effects)
    }
    @Test fun failedChangeReleasesAdmissionForExplicitRetry()=runBlocking {
        val setup=PairingSetup();var attempts=0
        try {setup.run {attempts++;error("synthetic fixed failure")};fail("failure swallowed")}
        catch(_:IllegalStateException) {}
        assertFalse(setup.busy.value);setup.run {attempts++}
        assertEquals(2,attempts);assertFalse(setup.busy.value)
    }
    @Test fun cancellationKeepsAdmissionUntilOwnedNonCancellableWorkFinishes()=runBlocking {
        val setup=PairingSetup();val entered=CompletableDeferred<Unit>();val release=CompletableDeferred<Unit>()
        val worker=launch(start=CoroutineStart.UNDISPATCHED) {setup.run {
            withContext(NonCancellable) {entered.complete(Unit);release.await()}
            ensureActive()
        }}
        entered.await();worker.cancel();yield();assertTrue(setup.busy.value)
        var replacements=0;setup.run {replacements++};assertEquals(0,replacements)
        release.complete(Unit);worker.join();assertFalse(setup.busy.value)
        setup.run {replacements++};assertEquals(1,replacements)
    }
}
