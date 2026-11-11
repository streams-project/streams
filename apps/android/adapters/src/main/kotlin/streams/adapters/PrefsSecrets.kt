package streams.adapters

import android.content.Context
import streams.ports.Secrets

/** The secrets port, in this app's private storage, which Android keeps from other apps. */
class PrefsSecrets(context: Context) : Secrets {
    private val prefs = context.getSharedPreferences("secrets", Context.MODE_PRIVATE)

    override fun get(name: String): String? = prefs.getString(name, null)

    // Written before this returns, and the disk's answer is passed on. A session token lost to a
    // killed app ends the session, so a write that did not land must be reported, not assumed.
    override fun put(name: String, value: String): Boolean = prefs.edit().putString(name, value).commit()

    override fun drop(name: String) {
        prefs.edit().remove(name).commit()
    }
}
