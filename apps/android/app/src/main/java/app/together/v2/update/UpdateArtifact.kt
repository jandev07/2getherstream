package app.together.v2.update

import java.io.File
import kotlinx.coroutines.CancellationException

/** Runs on the HTTP worker. Cancellation of its waiter can precede file creation. */
internal fun <T> writeUpdateArtifact(file:File,active:()->Boolean,write:()->T):T {
    try {
        if(!active()) throw CancellationException("Update cancelled")
        val result=write()
        if(!active()) throw CancellationException("Update cancelled")
        return result
    } catch(error:Exception) {file.delete();throw error}
}
