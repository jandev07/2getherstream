package app.together.v2.player

import android.content.Context
import kotlinx.coroutines.*
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import app.together.v2.session.SessionController

/** UI intentions go to the session controller when paired; local mode is the player proof. */
class PlayerProofController(private val context: Context) {
    val player = LibMpvAdapter(context)
    val status = player.status
    val session = SessionController(context,player)
    val logs=app.together.v2.diagnostics.PhoneLogs(context)
    var sessionMode by androidx.compose.runtime.mutableStateOf(false)
        private set
    private var pairingUrl: String? = null
    private var pairingToken: String? = null
    private var backgrounded = false
    private var localSourceRevision:Long?=null
    fun connectSession(url: String, token: String) {
        app.together.v2.session.sessionEndpoint(url)
        app.together.v2.session.validatePairingToken(token)
        localSource.cancel()
        localSourceRevision=null
        sessionMode=true;pairingUrl=url;pairingToken=token;backgrounded=false;session.connect(url,token)
    }
    fun forgetSession() {session.disconnect();localSourceRevision=null;sessionMode=false;pairingUrl=null;pairingToken=null;backgrounded=false}
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main.immediate)
    private suspend fun recordDiagnostic() {
        val sample=app.together.v2.diagnostics.phoneSample(status.value,session.diagnostics.value)
        try {logs.record(sample)} catch(e:CancellationException) {throw e} catch(_:Exception) { /* Disk failure never stops playback. */ }
    }
    private val journalSampler=app.together.v2.diagnostics.VisibleJournalSampler(scope,record=::recordDiagnostic)
    init {journalSampler.setActive(true)}
    private val localSource=LocalSourceLoader(scope,
        prepare={withContext(Dispatchers.IO) {ProofMedia.prepare(context)}},
        load={localSourceRevision=player.loadLocal(it)},failed={player.localPreparationFailed()})
    fun loadTest() {if(!sessionMode) localSource.request()}
    // Disconnect leaves the previous native file paused for teardown/reconnect.
    // Its duration/readiness must not grant unpaired control of session media.
    private fun localSourceReady():Boolean {
        val revision=localSourceRevision ?: return false
        val native=status.value
        return native.initialized && native.failure==null && native.sourceReady &&
            native.sourceRevision==revision && native.acceptedSourceRevision==revision
    }
    fun canPlay():Boolean=if(sessionMode) session.diagnostics.value.let {it.connected && it.mediaAvailable} else localSourceReady()
    fun play() { if(sessionMode) session.intent("play") else if(localSourceReady()) player.pause(false) }
    fun pause() { if(sessionMode) session.intent("pause") else player.pause(true) }
    fun canSeek():Boolean=if(sessionMode) session.canSeek() else localSourceReady() && status.value.durationMs>0
    fun seek(deltaMs: Long) {
        if(sessionMode) session.seekBy(deltaMs)
        else if(canSeek()) player.seek((status.value.positionMs+deltaMs).coerceIn(0,status.value.durationMs.coerceAtLeast(0)))
    }
    fun background() {
        val wasBackgrounded=backgrounded;backgrounded=true;journalSampler.setActive(false)
        if(sessionMode) session.disconnect() else player.pause(true)
        if(!wasBackgrounded) scope.launch {recordDiagnostic()}
    }
    fun foreground() {
        val resume=backgrounded;backgrounded=false
        if(sessionMode && resume) session.connect(checkNotNull(pairingUrl),checkNotNull(pairingToken))
        journalSampler.setActive(true)
    }
    fun close() { journalSampler.close();localSource.cancel();session.close();scope.cancel();logs.close(); player.close() }
}
