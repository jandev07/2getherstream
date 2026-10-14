package app.together.v2.session

import org.junit.Assert.*
import org.junit.Test

class SnapshotEnvelopeFieldsTest {
    @Test fun epochRetainsExactStringWithoutInventingUuidRestrictions() {
        for(value in listOf("synthetic-epoch","", "1")) assertEquals(value,PlaybackSnapshotFields.epoch(value))
    }
    @Test fun epochCannotBeCoercedIntoAuthorityIdentity() {
        for(value in listOf(null,1,1L,1.0,true,listOf("epoch"))) assertNull(PlaybackSnapshotFields.epoch(value))
    }
    @Test fun actualEndedFlagsAndOptionalStringFaultsMatchWireTypes() {
        for(ended in listOf(false,true)) for(fault in listOf(null,"","player_failed_phone","prepare_timeout"))
            assertTrue(PlaybackSnapshotFields.terminal(ended,fault))
    }
    @Test fun stringFlagsAndNonStringFaultsCannotBecomeAcceptedState() {
        for(value in listOf(null,"true","false",0,1,0.0)) assertFalse(PlaybackSnapshotFields.terminal(value,null))
        for(value in listOf(true,0,1L,1.0,listOf("fault"))) assertFalse(PlaybackSnapshotFields.terminal(false,value))
    }
}
