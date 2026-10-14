package app.together.v2

import android.content.Context
import android.content.Intent
import android.os.Bundle
import android.os.SystemClock
import android.view.accessibility.AccessibilityNodeInfo
import androidx.test.core.app.ActivityScenario
import androidx.test.core.app.ApplicationProvider
import androidx.test.platform.app.InstrumentationRegistry
import app.together.v2.player.PlayerProofController
import org.junit.Assert.*
import org.junit.Test
import java.io.File

/** Normal UI claim and normal saved restoration: no injected session token. */
class PairingScreenTest {
    private fun nodes(node:AccessibilityNodeInfo?):List<AccessibilityNodeInfo> {
        if(node==null)return emptyList()
        return listOf(node)+(0 until node.childCount).flatMap {nodes(node.getChild(it))}
    }
    private fun waitFor(check:()->Boolean) {
        val until=SystemClock.elapsedRealtime()+20000
        while(SystemClock.elapsedRealtime()<until) {if(check())return;SystemClock.sleep(100)}
        assertTrue("Normal pairing screen did not reach the expected state",check())
    }
    @Test fun enteredCodeConnectsAndNormalActivityRestoresSavedConnection() {
        val instrumentation=InstrumentationRegistry.getInstrumentation()
        val context=ApplicationProvider.getApplicationContext<Context>()
        val code=InstrumentationRegistry.getArguments().getString("pairingCode") ?: error("Pairing code required")
        val ciphertext=File(context.noBackupFilesDir,"pairing-v1.enc")
        val previous=if(ciphertext.exists())ciphertext.readBytes() else null
        assertFalse("Injected proof connection would bypass normal UI",File(context.filesDir,"proof-pairing.json").exists())
        try {
            ActivityScenario.launch<MainActivity>(Intent(context,MainActivity::class.java).putExtra("localProofOnly",true)).use {scenario ->
                lateinit var controller:PlayerProofController
                scenario.onActivity {controller=it.controller}
                waitFor {nodes(instrumentation.uiAutomation.rootInActiveWindow).count {it.isEditable}>=2}
                val fields=nodes(instrumentation.uiAutomation.rootInActiveWindow).filter {it.isEditable}
                assertTrue(fields[0].performAction(AccessibilityNodeInfo.ACTION_SET_TEXT,Bundle().apply {putCharSequence(AccessibilityNodeInfo.ACTION_ARGUMENT_SET_TEXT_CHARSEQUENCE,"http://127.0.0.1:47820")}))
                assertTrue(fields[1].performAction(AccessibilityNodeInfo.ACTION_SET_TEXT,Bundle().apply {putCharSequence(AccessibilityNodeInfo.ACTION_ARGUMENT_SET_TEXT_CHARSEQUENCE,code)}))
                // Setting text updates Compose asynchronously. Resolve a fresh,
                // enabled clickable node instead of clicking the stale disabled
                // semantics snapshot that existed before the code was entered.
                waitFor {
                    instrumentation.waitForIdleSync()
                    var button=nodes(instrumentation.uiAutomation.rootInActiveWindow).firstOrNull {it.text?.toString()=="Connect phone"}
                    while(button!=null && !button.isClickable)button=button.parent
                    button?.isEnabled==true && button.performAction(AccessibilityNodeInfo.ACTION_CLICK)
                }
                waitFor {controller.session.diagnostics.value.connected && controller.session.diagnostics.value.rttMs!=null}
                assertTrue("UI claim did not persist the encrypted connection",ciphertext.exists())
            }
            ActivityScenario.launch<MainActivity>(Intent(context,MainActivity::class.java)).use {scenario ->
                lateinit var controller:PlayerProofController
                scenario.onActivity {controller=it.controller}
                waitFor {controller.session.diagnostics.value.connected && controller.session.diagnostics.value.rttMs!=null}
                assertTrue(controller.sessionMode)
            }
        } finally {
            // Preserve any pre-existing emulator pairing; never remove its key.
            if(previous!=null)ciphertext.writeBytes(previous) else ciphertext.delete()
        }
    }
}
