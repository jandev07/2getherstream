package app.together.v2.update

import org.junit.Assert.*
import org.junit.Test

class PublicUpdateRequestTest {
    @Test fun publicMetadataDoesNotCarryPairingCredentialsOrExpiryParameters() {
        val request=publicUpdateRequest()
        assertEquals(PUBLIC_UPDATE_METADATA,request.url.toString())
        assertEquals("https",request.url.scheme)
        assertEquals("2gether-download.vgovind28.workers.dev",request.url.host)
        assertEquals("/release.json",request.url.encodedPath)
        assertEquals("",request.url.username);assertEquals("",request.url.password)
        assertNull(request.url.query);assertNull(request.url.fragment)
        assertNull(request.header("Authorization"));assertNull(request.header("Cookie"))
        assertEquals("no-store",request.header("Cache-Control"))
    }
}
