package app.together.v2.network

import okhttp3.*
import org.junit.Assert.*
import org.junit.Test
import java.io.IOException
import java.net.InetAddress
import java.net.ServerSocket
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicBoolean

class OwnedHttpClientTest {
    @Test fun closingOwnerCancelsAStalledRequestAndReleasesItsSocketAndExecutor() {
        ServerSocket(0,1,InetAddress.getByName("127.0.0.1")).use { server ->
            server.soTimeout=3000
            val requested=CountDownLatch(1);val connectionEnded=CountDownLatch(1)
            val failed=CountDownLatch(1);val responseArrived=AtomicBoolean(false)
            val peerClosed=AtomicBoolean(false)
            val worker=Thread {
                try {
                    server.accept().use { socket ->
                        socket.soTimeout=3000
                        val input=socket.getInputStream().bufferedReader()
                        while(true) {val line=input.readLine() ?: break;if(line.isEmpty()) break}
                        requested.countDown()
                        // No response is sent. Closing the owner must cancel the
                        // pending HTTP request rather than waiting for its timeout.
                        val result=runCatching {input.read()}
                        peerClosed.set(result.getOrNull()==-1 || result.exceptionOrNull()?.let {it is IOException && it !is java.net.SocketTimeoutException}==true)
                    }
                } finally {connectionEnded.countDown()}
            }
            val client=OkHttpClient.Builder().callTimeout(30,TimeUnit.SECONDS).build()
            val call=client.newCall(Request.Builder().url("http://127.0.0.1:${server.localPort}/stalled").build())
            worker.start()
            try {
                call.enqueue(object:Callback {
                    override fun onFailure(call:Call,e:IOException) {failed.countDown()}
                    override fun onResponse(call:Call,response:Response) {responseArrived.set(true);response.close()}
                })
                assertTrue("request must reach the owned fixture",requested.await(3,TimeUnit.SECONDS))
                client.closeOwned();client.closeOwned()
                assertTrue(call.isCanceled())
                assertTrue("cancelled call must report failure",failed.await(3,TimeUnit.SECONDS))
                assertTrue("server socket must be released",connectionEnded.await(3,TimeUnit.SECONDS))
                assertTrue("peer must close, not merely reach the fixture timeout",peerClosed.get())
                assertTrue(client.dispatcher.executorService.isShutdown)
                assertTrue(client.dispatcher.executorService.awaitTermination(3,TimeUnit.SECONDS))
                assertFalse(responseArrived.get())
            } finally {client.closeOwned();server.close();worker.join(4000)}
            assertFalse("owned fixture worker must finish",worker.isAlive)
        }
    }
}
