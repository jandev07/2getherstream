package app.together.v2.update

import androidx.compose.runtime.saveable.SaverScope
import java.io.File
import org.junit.Assert.*
import org.junit.Test
import org.junit.Rule
import org.junit.rules.TemporaryFolder

class UpdateSavedStateTest {
    @get:Rule val temp=TemporaryFolder()
    private val scope=object:SaverScope {override fun canBeSaved(value:Any)=value is Long || value is String}
    private val release=UpdateRelease(9,"Next","https://example.org/app.apk","a".repeat(64),123)
    @Test fun validatedPublicReleaseRoundTripsAndUpgradeDropsOldCandidate() {
        val saver=updateReleaseSaver(8)
        val saved=with(saver) {scope.save(release)}!!
        val restored=saver.restore(saved)!!
        assertEquals(release.code,restored.code);assertEquals(release.url,restored.url);assertEquals(release.hash,restored.hash)
        assertNull(updateReleaseSaver(9).restore(saved));assertNull(updateReleaseSaver(10).restore(saved))
    }
    @Test fun invalidOrCoercedSavedMetadataCannotBecomeCandidate() {
        val saver=updateReleaseSaver(8)
        for(fields in listOf(listOf(9,"Next",release.url,release.hash,123L),listOf(9L,"Next","http://example.org/a.apk",release.hash,123L),listOf(9L,"Next",release.url,"bad",123L),emptyList<Any>())) {
            assertNull(saver.restore(ArrayList(fields)))
        }
    }
    @Test fun ownedBasenameRoundTripsButMissingFileBecomesDownloadAgain() {
        val directory=temp.newFolder("updates");val file=File(directory,"12345678-1234-1234-1234-123456789abc.apk").apply {writeText("ordinary fixture")}
        val saver=updateApkSaver(directory);val name=with(saver){scope.save(file)}!!
        assertEquals(file,saver.restore(name));assertTrue(file.delete());assertNull(saver.restore(name))
    }
    @Test fun outsideAndTraversalPathsAreNotSavedOrRestored() {
        val directory=temp.newFolder("updates");val saver=updateApkSaver(directory)
        val outside=File(temp.root,"12345678-1234-1234-1234-123456789abc.apk").apply {writeText("ordinary fixture")}
        assertNull(with(saver){scope.save(outside)})
        for(name in listOf("../${outside.name}",outside.absolutePath,"verification-test.apk","")){assertNull(saver.restore(name))}
    }
}
