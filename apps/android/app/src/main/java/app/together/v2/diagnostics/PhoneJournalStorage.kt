package app.together.v2.diagnostics

import java.io.File
import java.util.ArrayDeque
import kotlinx.coroutines.*
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

/** Share disk ownership across Activity/controller instances during recreation. */
internal class PhoneJournalStorage(private val directory:File,private val fileBytes:Long=1048576) {
    private val current=File(directory,"phone-current.jsonl")
    private val previous=File(directory,"phone-previous.jsonl")
    private companion object {val disk=Mutex()}
    init {require(fileBytes>0)}
    suspend fun appendApproved(line:String)=withContext(Dispatchers.IO) {
        require(!line.contains('\n') && !line.contains('\r') && line.toByteArray(Charsets.UTF_8).size<=2048)
        disk.withLock {
            check(directory.isDirectory || directory.mkdirs())
            if(current.length()>=fileBytes) {check(!previous.exists() || previous.delete());check(current.renameTo(previous))}
            current.appendText(line+"\n",Charsets.UTF_8)
        }
    }
    suspend fun <T:Any> tail(limit:Int,decode:(String)->T?):List<T> =withContext(Dispatchers.IO) {
        require(limit in 1..4096)
        disk.withLock {
            val records=ArrayDeque<T>()
            for(file in listOf(previous,current)) if(file.exists() && file.length()<=fileBytes+2048) {
                file.bufferedReader(Charsets.UTF_8).useLines {lines ->lines.forEach {line ->
                    if(line.length<=2048) decode(line)?.let {records.addLast(it);if(records.size>limit) records.removeFirst()}
                }}
            }
            records.toList()
        }
    }
}
