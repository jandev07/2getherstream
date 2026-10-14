package app.together.v2

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import app.together.v2.player.LibMpvAdapter
import app.together.v2.session.SessionController
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith
import java.lang.reflect.InvocationTargetException

/** Controller boundary regression with synthetic snapshots/loaded flags.
 * No live server, saved pairing, TorBox, or native playback acceptance claim.
 */
@RunWith(AndroidJUnit4::class)
class SessionIdentityTest {
    @Test fun unresolvedIdentityIsRejectedEvenWhenPreviousSourceIsLoaded() {
        val instrumentation=InstrumentationRegistry.getInstrumentation()
        instrumentation.runOnMainSync {
            val player=LibMpvAdapter(instrumentation.targetContext)
            val session=SessionController(instrumentation.targetContext,player)
            try {
                // Exercise the real private acceptance boundary without adding a
                // public snapshot-injection method to the shipped controller.
                val accept=SessionController::class.java.getDeclaredMethod("accept",JSONObject::class.java).apply {isAccessible=true}
                val loaded=SessionController::class.java.getDeclaredField("loaded").apply {isAccessible=true}
                val epoch=SessionController::class.java.getDeclaredField("epoch").apply {isAccessible=true}
                val generation=SessionController::class.java.getDeclaredField("generation").apply {isAccessible=true}
                val state=SessionController::class.java.getDeclaredField("state").apply {isAccessible=true}
                epoch.set(session,"synthetic-source-identity-test");generation.setLong(session,1)
                state.set(session,snapshot(JSONObject().put("provider","fixture").put("torrent_id",0).put("file_id",0)).put("revision",1))
                for (alreadyLoaded in listOf(false,true)) {
                    loaded.setBoolean(session,alreadyLoaded)
                    for (identity in listOf(
                        JSONObject().put("provider","torbox").put("torrent_id",1).put("file_id",2),
                        JSONObject().put("provider","fixture").put("torrent_id",1).put("file_id",0),
                        JSONObject().put("provider","fixture").put("torrent_id",0).put("file_id",1))) {
                        try {accept.invoke(session,snapshot(identity));fail("Unresolved identity accepted")}
                        catch(error:InvocationTargetException) {assertTrue(error.cause is IllegalArgumentException)}
                    }
                }
                loaded.setBoolean(session,true)
                assertEquals(true,accept.invoke(session,snapshot(JSONObject().put("provider","fixture").put("torrent_id",0).put("file_id",0))))
                assertEquals(1L,session.diagnostics.value.generation)
                assertEquals(2L,session.diagnostics.value.revision)
                // JSONObject numeric coercion must not invent an authority version.
                for((field,value) in listOf("protocol" to "1","protocol" to 1.0,
                    "revision" to "2","revision" to 2.5,"revision" to -1,
                    "generation" to "1","generation" to 1.5,"generation" to -1)) {
                    val invalid=snapshot(JSONObject().put("provider","fixture").put("torrent_id",0).put("file_id",0)).put(field,value)
                    try {accept.invoke(session,invalid);fail("Coerced authority accepted")}
                    catch(error:InvocationTargetException) {assertTrue(error.cause is IllegalArgumentException)}
                    assertEquals(2L,session.diagnostics.value.revision)
                    assertEquals(1L,session.diagnostics.value.generation)
                }
                // Reject future malformed playback before generation/loaded changes.
                for((field,value) in listOf("position_ms" to "0","position_ms" to -1,
                    "anchor_ms" to 0.5,"playing" to "true")) {
                    loaded.setBoolean(session,true)
                    val invalid=snapshot(JSONObject().put("provider","fixture").put("torrent_id",0).put("file_id",0))
                        .put("generation",2).put("revision",3)
                    invalid.getJSONObject("timeline").put(field,value)
                    try {accept.invoke(session,invalid);fail("Coerced playback accepted")}
                    catch(error:InvocationTargetException) {assertTrue(error.cause is IllegalArgumentException)}
                    assertTrue(loaded.getBoolean(session));assertEquals(1L,generation.getLong(session))
                    assertEquals(2L,session.diagnostics.value.revision)
                }
                for (prepare in listOf(
                    JSONObject().put("generation",1).put("position_ms",0).put("deadline_ms",15000).put("ready",org.json.JSONArray()),
                    JSONObject().put("generation",2).put("position_ms","0").put("deadline_ms",15000).put("ready",org.json.JSONArray()),
                    JSONObject().put("generation",2).put("position_ms",0).put("deadline_ms",15000).put("ready",org.json.JSONArray().put("pc").put("pc")))) {
                    loaded.setBoolean(session,true)
                    val invalid=snapshot(JSONObject().put("provider","fixture").put("torrent_id",0).put("file_id",0))
                        .put("generation",2).put("revision",3).put("prepare",prepare)
                    try {accept.invoke(session,invalid);fail("Invalid preparation accepted")}
                    catch(error:InvocationTargetException) {assertTrue(error.cause is IllegalArgumentException)}
                    assertTrue(loaded.getBoolean(session));assertEquals(1L,generation.getLong(session))
                }
                // New-generation identity validation precedes loading/native effects.
                for(identity in listOf(
                    JSONObject().put("provider","fixture").put("torrent_id","0").put("file_id",0),
                    JSONObject().put("provider","fixture").put("torrent_id",0.5).put("file_id",0),
                    JSONObject().put("provider","torbox").put("torrent_id",1.5).put("file_id",0),
                    JSONObject().put("provider","torbox").put("torrent_id",1).put("file_id","0"),
                    JSONObject().put("provider",JSONObject.NULL).put("torrent_id",0).put("file_id",0))) {
                    loaded.setBoolean(session,true)
                    try {accept.invoke(session,snapshot(identity).put("generation",2).put("revision",3));fail("Coerced identity accepted")}
                    catch(error:InvocationTargetException) {assertTrue(error.cause is IllegalArgumentException)}
                    assertTrue(loaded.getBoolean(session));assertEquals(1L,generation.getLong(session))
                    assertEquals(2L,session.diagnostics.value.revision)
                }
                for(participant in listOf(
                    participant().put("buffering","false"),participant().put("connected",0),
                    participant().put("rtt_ms",0.5),participant().put("player_fault","unknown"),
                    participant().put("execution",JSONObject().put("generation","2").put("playing",false)
                        .put("position_ms",0).put("observed_server_ms",0).put("received_server_ms",0)),
                    participant().put("execution","invalid"))) {
                    loaded.setBoolean(session,true)
                    val invalid=snapshot(JSONObject().put("provider","fixture").put("torrent_id",0).put("file_id",0))
                        .put("generation",2).put("revision",3)
                    invalid.getJSONObject("participants").put("phone",participant)
                    try {accept.invoke(session,invalid);fail("Invalid participant accepted")}
                    catch(error:InvocationTargetException) {
                        assertTrue(error.cause is IllegalArgumentException || error.cause is org.json.JSONException)
                    }
                    assertTrue(loaded.getBoolean(session));assertEquals(1L,generation.getLong(session))
                    assertEquals(2L,session.diagnostics.value.revision)
                }
                for((field,value) in listOf("epoch" to 1,"epoch" to JSONObject.NULL,
                    "ended" to "true","ended" to JSONObject.NULL,"fault" to true,"fault" to 1)) {
                    loaded.setBoolean(session,true)
                    val invalid=snapshot(JSONObject().put("provider","fixture").put("torrent_id",0).put("file_id",0))
                        .put("generation",2).put("revision",3).put(field,value)
                    try {accept.invoke(session,invalid);fail("Coerced envelope accepted")}
                    catch(error:InvocationTargetException) {assertTrue(error.cause is IllegalArgumentException)}
                    assertTrue(loaded.getBoolean(session));assertEquals(1L,generation.getLong(session))
                    assertEquals(2L,session.diagnostics.value.revision)
                }
                val legacy=snapshot(JSONObject().put("provider","fixture").put("torrent_id",0).put("file_id",0))
                legacy.remove("ended");legacy.remove("fault")
                assertEquals(true,accept.invoke(session,legacy))
                assertFalse(session.diagnostics.value.ended);assertNull(session.diagnostics.value.fault)
                // An obsolete source snapshot is ignored before its source gate;
                // it cannot disconnect the current, supported source.
                assertEquals(false,accept.invoke(session,snapshot(JSONObject().put("provider","torbox").put("torrent_id",1).put("file_id",2)).put("revision",0).put("ended","invalid").put("fault",true)))
                assertEquals(2L,session.diagnostics.value.revision)
                // A valid new TorBox generation is accepted as metadata, but it
                // cannot reuse a loaded fixture or report native readiness.
                loaded.setBoolean(session,true)
                val torbox=snapshot(JSONObject().put("provider","torbox").put("torrent_id",1).put("file_id",2))
                    .put("generation",2).put("revision",3)
                    .put("prepare",JSONObject().put("generation",2).put("position_ms",0).put("deadline_ms",15000).put("ready",org.json.JSONArray()))
                accept.invoke(session,torbox)
                assertFalse("new source must await its own capability",loaded.getBoolean(session))
                assertEquals(2L,session.diagnostics.value.generation)
                assertEquals(0,session.diagnostics.value.readyCount)
            } finally {session.close();player.close()}
        }
    }
    private fun snapshot(media:JSONObject)=JSONObject()
        .put("protocol",1).put("epoch","synthetic-source-identity-test").put("generation",1).put("revision",2)
        .put("media",media).put("fault",JSONObject.NULL).put("prepare",JSONObject.NULL)
        .put("desired_playing",false).put("ended",false)
        .put("timeline",JSONObject().put("position_ms",0).put("anchor_ms",0).put("playing",false))
        .put("participants",JSONObject().put("pc",participant()).put("phone",participant()))
    private fun participant()=JSONObject().put("connected",false).put("buffering",false).put("rtt_ms",0)
        .put("execution",JSONObject.NULL).put("player_fault",JSONObject.NULL)
}
