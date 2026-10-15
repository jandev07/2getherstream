package app.together.v2

import android.os.SystemClock
import android.view.accessibility.AccessibilityNodeInfo
import androidx.compose.runtime.mutableStateOf
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.filters.SdkSuppress
import app.together.v2.update.AndroidUpdater
import java.io.File
import java.util.UUID
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith

/** Seeds only initial saved UI values. Production UpdateControls registers with
 * the real Activity registry; recreation uses its normal restored state.
 * No public HTTP, permission grant, installer confirmation or valid APK claim. */
@RunWith(AndroidJUnit4::class)
@SdkSuppress(minSdkVersion=29)
class UpdateRestoreTest {
    private val instrumentation=InstrumentationRegistry.getInstrumentation()
    private val context=instrumentation.targetContext
    private fun nodes(node:AccessibilityNodeInfo?):List<AccessibilityNodeInfo> =
        if(node==null) emptyList() else listOf(node)+ (0 until node.childCount).flatMap {nodes(node.getChild(it))}
    private fun text(value:String)=nodes(instrumentation.uiAutomation.rootInActiveWindow).any {it.text?.toString()==value}
    private fun waitFor(condition:()->Boolean) {
        val deadline=SystemClock.elapsedRealtime()+10000
        while(SystemClock.elapsedRealtime()<deadline) {instrumentation.waitForIdleSync();if(condition())return;SystemClock.sleep(50)}
        assertTrue("Update screen did not reach expected state",condition())
    }
    private fun exercise(deleteBeforeRecreation:Boolean) {
        val file=File(context.cacheDir,"updates/${UUID.randomUUID()}.apk")
        file.parentFile!!.mkdirs();file.writeText("Owned invalid archive fixture")
        val updater=AndroidUpdater(context)
        val code=try {updater.installedCode()+1} finally {updater.close()}
        val seed=listOf<Any>(mutableStateOf<Any>(arrayListOf(code,"Restore fixture","https://example.org/update.apk","a".repeat(64),file.length())),mutableStateOf<Any>(file.name))
        try {
            SavedUpdateScreen(context,seed).use {screen -> screen.launch().use {scenario ->
                waitFor {text("Install update")}
                if(deleteBeforeRecreation) assertTrue(file.delete())
                scenario.recreate()
                assertEquals(2,screen.creations)
                if(deleteBeforeRecreation) {
                    waitFor {text("Download update")}
                    assertFalse(text("Install update"))
                } else {
                    waitFor {text("Install update") && text("Downloaded update available. Tap Install update to verify it and continue.")}
                    var button=nodes(instrumentation.uiAutomation.rootInActiveWindow).first {it.text?.toString()=="Install update"}
                    while(!button.isClickable)button=checkNotNull(button.parent)
                    assertTrue(button.performAction(AccessibilityNodeInfo.ACTION_CLICK))
                    waitFor {text("Could not open this update. Download it again to retry.") && text("Download update")}
                    assertFalse("Rejected restored fixture must be removed",file.exists())
                }
            }}
        } finally {file.delete()}
    }
    @Test fun actualUpdateControlsRetainsDownloadedStateAcrossActivityRecreationAndReverifies() = exercise(false)
    @Test fun actualUpdateControlsMissingRestoredFileOffersDownloadAgain() = exercise(true)
}
