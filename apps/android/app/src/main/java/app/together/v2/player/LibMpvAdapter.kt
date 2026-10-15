package app.together.v2.player

import android.content.Context
import android.util.Log
import android.os.SystemClock
import app.together.v2.BuildConfig
import android.view.Surface
import android.graphics.SurfaceTexture
import android.view.TextureView
import io.github.yuroyami.libmpvkt.*
import io.github.yuroyami.libmpvkt.view.MpvOptions
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.update
import java.io.File
import java.util.concurrent.atomic.AtomicLong

data class PlayerStatus(val initialized: Boolean = false, val state: String = "Starting player",
                        val positionMs: Long = 0, val durationMs: Long = 0, val seeking: Boolean = false,
                        val buffering: Boolean = false, val error: String? = null,
                        val sampledAtMs: Long = 0, val eof:Boolean=false, val sourceRevision:Long=0, val sourceReady:Boolean=false,
                        val failure:String?=null, val acceptedSourceRevision:Long=0)

/** Native boundary. Surface callbacks never perform blocking mpv calls on Main.
 * Reuses Synkplay's libmpvKt engine; lifecycle work shares the command worker.
 */
@OptIn(InternalLibmpvKtApi::class)
class LibMpvAdapter internal constructor(context: Context, private val lifecycleProof: NativeLifecycleProof? = null) {
    val view = TextureView(context)
    private val calls = Dispatchers.IO.limitedParallelism(1)
    private val cleanup = PlayerCleanup(calls) { Log.e("TogetherPlayer","cleanup_failed") }
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main.immediate)
    private val mutableStatus = MutableStateFlow(PlayerStatus())
    val status: StateFlow<PlayerStatus> = mutableStatus
    @Volatile private var core: Mpv? = null
    @Volatile private var closed = false
    @Volatile private var probeEnabled = true
    @Volatile private var startEnabled = true
    /** Debug fault injection exercises stale-worker barriers, not CDN buffering. */
    fun setProbeEnabledForProof(enabled:Boolean) {check(BuildConfig.DEBUG);probeEnabled=enabled}
    /** Simulate a command accepted by the worker which never starts native playback. */
    fun setStartEnabledForProof(enabled:Boolean) {check(BuildConfig.DEBUG);startEnabled=enabled}
    fun failSourceForProof(file:File) {check(BuildConfig.DEBUG);loadLocal(file)}
    /** Deterministic queued-load proof; holds only the command worker, never Main. */
    internal fun enqueueWorkerBarrierForProof(barrier:()->Unit) {check(BuildConfig.DEBUG);queue(tagged=false,block=barrier)}
    private var surface: Surface? = null
    private val surfaceGeneration = AtomicLong(0)
    private val commandEpoch = AtomicLong(0)
    private val sourceRevision=AtomicLong(0)
    private var activeSourceRevision=0L
    @Volatile private var sourceEntry:Long?=null
    fun invalidateCommands() { commandEpoch.incrementAndGet() }
    /** Local fixture preparation errors do not invent an authoritative native fault. */
    internal fun localPreparationFailed() {
        mutableStatus.update {it.withLocalPreparationFailure()}
        pause(true)
    }
    /** Controller classifies an EOF too early for the authoritative media timeline. */
    fun unexpectedEnd() {mutableStatus.update {it.withUnexpectedEnd()};pause(true)}
    private var texture: SurfaceTexture? = null
    private val callback=object: TextureView.SurfaceTextureListener {
        override fun onSurfaceTextureAvailable(st:SurfaceTexture,width:Int,height:Int) {
            val owned=Surface(st);surface=owned;texture=st
            attach(owned,width,height,surfaceGeneration.incrementAndGet())
        }
        override fun onSurfaceTextureSizeChanged(st:SurfaceTexture,width:Int,height:Int) {
            val nativeSurface=surface
            val generation=surfaceGeneration.get()
            queue(tagged=false) {
                val mpv=core ?: return@queue
                if(surfaceGeneration.get()==generation && mpv.attachedSurface===nativeSurface)
                    mpv.set(MpvProperties.AndroidSurfaceSize,"${width}x$height").getOrThrow()
            }
        }
        override fun onSurfaceTextureDestroyed(st:SurfaceTexture):Boolean {
            val previous=surface;surface=null;texture=null;surfaceGeneration.incrementAndGet()
            // Retain the texture until the worker has detached mpv. Returning true
            // would let Android destroy it while a native property call is pending.
            if(previous==null) return true
            // Player cancellation must not discard a queued detach/release.
            cleanup.enqueue {
                try {core?.let {if(it.attachedSurface===previous) it.detachSurface()}}
                finally {try {previous.release()} finally {st.release()}}
            }
            return false
        }
        override fun onSurfaceTextureUpdated(st:SurfaceTexture) = Unit
    }
    init {
        check(lifecycleProof == null || BuildConfig.DEBUG)
        view.surfaceTextureListener=callback
        scope.launch {
            try {
                val prepared=withContext(calls) {
                    lifecycleProof?.beforeCreate?.invoke()
                    Mpv.create(context.applicationContext).also {
                        core=it
                        lifecycleProof?.afterCreate?.invoke()
                        try {
                            // Raw decoder/HTTP messages can contain private media URLs.
                            // Keep typed EndFile/native status diagnostics instead.
                            // Match the resolver's direct-URL/no-redirect policy.
                            // TLS verification alone does not reject HTTP downgrades.
                            val options=mutableMapOf("pause" to "yes","tls-verify" to "yes",
                                "stream-lavf-o" to "max_redirects=0","ytdl" to "no",
                                "terminal" to "no","msg-level" to "all=no")
                            if(NativeTestAudio.muted) options["mute"]="yes"
                            MpvOptions(cacheDir=context.cacheDir,tlsCaFile=NativeTls.installSystemRoots(context.applicationContext),extra=options)
                                .applyTo(MpvOptions.OptionSink {name,value ->it.setOption(name,value).getOrThrow()})
                            it.requestLogMessages(io.github.yuroyami.libmpvkt.MpvLogLevel.None).getOrThrow()
                            it.initialize().getOrThrow()
                        } catch(t:Throwable) {it.close();core=null;throw t}
                    }
                }
                // close() owns disposal on calls, including when cancellation
                // discards this withContext result after native creation finishes.
                if(closed) return@launch
                mutableStatus.value=PlayerStatus(initialized=true,state="Idle")
                surface?.let { attach(it,view.width,view.height,surfaceGeneration.get()) }
                scope.launch {
                    prepared.events.collect {event ->
                        if(event is MpvEvent.EndFile && event.reason==EndFileReason.Error) {
                            withContext(calls) {
                                if(event.playlistEntryId==sourceEntry && activeSourceRevision==sourceRevision.get()) {
                                    mutableStatus.update {it.withDecoderFailure()}
                                    Log.e("TogetherPlayer","decoder_failed")
                                }
                            }
                        }
                    }
                }
                // Position/state and their timestamp have one owner: the native
                // probe below. Delayed playback-flow events must not overwrite
                // an older position while retaining a newer probe timestamp.
            } catch(e:Exception) {
                if(e is CancellationException) throw e
                mutableStatus.value=PlayerStatus(state="Failed",error="Native player initialization failed")
                Log.e("TogetherPlayer","initialization_failed")
            }
        }
        scope.launch {
            while(isActive) {
                delay(100)
                if(!probeEnabled) continue
                withContext(calls) {
                    val mpv=core ?: return@withContext
                    try {
                        val paused=mpv.get(MpvProperties.Pause).getOrThrow()
                        val seeking=mpv.get(MpvProperties.Seeking).getOrThrow()
                        val cache=mpv.get(MpvProperties.PausedForCache).getOrThrow()
                        val eof=mpv.get(MpvProperties.EofReached).getOrThrow()
                        val duration=runCatching {mpv.get(MpvProperties.Duration).getOrThrow()}.getOrDefault(0.0)
                        val nativePosition=runCatching {mpv.get(MpvProperties.TimePos).getOrThrow()}.getOrNull()
                        val positionSampledAtMs=SystemClock.elapsedRealtime()
                        val positionMs=nativeTimeMillis(nativePosition)
                        val durationMs=nativeTimeMillis(duration)
                        val playing=runCatching {mpv.get(MpvProperties.PlaylistPlayingPos).getOrThrow()}.getOrDefault(-1)
                        val entry=if(playing>=0) runCatching {mpv.get(MpvProperty.Int64("playlist/$playing/id")).getOrThrow()}.getOrNull() else null
                        mutableStatus.update { current -> current.copy(
                            state=nativePlaybackState(current.failure,durationMs,cache,paused),
                            positionMs=positionMs ?: 0,durationMs=durationMs ?: 0,
                            seeking=seeking,buffering=cache,eof=eof,
                            sourceReady=current.failure==null && current.sourceRevision==activeSourceRevision && activeSourceRevision==sourceRevision.get() && sourceEntry!=null && entry==sourceEntry && durationMs!=null && durationMs>0 && positionMs!=null,
                            sampledAtMs=positionSampledAtMs) }
                    } catch(e:Exception) {
                        if(e is CancellationException) throw e
                        // Freshness guard freezes the room if this worker stalls.
                    }
                }
            }
        }
    }
    private fun attach(nativeSurface:Surface,width:Int,height:Int,generation:Long) = queue(tagged=false) {
        val mpv=core ?: return@queue
        if(surfaceGeneration.get()!=generation || !nativeSurface.isValid) return@queue
        mpv.attachSurface(nativeSurface,"gpu").getOrThrow()
        if(width>0 && height>0) mpv.set(MpvProperties.AndroidSurfaceSize,"${width}x$height").getOrThrow()
    }
    private fun queue(tagged:Boolean=true,block:()->Unit) {
        if(closed || !mutableStatus.value.initialized) return
        val epoch=commandEpoch.get()
        scope.launch(calls) {
            if(closed || (tagged && epoch!=commandEpoch.get())) return@launch
            try {block()}
            catch(e:Exception) {
                if(e is CancellationException) throw e
                mutableStatus.update {it.withCommandFailure()}
                Log.e("TogetherPlayer","operation_failed")
            }
        }
    }
    private fun loadSource(source:String,submittedFault:String?):Long {
        val revision=sourceRevision.incrementAndGet()
        mutableStatus.update {it.copy(sourceRevision=revision,sourceReady=false)}
        // Source revisions own decoder work. A buffering/pause generation only
        // invalidates transport controls; it must not discard a pending load.
        queue(tagged=false) {
            // A newer source can be selected within the same command epoch.
            // Do not issue an obsolete loadfile before it reaches the native API.
            if(revision!=sourceRevision.get()) return@queue
            lifecycleProof?.beforeSourceLoad?.invoke(revision)
            // Main can classify an EOF while this worker was queued/held. A new
            // pending fault needs fresh recovery authority, not this older load.
            if(!NativeSourceLoadFence.canSubmit(revision,sourceRevision.get(),submittedFault,mutableStatus.value.failure)) return@queue
            val mpv=checkNotNull(core);mpv.set(MpvProperties.Pause,true).getOrThrow();mpv.set(MpvProperties.Speed,1.0).getOrThrow()
            sourceEntry=null
            mpv.command(MpvCommand.of("loadfile",source,"replace")).getOrThrow()
            sourceEntry=mpv.get(MpvProperty.Int64("playlist/0/id")).getOrThrow()
            activeSourceRevision=revision
            // CAS retries against the latest status so a failure arriving during
            // the native command is retained rather than cleared by completion.
            mutableStatus.update {NativeSourceLoadFence.complete(it,revision,submittedFault)}
        }
        return revision
    }
    fun loadLocal(file:File)=loadLocal(file,mutableStatus.value.failure)
    internal fun loadLocal(file:File,admittedFault:String?)=loadSource(file.absolutePath,admittedFault)
    /** Session controller owns this operation; URLs are bearer credentials and never logged. */
    fun loadHttps(url:String):Long=loadHttps(url,mutableStatus.value.failure)
    internal fun loadHttps(url:String,admittedFault:String?):Long {
        MediaUrl.validate(url)
        return loadSource(url,admittedFault)
    }
    fun pause(paused:Boolean) = queue {if(paused || startEnabled) checkNotNull(core).set(MpvProperties.Pause,paused).getOrThrow()}
    fun speed(rate:Double) {
        require(rate.isFinite() && rate in 0.95..1.005)
        queue {checkNotNull(core).set(MpvProperties.Speed,rate).getOrThrow()}
    }
    fun seek(positionMs:Long) = queue {
        checkNotNull(core).command(MpvCommand.of("seek",(positionMs.coerceAtLeast(0)/1000.0).toString(),"absolute+exact")).getOrThrow()
    }
    fun close() {
        if(closed) return
        closed=true;surfaceGeneration.incrementAndGet();scope.cancel()
        view.surfaceTextureListener=null
        val oldSurface=surface;surface=null;val oldTexture=texture;texture=null
        cleanup.finish {
            // Transfer ownership here, after any already-running synchronous
            // initialization. Capturing core on Main could capture null before
            // that initialization publishes its handle and leak the late handle.
            val owned=core;core=null
            try {
                try {owned?.detachSurface()} finally {
                    try {
                        owned?.close()
                        if(owned!=null) lifecycleProof?.afterClose?.invoke()
                    } finally {try {oldSurface?.release()} finally {oldTexture?.release()}}
                }
            } finally {lifecycleProof?.afterCleanup?.invoke()}
        }
    }
}

/** Debug-only instrumentation barriers; no handles, media or credentials leave the adapter. */
internal class NativeLifecycleProof(
    val beforeCreate: () -> Unit,
    val afterCreate: () -> Unit,
    val afterClose: () -> Unit,
    val afterCleanup: () -> Unit,
    val beforeSourceLoad: (Long) -> Unit = {},
)
