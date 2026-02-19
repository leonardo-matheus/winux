package com.winux.connect.protocol

import com.winux.connect.data.*
import kotlinx.coroutines.*
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.*
import java.io.*
import java.net.Socket
import java.net.SocketException
import java.nio.charset.StandardCharsets
import javax.inject.Inject
import javax.inject.Singleton

/**
 * Winux Protocol Handler
 * Manages TCP connection and message exchange with desktop
 */
@Singleton
class WinuxProtocol @Inject constructor(
    private val encryption: Encryption
) {
    private var socket: Socket? = null
    private var writer: BufferedWriter? = null
    private var reader: BufferedReader? = null
    private var connectionJob: Job? = null

    private val _connectionState = MutableStateFlow(ConnectionState.DISCONNECTED)
    val connectionState: StateFlow<ConnectionState> = _connectionState.asStateFlow()

    private val _incomingMessages = Channel<WinuxMessage>(Channel.BUFFERED)
    val incomingMessages: Flow<WinuxMessage> = _incomingMessages.receiveAsFlow()

    private val _connectedDevice = MutableStateFlow<Device?>(null)
    val connectedDevice: StateFlow<Device?> = _connectedDevice.asStateFlow()

    companion object {
        const val PROTOCOL_VERSION = "1.0"
        const val DEFAULT_PORT = 51820
        const val CONNECT_TIMEOUT = 10000
        const val READ_TIMEOUT = 30000
        const val PING_INTERVAL = 15000L
        const val MESSAGE_DELIMITER = "\n"
    }

    /**
     * Connect to a Winux desktop
     */
    suspend fun connect(device: Device): Result<Unit> = withContext(Dispatchers.IO) {
        try {
            if (_connectionState.value == ConnectionState.CONNECTED) {
                disconnect()
            }

            _connectionState.value = ConnectionState.CONNECTING

            socket = Socket().apply {
                soTimeout = READ_TIMEOUT
                connect(java.net.InetSocketAddress(device.ipAddress, device.port), CONNECT_TIMEOUT)
            }

            writer = BufferedWriter(
                OutputStreamWriter(socket!!.getOutputStream(), StandardCharsets.UTF_8)
            )
            reader = BufferedReader(
                InputStreamReader(socket!!.getInputStream(), StandardCharsets.UTF_8)
            )

            _connectedDevice.value = device
            _connectionState.value = ConnectionState.CONNECTED

            // Start listening for messages
            startMessageListener()

            // Send hello message
            sendHello(device)

            Result.success(Unit)
        } catch (e: Exception) {
            _connectionState.value = ConnectionState.ERROR
            disconnect()
            Result.failure(e)
        }
    }

    /**
     * Disconnect from current device
     */
    fun disconnect() {
        connectionJob?.cancel()
        connectionJob = null

        try {
            writer?.close()
            reader?.close()
            socket?.close()
        } catch (e: Exception) {
            // Ignore close exceptions
        }

        writer = null
        reader = null
        socket = null
        _connectedDevice.value = null
        _connectionState.value = ConnectionState.DISCONNECTED
    }

    /**
     * Send a message to the connected device
     */
    suspend fun sendMessage(message: WinuxMessage): Result<Unit> = withContext(Dispatchers.IO) {
        try {
            val json = if (encryption.isEnabled()) {
                encryption.encrypt(message.toJson())
            } else {
                message.toJson()
            }

            writer?.apply {
                write(json + MESSAGE_DELIMITER)
                flush()
            } ?: return@withContext Result.failure(Exception("Not connected"))

            Result.success(Unit)
        } catch (e: Exception) {
            if (e is SocketException) {
                disconnect()
            }
            Result.failure(e)
        }
    }

    /**
     * Send hello message to establish connection
     */
    private suspend fun sendHello(device: Device) {
        val payload = mapOf(
            "protocolVersion" to PROTOCOL_VERSION,
            "deviceName" to android.os.Build.MODEL,
            "deviceType" to "phone",
            "osVersion" to "Android ${android.os.Build.VERSION.RELEASE}",
            "appVersion" to "1.0.0"
        )

        sendMessage(WinuxMessage(type = MessageType.HELLO, payload = payload))
    }

    /**
     * Start listening for incoming messages
     */
    private fun startMessageListener() {
        connectionJob = CoroutineScope(Dispatchers.IO).launch {
            try {
                while (isActive && socket?.isConnected == true) {
                    val line = reader?.readLine()

                    if (line == null) {
                        // Connection closed
                        disconnect()
                        break
                    }

                    val json = if (encryption.isEnabled()) {
                        encryption.decrypt(line)
                    } else {
                        line
                    }

                    WinuxMessage.fromJson(json)?.let { message ->
                        handleMessage(message)
                        _incomingMessages.send(message)
                    }
                }
            } catch (e: Exception) {
                if (e !is CancellationException) {
                    disconnect()
                }
            }
        }
    }

    /**
     * Handle incoming protocol messages
     */
    private suspend fun handleMessage(message: WinuxMessage) {
        when (message.type) {
            MessageType.PING -> {
                sendMessage(WinuxMessage(type = MessageType.PONG))
            }
            MessageType.DISCONNECT -> {
                disconnect()
            }
            else -> {
                // Other messages are passed to listeners
            }
        }
    }

    /**
     * Send notification to PC
     */
    suspend fun sendNotification(notification: NotificationPayload): Result<Unit> {
        val payload = mapOf(
            "id" to notification.id,
            "appName" to notification.appName,
            "appPackage" to notification.appPackage,
            "title" to notification.title,
            "text" to notification.text,
            "icon" to notification.icon,
            "timestamp" to notification.timestamp,
            "actions" to notification.actions.map { mapOf("id" to it.id, "label" to it.label) },
            "isOngoing" to notification.isOngoing,
            "priority" to notification.priority
        )

        return sendMessage(WinuxMessage(type = MessageType.NOTIFICATION, payload = payload))
    }

    /**
     * Send clipboard content to PC
     */
    suspend fun sendClipboard(content: String, mimeType: String = "text/plain"): Result<Unit> {
        val payload = mapOf(
            "content" to content,
            "mimeType" to mimeType
        )

        return sendMessage(WinuxMessage(type = MessageType.CLIPBOARD_CONTENT, payload = payload))
    }

    /**
     * Send media control command
     */
    suspend fun sendMediaControl(action: MediaAction, value: Int? = null): Result<Unit> {
        val payload = mutableMapOf<String, Any?>(
            "action" to action.name.lowercase()
        )
        value?.let { payload["value"] = it }

        return sendMessage(WinuxMessage(type = MessageType.MEDIA_CONTROL, payload = payload))
    }

    /**
     * Send battery status to PC
     */
    suspend fun sendBatteryStatus(level: Int, isCharging: Boolean, chargingType: String? = null): Result<Unit> {
        val payload = mapOf(
            "level" to level,
            "isCharging" to isCharging,
            "chargingType" to chargingType
        )

        return sendMessage(WinuxMessage(type = MessageType.BATTERY_STATUS, payload = payload))
    }

    /**
     * Request file transfer
     */
    suspend fun requestFileTransfer(
        fileName: String,
        fileSize: Long,
        mimeType: String,
        checksum: String? = null
    ): Result<String> {
        val transferId = java.util.UUID.randomUUID().toString()
        val payload = mapOf(
            "transferId" to transferId,
            "fileName" to fileName,
            "fileSize" to fileSize,
            "mimeType" to mimeType,
            "checksum" to checksum
        )

        return sendMessage(WinuxMessage(type = MessageType.FILE_TRANSFER_REQUEST, payload = payload))
            .map { transferId }
    }

    /**
     * Check if connected
     */
    fun isConnected(): Boolean = _connectionState.value == ConnectionState.CONNECTED
}
