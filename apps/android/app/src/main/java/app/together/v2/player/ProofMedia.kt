package app.together.v2.player

import android.content.Context
import java.io.File
import java.util.UUID

/** Atomic refresh prevents an old installed fixture or cancelled copy from being reused. */
internal object ProofMedia {
    fun prepare(context:Context):File {
        val target=File(context.filesDir,"clock-test.mp4")
        val temporary=File(context.filesDir,"clock-test-${UUID.randomUUID()}.tmp")
        try {
            context.assets.open("clock-test.mp4").use {input -> temporary.outputStream().use {input.copyTo(it)}}
            check(temporary.renameTo(target)) {"Proof fixture installation failed"}
        } finally {temporary.delete()}
        return target
    }
}
