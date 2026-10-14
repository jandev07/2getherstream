package app.together.v2.update

import androidx.compose.runtime.saveable.Saver
import androidx.compose.runtime.saveable.listSaver
import java.io.File

/** Only validated public metadata and an owned cache basename enter saved UI state.
 * Restoring a path does not verify the APK; installation still calls verifyArchive. */
internal fun updateReleaseSaver(installedCode:Long):Saver<UpdateRelease?,Any> = listSaver(
    save={release -> release?.let {it.validate();listOf(it.code,it.name,it.url,it.hash,it.bytes)} ?: emptyList()},
    restore={fields -> runCatching {
        require(fields.size==5)
        UpdateRelease(fields[0] as Long,fields[1] as String,fields[2] as String,fields[3] as String,fields[4] as Long)
            .apply {validate()}.takeIf {it.code>installedCode}
    }.getOrNull()}
)

private val updateBasename=Regex("[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}\\.apk")
internal fun updateApkSaver(directory:File):Saver<File?,String> = Saver(
    save={file -> file?.takeIf {updateBasename.matches(it.name) && it.canonicalFile.parentFile==directory.canonicalFile}?.name},
    restore={name -> if(!updateBasename.matches(name)) null else File(directory,name).takeIf {
        it.isFile && it.canonicalFile.parentFile==directory.canonicalFile
    }}
)
