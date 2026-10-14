package app.together.v2.session

import app.together.v2.BuildConfig
import java.net.URI

/** Header-safe device bearer text. Reject before saving it or constructing an
 * Authorization header; fixed errors never reveal the supplied credential. */
internal fun validatePairingToken(token:String) {
    require(token.length in 32..256 && token.all {it.code in 33..126}) {"Invalid pairing credential"}
}

/** The socket boundary accepts only the paired control endpoint, not an arbitrary path. */
internal fun sessionEndpoint(input:String):String {
    val invalid="Invalid session address"
    require(input.length<=8192 && input.toByteArray(Charsets.UTF_8).size<=8192) {invalid}
    val parsed=runCatching {URI(input)}.getOrNull()
    require(parsed!=null && parsed.rawPath=="/session") {invalid}
    val address=when {
        input.startsWith("wss://") -> "https://"+input.substring(6)
        input.startsWith("ws://") -> "http://"+input.substring(5)
        else -> throw IllegalArgumentException(invalid)
    }
    val base=pairingBase(address.removeSuffix("/session"))
    return (if(base.startsWith("https://")) "wss://"+base.substring(8) else "ws://"+base.substring(7))+"/session"
}

/** Validate before issuing a claim or replacing encrypted pairing. URI parser
 * exceptions can include the original address and any accidentally pasted key. */
fun pairingBase(input:String):String {
    val invalid="Enter the Together server HTTPS address"
    require(input.length<=8192 && input.toByteArray(Charsets.UTF_8).size<=8192) {invalid}
    val address=input.trim()
    require(address.none {it.isWhitespace() || Character.isISOControl(it)}) {invalid}
    val parsed=runCatching {URI(address)}.getOrNull()
    require(parsed!=null && !parsed.host.isNullOrEmpty() && parsed.rawUserInfo==null &&
        parsed.rawQuery==null && parsed.rawFragment==null && parsed.rawPath in listOf("","/") &&
        (parsed.port==-1 || parsed.port in 1..65535) && !parsed.rawAuthority.endsWith(':')) {invalid}
    require(parsed.scheme=="https" || (BuildConfig.DEBUG && parsed.scheme=="http" &&
        parsed.host=="127.0.0.1" && parsed.port==47820)) {"Pairing requires HTTPS"}
    return parsed.toASCIIString().trimEnd('/')
}
