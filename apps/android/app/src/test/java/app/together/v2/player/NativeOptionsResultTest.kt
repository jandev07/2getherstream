package app.together.v2.player

import io.github.yuroyami.libmpvkt.*
import io.github.yuroyami.libmpvkt.view.MpvOptions
import org.junit.Assert.*
import org.junit.Test
import java.io.File

/** Actual pinned options/result types; write results are synthetic, no JNI or TLS session. */
class NativeOptionsResultTest {
    private fun options()=MpvOptions(cacheDir=File("synthetic-cache"),tlsCaFile=File("synthetic-system-roots.pem"),
        extra=linkedMapOf("pause" to "yes","tls-verify" to "yes","stream-lavf-o" to "max_redirects=0",
            "ytdl" to "no","terminal" to "no","msg-level" to "all=no","mute" to "yes"))
    @Test fun checkedSinkPreservesExistingOptionSequenceAndRequiredValues() {
        val original=mutableListOf<Pair<String,String>>();val checked=mutableListOf<Pair<String,String>>()
        options().applyTo(MpvOptions.OptionSink {name,value ->original.add(name to value)})
        options().applyTo(MpvOptions.OptionSink {name,value ->checked.add(name to value);MpvResult.Ok(Unit).getOrThrow()})
        assertEquals(original,checked)
        val emitted=checked.toMap()
        for((name,value) in mapOf("config" to "no","vo" to "null","pause" to "yes","tls-verify" to "yes",
            "stream-lavf-o" to "max_redirects=0",
            "ytdl" to "no","terminal" to "no","msg-level" to "all=no","mute" to "yes"))assertEquals(value,emitted[name])
        assertEquals(File("synthetic-system-roots.pem").absolutePath,emitted["tls-ca-file"])
        assertEquals(File("synthetic-cache").absolutePath,emitted["gpu-shader-cache-dir"])
    }
    @Test fun checkedOptionFailureStopsBeforeInitializationAndLaterOptions() {
        for(target in listOf("tls-verify","tls-ca-file","stream-lavf-o","pause","ytdl","terminal","msg-level")) {
            val writes=mutableListOf<String>();var initialized=false
            val failure=runCatching {
                options().applyTo(MpvOptions.OptionSink {name,_ ->
                    writes.add(name)
                    val result:MpvResult<Unit> =if(name==target)MpvResult.Fail(MpvError.OPTION_ERROR,"synthetic private option detail") else MpvResult.Ok(Unit)
                    result.getOrThrow()
                })
                initialized=true
            }.exceptionOrNull()
            assertTrue(failure is MpvException);assertFalse(initialized);assertEquals(target,writes.last())
        }
    }
    @Test fun uncheckedSinkComparisonDiscardsRejectionAndContinues() {
        var rejected=false;var continuedAfterRejection=false
        options().applyTo(MpvOptions.OptionSink {name,_ ->
            if(rejected)continuedAfterRejection=true
            val result:MpvResult<Unit> =if(name=="tls-ca-file") {rejected=true;MpvResult.Fail(MpvError.OPTION_ERROR,"synthetic private option detail")} else MpvResult.Ok(Unit)
            // Exact unchecked result-discard policy of the pinned applyTo(Mpv) adapter.
            @Suppress("UNUSED_VARIABLE") val ignored=result
        })
        assertTrue(rejected);assertTrue(continuedAfterRejection)
    }
}
