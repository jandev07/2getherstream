package app.together.v2.network

import okhttp3.OkHttpClient

/** Only for clients created and owned by this app component, never shared clients. */
internal fun OkHttpClient.closeOwned() {
    dispatcher.cancelAll()
    dispatcher.executorService.shutdown()
    connectionPool.evictAll()
}
