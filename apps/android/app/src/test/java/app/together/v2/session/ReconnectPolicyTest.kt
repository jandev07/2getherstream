package app.together.v2.session

import org.junit.Assert.*
import org.junit.Test

class ReconnectPolicyTest {
    @Test fun outageBackoffIsBoundedAndJittered() {
        val policy=ReconnectPolicy()
        assertEquals(listOf(1000L,2000L,4000L,8000L,16000L,30000L,30000L),
            (0..6).map {policy.nextDelayMs(it*1000L,0.0)})
        repeat(100) {assertEquals(30000L,policy.nextDelayMs(100000,1.0))}
        policy.reset();assertEquals(1250L,policy.nextDelayMs(0,1.0))
    }
    @Test fun flappingSnapshotDoesNotResetButSustainedHealthDoes() {
        val policy=ReconnectPolicy()
        assertEquals(1000L,policy.nextDelayMs(0,0.0))
        policy.validatedSnapshot(1000);assertEquals(2000L,policy.nextDelayMs(2000,0.0))
        policy.validatedSnapshot(3000);policy.validatedSnapshot(32000)
        assertEquals(1000L,policy.nextDelayMs(33000,0.0))
    }
}
