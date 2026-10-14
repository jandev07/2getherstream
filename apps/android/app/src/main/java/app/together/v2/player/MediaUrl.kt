package app.together.v2.player

import java.net.URI

/** Native boundary defense; the PC resolver separately enforces the account-key guard. */
internal object MediaUrl {
    fun validate(url: String) {
        val bounded = url.length <= 8192 && url.toByteArray(Charsets.UTF_8).size <= 8192
        val shape = bounded && url.startsWith("https://") &&
            url.none { it.isWhitespace() || Character.isISOControl(it) }
        // URI exceptions include the original input. Never let them leave here.
        val parsed = if (shape) runCatching { URI(url) }.getOrNull() else null
        require(parsed != null && parsed.scheme == "https" && !parsed.host.isNullOrEmpty() &&
            parsed.rawUserInfo == null && parsed.rawFragment == null &&
            !parsed.host.trimEnd('.').equals("api.torbox.app", ignoreCase = true)) { "Invalid media HTTPS URL" }
    }
}
