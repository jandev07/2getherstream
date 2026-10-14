package app.together.v2.update

import kotlinx.coroutines.withTimeoutOrNull
import okhttp3.HttpUrl
import okhttp3.HttpUrl.Companion.toHttpUrl
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.Response
import java.io.IOException
import java.util.concurrent.TimeUnit

private val apkCdnHosts=setOf("release-assets.githubusercontent.com","objects.githubusercontent.com")
private fun HttpUrl.githubAsset():Boolean = host=="github.com" &&
    encodedPath.matches(Regex("/[^/]+/[^/]+/releases/download/[^/]+/[^/]+")) && query==null
private fun HttpUrl.safeAddress():Boolean = isHttps && port==443 && username.isEmpty() &&
    password.isEmpty() && fragment==null

/** Redirects are exclusive to public GitHub release assets. Pairing/control
 * clients retain their no-redirect policy; signed CDN queries stay private. */
internal fun apkRedirect(from:HttpUrl,location:String):HttpUrl {
    require(from.safeAddress() && (from.githubAsset() || from.host in apkCdnHosts))
    require(location.length in 1..4096 && location.none {it.isISOControl()})
    val next=requireNotNull(from.resolve(location))
    require(next.safeAddress() && (next.githubAsset() || next.host in apkCdnHosts))
    return next
}

private sealed interface ApkReply<out T> {
    data class Redirect(val address:HttpUrl):ApkReply<Nothing>
    data class Complete<T>(val value:T):ApkReply<T>
}

/** Fresh unauthenticated requests, bounded hops and one total download budget.
 * Every intermediate body is closed by the existing cancellation-safe owner. */
internal suspend fun <T> OkHttpClient.awaitApkDownload(
    address:String,timeoutMillis:Long=TimeUnit.MINUTES.toMillis(15),
    read:(Response,()->Boolean)->T
):T {
    require(!followRedirects && !followSslRedirects && timeoutMillis>0)
    val completed=withTimeoutOrNull(timeoutMillis) {
        var current=address.toHttpUrl()
        require(current.safeAddress() && current.query==null)
        val visited=mutableSetOf(current)
        val began=System.nanoTime()
        var redirects=0
        var result:ApkReply.Complete<T>?=null
        while(result==null) {
            val remaining=timeoutMillis-TimeUnit.NANOSECONDS.toMillis(System.nanoTime()-began)
            if(remaining<=0) throw IOException("Update download timed out. Please try again.")
            val request=Request.Builder().url(current).header("Accept-Encoding","identity").build()
            val reply=newCall(request).awaitUpdateRead(remaining) {response,active ->
                if(response.code in setOf(301,302,303,307,308)) {
                    require(redirects<3)
                    ApkReply.Redirect(apkRedirect(current,requireNotNull(response.header("Location"))))
                } else {
                    require(response.code==200)
                    ApkReply.Complete(read(response,active))
                }
            }
            when(reply) {
                is ApkReply.Redirect -> {
                    if(!visited.add(reply.address)) throw IOException("Update could not be verified. Please try again.")
                    current=reply.address;redirects++
                }
                is ApkReply.Complete -> result=reply
            }
        }
        result
    } ?: throw IOException("Update download timed out. Please try again.")
    return completed.value
}
