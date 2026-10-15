package app.together.v2

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import app.together.v2.player.LibMpvAdapter
import app.together.v2.player.NativeLifecycleProof
import io.github.yuroyami.libmpvkt.Mpv
import io.github.yuroyami.libmpvkt.MpvCommand
import io.github.yuroyami.libmpvkt.InternalLibmpvKtApi
import io.github.yuroyami.libmpvkt.getOrThrow
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.first
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith
import java.io.File
import java.net.URI
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicReference

/** Explicit owned loopback TLS fixture. Trust is changed only on each test
 * player, on its existing command worker; no Android trust-store mutation. */
@RunWith(AndroidJUnit4::class)
@OptIn(InternalLibmpvKtApi::class)
class NativeRedirectTest {
    @Test fun directVerifiedHttpsLoadsButNativeRedirectsFail()=runBlocking {
        val instrumentation=InstrumentationRegistry.getInstrumentation()
        val context=instrumentation.targetContext
        val arguments=InstrumentationRegistry.getArguments()
        val origin=checkNotNull(arguments.getString("nativeTlsOrigin"))
        val uri=URI(origin)
        require(uri.scheme=="https" && uri.host=="127.0.0.1" && uri.port>0 && uri.rawPath.isNullOrEmpty())
        val ca=File(context.cacheDir,"isolated-native-redirect-ca.pem")
        require(ca.isFile && ca.length() in 1..16384)
        try {
            // An untrusted direct request must fail before the fixture's CA is
            // admitted, independently of the redirect assertions below.
            for((route,trusted,ready) in listOf(
                Triple("direct",false,false), Triple("direct",true,true),
                Triple("secure",true,false), Triple("downgrade",true,false))) {
                val cleaned=CountDownLatch(1)
                var player:LibMpvAdapter?=null
                try {
                    withContext(Dispatchers.Main) {
                        player=LibMpvAdapter(context,NativeLifecycleProof({}, {}, {}, {cleaned.countDown()}))
                    }
                    val native=checkNotNull(player)
                    val initialized=withTimeout(15000) {native.status.first {it.initialized || it.error!=null}}
                    assertTrue("native initialization",initialized.initialized)
                    if(trusted) {
                        val configured=CountDownLatch(1)
                        val failure=AtomicReference<Throwable?>()
                        withContext(Dispatchers.Main) {
                            native.enqueueWorkerBarrierForProof {
                                try {
                                    val field=LibMpvAdapter::class.java.getDeclaredField("core").apply {isAccessible=true}
                                    val core=field.get(native) as Mpv
                                    core.command(MpvCommand.of("set","tls-ca-file",ca.absolutePath)).getOrThrow()
                                } catch(error:Throwable) {failure.set(error)}
                                finally {configured.countDown()}
                            }
                        }
                        assertTrue("native CA configuration timed out",configured.await(5,TimeUnit.SECONDS))
                        assertNull("native CA configuration rejected",failure.get())
                    }
                    val revision=withContext(Dispatchers.Main) {native.loadHttps("$origin/$route")}
                    val result=withTimeout(15000) {native.status.first {it.sourceRevision==revision && (it.sourceReady || it.failure!=null)}}
                    assertEquals("$route readiness (trusted=$trusted)",ready,result.sourceReady)
                    if(ready) {
                        assertNull(result.failure);assertNull(result.error)
                        assertTrue(result.durationMs>0);assertEquals("Paused",result.state)
                    } else {
                        assertEquals("decoder_failed",result.failure)
                        assertEquals("Failed",result.state)
                    }
                } finally {
                    withContext(Dispatchers.Main) {player?.close()}
                    if(player!=null) assertTrue("native cleanup",cleaned.await(10,TimeUnit.SECONDS))
                }
            }
        } finally {assertTrue("isolated CA cleanup",ca.delete())}
    }
}
