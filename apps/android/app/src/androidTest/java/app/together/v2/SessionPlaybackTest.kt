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
import org.junit.Assert.*
import org.junit.Assume.assumeTrue
import org.junit.Test
import org.junit.runner.RunWith
import java.io.File
import org.json.JSONObject
import kotlin.math.abs

/** Runs only with private debug pairing configuration and a live native PC host.
 * Both devices use real libmpv and their own local copy of the synthetic video.
 */
@RunWith(AndroidJUnit4::class)
class SessionPlaybackTest {
    @Test fun authoritativeStartPauseSeekSpamBackgroundReconnectAndResume() {
        val context=ApplicationProvider.getApplicationContext<Context>()
        assumeTrue("Requires live PC proof host",File(context.filesDir,"proof-pairing.json").exists())
        val runId=checkNotNull(InstrumentationRegistry.getArguments().getString("proofRunId")) { "Proof run ID required" }
        require(java.util.UUID.fromString(runId).toString()==runId && runId!="00000000-0000-0000-0000-000000000000") { "Invalid proof run ID" }
        ActivityScenario.launch(MainActivity::class.java).use { scenario ->
            lateinit var controller:PlayerProofController
            scenario.onActivity { controller=it.controller }
            val log=File(context.filesDir,"phone-session-proof.jsonl").bufferedWriter()
            var phase="setup"
            fun record():Pair<app.together.v2.session.SessionController.Diagnostics,app.together.v2.player.PlayerStatus> {
                val d=controller.session.diagnostics.value;val p=controller.status.value
                log.write(JSONObject().put("run_id",runId).put("client_ms",SystemClock.elapsedRealtime()).put("device","phone")
                    .put("phase",phase)
                    .put("generation",d.generation).put("revision",d.revision).put("connected",d.connected)
                    .put("position_ms",p.positionMs).put("state",p.state).put("preparing",d.preparing)
                    .put("sampled_at_ms",p.sampledAtMs).put("sample_age_ms",SystemClock.elapsedRealtime()-p.sampledAtMs)
                    .put("ready_count",d.readyCount).put("rtt_ms",d.rttMs).put("offset_ms",d.offsetMs)
                    .put("clock_sample_age_ms",d.clockSampleAgeMs).put("clock_history_size",d.clockHistorySize)
                    .put("last_clock_rtt_ms",d.lastClockRttMs).put("last_clock_offset_ms",d.lastClockOffsetMs)
                    .put("clock_lower_offset_ms",d.clockLowerOffsetMs).put("clock_upper_offset_ms",d.clockUpperOffsetMs)
                    .put("clock_support_samples",d.clockSupportSamples).put("raw_selected_offset_ms",d.rawSelectedOffsetMs)
                    .put("clock_support_age_ms",d.clockSupportAgeMs)
                    .put("clock_interval_consistent",d.clockLowerOffsetMs!=null && d.clockUpperOffsetMs!=null)
                    .put("drift_ms",d.driftMs).put("rate",d.rate).put("error",d.error).put("fault",d.fault)
                    .put("execution_count",d.executionCount).put("native_failure",p.failure)
                    .put("authoritative_playing",d.authoritativePlaying).put("desired_playing",d.desiredPlaying)
                    .put("source_ready",p.sourceReady).put("source_revision",p.sourceRevision).put("accepted_source_revision",p.acceptedSourceRevision)
                    .put("seeking",p.seeking).put("buffering",p.buffering).toString()+"\n");log.flush()
                return d to p
            }
            fun await(label:String, condition:()->Boolean) {
                val until=SystemClock.elapsedRealtime()+25000
                while(SystemClock.elapsedRealtime()<until) {
                    assertNull(label,controller.status.value.error);record()
                    if(condition()) return
                    SystemClock.sleep(100)
                }
                fail("$label: ${controller.status.value}; ${controller.session.diagnostics.value}")
            }
            try {
                await("both ready paused") { val d=controller.session.diagnostics.value
                    d.connected && d.rttMs!=null && !d.preparing && d.readyCount==2 && controller.status.value.durationMs>59000 }
                scenario.onActivity { controller.play() }
                await("synchronized start") { controller.status.value.state=="Playing" && controller.status.value.positionMs>1000 }
                await("both native start observations accepted") {controller.session.diagnostics.value.executionCount==2}
                val generation=controller.session.diagnostics.value.generation
                scenario.onActivity { controller.pause() }
                await("authoritative pause") { controller.status.value.state=="Paused" && controller.session.diagnostics.value.generation>generation }
                scenario.onActivity { controller.seek(20013-controller.status.value.positionMs) }
                await("seek barrier completes") { !controller.session.diagnostics.value.preparing && abs(controller.status.value.positionMs-20013)<=100 }
                scenario.onActivity { controller.play() }
                await("scheduled resume") { controller.status.value.state=="Playing" && controller.status.value.positionMs>20700 }
                scenario.onActivity { controller.pause() }
                await("pause before spam") { controller.status.value.state=="Paused" }
                val base=controller.status.value.positionMs
                scenario.onActivity { controller.seek(5000);controller.seek(-3000);controller.seek(1000) }
                await("latest rapid seek wins") { !controller.session.diagnostics.value.preparing && abs(controller.status.value.positionMs-(base+1000))<=100 }
                val beforeReconnect=controller.session.diagnostics.value.generation
                scenario.moveToState(Lifecycle.State.CREATED)
                SystemClock.sleep(1000)
                scenario.moveToState(Lifecycle.State.RESUMED)
                await("snapshot reconnect ready") { val d=controller.session.diagnostics.value
                    d.connected && d.generation>beforeReconnect && !d.preparing && d.readyCount==2 && d.rttMs!=null }
                scenario.onActivity { controller.play() }
                await("play after reconnect") { controller.status.value.state=="Playing" && controller.status.value.positionMs>base+1700 }
                scenario.onActivity {controller.player.setProbeEnabledForProof(false)}
                await("stale native worker freezes room") {val d=controller.session.diagnostics.value;d.preparing && !d.authoritativePlaying}
                val faultGeneration=controller.session.diagnostics.value.generation
                scenario.onActivity {controller.pause()}
                await("user pause during native fault") {val d=controller.session.diagnostics.value;d.generation>faultGeneration && !d.authoritativePlaying && !d.desiredPlaying}
                scenario.onActivity {controller.player.setProbeEnabledForProof(true)}
                await("native health recovery stays paused") {controller.session.diagnostics.value.generation>faultGeneration+1 && controller.session.diagnostics.value.fault==null && !controller.session.diagnostics.value.preparing && controller.status.value.state=="Paused"}
                SystemClock.sleep(600)
                assertEquals("recovery overrode user pause","Paused",controller.status.value.state)
                scenario.onActivity {controller.play()}
                await("explicit resume after native recovery") {controller.status.value.state=="Playing"}
                val beforeRejectedSend=controller.session.diagnostics.value.generation
                phase="command_rejection"
                await("local command send refusal invalidates control immediately") {
                    var rejected=false
                    scenario.onActivity {
                        rejected=controller.session.rejectCommandForProof()
                        if(rejected) assertFalse("Rejected send retained authority",controller.session.diagnostics.value.connected)
                    }
                    rejected
                }
                await("rejected send pauses native playback") {controller.status.value.state=="Paused"}
                await("reconnect preserves authority intent without replaying unsent pause") {
                    val d=controller.session.diagnostics.value
                    d.connected && d.generation>beforeRejectedSend && !d.preparing && d.rttMs!=null &&
                        d.desiredPlaying && controller.status.value.state=="Playing"
                }
                phase="setup"
                val beforeDrop=controller.session.diagnostics.value.generation
                scenario.onActivity {
                    controller.session.dropConnectionForProof()
                    assertFalse(controller.session.diagnostics.value.connected)
                }
                await("socket loss pauses local player") {controller.status.value.state=="Paused"}
                await("automatic snapshot reconnect resumes preserved play intent") {
                    val d=controller.session.diagnostics.value
                    d.connected && d.generation>beforeDrop && !d.preparing && d.rttMs!=null && controller.status.value.state=="Playing"
                }
                val steadySeconds=(InstrumentationRegistry.getArguments().getString("steadySeconds") ?: "60").toLong().coerceIn(60,7200)
                assertTrue("Fixture too short for requested uninterrupted run",controller.status.value.durationMs-controller.status.value.positionMs>steadySeconds*1000+5000)
                val steadyGeneration=controller.session.diagnostics.value.generation
                phase="steady"
                val until=SystemClock.elapsedRealtime()+steadySeconds*1000
                while(SystemClock.elapsedRealtime()<until) {
                    val (d,p)=record() // Preserve the exact failing observations before asserting.
                    assertTrue("Unexpected disconnection",d.connected)
                    assertEquals("Unexpected generation change",steadyGeneration,d.generation)
                    assertFalse("Unexpected prepare",d.preparing)
                    assertNull("Native execution fault",d.fault)
                    assertTrue("Missing fresh clock interval support",d.clockSupportSamples in 1..64)
                    assertTrue("Invalid raw clock estimate",d.rawSelectedOffsetMs?.isFinite()==true)
                    assertTrue("Expired clock interval support",d.clockSupportAgeMs?.let {it in 0..30000}==true)
                    val lower=d.clockLowerOffsetMs;val upper=d.clockUpperOffsetMs
                    if(lower!=null && upper!=null) {
                        assertTrue("Invalid clock bounds",lower.isFinite() && upper.isFinite() && lower<=upper)
                        assertEquals("Playback clock must use interval midpoint",lower/2+upper/2,d.offsetMs!!,0.001)
                    } else {
                        assertNull("Partial clock interval",lower);assertNull("Partial clock interval",upper)
                        assertEquals("Conflicting bounds must use explicit fallback",d.rawSelectedOffsetMs!!,d.offsetMs!!,0.001)
                    }
                    assertTrue("Excessive measured drift: ${d.driftMs} ms",abs(d.driftMs)<350)
                    val age=SystemClock.elapsedRealtime()-p.sampledAtMs
                    assertTrue("Stale native observation during steady playback: $age ms",age in 0..350)
                    SystemClock.sleep(100)
                }
                phase="recovery"
                scenario.onActivity { controller.pause() }
                await("final pause") {controller.status.value.state=="Paused"}
                val duration=controller.status.value.durationMs
                scenario.onActivity {controller.seek(duration-1500-controller.status.value.positionMs)}
                await("prepare near end") {!controller.session.diagnostics.value.preparing && abs(controller.status.value.positionMs-(duration-1500))<=80}
                scenario.onActivity {controller.play()}
                await("authoritative native EOF") {controller.session.diagnostics.value.ended && controller.status.value.state=="Paused"}
                val endGeneration=controller.session.diagnostics.value.generation
                SystemClock.sleep(600)
                assertTrue(abs(controller.status.value.positionMs-duration)<=1000)
                scenario.onActivity {controller.play()}
                await("replay prepares zero then plays") {
                    val d=controller.session.diagnostics.value
                    !d.ended && d.generation>endGeneration && !d.preparing && controller.status.value.state=="Playing" && controller.status.value.positionMs in 200..5000
                }
                scenario.onActivity {controller.pause()}
                await("pause after replay") {controller.status.value.state=="Paused"}
                await("both native pause observations accepted") {controller.session.diagnostics.value.executionCount==2}
                scenario.onActivity {controller.player.setStartEnabledForProof(false);controller.play()}
                await("missing native start freezes authoritative room") {controller.session.diagnostics.value.fault=="execution_timeout_phone" && controller.status.value.state=="Paused"}
                scenario.onActivity {controller.player.setStartEnabledForProof(true);controller.play()}
                await("explicit retry clears execution fault") {
                    val d=controller.session.diagnostics.value
                    d.fault==null && !d.preparing && d.executionCount==2 && controller.status.value.state=="Playing"
                }
                scenario.onActivity {controller.pause()}
                await("final acknowledged pause") {controller.session.diagnostics.value.executionCount==2 && controller.status.value.state=="Paused"}
                val invalid=File(context.cacheDir,"invalid-session-proof.mp4")
                try {
                    invalid.writeText("invalid synthetic media for synchronized decoder failure proof")
                    scenario.onActivity {controller.session.failSourceForProof(invalid)}
                    await("native decoder failure freezes room") {controller.session.diagnostics.value.fault=="player_failed_phone"}
                    val failedGeneration=controller.session.diagnostics.value.generation
                    SystemClock.sleep(800)
                    assertEquals("terminal failure retried itself",failedGeneration,controller.session.diagnostics.value.generation)
                    scenario.onActivity {controller.pause()}
                    await("explicit pause retains terminal fault") {controller.session.diagnostics.value.generation>failedGeneration && controller.session.diagnostics.value.fault=="player_failed_phone"}
                    scenario.onActivity {controller.seek(0)}
                    await("reload recovery preserves explicit pause") {
                        val d=controller.session.diagnostics.value;val p=controller.status.value
                        d.fault==null && !d.preparing && d.executionCount==2 && p.failure==null && p.sourceReady && p.state=="Paused"
                    }
                } finally {invalid.delete()}
                // The user-triggered log path uses the real paired control host;
                // no periodic uploader is active during the steady phase.
                val config=JSONObject(File(context.filesDir,"proof-pairing.json").readText())
                val logControlBase=config.getString("url").replaceFirst("ws://","http://").replaceFirst("wss://","https://").removeSuffix("/session")
                val sent=kotlinx.coroutines.runBlocking {controller.logs.send(app.together.v2.session.Pairing(logControlBase,config.getString("token")))}
                assertTrue("Recent phone diagnostics must reach the paired PC",sent>0)
            } finally {log.close()}
        }
    }
}
