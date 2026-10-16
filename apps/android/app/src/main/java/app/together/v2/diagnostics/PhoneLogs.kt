package app.together.v2.diagnostics

import android.content.Context
import android.os.SystemClock
import app.together.v2.player.PlayerStatus
import app.together.v2.session.SessionController
import app.together.v2.network.closeOwned
import app.together.v2.network.awaitDecoded
import app.together.v2.session.Pairing
import app.together.v2.session.pairingBase
import kotlinx.coroutines.*
import okhttp3.*
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.RequestBody.Companion.toRequestBody
import org.json.JSONObject
import org.json.JSONArray
import java.io.File
import java.io.IOException
import java.util.UUID
import java.util.concurrent.TimeUnit

/** Fixed fields only. Error text, URLs, identity metadata and credentials are excluded. */
internal fun phoneSample(player:PlayerStatus,sync:SessionController.Diagnostics):JSONObject=JSONObject()
    .put("elapsed_ms",SystemClock.elapsedRealtime()).put("utc_ms",System.currentTimeMillis())
    .put("generation",sync.generation.coerceAtLeast(0)).put("position_ms",player.positionMs.coerceIn(0,604800000))
    .put("duration_ms",player.durationMs.coerceIn(0,604800000)).put("connected",sync.connected)
    .put("buffering",player.buffering).put("seeking",player.seeking).put("preparing",sync.preparing)
    .put("source_ready",player.sourceReady).put("fault",phonePlayerFailed(player) || sync.fault!=null)
    .put("playback",phonePlaybackState(player,sync.ended))
    .put("drift_ms",sync.driftMs.coerceIn(-604800000,604800000))
    .put("rtt_ms",sync.rttMs?.takeIf {it.isFinite()}?.toLong()?.coerceIn(0,60000) ?: JSONObject.NULL)

class PhoneLogs(context:Context) {
    private val storage=PhoneJournalStorage(File(context.noBackupFilesDir,"diagnostics"))
    val recordingFailed=storage.recordingFailed
    private val client=OkHttpClient.Builder().followRedirects(false).followSslRedirects(false)
        .connectTimeout(5,TimeUnit.SECONDS).readTimeout(5,TimeUnit.SECONDS).callTimeout(8,TimeUnit.SECONDS).build()
    suspend fun record(sample:JSONObject) {
        // Clone only approved fields before yielding to disk work. Caller-owned
        // JSON and accidental URL/error fields never become private log text.
        val safe=safePhoneSample(sample)
        storage.appendApproved(safe.toString())
    }
    suspend fun send(pairing:Pairing):Int {
        val samples=storage.tail(128) {line ->runCatching {JSONObject(line)}.getOrNull()?.let {
            // Only our exact numeric/boolean/enum schema can leave private storage.
            val keys=setOf("elapsed_ms","utc_ms","generation","position_ms","duration_ms","connected","buffering","seeking","preparing","source_ready","fault","playback","drift_ms","rtt_ms")
            if(it.keys().asSequence().toSet()==keys) runCatching {safePhoneSample(it)}.getOrNull() else null
        }}
        val address=pairingBase(pairing.baseUrl)+"/diagnostics/phone"
        for(chunk in samples.chunked(10)) {
            val body=JSONObject().put("protocol",1).put("batch_id",UUID.randomUUID().toString()).put("samples",JSONArray(chunk))
            val request=Request.Builder().url(address).header("Authorization","Bearer ${pairing.token}")
                .post(body.toString().toRequestBody("application/json".toMediaType())).build()
            try {
                client.newCall(request).awaitDecoded {response ->
                    if(response.code!=204) throw IOException("Logs could not be sent")
                }
            } catch(error:CancellationException) {throw error}
            catch(_:Exception) {throw IOException("Logs could not be sent")}
        }
        return samples.size
    }
    fun close() {client.closeOwned()}
}
internal fun safePhoneSample(input:JSONObject):JSONObject {
    val output=JSONObject()
    for(key in listOf("elapsed_ms","utc_ms","generation","position_ms","duration_ms")) {
        val maximum=if(key in listOf("position_ms","duration_ms")) 604800000L else Long.MAX_VALUE
        output.put(key,phoneInteger(input.get(key),0L..maximum))
    }
    for(key in listOf("connected","buffering","seeking","preparing","source_ready","fault")) output.put(key,phoneBoolean(input.get(key)))
    val playback=input.getString("playback");require(playback in setOf("starting","loading","playing","paused","ended","failed"));output.put("playback",playback)
    output.put("drift_ms",phoneInteger(input.get("drift_ms"),-604800000L..604800000L))
    require(input.has("rtt_ms"))
    if(input.isNull("rtt_ms")) output.put("rtt_ms",JSONObject.NULL) else output.put("rtt_ms",phoneInteger(input.get("rtt_ms"),0L..60000L))
    return output
}
