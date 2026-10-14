package app.together.v2.network

import kotlinx.coroutines.*
import okhttp3.*
import okio.Buffer
import okio.BufferedSource
import okio.ForwardingSource
import okio.Timeout
import okio.buffer
import org.junit.Assert.*
import org.junit.Test
import java.io.IOException
import java.net.InetAddress
import java.net.ServerSocket
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit

class AwaitDecodedTest {
    private class PendingCall:Call {
        lateinit var callback:Callback
        var cancelled=false
        private var executed=false
        private val request=Request.Builder().url("https://example.invalid/pair").build()
        override fun request()=request
        override fun enqueue(responseCallback:Callback) {executed=true;callback=responseCallback}
        override fun cancel() {cancelled=true}
        override fun isCanceled()=cancelled
        override fun isExecuted()=executed
        override fun timeout()=Timeout()
        override fun clone():Call=PendingCall()
        override fun execute():Response=error("Blocking request must not be used")
    }
    private class TrackedBody:ResponseBody() {
        var closed=false
        private val bytes=object:ForwardingSource(Buffer().writeUtf8("reply")) {
            override fun close() {closed=true;super.close()}
        }.buffer()
        override fun contentType():MediaType?=null
        override fun contentLength()=5L
        override fun source():BufferedSource=bytes
    }
    private fun response(call:Call,body:ResponseBody)=Response.Builder().request(call.request())
        .protocol(Protocol.HTTP_1_1).code(200).message("OK").body(body).build()

    @Test fun cancellationAbortsRequestAndClosesLateResponseWithoutDecoding()=runBlocking {
        val call=PendingCall();val body=TrackedBody();var decoded=false
        val work=async(start=CoroutineStart.UNDISPATCHED) {call.awaitDecoded {decoded=true;it.body!!.string()}}
        work.cancelAndJoin()
        assertTrue(call.cancelled)
        call.callback.onResponse(call,response(call,body))
        assertFalse(decoded);assertTrue(body.closed);assertTrue(work.isCancelled)
    }
    @Test fun successfulDecodeClosesBodyBeforeDeliveringValue()=runBlocking {
        val call=PendingCall();val body=TrackedBody()
        val work=async(start=CoroutineStart.UNDISPATCHED) {call.awaitDecoded {it.body!!.string()}}
        call.callback.onResponse(call,response(call,body))
        assertTrue(body.closed);assertEquals("reply",work.await());assertFalse(call.cancelled)
    }
    @Test fun decodeFailureClosesBodyAndPreservesFailure()=runBlocking {
        supervisorScope {
            val call=PendingCall();val body=TrackedBody();val failure=IllegalArgumentException("Malformed reply")
            val work=async(start=CoroutineStart.UNDISPATCHED) {call.awaitDecoded<String> {throw failure}}
            call.callback.onResponse(call,response(call,body))
            assertTrue(body.closed)
            try {work.await();fail("Expected decode failure")} catch(error:IllegalArgumentException) {assertEquals(failure.message,error.message)}
        }
    }
    @Test fun lateTransportFailureDoesNotReviveCancelledOwner()=runBlocking {
        val call=PendingCall()
        val work=async(start=CoroutineStart.UNDISPATCHED) {call.awaitDecoded {"unexpected"}}
        work.cancelAndJoin()
        call.callback.onFailure(call,IOException("Request cancelled"))
        assertTrue(work.isCancelled);assertTrue(call.cancelled)
    }
    @Test fun cancellationInterruptsRealStalledBodyRead()=runBlocking {
        val server=ServerSocket(0,1,InetAddress.getByName("127.0.0.1"))
        val release=CountDownLatch(1)
        val worker=Thread {
            runCatching {server.accept().use {socket ->
                socket.soTimeout=3000
                val reader=socket.getInputStream().bufferedReader()
                while(true) {val line=reader.readLine() ?: break;if(line.isEmpty()) break}
                socket.getOutputStream().apply {
                    write("HTTP/1.1 200 OK\r\nContent-Length: 10\r\nConnection: close\r\n\r\nx".toByteArray());flush()
                }
                release.await(5,TimeUnit.SECONDS)
            }}
        }.apply {isDaemon=true;start()}
        val client=OkHttpClient.Builder().readTimeout(10,TimeUnit.SECONDS).build()
        try {
            val call=client.newCall(Request.Builder().url("http://127.0.0.1:${server.localPort}/pair").build())
            val reading=CompletableDeferred<Unit>()
            val work=async(start=CoroutineStart.UNDISPATCHED) {call.awaitDecoded {
                reading.complete(Unit);it.body!!.string()
            }}
            withTimeout(3000) {reading.await()}
            withTimeout(2000) {work.cancelAndJoin()}
            assertTrue(call.isCanceled());assertTrue(work.isCancelled)
            withTimeout(2000) {while(client.dispatcher.runningCallsCount()!=0) delay(10)}
        } finally {
            release.countDown();server.close();worker.join(2000)
            client.dispatcher.cancelAll();client.dispatcher.executorService.shutdownNow();client.connectionPool.evictAll()
        }
    }
}
