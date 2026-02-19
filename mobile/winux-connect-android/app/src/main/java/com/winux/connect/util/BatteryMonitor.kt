package com.winux.connect.util

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.os.BatteryManager
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.callbackFlow
import javax.inject.Inject
import javax.inject.Singleton

/**
 * Monitors battery status and provides updates
 */
@Singleton
class BatteryMonitor @Inject constructor(
    @ApplicationContext private val context: Context
) {
    private val batteryManager: BatteryManager by lazy {
        context.getSystemService(Context.BATTERY_SERVICE) as BatteryManager
    }

    private val _batteryStatus = MutableStateFlow(getBatteryStatus())
    val batteryStatus: StateFlow<BatteryStatus> = _batteryStatus.asStateFlow()

    private var receiver: BroadcastReceiver? = null

    /**
     * Start monitoring battery status changes
     */
    fun startMonitoring() {
        if (receiver != null) return

        receiver = object : BroadcastReceiver() {
            override fun onReceive(context: Context?, intent: Intent?) {
                intent?.let {
                    _batteryStatus.value = parseBatteryIntent(it)
                }
            }
        }

        val filter = IntentFilter().apply {
            addAction(Intent.ACTION_BATTERY_CHANGED)
            addAction(Intent.ACTION_POWER_CONNECTED)
            addAction(Intent.ACTION_POWER_DISCONNECTED)
        }

        context.registerReceiver(receiver, filter)
    }

    /**
     * Stop monitoring battery status
     */
    fun stopMonitoring() {
        receiver?.let {
            try {
                context.unregisterReceiver(it)
            } catch (e: Exception) {
                // Ignore if already unregistered
            }
            receiver = null
        }
    }

    /**
     * Get current battery status
     */
    fun getBatteryStatus(): BatteryStatus {
        val intent = context.registerReceiver(null, IntentFilter(Intent.ACTION_BATTERY_CHANGED))
        return intent?.let { parseBatteryIntent(it) } ?: BatteryStatus()
    }

    /**
     * Parse battery status from intent
     */
    private fun parseBatteryIntent(intent: Intent): BatteryStatus {
        val level = intent.getIntExtra(BatteryManager.EXTRA_LEVEL, -1)
        val scale = intent.getIntExtra(BatteryManager.EXTRA_SCALE, -1)
        val percentage = if (level >= 0 && scale > 0) {
            (level * 100) / scale
        } else {
            -1
        }

        val status = intent.getIntExtra(BatteryManager.EXTRA_STATUS, -1)
        val isCharging = status == BatteryManager.BATTERY_STATUS_CHARGING ||
                status == BatteryManager.BATTERY_STATUS_FULL

        val plugged = intent.getIntExtra(BatteryManager.EXTRA_PLUGGED, -1)
        val chargingType = when (plugged) {
            BatteryManager.BATTERY_PLUGGED_AC -> "AC"
            BatteryManager.BATTERY_PLUGGED_USB -> "USB"
            BatteryManager.BATTERY_PLUGGED_WIRELESS -> "Wireless"
            else -> null
        }

        val temperature = intent.getIntExtra(BatteryManager.EXTRA_TEMPERATURE, -1)
        val temperatureCelsius = if (temperature >= 0) temperature / 10f else null

        val health = intent.getIntExtra(BatteryManager.EXTRA_HEALTH, -1)
        val healthStatus = when (health) {
            BatteryManager.BATTERY_HEALTH_GOOD -> BatteryHealth.GOOD
            BatteryManager.BATTERY_HEALTH_OVERHEAT -> BatteryHealth.OVERHEAT
            BatteryManager.BATTERY_HEALTH_DEAD -> BatteryHealth.DEAD
            BatteryManager.BATTERY_HEALTH_OVER_VOLTAGE -> BatteryHealth.OVER_VOLTAGE
            BatteryManager.BATTERY_HEALTH_COLD -> BatteryHealth.COLD
            else -> BatteryHealth.UNKNOWN
        }

        return BatteryStatus(
            level = percentage,
            isCharging = isCharging,
            chargingType = chargingType,
            temperature = temperatureCelsius,
            health = healthStatus
        )
    }

    /**
     * Get battery status as a Flow (alternative to StateFlow)
     */
    fun observeBatteryStatus(): Flow<BatteryStatus> = callbackFlow {
        val receiver = object : BroadcastReceiver() {
            override fun onReceive(context: Context?, intent: Intent?) {
                intent?.let {
                    trySend(parseBatteryIntent(it))
                }
            }
        }

        val filter = IntentFilter().apply {
            addAction(Intent.ACTION_BATTERY_CHANGED)
            addAction(Intent.ACTION_POWER_CONNECTED)
            addAction(Intent.ACTION_POWER_DISCONNECTED)
        }

        // Send initial status
        trySend(getBatteryStatus())

        context.registerReceiver(receiver, filter)

        awaitClose {
            try {
                context.unregisterReceiver(receiver)
            } catch (e: Exception) {
                // Ignore
            }
        }
    }

    /**
     * Check if device is in low battery state
     */
    fun isLowBattery(): Boolean {
        return _batteryStatus.value.level in 0..15
    }

    /**
     * Check if device is in critical battery state
     */
    fun isCriticalBattery(): Boolean {
        return _batteryStatus.value.level in 0..5
    }
}

/**
 * Battery status data class
 */
data class BatteryStatus(
    val level: Int = -1,
    val isCharging: Boolean = false,
    val chargingType: String? = null,
    val temperature: Float? = null,
    val health: BatteryHealth = BatteryHealth.UNKNOWN
) {
    val levelText: String
        get() = if (level >= 0) "$level%" else "Unknown"

    val statusText: String
        get() = when {
            isCharging && chargingType != null -> "Charging ($chargingType)"
            isCharging -> "Charging"
            else -> "Discharging"
        }
}

/**
 * Battery health states
 */
enum class BatteryHealth {
    UNKNOWN,
    GOOD,
    OVERHEAT,
    DEAD,
    OVER_VOLTAGE,
    COLD
}
