package app.together.v2.update

import app.together.v2.network.awaitDecoded
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.isActive
import okhttp3.Call
import okhttp3.Response
import java.io.IOException
import java.util.concurrent.TimeUnit

internal const val UPDATE_METADATA_TIMEOUT_MS=12_000L

private class UpdateReadFailure(code:Int):IOException(
    if(code==503) "Updates are not available yet. Try again later."
    else "Update could not be verified. Please try again."
)

/** Use the shared response owner, including late-response cleanup. Raw HTTP and
 * archive errors never become product feedback; cancellation remains cancellation. */
internal suspend fun <T> Call.awaitUpdateRead(timeoutMillis:Long?=null,read:(Response,()->Boolean)->T):T {
    val owner=currentCoroutineContext()
    if(timeoutMillis!=null) {
        require(timeoutMillis>0)
        timeout().timeout(timeoutMillis,TimeUnit.MILLISECONDS)
    }
    return try {
        awaitDecoded {response ->
            try {
                owner.ensureActive()
                val result=read(response) {owner.isActive}
                owner.ensureActive()
                result
            } catch(error:CancellationException) {throw error}
            catch(_:Exception) {throw UpdateReadFailure(response.code)}
        }
    } catch(error:CancellationException) {throw error}
    catch(error:UpdateReadFailure) {throw error}
    catch(_:Exception) {throw IOException("Could not connect. Check your connection and try again.")}
}
