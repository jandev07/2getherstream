package app.together.v2.session

import org.junit.Assert.*
import org.junit.Test
import java.io.InputStream

class PairingReadTest {
    @Test fun exactBoundWithPartialReadsIsAccepted() {
        val expected=ByteArray(MAX_PAIRING_BYTES) {(it%127).toByte()}
        val source=object:InputStream() {
            var position=0
            override fun read():Int=if(position==expected.size) -1 else expected[position++].toInt() and 255
            override fun read(bytes:ByteArray,offset:Int,length:Int):Int {
                if(position==expected.size) return -1
                val count=minOf(7,length,expected.size-position)
                expected.copyInto(bytes,offset,position,position+count);position+=count;return count
            }
        }
        assertArrayEquals(expected,readPairingBytes(source))
    }
    @Test fun unboundedSourceStopsAfterOneOverflowByte() {
        val source=object:InputStream() {
            var consumed=0
            override fun read():Int {consumed++;return 42}
        }
        assertTrue(runCatching {readPairingBytes(source)}.isFailure)
        assertEquals(MAX_PAIRING_BYTES+1,source.consumed)
        assertTrue(runCatching {readPairingBytes(byteArrayOf().inputStream())}.isFailure)
    }
}
