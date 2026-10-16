package app.together.v2

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import app.together.v2.player.LibMpvAdapter
import app.together.v2.player.NativeLifecycleProof
import app.together.v2.session.SessionController
import kotlinx.coroutines.*
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Assume.assumeTrue
import org.junit.Test
import org.junit.runner.RunWith
import java.io.File
import java.util.UUID
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicInteger

/** Normal socket callbacks and native fixture, with a synthetic PC participant.
 * No pairing store, provider delivery, PC decoder or synchronization claim. */
@RunWith(AndroidJUnit4::class)
class QueuedControlSocketTest {
    @Test fun rapidPlayPauseCrossesOnlyItsOwnSuccessfulAck()=runBlocking<Unit> {
        val instrumentation=InstrumentationRegistry.getInstrumentation();val args=InstrumentationRegistry.getArguments()
        assumeTrue("Requires owned loopback Host",args.getString("controlEndpoint")!=null)
        val endpoint=checkNotNull(args.getString("controlEndpoint"));val token=checkNotNull(args.getString("controlToken"))
        val run=UUID.fromString(checkNotNull(args.getString("proofRunId"))).toString()
        val file=File(instrumentation.targetContext.filesDir,"android-queued-control-proof.json")
        val receipt=JSONObject().put("run_id",run).put("phase","setup")
        file.writeText(receipt.toString())
        val cleaned=CountDownLatch(1);val closed=AtomicInteger();var player:LibMpvAdapter?=null;var session:SessionController?=null
        fun field(owner:Any,name:String)=owner.javaClass.getDeclaredField(name).apply {isAccessible=true}
        try {
            val pair=withContext(Dispatchers.Main) {
                val native=LibMpvAdapter(instrumentation.targetContext,NativeLifecycleProof({}, {}, {closed.incrementAndGet()}, {cleaned.countDown()}))
                player=native
                val control=SessionController(instrumentation.targetContext,native);session=control
                control.connect(endpoint,token);native to control
            }
            val native=pair.first;val control=pair.second
            suspend fun settled(generation:Long=0) {
                withTimeout(25000) {
                    while(true) {
                        val ready=withContext(Dispatchers.Main) {
                            val d=control.diagnostics.value;val p=native.status.value
                            check(p.error==null) {"Native fixture failed"}
                            d.connected && d.generation>=generation && !d.preparing && !d.desiredPlaying &&
                                d.readyCount==2 && d.fault==null && d.error==null && p.state=="Paused" &&
                                control.canSeek() && field(control,"inflight").get(control)==null && field(control,"wanted").get(control)==null
                        }
                        if(ready) break
                        delay(10)
                    }
                }
            }
            settled()
            val base=withContext(Dispatchers.Main) {
                val generation=control.diagnostics.value.generation
                // Main cannot process the real socket ACK between these calls.
                control.intent("play");val sequence=field(control,"inflight").get(control) as Long
                control.intent("pause")
                val wanted=checkNotNull(field(control,"wanted").get(control))
                assertEquals(generation,field(wanted,"generation").getLong(wanted))
                assertEquals("pause",(field(wanted,"action").get(wanted) as JSONObject).getString("type"))
                receipt.put("base_generation",generation).put("play_sequence",sequence).put("queued_generation",generation).put("phase","waiting_ack")
                file.writeText(receipt.toString());generation
            }
            settled(base+2)
            withContext(Dispatchers.Main) {
                receipt.put("final_generation",control.diagnostics.value.generation).put("desired_playing",control.diagnostics.value.desiredPlaying)
                    .put("native_paused",native.status.value.state=="Paused").put("native_source_ready",native.status.value.sourceReady)
                    .put("phase","complete")
                file.writeText(receipt.toString())
            }
        } finally {
            withContext(NonCancellable+Dispatchers.Main) {session?.close();player?.close()}
            if(player!=null) {assertTrue("queued control native cleanup",cleaned.await(20,TimeUnit.SECONDS));assertEquals(1,closed.get())}
            receipt.put("native_closed",closed.get());file.writeText(receipt.toString())
        }
    }
}
