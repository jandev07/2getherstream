package app.together.v2

import android.os.SystemClock
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import app.together.v2.player.PlayerProofController
import app.together.v2.player.PlayerStatus
import app.together.v2.session.SessionController
import kotlinx.coroutines.flow.MutableStateFlow
import okhttp3.Request
import okhttp3.WebSocket
import okio.ByteString
import org.json.JSONObject
import org.json.JSONArray
import java.io.File
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith

/** Actual public UI controller and session queue; source/native statuses and
 * in-flight transport are synthetic. No socket IO, decoded media or timing proof. */
@RunWith(AndroidJUnit4::class)
class RelativeSeekTest {
    @Test fun relativeSeekRefusesUnsettledSourceAndUsesCurrentReadyPosition() {
        val instrumentation=InstrumentationRegistry.getInstrumentation()
        instrumentation.runOnMainSync {
            val controller=PlayerProofController(instrumentation.targetContext)
            val observations=JSONArray()
            val receipt=File(instrumentation.targetContext.filesDir,"relative-seek-settled-proof.json")
            val session=controller.session
            fun field(owner:Any,name:String)=owner.javaClass.getDeclaredField(name).apply {isAccessible=true}
            @Suppress("UNCHECKED_CAST")
            val native=field(controller.player,"mutableStatus").get(controller.player) as MutableStateFlow<PlayerStatus>
            @Suppress("UNCHECKED_CAST")
            val diagnostics=field(session,"mutable").get(session) as MutableStateFlow<SessionController.Diagnostics>
            @Suppress("UNCHECKED_CAST")
            val mode=field(controller,"sessionMode\$delegate").get(controller) as androidx.compose.runtime.MutableState<Boolean>
            val snapshot=JSONObject().put("media",JSONObject().put("provider","fixture")).put("prepare",JSONObject.NULL)
            val ready=PlayerStatus(initialized=true,state="Paused",positionMs=15000,durationMs=30000,
                sampledAtMs=SystemClock.elapsedRealtime(),sourceRevision=4,acceptedSourceRevision=4,sourceReady=true)
            try {
                mode.value=true;diagnostics.value=SessionController.Diagnostics(connected=true,mediaAvailable=true)
                field(session,"state").set(session,snapshot);field(session,"epoch").set(session,"relative-seek-test")
                field(session,"generation").setLong(session,8);field(session,"expectedSourceRevision").setLong(session,4)
                field(session,"inflight").set(session,1L)
                field(session,"socket").set(session,object:WebSocket {
                    override fun request()=Request.Builder().url("https://example.org").build()
                    override fun queueSize()=0L
                    override fun send(text:String):Boolean=error("No transport send expected")
                    override fun send(bytes:ByteString):Boolean=error("No transport send expected")
                    override fun close(code:Int,reason:String?)=true
                    override fun cancel() {}
                })
                fun rejected(label:String,status:PlayerStatus=ready,loaded:Boolean=true,loading:Boolean=false,pending:Boolean=false,prepare:Boolean=false,fault:String?=null) {
                    native.value=status;field(session,"loaded").setBoolean(session,loaded);field(session,"loading").setBoolean(session,loading)
                    field(session,"seekPending").setBoolean(session,pending);snapshot.put("prepare",if(prepare) JSONObject() else JSONObject.NULL)
                    snapshot.put("fault",fault ?: JSONObject.NULL)
                    val enabled=controller.canSeek()
                    field(session,"wanted").set(session,null);controller.seek(10000)
                    observations.put(JSONObject().put("case",label).put("can_seek",enabled)
                        .put("queued",field(session,"wanted").get(session)!=null))
                    receipt.writeText(observations.toString())
                    assertFalse("Relative seek enabled from $label",enabled)
                    assertNull("Relative seek queued from $label",field(session,"wanted").get(session))
                }
                rejected("old native source",ready.copy(sourceRevision=3,acceptedSourceRevision=3))
                rejected("unloaded replacement",loaded=false)
                rejected("grant/load in flight",loading=true)
                rejected("authority seek pending",pending=true)
                rejected("prepare barrier",prepare=true)
                rejected("native seek in progress",ready.copy(seeking=true))
                rejected("unready native source",ready.copy(sourceReady=false))
                rejected("stale native probe",ready.copy(sampledAtMs=SystemClock.elapsedRealtime()-1500))
                rejected("pending native failure",ready.copy(failure="command_failed"))
                rejected("room native failure",fault="player_failed_pc")
                rejected("native buffering",ready.copy(state="Buffering",buffering=true))
                native.value=ready.copy(sampledAtMs=SystemClock.elapsedRealtime());field(session,"loaded").setBoolean(session,true)
                field(session,"loading").setBoolean(session,false);field(session,"seekPending").setBoolean(session,false)
                snapshot.put("prepare",JSONObject.NULL);field(session,"wanted").set(session,null)
                assertTrue("Settled healthy source remains seekable",controller.canSeek())
                controller.seek(10000)
                val queued=checkNotNull(field(session,"wanted").get(session))
                assertEquals(8L,field(queued,"generation").getLong(queued))
                assertEquals(25000L,(field(queued,"action").get(queued) as JSONObject).getLong("position_ms"))
            } finally {controller.close();receipt.writeText(observations.toString())}
        }
    }
}
