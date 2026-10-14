package app.together.v2.diagnostics

import org.junit.Assert.*
import org.junit.Test

class PhoneSampleValuesTest {
    @Test fun integersPreserveBoundsWithoutNarrowingOrCoercion() {
        assertEquals(Long.MAX_VALUE,phoneInteger(Long.MAX_VALUE,0L..Long.MAX_VALUE))
        assertEquals(-604800000L,phoneInteger(-604800000,-604800000L..604800000L))
        for(value in listOf<Any?>(null,"1",true,1.0,1.5,Double.NaN,Double.POSITIVE_INFINITY,Long.MIN_VALUE,-1L)) {
            assertTrue(runCatching {phoneInteger(value,0L..Long.MAX_VALUE)}.isFailure)
        }
        for(value in listOf(-604800001L,604800001L)) {
            assertTrue(runCatching {phoneInteger(value,-604800000L..604800000L)}.isFailure)
        }
        assertEquals(60000L,phoneInteger(60000,0L..60000L))
        assertTrue(runCatching {phoneInteger(60001,0L..60000L)}.isFailure)
    }
    @Test fun booleansRejectTextAndNumericAlternatives() {
        assertTrue(phoneBoolean(true));assertFalse(phoneBoolean(false))
        for(value in listOf<Any?>(null,"true","false",0,1,0.0)) {
            assertTrue(runCatching {phoneBoolean(value)}.isFailure)
        }
    }
}
