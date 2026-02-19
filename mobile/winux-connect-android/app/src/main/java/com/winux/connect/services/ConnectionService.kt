package com.winux.connect.services

import android.app.Notification
import android.app.PendingIntent
import android.app.Service
import android.content.Intent
import android.os.Binder
import android.os.IBinder
import androidx.core.app.NotificationCompat
import com.winux.connect.MainActivity
import com.winux.connect.R
import com.winux.connect.WinuxConnectApp
import com.winux.connect.data.*
import com.winux.connect.protocol.Discovery
import com.winux.connect.protocol.DiscoveryEvent
import com.winux.connect.protocol.WinuxProtocol
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.*
import javax.inject.Inject

/**
 * Foreground service for maintaining connection with Winux desktop
 */
@AndroidEntryPoint
class ConnectionService : Service() {

    @Inject
    lateinit var protocol: WinuxProtocol

    @Inject
    lateinit var discovery: Discovery

    @Inject
    lateinit var deviceRepository: DeviceRepository

    @Inject
    lateinit var clipboardService: ClipboardService

    @Inject
    lateinit var batteryMonitor: com.winux.connect.util.BatteryMonitor

    private val serviceScope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    private val binder = LocalBinder()

    private val _discoveredDevices = MutableStateFlow<List<Device>>(emptyList())
    val discoveredDevices: StateFlow<List<Device>> = _discoveredDevices.asStateFlow()

    private val _isDiscovering = MutableStateFlow(false)
    val isDiscovering: StateFlow<Boolean> = _isDiscovering.asStateFlow()

    private var reconnectJob: Job? = null

    companion object {
        const val NOTIFICATION_ID = 1001
        const val ACTION_CONNECT = "com.winux.connect.ACTION_CONNECT"
        const val ACTION_DISCONNECT = "com.winux.connect.ACTION_DISCONNECT"
        const val ACTION_START_DISCOVERY = "com.winux.connect.ACTION_START_DISCOVERY"
        const val ACTION_STOP_DISCOVERY = "com.winux.connect.ACTION_STOP_DISCOVERY"
        const val EXTRA_DEVICE_ID = "device_id"

        private const val RECONNECT_DELAY = 5000L
        private const val MAX_RECONNECT_ATTEMPTS = 5
    }

    inner class LocalBinder : Binder() {
        fun getService(): ConnectionService = this@ConnectionService
    }

    override fun onBind(intent: Intent?): IBinder = binder

    override fun onCreate() {
        super.onCreate()
        startForeground(NOTIFICATION_ID, createNotification())
        setupMessageHandler()
        setupBatteryMonitor()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            ACTION_CONNECT -> {
                val deviceId = intent.getStringExtra(EXTRA_DEVICE_ID)
                deviceId?.let { connectToDevice(it) }
            }
            ACTION_DISCONNECT -> {
                disconnect()
            }
            ACTION_START_DISCOVERY -> {
                startDiscovery()
            }
            ACTION_STOP_DISCOVERY -> {
                stopDiscovery()
            }
        }

