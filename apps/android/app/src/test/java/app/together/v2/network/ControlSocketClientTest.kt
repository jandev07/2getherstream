package app.together.v2.network

import okhttp3.*
import org.junit.Assert.*
import org.junit.Test
import java.net.InetAddress
import java.net.ServerSocket
import java.security.MessageDigest
import java.util.Base64
import java.util.Collections
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicInteger

class ControlSocketClientTest {
    @Test fun redirectedHandshakeDoesNotLeaveValidatedSessionPath() {
        for(code in listOf(302,307)) {
            val server=ServerSocket(0,2,InetAddress.getByName("127.0.0.1"))
            val paths=Collections.synchronizedList(mutableListOf<String>())
            val bearerRequests=AtomicInteger();val failed=CountDownLatch(1);val reply=AtomicInteger()
            val worker=Thread {
                runCatching {
                    repeat(2) {index ->server.accept().use {socket ->
                        socket.soTimeout=3000
                        val input=socket.getInputStream().bufferedReader()
                        paths+=input.readLine().split(' ')[1]
                        while(true) {
                            val line=input.readLine() ?: break
                            if(line.isEmpty()) break
                            if(line.equals("Authorization: Bearer synthetic-control-credential-00000000",true)) bearerRequests.incrementAndGet()
                        }
                        val response=if(index==0) "HTTP/1.1 $code Redirect\r\nLocation: /unexpected\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                            else "HTTP/1.1 400 Unexpected\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                        socket.getOutputStream().apply {write(response.toByteArray());flush()}
                    }}
                }
            }.apply {isDaemon=true;start()}
            val client=controlSocketClient()
            val socket=client.newWebSocket(Request.Builder().url("ws://127.0.0.1:${server.localPort}/session")
                .header("Authorization","Bearer synthetic-control-credential-00000000").build(),object:WebSocketListener() {
                override fun onFailure(webSocket:WebSocket,t:Throwable,response:Response?) {
                    reply.set(response?.code ?: -1);failed.countDown()
                }
            })
            try {
                assertTrue("handshake must fail within fixture budget",failed.await(4,TimeUnit.SECONDS))
                assertEquals("redirected handshake left the validated path",listOf("/session"),paths.toList())
                assertEquals(1,bearerRequests.get());assertEquals(code,reply.get())
            } finally {socket.cancel();client.closeOwned();server.close();worker.join(4000)}
            assertFalse("owned listener must finish",worker.isAlive)
        }
    }

    @Test fun directWebSocketUpgradeStillOpensWithExistingTimingAndBearer() {
        val server=ServerSocket(0,1,InetAddress.getByName("127.0.0.1"))
        val opened=CountDownLatch(1);val release=CountDownLatch(1);val ended=CountDownLatch(1)
        val bearerRequests=AtomicInteger();val requests=AtomicInteger()
        val worker=Thread {
            try {
                server.accept().use {socket ->
                    socket.soTimeout=3000
                    val input=socket.getInputStream().bufferedReader()
                    check(input.readLine().startsWith("GET /session "));requests.incrementAndGet()
                    var key:String?=null
                    while(true) {
                        val line=input.readLine() ?: break
                        if(line.isEmpty()) break
                        if(line.startsWith("Sec-WebSocket-Key:",true)) key=line.substringAfter(':').trim()
                        if(line.equals("Authorization: Bearer synthetic-control-credential-00000000",true)) bearerRequests.incrementAndGet()
                    }
                    val accept=Base64.getEncoder().encodeToString(MessageDigest.getInstance("SHA-1")
                        .digest((checkNotNull(key)+"258EAFA5-E914-47DA-95CA-C5AB0DC85B11").toByteArray()))
                    socket.getOutputStream().apply {
                        write("HTTP/1.1 101 Switching Protocols\r\nConnection: Upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Accept: $accept\r\n\r\n".toByteArray());flush()
                    }
                    release.await(4,TimeUnit.SECONDS)
                }
            } finally {ended.countDown()}
        }.apply {isDaemon=true;start()}
        val client=controlSocketClient()
        val socket=client.newWebSocket(Request.Builder().url("ws://127.0.0.1:${server.localPort}/session")
            .header("Authorization","Bearer synthetic-control-credential-00000000").build(),object:WebSocketListener() {
            override fun onOpen(webSocket:WebSocket,response:Response) {opened.countDown()}
        })
        try {
            assertTrue("direct upgrade did not open",opened.await(4,TimeUnit.SECONDS))
            assertEquals(1,requests.get());assertEquals(1,bearerRequests.get())
            assertEquals(2000,client.pingIntervalMillis);assertEquals(10000,client.connectTimeoutMillis)
            assertFalse(client.followRedirects);assertFalse(client.followSslRedirects)
        } finally {socket.cancel();release.countDown();client.closeOwned();server.close();worker.join(4000)}
        assertEquals(0L,ended.count);assertFalse(worker.isAlive)
    }
}
