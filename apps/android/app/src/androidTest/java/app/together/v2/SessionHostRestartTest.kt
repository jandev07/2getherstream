package app.together.v2

import android.content.Context
import android.os.SystemClock
import androidx.test.core.app.ActivityScenario
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import app.together.v2.player.PlayerProofController
import app.together.v2.session.SessionController
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Assume.assumeTrue
import org.junit.Test
import org.junit.runner.RunWith
import java.io.File
import java.util.UUID

/** Requires an owned runner which gracefully stops and replaces the local PC
 * fixture host after ready.json, waiting for frozen.json before replacement.
 * No pairing vault, provider account or physical-phone acceptance is exercised.
 */
@RunWith(AndroidJUnit4::class)
class SessionHostRestartTest {
    @Test fun nativePhoneRejoinsNewPcEpochPaused() {
        val context=ApplicationProvider.getApplicationContext<Context>()
        val arguments=InstrumentationRegistry.getArguments()
        assumeTrue("Requires owned PC restart orchestration",arguments.getString("pcRestartProof")=="true" &&
            File(context.filesDir,"proof-pairing.json").isFile)
        val runId=checkNotNull(arguments.getString("proofRunId"))
        require(UUID.fromString(runId).toString()==runId && UUID.fromString(runId)!=UUID(0,0))
        val ready=File(context.filesDir,"phone-host-restart-ready.json")
        val frozen=File(context.filesDir,"phone-host-restart-frozen.json")
        val result=File(context.filesDir,"phone-host-restart-result.json")
        listOf(ready,frozen,result).forEach {it.delete()}
        // Read the actual accepted authority; no production diagnostic API or
        // injected snapshot is added for this test.
        val epochField=SessionController::class.java.getDeclaredField("epoch").apply {isAccessible=true}
        ActivityScenario.launch(MainActivity::class.java).use {scenario ->
            lateinit var controller:PlayerProofController
            scenario.onActivity {controller=it.controller}
            fun epoch():String? {
                var value:String?=null
                scenario.onActivity {value=epochField.get(controller.session) as String?}
                return value
            }
            val log=File(context.filesDir,"phone-host-restart.jsonl").bufferedWriter()
            var phase="initial"
            fun record() {
                val d=controller.session.diagnostics.value;val p=controller.status.value
                log.write(JSONObject().put("run_id",runId).put("phase",phase)
                    .put("client_ms",SystemClock.elapsedRealtime()).put("epoch",epoch())
                    .put("generation",d.generation).put("connected",d.connected)
                    .put("preparing",d.preparing).put("ready_count",d.readyCount)
                    .put("fault",d.fault).put("session_error",d.error).put("execution_count",d.executionCount)
                    .put("desired_playing",d.desiredPlaying).put("authoritative_playing",d.authoritativePlaying)
                    .put("state",p.state).put("position_ms",p.positionMs)
                    .put("source_revision",p.sourceRevision).put("accepted_source_revision",p.acceptedSourceRevision)
                    .put("source_ready",p.sourceReady).put("native_failure",p.failure)
                    .put("error",p.error).toString()+"\n")
                log.flush()
            }
            fun await(label:String,condition:()->Boolean) {
                val until=SystemClock.elapsedRealtime()+30000
                while(SystemClock.elapsedRealtime()<until) {
                    record();assertNull(label,controller.status.value.error)
                    if(condition()) return
                    SystemClock.sleep(100)
                }
                fail("$label: ${controller.status.value}; ${controller.session.diagnostics.value}")
            }
            fun readyPaused():Boolean {
                val d=controller.session.diagnostics.value;val p=controller.status.value
                return d.connected && d.rttMs!=null && !d.preparing && d.readyCount==2 && d.fault==null && d.error==null &&
                    p.sourceReady && p.failure==null && p.acceptedSourceRevision==p.sourceRevision &&
                    p.state=="Paused" && p.durationMs>59000
            }
            try {
                await("initial native pair ready",::readyPaused)
                val before=checkNotNull(epoch());val oldSource=controller.status.value.sourceRevision
                scenario.onActivity {controller.play()}
                await("playing before PC restart") {controller.status.value.state=="Playing" &&
                    controller.status.value.positionMs>1000 && controller.session.diagnostics.value.executionCount==2}
                phase="awaiting_stop"
                ready.writeText(JSONObject().put("run_id",runId).put("epoch",before).toString())
                await("native phone freezes on old host shutdown") {
                    !controller.session.diagnostics.value.connected && controller.status.value.state=="Paused"
                }
                phase="frozen"
                record()
                frozen.writeText(JSONObject().put("run_id",runId).put("old_epoch",before).toString())
                phase="awaiting_new_host"
                await("new epoch native pair ready paused") {
                    val next=epoch()
                    next!=null && next!=before && readyPaused() && controller.status.value.sourceRevision>oldSource
                }
                val after=checkNotNull(epoch())
                val d=controller.session.diagnostics.value
                assertFalse("new host inherited old play intent",d.desiredPlaying)
                assertFalse("new host resumed old timeline",d.authoritativePlaying)
                SystemClock.sleep(600)
                record();assertEquals("Paused",controller.status.value.state)
                phase="explicit_resume"
                scenario.onActivity {controller.play()}
                await("explicit play on replacement host") {epoch()==after &&
                    controller.status.value.state=="Playing" && controller.status.value.positionMs>1000 &&
                    controller.session.diagnostics.value.executionCount==2}
                scenario.onActivity {controller.pause()}
                await("replacement host acknowledged pause") {readyPaused() && controller.session.diagnostics.value.executionCount==2}
                phase="complete";record()
                result.writeText(JSONObject().put("run_id",runId).put("old_epoch",before).put("new_epoch",after)
                    .put("old_source_revision",oldSource).put("new_source_revision",controller.status.value.sourceRevision)
                    .put("frozen_before_replacement",true).put("explicit_resume_and_pause",true).toString())
            } finally {log.close()}
        }
    }
}
