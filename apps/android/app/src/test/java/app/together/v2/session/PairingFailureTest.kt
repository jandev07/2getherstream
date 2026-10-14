package app.together.v2.session

import org.junit.Assert.*
import org.junit.Test

class PairingFailureTest {
    @Test fun protocolRejectsCoercibleStringsFractionsAndWrongIntegers() {
        validatePairingProtocol(1);validatePairingProtocol(1L)
        for(value in listOf<Any>("1",1.0,1.5,0,2,Long.MAX_VALUE,true)) {
            try {validatePairingProtocol(value);fail("Invalid protocol accepted")}
            catch(error:IllegalArgumentException) {assertEquals("Invalid pairing response",error.message)}
        }
    }
    @Test fun errorsExplainRecoveryWithoutEchoingPrivateExceptionText() {
        val marker="PRIVATE_URL_OR_TOKEN"
        val unreachable=pairingFailureMessage(java.io.IOException(marker))
        assertTrue(unreachable.contains("Could not reach Windows"));assertFalse(unreachable.contains(marker))
        assertTrue(pairingFailureMessage(PairingRejected(403)).contains("expired"))
        assertTrue(pairingFailureMessage(InvalidPairingResponse()).contains("incompatible pairing response"))
        assertTrue(pairingFailureMessage(javax.net.ssl.SSLException(marker)).contains("secure connection"))
        assertFalse(pairingFailureMessage(IllegalArgumentException(marker)).contains(marker))
        assertFalse(pairingFailureMessage(Exception(marker)).contains(marker))
    }
}
