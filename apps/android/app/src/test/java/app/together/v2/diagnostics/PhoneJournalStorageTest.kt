package app.together.v2.diagnostics

import java.io.File
import java.nio.file.Files
import kotlinx.coroutines.*
import org.junit.Assert.*
import org.junit.Test

class PhoneJournalStorageTest {
    private fun cleanup(directory:File) {
        check(directory.canonicalFile.parentFile==File(System.getProperty("java.io.tmpdir")).canonicalFile)
        check(directory.name.startsWith("phone-journal-"))
        check(directory.deleteRecursively())
    }
    @Test fun recreatedOwnersShareRotationAndKeepCompleteBoundedRecords()=runBlocking {
        val directory=Files.createTempDirectory("phone-journal-owners").toFile()
        try {
            val first=PhoneJournalStorage(directory,64);val second=PhoneJournalStorage(directory,64)
            coroutineScope {listOf(first,second).mapIndexed {owner,storage ->async {
                repeat(100) {storage.appendApproved((owner*100+it).toString())}
            }}.awaitAll()}
            val files=directory.listFiles().orEmpty();assertEquals(2,files.size)
            assertTrue(files.all {it.length()<=68})
            val values=first.tail(128) {it.toIntOrNull()}
            assertFalse(values.isEmpty());assertEquals(values.size,values.distinct().size)
            assertEquals(files.sumOf {it.readLines().size},values.size)
            assertTrue(values.all {it in 0..199})
            assertEquals(values,second.tail(128) {it.toIntOrNull()})
        } finally {cleanup(directory)}
    }
    @Test fun validTailSpansRotationAndSkipsMalformedRowsWithoutLosingOlderValidSamples()=runBlocking {
        val directory=Files.createTempDirectory("phone-journal-tail").toFile()
        try {
            File(directory,"phone-previous.jsonl").writeText("1\n2\ninvalid\n")
            File(directory,"phone-current.jsonl").writeText("3\ninvalid\n4\n")
            val storage=PhoneJournalStorage(directory)
            assertEquals(listOf(2,3,4),storage.tail(3) {it.toIntOrNull()})
            assertTrue(runCatching {storage.appendApproved("line\nsecond")}.isFailure)
            assertTrue(runCatching {storage.appendApproved("é".repeat(1025))}.isFailure)
            assertTrue(runCatching {storage.tail(0) {it}}.isFailure)
        } finally {cleanup(directory)}
    }
}
