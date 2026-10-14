package app.together.v2

import android.content.Intent
import android.net.Uri
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.*
import androidx.compose.ui.platform.LocalContext

// Manual fixed public link, independent of the current PC, bearer and media.
@Composable
internal fun SourceNotices() {
    val context=LocalContext.current
    var notice by remember { mutableStateOf<String?>(null) }
    TextButton(onClick={
        try {
            context.startActivity(Intent(Intent.ACTION_VIEW,Uri.parse("https://github.com/jandev07/2getherstream/blob/main/THIRD-PARTY-NOTICES.md")))
            notice=null
        } catch(_:Exception) { notice="Could not open the source and licenses page. Visit github.com/jandev07/2getherstream." }
    }) { Text("Source code and licenses") }
    notice?.let { Text(it) }
}
