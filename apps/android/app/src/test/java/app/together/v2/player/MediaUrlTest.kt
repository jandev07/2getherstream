package app.together.v2.player

import org.junit.Assert.*
import org.junit.Test

class MediaUrlTest {
    @Test fun acceptsSafeCapabilitiesAndEncodedPaths() {
        listOf("https://cdn.invalid/movie.mp4?cap=file-token", "https://cdn.invalid:8443/a%20b.mp4", "https://[::1]/fixture.mp4",
            "https://cdn.invalid./file?cap=opaque", "https://api.torbox.app.cdn.invalid/file?cap=opaque").forEach(MediaUrl::validate)
    }
    @Test fun rejectsUnsafeOrMalformedSourcesWithFixedErrors() {
        listOf("http://cdn.invalid/private-secret", "https://", "https:///private-secret",
            "https://user:private-secret@cdn.invalid/movie", "https://@cdn.invalid/movie",
            "https://cdn.invalid/movie#private-secret", "https://API.TORBOX.APP/v1?token=private-secret",
            "https://api.torbox.app./v1?token=private-secret", "https://API.TORBOX.APP.:8443/private-secret",
            "https://%61pi.torbox.app./private-secret", "https://api.torbox.app../private-secret",
            "https://cdn.invalid/\nprivate-secret", "https://cdn.invalid/\u0000private-secret",
            "https://cdn.invalid/private secret", "https://cdn.invalid\\private-secret", "https://cdn.invalid:invalid/private-secret",
            "https://cdn.invalid/" + "a".repeat(8192)).forEach { url ->
            val error = runCatching { MediaUrl.validate(url) }.exceptionOrNull()
            assertTrue(error is IllegalArgumentException)
            assertEquals("Invalid media HTTPS URL", error?.message)
            assertFalse(error?.cause?.message?.contains("private-secret") == true)
        }
    }
}
