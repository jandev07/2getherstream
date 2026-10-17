package app.together.v2

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import app.together.v2.player.PlayerProofController
import java.io.File
import org.json.JSONObject
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import app.together.v2.session.*
import kotlinx.coroutines.*
import androidx.lifecycle.Lifecycle

class MainActivity : ComponentActivity() {
    private val setupScope=CoroutineScope(SupervisorJob()+Dispatchers.Main.immediate)
    private val pairingClient=PairingClient()
    private val pairingSetup=PairingSetup()
    private var pairingRestore:PairingRestorer<Pairing>?=null
    private var pendingPairing:Pairing?=null
    private var pairingError by mutableStateOf<String?>(null)
    private fun usePairing(pairing:Pairing) {
        pendingPairing=pairing
        if(lifecycle.currentState.isAtLeast(Lifecycle.State.STARTED)) {pendingPairing=null;controller.connectSession(pairing.socketUrl,pairing.token)}
    }
    lateinit var controller: PlayerProofController
        private set
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        controller = PlayerProofController(applicationContext)
        val store=PairingStore(applicationContext)
        var proofConfigured=false
        // Debug-only, private adb test configuration. Never bundled credentials.
        if (BuildConfig.DEBUG && !intent.getBooleanExtra("localProofOnly",false)) {
            val pairing=File(filesDir,"proof-pairing.json")
            if(pairing.exists()) {
                val config=JSONObject(pairing.readText())
                controller.connectSession(config.getString("url"),config.getString("token"))
                proofConfigured=true
            }
        }
        if(!proofConfigured && !intent.getBooleanExtra("localProofOnly",false)) {
            pairingRestore=PairingRestorer(setupScope,load={withContext(Dispatchers.IO) {store.load()}},
                deliver=::usePairing,failed={pairingError="Stored connection could not be opened. Pair this phone again."})
                .also {it.start()}
        }
        setContent {
            var server by remember {mutableStateOf("")}
            var code by remember {mutableStateOf("")}
            val pairingBusy by pairingSetup.busy.collectAsState()
            val isEmulator=BuildConfig.DEBUG && (android.os.Build.FINGERPRINT.contains("generic") || android.os.Build.MODEL.contains("sdk"))
            var forgetConfirm by remember {mutableStateOf(false)}
            val state by controller.status.collectAsState()
            val sync by controller.session.diagnostics.collectAsState()
            SideEffect {controller.player.view.keepScreenOn=state.state=="Playing" || sync.preparing}
            MaterialTheme(colorScheme = darkColorScheme(background = Color(0xFF07090E), primary = Color(0xFFBF9EFC))) {
                Surface(Modifier.fillMaxSize()) {
                    Column(Modifier.safeDrawingPadding().verticalScroll(rememberScrollState()).padding(20.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
                        Text("2gether", style = MaterialTheme.typography.headlineMedium)
                        Text("Watch together", color = MaterialTheme.colorScheme.primary)
                        AndroidView(factory = { controller.player.view }, modifier = Modifier.fillMaxWidth().aspectRatio(16f / 9f))
                        Text("${if(sync.ended) "Ended" else state.state} · ${state.positionMs / 1000}s / ${state.durationMs / 1000}s")
                        if(controller.sessionMode) Text(if(!sync.connected) "Reconnecting to your Windows app…" else if(!sync.mediaAvailable) "Connected. Choose a movie on Windows to get started." else if(sync.preparing) "Getting both players ready…" else "Connected to your shared session")
                        if (state.seeking) Text("Seeking…")
                        if (state.buffering) Text("Buffering…")
                        state.error?.let { Text(it, color = MaterialTheme.colorScheme.error) }
                        sync.error?.let { Text(it, color = MaterialTheme.colorScheme.error) }
                        sync.fault?.let {Text("Playback needs to recover. Tap Retry playback to try again.",color=MaterialTheme.colorScheme.error)}
                        pairingError?.let {Text(it,color=MaterialTheme.colorScheme.error)}
                        if(!controller.sessionMode) {
                            Text("Connect to your Windows app",style=MaterialTheme.typography.titleMedium)
                            OutlinedTextField(value=server,onValueChange={server=it},label={Text("Server address shown on Windows")},placeholder={Text("https://your-control-address")},singleLine=true,modifier=Modifier.fillMaxWidth())
                            if(isEmulator) OutlinedButton(onClick={server="http://127.0.0.1:47820"},enabled=!pairingBusy) {Text("Use this PC emulator")}
                            Text("Windows → Connection shows the address and code. A physical phone needs the HTTPS address.")
                            OutlinedTextField(value=code,onValueChange={code=it},label={Text("Code shown on Windows")},singleLine=true,modifier=Modifier.fillMaxWidth())
                            Button(enabled=!pairingBusy && server.isNotBlank() && code.isNotBlank(),onClick={setupScope.launch {pairingSetup.run {
                                pairingRestore?.cancel();pendingPairing=null
                                pairingError=null
                                try {
                                    val pairing=pairingClient.claim(server,code)
                                    withContext(Dispatchers.IO) {store.save(pairing)}
                                    code="";usePairing(pairing)
                                } catch(e:CancellationException) {throw e}
                                catch(e:Exception) {pairingError=pairingFailureMessage(e)}
                            }}}) {Text(if(pairingBusy) "Connecting…" else "Connect phone")}
                        } else {
                            OutlinedButton(enabled=!pairingBusy,onClick={forgetConfirm=true}) {Text("Connect to a different PC")}
                        }
                        Button(onClick = controller::loadTest, enabled = state.initialized && !controller.sessionMode) { Text("Load test video") }
                        Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                            Button(onClick = controller::play, enabled = state.initialized && controller.canPlay()) { Text(if(sync.fault!=null) "Retry playback" else if(sync.ended) "Replay" else "Play") }
                            Button(onClick = controller::pause, enabled = state.initialized && (if(controller.sessionMode) sync.connected && sync.mediaAvailable else state.durationMs>0)) { Text("Pause") }
                        }
                        Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                            OutlinedButton(onClick = { controller.seek(-10000) }, enabled = controller.canSeek()) { Text("−10 seconds") }
                            OutlinedButton(onClick = { controller.seek(10000) }, enabled = controller.canSeek()) { Text("+10 seconds") }
                        }
                        app.together.v2.update.UpdateControls()
                        app.together.v2.diagnostics.LogControls(controller.logs)
                        SourceNotices()
                        if(forgetConfirm) AlertDialog(onDismissRequest={forgetConfirm=false},title={Text("Change your connection?")},
                            text={Text("This disconnects your current session. You'll need a new code from Windows to connect again.")},
                            confirmButton={TextButton(enabled=!pairingBusy,onClick={forgetConfirm=false;setupScope.launch {pairingSetup.run {
                                pairingRestore?.cancel();pendingPairing=null;pairingError=null
                                try {
                                    withContext(Dispatchers.IO) {store.clear()}
                                    controller.forgetSession()
                                } catch(e:CancellationException) {throw e}
                                catch(_:Exception) {pairingError="Could not remove the saved connection. Try changing your connection again."}
                            }}}) {Text("Disconnect")}},dismissButton={TextButton(onClick={forgetConfirm=false}) {Text("Keep connection")}})
                    }
                }
            }
        }
    }
    override fun onStop() { controller.background(); super.onStop() }
    override fun onStart() { super.onStart();if(::controller.isInitialized) {pendingPairing?.let {pendingPairing=null;controller.connectSession(it.socketUrl,it.token)} ?: controller.foreground()} }
    override fun onDestroy() {pairingRestore?.close();setupScope.cancel();pairingClient.close();controller.close(); super.onDestroy() }
}
