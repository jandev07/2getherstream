package app.together.v2.update

import okhttp3.Request

internal const val PUBLIC_UPDATE_METADATA="https://2gether-download.vgovind28.workers.dev/release.json"
internal fun publicUpdateRequest():Request = Request.Builder().url(PUBLIC_UPDATE_METADATA)
    .header("Cache-Control","no-store").build()
