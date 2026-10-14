package app.together.v2.update

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

class UpdateHttpTest {
    private class PendingCall:Call {
        lateinit var callback:Callback
        var cancelled=false
        private val callTimeout=Timeout()
        private val request=Request.Builder().url("https://example.invalid/update").build()
        override fun request()=request
        override fun enqueue(responseCallback:Callback) {callback=responseCallback}
        override fun cancel() {cancelled=true}
        override fun isCanceled()=cancelled
        override fun isExecuted()=::callback.isInitialized
        override fun timeout()=callTimeout
        override fun clone():Call=PendingCall()
        override fun execute():Response=error("Blocking request must not be used")
    }
    private class Body:ResponseBody() {
        var closed=false
        private val bytes=object:ForwardingSource(Buffer().writeUtf8("metadata")) {
            override fun close() {closed=true;super.close()}
        }.buffer()
        override fun contentType():MediaType?=null
        override fun contentLength()=8L
        override fun source():BufferedSource=bytes
    }
    private fun response(call:Call,body:Body,code:Int=200)=Response.Builder().request(call.request())
        .protocol(Protocol.HTTP_1_1).code(code).message("Synthetic reply").body(body).build()

    @Test fun metadataBudgetOverridesOnlyItsCallAndDownloadRetainsLongAllowance()=runBlocking {
        val metadata=PendingCall();val download=PendingCall()
        val allowance=TimeUnit.MINUTES.toNanos(15)
        metadata.timeout().timeout(15,TimeUnit.MINUTES);download.timeout().timeout(15,TimeUnit.MINUTES)
        val checking=async(start=CoroutineStart.UNDISPATCHED) {metadata.awaitUpdateRead(UPDATE_METADATA_TIMEOUT_MS) {_,_->7}}
        val downloading=async(start=CoroutineStart.UNDISPATCHED) {download.awaitUpdateRead {_,_->9}}
        assertEquals(TimeUnit.MILLISECONDS.toNanos(UPDATE_METADATA_TIMEOUT_MS),metadata.timeout().timeoutNanos())
        assertEquals(allowance,download.timeout().timeoutNanos())
        metadata.callback.onResponse(metadata,response(metadata,Body()));download.callback.onResponse(download,response(download,Body()))
        assertEquals(7,checking.await());assertEquals(9,downloading.await())
    }
    @Test fun metadataTimeoutAbortsActualStalledHttpBodyWithoutWaitingForDownloadBudget()=runBlocking {
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
        val client=OkHttpClient.Builder().callTimeout(15,TimeUnit.MINUTES).readTimeout(30,TimeUnit.SECONDS).build()
        val call=client.newCall(Request.Builder().url("http://127.0.0.1:${server.localPort}/updates/android").build())
        var beganReading=false
        try {
            try {
                withTimeout(5000) {call.awaitUpdateRead(timeoutMillis=500) {reply,active ->
                    check(active());beganReading=true;reply.body!!.string()
                }}
                fail("Stalled metadata returned a result")
            } catch(error:IOException) {
                assertEquals("Update could not be verified. Please try again.",error.message)
            }
            assertTrue(beganReading);assertTrue(call.isCanceled())
            withTimeout(2000) {while(client.dispatcher.runningCallsCount()!=0) delay(10)}
            assertEquals(TimeUnit.MINUTES.toMillis(15).toInt(),client.callTimeoutMillis)
        } finally {
            release.countDown();server.close();worker.join(2000)
            client.dispatcher.cancelAll();client.dispatcher.executorService.shutdownNow();client.connectionPool.evictAll()
        }
    }

    @Test fun cancelledUpdateNeverDecodesLateResponse()=runBlocking {
        val call=PendingCall();val body=Body();var decoded=false
        val work=async(start=CoroutineStart.UNDISPATCHED) {call.awaitUpdateRead {_,_->decoded=true;7}}
        work.cancelAndJoin();call.callback.onResponse(call,response(call,body))
        assertTrue(call.cancelled);assertTrue(body.closed);assertFalse(decoded);assertTrue(work.isCancelled)
    }
    @Test fun successfulStreamingReadClosesResponseAndChecksOwner()=runBlocking {
        val call=PendingCall();val body=Body()
        val work=async(start=CoroutineStart.UNDISPATCHED) {call.awaitUpdateRead {reply,active ->
            assertTrue(active());reply.body!!.string()
        }}
        call.callback.onResponse(call,response(call,body))
        assertEquals("metadata",work.await());assertTrue(body.closed)
    }
    @Test fun cancellationDuringDecodeCannotDeliverAnArchiveResult()=runBlocking {
        val call=PendingCall();val body=Body();lateinit var work:Deferred<Int>
        work=async(start=CoroutineStart.UNDISPATCHED) {call.awaitUpdateRead {_,active ->
            work.cancel();assertFalse(active());7
        }}
        call.callback.onResponse(call,response(call,body))
        work.join();assertTrue(body.closed);assertTrue(work.isCancelled);assertTrue(call.cancelled)
    }
    @Test fun responseFailuresUseFixedMessagesWithoutPrivateDetails()=runBlocking {
        supervisorScope {
            for(code in listOf(200,503)) {
                val call=PendingCall();val body=Body()
                val work=async(start=CoroutineStart.UNDISPATCHED) {call.awaitUpdateRead<Int> {_,_->
                    throw IOException("https://private.invalid/token=synthetic-private-secret")
                }}
                call.callback.onResponse(call,response(call,body,code))
                try {work.await();fail("Expected update failure")}
                catch(error:IOException) {
                    assertEquals(if(code==503) "Updates are not available yet. Try again later."
                        else "Update could not be verified. Please try again.",error.message)
                    assertFalse(error.stackTraceToString().contains("synthetic-private-secret"));assertFalse(error.stackTraceToString().contains("https://private.invalid"))
                }
                assertTrue(body.closed)
            }
        }
    }
    @Test fun transportFailureIsPrivateAndLateFailureCannotReviveCancelledUpdate()=runBlocking {
        supervisorScope {
            val call=PendingCall()
            val work=async(start=CoroutineStart.UNDISPATCHED) {call.awaitUpdateRead {_,_->7}}
            call.callback.onFailure(call,IOException("https://private.invalid/token=synthetic-private-secret"))
            try {work.await();fail("Expected transport failure")}
            catch(error:IOException) {assertEquals("Could not connect. Check your connection and try again.",error.message);assertFalse(error.stackTraceToString().contains("synthetic-private-secret"));assertFalse(error.stackTraceToString().contains("https://private.invalid"))}
            val late=PendingCall();val cancelled=async(start=CoroutineStart.UNDISPATCHED) {late.awaitUpdateRead {_,_->7}}
            cancelled.cancelAndJoin();late.callback.onFailure(late,IOException("Late response"))
            assertTrue(cancelled.isCancelled);assertTrue(late.cancelled)
        }
    }
}
