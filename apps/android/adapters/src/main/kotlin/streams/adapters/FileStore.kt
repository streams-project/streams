package streams.adapters

import android.content.Context
import java.io.File
import java.security.MessageDigest
import streams.ports.Store

private const val PART = ".part"

/**
 * The store port, as one file per key in this app's private folder. A key can be longer than a
 * file name, so the name is its hash and the first line of the file is the key itself.
 */
class FileStore(context: Context) : Store {
    private val folder = File(context.filesDir, "store").apply { mkdirs() }

    private fun fileOf(key: String): File {
        val hash = MessageDigest.getInstance("SHA-256").digest(key.toByteArray())
        return File(folder, hash.joinToString("") { "%02x".format(it) })
    }

    override fun get(key: String): String? {
        val file = fileOf(key).takeIf(File::exists) ?: return null
        return file.readText().substringAfter('\n')
    }

    /** Written whole and then moved into place, so a killed app leaves the old value or the new. */
    override fun put(key: String, value: String) {
        val file = fileOf(key)
        val part = File(folder, file.name + PART)
        part.writeText("$key\n$value")
        part.renameTo(file)
    }

    override fun delete(key: String) {
        fileOf(key).delete()
    }

    override fun keys(prefix: String): List<String> =
        folder.listFiles().orEmpty()
            .filterNot { it.name.endsWith(PART) }
            .map { file -> file.bufferedReader().use { it.readLine().orEmpty() } }
            .filter { it.startsWith(prefix) }
}
