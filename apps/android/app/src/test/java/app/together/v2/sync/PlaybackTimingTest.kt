package app.together.v2.sync
import org.junit.Assert.*
import org.junit.Test
class PlaybackTimingTest {
    @Test fun executionTimestampUsesTheNativeObservationAndRejectsSaturation() {
        assertEquals(2500L,PlaybackTiming.serverAtSample(2000,2400,500.0))
        assertEquals(0L,PlaybackTiming.serverAtSample(1000,1000,-2000.0))
        assertNull(PlaybackTiming.serverAtSample(1000,2001,0.0))
        assertNull(PlaybackTiming.serverAtSample(2000,1999,0.0))
        assertNull(PlaybackTiming.serverAtSample(1000,1000,Double.MAX_VALUE))
        assertNull(PlaybackTiming.serverAtSample(1000,1000,Double.NaN))
    }
    @Test fun delayedControlTickDoesNotInventPlaybackDrift() {
        // Observed position 2500 at client 2000; loop sees it 400 ms later.
        assertEquals(2500L,PlaybackTiming.targetAtSample(1000,1000,true,2000,2400,500.0))
        assertEquals(2500L,PlaybackTiming.targetAtSample(1000,1000,true,2000,2000,500.0))
        assertEquals(1000L,PlaybackTiming.targetAtSample(1000,1000,false,2000,2400,500.0))
    }
    @Test fun invalidOrStaleObservationCannotDriveCorrection() {
        assertNull(PlaybackTiming.targetAtSample(0,0,true,1000,2001,0.0))
        assertNull(PlaybackTiming.targetAtSample(0,0,true,2000,1999,0.0))
        assertNull(PlaybackTiming.targetAtSample(0,0,true,0,10,0.0))
        assertNull(PlaybackTiming.targetAtSample(0,0,true,1000,1000,Double.NaN))
        assertEquals(0L,PlaybackTiming.targetAtSample(0,2000,true,1000,1000,0.0))
    }
    @Test fun oversizedClockAndTimelineCannotWrapIntoNegativeSeek() {
        assertNull(PlaybackTiming.targetAtSample(0,0,true,1000,1000,Double.MAX_VALUE))
        assertNull(PlaybackTiming.targetAtSample(0,0,true,1000,1000,Long.MAX_VALUE.toDouble()))
        assertNull(PlaybackTiming.targetAtSample(Long.MAX_VALUE,0,true,1000,1000,0.0))
        assertEquals(Long.MAX_VALUE,PlaybackTiming.targetAtSample(Long.MAX_VALUE-1000,0,true,1000,1000,0.0))
        assertEquals(1200L,PlaybackTiming.targetAtSample(1200,0,false,1000,1000,Double.MAX_VALUE))
        assertEquals(1200L,PlaybackTiming.targetAtSample(1200,0,true,1000,1000,-2000.0))
    }
}
