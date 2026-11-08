package streams.app

import android.app.UiModeManager
import android.content.res.Configuration
import android.os.Bundle
import android.view.KeyEvent
import android.view.MotionEvent
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import contract.DeviceClass
import contract.Form
import streams.adapters.FileStore
import streams.adapters.HttpNetwork
import streams.adapters.Media3Player
import streams.adapters.Media3Reel
import streams.adapters.PrefsSecrets
import streams.core.Host
import streams.phone.Root
import streams.tv.Television

/** Plugs adapters into ports and hands the surfaces the result. Nothing else knows both sides. */
class Main : ComponentActivity() {
    private lateinit var host: Host

    override fun onCreate(saved: Bundle?) {
        super.onCreate(saved)
        val modes = getSystemService(UiModeManager::class.java)
        val tv = modes.currentModeType == Configuration.UI_MODE_TYPE_TELEVISION
        host = Host(
            network = HttpNetwork(this),
            secrets = PrefsSecrets(this),
            store = FileStore(this),
            player = Media3Player(this),
            reel = Media3Reel(),
            deviceClass = if (tv) DeviceClass.TV else DeviceClass.PHONE,
            form = if (tv) Form.TV else Form.PHONE,
            version = "0.1.0",
        )
        setContent { if (tv) Television(host) else Root(host, false) }
    }

    override fun onStart() {
        super.onStart()
        host.shown(true)
    }

    override fun onStop() {
        host.shown(false)
        super.onStop()
    }

    override fun dispatchKeyEvent(event: KeyEvent): Boolean {
        if (event.action == KeyEvent.ACTION_DOWN) host.touched()
        return super.dispatchKeyEvent(event)
    }

    override fun dispatchTouchEvent(event: MotionEvent): Boolean {
        if (event.action == MotionEvent.ACTION_DOWN) host.touched()
        return super.dispatchTouchEvent(event)
    }
}
