package app.together.v2

import androidx.test.ext.junit.runners.AndroidJUnit4
import app.together.v2.session.validMediaIdentity
import app.together.v2.session.sameMediaIdentity
import app.together.v2.session.MediaCapabilityClient
import kotlinx.coroutines.runBlocking
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith
import java.net.InetAddress
import java.net.ServerSocket
import java.util.concurrent.atomic.AtomicBoolean

@RunWith(AndroidJUnit4::class)
class MediaCapabilityTest {
    private val credential="phone-synthetic-pairing-00000000000000"
    private fun identity()=JSONObject().put("provider","torbox").put("torrent_id",1L).put("file_id",2L)
    private fun grant()=JSONObject().put("protocol",1).put("epoch","synthetic-epoch").put("generation",3L)
        .put("source_revision",1L).put("media",identity()).put("url","https://cdn.invalid/phone?cap=synthetic")
    private fun exchange(body:String,status:Int=200,headers:String=""):String? {
        val received=AtomicBoolean(false)
        ServerSocket(0,1,InetAddress.getByName("127.0.0.1")).use { server ->
            val worker=Thread {
                server.accept().use { socket ->
                    socket.soTimeout=5000
                    val reader=socket.getInputStream().bufferedReader()
                    val first=reader.readLine();var authenticated=false
                    while(true) {val line=reader.readLine() ?: break;if(line.isEmpty()) break
                        if(line=="Authorization: Bearer $credential") authenticated=true}
                    received.set(first.startsWith("GET /media?") && first.contains("generation=3") && authenticated)
                    val bytes=body.toByteArray(Charsets.UTF_8)
                    socket.getOutputStream().write(("HTTP/1.1 $status Test\r\nContent-Length: ${bytes.size}\r\nConnection: close\r\n$headers\r\n").toByteArray())
                    socket.getOutputStream().write(bytes);socket.getOutputStream().flush()
                }
            }
            worker.start()
            val result=runBlocking {
                val client=MediaCapabilityClient()
                try {runCatching {client.request("ws://127.0.0.1:${server.localPort}/session",credential,"synthetic-epoch",3,identity()).deliver {it}}.getOrNull()}
                finally {client.close()}
            }
            worker.join(6000);assertFalse("owned response worker must finish",worker.isAlive)
            assertTrue("control metadata request must use paired authentication",received.get())
            return result
        }
    }
    @Test fun canonicalIdentityAdapterPreservesLegacyFixtureAndExactMetadataComparison() {
        assertTrue(validMediaIdentity(JSONObject().put("torrent_id",0).put("file_id",0L)))
        assertFalse(validMediaIdentity(JSONObject().put("provider",JSONObject.NULL).put("torrent_id",0).put("file_id",0)))
        assertTrue(sameMediaIdentity(identity(),identity().put("torrent_id",1)))
        assertFalse(sameMediaIdentity(identity(),identity().put("torrent_id","1")))
        assertFalse(sameMediaIdentity(identity(),identity().put("torrent_id",1.0)))
        assertFalse(sameMediaIdentity(identity(),identity().put("filename","different")))
        assertFalse(sameMediaIdentity(identity().put("filename","first"),identity().put("filename","second")))
        for(value in listOf("1",1.0,1.5,true,JSONObject.NULL)) {
            assertFalse(validMediaIdentity(identity().put("torrent_id",value)))
        }
    }
    @Test fun authenticatedGrantIsBoundToCanonicalFileAndReturnedWithoutMediaProxy() {
        val result=exchange(grant().toString())
        assertTrue("expected only the synthetic phone capability",result?.startsWith("https://cdn.invalid/phone?")==true)
    }
    @Test fun staleEpochGenerationAndDifferentFileAreRejected() {
        for(body in listOf(grant().put("epoch","different"),grant().put("generation",4),grant().put("media",identity().put("file_id",9)))) {
            assertNull("mismatched authority must not authorize a load",exchange(body.toString()))
        }
    }
    @Test fun redirectedControlAndOversizedMetadataAreRejected() {
        assertNull(exchange("",302,"Location: https://api.torbox.app/private\r\n"))
        assertNull(exchange("x".repeat(16385)))
    }
    @Test fun coercedProtocolGenerationAndSourceRevisionCannotAuthorizeNativeLoad() {
        for(field in listOf("protocol","generation","source_revision")) {
            val expected=if(field=="generation") 3 else 1
            for(value in listOf(expected.toString(),expected+0.9,true,JSONObject.NULL)) {
                assertNull("coerced authority must be rejected",exchange(grant().put(field,value).toString()))
            }
        }
    }
    @Test fun apiOriginAndMalformedPayloadCannotReachNativePlayer() {
        assertNull(exchange(grant().put("url","https://api.torbox.app./private").toString()))
        assertNull(exchange("PRIVATE_INVALID_PAYLOAD"))
    }
    @Test fun closedClientRejectsFurtherRequestsWithFixedPrivateError()=runBlocking {
        val client=MediaCapabilityClient();client.close();client.close()
        val error=runCatching {client.request("ws://127.0.0.1:47820/session",credential,"synthetic-epoch",3,identity())}.exceptionOrNull()
        assertEquals("Media access closed",error?.message);assertNull(error?.cause)
        assertFalse(error.toString().contains(credential));assertFalse(error.toString().contains("127.0.0.1"))
    }
}
