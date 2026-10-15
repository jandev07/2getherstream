package app.together.v2

import android.content.Context
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import app.together.v2.session.Pairing
import app.together.v2.session.PairingStore
import app.together.v2.session.pairingBase
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith
import java.io.File
import java.util.UUID

@RunWith(AndroidJUnit4::class)
class PairingStoreTest {
    @Test fun failedFileRemovalPreservesKeyAndReportsRetryableFailure() {
        val app=ApplicationProvider.getApplicationContext<Context>()
        val id=UUID.randomUUID().toString()
        val directory=File(app.noBackupFilesDir,"pairing-delete-$id").apply {check(mkdir())}
        val context=object:android.content.ContextWrapper(app) {
            override fun getNoBackupFilesDir():File=directory
        }
        val store=PairingStore(context,"together2-delete-$id","owned.enc")
        val file=File(directory,"owned.enc")
        val token=UUID.randomUUID().toString()+UUID.randomUUID().toString()
        try {
            store.save(Pairing("https://together.example",token))
            val original=file.readBytes()
            // Remove directory write permission only in this UUID-owned directory.
            // Its file remains readable; AtomicFile.delete cannot unlink it.
            android.system.Os.chmod(directory.absolutePath,320) // 0500
            val failure=try {runCatching {store.clear()}.exceptionOrNull()}
                finally {android.system.Os.chmod(directory.absolutePath,448)} // 0700
            assertTrue("Blocked file removal was reported as success",failure is IllegalStateException)
            assertEquals("Stored pairing could not be removed",failure?.message)
            assertNull(failure?.cause)
            assertArrayEquals(original,file.readBytes())
            val restored=checkNotNull(store.load())
            assertTrue("Failed removal discarded the pairing key",restored.token==token)
            store.clear()
            assertNull(store.load())
            assertFalse(file.exists())
        } finally {
            android.system.Os.chmod(directory.absolutePath,448)
            store.clear()
            check(directory.delete())
        }
    }
    @Test fun separateOwnersShareAtomicFileAndKeystoreOwnership() {
        val context=ApplicationProvider.getApplicationContext<Context>()
        val id=UUID.randomUUID().toString();val name="pairing-owners-$id.enc";val alias="together2-owners-$id"
        val owners=listOf(PairingStore(context,alias,name),PairingStore(context,alias,name))
        val tokens=listOf(UUID.randomUUID().toString()+UUID.randomUUID().toString(),UUID.randomUUID().toString()+UUID.randomUUID().toString())
        val executor=java.util.concurrent.Executors.newFixedThreadPool(2)
        val start=java.util.concurrent.CountDownLatch(1)
        try {
            val tasks=owners.mapIndexed {index,owner ->executor.submit(java.util.concurrent.Callable {
                check(start.await(5,java.util.concurrent.TimeUnit.SECONDS))
                repeat(10) {
                    owner.save(Pairing("https://together.example",tokens[index]))
                    val restored=checkNotNull(owners[1-index].load())
                    assertTrue("Mixed pairing ciphertext/key",restored.token in tokens)
                    assertEquals("wss://together.example/session",restored.socketUrl)
                }
            })}
            start.countDown();tasks.forEach {it.get(30,java.util.concurrent.TimeUnit.SECONDS)}
            owners[0].clear();assertNull(owners[1].load())
        } finally {
            start.countDown();executor.shutdownNow()
            check(executor.awaitTermination(5,java.util.concurrent.TimeUnit.SECONDS))
            owners[0].clear()
        }
    }
    @Test fun keystoreRoundTripTamperDetectionAndExplicitForget() {
        val context=ApplicationProvider.getApplicationContext<Context>()
        val id=UUID.randomUUID().toString();val name="pairing-test-$id.enc"
        val store=PairingStore(context,"together2-test-$id",name)
        val file=File(context.noBackupFilesDir,name)
        val token=UUID.randomUUID().toString()+UUID.randomUUID().toString()
        try {
            assertNull(store.load());store.save(Pairing("https://together.example",token))
            val restored=checkNotNull(store.load());assertTrue("Credential did not round trip",token==restored.token);assertEquals("wss://together.example/session",restored.socketUrl)
            assertFalse("Ciphertext contains credential",String(file.readBytes()).contains(token))
            val corrupted=file.readBytes();corrupted[corrupted.lastIndex]=(corrupted.last().toInt() xor 1).toByte();file.writeBytes(corrupted)
            assertTrue("Tampered pairing accepted",runCatching {store.load()}.isFailure)
            assertArrayEquals(corrupted,file.readBytes())
            store.clear();assertNull(store.load())
        } finally {store.clear()}
    }
    @Test fun oversizedReplacementPreservesSavedPairingAndBoundedLoadRejectsInvalidFiles() {
        val context=ApplicationProvider.getApplicationContext<Context>()
        val id=UUID.randomUUID().toString();val name="pairing-bound-$id.enc"
        val store=PairingStore(context,"together2-bound-$id",name)
        val file=File(context.noBackupFilesDir,name)
        val token=UUID.randomUUID().toString()+UUID.randomUUID().toString()
        try {
            store.save(Pairing("https://together.example",token));val original=file.readBytes()
            for(invalidToken in listOf(token+"\r\n",token+" ",token+"é","x".repeat(31),"x".repeat(257))) {
                val failure=runCatching {store.save(Pairing("https://together.example",invalidToken))}.exceptionOrNull()
                assertEquals("Invalid pairing credential",failure?.message)
                assertNull(failure?.cause)
                assertArrayEquals(original,file.readBytes())
                assertTrue(checkNotNull(store.load()).token==token)
            }
            for(base in listOf("https://example.com:0","https://example.com:65536",
                "https://user:PRIVATE_MARKER@[bad")) {
                val failure=runCatching {store.save(Pairing(base,token))}.exceptionOrNull()
                assertTrue(failure is IllegalArgumentException)
                assertEquals("Enter the Together server HTTPS address",failure?.message)
                assertNull(failure?.cause)
                assertArrayEquals(original,file.readBytes())
                assertTrue(checkNotNull(store.load()).token==token)
            }
            val largeBase="https://"+("a".repeat(63)+".").repeat(80)+"example"
            assertEquals(largeBase,pairingBase(largeBase))
            val rejected=runCatching {store.save(Pairing(largeBase,token))}.exceptionOrNull()
            assertTrue(rejected is IllegalArgumentException)
            assertEquals("Pairing is too large",rejected?.message)
            assertArrayEquals(original,file.readBytes())
            assertTrue(checkNotNull(store.load()).token==token)
            for(size in listOf(0,4097)) {
                val invalid=ByteArray(size) {42};file.writeBytes(invalid)
                assertTrue(runCatching {store.load()}.isFailure)
                assertArrayEquals(invalid,file.readBytes())
            }
        } finally {store.clear()}
    }
    @Test fun pairingRejectsCredentialUrlsAndInsecureRemoteAddresses() {
        for(url in listOf("http://example.com","https://user:secret@example.com","https://example.com/?token=secret","https://example.com/#secret","https://example.com/path")) {
            assertTrue(runCatching {pairingBase(url)}.isFailure)
        }
        assertEquals("https://together.example",pairingBase("https://together.example/"))
        val malformed=runCatching {pairingBase("https://user:PRIVATE_MARKER@[bad")}.exceptionOrNull()
        assertEquals("Enter the Together server HTTPS address",malformed?.message)
        assertNull(malformed?.cause)
        assertFalse(malformed.toString().contains("PRIVATE_MARKER"))
    }
}
