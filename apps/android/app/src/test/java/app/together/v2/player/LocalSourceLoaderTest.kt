package app.together.v2.player

import java.io.File
import kotlinx.coroutines.*
import org.junit.Assert.*
import org.junit.Test

class LocalSourceLoaderTest {
    @Test fun sessionTakeoverRejectsLateNonCancellablePreparation()=runBlocking {
        val scope=CoroutineScope(coroutineContext+Dispatchers.Unconfined)
        val entered=CompletableDeferred<Unit>();val release=CompletableDeferred<Unit>()
        var deliveries=0
        val loader=LocalSourceLoader(scope,prepare={
            entered.complete(Unit)
            withContext(NonCancellable) {release.await();File("obsolete-local.mp4")}
        },load={deliveries++},failed={fail("obsolete preparation reported failure")})
        loader.request();entered.await();loader.cancel();release.complete(Unit)
        yield();assertEquals(0,deliveries)
    }
    @Test fun newerLocalRequestReplacesPendingOneAndDeliversOnce()=runBlocking {
        val scope=CoroutineScope(coroutineContext+Dispatchers.Unconfined)
        val release=CompletableDeferred<Unit>();var requests=0
        val deliveries=mutableListOf<String>()
        val loader=LocalSourceLoader(scope,prepare={
            if(++requests==1) withContext(NonCancellable) {release.await();File("obsolete.mp4")}
            else File("current.mp4")
        },load={deliveries.add(it.name)},failed={fail("unexpected preparation failure")})
        loader.request();loader.request();release.complete(Unit);yield()
        assertEquals(listOf("current.mp4"),deliveries)
        loader.cancel();loader.cancel()
    }
    @Test fun preparationFailureIsContainedAndNextRequestCanSucceed()=runBlocking {
        var unhandled=0;var failures=0;var attempts=0;var deliveries=0
        val owner=SupervisorJob()
        val scope=CoroutineScope(owner+Dispatchers.Unconfined+CoroutineExceptionHandler {_,_ ->unhandled++})
        try {
            val loader=LocalSourceLoader(scope,prepare={
                if(++attempts==1) throw java.io.IOException("private path must not escape")
                File("current.mp4")
            },load={deliveries++},failed={failures++})
            loader.request();yield()
            assertEquals(1,failures);assertEquals(0,unhandled)
            loader.request();yield()
            assertEquals(1,deliveries);assertEquals(1,failures)
        } finally {owner.cancel()}
    }
    @Test fun loadHandoffExceptionIsReportedWithoutEscapingOwner()=runBlocking {
        var failures=0;var unhandled=0
        val owner=SupervisorJob()
        val scope=CoroutineScope(owner+Dispatchers.Unconfined+CoroutineExceptionHandler {_,_ ->unhandled++})
        try {
            val loader=LocalSourceLoader(scope,prepare={File("current.mp4")},
                load={throw java.io.IOException("private native error")},failed={failures++})
            loader.request();yield();assertEquals(1,failures);assertEquals(0,unhandled)
        } finally {owner.cancel()}
    }
    @Test fun cancelledPreparationFailureCannotOverwriteCurrentSession()=runBlocking {
        val scope=CoroutineScope(coroutineContext+Dispatchers.Unconfined)
        val entered=CompletableDeferred<Unit>();val release=CompletableDeferred<Unit>();var failures=0
        val loader=LocalSourceLoader(scope,prepare={
            entered.complete(Unit)
            withContext(NonCancellable) {release.await();throw java.io.IOException("obsolete failure")}
        },load={fail("obsolete delivery")},failed={failures++})
        loader.request();entered.await();loader.cancel();release.complete(Unit);yield()
        assertEquals(0,failures)
    }

}
