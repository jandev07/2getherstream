package app.together.v2

import android.os.Bundle
import androidx.test.runner.AndroidJUnitRunner
import app.together.v2.player.NativeTestAudio

/** Set before Activity/player creation, including direct adapter tests. */
class MutedTestRunner : AndroidJUnitRunner() {
    override fun onCreate(arguments: Bundle?) {
        NativeTestAudio.enableMutedProofs()
        super.onCreate(arguments)
    }
}
