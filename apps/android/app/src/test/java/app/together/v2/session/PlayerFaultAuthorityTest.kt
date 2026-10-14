package app.together.v2.session

import org.junit.Assert.*
import org.junit.Test

class PlayerFaultAuthorityTest {
    private val faults=listOf("load_failed","decoder_failed","command_failed","unexpected_end")
    @Test fun everyRecordedFaultMustMatchThePendingNativeFault() {
        for(native in faults) for(recorded in faults) {
            assertEquals(native==recorded,PlayerFaultAuthority.canRecover(native,recorded,true,true,true))
        }
    }
    @Test fun freshExplicitPreparationAndClearedRoomFaultAreStillRequired() {
        for(fault in faults) {
            assertTrue(PlayerFaultAuthority.canRecover(fault,fault,true,true,true))
            assertFalse(PlayerFaultAuthority.canRecover(fault,fault,false,true,true))
            assertFalse(PlayerFaultAuthority.canRecover(fault,fault,true,false,true))
            assertFalse(PlayerFaultAuthority.canRecover(fault,fault,true,true,false))
        }
    }
    @Test fun absentUnknownOrCoercedRecordedFaultCannotAuthorizeRecovery() {
        for(value in listOf(null,"",true,1,"DecoderFailed","private_invalid_fault")) {
            assertFalse(PlayerFaultAuthority.known(value))
            assertFalse(PlayerFaultAuthority.canRecover("decoder_failed",value,true,true,true))
        }
        assertFalse(PlayerFaultAuthority.canRecover(null,null,true,true,true))
        assertFalse(PlayerFaultAuthority.canRecover("private_invalid_fault","private_invalid_fault",true,true,true))
    }
    @Test fun changedNativeResultRequiresNewMatchingAuthorityAfterWait() {
        assertTrue(PlayerFaultAuthority.canRecover("decoder_failed","decoder_failed",true,true,true))
        assertFalse(PlayerFaultAuthority.canRecover("command_failed","decoder_failed",true,true,true))
        assertTrue(PlayerFaultAuthority.canRecover("command_failed","command_failed",true,true,true))
    }
}
