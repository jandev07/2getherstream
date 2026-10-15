package app.together.v2

import android.content.Context
import android.content.Intent
import android.os.SystemClock
import androidx.lifecycle.Lifecycle
import androidx.test.core.app.ActivityScenario
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import app.together.v2.player.PlayerProofController
import app.together.v2.player.LibMpvAdapter
import app.together.v2.player.nativeTimeMillis
import io.github.yuroyami.libmpvkt.*
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith
import java.io.File
import java.util.UUID
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicLong
import java.util.concurrent.atomic.AtomicReference

/** Numeric diagnostic of native progress after a completed unpause submission.
 * No authority/clock estimator, network, rate correction or sync acceptance. */
@RunWith(AndroidJUnit4::class)
@OptIn(InternalLibmpvKtApi::class)
class NativeStartupObservationTest {
    @Test fun recordsLocalStartAndBackgroundResumeProgress() {
        val context=ApplicationProvider.getApplicationContext<Context>()
        val run=checkNotNull(InstrumentationRegistry.getArguments().getString("startupRun"))
        require(UUID.fromString(run).toString()==run)
        val output=File(context.cacheDir,"native-startup-$run.jsonl")
        val launch=Intent(context,MainActivity::class.java).putExtra("localProofOnly",true)
        output.bufferedWriter().use {log ->
            ActivityScenario.launch<MainActivity>(launch).use {scenario ->
                lateinit var controller:PlayerProofController
                scenario.onActivity {controller=it.controller}
                fun await(label:String,condition:()->Boolean) {
                    val deadline=SystemClock.elapsedRealtime()+15000
                    while(SystemClock.elapsedRealtime()<deadline) {
                        assertNull(label,controller.status.value.error)
                        assertNull(label,controller.status.value.failure)
                        if(condition()) return
                        SystemClock.sleep(25)
                    }
                    fail(label)
                }
                await("native initialization") {controller.status.value.initialized}
                scenario.onActivity {controller.loadTest()}
                await("local source ready paused") {val p=controller.status.value;p.sourceReady && p.state=="Paused" && !p.seeking}
                for(phase in listOf("initial_start","after_background")) {
                    if(phase=="after_background") {
                        scenario.moveToState(Lifecycle.State.CREATED)
                        SystemClock.sleep(500)
                        scenario.moveToState(Lifecycle.State.RESUMED)
                        await("resumed source ready paused") {val p=controller.status.value;p.sourceReady && p.state=="Paused" && !p.seeking}
                    }
                    val startingPosition=AtomicLong()
                    val completedAt=AtomicLong()
                    val failure=AtomicReference<Throwable?>()
                    val submitted=CountDownLatch(1)
                    scenario.onActivity {
                        controller.player.enqueueWorkerBarrierForProof {
                            try {
                                val field=LibMpvAdapter::class.java.getDeclaredField("core").apply {isAccessible=true}
                                val core=field.get(controller.player) as Mpv
                                check(core.get(MpvProperties.Pause).getOrThrow())
                                startingPosition.set(checkNotNull(nativeTimeMillis(core.get(MpvProperties.TimePos).getOrThrow())))
                                core.set(MpvProperties.Pause,false).getOrThrow()
                                completedAt.set(SystemClock.elapsedRealtime())
                            } catch(error:Throwable) {failure.set(error)}
                            finally {submitted.countDown()}
                        }
                    }
                    assertTrue("unpause worker completion",submitted.await(5,TimeUnit.SECONDS))
                    assertNull("native unpause rejected",failure.get())
                    val anchor=completedAt.get()
                    val initial=startingPosition.get()
                    var observations=0
                    var previousSample=0L
                    while(SystemClock.elapsedRealtime()-anchor<5000) {
                        val p=controller.status.value
                        assertNull(p.error);assertNull(p.failure)
                        if(p.sampledAtMs>=anchor && p.sampledAtMs!=previousSample) {
                            previousSample=p.sampledAtMs;observations++
                            log.write(JSONObject().put("run_id",run).put("phase",phase)
                                .put("command_completed_ms",anchor).put("initial_position_ms",initial)
                                .put("sampled_at_ms",p.sampledAtMs).put("observed_at_ms",SystemClock.elapsedRealtime())
                                .put("position_ms",p.positionMs).put("state",p.state)
                                .put("buffering",p.buffering).put("seeking",p.seeking).put("source_ready",p.sourceReady)
                                .toString()+"\n")
                            log.flush()
                        }
                        SystemClock.sleep(25)
                    }
                    assertTrue("native observations missing",observations>=10)
                    assertTrue("native progress missing",controller.status.value.positionMs>initial+1000)
                    scenario.onActivity {controller.pause()}
                    await("native paused") {controller.status.value.state=="Paused"}
                }
            }
        }
    }
}
