package app.together.v2.session

/** Canonical IDs use exact signed JSON integers supported by the Android player. */
internal object MediaIdentityFields {
    fun valid(provider:Any?,torrentId:Any?,fileId:Any?):Boolean {
        val torrent=MediaGrantNumbers.integer(torrentId) ?: return false
        val file=MediaGrantNumbers.integer(fileId) ?: return false
        return when(provider) {
            "fixture" -> torrent==0L && file==0L
            "torbox" -> torrent>0L && file>=0L
            else -> false
        }
    }
}
