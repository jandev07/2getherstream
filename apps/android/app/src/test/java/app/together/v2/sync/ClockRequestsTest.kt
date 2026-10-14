package app.together.v2.sync

import org.junit.Assert.*
import org.junit.Test

class ClockRequestsTest {
    @Test fun unmatchedAndDuplicateRepliesCannotReplaceGoodCalibration() {
        val requests=ClockRequests()
        val estimator=ClockEstimator()
        fun reply(t0:Double,t1:Double,t2:Double,t3:Long) {
            if(requests.consume(t0,t3)) estimator.observe(t0,t1,t2,t3.toDouble())
        }
        requests.issue(100)
        reply(100.0,620.0,622.0,142)
        // These would look like lower-RTT estimator samples, but neither is an
        // outstanding exchange; they must not displace the good calibration.
        reply(100.0,9100.0,9143.0,143)
        reply(200.0,9200.0,9242.0,242)
        assertEquals(500.0,estimator.estimate(242.0)!!.offsetMs,0.001)
        requests.issue(300);requests.reset();estimator.reset()
        reply(300.0,820.0,822.0,342)
        assertNull(estimator.estimate(342.0))
    }
    @Test fun repliesMustMatchAnIssuedRequestExactlyOnce() {
        val requests=ClockRequests()
        assertFalse(requests.consume(100.0,140))
        assertTrue(requests.issue(100))
        assertFalse(requests.issue(100))
        assertFalse(requests.consume(100.5,140))
        assertTrue(requests.consume(100.0,140))
        assertFalse(requests.consume(100.0,141))
        assertFalse(requests.consume(Double.NaN,141))
        assertFalse(requests.consume(Double.POSITIVE_INFINITY,141))
    }
    @Test fun reorderingIsAllowedButResetExpiryAndCapacityFenceOldReplies() {
        val requests=ClockRequests()
        requests.issue(100);requests.issue(200)
        assertTrue(requests.consume(200.0,250))
        assertTrue(requests.consume(100.0,260))
        requests.issue(300);requests.reset()
        assertFalse(requests.consume(300.0,350))
        requests.issue(400)
        assertFalse(requests.consume(400.0,5401))
        for (stamp in 6000L..6016L) assertTrue(requests.issue(stamp))
        assertFalse(requests.consume(6000.0,6020))
        assertTrue(requests.consume(6001.0,6020))
        assertTrue(requests.consume(6016.0,6020))
    }
    @Test fun invalidArrivalCannotCalibrateOrBeReplayed() {
        val requests=ClockRequests()
        assertFalse(requests.issue(-1))
        requests.issue(100)
        assertFalse(requests.consume(100.0,99))
        assertFalse(requests.consume(100.0,140))
        assertFalse(requests.consume(-1.0,140))
        assertFalse(requests.consume(1e30,140))
    }
}
