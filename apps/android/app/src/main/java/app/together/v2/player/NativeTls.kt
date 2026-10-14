package app.together.v2.player

import android.content.Context
import android.util.Base64
import java.io.File
import java.security.KeyStore
import java.security.cert.X509Certificate

/** libmpv's native TLS backend does not automatically use Android's Java trust store. */
internal object NativeTls {
    fun installSystemRoots(context: Context): File {
        val store = KeyStore.getInstance("AndroidCAStore").apply { load(null) }
        val roots = store.aliases().toList().filter { it.startsWith("system:") }
            .mapNotNull { store.getCertificate(it) as? X509Certificate }
        check(roots.isNotEmpty()) { "Android system trust store unavailable" }
        val target = File(context.filesDir, "native-system-roots.pem")
        // Startup/teardown can overlap across adapters. Each writer owns an
        // exclusive temporary file; atomic rename publishes only a complete PEM.
        val temporary = File.createTempFile("native-system-roots-", ".tmp", context.filesDir)
        try {
            temporary.bufferedWriter(Charsets.US_ASCII).use { writer ->
                for (certificate in roots) {
                    writer.appendLine("-----BEGIN CERTIFICATE-----")
                    writer.appendLine(Base64.encodeToString(certificate.encoded, Base64.NO_WRAP).chunked(64).joinToString("\n"))
                    writer.appendLine("-----END CERTIFICATE-----")
                }
            }
            check(temporary.renameTo(target)) { "Native trust bundle installation failed" }
        } finally { temporary.delete() }
        return target
    }
}
