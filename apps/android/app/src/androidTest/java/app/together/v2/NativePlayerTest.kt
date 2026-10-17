package app.together.v2

import android.content.pm.ActivityInfo
import android.content.Intent
import androidx.test.core.app.ApplicationProvider
import android.content.Context
import android.os.SystemClock
import androidx.test.core.app.ActivityScenario
import androidx.test.ext.junit.runners.AndroidJUnit4
import app.together.v2.player.PlayerProofController
import app.together.v2.player.PlayerStatus
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith
import kotlin.math.abs
import android.graphics.Bitmap
import java.io.File
import app.together.v2.player.LibMpvAdapter
import app.together.v2.player.NativeLifecycleProof
import androidx.test.platform.app.InstrumentationRegistry
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicInteger

/** Real packaged libmpv, Android surface and Activity lifecycle; no mock player. */
@RunWith(AndroidJUnit4::class)
class NativePlayerTest {
    @Test fun pendingUnexpectedEndAfterLoadAdmissionSurvivesWorkerCommitBoundary() {
        val instrumentation=InstrumentationRegistry.getInstrumentation()
        val context=ApplicationProvider.getApplicationContext<Context>()
        val fixture=File(context.filesDir,"pending-fault-source-proof.mp4")
        context.assets.open("clock-test.mp4").use {input ->fixture.outputStream().use {input.copyTo(it)}}
        val entered=CountDownLatch(1);val release=CountDownLatch(1);val processed=CountDownLatch(1)
        val cleaned=CountDownLatch(1)
        var constructed:LibMpvAdapter?=null
        val proof=NativeLifecycleProof(beforeCreate={},afterCreate={},afterClose={},afterCleanup={cleaned.countDown()},
            beforeSourceLoad={revision ->if(revision==1L) {entered.countDown();check(release.await(20,TimeUnit.SECONDS))}})
        try {
            lateinit var player:LibMpvAdapter
            instrumentation.runOnMainSync {player=LibMpvAdapter(context,proof);constructed=player}
            val initializedUntil=SystemClock.elapsedRealtime()+20000
            while(!player.status.value.initialized && SystemClock.elapsedRealtime()<initializedUntil) SystemClock.sleep(20)
            assertTrue(player.status.value.initialized)
            instrumentation.runOnMainSync {player.loadLocal(fixture,null)}
            assertTrue(entered.await(10,TimeUnit.SECONDS))
            instrumentation.runOnMainSync {
                // Public controller-classified fault, not a simulated decoder crash.
                player.unexpectedEnd()
                player.enqueueWorkerBarrierForProof {processed.countDown()}
            }
            release.countDown();assertTrue(processed.await(10,TimeUnit.SECONDS))
            assertEquals("unexpected_end",player.status.value.failure)
            assertEquals(0L,player.status.value.acceptedSourceRevision)
            var recovery=0L
            instrumentation.runOnMainSync {recovery=player.loadLocal(fixture,"unexpected_end")}
            val acceptedUntil=SystemClock.elapsedRealtime()+20000
            while(player.status.value.acceptedSourceRevision!=recovery && SystemClock.elapsedRealtime()<acceptedUntil) SystemClock.sleep(20)
            assertEquals(recovery,player.status.value.acceptedSourceRevision);assertNull(player.status.value.failure)
        } finally {
            release.countDown()
            constructed?.let {player ->instrumentation.runOnMainSync {player.close()};assertTrue(cleaned.await(10,TimeUnit.SECONDS))}
            fixture.delete()
        }
    }

    @Test fun newerSourceReceiptRejectsQueuedLoadWithoutCommandInvalidation() {
        val instrumentation=InstrumentationRegistry.getInstrumentation()
        val context=ApplicationProvider.getApplicationContext<Context>()
        val fixture=File(context.filesDir,"queued-source-proof.mp4")
        context.assets.open("clock-test.mp4").use {input ->fixture.outputStream().use {input.copyTo(it)}}
        val entered=CountDownLatch(1);val release=CountDownLatch(1);val cleaned=CountDownLatch(1)
        val nativeLoads=java.util.concurrent.CopyOnWriteArrayList<Long>()
        var constructed:LibMpvAdapter?=null
        val proof=NativeLifecycleProof(beforeCreate={},afterCreate={},afterClose={},
            afterCleanup={cleaned.countDown()},beforeSourceLoad={nativeLoads.add(it)})
        try {
            lateinit var player:LibMpvAdapter
            instrumentation.runOnMainSync {player=LibMpvAdapter(context,proof);constructed=player}
            val initializedUntil=SystemClock.elapsedRealtime()+20000
            while(!player.status.value.initialized && SystemClock.elapsedRealtime()<initializedUntil) SystemClock.sleep(20)
            assertTrue("Native adapter initialization timed out",player.status.value.initialized)
            instrumentation.runOnMainSync {
                player.enqueueWorkerBarrierForProof {entered.countDown();check(release.await(20,TimeUnit.SECONDS))}
            }
            assertTrue("Worker did not enter queued-source barrier",entered.await(10,TimeUnit.SECONDS))
            var old=0L;var latest=0L
            instrumentation.runOnMainSync {
                old=player.loadLocal(fixture)
                // Deliberately no invalidateCommands(): source revision alone fences this.
                latest=player.loadLocal(fixture)
            }
            assertTrue(latest>old);assertTrue(nativeLoads.isEmpty())
            release.countDown()
            val acceptedUntil=SystemClock.elapsedRealtime()+20000
            while(player.status.value.acceptedSourceRevision!=latest && SystemClock.elapsedRealtime()<acceptedUntil) SystemClock.sleep(20)
            assertEquals("Latest native source was not accepted",latest,player.status.value.acceptedSourceRevision)
            assertEquals("Superseded source reached the real native load boundary",listOf(latest),nativeLoads.toList())
        } finally {
            release.countDown()
            constructed?.let {owned ->instrumentation.runOnMainSync {owned.close()};assertTrue("Queued-source cleanup timed out",cleaned.await(20,TimeUnit.SECONDS))}
            fixture.delete()
        }
    }

