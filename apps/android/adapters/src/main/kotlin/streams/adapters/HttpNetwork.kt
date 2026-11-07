package streams.adapters

import android.content.Context
import android.net.ConnectivityManager
import java.io.IOException
import java.net.HttpURLConnection
import java.net.SocketTimeoutException
import java.net.URL
import java.util.concurrent.Executors
import streams.ports.Answer
import streams.ports.Call
import streams.ports.Network
import streams.ports.NoReply
import streams.ports.Reply

/** The network port, on the system's own connection. */
class HttpNetwork(private val context: Context) : Network {
    private val pool = Executors.newFixedThreadPool(4)

    override fun watch(changed: (online: Boolean) -> Unit) {
        val links = context.getSystemService(ConnectivityManager::class.java)
        links.registerDefaultNetworkCallback(object : ConnectivityManager.NetworkCallback() {
            override fun onAvailable(network: android.net.Network) = changed(true)
            override fun onLost(network: android.net.Network) = changed(false)
        })
    }

    override fun send(call: Call, done: (Answer) -> Unit) {
        pool.execute { done(run(call)) }
    }

    private fun run(call: Call): Answer = try {
        val link = URL(call.url).openConnection() as HttpURLConnection
        link.requestMethod = call.method
        link.connectTimeout = call.deadlineMs.toInt()
        link.readTimeout = call.deadlineMs.toInt()
        call.headers.forEach { (name, value) -> link.setRequestProperty(name, value) }
        call.body?.let { body ->
            link.doOutput = true
            link.setRequestProperty("content-type", "application/json")
            link.outputStream.use { it.write(body.toByteArray()) }
        }
        val status = link.responseCode
        val stream = if (status < 400) link.inputStream else link.errorStream
        val text = stream?.bufferedReader()?.use { it.readText() }.orEmpty()
        Reply(status, text, link.getHeaderField("etag"))
    } catch (_: SocketTimeoutException) {
        NoReply("timeout")
    } catch (_: IOException) {
        NoReply("offline")
    }
}
