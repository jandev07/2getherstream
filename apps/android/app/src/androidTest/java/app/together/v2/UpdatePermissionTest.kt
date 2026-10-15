package app.together.v2

import android.os.SystemClock
import android.view.KeyEvent
import android.view.accessibility.AccessibilityNodeInfo
import androidx.compose.runtime.mutableStateOf
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.filters.SdkSuppress
import androidx.test.platform.app.InstrumentationRegistry
import app.together.v2.update.AndroidUpdater
import app.together.v2.update.UpdateRelease
import java.io.File
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith

/** Exact signed higher-version archive staged by the owned runner. Seeds only
 * saved metadata/path, then uses production verification/Settings/installer.
 * Cancels system installer: no confirmation, HTTP or physical-phone assertion. */
@RunWith(AndroidJUnit4::class)
@SdkSuppress(minSdkVersion=29)
class UpdatePermissionTest {
    private val instrumentation=InstrumentationRegistry.getInstrumentation()
    private val context=instrumentation.targetContext
    private fun nodes(node:AccessibilityNodeInfo?):List<AccessibilityNodeInfo> =
        if(node==null) emptyList() else listOf(node)+(0 until node.childCount).flatMap {nodes(node.getChild(it))}
    private fun screen()=nodes(instrumentation.uiAutomation.rootInActiveWindow)
    private fun text(value:String)=screen().any {it.text?.toString()==value}
    private fun waitFor(condition:()->Boolean) {
        val deadline=SystemClock.elapsedRealtime()+10000
        while(SystemClock.elapsedRealtime()<deadline) {instrumentation.waitForIdleSync();if(condition())return;SystemClock.sleep(50)}
        assertTrue("Update permission flow did not reach expected state",condition())
    }
    private fun click(node:AccessibilityNodeInfo) {
        var button=node
        while(!button.isClickable)button=checkNotNull(button.parent)
        assertTrue("Update flow click was rejected",button.performAction(AccessibilityNodeInfo.ACTION_CLICK))
    }
    private fun install()=click(screen().first {it.text?.toString()=="Install update"})
    @Test fun verifiedArchiveSurvivesPermissionReturnAndRecreationThenOpensInstaller() {
        val run=checkNotNull(InstrumentationRegistry.getArguments().getString("permissionRun"))
        require(run.matches(Regex("[a-f0-9-]{36}")))
        val file=File(context.cacheDir,"updates/$run.apk")
        val hash=checkNotNull(InstrumentationRegistry.getArguments().getString("permissionHash"))
        val updater=AndroidUpdater(context)
        val release=UpdateRelease(9,"Permission fixture","https://example.org/update.apk",hash,file.length())
        try {
            assertEquals(8L,updater.installedCode())
            assertFalse("Owned runner must begin with install permission denied",updater.canInstall())
            updater.verifyArchive(file,release)
        } finally {updater.close()}
        val seed=listOf<Any>(mutableStateOf<Any>(arrayListOf(release.code,release.name,release.url,release.hash,release.bytes)),mutableStateOf<Any>(file.name))
        try {
            SavedUpdateScreen(context,seed).use {screen -> screen.launch().use {scenario ->
                waitFor {text("Install update")}
                install()
                waitFor {screen().any {it.packageName?.toString()=="com.android.settings"} && screen().any {it.isCheckable && !it.isChecked}}
                click(screen().first {it.isCheckable && !it.isChecked})
                waitFor {screen().any {it.isCheckable && it.isChecked}}
                instrumentation.sendKeyDownUpSync(KeyEvent.KEYCODE_BACK)
                waitFor {text("Install update") && text("Allow updates from 2gether on the next screen, then return here and tap Install update again.")}
                assertTrue("Returning from Settings lost the archive",file.isFile)
                scenario.recreate()
                waitFor {text("Install update") && text("Downloaded update available. Tap Install update to verify it and continue.")}
                val verifier=AndroidUpdater(context)
                try {assertTrue(verifier.canInstall());verifier.verifyArchive(file,release)} finally {verifier.close()}
                install()
                waitFor {screen().any {it.packageName?.toString()?.contains("packageinstaller")==true} && screen().any {it.text?.toString()?.let {label -> label.equals("Install",true) || label.equals("Update",true)}==true}}
                // Leave no installer pending and do not confirm an upgrade in this test.
                instrumentation.sendKeyDownUpSync(KeyEvent.KEYCODE_BACK)
                waitFor {text("Install update")}
                assertTrue("Cancelled installer removed the verified archive",file.isFile)
            }}
        } finally {file.delete()}
    }
}
