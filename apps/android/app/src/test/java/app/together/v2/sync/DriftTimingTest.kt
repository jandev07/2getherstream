package app.together.v2.sync

import org.junit.Assert.*
import org.junit.Test

class DriftTimingTest {
    @Test fun physicalResponseMatchesDifferentObservationCadences() {
        fun rates(step:Long):List<Double> {
            val d=DriftController();var rate=1.0;val result=mutableListOf<Double>()
            d.update(10000,10000,0,1,true,true)
            for(now in step..3000L step step) {
                val actual=if(now<=1000) 10400L else 10000L
                when(val correction=d.update(10000,actual,now,1,true,true)) {
                    is Correction.Rate->rate=correction.value
                    Correction.Hold->{}
                    is Correction.Seek->fail("Small drift must not seek")
                }
                if(now%100==0L) result.add(rate)
            }
            return result
        }
        val fast=rates(10);assertEquals(fast,rates(100))
        assertTrue(fast.contains(0.995));assertEquals(1.0,fast.last(),0.0)
    }
    @Test fun duplicateAndReorderedObservationsCannotChangeTheFilterOrSeek() {
        val d=DriftController()
        assertEquals(Correction.Hold,d.update(10000,10000,1000,1,true,true))
        assertEquals(Correction.Hold,d.update(10000,10200,1100,1,true,true))
        assertEquals(Correction.Hold,d.update(10000,16000,1100,1,true,true))
        assertEquals(Correction.Hold,d.update(10000,16000,1099,1,true,true))
        assertEquals(Correction.Rate(0.995),d.update(10000,10200,1200,1,true,true))
        assertEquals(Correction.Rate(1.0),d.update(10000,10200,1200,2,true,true))
        assertEquals(Correction.Seek(10000),d.update(10000,16000,1300,2,true,true))
        assertEquals(Correction.Hold,d.update(10000,16000,1300,2,true,true))
    }

    @Test fun opposingSmallErrorsConvergeWithoutSeeksOrExcessRateCommands() {
        val controllers=arrayOf(DriftController(),DriftController())
        val positions=doubleArrayOf(100130.0,99870.0);val rates=doubleArrayOf(1.0,1.0);val changes=intArrayOf(0,0)
        for(now in 0L..30000L step 100L) {
            for(i in 0..1) {
                if(now>0) positions[i]+=100.0*rates[i]
                when(val correction=controllers[i].update(100000+now,kotlin.math.round(positions[i]).toLong(),now,1,true,true)) {
                    is Correction.Rate->{assertTrue(correction.value in 0.995..1.005);rates[i]=correction.value;changes[i]++}
                    Correction.Hold->{}
                    is Correction.Seek->fail("Small opposing errors must not seek")
                }
            }
        }
        assertTrue("Opposing errors must converge",kotlin.math.abs(positions[0]-positions[1])<60.0)
        assertArrayEquals(doubleArrayOf(1.0,1.0),rates,0.0);assertTrue(changes.all {it<=3})
    }
    @Test fun boundedProbeNoiseDoesNotStartRateChatter() {
        val controller=DriftController()
        for(now in 0L..10000L step 100L) {
            val noise=if(now%200==0L) 20L else -20L
            assertEquals(Correction.Hold,controller.update(100000+now,100000+now+noise,now,1,true,true))
        }
    }

    // Synthetic reporting cadence, not Android native/frame timing evidence.
    @Test fun quantizedPositionsWithFreshReadsDoNotRequestLargeCorrections() {
        for(phase in listOf(0L,25L,50L,75L,99L)) {
            val controller=DriftController();var changes=0
            for(now in 0L..10000L step 10L) {
                val position=100000+((now+phase)/100)*100-phase
                when(val correction=controller.update(100000+now,position,now,1,true,true)) {
                    is Correction.Rate->{assertTrue(correction.value in 0.995..1.005);changes++}
                    Correction.Hold->{}
                    is Correction.Seek->fail("Coarse reporting must not request seek")
                }
            }
            assertTrue("Coarse reporting caused rate chatter",changes<=2)
        }
    }
    @Test fun frozenPositionWithFreshReadsRequestsAuthorityAfterExistingLagWindow() {
        val controller=DriftController();var firstSeek:Long?=null
        for(now in 0L..5200L step 10L) {
            val result=controller.update(100000+now,100000,now,1,true,true)
            if(result is Correction.Seek) {assertEquals(100000+now,result.positionMs);firstSeek=now;break}
        }
        assertTrue(firstSeek!=null && firstSeek in 5001L..5200L)
    }
    @Test fun pausedOrExpiredClockDoesNotInterpretRepeatedPositionAsLag() {
        for((steady,fresh) in listOf(false to true,true to false)) {
            val controller=DriftController()
            for(now in 0L..10000L step 100L) {
                assertEquals(Correction.Rate(1.0),controller.update(100000+now,100000,now,1,steady,fresh))
            }
        }
    }
}
