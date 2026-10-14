package app.together.v2.diagnostics

import kotlinx.coroutines.*

/** Lifecycle calls are serialized by the component owner; no background timer is retained. */
internal class VisibleJournalSampler(private val scope:CoroutineScope,private val intervalMs:Long=2000,
    private val record:suspend ()->Unit) {
    private var job:Job?=null
    private var closed=false
    init {require(intervalMs>0)}
    fun setActive(active:Boolean) {
        if(closed) return
        if(!active) {job?.cancel();job=null;return}
        if(job!=null) return
        job=scope.launch {while(isActive) {record();delay(intervalMs)}}
    }
    fun close() {closed=true;job?.cancel();job=null}
}
