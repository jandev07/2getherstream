package app.together.v2.session

import android.content.Context
import android.os.SystemClock
import android.util.Log
import app.together.v2.BuildConfig
import app.together.v2.network.closeOwned
import app.together.v2.network.controlSocketClient
import app.together.v2.player.LibMpvAdapter
import app.together.v2.player.PlayerStatus
import app.together.v2.player.ProofMedia
import app.together.v2.sync.*
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.*
import okhttp3.*
import org.json.JSONObject
import kotlin.math.abs

/** Authoritative-state follower. Socket callbacks, intention queue and scheduling
 * are serialized on Main; all mpv work stays in the adapter's worker queue.
 * Media capabilities use an authenticated control request; media bytes go directly to libmpv.
 */
class SessionController(private val context: Context, private val player: LibMpvAdapter) {
    private val mediaFailureNotice="Couldn't prepare the video yet. You can tap Play to retry."
    private val commandFailureNotice="Control request rejected; state refreshed"
    // Matches the reducer: one frame at 10 fps, independent of drift correction.
    private val readyPositionToleranceMs = 100L
    data class Diagnostics(val connected: Boolean = false, val generation: Long = 0,
        val revision: Long = 0, val preparing: Boolean = false, val readyCount: Int = 0,
        val rttMs: Double? = null, val offsetMs: Double? = null, val driftMs: Long = 0,
        val rate: Double = 1.0, val error: String? = null, val ended:Boolean=false,
        val executionCount:Int=0, val fault:String?=null, val mediaAvailable:Boolean=false,
        val authoritativePlaying:Boolean=false,val desiredPlaying:Boolean=false,
        val clockSampleAgeMs:Long?=null,val clockHistorySize:Int=0,
        val lastClockRttMs:Double?=null,val lastClockOffsetMs:Double?=null,
        val clockLowerOffsetMs:Double?=null,val clockUpperOffsetMs:Double?=null,
        val clockSupportSamples:Int=0,val rawSelectedOffsetMs:Double?=null,
        val clockSupportAgeMs:Long?=null)
    private val mutable = MutableStateFlow(Diagnostics())
    val diagnostics: StateFlow<Diagnostics> = mutable
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main.immediate)
    private val http = controlSocketClient()
    private val reconnect=ReconnectPolicy()
    private val clock = ClockEstimator()
    private val clockRequests = ClockRequests()
    private val correction = DriftController()
    private var socket: WebSocket? = null
    private var endpoint: String? = null
    private var credential: String? = null
    private var retry: Job? = null
    private var stopped = true
    private var state: JSONObject? = null
    private var generation = -1L
    private var epoch: String? = null
    private var seq = 0L
    private var lastSeen = 0L
    private var inflight: Long? = null
    private var inflightAuthority:QueuedControl<String>?=null
    private var inflightAt=0L
    private var loadJob:Job?=null
    private var wanted: QueuedControl<JSONObject>? = null
    private var sentReady = false
    private var sentEnded=false
    private var sentExecution=false
    private var reportedBuffering: Boolean? = null
    private var preparedAt = 0L
    private var seekPending = false
    private var loaded = false
    private var expectedSourceRevision=0L
    private var loading = false
    private val mediaCapabilities=MediaCapabilityClient()
    private var mediaRetryAt=0L
    private var loadedIdentity:String?=null
    private var providerBinding:MediaSourceBinding?=null
    private var loadAttemptGeneration=-1L
    private var readySince: Long? = null
    private var started = false
    private var rate = 1.0
    private var correctedSampleAt = -1L
    private var rejectNextCommandForProof = false
    init {
        scope.launch {
            while (isActive) {
                // Keep scheduling precision while media is present. An idle or
                // disconnected app has no playback deadline and should not wake
                // Main one hundred times a second just to return from follow().
                delay(when {
                    !mutable.value.connected -> 250L
                    state == null || state!!.isNull("media") -> 100L
                    else -> 10L
                })
                if (mutable.value.connected && now()-lastSeen>6000) lost(socket,"control_silence")
                if (mutable.value.connected && inflight!=null && now()-inflightAt>8000) lost(socket,"ack_timeout")
                try { follow() }
                catch(e:Exception) {
                    if(e is CancellationException) throw e
                    // A malformed state must not kill this child while clock
                    // callbacks keep the socket looking connected indefinitely.
                    lost(socket,"invalid_playback_state")
                }
            }
        }
        scope.launch {
            while (isActive) {
                delay(500)
                requestClock()
            }
        }
    }
    private fun now() = SystemClock.elapsedRealtime()
    private fun resetClock() {
        clock.reset();clockRequests.reset()
        mutable.value=mutable.value.copy(rttMs=null,offsetMs=null,clockSampleAgeMs=null,
            clockHistorySize=0,lastClockRttMs=null,lastClockOffsetMs=null,
            clockLowerOffsetMs=null,clockUpperOffsetMs=null,clockSupportSamples=0,rawSelectedOffsetMs=null,clockSupportAgeMs=null)
    }
    private fun requestClock() {
        if (!mutable.value.connected) return
        val current=socket ?: return
        val stamp=now()
        if (clockRequests.issue(stamp) && !current.send(JSONObject().put("type","clock").put("protocol",1).put("t0_ms",stamp).toString())) {
            lost(current,"clock_send_failed")
        }
    }
    fun connect(url: String, token: String) {
        val validated=sessionEndpoint(url)
        validatePairingToken(token)
        disconnect(); endpoint=validated; credential=token; stopped=false; open()
    }
    private fun open() {
        if (stopped) return
        val request=Request.Builder().url(checkNotNull(endpoint)).header("Authorization","Bearer ${checkNotNull(credential)}").build()
        socket=http.newWebSocket(request, object: WebSocketListener() {
            override fun onOpen(webSocket: WebSocket, response: Response) { scope.launch {
                if (socket !== webSocket) return@launch
                lastSeen=now(); seq=0; resetClock(); state=null; inflight=null;inflightAuthority=null
                mutable.value=Diagnostics(connected=true)
                requestClock()
            } }
            override fun onMessage(webSocket: WebSocket, text: String) { val received=now(); scope.launch {
                if (socket !== webSocket) return@launch
                if (text.toByteArray().size > 16384) { lost(webSocket); return@launch }
                try {
                    val message=JSONObject(text)
                    when(message.getString("type")) {
                        "clock" -> {
                            if (MediaGrantNumbers.integer(message.opt("protocol")) != 1L || message.opt("epoch") != epoch) return@launch
                            val echo=clockReplyMillis(message.opt("t0_ms")) ?: return@launch
                            val serverReceived=clockReplyMillis(message.opt("t1_ms")) ?: return@launch
                            val serverSent=clockReplyMillis(message.opt("t2_ms")) ?: return@launch
                            if(serverSent<serverReceived) return@launch
                            if (clockRequests.consume(echo,received)) {
                                // Only this connection's one-use clock reply is
                                // liveness. Old/replayed/unmatched traffic must
                                // not postpone the control-silence pause.
                                // A genuine delayed estimator outlier still
                                // proves transport liveness.
                                lastSeen=received
                                val candidate=clock.observe(echo,serverReceived,serverSent,received.toDouble())
                                if(candidate!=null) mutable.value=mutable.value.copy(lastClockRttMs=candidate.rttMs,
                                    lastClockOffsetMs=candidate.offsetMs,clockHistorySize=clock.sampleCount)
                            }
                        }
                        "snapshot", "ack" -> {
                            val acknowledgement=if(message.getString("type")=="ack") {
                                requireNotNull(MediaGrantNumbers.integer(message.opt("seq"))).also {require(it>0)}
                            } else null
                            // Ignored stale state is neither liveness nor command acknowledgement.
                            if (!accept(message.getJSONObject("state"))) return@launch
                            lastSeen=received
                            if (acknowledgement!=null && acknowledgement==inflight) {
                                val completedAuthority=inflightAuthority
                                inflight=null;inflightAuthority=null
                                if (!message.isNull("error")) {
                                    // Stale acknowledgements cannot authorize a player effect.
                                    mutable.value=mutable.value.copy(error=commandFailureNotice)
                                    sentReady=false
                                    sentEnded=false
                                    sentExecution=false
                                } else {
                                    // ACK state is captured atomically with the successful command.
                                    // Only this matched success may carry a waiting control through
                                    // the command's own generation advance (e.g. rapid Play/Pause).
                                    if(completedAuthority!=null) wanted=wanted?.afterSuccessfulCommand(completedAuthority,epoch,generation)
                                    if(mutable.value.error==commandFailureNotice) mutable.value=mutable.value.copy(error=null)
                                }
                            }
                            flush()
                        }
                        else -> lost(webSocket,"invalid_control_type")
                    }
                } catch (e: Exception) { lost(webSocket,"invalid_control_${e.javaClass.simpleName}") }
            } }
            override fun onFailure(webSocket: WebSocket, t: Throwable, response: Response?) { scope.launch {
                if(socket !== webSocket) return@launch
                if(response?.code in listOf(401,403)) {
                    disconnect();mutable.value=Diagnostics(error="Pairing credential rejected; pair this device again")
                } else lost(webSocket,"transport_failure")
            } }
            override fun onClosing(webSocket: WebSocket, code: Int, reason: String) { scope.launch {
                // A peer close is already loss of control authority. Do not wait
                // for final socket teardown/control-silence timeout to pause and
                // reconnect; never log the peer-supplied reason text.
                lost(webSocket,"server_closing")
            } }
            override fun onClosed(webSocket: WebSocket, code: Int, reason: String) { scope.launch { lost(webSocket) } }
        })
    }
    private fun accept(next: JSONObject):Boolean {
        require(MediaGrantNumbers.integer(next.opt("protocol"))==1L)
        val incomingRevision=requireNotNull(MediaGrantNumbers.integer(next.opt("revision"))).also {require(it>=0)}
        val incomingGeneration=requireNotNull(MediaGrantNumbers.integer(next.opt("generation"))).also {require(it>=0)}
        val incomingEpoch=requireNotNull(PlaybackSnapshotFields.epoch(next.opt("epoch")))
        if (epoch==incomingEpoch && state != null && incomingRevision < state!!.getLong("revision")) return false
        if(epoch==incomingEpoch && state!=null && generation==incomingGeneration) {
            require(sameMediaIdentity(state!!.optJSONObject("media"),next.optJSONObject("media"))) {"Media changed without a new generation"}
        }
        // Preserve the wire default only for an absent ended field; explicit null
        // or string booleans must not become accepted playback state.
        val ended=if(next.has("ended")) next.opt("ended") else false
        val roomFault=if(next.isNull("fault")) null else next.opt("fault")
        require(PlaybackSnapshotFields.terminal(ended,roomFault))
        val timeline=next.getJSONObject("timeline")
        require(PlaybackSnapshotFields.timeline(timeline.opt("position_ms"),timeline.opt("anchor_ms"),
            timeline.opt("playing"),next.opt("desired_playing")))
        val prep=if(next.isNull("prepare")) null else next.getJSONObject("prepare").also {
            require(PlaybackSnapshotFields.preparation(it.opt("generation"),it.opt("position_ms"),
                it.opt("deadline_ms"),incomingGeneration))
            // The diagnostic READY count must be safe before loading starts too.
            val ready=it.getJSONArray("ready")
            require(ready.length()<=2)
            for(index in 0 until ready.length()) {
                val actor=ready.opt(index)
                require(actor=="pc" || actor=="phone")
                if(index>0) require(actor!=ready.opt(index-1))
            }
        }
        val participants=next.getJSONObject("participants")
        var executionCount=0
        for(index in 0..1) {
            val participant=participants.getJSONObject(if(index==0) "pc" else "phone")
            val recordedFault=if(participant.isNull("player_fault")) null else participant.opt("player_fault")
            require(PlaybackSnapshotFields.participant(participant.opt("connected"),participant.opt("buffering"),
                participant.opt("rtt_ms"),recordedFault))
            if(!participant.isNull("execution")) {
                val execution=participant.getJSONObject("execution")
                require(PlaybackSnapshotFields.execution(execution.opt("generation"),execution.opt("playing"),
                    execution.opt("position_ms"),execution.opt("observed_server_ms"),execution.opt("received_server_ms")))
                if(MediaGrantNumbers.integer(execution.opt("generation"))==incomingGeneration) executionCount++
            }
        }
        // Validate every snapshot before consulting loaded/native readiness.
        // The old fixture must never stand in for an unresolved new source.
        if (!next.isNull("media")) {
            val identity=next.getJSONObject("media")
            require(validMediaIdentity(identity))
            if(loadedIdentity?.let {sameMediaIdentity(JSONObject(it),identity)}!=true) loaded=false
        }
        if (epoch != incomingEpoch) { epoch=incomingEpoch; generation=-1; resetClock(); loaded=false }
        if (generation != incomingGeneration) {
            cancelLoading()
            mediaRetryAt=0L
            if(mutable.value.error==mediaFailureNotice) mutable.value=mutable.value.copy(error=null)
            // A fresh grant is bound to each prepare generation, including recovery.
            if(!next.isNull("media") && next.getJSONObject("media").optString("provider")=="torbox") loaded=false
            // An unaccepted current source is still pending on the native worker.
            // Do not reload it just because buffering advanced authority.
            if(player.status.value.sourceRevision!=expectedSourceRevision) loaded=false
            generation=incomingGeneration;player.invalidateCommands();player.pause(true);player.speed(1.0);rate=1.0;correction.reset()
            seekPending=!next.isNull("media");sentReady=false;readySince=null;started=false;preparedAt=now();reportedBuffering=null
            sentEnded=false
            sentExecution=false;correctedSampleAt=-1L
        }
        val nativeFault=player.status.value.failure
        val nativeFailure=nativeFault!=null
        val recoveryAuthorized=recoveryAuthorized(next,nativeFault)
        if(nativeFailure && recoveryAuthorized) loaded=false
        state=next
        reconnect.validatedSnapshot(now())
        if (!next.isNull("media") && next.isNull("fault") && !loaded && !loading && (!nativeFailure || recoveryAuthorized) && player.status.value.initialized && now()>=mediaRetryAt) {
            loading=true;loadAttemptGeneration=generation
            val loadGeneration=generation;val loadEpoch=epoch;val loadSocket=socket
            val loadIdentity=JSONObject(next.getJSONObject("media").toString())
            loadJob=scope.launch {
                try {
                    if(loadIdentity.optString("provider","fixture")=="fixture") {
                        val fixture=withContext(Dispatchers.IO) {ProofMedia.prepare(context)}
                        ensureActive()
                        if(generation!=loadGeneration || epoch!=loadEpoch || socket !== loadSocket || stopped) return@launch
                        val admittedFault=player.status.value.failure
                        requireRecoveryAuthority(admittedFault)
                        expectedSourceRevision=player.loadLocal(fixture,admittedFault)
                        providerBinding=null
                    } else {
                        val capability=mediaCapabilities.request(checkNotNull(endpoint),checkNotNull(credential),checkNotNull(loadEpoch),loadGeneration,loadIdentity)
                        ensureActive()
                        if(generation!=loadGeneration || epoch!=loadEpoch || socket !== loadSocket || stopped) return@launch
                        val sameIdentity=loadedIdentity?.let {sameMediaIdentity(JSONObject(it),loadIdentity)}==true
                        val sameAuthority=sameIdentity && providerBinding?.epoch==loadEpoch && providerBinding?.providerRevision==capability.sourceRevision
                        // A fresh grant authenticates the generation; it does not
                        // authorize discarding a pending decoder result. Wait for
                        // readiness/failure while callbacks and native work continue.
                        val settleUntil=now()+15000
                        while(sameAuthority) {
                            val pending=player.status.value
                            if(pending.sourceRevision!=expectedSourceRevision || pending.failure!=null ||
                                (pending.acceptedSourceRevision==expectedSourceRevision && pending.sourceReady)) break
                            if(now()>=settleUntil) error("Native source is still preparing")
                            delay(25);ensureActive()
                            if(generation!=loadGeneration || epoch!=loadEpoch || socket!==loadSocket || stopped) return@launch
                        }
                        val native=player.status.value
                        // Recheck the actual pending result after network/native waits;
                        // a previously authorized different fault is insufficient.
                        requireRecoveryAuthority(native.failure)
                        val nativeFresh=native.sampledAtMs>0 && now()-native.sampledAtMs in 0..1000
                        val reuse=sameIdentity && providerBinding?.reusable(checkNotNull(loadEpoch),capability.sourceRevision,
                            native.sourceRevision,native.acceptedSourceRevision,native.sourceReady && nativeFresh,native.failure!=null)==true
                        if(!reuse) expectedSourceRevision=capability.deliver {player.loadHttps(it,native.failure)}
                        providerBinding=MediaSourceBinding(checkNotNull(loadEpoch),capability.sourceRevision,expectedSourceRevision)
                    }
                    loadedIdentity=loadIdentity.toString();loaded=true;loading=false;preparedAt=now();mediaRetryAt=0L
                    if(mutable.value.error==mediaFailureNotice) mutable.value=mutable.value.copy(error=null)
                } catch(e:CancellationException) {throw e}
                catch(_:Exception) {
                    if(generation==loadGeneration && epoch==loadEpoch && socket===loadSocket) {
                        loading=false;loaded=false;mediaRetryAt=now()+3000;player.pause(true)
                        mutable.value=mutable.value.copy(error=mediaFailureNotice)
                    }
                }
            }
        }
        mutable.value=mutable.value.copy(generation=generation,revision=incomingRevision,preparing=prep!=null,
            ended=ended as Boolean,
            fault=roomFault as String?,
            mediaAvailable=!next.isNull("media"),
            authoritativePlaying=next.getJSONObject("timeline").getBoolean("playing"),desiredPlaying=next.getBoolean("desired_playing"),
            executionCount=executionCount,
            readyCount=prep?.getJSONArray("ready")?.length() ?: if(next.isNull("media") || !next.isNull("fault")) 0 else 2)
        return true
    }
    // A relative target may only use the settled native source for current
    // authority. Internal absolute recovery/correction intentions stay separate.
    private fun relativeSeekStatus():PlayerStatus? {
        val snapshot=state ?: return null
        if(!mutable.value.connected || socket==null || snapshot.isNull("media") ||
            !snapshot.isNull("prepare") || !snapshot.isNull("fault") || !loaded || loading || seekPending) return null
        val native=player.status.value
        return native.takeIf {it.initialized && it.durationMs>0 && !it.seeking && !it.buffering && it.failure==null &&
            it.sourceReady && expectedSourceRevision>0 && it.sourceRevision==expectedSourceRevision &&
            it.acceptedSourceRevision==expectedSourceRevision && it.sampledAtMs>0 && now()-it.sampledAtMs in 0..1000}
    }
    fun canSeek():Boolean=relativeSeekStatus()!=null
    fun seekBy(deltaMs:Long) {
        val native=relativeSeekStatus() ?: return
        intent("seek",(native.positionMs+deltaMs).coerceIn(0,native.durationMs))
    }
    fun intent(type: String, positionMs: Long? = null) {
        require(type in listOf("play","pause","seek"))
        // Stale screen callbacks while disconnected must not queue a future command.
        if(!mutable.value.connected || state==null || socket==null) return
        val authority=epoch ?: return
        wanted=QueuedControl(authority,generation,JSONObject().put("type",type).also {
            if(positionMs!=null) it.put("position_ms",positionMs.coerceAtLeast(0))
        })
        flush()
    }
    private fun flush() {
        if (inflight!=null || !mutable.value.connected || state==null) return
        val pending=wanted ?: return
        wanted=null
        val action=pending.forAuthority(epoch,generation) ?: return
        send(action)
    }
    private fun send(action: JSONObject): Boolean {
        if (inflight!=null || !mutable.value.connected || state==null) return false
        val current=socket ?: return false
        val next=++seq
        val rejectedForProof=BuildConfig.DEBUG && rejectNextCommandForProof
        rejectNextCommandForProof=false
        val accepted=!rejectedForProof && current.send(JSONObject().put("type","command").put("protocol",1).put("epoch",epoch)
            .put("seq",next).put("generation",generation).put("action",action).toString())
        if(accepted) {inflight=next;inflightAt=now();inflightAuthority=QueuedControl(checkNotNull(epoch),generation,action.getString("type"))}
        else lost(current,"command_send_failed")
        return accepted
    }
    private fun recoveryAuthorized(snapshot:JSONObject,nativeFault:String?,freshAttempt:Boolean=loadAttemptGeneration!=generation):Boolean =
        PlayerFaultAuthority.canRecover(nativeFault,snapshot.getJSONObject("participants").getJSONObject("phone").opt("player_fault"),
            !snapshot.isNull("prepare"),snapshot.isNull("fault"),freshAttempt)

    // This load already owns its attempt; recheck the latest authority after waits
    // without requiring a second attempt in the same generation.
    private fun requireRecoveryAuthority(nativeFault:String?) {
        if(nativeFault!=null && !recoveryAuthorized(checkNotNull(state),nativeFault,freshAttempt=true))
            error("Native recovery requires authority")
    }

    private fun follow() {
        val s=state ?: return
        if (!mutable.value.connected) return
        val p=player.status.value
        if(!p.initialized) return
        val sourceReady=p.sourceReady && p.sourceRevision==expectedSourceRevision
        val nativeFresh=p.sampledAtMs>0 && now()-p.sampledAtMs in 0..1000
        val serverPlayerFault=s.getJSONObject("participants").getJSONObject("phone").optString("player_fault","").takeUnless {it=="null" || it.isEmpty()}
        if(p.failure!=null && !s.isNull("media")) {
            // A buffering ACK may change generation while a native failure waits
            // behind an inflight command. Never replace the failed source before
            // the PC records it and a user explicitly requests recovery.
            val recoveryAuthorized=recoveryAuthorized(s,p.failure)
            if(recoveryAuthorized) {accept(s);return}
            if(loading || p.sourceRevision!=expectedSourceRevision) return
            player.pause(true)
            if((serverPlayerFault!=p.failure || !s.isNull("prepare")) && wanted==null) send(JSONObject().put("type","player_failed").put("code",p.failure))
            return
        }
        if (!loaded && !loading && !s.isNull("media") && s.isNull("fault")) {
            // The latest authority snapshot is already validated. While a failed
            // grant waits for its retry deadline, do not reparse it every 10 ms.
            if(now()>=mediaRetryAt) accept(s)
            return
        }
        // Selection and its reported ages describe one calibration instant.
        // A second clock read can cross expiry after a still-fresh selection.
        val calibratedAt=now().toDouble()
        val estimate=clock.estimateInterval(calibratedAt)
        val sample=estimate?.sample
        mutable.value=mutable.value.copy(rttMs=sample?.rttMs,offsetMs=sample?.offsetMs,
            clockSampleAgeMs=sample?.let {(calibratedAt-it.receivedMs).toLong()},clockHistorySize=clock.sampleCount,
            clockLowerOffsetMs=estimate?.lowerOffsetMs,clockUpperOffsetMs=estimate?.upperOffsetMs,
            clockSupportSamples=estimate?.supportSamples ?: 0,rawSelectedOffsetMs=estimate?.lowestRttOffsetMs,
            clockSupportAgeMs=estimate?.let {(calibratedAt-it.oldestReceivedMs).toLong()})
        val prep=s.optJSONObject("prepare")
        val timeline=s.getJSONObject("timeline")
        val target=prep?.getLong("position_ms") ?: timeline.getLong("position_ms")
        if(serverPlayerFault!=null && sourceReady && nativeFresh) {
            if(wanted==null) send(JSONObject().put("type","player_recovered"))
            return
        }
        if(p.eof && nativeFresh && sourceReady && started && timeline.getBoolean("playing") && prep==null && !p.seeking && !sentEnded && wanted==null) {
            val checkedAt=now()
            val estimatedPosition=sample?.let {PlaybackTiming.targetAtSample(target,timeline.getLong("anchor_ms"),true,checkedAt,checkedAt,it.offsetMs)}
            if(p.durationMs>0 && (abs(p.positionMs-p.durationMs)>1000 || (estimatedPosition!=null && p.durationMs-estimatedPosition>2500))) {
                player.unexpectedEnd();return
            }
            sentEnded=send(JSONObject().put("type","ended").put("position_ms",p.positionMs).put("duration_ms",p.durationMs))
        }
        if(!mutable.value.connected) return // send() may have invalidated this snapshot.
        if (seekPending && loaded && sourceReady && p.durationMs>0 && now()-preparedAt>150) {
            player.pause(true);player.seek(target); seekPending=false;preparedAt=now();readySince=null;return
        }
        val unavailable=p.buffering || !nativeFresh || !sourceReady || sample==null
        if (loaded && !seekPending && reportedBuffering!=unavailable && wanted==null) {
            val serverBuffering=s.getJSONObject("participants").getJSONObject("phone").getBoolean("buffering")
            if(serverBuffering==unavailable) reportedBuffering=unavailable
            else if(send(JSONObject().put("type","buffering").put("active",unavailable))) reportedBuffering=unavailable
        }
        if(!mutable.value.connected) return // Never resume from the captured pre-loss state.
        if(prep!=null) {
            started=false
            if (p.state=="Playing") player.pause(true)
            if(loaded && sourceReady && !seekPending && !p.seeking && !unavailable && p.state=="Paused" &&
                now()-preparedAt>=150 && abs(p.positionMs-target)<=readyPositionToleranceMs) {
                val since=readySince ?: now().also {readySince=it}
                if(now()-since>=150 && !sentReady && wanted==null) sentReady=send(JSONObject().put("type","ready").put("position_ms",p.positionMs))
            } else readySince=null
            return
        }
        if(timeline.getBoolean("playing") && s.isNull("fault") && sample!=null && loaded && sourceReady && !unavailable && !seekPending) {
            val serverNow=(now()+sample.offsetMs).toLong()
            val anchor=timeline.getLong("anchor_ms")
            if(!started && serverNow>=anchor) {
                if(serverNow-anchor>200) { intent("seek",target+(serverNow-anchor));return }
                player.pause(false);started=true
            }
            if(started) {
                val authoritative=PlaybackTiming.targetAtSample(target,anchor,true,p.sampledAtMs,now(),sample.offsetMs) ?: return
                mutable.value=mutable.value.copy(driftMs=p.positionMs-authoritative)
                if(correctedSampleAt!=p.sampledAtMs) {
                correctedSampleAt=p.sampledAtMs
                when(val result=correction.update(authoritative,p.positionMs,p.sampledAtMs,generation,!p.seeking && p.state=="Playing",true)) {
                    is Correction.Rate -> {if(rate!=result.value) player.speed(result.value);rate=result.value;mutable.value=mutable.value.copy(rate=rate)}
                    is Correction.Seek -> intent("seek",result.positionMs)
                    Correction.Hold -> {}
                }
                }
            }
        } else {
            started=false
            if(p.state=="Playing") player.pause(true)
            if(rate!=1.0) {player.speed(1.0);rate=1.0}
        }
        if(s.isNull("fault") && loaded && sourceReady && nativeFresh && !unavailable && !seekPending && !p.seeking && !sentExecution && wanted==null) {
            val checkedAt=now()
            val observedServer=PlaybackTiming.serverAtSample(p.sampledAtMs,checkedAt,sample.offsetMs) ?: return
            val playing=timeline.getBoolean("playing")
            val anchor=timeline.getLong("anchor_ms")
            val observedTarget=PlaybackTiming.targetAtSample(target,anchor,playing,p.sampledAtMs,checkedAt,sample.offsetMs) ?: return
            if(p.state==(if(playing) "Playing" else "Paused") && (!playing || (started && observedServer>=anchor && p.positionMs>=target+20)) && abs(p.positionMs-observedTarget)<=500) {
                sentExecution=send(JSONObject().put("type","executed").put("playing",playing).put("position_ms",p.positionMs).put("observed_server_ms",observedServer))
            }
        }
    }
    private fun lost(old: WebSocket?,reason:String="connection_lost") {
        if(socket !== old || stopped) return
        Log.w("TogetherSession","event=$reason generation=$generation inflight=${inflight!=null}")
        cancelLoading()
        socket=null;old?.cancel();state=null;inflight=null;inflightAuthority=null;wanted=null;generation=-1;loaded=false;loading=false
        rejectNextCommandForProof=false
        player.invalidateCommands();player.pause(true);player.speed(1.0);resetClock();correction.reset()
        mutable.value=Diagnostics(error="Connection lost; waiting to reconnect")
        val retryDelay=reconnect.nextDelayMs(now())
        retry?.cancel();retry=scope.launch {delay(retryDelay);open()}
    }
    private fun cancelLoading() {loadJob?.cancel();loadJob=null;loading=false}
    /** Debug-only transport failure; exercises automatic reconnect, not Activity lifecycle. */
    fun dropConnectionForProof() {check(BuildConfig.DEBUG);lost(socket)}
    /** Debug-only local send refusal, before any command reaches the authority. */
    fun rejectCommandForProof():Boolean {
        check(BuildConfig.DEBUG)
        if(inflight!=null || wanted!=null || !mutable.value.connected || state==null) return false
        rejectNextCommandForProof=true;intent("pause");return true
    }
    /** Corrupt the current native source while retaining the room's canonical fixture identity. */
    fun failSourceForProof(file:java.io.File) {check(BuildConfig.DEBUG);expectedSourceRevision=player.loadLocal(file);loaded=true}
    fun disconnect() {
        cancelLoading();reconnect.reset()
        stopped=true;retry?.cancel();val old=socket;socket=null;old?.close(1000,"Leaving session")
        state=null;wanted=null;inflight=null;inflightAuthority=null;loaded=false;generation=-1;epoch=null
        providerBinding=null;loadedIdentity=null;expectedSourceRevision=0;loadAttemptGeneration=-1L
        rejectNextCommandForProof=false
        player.invalidateCommands();player.pause(true);player.speed(1.0);resetClock();correction.reset();mutable.value=Diagnostics()
    }
    fun close() { val old=socket;disconnect();old?.cancel();scope.cancel();mediaCapabilities.close();http.closeOwned();endpoint=null;credential=null }
}
