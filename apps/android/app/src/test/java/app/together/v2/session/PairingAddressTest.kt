package app.together.v2.session

import org.junit.Assert.*
import org.junit.Test

class PairingAddressTest {
    @Test fun rejectsHeaderUnsafeCredentialsWithFixedErrors() {
        for(token in listOf("a".repeat(31),"a".repeat(257),"PRIVATE_MARKER".repeat(3)+"\r\n",
            "PRIVATE_MARKER".repeat(3)+" ","PRIVATE_MARKER".repeat(3)+"\u0000",
            "PRIVATE_MARKER".repeat(3)+"\u007f","PRIVATE_MARKER".repeat(3)+"é")) {
            val failure=runCatching {validatePairingToken(token)}.exceptionOrNull()
            assertEquals("Invalid pairing credential",failure?.message)
            assertNull(failure?.cause)
            assertFalse(failure.toString().contains("PRIVATE_MARKER"))
        }
        for(token in listOf("a".repeat(32),"a".repeat(256),"01234567-89ab-cdef-0123-456789abcdef")) {
            validatePairingToken(token)
        }
    }
    @Test fun malformedCredentialBearingAddressesHaveFixedErrorsWithoutCauses() {
        for(address in listOf("https://user:PRIVATE_MARKER@[bad", "https://example.com/%PRIVATE_MARKER",
            "https://example.com/?token=PRIVATE_MARKER", "https://PRIVATE_MARKER@example.com",
            "https://example.com/#PRIVATE_MARKER", "https://example.com/PRIVATE_MARKER",
            "https://example.com/\u0000PRIVATE_MARKER")) {
            val failure=runCatching {pairingBase(address)}.exceptionOrNull()
            assertTrue(failure is IllegalArgumentException)
            assertEquals("Enter the Together server HTTPS address",failure?.message)
            assertNull(failure?.cause)
            assertFalse(failure.toString().contains("PRIVATE_MARKER"))
        }
    }
    @Test fun rejectsInvalidPortsAuthoritiesAndOversizedInput() {
        for(address in listOf("https://example.com:0","https://example.com:65536",
            "https://example.com:","https://@example.com", "https://example.com\\private",
            "https://example.com\n.private", "https://"+"a".repeat(8192))) {
            assertEquals("Enter the Together server HTTPS address",
                runCatching {pairingBase(address)}.exceptionOrNull()?.message)
        }
    }
    @Test fun preservesCanonicalHttpsAndOnlyTheExistingDebugLoopbackException() {
        assertEquals("https://together.example",pairingBase("  https://together.example/  "))
        assertEquals("https://together.example:443",pairingBase("https://together.example:443/"))
        assertEquals("https://[::1]:8443",pairingBase("https://[::1]:8443/"))
        assertEquals("http://127.0.0.1:47820",pairingBase("http://127.0.0.1:47820/"))
        for(address in listOf("http://example.com","http://127.0.0.1:47821","http://localhost:47820")) {
            assertEquals("Pairing requires HTTPS",runCatching {pairingBase(address)}.exceptionOrNull()?.message)
        }
    }
}
