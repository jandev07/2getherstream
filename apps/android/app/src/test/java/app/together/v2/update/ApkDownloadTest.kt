package app.together.v2.update

import kotlinx.coroutines.*
import okhttp3.*
import okhttp3.HttpUrl.Companion.toHttpUrl
import okio.Buffer
import okio.BufferedSource
import okio.ForwardingSource
import okio.buffer
import org.junit.Assert.*
import org.junit.Test
import java.io.IOException
import java.net.InetAddress
import java.net.ServerSocket
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit

class ApkDownloadTest {
    private val asset="https://github.com/jandev07/2getherstream/releases/download/alpha-test/2gether.apk"
    private val cdn="https://release-assets.githubusercontent.com/example/apk?signature=synthetic-private-value"
    private class Body:ResponseBody() {
        var closed=false
        private val buffer=object:ForwardingSource(Buffer().writeUtf8("ordinary bytes")) {
            override fun close() {closed=true;super.close()}
        }.buffer()
        override fun contentType():MediaType?=null
        override fun contentLength()=14L
        override fun source():BufferedSource=buffer
    }
    private fun client(responses:List<Pair<Int,String?>>,bodies:MutableList<Body>,requests:MutableList<Request>)=
        OkHttpClient.Builder().followRedirects(false).followSslRedirects(false).addInterceptor {chain ->
            val index=requests.size;requests+=chain.request()
            val (code,location)=responses[index]
            val body=Body();bodies+=body
            Response.Builder().request(chain.request()).protocol(Protocol.HTTP_1_1).code(code).message("Synthetic")
                .body(body).apply {if(location!=null) header("Location",location)}.build()
        }.build()
    private fun close(client:OkHttpClient) {
        client.dispatcher.cancelAll();client.dispatcher.executorService.shutdownNow();client.connectionPool.evictAll()
    }

