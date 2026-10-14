package app.together.v2.sync

import org.junit.Assert.*
import org.junit.Test

class ClockIntervalTest {
    @Test fun overlapConstrainsLowestRttPointOutsideOtherFreshExchange() {
        val clock=ClockEstimator()
        clock.observe(0.0,100.0,100.0,100.0)
        clock.observe(1000.0,1190.0,1190.0,1110.0)
        val estimate=clock.estimateInterval(1110.0)!!
        assertEquals(50.0,clock.estimate(1110.0)!!.offsetMs,0.001)
        assertEquals(90.0,estimate.sample.offsetMs,0.001)
        assertEquals(50.0,estimate.lowestRttOffsetMs,0.001)
        assertEquals(80.0,estimate.lowerOffsetMs!!,0.001)
        assertEquals(100.0,estimate.upperOffsetMs!!,0.001)
        assertEquals(100.0,estimate.sample.rttMs,0.001)
        assertEquals(100.0,estimate.sample.receivedMs,0.001)
        assertEquals(2,estimate.supportSamples)
        assertEquals(100.0,estimate.oldestReceivedMs,0.001)
    }
    @Test fun directionalJitterDoesNotReplaceConsensusWithFasterBiasedPoint() {
        val clock=ClockEstimator()
        clock.observe(0.0,1000.0,1000.0,800.0)
        clock.observe(1000.0,1800.0,1800.0,1800.0)
        assertEquals(500.0,clock.estimateInterval(1800.0)!!.sample.offsetMs,0.001)
        clock.observe(2000.0,2900.0,2900.0,2700.0)
        assertEquals(550.0,clock.estimate(2700.0)!!.offsetMs,0.001)
        assertEquals(500.0,clock.estimateInterval(2700.0)!!.sample.offsetMs,0.001)
    }
    @Test fun conflictingBoundsFallBackWithoutClaimingConsistency() {
        val clock=ClockEstimator()
        clock.observe(0.0,100.0,100.0,100.0)
        clock.observe(1000.0,1320.0,1320.0,1120.0)
        val estimate=clock.estimateInterval(1120.0)!!
        assertEquals(50.0,estimate.sample.offsetMs,0.001)
        assertNull(estimate.lowerOffsetMs);assertNull(estimate.upperOffsetMs)
        assertEquals(2,estimate.supportSamples)
    }
    @Test fun expiryFutureSamplesAndResetFenceSupport() {
        val clock=ClockEstimator()
        clock.observe(0.0,100.0,100.0,100.0)
        clock.observe(1000.0,1190.0,1190.0,1110.0)
        assertEquals(1,clock.estimateInterval(100.0)!!.supportSamples)
        val boundary=clock.estimateInterval(30100.0)!!
        assertEquals(2,boundary.supportSamples)
        assertEquals(30000.0,30100.0-boundary.oldestReceivedMs,0.001)
        val estimate=clock.estimateInterval(30101.0)!!
        assertEquals(1,estimate.supportSamples)
        assertEquals(135.0,estimate.sample.offsetMs,0.001)
        assertEquals(1110.0,estimate.oldestReceivedMs,0.001)
        assertNull(clock.estimateInterval(31111.0));assertNull(clock.estimateInterval(Double.NaN))
        clock.reset();assertNull(clock.estimateInterval(1110.0))
    }
    @Test fun stableDirectionalAsymmetryRemainsUnknownClockError() {
        val clock=ClockEstimator()
        for(i in 0 until 64) {
            val t0=i*500.0
            clock.observe(t0,t0+1100,t0+1100,t0+800)
        }
        val estimate=clock.estimateInterval(32300.0)!!
        assertEquals(700.0,estimate.sample.offsetMs,0.001)
        assertEquals(300.0,estimate.lowerOffsetMs!!,0.001)
        assertEquals(1100.0,estimate.upperOffsetMs!!,0.001)
        assertTrue(estimate.supportSamples<=64)
    }
    @Test fun generatedTransitProfilesKeepTrueOffsetInsideBounds() {
        // Independent endpoint/delay model, including server processing and
        // asymmetric/zero transit. This is conditional model evidence only.
        for(trueOffset in listOf(-5000.0,0.0,5000.0)) {
            val clock=ClockEstimator()
            var send=10000.0
            for(forward in listOf(0.0,1.0,250.0,500.0,750.0)) {
                for(backward in listOf(0.0,1.0,250.0,500.0,750.0)) {
                    for(work in listOf(0.0,3.0,50.0)) {
                        val receiveAtServer=send+forward+trueOffset
                        val sendAtServer=receiveAtServer+work
                        val receiveAtClient=send+forward+work+backward
                        assertNotNull(clock.observe(send,receiveAtServer,sendAtServer,receiveAtClient))
                        val result=clock.estimateInterval(receiveAtClient)!!
                        val lower=result.lowerOffsetMs!!
                        val upper=result.upperOffsetMs!!
                        assertTrue(lower<=trueOffset && trueOffset<=upper)
                        assertTrue(kotlin.math.abs(result.sample.offsetMs-trueOffset)<=(upper-lower)/2)
                        assertTrue(result.supportSamples in 1..64)
                        send+=2000
                    }
                }
            }
        }
    }
}
