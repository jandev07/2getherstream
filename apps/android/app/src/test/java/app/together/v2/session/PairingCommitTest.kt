package app.together.v2.session

import java.io.File
import org.junit.Assert.*
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder

/** Actual production readback with ordinary JVM files, not AtomicFile or Keystore. */
class PairingCommitTest {
    @get:Rule val temporary=TemporaryFolder()
    private val expected=ByteArray(MAX_PAIRING_BYTES) {(it%127).toByte()}
    private fun rejected(file:File) {
        val failure=runCatching {verifyPairingCommit(file,expected)}.exceptionOrNull()
        assertTrue("Unpublished pairing bytes accepted",failure is IllegalStateException)
        assertEquals("Stored pairing could not be saved",failure?.message)
        assertNull(failure?.cause)
        assertFalse(failure.toString().contains(file.absolutePath))
    }
    @Test fun exactBoundedCiphertextIsAcceptedWithoutChangingIt() {
        val file=temporary.newFile("published.enc").apply {writeBytes(expected)}
        verifyPairingCommit(file,expected)
        assertArrayEquals(expected,file.readBytes())
    }
    @Test fun stagedFileAloneIsNotPublicationAndExplicitRetryCanSucceed() {
        val file=File(temporary.root,"pending.enc")
        val staged=File(file.path+".new").apply {writeBytes(expected)}
        rejected(file)
        assertFalse(file.exists());assertArrayEquals(expected,staged.readBytes())
        assertTrue(staged.renameTo(file))
        verifyPairingCommit(file,expected)
        assertArrayEquals(expected,file.readBytes())
    }
    @Test fun blockedDirectoryAndStagedBytesRemainUntouched() {
        val file=temporary.newFolder("blocked.enc")
        val occupied=File(file,"occupied.bin").apply {writeText("owned test file")}
        val staged=File(file.path+".new").apply {writeBytes(expected)}
        rejected(file)
        assertTrue(file.isDirectory);assertEquals("owned test file",occupied.readText())
        assertArrayEquals(expected,staged.readBytes())
    }
    @Test fun staleTruncatedAndOversizedFilesAreRejectedWithoutRollback() {
        val file=temporary.newFile("existing.enc")
        for(bytes in listOf(byteArrayOf(),byteArrayOf(1,2,3),ByteArray(MAX_PAIRING_BYTES) {42},ByteArray(MAX_PAIRING_BYTES+1) {42})) {
            file.writeBytes(bytes);rejected(file)
            assertArrayEquals(bytes,file.readBytes())
        }
    }
}
