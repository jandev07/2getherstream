package app.together.v2

import android.content.Context
import android.content.Intent
import android.os.SystemClock
import androidx.test.core.app.ActivityScenario
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import app.together.v2.session.*
import app.together.v2.player.PlayerProofController
import kotlinx.coroutines.runBlocking
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Assume.assumeTrue
import org.junit.Test
import org.junit.runner.RunWith
import java.io.File
import java.util.UUID

@RunWith(AndroidJUnit4::class)
class PairingConnectionTest {
    @Test fun oneTimeClaimEncryptedPersistenceAndAuthenticatedConnection()=runBlocking {
        val context=ApplicationProvider.getApplicationContext<Context>()
        val configFile=File(context.filesDir,"proof-pairing.json")
        assumeTrue("Requires isolated live pairing proof host",configFile.exists())
        val config=JSONObject(configFile.readText())
        val id=UUID.randomUUID().toString();val store=PairingStore(context,"together2-connection-test-$id","pairing-connection-$id.enc")
        val client=PairingClient()
        try {
            val paired=client.claim("http://127.0.0.1:47820",config.getString("code"))
            assertTrue("Claim returned wrong device credential",paired.token==config.getString("token"))
            assertTrue("Pairing nonce could be replayed",runCatching {client.claim(paired.baseUrl,config.getString("code"))}.isFailure)
            store.save(paired)
            val restored=checkNotNull(store.load())
            val launch=Intent(context,MainActivity::class.java).putExtra("localProofOnly",true)
            ActivityScenario.launch<MainActivity>(launch).use {scenario ->
                lateinit var controller:PlayerProofController
                scenario.onActivity {controller=it.controller;controller.connectSession(restored.socketUrl,restored.token)}
                val until=SystemClock.elapsedRealtime()+20000
                while(SystemClock.elapsedRealtime()<until && controller.session.diagnostics.value.rttMs==null) SystemClock.sleep(100)
                assertTrue("Stored device credential did not authenticate",controller.session.diagnostics.value.connected)
                assertNotNull("No authenticated clock response",controller.session.diagnostics.value.rttMs)
            }
        } finally {client.close();store.clear()}
    }
}
