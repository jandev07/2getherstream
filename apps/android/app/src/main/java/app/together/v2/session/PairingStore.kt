package app.together.v2.session

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.AtomicFile
import org.json.JSONObject
import java.io.File
import java.io.InputStream
import java.nio.ByteBuffer
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

internal const val MAX_PAIRING_BYTES=4096
/** Read at most one byte beyond the bound, even with partial reads or file growth. */
internal fun readPairingBytes(input:InputStream):ByteArray {
    val bytes=ByteArray(MAX_PAIRING_BYTES+1)
    var count=0
    while(count<bytes.size) {
        val read=input.read(bytes,count,bytes.size-count)
        if(read<0) break
        if(read==0) {
            val next=input.read()
            if(next<0) break
            bytes[count++]=next.toByte()
        } else count+=read
    }
    require(count in 1..MAX_PAIRING_BYTES) {"Stored pairing is invalid"}
    return bytes.copyOf(count)
}

/** Observe publication after AtomicFile closes its writer; never roll it back here. */
internal fun verifyPairingCommit(file:File,expected:ByteArray) {
    val published=try {
        file.isFile && file.inputStream().use {readPairingBytes(it)}.contentEquals(expected)
    } catch(_:Exception) {false}
    check(published) {"Stored pairing could not be saved"}
}

/** No data-class toString: bearer credentials must never appear in diagnostics. */
class Pairing(val baseUrl:String,val token:String) {
    init {validatePairingToken(token)}
    val socketUrl:String get()=baseUrl.replaceFirst("https://","wss://").replaceFirst("http://","ws://")+"/session"
}
/** AES-GCM key stays in Android Keystore; ciphertext stays in private, non-backed-up storage. */
class PairingStore(context:Context,private val alias:String="together2-pairing-v1",name:String="pairing-v1.enc") {
    // AtomicFile and Keystore ownership must span separate UI/update/log owners.
    private companion object {val disk=Any()}
    private val file=AtomicFile(File(context.noBackupFilesDir,name))
    private fun key(create:Boolean=true):SecretKey {
        val store=KeyStore.getInstance("AndroidKeyStore").apply {load(null)}
        (store.getKey(alias,null) as? SecretKey)?.let {return it}
        check(create) {"Stored pairing key unavailable"}
        return KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES,"AndroidKeyStore").apply {
            init(KeyGenParameterSpec.Builder(alias,KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
                .setBlockModes(KeyProperties.BLOCK_MODE_GCM).setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE).setKeySize(256).build())
        }.generateKey()
    }
    fun save(pairing:Pairing) {synchronized(disk) {
        val base=pairingBase(pairing.baseUrl)
        val plaintext=JSONObject().put("version",1).put("base",base).put("token",pairing.token).toString().toByteArray()
        try {
            val cipher=Cipher.getInstance("AES/GCM/NoPadding").apply {init(Cipher.ENCRYPT_MODE,key());updateAAD("Together2 pairing v1".toByteArray())}
            val encrypted=cipher.doFinal(plaintext)
            val bytes=ByteBuffer.allocate(1+cipher.iv.size+encrypted.size).put(cipher.iv.size.toByte()).put(cipher.iv).put(encrypted).array()
            require(bytes.size<=MAX_PAIRING_BYTES) {"Pairing is too large"}
            val output=file.startWrite()
            try {output.write(bytes);file.finishWrite(output)} catch(e:Exception) {file.failWrite(output);throw e}
            // finishWrite may only log a failed rename. Confirm the bounded
            // ciphertext before setup opens the session. This is outside the
            // rollback catch: failWrite must not undo an already finished write.
            verifyPairingCommit(file.baseFile,bytes)
        } finally {plaintext.fill(0)}
    }
    }
    fun load():Pairing? =synchronized(disk) {
        val backup=File(file.baseFile.path+".bak")
        if(!file.baseFile.exists() && !backup.exists()) return@synchronized null
        val bytes=file.openRead().use {readPairingBytes(it)};val size=bytes.firstOrNull()?.toInt() ?: error("Stored pairing is invalid")
        require(size==12 && bytes.size>size+17) {"Stored pairing is invalid"}
        val iv=bytes.copyOfRange(1,1+size)
        val cipher=Cipher.getInstance("AES/GCM/NoPadding").apply {init(Cipher.DECRYPT_MODE,key(false),GCMParameterSpec(128,iv));updateAAD("Together2 pairing v1".toByteArray())}
        val plaintext=cipher.doFinal(bytes,1+size,bytes.size-1-size)
        try {
            val parsed=JSONObject(String(plaintext));require(parsed.getInt("version")==1)
            val token=parsed.getString("token")
            Pairing(pairingBase(parsed.getString("base")),token)
        } finally {plaintext.fill(0)}
    }
    fun clear() {synchronized(disk) {
        file.delete()
        // AtomicFile.delete() ignores failed unlinks. Keep the key until all
        // owned file variants are gone so a failed Forget can be retried.
        check(listOf(file.baseFile,File(file.baseFile.path+".bak"),File(file.baseFile.path+".new")).none {it.exists()}) {
            "Stored pairing could not be removed"
        }
        KeyStore.getInstance("AndroidKeyStore").apply {load(null);deleteEntry(alias)}
    }}
}
