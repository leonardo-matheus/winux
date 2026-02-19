package com.winux.connect.services

import android.app.Notification
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.ServiceConnection
import android.graphics.Bitmap
import android.graphics.drawable.BitmapDrawable
import android.graphics.drawable.Icon
import android.os.Build
import android.os.IBinder
import android.service.notification.NotificationListenerService
import android.service.notification.StatusBarNotification
import android.util.Base64
import android.util.Log
import com.winux.connect.data.NotificationPayload
import com.winux.connect.data.NotificationAction
import com.winux.connect.protocol.WinuxProtocol
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.*
import java.io.ByteArrayOutputStream
import javax.inject.Inject

/**
 * NotificationListenerService that forwards phone notifications to Winux desktop
 */
@AndroidEntryPoint
class NotificationListener : NotificationListenerService() {

    @Inject
    lateinit var protocol: WinuxProtocol

    private val serviceScope = CoroutineScope(Dispatchers.IO + SupervisorJob())

    // Apps to ignore (system apps, our own app, etc.)
    private val ignoredPackages = setOf(
        "com.winux.connect",
        "com.android.systemui",
        "com.android.providers.downloads",
        "android"
    )

    companion object {
        private const val TAG = "WinuxNotificationListener"
        private const val ICON_SIZE = 96
    }

    override fun onCreate() {
        super.onCreate()
        Log.d(TAG, "NotificationListener created")
    }

    override fun onDestroy() {
        super.onDestroy()
        serviceScope.cancel()
        Log.d(TAG, "NotificationListener destroyed")
    }

    override fun onListenerConnected() {
        super.onListenerConnected()
        Log.d(TAG, "NotificationListener connected")
    }

    override fun onListenerDisconnected() {
        super.onListenerDisconnected()
        Log.d(TAG, "NotificationListener disconnected")
    }

    override fun onNotificationPosted(sbn: StatusBarNotification?) {
        sbn ?: return

        // Skip ignored packages
        if (sbn.packageName in ignoredPackages) return

        // Skip group summary notifications
        if (sbn.notification.flags and Notification.FLAG_GROUP_SUMMARY != 0) return

        // Only forward if connected
        if (!protocol.isConnected()) return

        serviceScope.launch {
            try {
                val payload = extractNotificationPayload(sbn)
                protocol.sendNotification(payload)
                Log.d(TAG, "Forwarded notification from ${sbn.packageName}")
            } catch (e: Exception) {
                Log.e(TAG, "Failed to forward notification", e)
            }
        }
    }

    override fun onNotificationRemoved(sbn: StatusBarNotification?) {
        sbn ?: return

        if (!protocol.isConnected()) return

        // Optionally notify desktop that notification was dismissed
        // This could be used to sync notification dismissals
    }

    /**
     * Extract notification payload from StatusBarNotification
     */
    private fun extractNotificationPayload(sbn: StatusBarNotification): NotificationPayload {
        val notification = sbn.notification
        val extras = notification.extras

        val appName = getAppName(sbn.packageName)
        val title = extras.getCharSequence(Notification.EXTRA_TITLE)?.toString() ?: ""
        val text = extras.getCharSequence(Notification.EXTRA_TEXT)?.toString()
            ?: extras.getCharSequence(Notification.EXTRA_BIG_TEXT)?.toString()
            ?: ""

        val icon = extractIcon(notification, sbn.packageName)

        val actions = notification.actions?.mapIndexed { index, action ->
            NotificationAction(
                id = "${sbn.key}_action_$index",
                label = action.title?.toString() ?: ""
            )
        } ?: emptyList()

        return NotificationPayload(
            id = sbn.key,
            appName = appName,
            appPackage = sbn.packageName,
            title = title,
            text = text,
            icon = icon,
            timestamp = sbn.postTime,
            actions = actions,
            isOngoing = notification.flags and Notification.FLAG_ONGOING_EVENT != 0,
            priority = notification.priority
        )
    }

    /**
     * Get app name from package name
     */
    private fun getAppName(packageName: String): String {
        return try {
            val pm = packageManager
            val appInfo = pm.getApplicationInfo(packageName, 0)
            pm.getApplicationLabel(appInfo).toString()
        } catch (e: Exception) {
            packageName
        }
    }

    /**
     * Extract and encode notification icon
     */
    private fun extractIcon(notification: Notification, packageName: String): String? {
        return try {
            val icon: Icon? = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
                notification.smallIcon ?: notification.getLargeIcon()
            } else {
                null
            }

            icon?.let { iconObj ->
                val drawable = iconObj.loadDrawable(this)
                if (drawable is BitmapDrawable) {
                    val bitmap = Bitmap.createScaledBitmap(
                        drawable.bitmap,
                        ICON_SIZE,
                        ICON_SIZE,
                        true
                    )
                    encodeBitmapToBase64(bitmap)
                } else {
                    null
                }
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to extract icon", e)
            null
        }
    }

    /**
     * Encode bitmap to Base64 string
     */
    private fun encodeBitmapToBase64(bitmap: Bitmap): String {
        val stream = ByteArrayOutputStream()
        bitmap.compress(Bitmap.CompressFormat.PNG, 80, stream)
        val bytes = stream.toByteArray()
        return Base64.encodeToString(bytes, Base64.NO_WRAP)
    }

    /**
     * Check if notification listener permission is granted
     */
    companion object {
        fun isPermissionGranted(context: Context): Boolean {
            val flat = android.provider.Settings.Secure.getString(
                context.contentResolver,
                "enabled_notification_listeners"
            )
            return flat?.contains(context.packageName) == true
        }

        fun requestPermission(context: Context) {
            val intent = Intent("android.settings.ACTION_NOTIFICATION_LISTENER_SETTINGS")
            intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            context.startActivity(intent)
        }
    }
}
