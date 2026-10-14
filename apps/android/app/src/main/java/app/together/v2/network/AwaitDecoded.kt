package app.together.v2.network

import kotlinx.coroutines.suspendCancellableCoroutine
import okhttp3.Call
import okhttp3.Callback
import okhttp3.Response
import java.io.IOException

/** Decode on OkHttp's worker while retaining response ownership. Cancellation
 * aborts both connection waits and body reads; a late response is still closed. */
internal suspend fun <T> Call.awaitDecoded(decode:(Response)->T):T = suspendCancellableCoroutine { continuation ->
    continuation.invokeOnCancellation {cancel()}
    enqueue(object:Callback {
        override fun onFailure(call:Call,e:IOException) {continuation.resumeWith(Result.failure(e))}
        override fun onResponse(call:Call,response:Response) {
            val result=runCatching {response.use {
                if(!continuation.isActive) throw kotlinx.coroutines.CancellationException("Request cancelled")
                decode(it)
            }}
            continuation.resumeWith(result)
        }
    })
}
