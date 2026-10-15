package app.together.v2.network

import okhttp3.OkHttpClient
import java.util.concurrent.TimeUnit

/** The paired /session endpoint is exact. A redirect cannot select another
 * control path/origin; reconnect must keep using the saved validated endpoint. */
internal fun controlSocketClient():OkHttpClient = OkHttpClient.Builder()
    .followRedirects(false).followSslRedirects(false)
    .pingInterval(2,TimeUnit.SECONDS).connectTimeout(10,TimeUnit.SECONDS).build()
