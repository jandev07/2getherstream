package app.together.v2.session

import app.together.v2.network.awaitDecoded
import app.together.v2.network.closeOwned
import okhttp3.OkHttpClient
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import org.json.JSONObject
import java.util.concurrent.TimeUnit

/** One-time setup. TLS validation is enabled and cross-origin redirects are rejected. */
class PairingClient {
    private val http=OkHttpClient.Builder().followRedirects(false).followSslRedirects(false).callTimeout(15,TimeUnit.SECONDS).build()
    suspend fun claim(server:String,code:String):Pairing {
        val base=pairingBase(server)
        val normalized=code.trim().replace("-","").lowercase()
        require(normalized.matches(Regex("[a-f0-9]{12}"))) {"Enter the code shown on Windows"}
        val body=JSONObject().put("code",normalized).toString().toRequestBody("application/json".toMediaType())
        val request=Request.Builder().url("$base/pair").post(body).build()
        return http.newCall(request).awaitDecoded {response ->
            if(response.code!=200) throw PairingRejected(response.code)
            val source=checkNotNull(response.body).source()
            if(source.request(4097)) throw InvalidPairingResponse()
            val bytes=source.readByteArray()
            try {
                val json=JSONObject(String(bytes,Charsets.UTF_8))
                // JSONObject getters coerce strings/numbers. Do not save a
                // coerced credential or accept a fractional protocol version.
                validatePairingProtocol(json.get("protocol"))
                val token=json.get("token");require(token is String)
                Pairing(base,token)
            } catch(_:Exception) {throw InvalidPairingResponse()}
        }
    }
    fun close() {http.closeOwned()}
}

internal fun validatePairingProtocol(value:Any) {
    require((value is Int && value==1) || (value is Long && value==1L)) {"Invalid pairing response"}
}

/** Fixed categories for UX; never include an endpoint, response body or token. */
class PairingRejected(val status:Int):Exception("Pairing rejected")
class InvalidPairingResponse:Exception("Invalid pairing response")
fun pairingFailureMessage(error:Exception):String=when(error) {
    is InvalidPairingResponse -> "Windows sent an incompatible pairing response. Check that both apps use the latest Alpha, then try a new code."
    is PairingRejected -> if(error.status==403) "The code expired or was already used. Generate a new code on Windows." else "Windows could not accept pairing. Keep the app open and try a new code."
    is java.net.SocketTimeoutException -> "Windows did not respond. Check the server address and keep the Windows app open."
    is javax.net.ssl.SSLException -> "The secure connection failed. Check the HTTPS address shown on Windows."
    is java.io.IOException -> "Could not reach Windows. For the PC emulator, use the local address shown on Windows. For a phone, use the HTTPS tunnel address."
    is IllegalArgumentException -> "Enter the server address and the 12-character code shown on Windows."
    else -> "Pairing could not be saved or opened. Generate a new code and try again."
}
