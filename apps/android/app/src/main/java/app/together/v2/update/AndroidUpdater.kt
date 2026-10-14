package app.together.v2.update

import android.content.Context
import android.content.Intent
import android.content.pm.PackageInfo
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Build
import android.provider.Settings
import androidx.core.content.FileProvider
import app.together.v2.session.Pairing
import app.together.v2.network.closeOwned
import app.together.v2.session.pairingBase
import okhttp3.*
import org.json.JSONObject
import java.io.File
import java.io.IOException
import java.security.MessageDigest
import java.util.UUID
import java.util.concurrent.TimeUnit

/** APK bytes go directly to the release host, without paired control credentials. */
class AndroidUpdater(private val context:Context) {
    private val client=OkHttpClient.Builder().followRedirects(false).followSslRedirects(false)
        .connectTimeout(15,TimeUnit.SECONDS).readTimeout(30,TimeUnit.SECONDS).callTimeout(15,TimeUnit.MINUTES).build()
    private val directory=File(context.cacheDir,"updates")
    private val flags=if(Build.VERSION.SDK_INT>=28) PackageManager.GET_SIGNING_CERTIFICATES else PackageManager.GET_SIGNATURES
    @Suppress("DEPRECATION") private fun version(info:PackageInfo)=if(Build.VERSION.SDK_INT>=28) info.longVersionCode else info.versionCode.toLong()
    @Suppress("DEPRECATION") private fun signers(info:PackageInfo):Set<String> {
        val signatures=if(Build.VERSION.SDK_INT>=28) info.signingInfo?.apkContentsSigners else info.signatures
        return signatures?.map {sha256(it.toByteArray())}?.toSet() ?: emptySet()
    }
    @Suppress("DEPRECATION") fun installedCode()=version(context.packageManager.getPackageInfo(context.packageName,flags))
    /** Public release metadata contains no pairing or provider credentials. */
    suspend fun check():UpdateRelease? = checkRelease(publicUpdateRequest())
    suspend fun check(pairing:Pairing):UpdateRelease? {
        val base=pairingBase(pairing.baseUrl)
        val request=Request.Builder().url("$base/updates/android").header("Authorization","Bearer ${pairing.token}").header("Cache-Control","no-store").build()
        return checkRelease(request)
    }
    private suspend fun checkRelease(request:Request):UpdateRelease? {
        val release=fetch(request,timeoutMillis=UPDATE_METADATA_TIMEOUT_MS) {response,active ->
            if(response.code==503) throw IOException("Updates are not available yet. Try again later.")
            require(response.code==200)
            val buffer=java.io.ByteArrayOutputStream()
            checkNotNull(response.body).byteStream().use {input ->
                val chunk=ByteArray(1024)
                while(true) {check(active());val size=input.read(chunk);if(size<0) break;require(buffer.size()+size<=8192);buffer.write(chunk,0,size)}
            }
            UpdateRelease.parse(JSONObject(buffer.toString("UTF-8")))
        }
        return release.takeIf {it.code>installedCode()}
    }
    suspend fun download(release:UpdateRelease,progress:(Int)->Unit):File {
        release.validate();require(release.code>installedCode())
        directory.mkdirs()
        val file=File(directory,"${UUID.randomUUID()}.apk")
        try {
            client.awaitApkDownload(release.url) {response,active ->writeUpdateArtifact(file,active) {
                require(response.code==200)
                val body=checkNotNull(response.body)
                require(body.contentLength()==-1L || body.contentLength()==release.bytes)
                val digest=MessageDigest.getInstance("SHA-256");var total=0L;var last=-1
                body.byteStream().use {input -> file.outputStream().use {output ->
                    val chunk=ByteArray(65536)
                    while(true) {
                        check(active());val size=input.read(chunk);if(size<0) break
                        total+=size;require(total<=release.bytes);output.write(chunk,0,size);digest.update(chunk,0,size)
                        val percent=(total*100/release.bytes).toInt();if(percent!=last) {last=percent;progress(percent)}
                    }
                }}
                require(total==release.bytes && hex(digest.digest()).equals(release.hash,true))
                verifyArchive(file,release);file
            }}
            return file
        } catch(error:Exception) {file.delete();throw error}
    }
    @Suppress("DEPRECATION") fun verifyArchive(file:File,release:UpdateRelease) {
        require(file.canonicalFile.parentFile==directory.canonicalFile && file.length()==release.bytes)
        val digest=MessageDigest.getInstance("SHA-256")
        file.inputStream().use {input -> val chunk=ByteArray(65536);while(true) {val n=input.read(chunk);if(n<0) break;digest.update(chunk,0,n)}}
        require(hex(digest.digest()).equals(release.hash,true))
        val installed=context.packageManager.getPackageInfo(context.packageName,flags)
        val candidate=checkNotNull(context.packageManager.getPackageArchiveInfo(file.path,flags))
        require(candidate.packageName==context.packageName && version(candidate)==release.code && version(candidate)>version(installed))
        val expected=signers(installed)
        require(expected.isNotEmpty() && signers(candidate)==expected)
    }
    fun installerIntent(file:File):Intent {
        require(file.canonicalFile.parentFile==directory.canonicalFile && file.isFile)
        if(!context.packageManager.canRequestPackageInstalls()) return Intent(Settings.ACTION_MANAGE_UNKNOWN_APP_SOURCES,Uri.parse("package:${context.packageName}"))
        val uri=FileProvider.getUriForFile(context,"${context.packageName}.updates",file)
        return Intent(Intent.ACTION_VIEW).setDataAndType(uri,"application/vnd.android.package-archive")
            .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION).also {it.clipData=android.content.ClipData.newRawUri("2gether update",uri)}
    }
    fun canInstall()=context.packageManager.canRequestPackageInstalls()
    fun close() {client.closeOwned()}
    private suspend fun <T> fetch(request:Request,timeoutMillis:Long?=null,read:(Response,()->Boolean)->T):T =
        client.newCall(request).awaitUpdateRead(timeoutMillis,read)

}
private fun hex(bytes:ByteArray)=bytes.joinToString("") {"%02x".format(it)}
private fun sha256(bytes:ByteArray)=hex(MessageDigest.getInstance("SHA-256").digest(bytes))
