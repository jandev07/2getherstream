package app.together.v2

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import app.together.v2.player.LibMpvAdapter
import app.together.v2.player.NativeLifecycleProof
import io.github.yuroyami.libmpvkt.*
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.first
import org.json.JSONArray
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith
import java.io.File
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicInteger
import java.util.concurrent.atomic.AtomicReference

/** Real native command rejection on the existing serial worker. No decoder
 * crash, session recovery authority or presented-frame accuracy is simulated. */
@RunWith(AndroidJUnit4::class)
@OptIn(InternalLibmpvKtApi::class)
class NativeControlFaultTest {
    @Test fun healthyHeadlessLoadPlayPauseCloses()=runBlocking<Unit> {healthyClose(waitForProgress=true)}
    @Test fun immediateHeadlessStartPauseCloses()=runBlocking<Unit> {healthyClose(waitForProgress=false)}

    private suspend fun healthyClose(waitForProgress:Boolean) {
        val context=InstrumentationRegistry.getInstrumentation().targetContext
        val fixture=File(context.filesDir,"healthy-headless-close-proof.mp4")
        context.assets.open("clock-test.mp4").use {input ->fixture.outputStream().use {input.copyTo(it)}}
        val cleaned=CountDownLatch(1)
        val nativeClosed=AtomicInteger()
        var constructed:LibMpvAdapter?=null
        try {
            val player=withContext(Dispatchers.Main) {
                LibMpvAdapter(context,NativeLifecycleProof({}, {}, {nativeClosed.incrementAndGet()}, {cleaned.countDown()})).also {constructed=it}
            }
            assertTrue(withTimeout(15000) {player.status.first {it.initialized || it.error!=null}}.initialized)
            val revision=withContext(Dispatchers.Main) {player.loadLocal(fixture,null)}
            withTimeout(15000) {player.status.first {it.acceptedSourceRevision==revision && it.sourceReady}}
            withContext(Dispatchers.Main) {player.pause(false)}
            if(waitForProgress) withTimeout(10000) {player.status.first {it.state=="Playing" && it.positionMs>=100}}
            val paused=CountDownLatch(1)
            withContext(Dispatchers.Main) {
                player.pause(true)
                player.enqueueWorkerBarrierForProof {paused.countDown()}
            }
            assertTrue("pause command completion",paused.await(10,TimeUnit.SECONDS))
        } finally {
            withContext(NonCancellable+Dispatchers.Main) {constructed?.close()}
            if(constructed!=null) assertTrue("healthy native cleanup timed out",cleaned.await(20,TimeUnit.SECONDS))
            assertEquals("native close must return exactly once",1,nativeClosed.get())
            assertTrue(fixture.delete())
        }
    }

