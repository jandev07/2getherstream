package app.together.v2.player

/** Invalid native metadata is unavailable, never a saturated/truncated ready observation. */
internal fun nativeTimeMillis(seconds:Double?):Long? {
    if(seconds==null || !seconds.isFinite() || seconds !in 0.0..604800.0) return null
    return (seconds*1000).toLong()
}
