package app.together.v2.sync
import org.junit.Assert.*
import org.junit.Test
class ClockEstimatorTest {
    @Test fun derivedOffsetOverflowCannotDisplaceGoodCalibration() {
        for(times in listOf(
            listOf(0.0,Double.MAX_VALUE,Double.MAX_VALUE,0.0),
            listOf(Double.MAX_VALUE,0.0,0.0,Double.MAX_VALUE)
        )) {
            val c=ClockEstimator()
            c.observe(100.0,620.0,622.0,142.0)
            assertNull(c.observe(times[0],times[1],times[2],times[3]))
            assertEquals(1,c.sampleCount)
            assertEquals(500.0,c.estimate(142.0)!!.offsetMs,0.001)
            assertEquals(40.0,c.estimate(142.0)!!.rttMs,0.001)
        }
    }
    @Test fun delayedRepliesDoNotEvictStillFreshBetterCalibrationAfterFourSeconds() {
        val c=ClockEstimator()
        c.observe(100.0,1019.0,1019.0,938.0)
        for(i in 0 until 20) {
            val t0=2000.0+i*500.0
            c.observe(t0,t0+781.0,t0+781.0,t0+1087.0)
        }
        assertEquals(500.0,c.estimate(12587.0)!!.offsetMs,0.001)
        assertEquals(838.0,c.estimate(12587.0)!!.rttMs,0.001)
        assertEquals(237.5,c.estimate(30939.0)!!.offsetMs,0.001)
        assertNull(c.estimate(50000.0))
    }
    @Test fun newerLowerRttCalibrationCanReplaceHistoryAndResetClearsIt() {
        val c=ClockEstimator()
        c.observe(100.0,1019.0,1019.0,938.0)
        c.observe(2000.0,2530.0,2530.0,2040.0)
        assertEquals(510.0,c.estimate(2040.0)!!.offsetMs,0.001)
        c.reset();assertNull(c.estimate(2040.0))
    }
    @Test fun delayedAndMalformedClockSamplesDoNotEraseGoodCalibration() {
        val c=ClockEstimator()
        c.observe(100.0,620.0,622.0,142.0)
        assertNull(c.observe(200.0,790.0,792.0,2202.0))
        assertNull(c.observe(Double.NaN,0.0,0.0,0.0))
        assertNull(c.observe(100.0,620.0,600.0,142.0))
        assertEquals(500.0,c.estimate(2202.0)!!.offsetMs,0.001)
        assertNull(c.estimate(31000.0))
    }
    @Test fun lowRttSampleAndExpiry() {
        val c = ClockEstimator()
        c.observe(100.0, 620.0, 622.0, 142.0)
        c.observe(200.0, 790.0, 792.0, 322.0)
        assertEquals(500.0, c.estimate(322.0)!!.offsetMs, 0.001)
        assertNull(c.estimate(31000.0))
        c.reset(); assertNull(c.estimate(400.0))
    }
    @Test fun boundedDriftCorrection() {
        val d = DriftController()
        assertEquals(Correction.Rate(0.995), d.update(1000,1400,0,1,true,true))
        assertEquals(Correction.Hold, d.update(1000,1400,1000,1,true,true))
        assertEquals(Correction.Seek(1000), d.update(1000,6000,2000,1,true,true))
        assertEquals(Correction.Rate(1.0), d.update(5000,1000,3000,1,true,false))
        assertEquals(Correction.Rate(1.0), d.update(5000,1000,4000,1,false,true))
    }
    @Test fun syncplayLadderAndGenerationReset() {
        val d = DriftController()
        assertEquals(Correction.Rate(0.95), d.update(10000,12000,0,1,true,true))
        assertEquals(Correction.Hold, d.update(10000,11000,1000,1,true,true))
        assertEquals(Correction.Rate(1.0), d.update(10000,10050,2000,1,true,true))
        d.reset()
        assertEquals(Correction.Rate(1.005), d.update(10000,4000,0,1,true,true))
        assertEquals(Correction.Hold, d.update(10000,4000,3000,1,true,true))
        assertEquals(Correction.Seek(10000), d.update(10000,4000,4000,1,true,true))
        assertEquals(Correction.Rate(1.0), d.update(1000,1400,5000,2,true,true))
    }
}