    @Test fun closeWhileCreationIsBlockedReleasesLateNativeHandleExactlyOnce() {
        val entered=CountDownLatch(1)
        val release=CountDownLatch(1)
        val cleaned=CountDownLatch(1)
        val created=AtomicInteger()
        val closed=AtomicInteger()
        val instrumentation=InstrumentationRegistry.getInstrumentation()
        lateinit var player:LibMpvAdapter
        var constructed:LibMpvAdapter?=null
        val proof=NativeLifecycleProof(
            beforeCreate={entered.countDown();check(release.await(20,TimeUnit.SECONDS))},
            afterCreate={created.incrementAndGet()},
            afterClose={closed.incrementAndGet()},
            afterCleanup={cleaned.countDown()},
        )
        try {
            instrumentation.runOnMainSync {
                player=LibMpvAdapter(ApplicationProvider.getApplicationContext<Context>(),proof)
                constructed=player
            }
            assertTrue("Native worker did not reach creation barrier",entered.await(10,TimeUnit.SECONDS))
            instrumentation.runOnMainSync {
                // Must return while native creation is blocked on a different worker.
                player.close()
                player.close()
            }
            assertEquals("Native creation escaped barrier",0,created.get())
            assertEquals("Cleanup overtook creation worker",1L,cleaned.count)
            release.countDown()
            assertTrue("Late native handle cleanup timed out",cleaned.await(20,TimeUnit.SECONDS))
            assertEquals("Real native handle was not created",1,created.get())
            assertEquals("Late native handle was not closed exactly once",1,closed.get())
            assertFalse("Closed player was published as initialized",player.status.value.initialized)
        } finally {
            release.countDown()
            constructed?.let {owned -> instrumentation.runOnMainSync {owned.close()}}
        }
    }

    @Test fun loadPlayPauseExactSeekRotateAndResume() {
        val launch=Intent(ApplicationProvider.getApplicationContext<Context>(),MainActivity::class.java).putExtra("localProofOnly",true)
        ActivityScenario.launch<MainActivity>(launch).use { scenario ->
            lateinit var controller: PlayerProofController
            scenario.onActivity { controller = it.controller }
            fun await(label: String, timeoutMs: Long = 20000, allowPendingDecoderFailure:Boolean=false, condition: (PlayerStatus) -> Boolean): PlayerStatus {
                val until = SystemClock.elapsedRealtime() + timeoutMs
                while (SystemClock.elapsedRealtime() < until) {
                    val state = controller.status.value
                    if(allowPendingDecoderFailure && state.failure=="decoder_failed")
                        assertEquals(label,"Native media playback failed",state.error)
                    else assertNull("$label: ${state.error}", state.error)
                    if (condition(state)) return state
                    SystemClock.sleep(40)
                }
                fail("$label timed out: ${controller.status.value}")
                error("unreachable")
            }
            await("initialize") { it.initialized }
            scenario.onActivity { it.controller.loadTest() }
            await("load paused") { it.durationMs >= 59000 && it.state == "Paused" && !it.seeking }
            SystemClock.sleep(500)
            scenario.onActivity {
                val image=checkNotNull(controller.player.view.bitmap) {"Native video has no rendered frame"}
                var colored=0
                for(y in 1..8) for(x in 1..8) {
                    if((image.getPixel(image.width*x/9,image.height*y/9) and 0xFFFFFF)>0x202020) colored++
                }
                assertTrue("Native video frame is black",colored>20)
                File(it.filesDir,"native-player-frame.png").outputStream().use {out -> image.compress(Bitmap.CompressFormat.PNG,100,out)}
                image.recycle()
            }
            scenario.onActivity { it.controller.play() }
            await("play advances") { it.positionMs > 700 && it.state == "Playing" }
            scenario.onActivity { it.controller.pause() }
            val paused = await("pause") { it.state == "Paused" }.positionMs
            SystemClock.sleep(600)
            assertTrue("paused position moved", abs(controller.status.value.positionMs - paused) <= 80)
            scenario.onActivity { it.controller.player.seek(20013) }
            await("exact seek") { !it.seeking && abs(it.positionMs - 20013) <= 100 && it.state == "Paused" }
            scenario.onActivity { it.requestedOrientation = ActivityInfo.SCREEN_ORIENTATION_LANDSCAPE }
            SystemClock.sleep(800)
            scenario.onActivity { assertSame("rotation recreated player", controller, it.controller) }
            assertTrue(abs(controller.status.value.positionMs - 20013) <= 100)
            scenario.onActivity { it.controller.play() }
            await("resume after rotation") { it.positionMs >= 20700 && it.state == "Playing" }
            scenario.onActivity { it.controller.pause() }
            await("final pause") { it.state == "Paused" }
            val invalid=File(ApplicationProvider.getApplicationContext<Context>().cacheDir,"invalid-native-proof.mp4")
            try {
                invalid.writeText("invalid synthetic media for decoder failure proof")
                scenario.onActivity {controller.player.failSourceForProof(invalid)}
                await("terminal decoder failure",allowPendingDecoderFailure=true) {it.failure=="decoder_failed" && !it.sourceReady}
                scenario.onActivity {controller.loadTest()}
                await("native decoder recovers after new source",allowPendingDecoderFailure=true) {it.failure==null && it.sourceReady && it.durationMs>59000 && it.state=="Paused"}
            } finally {invalid.delete()}
        }
    }
}
