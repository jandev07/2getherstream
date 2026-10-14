package app.together.v2.session

import org.junit.Assert.*
import org.junit.Test

class MediaIdentityFieldsTest {
    @Test fun fixtureRequiresExactZeroIds() {
        assertTrue(MediaIdentityFields.valid("fixture",0,0L))
        assertTrue(MediaIdentityFields.valid("fixture",0L,0))
        assertFalse(MediaIdentityFields.valid("fixture",1,0))
        assertFalse(MediaIdentityFields.valid("fixture",0,1))
    }
    @Test fun torboxRequiresPositiveTorrentAndNonnegativeFile() {
        assertTrue(MediaIdentityFields.valid("torbox",1,0))
        assertTrue(MediaIdentityFields.valid("torbox",Long.MAX_VALUE,Long.MAX_VALUE))
        assertFalse(MediaIdentityFields.valid("torbox",0,0))
        assertFalse(MediaIdentityFields.valid("torbox",-1,0))
        assertFalse(MediaIdentityFields.valid("torbox",1,-1))
    }
    @Test fun idsCannotBeCoercedOrTruncated() {
        for(value in listOf(null,"0","1",0.0,1.0,1.5,true,Float.NaN,Double.POSITIVE_INFINITY)) {
            assertFalse(MediaIdentityFields.valid("fixture",value,0))
            assertFalse(MediaIdentityFields.valid("fixture",0,value))
            assertFalse(MediaIdentityFields.valid("torbox",value,0))
            assertFalse(MediaIdentityFields.valid("torbox",1,value))
        }
    }
    @Test fun providerMustBeAnExactKnownString() {
        for(value in listOf(null,"","Torbox","unknown",0,true)) {
            assertFalse(MediaIdentityFields.valid(value,0,0))
            assertFalse(MediaIdentityFields.valid(value,1,0))
        }
    }
}