    @Test fun githubRedirectStreamsFinalBodyAndClosesEveryResponseWithoutCredentials()=runBlocking {
        val bodies=mutableListOf<Body>();val requests=mutableListOf<Request>()
        val client=client(listOf(302 to cdn,200 to null),bodies,requests)
        try {
            assertEquals("ordinary bytes",client.awaitApkDownload(asset) {reply,active ->check(active());reply.body!!.string()})
            assertEquals(2,requests.size);assertTrue(bodies.all {it.closed})
            for(request in requests) {
                assertNull(request.header("Authorization"));assertNull(request.header("Cookie"));assertNull(request.header("Referer"))
                assertEquals("identity",request.header("Accept-Encoding"))
            }
            assertEquals("synthetic-private-value",requests.last().url.queryParameter("signature"))
        } finally {close(client)}
    }
    @Test fun directExistingHostStillWorksButCannotRedirectToAnotherHost()=runBlocking {
        for(code in listOf(200,302)) {
            val bodies=mutableListOf<Body>();val requests=mutableListOf<Request>()
            val client=client(listOf(code to cdn),bodies,requests)
            try {
                if(code==200) assertEquals("ordinary bytes",client.awaitApkDownload("https://existing-release.invalid/app.apk") {r,_->r.body!!.string()})
                else try {client.awaitApkDownload("https://existing-release.invalid/app.apk") {_,_->fail("Redirect decoded")};fail("Redirect accepted")}
                    catch(error:IOException) {assertEquals("Update could not be verified. Please try again.",error.message)}
                assertEquals(1,requests.size);assertTrue(bodies.single().closed)
            } finally {close(client)}
        }
    }
    @Test fun untrustedMalformedAndDowngradedTargetsFailBeforeSecondRequest()=runBlocking {
        for(location in listOf("http://release-assets.githubusercontent.com/app", "https://evil.invalid/apk",
            "https://release-assets.githubusercontent.com.evil.invalid/apk", "https://user:secret@release-assets.githubusercontent.com/apk",
            "https://release-assets.githubusercontent.com:444/apk", "https://release-assets.githubusercontent.com/apk#fragment",
            "https://github.com/login", "https://github.com/a/b/releases/download/tag/apk?token=private")) {
            val bodies=mutableListOf<Body>();val requests=mutableListOf<Request>();val client=client(listOf(302 to location),bodies,requests)
            try {
                try {client.awaitApkDownload(asset) {_,_->fail("Untrusted response decoded")};fail("Target admitted")}
                catch(error:IOException) {assertFalse(error.stackTraceToString().contains("signature="));assertFalse(error.stackTraceToString().contains("evil.invalid"))}
                assertEquals(1,requests.size);assertTrue(bodies.single().closed)
            } finally {close(client)}
        }
    }
    @Test fun loopsAndTooManyRedirectsAreBoundedAndClosed()=runBlocking {
        for(locations in listOf(listOf(asset),List(4) {"https://objects.githubusercontent.com/$it?signature=synthetic-private-value"})) {
            val bodies=mutableListOf<Body>();val requests=mutableListOf<Request>();val client=client(locations.map {302 to it},bodies,requests)
            try {
                try {client.awaitApkDownload(asset) {_,_->fail("Loop decoded")};fail("Unbounded redirect")}
                catch(error:IOException) {assertFalse(error.stackTraceToString().contains("synthetic-private-value"))}
                assertEquals(locations.size,requests.size);assertTrue(bodies.all {it.closed})
            } finally {close(client)}
        }
    }
    @Test fun onlyCanonicalGithubAssetCanEnterRedirectChain() {
        assertEquals("objects.githubusercontent.com",apkRedirect(asset.toHttpUrl(),"https://objects.githubusercontent.com/apk").host)
        for(source in listOf("https://github.com/login","https://other.invalid/app.apk","http://github.com/a/b/releases/download/tag/apk")) {
            try {apkRedirect(source.toHttpUrl(),cdn);fail("Wrong initial source admitted")} catch(_:IllegalArgumentException) {}
        }
        // OkHttp rejects NUL when constructing a response header, before the
        // decoder can own a body. Exercise this policy input directly instead.
        try {apkRedirect(asset.toHttpUrl(),"\u0000");fail("Control character admitted")} catch(_:IllegalArgumentException) {}
    }
    @Test fun realLoopbackRedirectThenStalledBodyHasOneBudgetAndClosesTransport()=runBlocking {
        val server=ServerSocket(0,2,InetAddress.getByName("127.0.0.1"));val bodyStarted=CountDownLatch(1);val release=CountDownLatch(1)
        val worker=Thread {
            runCatching {
                repeat(2) {index ->server.accept().use {socket ->
                    socket.soTimeout=3000
                    val reader=socket.getInputStream().bufferedReader()
                    while(true) {val line=reader.readLine() ?: break;if(line.isEmpty()) break}
                    val response=if(index==0) "HTTP/1.1 302 Found\r\nLocation: $cdn\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                        else "HTTP/1.1 200 OK\r\nContent-Length: 10\r\nConnection: close\r\n\r\nx"
                    socket.getOutputStream().apply {write(response.toByteArray());flush()}
                    if(index==1) {bodyStarted.countDown();release.await(5,TimeUnit.SECONDS)}
                }}
            }
        }.apply {isDaemon=true;start()}
        // Test-only mapping: production redirect policy sees HTTPS GitHub/CDN
        // addresses; actual transport goes to an isolated ordinary HTTP listener.
        val requests=mutableListOf<Request>()
        val client=OkHttpClient.Builder().followRedirects(false).followSslRedirects(false)
            .callTimeout(15,TimeUnit.MINUTES).addInterceptor {chain ->
                requests+=chain.request()
                chain.proceed(chain.request().newBuilder().url("http://127.0.0.1:${server.localPort}/bytes").build())
            }.build()
        try {
            try {withTimeout(5000) {client.awaitApkDownload(asset,timeoutMillis=500) {r,_->r.body!!.string()}};fail("Stalled download completed")}
            catch(error:IOException) {assertFalse(error.stackTraceToString().contains("synthetic-private-value"))}
            assertEquals(0L,bodyStarted.count);assertEquals(2,requests.size)
            withTimeout(2000) {while(client.dispatcher.runningCallsCount()!=0) delay(10)}
            assertEquals(TimeUnit.MINUTES.toMillis(15).toInt(),client.callTimeoutMillis)
        } finally {release.countDown();server.close();worker.join(2000);close(client)}
    }
    @Test fun waiterCancellationDuringRedirectClosesReplyWithoutFollowingIt()=runBlocking {
        val entered=CountDownLatch(1);val release=CountDownLatch(1);val bodies=mutableListOf<Body>();var calls=0
        val client=OkHttpClient.Builder().followRedirects(false).followSslRedirects(false).addInterceptor {chain ->
            calls++;entered.countDown();release.await(3,TimeUnit.SECONDS)
            val body=Body();bodies+=body
            Response.Builder().request(chain.request()).protocol(Protocol.HTTP_1_1).code(302).message("Synthetic")
                .header("Location",cdn).body(body).build()
        }.build()
        try {
            val work=async(start=CoroutineStart.UNDISPATCHED) {client.awaitApkDownload(asset) {_,_->7}}
            assertTrue(withContext(Dispatchers.IO) {entered.await(2,TimeUnit.SECONDS)})
            work.cancelAndJoin();release.countDown()
            withTimeout(2000) {while(client.dispatcher.runningCallsCount()!=0) delay(10)}
            assertTrue(work.isCancelled);assertEquals(1,calls);assertTrue(bodies.single().closed)
        } finally {release.countDown();close(client)}
    }
}
