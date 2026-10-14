package app.together.v2.session

import org.junit.Assert.*
import org.junit.Test

class PlaybackSnapshotFieldsTest {
    @Test fun exactIntegerMillisecondsAndBooleansRemainValid() {
        assertTrue(PlaybackSnapshotFields.timeline(0,0L,false,false))
        assertTrue(PlaybackSnapshotFields.timeline(Long.MAX_VALUE,Long.MAX_VALUE,true,true))
        assertTrue(PlaybackSnapshotFields.preparation(2,0L,15000,2))
        assertTrue(PlaybackSnapshotFields.preparation(Long.MAX_VALUE,Long.MAX_VALUE,Long.MAX_VALUE,Long.MAX_VALUE))
    }
    @Test fun invalidTimelineNumbersCannotBecomeSeekOrStartTargets() {
        for(value in listOf(null,"0",0.0,0.5,true,-1,-1L,Float.NaN,Double.POSITIVE_INFINITY)) {
            assertFalse(PlaybackSnapshotFields.timeline(value,0,false,false))
            assertFalse(PlaybackSnapshotFields.timeline(0,value,false,false))
        }
    }
    @Test fun stringAndNumericFlagsCannotBecomePlayingIntentions() {
        for(value in listOf(null,"true","false",1,0,1.0)) {
            assertFalse(PlaybackSnapshotFields.timeline(0,0,value,false))
            assertFalse(PlaybackSnapshotFields.timeline(0,0,false,value))
        }
    }
    @Test fun preparationRequiresExactCurrentGenerationAndCompleteMilliseconds() {
        assertFalse(PlaybackSnapshotFields.preparation(1,0,15000,2))
        assertFalse(PlaybackSnapshotFields.preparation(2,0,15000,-1))
        for(value in listOf(null,"2",2.0,2.5,true,-1,-1L)) {
            assertFalse(PlaybackSnapshotFields.preparation(value,0,15000,2))
            assertFalse(PlaybackSnapshotFields.preparation(2,value,15000,2))
            assertFalse(PlaybackSnapshotFields.preparation(2,0,value,2))
        }
    }
}
