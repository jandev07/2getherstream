package app.together.v2.player

import app.together.v2.BuildConfig

/** Process-local instrumentation setting. Never changes Android's audio volume. */
internal object NativeTestAudio {
    @Volatile var muted = false
        private set
    fun enableMutedProofs() { check(BuildConfig.DEBUG); muted = true }
}
