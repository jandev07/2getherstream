package app.together.v2.session

import org.junit.Assert.*
import org.junit.Test

class MediaGrantNumbersTest {
    @Test fun validGrantRequiresExactIntegerAuthorityAndPositiveSourceRevision() {
        assertTrue(MediaGrantNumbers.matches(1,3L,1L,3))
        assertTrue(MediaGrantNumbers.matches(1L,Long.MAX_VALUE,Long.MAX_VALUE,Long.MAX_VALUE))
        assertFalse(MediaGrantNumbers.matches(1,4,1,3))
        assertFalse(MediaGrantNumbers.matches(2,3,1,3))
        assertFalse(MediaGrantNumbers.matches(1,3,0,3))
        assertFalse(MediaGrantNumbers.matches(1,3,-1,3))
        assertFalse(MediaGrantNumbers.matches(1,-1,1,-1))
    }
    @Test fun stringsFractionsBooleansAndMissingFieldsCannotAuthorizeMedia() {
        for(value in listOf(null,"3",3.0,3.9,true,Float.NaN,Double.POSITIVE_INFINITY)) {
            assertFalse(MediaGrantNumbers.matches(1,value,1,3))
        }
        for(value in listOf(null,"1",1.0,1.9,true)) {
            assertFalse(MediaGrantNumbers.matches(value,3,1,3))
            assertFalse(MediaGrantNumbers.matches(1,3,value,3))
        }
    }
}
