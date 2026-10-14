package app.together.v2.player

import org.junit.Assert.*
import org.junit.Test

class NativeTimingTest {
    @Test fun invalidNativeObservationsCannotBecomeReadyPositions() {
        for(value in listOf(null,Double.NaN,Double.POSITIVE_INFINITY,Double.NEGATIVE_INFINITY,-0.001,604800.001,Double.MAX_VALUE)) {
            assertNull(nativeTimeMillis(value))
        }
    }
    @Test fun validFrameTimesKeepPrecisionAndProtocolBoundary() {
        assertEquals(0L,nativeTimeMillis(0.0))
        assertEquals(41L,nativeTimeMillis(0.0419))
        assertEquals(12345L,nativeTimeMillis(12.345))
        assertEquals(604800000L,nativeTimeMillis(604800.0))
    }
}
