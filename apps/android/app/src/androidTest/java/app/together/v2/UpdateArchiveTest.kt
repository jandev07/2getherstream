package app.together.v2

import android.content.Context
import androidx.core.content.FileProvider
import androidx.test.core.app.ApplicationProvider
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.ext.junit.runners.AndroidJUnit4
import app.together.v2.update.AndroidUpdater
import app.together.v2.update.UpdateRelease
import app.together.v2.session.Pairing
import app.together.v2.session.PairingStore
import java.io.File
import java.security.MessageDigest
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith

/** Two explicit instrumentation invocations separated by an owned adb upgrade.
 * No HTTP, installer confirmation, playback or real pairing mutation. */
@RunWith(AndroidJUnit4::class)
class UpdateArchiveTest {
    private val context=ApplicationProvider.getApplicationContext<Context>()
    private val run=checkNotNull(InstrumentationRegistry.getArguments().getString("archiveRun"))
        .also { require(it.matches(Regex("[a-f0-9-]{36}"))) }
    private val basename="archive-validation-$run"
    private val directory=File(context.cacheDir,"updates")
    private val file=File(directory,"$basename.apk")
    private val store=PairingStore(context,"$basename-key","$basename.enc")
    private fun hash(file:File):String {
        val digest=MessageDigest.getInstance("SHA-256")
        file.inputStream().use { input -> val buffer=ByteArray(65536);while(true){val n=input.read(buffer);if(n<0)break;digest.update(buffer,0,n)} }
        return digest.digest().joinToString("") {"%02x".format(it)}
    }
    private fun normalPairingIdentity():String {
        val pairing=PairingStore(context).load() ?: return "absent"
        val bytes=(pairing.baseUrl+"\u0000"+pairing.token).toByteArray()
        return try {MessageDigest.getInstance("SHA-256").digest(bytes).joinToString("") {"%02x".format(it)}} finally {bytes.fill(0)}
    }
    @Test fun higherSignedArchiveAndContentProviderAreVerifiedBeforeUpgrade() {
        val updater=AndroidUpdater(context)
        try {
            assertEquals(8L,updater.installedCode())
            val expected=checkNotNull(InstrumentationRegistry.getArguments().getString("archiveHash"))
            assertEquals(expected,hash(file))
            val release=UpdateRelease(9,"Verification target","https://example.org/2gether.apk",expected,file.length())
            updater.verifyArchive(file,release)
            val uri=FileProvider.getUriForFile(context,"${context.packageName}.updates",file)
            assertEquals("content",uri.scheme)
            context.contentResolver.openInputStream(uri).use { assertNotNull(it);assertEquals(file.length(),generateSequence { it!!.read(ByteArray(65536)).takeIf { n -> n>=0 } }.sumOf { n -> n.toLong() }) }
            assertTrue(runCatching {updater.verifyArchive(file,UpdateRelease(8,release.name,release.url,release.hash,release.bytes))}.isFailure)
            assertTrue(runCatching {updater.verifyArchive(file,UpdateRelease(9,release.name,release.url,"0".repeat(64),release.bytes))}.isFailure)
            assertTrue(runCatching {updater.verifyArchive(file,UpdateRelease(9,release.name,release.url,release.hash,file.length()+1))}.isFailure)
            val copy=File(directory,"$basename-tampered.apk")
            try {file.copyTo(copy);copy.appendBytes(byteArrayOf(1));assertTrue(runCatching {updater.verifyArchive(copy,release)}.isFailure)}finally {copy.delete()}
            File(context.cacheDir,"$basename-normal-identity").writeText(normalPairingIdentity())
            store.save(Pairing("https://validation.invalid","validation-"+run.replace("-","")))
            assertTrue(context.getSharedPreferences(basename,Context.MODE_PRIVATE).edit().putString("retained",run).commit())
        } finally {updater.close()}
    }
    @Test fun packageUpgradeRetainsIsolatedEncryptedPairingAndSettings() {
        val updater=AndroidUpdater(context)
        try {
            assertEquals(9L,updater.installedCode())
            val saved=checkNotNull(store.load())
            assertTrue(saved.baseUrl=="https://validation.invalid" && saved.token=="validation-"+run.replace("-",""))
            assertTrue(context.getSharedPreferences(basename,Context.MODE_PRIVATE).getString("retained",null)==run)
            val normal=File(context.cacheDir,"$basename-normal-identity")
            assertTrue(normal.readText()==normalPairingIdentity())
        } finally {updater.close()}
    }
    @Test fun cleanupOwnedArchiveValidation() {
        store.clear()
        context.getSharedPreferences(basename,Context.MODE_PRIVATE).edit().clear().commit()
        File(context.cacheDir,"$basename-normal-identity").delete()
        file.delete()
        File(directory,"$basename-tampered.apk").delete()
    }
}
