package streams.adapters

import android.app.UiModeManager
import android.content.Context
import android.content.res.Configuration
import contract.DeviceClass

/**
 * What this device is. One place reads it, because three core policies key on exactly this field:
 * the client version floor, the per class start deadline, and how long a profile may be left before
 * the gate asks again. So it is read from the device itself and never from a flag: a tablet that
 * called itself a phone was held to a phone's rules, and a device that calls itself a television is
 * never asked who is watching.
 */
object ThisDevice {
    /** Android's own line between a phone's screen and a tablet's, in density independent pixels. */
    private const val TABLET_WIDTH_DP = 600

    fun deviceClass(context: Context): DeviceClass {
        val modes = context.getSystemService(UiModeManager::class.java)
        return when (modes?.currentModeType) {
            Configuration.UI_MODE_TYPE_TELEVISION -> DeviceClass.TV
            Configuration.UI_MODE_TYPE_CAR -> DeviceClass.CAR
            // Everything held in a hand is told apart by the screen it has, which is the only thing
            // Android offers: there is no flag for a tablet.
            else ->
                if (context.resources.configuration.smallestScreenWidthDp >= TABLET_WIDTH_DP) {
                    DeviceClass.TABLET
                } else {
                    DeviceClass.PHONE
                }
        }
    }
}
