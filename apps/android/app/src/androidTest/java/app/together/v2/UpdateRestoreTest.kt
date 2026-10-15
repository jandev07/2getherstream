package app.together.v2

import android.app.Activity
import android.app.Application
import android.content.Intent
import android.os.Bundle
import android.os.SystemClock
import android.view.accessibility.AccessibilityNodeInfo
import androidx.activity.compose.setContent
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.remember
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.LocalSaveableStateRegistry
import androidx.compose.runtime.saveable.SaveableStateRegistry
import androidx.test.core.app.ActivityScenario
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.filters.SdkSuppress
import app.together.v2.update.AndroidUpdater
import app.together.v2.update.UpdateControls
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
        val application=context.applicationContext as Application
        val file=File(context.cacheDir,"updates/${UUID.randomUUID()}.apk")
        file.parentFile!!.mkdirs();file.writeText("Owned invalid archive fixture")
        val updater=AndroidUpdater(context)
        val code=try {updater.installedCode()+1} finally {updater.close()}
        val seed=listOf<Any>(mutableStateOf<Any>(arrayListOf(code,"Restore fixture","https://example.org/update.apk","a".repeat(64),file.length())),mutableStateOf<Any>(file.name))
        var creations=0
        val callbacks=object:Application.ActivityLifecycleCallbacks {
            override fun onActivityCreated(activity:Activity,state:Bundle?) {}
            override fun onActivityPostCreated(activity:Activity,state:Bundle?) {
                if(activity !is MainActivity)return
                val initial=creations++==0
                activity.setContent {
                    val owner=checkNotNull(LocalSaveableStateRegistry.current)
                    val registry=remember(owner) {
                        if(!initial) owner else object:SaveableStateRegistry by owner {
                            private val values=ArrayDeque(seed)
                            override fun consumeRestored(key:String):Any? = if(values.isEmpty()) owner.consumeRestored(key) else values.removeFirst()
                        }
                    }
                    CompositionLocalProvider(LocalSaveableStateRegistry provides registry) {MaterialTheme {UpdateControls()}}
                }
            }
            override fun onActivityStarted(a:Activity) {}
            override fun onActivityResumed(a:Activity) {}
            override fun onActivityPaused(a:Activity) {}
            override fun onActivityStopped(a:Activity) {}
            override fun onActivitySaveInstanceState(a:Activity,b:Bundle) {}
            override fun onActivityDestroyed(a:Activity) {}
        }
        application.registerActivityLifecycleCallbacks(callbacks)
        try {
            ActivityScenario.launch<MainActivity>(Intent(context,MainActivity::class.java).putExtra("localProofOnly",true)).use {scenario ->
                waitFor {text("Install update")}
                if(deleteBeforeRecreation) assertTrue(file.delete())
                scenario.recreate()
                assertEquals(2,creations)
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
            }
        } finally {application.unregisterActivityLifecycleCallbacks(callbacks);file.delete()}
    }
    @Test fun actualUpdateControlsRetainsDownloadedStateAcrossActivityRecreationAndReverifies() = exercise(false)
    @Test fun actualUpdateControlsMissingRestoredFileOffersDownloadAgain() = exercise(true)
}
