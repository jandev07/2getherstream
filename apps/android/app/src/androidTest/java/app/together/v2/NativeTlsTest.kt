package app.together.v2

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import app.together.v2.player.NativeTls
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith
import java.security.cert.CertificateFactory
import java.util.concurrent.CountDownLatch
import java.util.concurrent.Executors
import java.util.concurrent.TimeUnit

@RunWith(AndroidJUnit4::class)
class NativeTlsTest {
    @Test fun concurrentTrustBundleInstallationPublishesCompleteFiles() {
        val context=InstrumentationRegistry.getInstrumentation().targetContext
        val executor=Executors.newFixedThreadPool(8)
        val ready=CountDownLatch(8);val start=CountDownLatch(1)
        try {
            val tasks=(1..8).map {executor.submit<Int> {
                ready.countDown();check(start.await(10,TimeUnit.SECONDS))
                val file=NativeTls.installSystemRoots(context)
                file.inputStream().use {CertificateFactory.getInstance("X.509").generateCertificates(it).size}
            }}
            assertTrue(ready.await(10,TimeUnit.SECONDS));start.countDown()
            val counts=tasks.map {it.get(30,TimeUnit.SECONDS)}
            assertTrue(counts.all {it>0});assertEquals(1,counts.toSet().size)
            assertTrue(context.filesDir.listFiles().orEmpty().none {it.name.startsWith("native-system-roots") && it.extension=="tmp"})
        } finally {start.countDown();executor.shutdownNow();assertTrue(executor.awaitTermination(10,TimeUnit.SECONDS))}
    }
}
