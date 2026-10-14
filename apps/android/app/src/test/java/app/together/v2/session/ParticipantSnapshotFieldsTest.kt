package app.together.v2.session

import org.junit.Assert.*
import org.junit.Test

class ParticipantSnapshotFieldsTest {
    @Test fun realFlagsExactNonnegativeNumbersAndOptionalKnownFaultsAreValid() {
        assertTrue(PlaybackSnapshotFields.participant(false,true,0,null))
        for(fault in listOf("load_failed","decoder_failed","command_failed","unexpected_end"))
            assertTrue(PlaybackSnapshotFields.participant(true,false,Long.MAX_VALUE,fault))
        assertTrue(PlaybackSnapshotFields.execution(1,true,0L,0,Long.MAX_VALUE))
        // Prior-generation execution may remain metadata; it must not be counted current.
        assertTrue(PlaybackSnapshotFields.execution(0,false,0,0,0))
    }
    @Test fun coerciveFlagsAndRttAreRejected() {
        for(value in listOf(null,"false",0,1,0.0)) {
            assertFalse(PlaybackSnapshotFields.participant(value,false,0,null))
            assertFalse(PlaybackSnapshotFields.participant(false,value,0,null))
            assertFalse(PlaybackSnapshotFields.execution(0,value,0,0,0))
        }
        for(value in listOf(null,"0",0.0,0.5,true,-1,-1L))
            assertFalse(PlaybackSnapshotFields.participant(false,false,value,null))
    }
    @Test fun executionRequiresCompleteExactNonnegativeMillisecondsAndGeneration() {
        for(value in listOf(null,"1",1.0,1.5,true,-1,-1L)) {
            assertFalse(PlaybackSnapshotFields.execution(value,false,0,0,0))
            assertFalse(PlaybackSnapshotFields.execution(1,false,value,0,0))
            assertFalse(PlaybackSnapshotFields.execution(1,false,0,value,0))
            assertFalse(PlaybackSnapshotFields.execution(1,false,0,0,value))
        }
    }
    @Test fun unknownFaultCannotBecomeRecoveryAuthority() {
        for(value in listOf("",true,0,"null","DecoderFailed","unknown"))
            assertFalse(PlaybackSnapshotFields.participant(false,false,0,value))
    }
}
