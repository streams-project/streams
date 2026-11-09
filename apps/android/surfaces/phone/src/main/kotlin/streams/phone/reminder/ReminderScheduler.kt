package streams.phone.reminder

import android.app.AlarmManager
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.os.Build
import contract.TitleKey
import java.util.Calendar
import streams.shared.R

class ReminderReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        val title = intent.getStringExtra("title_name") ?: return
        val key = intent.getStringExtra("title_key") ?: return
        val channelId = "reminders"

        val manager = context.getSystemService(Context.NOTIFICATION_SERVICE) as? NotificationManager ?: return
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(channelId, "Reminders", NotificationManager.IMPORTANCE_HIGH).apply {
                description = "Notifications for arriving titles you asked to be reminded about"
            }
            manager.createNotificationChannel(channel)
        }

        val launchIntent = context.packageManager.getLaunchIntentForPackage(context.packageName)?.apply {
            putExtra("title_key", key)
            flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TOP
        }
        val pending = PendingIntent.getActivity(
            context,
            key.hashCode(),
            launchIntent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
        )

        val builder = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            Notification.Builder(context, channelId)
        } else {
            @Suppress("DEPRECATION")
            Notification.Builder(context)
        }

        val notification = builder
            .setSmallIcon(R.drawable.icon_bell)
            .setContentTitle(title)
            .setContentText("Now available to watch")
            .setAutoCancel(true)
            .setContentIntent(pending)
            .build()

        manager.notify(key.hashCode(), notification)
    }
}

object ReminderScheduler {
    fun schedule(context: Context, titleKey: TitleKey, titleName: String, releaseDateYmd: String) {
        val alarmManager = context.getSystemService(Context.ALARM_SERVICE) as? AlarmManager ?: return
        val intent = Intent(context, ReminderReceiver::class.java).apply {
            action = "streams.action.REMINDER"
            putExtra("title_key", titleKey.value)
            putExtra("title_name", titleName)
        }
        val pending = PendingIntent.getBroadcast(
            context,
            titleKey.value.hashCode(),
            intent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
        )

        val parts = releaseDateYmd.split("-").mapNotNull { it.toIntOrNull() }
        val triggerMs = if (parts.size == 3) {
            val cal = Calendar.getInstance().apply {
                set(Calendar.YEAR, parts[0])
                set(Calendar.MONTH, parts[1] - 1)
                set(Calendar.DAY_OF_MONTH, parts[2])
                set(Calendar.HOUR_OF_DAY, 9)
                set(Calendar.MINUTE, 0)
                set(Calendar.SECOND, 0)
                set(Calendar.MILLISECOND, 0)
            }
            maxOf(cal.timeInMillis, System.currentTimeMillis() + 1000)
        } else {
            System.currentTimeMillis() + 1000
        }

        try {
            alarmManager.setAndAllowWhileIdle(AlarmManager.RTC_WAKEUP, triggerMs, pending)
        } catch (_: SecurityException) {
            alarmManager.set(AlarmManager.RTC_WAKEUP, triggerMs, pending)
        }
    }

    fun cancel(context: Context, titleKey: TitleKey) {
        val alarmManager = context.getSystemService(Context.ALARM_SERVICE) as? AlarmManager ?: return
        val intent = Intent(context, ReminderReceiver::class.java).apply {
            action = "streams.action.REMINDER"
        }
        val pending = PendingIntent.getBroadcast(
            context,
            titleKey.value.hashCode(),
            intent,
            PendingIntent.FLAG_NO_CREATE or PendingIntent.FLAG_IMMUTABLE,
        )
        if (pending != null) {
            alarmManager.cancel(pending)
            pending.cancel()
        }
    }
}
