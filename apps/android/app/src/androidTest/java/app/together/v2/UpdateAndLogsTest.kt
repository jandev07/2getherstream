package app.together.v2
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.core.app.ApplicationProvider
import android.content.Context
import app.together.v2.update.*
import app.together.v2.diagnostics.*
import app.together.v2.player.PlayerStatus
import app.together.v2.session.SessionController
import org.junit.Test
import org.junit.Assert.*
import org.junit.runner.RunWith
import org.json.JSONObject
import java.io.File
import java.security.MessageDigest

@RunWith(AndroidJUnit4::class) class UpdateAndLogsTest {
    @Test fun releasePolicyRejectsCredentialsAndInvalidMetadata() {
        for(url in listOf("http://example.org/update.apk","https://user:secret@example.org/update.apk","https://example.org/update.apk?key=secret","https://example.org/update.apk#fragment")) {
            assertTrue(runCatching {UpdateRelease(3,"Alpha","$url","a".repeat(64),100).validate()}.isFailure)
        }
        UpdateRelease(3,"Alpha","https://example.org/update.apk","a".repeat(64),100).validate()
        val wrong=JSONObject().put("protocol",1).put("package","legacy.app").put("version_code",3).put("version_name","Alpha")
            .put("apk_url","https://example.org/update.apk").put("sha256","a".repeat(64)).put("size_bytes",100)
        assertTrue(runCatching {UpdateRelease.parse(wrong)}.isFailure)
        wrong.put("package","app.together.v2")
        for((key,value) in listOf("protocol" to 1.9,"version_code" to "3","size_bytes" to 100.5,"version_name" to 123)) {
            val changed=JSONObject(wrong.toString()).put(key,value)
            assertTrue(runCatching {UpdateRelease.parse(changed)}.isFailure)
        }
    }
    @Test fun corruptedApkCannotOpenAsUpdate() {
        val context=ApplicationProvider.getApplicationContext<Context>();val updater=AndroidUpdater(context)
        val directory=File(context.cacheDir,"updates").apply {mkdirs()};val file=File(directory,"verification-test.apk")
        try {
            file.writeText("not an APK")
            val hash=MessageDigest.getInstance("SHA-256").digest(file.readBytes()).joinToString("") {"%02x".format(it)}
            val release=UpdateRelease(updater.installedCode()+1,"Next","https://example.org/update.apk",hash,file.length())
            assertTrue(runCatching {updater.verifyArchive(file,release)}.isFailure)
            file.appendText("tampered");assertTrue(runCatching {updater.verifyArchive(file,release)}.isFailure)
            val outside=File(context.cacheDir,"outside.apk").apply {writeText("x")}
            try {assertTrue(runCatching {updater.installerIntent(outside)}.isFailure)} finally {outside.delete()}
        } finally {file.delete();updater.close()}
    }
    @Test fun phoneLogsContainOnlyBoundedStructuredValues() {
        val sample=phoneSample(PlayerStatus(error="SECRET_URL_AND_TOKEN",failure="SECRET_EXCEPTION",positionMs=Long.MAX_VALUE),
            SessionController.Diagnostics(error="SECRET_URL",fault="SECRET_KEY",driftMs=Long.MIN_VALUE,rttMs=Double.NaN))
        assertFalse(sample.toString().contains("SECRET"));assertEquals(604800000L,sample.getLong("position_ms"))
        assertEquals(-604800000L,sample.getLong("drift_ms"));assertTrue(sample.isNull("rtt_ms"))
        assertEquals(sample.toString(),safePhoneSample(sample).toString())
        sample.put("raw_error","PRIVATE_EXCEPTION_URL").put("url","https://private.invalid/TOKEN")
        val sanitized=safePhoneSample(sample)
        assertFalse(sanitized.toString().contains("PRIVATE_EXCEPTION_URL"));assertFalse(sanitized.has("url"));assertFalse(sanitized.has("raw_error"))
        sample.put("playback","SECRET_URL");assertTrue(runCatching {safePhoneSample(sample)}.isFailure)
    }
    @Test fun phoneLogsRejectCoercedJsonTypesAndMissingNullableField() {
        val original=phoneSample(PlayerStatus(),SessionController.Diagnostics())
        for((key,value) in listOf("position_ms" to 1.5,"utc_ms" to "100","fault" to "true","rtt_ms" to 1.0)) {
            val changed=JSONObject(original.toString()).put(key,value)
            assertTrue(runCatching {safePhoneSample(changed)}.isFailure)
        }
        val missing=JSONObject(original.toString()).apply {remove("rtt_ms")}
        assertTrue(runCatching {safePhoneSample(missing)}.isFailure)
    }
}
