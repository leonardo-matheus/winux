package com.winux.connect.widget

import android.app.PendingIntent
import android.appwidget.AppWidgetManager
import android.appwidget.AppWidgetProvider
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.view.View
import android.widget.RemoteViews
import com.winux.connect.MainActivity
import com.winux.connect.R
import com.winux.connect.data.ConnectionState

/**
 * Widget provider for displaying connection status on home screen
 */
class ConnectionStatusWidget : AppWidgetProvider() {

    companion object {
        const val ACTION_UPDATE = "com.winux.connect.ACTION_WIDGET_UPDATE"
        const val EXTRA_CONNECTION_STATE = "connection_state"
        const val EXTRA_DEVICE_NAME = "device_name"

        /**
         * Update all widget instances with new connection state
         */
        fun updateWidgets(context: Context, state: ConnectionState, deviceName: String? = null) {
            val intent = Intent(context, ConnectionStatusWidget::class.java).apply {
                action = ACTION_UPDATE
                putExtra(EXTRA_CONNECTION_STATE, state.name)
                putExtra(EXTRA_DEVICE_NAME, deviceName)
            }
            context.sendBroadcast(intent)
        }
    }

    override fun onUpdate(
        context: Context,
        appWidgetManager: AppWidgetManager,
        appWidgetIds: IntArray
    ) {
        for (appWidgetId in appWidgetIds) {
            updateAppWidget(context, appWidgetManager, appWidgetId, ConnectionState.DISCONNECTED, null)
        }
    }

    override fun onReceive(context: Context, intent: Intent) {
        super.onReceive(context, intent)

        if (intent.action == ACTION_UPDATE) {
            val stateName = intent.getStringExtra(EXTRA_CONNECTION_STATE)
            val deviceName = intent.getStringExtra(EXTRA_DEVICE_NAME)
            val state = try {
                ConnectionState.valueOf(stateName ?: ConnectionState.DISCONNECTED.name)
            } catch (e: Exception) {
                ConnectionState.DISCONNECTED
            }

            val appWidgetManager = AppWidgetManager.getInstance(context)
            val componentName = ComponentName(context, ConnectionStatusWidget::class.java)
            val appWidgetIds = appWidgetManager.getAppWidgetIds(componentName)

            for (appWidgetId in appWidgetIds) {
                updateAppWidget(context, appWidgetManager, appWidgetId, state, deviceName)
            }
        }
    }

    private fun updateAppWidget(
        context: Context,
        appWidgetManager: AppWidgetManager,
        appWidgetId: Int,
        state: ConnectionState,
        deviceName: String?
    ) {
        val views = RemoteViews(context.packageName, R.layout.widget_connection_status)

        // Update status text
        val statusText = when (state) {
            ConnectionState.CONNECTED -> context.getString(R.string.status_connected)
            ConnectionState.CONNECTING -> context.getString(R.string.status_connecting)
            ConnectionState.PAIRING -> "Pairing..."
            ConnectionState.ERROR -> "Error"
            ConnectionState.DISCONNECTED -> context.getString(R.string.status_disconnected)
        }
        views.setTextViewText(R.id.status_text, statusText)

        // Update status indicator color
        val indicatorColor = when (state) {
            ConnectionState.CONNECTED -> context.getColor(R.color.status_connected)
            ConnectionState.CONNECTING, ConnectionState.PAIRING -> context.getColor(R.color.status_connecting)
            ConnectionState.ERROR -> context.getColor(R.color.status_error)
            ConnectionState.DISCONNECTED -> context.getColor(R.color.status_disconnected)
        }
        views.setInt(R.id.status_indicator, "setColorFilter", indicatorColor)

        // Update device name
        if (state == ConnectionState.CONNECTED && !deviceName.isNullOrEmpty()) {
            views.setTextViewText(R.id.device_name, deviceName)
            views.setViewVisibility(R.id.device_name, View.VISIBLE)
        } else {
            views.setViewVisibility(R.id.device_name, View.GONE)
        }

        // Set click intent to open app
        val openAppIntent = Intent(context, MainActivity::class.java)
        val pendingIntent = PendingIntent.getActivity(
            context,
            0,
            openAppIntent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )
        views.setOnClickPendingIntent(R.id.connect_button, pendingIntent)

        appWidgetManager.updateAppWidget(appWidgetId, views)
    }

    override fun onEnabled(context: Context) {
        // Widget added for the first time
    }

    override fun onDisabled(context: Context) {
        // Last widget removed
    }
}
