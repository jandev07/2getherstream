package app.together.v2.session

import org.junit.Assert.*
import org.junit.Test

class MediaSourceBindingTest {
    private val binding=MediaSourceBinding("pc-session",7,3)
    @Test fun newGrantForSameAcceptedFileCanReuseNativeSourceAcrossPauseAndSeek() {
        assertTrue(binding.reusable("pc-session",7,3,3,true,false))
    }
    @Test fun newPcOrRefreshedProviderCapabilityRequiresReload() {
        assertFalse(binding.reusable("another-pc-session",7,3,3,true,false))
        assertFalse(binding.reusable("pc-session",8,3,3,true,false))
        assertFalse(MediaSourceBinding("pc-session",0,3).reusable("pc-session",0,3,3,true,false))
    }
    @Test fun replacedPendingUnavailableAndFailedNativeSourceCannotBeReused() {
        assertFalse(binding.reusable("pc-session",7,4,4,true,false))
        assertFalse(binding.reusable("pc-session",7,3,4,true,false))
        assertFalse(binding.reusable("pc-session",7,3,3,false,false))
        assertFalse(binding.reusable("pc-session",7,3,3,true,true))
        assertFalse(MediaSourceBinding("pc-session",7,0).reusable("pc-session",7,0,0,true,false))
    }
    @Test fun capabilityDebugTextCannotExposePrivatePlaybackUrl() {
        val direct="https://cdn.invalid/video?synthetic-private-capability"
        val capability=MediaCapability(7,direct)
        assertEquals(7L,capability.sourceRevision)
        assertFalse(capability.toString().contains("synthetic-private-capability"))
        assertEquals(direct,capability.deliver {it})
    }
}
