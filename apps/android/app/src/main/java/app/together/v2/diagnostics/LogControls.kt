package app.together.v2.diagnostics
import androidx.compose.runtime.*
import androidx.compose.material3.*
import androidx.compose.ui.platform.LocalContext
import app.together.v2.session.PairingStore
import kotlinx.coroutines.*

@Composable fun LogControls(logs:PhoneLogs) {
    val context=LocalContext.current;val scope=rememberCoroutineScope()
    var busy by remember {mutableStateOf(false)}
    var message by remember {mutableStateOf("Recent playback logs stay on this phone. Send them when you need help.")}
    Text("Help with playback",style=MaterialTheme.typography.titleMedium);Text(message)
    OutlinedButton(enabled=!busy,onClick={scope.launch {
        busy=true
        try {
            val pairing=withContext(Dispatchers.IO) {PairingStore(context).load()}
            if(pairing==null) message="Connect to your Windows app first to send logs."
            else {val count=logs.send(pairing);message=if(count==0) "No playback logs yet. Try again after using the player." else "Logs sent to your PC. You can return to watching."}
        } catch(e:CancellationException) {throw e}
        catch(_:Exception) {message="Could not send all logs. Make sure your Windows app is open and connected, then try again."}
        finally {busy=false}
    }}) {Text(if(busy) "Sending logs…" else "Send logs to PC")}
}
