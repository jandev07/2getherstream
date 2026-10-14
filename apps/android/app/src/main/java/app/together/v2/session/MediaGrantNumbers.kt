package app.together.v2.session

/** Control grants must carry JSON integers, without JSONObject's coercions. */
internal object MediaGrantNumbers {
    fun integer(value:Any?):Long?=when(value) {
        is Int -> value.toLong()
        is Long -> value
        else -> null
    }
    fun matches(protocol:Any?,generation:Any?,sourceRevision:Any?,expectedGeneration:Long):Boolean {
        val revision=integer(sourceRevision) ?: return false
        return expectedGeneration>=0 && integer(protocol)==1L &&
            integer(generation)==expectedGeneration && revision>0
    }
}
