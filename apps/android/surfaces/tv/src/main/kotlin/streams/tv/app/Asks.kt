package streams.tv

import streams.ports.Shell
import streams.shared.read as sharedRead
import streams.shared.write as sharedWrite

/** The text of a read, or nothing when it did not work. */
suspend fun Shell.read(path: String): String? = sharedRead(path)

/** The text of a change, or nothing when it was refused. */
suspend fun Shell.write(method: String, path: String, body: String): String? = sharedWrite(method, path, body)
