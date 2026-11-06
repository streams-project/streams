package streams.app

import android.app.UiModeManager
import android.content.res.Configuration
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import contract.DeviceClass
import contract.Form
import streams.adapters.HttpNetwork
import streams.adapters.Media3Player
import streams.adapters.PrefsSecrets
import streams.core.Host
import streams.surfaces.Root

/** Plugs adapters into ports and hands the surfaces the result. Nothing else knows both sides. */
class Main : ComponentActivity() {
    override fun onCreate(saved: Bundle?) {
        super.onCreate(saved)
        val modes = getSystemService(UiModeManager::class.java)
        val tv = modes.currentModeType == Configuration.UI_MODE_TYPE_TELEVISION
        val host = Host(
            network = HttpNetwork(),
            secrets = PrefsSecrets(this),
            player = Media3Player(this),
            deviceClass = if (tv) DeviceClass.TV else DeviceClass.PHONE,
            form = if (tv) Form.TV else Form.PHONE,
            version = "0.1.0",
        )
        setContent { Root(host, tv) }
    }
}
