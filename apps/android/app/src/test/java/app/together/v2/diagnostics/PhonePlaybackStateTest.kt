package app.together.v2.diagnostics

import app.together.v2.player.PlayerStatus
import org.junit.Assert.*
import org.junit.Test

/** Actual safe category projection; JSONObject adapter/JNI are not executed here. */
class PhonePlaybackStateTest {
    @Test fun failedInitializationIsFailedWithoutInventingPlayerFaultCode() {
        val failed=PlayerStatus(state="Failed",error="Native player initialization failed")
        assertNull(failed.failure);assertTrue(phonePlayerFailed(failed))
        assertEquals("failed",phonePlaybackState(failed,false))
        assertEquals("failed",phonePlaybackState(failed,true))
    }
    @Test fun existingStartingLoadingAndPlaybackCategoriesStayUnchanged() {
        assertFalse(phonePlayerFailed(PlayerStatus()))
        assertEquals("starting",phonePlaybackState(PlayerStatus(),false))
        assertEquals("loading",phonePlaybackState(PlayerStatus(initialized=true,state="Idle"),false))
        assertEquals("paused",phonePlaybackState(PlayerStatus(initialized=true,state="Paused"),false))
        assertEquals("playing",phonePlaybackState(PlayerStatus(initialized=true,state="Playing"),false))
        assertEquals("ended",phonePlaybackState(PlayerStatus(initialized=true,state="Playing"),true))
        assertEquals("failed",phonePlaybackState(PlayerStatus(initialized=true,state="Failed",failure="command_failed"),true))
    }
    @Test fun recoveryObservationAndRecoverableNoticeDoNotInventFailure() {
        // Matching load can clear the fault before the next native state probe.
        val recovered=PlayerStatus(initialized=true,state="Failed",failure=null,sourceReady=false)
        assertFalse(phonePlayerFailed(recovered));assertEquals("loading",phonePlaybackState(recovered,false))
        val localNotice=PlayerStatus(initialized=true,state="Paused",error="Could not prepare the sample video. Try loading it again.")
        assertFalse(phonePlayerFailed(localNotice));assertEquals("paused",phonePlaybackState(localNotice,false))
    }
}