        return START_STICKY
    }

    override fun onDestroy() {
        super.onDestroy()
        serviceScope.cancel()
        discovery.stopDiscovery()
        protocol.disconnect()
    }

    /**
     * Connect to a device by ID
     */
    fun connectToDevice(deviceId: String) {
        serviceScope.launch {
            val device = deviceRepository.getDeviceById(deviceId) ?: return@launch

            val result = protocol.connect(device)
            if (result.isSuccess) {
                deviceRepository.updateLastConnected(deviceId)
                updateNotification()
            }
        }
    }

    /**
     * Connect to a device
     */
    fun connectToDevice(device: Device) {
        serviceScope.launch {
            val result = protocol.connect(device)
            if (result.isSuccess) {
                deviceRepository.updateLastConnected(device.id)
                updateNotification()
            }
        }
    }

    /**
     * Disconnect from current device
     */
    fun disconnect() {
        reconnectJob?.cancel()
        protocol.disconnect()
        updateNotification()
    }

    /**
     * Start device discovery
     */
    fun startDiscovery() {
        if (_isDiscovering.value) return

        _isDiscovering.value = true
        _discoveredDevices.value = emptyList()

        serviceScope.launch {
            discovery.discoverDevices().collect { event ->
                when (event) {
                    is DiscoveryEvent.DeviceFound -> {
                        val currentList = _discoveredDevices.value.toMutableList()
                        val existingIndex = currentList.indexOfFirst {
                            it.ipAddress == event.device.ipAddress
                        }

                        if (existingIndex >= 0) {
                            currentList[existingIndex] = event.device
                        } else {
                            currentList.add(event.device)
                        }

                        _discoveredDevices.value = currentList

                        // Save to repository
                        deviceRepository.upsertDiscoveredDevice(
                            name = event.device.name,
                            hostname = event.device.hostname,
                            ipAddress = event.device.ipAddress,
                            port = event.device.port,
                            deviceType = event.device.deviceType
                        )
                    }
                    is DiscoveryEvent.DeviceLost -> {
                        _discoveredDevices.value = _discoveredDevices.value.filter {
                            it.name != event.serviceName
                        }
                    }
                    is DiscoveryEvent.Stopped -> {
                        _isDiscovering.value = false
                    }
                    is DiscoveryEvent.Error -> {
                        _isDiscovering.value = false
                    }
                    else -> {}
                }
            }
        }
    }

    /**
     * Stop device discovery
     */
    fun stopDiscovery() {
        discovery.stopDiscovery()
        _isDiscovering.value = false
    }

    /**
     * Setup handler for incoming messages
     */
    private fun setupMessageHandler() {
        serviceScope.launch {
            protocol.incomingMessages.collect { message ->
                handleIncomingMessage(message)
            }
        }

        // Monitor connection state
        serviceScope.launch {
            protocol.connectionState.collect { state ->
                when (state) {
                    ConnectionState.DISCONNECTED -> {
                        updateNotification()
                        // Attempt reconnection to paired device
                        scheduleReconnect()
                    }
                    ConnectionState.CONNECTED -> {
                        updateNotification()
                        reconnectJob?.cancel()
                    }
                    else -> {}
                }
            }
        }
    }

    /**
     * Handle incoming protocol messages
     */
    private suspend fun handleIncomingMessage(message: WinuxMessage) {
        when (message.type) {
            MessageType.CLIPBOARD_CONTENT -> {
                val content = message.payload["content"] as? String
                content?.let { clipboardService.setClipboard(it) }
            }
            MessageType.COMMAND_RING, MessageType.COMMAND_FIND_PHONE -> {
                // Trigger ring/find phone
                triggerFindPhone()
            }
            MessageType.NOTIFICATION -> {
                // Show notification from PC
                showPcNotification(message)
            }
            MessageType.BATTERY_REQUEST -> {
                // Send battery status
                val status = batteryMonitor.getBatteryStatus()
                protocol.sendBatteryStatus(
                    level = status.level,
                    isCharging = status.isCharging,
                    chargingType = status.chargingType
                )
            }
            else -> {}
        }
    }

    /**
     * Setup battery monitoring
     */
    private fun setupBatteryMonitor() {
        serviceScope.launch {
            batteryMonitor.batteryStatus.collect { status ->
                if (protocol.isConnected()) {
                    protocol.sendBatteryStatus(
                        level = status.level,
                        isCharging = status.isCharging,
                        chargingType = status.chargingType
                    )
                }
            }
        }
    }

    /**
     * Schedule reconnection attempt
     */
    private fun scheduleReconnect() {
        reconnectJob?.cancel()
        reconnectJob = serviceScope.launch {
            var attempts = 0

            while (attempts < MAX_RECONNECT_ATTEMPTS) {
                delay(RECONNECT_DELAY * (attempts + 1))

                // Try to reconnect to last connected paired device
                deviceRepository.pairedDevices.first().firstOrNull()?.let { device ->
                    val result = protocol.connect(device)
                    if (result.isSuccess) {
                        return@launch
                    }
                }

                attempts++
            }
        }
    }

    /**
     * Trigger find phone functionality
     */
    private fun triggerFindPhone() {
        // TODO: Implement ring/vibrate/flashlight
    }

    /**
     * Show notification from PC
     */
    private fun showPcNotification(message: WinuxMessage) {
        // TODO: Implement showing PC notifications
    }

    /**
     * Create foreground service notification
     */
    private fun createNotification(): Notification {
        val intent = Intent(this, MainActivity::class.java).apply {
            flags = Intent.FLAG_ACTIVITY_SINGLE_TOP
        }

        val pendingIntent = PendingIntent.getActivity(
            this,
            0,
            intent,
            PendingIntent.FLAG_IMMUTABLE
        )

        val state = protocol.connectionState.value
        val device = protocol.connectedDevice.value

        val title = when (state) {
            ConnectionState.CONNECTED -> getString(R.string.status_connected)
            ConnectionState.CONNECTING -> getString(R.string.status_connecting)
            else -> getString(R.string.status_disconnected)
        }

        val text = when (state) {
            ConnectionState.CONNECTED -> device?.name ?: getString(R.string.status_connected)
            ConnectionState.CONNECTING -> getString(R.string.status_connecting_to, device?.name ?: "")
            else -> getString(R.string.tap_to_connect)
        }

        return NotificationCompat.Builder(this, WinuxConnectApp.CHANNEL_CONNECTION)
            .setContentTitle(title)
            .setContentText(text)
            .setSmallIcon(R.drawable.ic_notification)
            .setContentIntent(pendingIntent)
            .setOngoing(true)
            .setSilent(true)
            .build()
    }

    /**
     * Update notification with current state
     */
    private fun updateNotification() {
        val notification = createNotification()
        val notificationManager = getSystemService(NOTIFICATION_SERVICE) as android.app.NotificationManager
        notificationManager.notify(NOTIFICATION_ID, notification)
    }
}
