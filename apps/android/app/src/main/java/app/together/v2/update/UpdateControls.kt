package app.together.v2.update

import androidx.compose.runtime.*
import androidx.compose.material3.*
import androidx.compose.foundation.layout.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import kotlinx.coroutines.*
import java.io.File

@Composable fun UpdateControls() {
    val context=LocalContext.current
    val updater=remember {AndroidUpdater(context.applicationContext)}
    val scope=rememberCoroutineScope()
    var busy by remember {mutableStateOf(false)}
    var message by remember {mutableStateOf("Check for a newer version of 2gether.")}
    var release by remember {mutableStateOf<UpdateRelease?>(null)}
    var apk by remember {mutableStateOf<File?>(null)}
    var progress by remember {mutableIntStateOf(-1)}
    DisposableEffect(updater) {onDispose {updater.close()}}
    Text("App updates",style=MaterialTheme.typography.titleMedium)
    Text(message)
    if(progress>=0 && busy) {LinearProgressIndicator(progress={progress/100f},modifier=Modifier.fillMaxWidth());Text("$progress% downloaded")}
    OutlinedButton(enabled=!busy,onClick={scope.launch {
        busy=true;progress=-1
        try {
                val found=updater.check()
                apk?.delete();apk=null;release=found
                message=if(found==null) "You're up to date." else "Version ${found.name} is available (${found.bytes/1048576} MB). Your connection settings will be kept."
        } catch(e:CancellationException) {throw e}
        catch(_:Exception) {message="Could not check for updates. Check your internet connection and try again later."}
        finally {busy=false}
    }}) {Text("Check for updates")}
    release?.let {candidate ->
        if(apk==null) Button(enabled=!busy,onClick={scope.launch {
            busy=true;progress=0;message="Downloading ${candidate.name}…"
            try {
                apk=updater.download(candidate) {value ->scope.launch {progress=value}}
                message="Update verified and ready. Android will ask you to confirm installation."
            } catch(e:CancellationException) {throw e}
            catch(_:Exception) {message="Download could not be verified. Check your connection and try Download again."}
            finally {busy=false}
        }}) {Text(if(busy) "Downloading…" else "Download update")}
        else Button(enabled=!busy,onClick={scope.launch {
            busy=true
            try {
                val file=checkNotNull(apk)
                withContext(Dispatchers.IO) {updater.verifyArchive(file,candidate)}
                if(!updater.canInstall()) message="Allow updates from 2gether on the next screen, then return here and tap Install update again."
                context.startActivity(updater.installerIntent(file))
            } catch(e:CancellationException) {throw e}
            catch(_:Exception) {apk?.delete();apk=null;message="Could not open this update. Download it again to retry."}
            finally {busy=false}
        }}) {Text("Install update")}
    }
}
