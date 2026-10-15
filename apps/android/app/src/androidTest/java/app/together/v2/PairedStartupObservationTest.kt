package app.together.v2

import android.content.Context
import android.os.SystemClock
import androidx.lifecycle.Lifecycle
import androidx.test.core.app.ActivityScenario
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import app.together.v2.player.LibMpvAdapter
import app.together.v2.player.PlayerProofController
import io.github.yuroyami.libmpvkt.*
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith
import java.io.File
import java.util.UUID
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicReference

/** Explicit paired timing diagnostic, not the full playback acceptance test.
 * Additional numeric native reads occur only on the existing worker. Raw native
 * messages, media identity, URLs and credentials are never written here. */
@RunWith(AndroidJUnit4::class)
@OptIn(InternalLibmpvKtApi::class)
class PairedStartupObservationTest {
    @Test fun recordsNativeClocksAcrossPairedStartAndReconnect() {
        val context=ApplicationProvider.getApplicationContext<Context>()
        val run=checkNotNull(InstrumentationRegistry.getArguments().getString("proofRunId"))
        require(UUID.fromString(run).toString()==run)
        check(File(context.filesDir,"proof-pairing.json").isFile)
        File(context.cacheDir,"paired-startup-$run.jsonl").bufferedWriter().use {log ->
            ActivityScenario.launch(MainActivity::class.java).use {scenario ->
                lateinit var controller:PlayerProofController
                scenario.onActivity {controller=it.controller}
                fun await(label:String,condition:()->Boolean) {
                    val until=SystemClock.elapsedRealtime()+25000
                    while(SystemClock.elapsedRealtime()<until) {
                        assertNull(label,controller.status.value.error)
                        assertNull(label,controller.status.value.failure)
                        if(condition()) return
                        SystemClock.sleep(50)
                    }
                    fail(label)
                }
                await("both native sources ready paused") {
                    val d=controller.session.diagnostics.value;val p=controller.status.value
                    d.connected && d.readyCount==2 && !d.preparing && d.offsetMs!=null && p.sourceReady && p.state=="Paused"
                }
                val metadata=AtomicReference<JSONObject?>()
                val metadataRead=CountDownLatch(1)
                scenario.onActivity {
                    controller.player.enqueueWorkerBarrierForProof {
                        try {
                            val field=LibMpvAdapter::class.java.getDeclaredField("core").apply {isAccessible=true}
                            val core=field.get(controller.player) as Mpv
                            fun category(name:String,known:Set<String>):String =
                                runCatching {core.get(MpvProperty.Str(name)).getOrThrow()}.getOrNull()
                                    ?.let {if(it in known) it else "other"} ?: "unavailable"
                            metadata.set(JSONObject()
                                .put("width",runCatching {core.get(MpvProperty.Int64("width")).getOrThrow()}.getOrNull() ?: JSONObject.NULL)
                                .put("height",runCatching {core.get(MpvProperty.Int64("height")).getOrThrow()}.getOrNull() ?: JSONObject.NULL)
                                .put("codec",category("video-format",setOf("h264","hevc","av1","vp8","vp9","mpeg4","mpeg2video")))
                                .put("hardware_decoder",category("hwdec-current",setOf("no","mediacodec","mediacodec-copy"))))
                        } finally {metadataRead.countDown()}
                    }
                }
                assertTrue("native metadata read timed out",metadataRead.await(5,TimeUnit.SECONDS))
                assertNotNull("native metadata unavailable",metadata.get())
                for(phase in listOf("initial_start","after_background","after_socket_loss")) {
                    when(phase) {
                        "initial_start" -> scenario.onActivity {controller.play()}
                        "after_background" -> {
                            val generation=controller.session.diagnostics.value.generation
                            scenario.moveToState(Lifecycle.State.CREATED)
                            SystemClock.sleep(1000)
                            scenario.moveToState(Lifecycle.State.RESUMED)
                            await("background reconnect ready") {
                                val d=controller.session.diagnostics.value
                                d.connected && d.generation>generation && !d.preparing && d.readyCount==2 && d.offsetMs!=null
                            }
                            scenario.onActivity {controller.play()}
                        }
                        else -> {
                            val generation=controller.session.diagnostics.value.generation
                            scenario.onActivity {controller.session.dropConnectionForProof()}
                            await("socket reconnect ready") {
                                val d=controller.session.diagnostics.value
                                d.connected && d.generation>generation && !d.preparing && d.offsetMs!=null
                            }
                        }
                    }
                    await("native playing") {controller.status.value.state=="Playing"}
                    val until=SystemClock.elapsedRealtime()+10000
                    var count=0
                    while(SystemClock.elapsedRealtime()<until) {
                        val d=controller.session.diagnostics.value;val p=controller.status.value
                        assertNull(p.error);assertNull(p.failure)
                        val completed=CountDownLatch(1)
                        val result=AtomicReference<JSONObject?>()
                        val failure=AtomicReference<Throwable?>()
                        scenario.onActivity {
                            controller.player.enqueueWorkerBarrierForProof {
                                try {
                                    val field=LibMpvAdapter::class.java.getDeclaredField("core").apply {isAccessible=true}
                                    val core=field.get(controller.player) as Mpv
                                    val before=SystemClock.elapsedRealtime()
                                    fun finite(property:MpvProperty<Double>):Any =
                                        runCatching {core.get(property).getOrThrow()}.getOrNull()
                                            ?.takeIf {it.isFinite()} ?: JSONObject.NULL
                                    val row=JSONObject().put("native_before_ms",before)
                                        .put("time_pos_seconds",finite(MpvProperties.TimePos))
                                        .put("audio_pts_seconds",finite(MpvProperty.Dbl("audio-pts")))
                                        .put("avsync_seconds",finite(MpvProperties.Avsync))
                                        .put("speed",finite(MpvProperties.Speed))
                                        .put("audio_speed_correction",finite(MpvProperties.AudioSpeedCorrection))
                                        .put("video_speed_correction",finite(MpvProperties.VideoSpeedCorrection))
                                        .put("frame_drops",runCatching {core.get(MpvProperties.FrameDropCount).getOrThrow()}.getOrNull() ?: JSONObject.NULL)
                                        .put("native_after_ms",SystemClock.elapsedRealtime())
                                    result.set(row)
                                } catch(error:Throwable) {failure.set(error)}
                                finally {completed.countDown()}
                            }
                        }
                        assertTrue("native diagnostic read timed out",completed.await(5,TimeUnit.SECONDS))
                        assertNull("native diagnostic read failed",failure.get())
                        log.write(checkNotNull(result.get()).put("run_id",run).put("phase",phase).put("stream",metadata.get())
                            .put("generation",d.generation).put("revision",d.revision)
                            .put("connected",d.connected).put("preparing",d.preparing)
                            .put("fault",d.fault ?: JSONObject.NULL)
                            .put("offset_ms",d.offsetMs ?: JSONObject.NULL).put("authority_drift_ms",d.driftMs)
                            .put("commanded_rate",d.rate).put("position_ms",p.positionMs)
                            .put("sampled_at_ms",p.sampledAtMs).put("state",p.state)
                            .put("buffering",p.buffering).put("seeking",p.seeking).put("source_ready",p.sourceReady)
                            .toString()+"\n")
                        log.flush();count++
                        // Avoid adding dozens of synchronous native requests per
                        // second to the production probe while diagnosing load.
                        SystemClock.sleep(1000)
                    }
                    assertTrue("diagnostic observations missing",count>=5)
                    if(phase=="initial_start") {
                        scenario.onActivity {controller.pause()}
                        await("pause before background") {controller.status.value.state=="Paused"}
                    }
                }
                scenario.onActivity {controller.pause()}
                await("final native pause") {controller.status.value.state=="Paused"}
            }
        }
    }
}
