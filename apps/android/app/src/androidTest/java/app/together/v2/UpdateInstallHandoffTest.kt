package app.together.v2

import android.content.Context
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.filters.SdkSuppress
import androidx.test.platform.app.InstrumentationRegistry
import app.together.v2.session.Pairing
import app.together.v2.session.PairingStore
import app.together.v2.update.AndroidUpdater
import java.io.File
import java.security.MessageDigest
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith

/** Explicit owned upgrade only. Confirmation kills the first invocation; its
 * output is a handoff, never a passed JUnit receipt. Post-upgrade checks must pass. */
@RunWith(AndroidJUnit4::class)
@SdkSuppress(minSdkVersion=29)
class UpdateInstallHandoffTest {
    private val context=ApplicationProvider.getApplicationContext<Context>()
    private val run=checkNotNull(InstrumentationRegistry.getArguments().getString("permissionRun"))
        .also {require(it.matches(Regex("[a-f0-9-]{36}")))}
    private val name="installer-validation-$run"
    private val store=PairingStore(context,"$name-key","$name.enc")
    private val normalIdentity=File(context.cacheDir,"$name-normal-identity")
    private val submitted=File(context.cacheDir,"$name-submitted")
    private fun digest(bytes:ByteArray)=MessageDigest.getInstance("SHA-256").digest(bytes).joinToString("") {"%02x".format(it)}
    private fun normalPairingIdentity():String {
        val pairing=PairingStore(context).load() ?:return "absent"
        val bytes=(pairing.baseUrl+"\u0000"+pairing.token).toByteArray()
        return try {digest(bytes)} finally {bytes.fill(0)}
    }
    private fun installedHash():String {
        val digest=MessageDigest.getInstance("SHA-256")
        File(context.applicationInfo.sourceDir).inputStream().use {input ->
            val bytes=ByteArray(65536)
            while(true) {val size=input.read(bytes);if(size<0)break;digest.update(bytes,0,size)}
        }
        return digest.digest().joinToString("") {"%02x".format(it)}
    }
    @Test fun confirmHigherVersionThroughSystemInstaller() {
        val updater=AndroidUpdater(context)
        try {assertEquals(8L,updater.installedCode())} finally {updater.close()}
        normalIdentity.writeText(normalPairingIdentity())
        store.save(Pairing("https://validation.invalid","validation-"+run.replace("-","")))
        assertTrue(context.getSharedPreferences(name,Context.MODE_PRIVATE).edit().putString("retained",run).commit())
        UpdatePermissionTest().exercise(true) {
            submitted.outputStream().use {it.write("confirmation-submitted".toByteArray());it.fd.sync()}
        }
    }
    @Test fun exactInstalledArchiveRetainsIsolatedPairingAndSettingsAfterConfirmation() {
        val updater=AndroidUpdater(context)
        try {assertEquals(9L,updater.installedCode())} finally {updater.close()}
        assertEquals("confirmation-submitted",submitted.readText())
        val expected=checkNotNull(InstrumentationRegistry.getArguments().getString("permissionHash"))
        require(expected.matches(Regex("[a-f0-9]{64}")))
        assertTrue("System installed a different archive",installedHash()==expected)
        val pairing=checkNotNull(store.load())
        assertTrue("Isolated encrypted pairing changed",pairing.baseUrl=="https://validation.invalid" && pairing.token=="validation-"+run.replace("-",""))
        assertTrue("Isolated setting changed",context.getSharedPreferences(name,Context.MODE_PRIVATE).getString("retained",null)==run)
        assertTrue("Normal pairing changed",normalIdentity.readText()==normalPairingIdentity())
    }
    @Test fun cleanupOwnedInstallerValidation() {
        store.clear()
        assertTrue(context.getSharedPreferences(name,Context.MODE_PRIVATE).edit().clear().commit())
        for(file in listOf(normalIdentity,submitted,File(context.cacheDir,"updates/$run.apk"))) {
            if(file.exists())assertTrue("Owned installer fixture cleanup failed",file.delete())
        }
    }
}
