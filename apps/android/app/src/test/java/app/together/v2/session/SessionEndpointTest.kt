package app.together.v2.session

import org.junit.Assert.*
import org.junit.Test

class SessionEndpointTest {
    @Test fun secureControlAndAuthorizedDebugLoopbackUseExactSessionPath() {
        assertEquals("wss://pc.example.org:443/session",sessionEndpoint("wss://pc.example.org:443/session"))
        assertEquals("ws://127.0.0.1:47820/session",sessionEndpoint("ws://127.0.0.1:47820/session"))
    }
    @Test fun credentialUrlsQueryFragmentsOtherPathsAndSchemesFailBeforeSocketCreation() {
        for(address in listOf("wss://user:PRIVATE_CREDENTIAL@pc.example.org/session",
            "wss://pc.example.org/session?key=PRIVATE_CREDENTIAL","wss://pc.example.org/session#PRIVATE_CREDENTIAL",
            "wss://pc.example.org/other/session","wss://pc.example.org//session","wss://pc.example.org/session/",
            "https://pc.example.org/session","ws://pc.example.org/session","ws://127.0.0.1:47821/session",
            "wss://pc.example.org:0/session","wss:///session")) {
            val error=runCatching {sessionEndpoint(address)}.exceptionOrNull()
            assertNotNull(error);assertFalse(error.toString().contains("PRIVATE_CREDENTIAL"))
        }
    }
    @Test fun oversizedOrControlBearingSocketInputsHaveFixedPrivateErrors() {
        for(address in listOf("wss://pc.example.org/session\n", "wss://"+"x".repeat(8192)+"/session")) {
            assertNotNull(runCatching {sessionEndpoint(address)}.exceptionOrNull())
        }
        for(token in listOf("x".repeat(257),"x".repeat(31),"x".repeat(32)+"\r\nPRIVATE_CREDENTIAL")) {
            val error=runCatching {validatePairingToken(token)}.exceptionOrNull()
            assertEquals("Invalid pairing credential",error?.message)
            assertFalse(error.toString().contains("PRIVATE_CREDENTIAL"))
        }
    }
}
