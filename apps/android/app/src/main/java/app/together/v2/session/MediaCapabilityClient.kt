package app.together.v2.session

import app.together.v2.player.MediaUrl
import app.together.v2.BuildConfig
import app.together.v2.network.closeOwned
import app.together.v2.network.awaitDecoded
import kotlinx.coroutines.CancellationException
import okhttp3.*
import okhttp3.HttpUrl.Companion.toHttpUrl
import org.json.JSONObject
import java.io.IOException
import java.util.concurrent.TimeUnit

internal fun validMediaIdentity(identity:JSONObject):Boolean = MediaIdentityFields.valid(
    if(identity.has("provider")) identity.opt("provider") else "fixture",
    identity.opt("torrent_id"),identity.opt("file_id"))

internal fun sameMediaIdentity(first:JSONObject?,second:JSONObject?):Boolean {
    if(first==null || second==null) return first==null && second==null
    val keys=first.keys().asSequence().toSet()+second.keys().asSequence().toSet()
    return keys.all { key ->
        val received=first.opt(key);val expected=second.opt(key)
        if(received is Number && expected is Number) received.toString()==expected.toString() else received==expected
    }
}

/** Authenticated control metadata only. The native player reads media directly from its CDN. */
class MediaCapability internal constructor(val sourceRevision:Long,private val url:String) {
    // Deliberately not a data class: toString must never expose the private URL.
    fun <T> deliver(consumer:(String)->T):T=consumer(url)
}
class MediaCapabilityClient {
    private val closed=java.util.concurrent.atomic.AtomicBoolean(false)
    private val client=OkHttpClient.Builder().followRedirects(false).followSslRedirects(false)
        .connectTimeout(10,TimeUnit.SECONDS).readTimeout(10,TimeUnit.SECONDS).callTimeout(12,TimeUnit.SECONDS).build()

    suspend fun request(endpoint:String,credential:String,epoch:String,generation:Long,identity:JSONObject):MediaCapability {
        if(closed.get()) throw IOException("Media access closed")
        val scheme=if(endpoint.startsWith("wss://")) "https://" else "http://"
        val control=(scheme+endpoint.substringAfter("://")).toHttpUrl()
        require(control.username.isEmpty() && control.password.isEmpty())
        require(control.isHttps || (BuildConfig.DEBUG && control.host=="127.0.0.1"))
        require(control.encodedPath=="/session" && control.encodedQuery==null && control.fragment==null)
        val address=control.newBuilder().encodedPath("/media").addQueryParameter("epoch",epoch)
            .addQueryParameter("generation",generation.toString()).build()
        val request=Request.Builder().url(address).header("Authorization","Bearer $credential")
            .header("Cache-Control","no-store").build()
        return try {
            require(validMediaIdentity(identity) && identity.opt("provider")=="torbox")
            client.newCall(request).awaitDecoded {response ->
                require(response.code==200)
                val body=checkNotNull(response.body)
                // Bound reads independently of the peer's Content-Length.
                val buffer=java.io.ByteArrayOutputStream()
                body.byteStream().use {input ->
                    val chunk=ByteArray(2048)
                    while(true) {
                        val size=input.read(chunk);if(size<0) break
                        require(buffer.size()+size<=16384);buffer.write(chunk,0,size)
                    }
                }
                val value=JSONObject(buffer.toString(Charsets.UTF_8.name()))
                require(MediaGrantNumbers.matches(value.opt("protocol"),value.opt("generation"),value.opt("source_revision"),generation))
                require(value.opt("epoch") is String && value.getString("epoch")==epoch)
                val media=value.getJSONObject("media")
                require(validMediaIdentity(media) && media.opt("provider")=="torbox")
                require(sameMediaIdentity(media,identity))
                require(value.opt("url") is String)
                val direct=value.getString("url").also {MediaUrl.validate(it)}
                MediaCapability(checkNotNull(MediaGrantNumbers.integer(value.opt("source_revision"))),direct)
            }
        } catch(error:CancellationException) {throw error}
        catch(_:Exception) {throw IOException("Media access unavailable")}
    }

    fun close() {
        if(!closed.compareAndSet(false,true)) return
        client.closeOwned()
    }
}