    @Test fun pendingCommandFaultRejectsQueuedStartAndCorrectionButAllowsFreezeAndRecovery()=runBlocking<Unit> {
        val context=InstrumentationRegistry.getInstrumentation().targetContext
        val fixture=File(context.filesDir,"queued-control-fault-proof.mp4")
        val receipt=File(context.filesDir,"native-control-fault-proof.json")
        val rows=JSONArray()
        receipt.writeText(rows.toString())
        File(context.filesDir,"native-control-fault-cleanup-stacks.txt").delete()
        context.assets.open("clock-test.mp4").use {input ->fixture.outputStream().use {input.copyTo(it)}}
        val cleaned=CountDownLatch(1)
        val nativeClosed=AtomicInteger()
        var constructed:LibMpvAdapter?=null
        var heldRelease:CountDownLatch?=null
        var bodyFailure:Throwable?=null
        try {
            val player=withContext(Dispatchers.Main) {
                LibMpvAdapter(context,NativeLifecycleProof({}, {}, {nativeClosed.incrementAndGet()}, {cleaned.countDown()})).also {constructed=it}
            }
            assertTrue(withTimeout(15000) {player.status.first {it.initialized || it.error!=null}}.initialized)
            val field=LibMpvAdapter::class.java.getDeclaredField("core").apply {isAccessible=true}
            suspend fun observe(phase:String):JSONObject {
                val done=CountDownLatch(1)
                val result=AtomicReference<JSONObject?>()
                val failure=AtomicReference<Throwable?>()
                withContext(Dispatchers.Main) {
                    player.enqueueWorkerBarrierForProof {
                        try {
                            val core=field.get(player) as Mpv
                            result.set(JSONObject().put("phase",phase)
                                .put("paused",core.get(MpvProperties.Pause).getOrThrow())
                                .put("rate",core.get(MpvProperties.Speed).getOrThrow())
                                .put("failure",player.status.value.failure ?: JSONObject.NULL))
                        } catch(error:Throwable) {failure.set(error)}
                        finally {done.countDown()}
                    }
                }
                assertTrue("native observation timed out",done.await(10,TimeUnit.SECONDS))
                assertNull("native property observation rejected",failure.get())
                return checkNotNull(result.get()).also {rows.put(it);receipt.writeText(rows.toString())}
            }
            suspend fun load(admittedFault:String?) {
                val revision=withContext(Dispatchers.Main) {player.loadLocal(fixture,admittedFault)}
                // The old pending fault remains visible until this queued load
                // is accepted. Its new requested revision alone is not a result.
                val ready=withTimeout(15000) {player.status.first {it.sourceRevision==revision && it.acceptedSourceRevision==revision && it.sourceReady}}
                assertNull("explicit load failed",ready.failure)
                assertTrue(ready.sourceReady)
            }
            load(null)
            for((initial,queued) in listOf(1.0 to 1.005,1.005 to 0.995)) {
                withContext(Dispatchers.Main) {player.speed(initial)}
                assertEquals(initial,observe("healthy_setup").getDouble("rate"),0.000001)
                val entered=CountDownLatch(1);val release=CountDownLatch(1)
                heldRelease=release
                withContext(Dispatchers.Main) {
                    player.enqueueWorkerBarrierForProof {
                        entered.countDown();check(release.await(20,TimeUnit.SECONDS))
                        // The actual queue catch classifies the pinned native API's
                        // rejected unknown command before following controls run.
                        (field.get(player) as Mpv).command(MpvCommand.of("2gether-invalid-control-proof")).getOrThrow()
                    }
                }
                assertTrue("fault barrier not entered",entered.await(10,TimeUnit.SECONDS))
                withContext(Dispatchers.Main) {player.pause(false);player.speed(queued)}
                release.countDown()
                val fault=observe("pending_fault_queued_start_and_rate")
                assertEquals("command_failed",fault.getString("failure"))
                assertTrue("queued start ran after native command failure",fault.getBoolean("paused"))
                assertEquals("queued correction ran after native command failure",initial,fault.getDouble("rate"),0.000001)
                withContext(Dispatchers.Main) {player.pause(true);player.speed(1.0)}
                val frozen=observe("fault_freeze_and_reset")
                assertTrue(frozen.getBoolean("paused"));assertEquals(1.0,frozen.getDouble("rate"),0.000001)
                assertEquals("command_failed",frozen.getString("failure"))
                load("command_failed")
                withContext(Dispatchers.Main) {player.pause(false);player.speed(queued)}
                val recovered=observe("explicit_adapter_recovery")
                assertFalse(recovered.getBoolean("paused"));assertEquals(queued,recovered.getDouble("rate"),0.000001)
                assertTrue(recovered.isNull("failure"))
                withContext(Dispatchers.Main) {player.pause(true);player.speed(1.0)}
                observe("next_case_paused")
            }
        } catch(error:Throwable) {bodyFailure=error;throw error}
        finally {
            heldRelease?.countDown()
            // Retain an isolated Java worker stack if this owned native release
            // stalls. No user session, URL or media identity enters the dump.
            val watcher=Thread({
                if(!cleaned.await(5,TimeUnit.SECONDS)) {
                    File(context.filesDir,"native-control-fault-cleanup-stacks.txt").writeText(
                        Thread.getAllStackTraces().entries.joinToString("\n\n") {(thread,stack) ->
                            thread.name+" "+thread.state+"\n"+stack.joinToString("\n")})
                }
            },"owned-native-cleanup-observer").apply {isDaemon=true;start()}
            withContext(NonCancellable+Dispatchers.Main) {constructed?.close()}
            val released=constructed==null || cleaned.await(20,TimeUnit.SECONDS)
            watcher.join(1000)
            if(!released || nativeClosed.get()!=1) {
                val error=AssertionError(if(!released) "native cleanup timed out" else "native close did not return exactly once")
                if(bodyFailure!=null) bodyFailure.addSuppressed(error) else throw error
            } else assertTrue("owned fixture cleanup",fixture.delete())
        }
    }
}
