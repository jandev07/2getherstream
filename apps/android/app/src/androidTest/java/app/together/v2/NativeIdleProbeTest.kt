package app.together.v2

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import app.together.v2.player.LibMpvAdapter
import app.together.v2.player.NativeLifecycleProof
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.first
import org.json.JSONArray
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith
import java.io.File
import java.util.UUID
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicInteger

/** Pre-first-source native polling only; no claim about CPU, battery or frame timing. */
@RunWith(AndroidJUnit4::class)
class NativeIdleProbeTest {
    @Test fun firstSourceStartsSamplingWithoutPollingEmptyPlayer()=runBlocking<Unit> {
        val context=InstrumentationRegistry.getInstrumentation().targetContext
        val fixture=File(context.filesDir,"idle-probe-${UUID.randomUUID()}.mp4")
        val receiptFile=File(context.filesDir,"native-idle-probe-proof.json")
        val receipt=JSONObject().put("phase","initializing");val idle=JSONArray()
        val probes=AtomicInteger()
        val cleaned=CountDownLatch(1);val closed=AtomicInteger();var owned:LibMpvAdapter?=null
        try {
            val player=withContext(Dispatchers.Main) {
                LibMpvAdapter(context,NativeLifecycleProof({}, {}, {closed.incrementAndGet()}, {cleaned.countDown()},beforeProbe={probes.incrementAndGet();Unit})).also {owned=it}
            }
            assertTrue(withTimeout(15000) {player.status.first {it.initialized || it.error!=null}}.initialized)
            repeat(10) {
                delay(75)
                val p=player.status.value
                idle.put(JSONObject().put("sampled_at_ms",p.sampledAtMs).put("state",p.state)
                    .put("source_revision",p.sourceRevision).put("source_ready",p.sourceReady))
            }
            receipt.put("idle",idle).put("idle_probe_entries",probes.get()).put("phase","idle_observed");receiptFile.writeText(receipt.toString())
            assertEquals("Empty player must not enter the native property probe",0,probes.get())
            assertTrue("Empty native player must not publish probe samples",(0 until idle.length()).all {idle.getJSONObject(it).getLong("sampled_at_ms")==0L})
            assertEquals("Idle",player.status.value.state)
            context.assets.open("clock-test.mp4").use {input ->fixture.outputStream().use {input.copyTo(it)}}
            val revision=withContext(Dispatchers.Main) {player.loadLocal(fixture,null)}
            val loaded=withTimeout(15000) {player.status.first {it.sourceReady && it.acceptedSourceRevision==revision && it.sampledAtMs>0}}
            assertTrue("Submitted source enables native property probes",probes.get()>0)
            receipt.put("loaded_sample_ms",loaded.sampledAtMs).put("loaded_duration_ms",loaded.durationMs).put("loaded_probe_entries",probes.get())
            withContext(Dispatchers.Main) {player.pause(false)}
            val playing=withTimeout(10000) {player.status.first {it.state=="Playing" && it.positionMs>=100}}
            withContext(Dispatchers.Main) {player.pause(true)}
            val paused=withTimeout(10000) {player.status.first {it.state=="Paused" && it.sampledAtMs>playing.sampledAtMs}}
            assertNull(paused.failure);assertTrue(paused.sourceReady)
            receipt.put("playing_position_ms",playing.positionMs).put("paused_sample_ms",paused.sampledAtMs).put("phase","complete")
        } finally {
            withContext(NonCancellable+Dispatchers.Main) {owned?.close()}
            if(owned!=null) {assertTrue("idle probe native cleanup",cleaned.await(20,TimeUnit.SECONDS));assertEquals(1,closed.get())}
            receipt.put("native_closed",closed.get());receiptFile.writeText(receipt.toString())
            if(fixture.exists()) assertTrue(fixture.delete())
        }
    }
}
