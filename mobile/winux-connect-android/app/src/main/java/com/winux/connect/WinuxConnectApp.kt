package com.winux.connect

import android.app.Application
import android.app.NotificationChannel
import android.app.NotificationManager
import android.content.Context
import android.os.Build
import dagger.hilt.android.HiltAndroidApp

/**
 * Main Application class for Winux Connect
 * Handles app-wide initialization and notification channels
 */
@HiltAndroidApp
class WinuxConnectApp : Application() {

    companion object {
        const val CHANNEL_CONNECTION = "winux_connection"
        const val CHANNEL_FILE_TRANSFER = "winux_file_transfer"
        const val CHANNEL_NOTIFICATIONS = "winux_notifications"
        const val CHANNEL_COMMANDS = "winux_commands"
    }

    override fun onCreate() {
        super.onCreate()
        createNotificationChannels()
    }

    private fun createNotificationChannels() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val notificationManager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager

            // Connection status channel
            val connectionChannel = NotificationChannel(
                CHANNEL_CONNECTION,
                getString(R.string.channel_connection),
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = getString(R.string.channel_connection_desc)
                setShowBadge(false)
            }

            // File transfer channel
            val fileTransferChannel = NotificationChannel(
                CHANNEL_FILE_TRANSFER,
                getString(R.string.channel_file_transfer),
                NotificationManager.IMPORTANCE_DEFAULT
            ).apply {
                description = getString(R.string.channel_file_transfer_desc)
            }

            // PC notifications channel
            val notificationsChannel = NotificationChannel(
                CHANNEL_NOTIFICATIONS,
                getString(R.string.channel_notifications),
                NotificationManager.IMPORTANCE_DEFAULT
            ).apply {
                description = getString(R.string.channel_notifications_desc)
            }

            // Remote commands channel
            val commandsChannel = NotificationChannel(
                CHANNEL_COMMANDS,
                getString(R.string.channel_commands),
                NotificationManager.IMPORTANCE_HIGH
            ).apply {
                description = getString(R.string.channel_commands_desc)
                enableVibration(true)
                enableLights(true)
            }

            notificationManager.createNotificationChannels(
                listOf(connectionChannel, fileTransferChannel, notificationsChannel, commandsChannel)
            )
        }
    }
}
