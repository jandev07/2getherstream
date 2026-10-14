package app.together.v2.session

import app.together.v2.sync.ClockEstimator
import app.together.v2.sync.ClockRequests
import org.junit.Assert.*
import org.junit.Test

class ClockReplyNumbersTest {
    @Test fun integerClockMillisRemainExactAtSupportedBoundaries() {
        assertEquals(0.0,clockReplyMillis(0)!!,0.0)
        assertEquals(1200.0,clockReplyMillis(1200L)!!,0.0)
        assertEquals(9007199254740991.0,clockReplyMillis(9007199254740991L)!!,0.0)
    }
    @Test fun coercibleAndPrecisionLosingClockTimesAreRejected() {
        for(value in listOf(null,"1200",1200.0,1200.5,true,-1L,Long.MAX_VALUE,
            9007199254740992L,Double.NaN,Double.POSITIVE_INFINITY)) {
            assertNull(clockReplyMillis(value))
        }
    }
    @Test fun rejectedCoercionDoesNotConsumeRealOutstandingClockExchange() {
        val requests=ClockRequests();val estimator=ClockEstimator()
        assertTrue(requests.issue(100))
        val invalid=clockReplyMillis("100")
        if(invalid!=null && requests.consume(invalid,142)) estimator.observe(invalid,620.0,622.0,142.0)
        assertNull(estimator.estimate(142.0))
        val valid=clockReplyMillis(100L)!!
        assertTrue(requests.consume(valid,142))
        assertNotNull(estimator.observe(valid,620.0,622.0,142.0))
        assertEquals(500.0,estimator.estimate(142.0)!!.offsetMs,0.001)
        assertFalse(requests.consume(valid,143))
    }
}
