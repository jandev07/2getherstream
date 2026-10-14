package app.together.v2.player

import kotlinx.coroutines.*

/** Release work must outlive cancellation of playback and UI jobs. All blocks
 * are synchronous and share the player's serial native-call dispatcher. */
internal class PlayerCleanup(dispatcher: CoroutineDispatcher, onFailure: () -> Unit) {
    private val job = SupervisorJob()
    // Native teardown failures must be observable without falling through to
    // Android's uncaught coroutine handler and terminating the app. Never pass
    // exception messages (which may contain media credentials) to diagnostics.
    private val failures = CoroutineExceptionHandler { _, _ -> onFailure() }
    private val scope = CoroutineScope(job + dispatcher + failures)

    fun enqueue(block: () -> Unit): Job = scope.launch { block() }

    /** Called once, after surface callbacks have been disconnected. Completion
     * waits for already queued releases instead of cancelling their bodies. */
    fun finish(block: () -> Unit): Job {
        val finalRelease = enqueue(block)
        job.complete()
        return finalRelease
    }
}
