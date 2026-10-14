package app.together.v2.player

import java.io.File
import kotlinx.coroutines.*

/** Main-thread owner for optional local proof loads. A late preparation result
 * must never replace media after the authoritative session takes ownership. */
internal class LocalSourceLoader(
    private val scope:CoroutineScope,
    private val prepare:suspend ()->File,
    private val load:(File)->Unit,
    private val failed:()->Unit
) {
    private var pending:Job?=null
    fun request() {
        cancel()
        pending=scope.launch {
            try {
                val file=prepare()
                // Preparation can involve blocking IO or a non-cancellable cleanup.
                // Even if it returns after cancellation, delivery is forbidden.
                ensureActive()
                load(file)
            } catch(error:CancellationException) {throw error}
            catch(_:Exception) {
                // A replaced local request cannot publish an error into a session.
                ensureActive()
                failed()
            }
        }
    }
    fun cancel() {pending?.cancel();pending=null}
}
