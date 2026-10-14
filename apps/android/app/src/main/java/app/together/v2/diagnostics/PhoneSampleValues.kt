package app.together.v2.diagnostics

import app.together.v2.player.PlayerStatus

/** Diagnostics must retain JSON types rather than coerce strings or truncate floats. */
internal fun phoneInteger(value:Any?,range:LongRange):Long {
    require(value is Byte || value is Short || value is Int || value is Long)
    return (value as Number).toLong().also {require(it in range)}
}
internal fun phoneBoolean(value:Any?):Boolean {
    require(value is Boolean)
    return value
}

/** Initialization failure is diagnostic failure, not a new authoritative fault code. */
internal fun phonePlayerFailed(player:PlayerStatus):Boolean =
    player.failure!=null || (!player.initialized && player.state=="Failed")

internal fun phonePlaybackState(player:PlayerStatus,ended:Boolean):String = when {
    phonePlayerFailed(player) -> "failed"
    ended -> "ended"
    player.state=="Playing" -> "playing"
    player.state=="Paused" -> "paused"
    player.initialized -> "loading"
    else -> "starting"
}
