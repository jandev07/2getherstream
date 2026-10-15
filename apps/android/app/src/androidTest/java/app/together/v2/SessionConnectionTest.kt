package app.together.v2

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import app.together.v2.player.LibMpvAdapter
import app.together.v2.player.NativeLifecycleProof
import app.together.v2.session.SessionController
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.first
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith
import java.io.DataOutputStream
import java.net.InetAddress
import java.net.ServerSocket
import java.security.MessageDigest
import java.util.Base64
import java.util.Collections
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicInteger
import java.util.concurrent.atomic.AtomicReference

/** Actual controller/socket callbacks and idle native ownership. Synthetic
 * authority over device loopback; no media, persisted pairing or WAN proof. */
@RunWith(AndroidJUnit4::class)
class SessionConnectionTest {
    @Test fun rejectedRedirectReconnectsToSavedEndpointAndAcceptsIdleSnapshot()=runBlocking {
        val context=InstrumentationRegistry.getInstrumentation().targetContext
        val server=ServerSocket(47820,2,InetAddress.getByName("127.0.0.1"))
        val allowDirect=CountDownLatch(1);val release=CountDownLatch(1);val cleaned=CountDownLatch(1)
        val created=AtomicInteger();val closed=AtomicInteger();val bearers=AtomicInteger()
        val paths=Collections.synchronizedList(mutableListOf<String>())
        val serverFailure=AtomicReference<Throwable?>()
        val worker=Thread {
            try {
                repeat(2) {index ->server.accept().use {socket ->
                    socket.soTimeout=5000
                    val input=socket.getInputStream().bufferedReader()
                    paths+=input.readLine().split(' ')[1]
                    var key:String?=null
                    while(true) {
                        val line=input.readLine() ?: break
                        if(line.isEmpty()) break
                        if(line.startsWith("Sec-WebSocket-Key:",true)) key=line.substringAfter(':').trim()
                        if(line.equals("Authorization: Bearer synthetic-session-connection-00000000",true)) bearers.incrementAndGet()
                    }
                    val output=DataOutputStream(socket.getOutputStream())
                    if(index==0) {
                        output.writeBytes("HTTP/1.1 302 Redirect\r\nLocation: /unexpected\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                        output.flush()
                        check(allowDirect.await(5,TimeUnit.SECONDS))
                    } else {
                        val accept=Base64.getEncoder().encodeToString(MessageDigest.getInstance("SHA-1")
                            .digest((checkNotNull(key)+"258EAFA5-E914-47DA-95CA-C5AB0DC85B11").toByteArray()))
                        output.writeBytes("HTTP/1.1 101 Switching Protocols\r\nConnection: Upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Accept: $accept\r\n\r\n")
                        val payload=JSONObject().put("type","snapshot").put("state",snapshot()).toString().toByteArray()
                        check(payload.size in 126..65535)
                        output.writeByte(0x81);output.writeByte(126);output.writeShort(payload.size)
                        output.write(payload);output.flush()
                        check(release.await(5,TimeUnit.SECONDS))
                    }
                }}
            } catch(error:Throwable) {serverFailure.set(error)}
        }.apply {isDaemon=true;start()}
        var player:LibMpvAdapter?=null
        var session:SessionController?=null
        try {
            withContext(Dispatchers.Main) {
                player=LibMpvAdapter(context,NativeLifecycleProof({}, {created.incrementAndGet()},
                    {closed.incrementAndGet()}, {cleaned.countDown()}))
                session=SessionController(context,checkNotNull(player))
            }
            val native=checkNotNull(player)
            val controller=checkNotNull(session)
            val initialized=withTimeout(10000) {native.status.first {it.initialized || it.error!=null}}
            assertTrue("idle native initialization failed",initialized.initialized)
            withContext(Dispatchers.Main) {
                controller.connect("ws://127.0.0.1:47820/session","synthetic-session-connection-00000000")
            }
            val lost=withTimeout(5000) {controller.diagnostics.first {it.error=="Connection lost; waiting to reconnect"}}
            assertFalse(lost.connected);assertEquals(listOf("/session"),paths.toList())
            allowDirect.countDown()
            val recovered=withTimeout(5000) {controller.diagnostics.first {it.connected && it.revision==2L}}
            assertEquals(1L,recovered.generation);assertNull(recovered.error)
            assertFalse(recovered.mediaAvailable);assertFalse(recovered.authoritativePlaying)
            assertEquals(listOf("/session","/session"),paths.toList());assertEquals(2,bearers.get())
            withContext(Dispatchers.Main) {controller.disconnect();assertFalse(controller.diagnostics.value.connected)}
        } finally {
            withContext(Dispatchers.Main) {session?.close();player?.close()}
            allowDirect.countDown();release.countDown();server.close();worker.join(5000)
            if(player!=null) assertTrue("idle native cleanup did not finish",cleaned.await(5,TimeUnit.SECONDS))
        }
        assertFalse(worker.isAlive);assertNull(serverFailure.get())
        assertEquals(1,created.get());assertEquals(1,closed.get())
    }

    private fun snapshot()=JSONObject().put("protocol",1).put("epoch","session-connection-test")
        .put("generation",1).put("revision",2).put("media",JSONObject.NULL)
        .put("fault",JSONObject.NULL).put("prepare",JSONObject.NULL).put("desired_playing",false).put("ended",false)
        .put("timeline",JSONObject().put("position_ms",0).put("anchor_ms",0).put("playing",false))
        .put("participants",JSONObject().put("pc",participant()).put("phone",participant()))
    private fun participant()=JSONObject().put("connected",false).put("buffering",false).put("rtt_ms",0)
        .put("execution",JSONObject.NULL).put("player_fault",JSONObject.NULL)
}
