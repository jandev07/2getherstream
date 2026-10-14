package app.together.v2.update

import app.together.v2.session.MediaGrantNumbers
import okhttp3.HttpUrl.Companion.toHttpUrl
import org.json.JSONObject

class UpdateRelease(val code:Long,val name:String,val url:String,val hash:String,val bytes:Long) {
    companion object {
        fun parse(json:JSONObject):UpdateRelease {
            return fromFields(json::opt)
        }
        // One parser for JSON and context-free validation. Never use JSONObject's
        // string/fraction coercions to invent a protocol or release version.
        internal fun fromFields(field:(String)->Any?):UpdateRelease {
            require(MediaGrantNumbers.integer(field("protocol"))==1L && field("package")=="app.together.v2")
            fun text(key:String)=requireNotNull(field(key) as? String)
            return UpdateRelease(requireNotNull(MediaGrantNumbers.integer(field("version_code"))),
                text("version_name"),text("apk_url"),text("sha256"),
                requireNotNull(MediaGrantNumbers.integer(field("size_bytes")))).also {it.validate()}
        }
    }
    fun validate() {
        require(code in 1..Int.MAX_VALUE.toLong() && name.length in 1..64 && name.none {it.isISOControl()})
        require(bytes in 1..536870912 && hash.matches(Regex("[a-fA-F0-9]{64}")))
        val address=url.toHttpUrl()
        require(url.length<=2048 && address.isHttps && address.username.isEmpty() && address.password.isEmpty() && address.query==null && address.fragment==null)
    }
}

