package app.together.v2

import android.content.Context
import android.content.Intent
import android.os.SystemClock
import androidx.test.core.app.ActivityScenario
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import app.together.v2.player.PlayerProofController
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith
import kotlin.math.abs
import java.io.File

/** Public pinned AndroidX test media, loaded directly by native libmpv over HTTPS. */
@RunWith(AndroidJUnit4::class)
class NativeHttpsTest {
    @Test fun newerSourceReceiptRejectsQueuedHttpsLoad() {
        val context=ApplicationProvider.getApplicationContext<Context>()
        val local=File(context.filesDir,"replacement-proof.mp4")
        context.assets.open("clock-test.mp4").use {input ->local.outputStream().use {input.copyTo(it)}}
        val launch=Intent(context,MainActivity::class.java).putExtra("localProofOnly",true)
        try {
            ActivityScenario.launch<MainActivity>(launch).use {scenario ->
                lateinit var controller:PlayerProofController
                scenario.onActivity {controller=it.controller}
                val initializedUntil=SystemClock.elapsedRealtime()+20000
                while(!controller.status.value.initialized && SystemClock.elapsedRealtime()<initializedUntil) SystemClock.sleep(40)
                assertTrue(controller.status.value.initialized)
                var receipt=0L
                scenario.onActivity {
                    val old=controller.player.loadHttps("https://raw.githubusercontent.com/androidx/media/8c6678b657ede1e7883fc164ef73ed483c7796c3/libraries/test_data/src/test/assets/media/mp4/sample.mp4")
                    controller.player.invalidateCommands()
                    receipt=controller.player.loadLocal(local)
                    assertTrue(receipt>old)
                }
                val loadedUntil=SystemClock.elapsedRealtime()+20000
                while(SystemClock.elapsedRealtime()<loadedUntil) {
                    val p=controller.status.value
                    assertNull(p.error)
                    if(p.sourceRevision==receipt && p.sourceReady && p.durationMs>=59000) break
                    SystemClock.sleep(40)
                }
                assertTrue(controller.status.value.sourceReady)
                assertEquals(receipt,controller.status.value.sourceRevision)
                assertTrue(controller.status.value.durationMs>=59000)
                // Direct adapter loads are native API tests, not a controller-owned local source.
                scenario.onActivity {controller.player.pause(false)}
                SystemClock.sleep(2000)
                assertEquals(receipt,controller.status.value.sourceRevision)
                assertTrue(controller.status.value.durationMs>=59000)
                assertTrue(controller.status.value.positionMs>500)
            }
        } finally {local.delete()}
    }
    @Test fun verifiedHttpsLoadsPlaysPausesAndSeeks() {
        val launch=Intent(ApplicationProvider.getApplicationContext<Context>(),MainActivity::class.java).putExtra("localProofOnly",true)
        ActivityScenario.launch<MainActivity>(launch).use { scenario ->
            lateinit var controller:PlayerProofController
            scenario.onActivity {controller=it.controller}
            fun await(label:String,condition:()->Boolean) {
                val until=SystemClock.elapsedRealtime()+30000
                while(SystemClock.elapsedRealtime()<until) {
                    assertNull(label,controller.status.value.error)
                    if(condition()) return
                    SystemClock.sleep(40)
                }
                fail("$label: ${controller.status.value}")
            }
            await("native initialized") {controller.status.value.initialized}
            scenario.onActivity {controller.player.loadHttps("https://raw.githubusercontent.com/androidx/media/8c6678b657ede1e7883fc164ef73ed483c7796c3/libraries/test_data/src/test/assets/media/mp4/sample.mp4")}
            await("HTTPS loaded paused") {val p=controller.status.value;p.durationMs>500 && p.state=="Paused" && !p.seeking}
            val accepted=controller.status.value.acceptedSourceRevision
            val revision=controller.status.value.sourceRevision
            scenario.onActivity {
                // Synthetic rejection inputs; no API request or account key.
                listOf("https://api.torbox.app./private-input","https://%61pi.torbox.app./private-input").forEach {url ->
                    val failure=runCatching {controller.player.loadHttps(url)}.exceptionOrNull()
                    assertTrue(failure is IllegalArgumentException)
                    assertEquals("Invalid media HTTPS URL",failure?.message)
                }
            }
            SystemClock.sleep(200)
            assertEquals(accepted,controller.status.value.acceptedSourceRevision)
            assertEquals(revision,controller.status.value.sourceRevision)
            assertTrue(controller.status.value.sourceReady)
            assertEquals("Paused",controller.status.value.state)
            scenario.onActivity {controller.player.pause(false)}
            await("HTTPS play advances") {val p=controller.status.value;p.positionMs>200 && p.state=="Playing"}
            scenario.onActivity {controller.pause()}
            await("HTTPS pause") {controller.status.value.state=="Paused"}
            val position=controller.status.value.positionMs
            SystemClock.sleep(400)
            assertTrue("HTTPS paused position moved",abs(position-controller.status.value.positionMs)<=80)
            val target=controller.status.value.durationMs/2
            scenario.onActivity {controller.player.seek(target)}
            await("HTTPS exact seek") {val p=controller.status.value;abs(p.positionMs-target)<=80 && !p.seeking && p.state=="Paused"}
        }
    }
}
