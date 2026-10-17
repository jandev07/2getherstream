package app.together.v2

import android.content.Context
import android.os.SystemClock
import androidx.test.core.app.ActivityScenario
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import app.together.v2.player.PlayerProofController
import org.json.JSONArray
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Assume.assumeTrue
import org.junit.Test
import org.junit.runner.RunWith
import java.io.File
import java.util.UUID

/** Actual native session-to-local controls, without touching the pairing vault. */
@RunWith(AndroidJUnit4::class)
class SessionExitTest {
    @Test fun leavingSessionRequiresExplicitLocalSourceBeforeResume() {
        val context=ApplicationProvider.getApplicationContext<Context>()
        assumeTrue("Requires owned native PC fixture host",File(context.filesDir,"proof-pairing.json").isFile)
        val run=checkNotNull(InstrumentationRegistry.getArguments().getString("proofRunId"))
        require(UUID.fromString(run).toString()==run && UUID.fromString(run)!=UUID(0,0))
        val result=JSONObject().put("run_id",run).put("phase","initial")
        val output=File(context.filesDir,"phone-session-exit.json")
        ActivityScenario.launch(MainActivity::class.java).use {scenario ->
            lateinit var controller:PlayerProofController
            scenario.onActivity {controller=it.controller}
            fun await(label:String,condition:()->Boolean) {
                val until=SystemClock.elapsedRealtime()+25000
                while(SystemClock.elapsedRealtime()<until) {
                    assertNull(label,controller.status.value.error)
                    if(condition()) return
                    SystemClock.sleep(100)
                }
                fail("$label: ${controller.status.value}; ${controller.session.diagnostics.value}")
            }
            try {
                await("actual native pair ready") {
                    val d=controller.session.diagnostics.value;val p=controller.status.value
                    d.connected && !d.preparing && d.readyCount==2 && d.rttMs!=null && d.fault==null &&
                        p.sourceReady && p.acceptedSourceRevision==p.sourceRevision && p.durationMs>59000
                }
                scenario.onActivity {assertTrue("Paired Play remains enabled",controller.canPlay());controller.play()}
                await("session source playing") {controller.status.value.state=="Playing" && controller.status.value.positionMs>1000}
                val sessionRevision=controller.status.value.sourceRevision
                scenario.onActivity {controller.forgetSession()}
                await("leaving session pauses native source") {controller.status.value.state=="Paused"}
                assertFalse(controller.sessionMode)
                val frozen=controller.status.value
                // These are the public callbacks used by the now-local screen.
                scenario.onActivity {
                    assertFalse("Previous session must not enable local Play",controller.canPlay())
                    controller.play();controller.seek(10000)
                }
                val samples=JSONArray();var previous=frozen.sampledAtMs
                repeat(15) {
                    await("fresh post-exit native sample") {controller.status.value.sampledAtMs>previous}
                    val p=controller.status.value;previous=p.sampledAtMs
                    samples.put(JSONObject().put("sampled_at_ms",p.sampledAtMs).put("state",p.state)
                        .put("position_ms",p.positionMs).put("source_revision",p.sourceRevision))
                    SystemClock.sleep(100)
                }
                result.put("phase","post_exit_observed").put("session_revision",sessionRevision)
                    .put("frozen_position_ms",frozen.positionMs).put("samples",samples)
                output.writeText(result.toString())
                assertTrue("Unpaired local callbacks resumed the previous session movie",
                    (0 until samples.length()).all {samples.getJSONObject(it).getString("state")=="Paused"})
                assertTrue("Unpaired local callbacks sought the previous session movie",
                    (0 until samples.length()).all {kotlin.math.abs(samples.getJSONObject(it).getLong("position_ms")-frozen.positionMs)<=100})
                assertFalse("Previous session must not enable local seek",controller.canSeek())
                scenario.onActivity {controller.loadTest()}
                await("explicit local source ready") {
                    val p=controller.status.value
                    p.sourceRevision>sessionRevision && p.acceptedSourceRevision==p.sourceRevision && p.sourceReady
                }
                scenario.onActivity {assertTrue("Explicit local Play enabled",controller.canPlay());controller.play()}
                await("explicit local source can play") {controller.status.value.state=="Playing" && controller.status.value.positionMs>200}
                scenario.onActivity {controller.pause()}
                await("explicit local pause") {controller.status.value.state=="Paused"}
                assertTrue(controller.canSeek())
                scenario.onActivity {controller.seek(10000)}
                await("explicit local source can seek") {kotlin.math.abs(controller.status.value.positionMs-10000)<1000}
                result.put("phase","complete").put("local_source_revision",controller.status.value.sourceRevision)
            } finally {output.writeText(result.toString())}
        }
    }
}
