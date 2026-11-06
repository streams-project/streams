package streams.core

/** The core's functions, through bridge.c. A handle is used from one thread at a time. */
object NativeCore {
    init {
        System.loadLibrary("streamscore")
    }

    @JvmStatic external fun abi(): Int
    @JvmStatic external fun open(config: ByteArray): Long
    @JvmStatic external fun step(handle: Long, input: ByteArray): ByteArray
    @JvmStatic external fun close(handle: Long)
}
