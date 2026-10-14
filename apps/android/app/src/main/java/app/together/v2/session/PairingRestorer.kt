package app.together.v2.session

import kotlinx.coroutines.*

/** Owner calls are serialized on Main. A user connection choice supersedes restore. */
internal class PairingRestorer<T:Any>(private val scope:CoroutineScope,private val load:suspend ()->T?,
    private val deliver:(T)->Unit,private val failed:()->Unit) {
    private var job:Job?=null
    private var revision=0L
    private var closed=false
    fun start() {
        if(closed) return
        cancel()
        val request=revision
        job=scope.launch {
            try {
                val value=load()
                if(isActive && !closed && request==revision && value!=null) deliver(value)
            } catch(error:CancellationException) {throw error}
            catch(_:Exception) {if(isActive && !closed && request==revision) failed()}
        }
    }
    fun cancel() {revision++;job?.cancel();job=null}
    fun close() {closed=true;cancel()}
}
