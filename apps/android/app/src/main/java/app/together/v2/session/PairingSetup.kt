package app.together.v2.session

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

/** Calls are serialized on Main. Storage and HTTP waits retain one setup owner. */
internal class PairingSetup {
    private val mutable=MutableStateFlow(false)
    val busy:StateFlow<Boolean> = mutable
    suspend fun run(change:suspend ()->Unit) {
        if(mutable.value) return
        mutable.value=true
        try {change()} finally {mutable.value=false}
    }
}
