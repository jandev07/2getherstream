package app.together.v2.update

import org.junit.Assert.*
import org.junit.Test

class UpdateReleaseTest {
    @Test fun unsupportedDownloadPortIsRejectedBeforeAdvertisingRelease() {
        val unsupported=fields()+mapOf("apk_url" to "https://example.invalid:444/update.apk")
        assertTrue("Metadata accepted an address the downloader refuses",runCatching {UpdateRelease.fromFields(unsupported::get)}.isFailure)
        val explicitDefault=fields()+mapOf("apk_url" to "https://example.invalid:443/update.apk")
        assertEquals(3L,UpdateRelease.fromFields(explicitDefault::get).code)
    }
    private fun fields():Map<String,Any?> = mapOf("protocol" to 1,"package" to "app.together.v2",
        "version_code" to 3,"version_name" to "Alpha","apk_url" to "https://example.invalid/update.apk",
        "sha256" to "a".repeat(64),"size_bytes" to 4096)

    @Test fun exactIntegersProduceTheSameReleaseWithoutAndroidContext() {
        val small=UpdateRelease.fromFields(fields()::get)
        assertEquals(3L,small.code);assertEquals(4096L,small.bytes);assertEquals("Alpha",small.name)
        val large=fields()+mapOf("protocol" to 1L,"version_code" to Int.MAX_VALUE.toLong(),"size_bytes" to 536870912L)
        val release=UpdateRelease.fromFields(large::get)
        assertEquals(Int.MAX_VALUE.toLong(),release.code);assertEquals(536870912L,release.bytes)
    }
    @Test fun coercedNumbersCannotDeclareProtocolVersionOrSize() {
        for(key in listOf("protocol","version_code","size_bytes")) {
            for(value in listOf(null,"1",1.0,1.9,true,Float.NaN,Double.POSITIVE_INFINITY)) {
                val changed=fields()+mapOf(key to value)
                assertTrue("Coerced $key accepted",runCatching {UpdateRelease.fromFields(changed::get)}.isFailure)
            }
        }
        for((key,value) in listOf("protocol" to 2,"version_code" to 0,"version_code" to Long.MAX_VALUE,
            "size_bytes" to -1,"size_bytes" to 536870913L)) {
            val changed=fields()+mapOf(key to value)
            assertTrue(runCatching {UpdateRelease.fromFields(changed::get)}.isFailure)
        }
    }
    @Test fun metadataTextCannotBeManufacturedFromNonStringFields() {
        for(key in listOf("package","version_name","apk_url","sha256")) {
            for(value in listOf(null,123,true,listOf("Alpha"))) {
                val changed=fields()+mapOf(key to value)
                assertTrue("Coerced $key accepted",runCatching {UpdateRelease.fromFields(changed::get)}.isFailure)
            }
        }
    }
}
