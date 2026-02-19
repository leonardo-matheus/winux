package com.winux.connect.protocol

import com.winux.connect.data.*
import com.winux.connect.util.QRCodeGenerator
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import java.security.SecureRandom
import javax.inject.Inject
import javax.inject.Singleton

/**
 * Handles device pairing for Winux Connect
 */
@Singleton
class Pairing @Inject constructor(
    private val protocol: WinuxProtocol,
    private val encryption: Encryption,
    private val qrCodeGenerator: QRCodeGenerator,
    private val deviceRepository: DeviceRepository
) {
    private val _pairingState = MutableStateFlow<PairingState>(PairingState.Idle)
    val pairingState: StateFlow<PairingState> = _pairingState.asStateFlow()

    private var currentDevice: Device? = null
    private var currentPin: String? = null

    companion object {
        const val PIN_LENGTH = 6
        private val secureRandom = SecureRandom()
    }

    /**
     * Generate a random PIN for pairing
     */
    fun generatePin(): String {
        val pin = StringBuilder()
        repeat(PIN_LENGTH) {
            pin.append(secureRandom.nextInt(10))
        }
        return pin.toString().also { currentPin = it }
    }

    /**
     * Generate QR code data for pairing
     */
    fun generateQRCodeData(): String {
        val pin = currentPin ?: generatePin()
        val publicKey = encryption.getPublicKey()

        return buildString {
            append("winux://pair?")
            append("pin=$pin")
            append("&key=$publicKey")
            append("&device=${android.os.Build.MODEL}")
            append("&type=phone")
        }
    }

    /**
     * Generate QR code bitmap
     */
    suspend fun generateQRCodeBitmap(size: Int = 512): android.graphics.Bitmap? {
        val data = generateQRCodeData()
        return qrCodeGenerator.generate(data, size)
    }

    /**
     * Start pairing process with a device
     */
    suspend fun startPairing(device: Device): Result<Unit> {
        currentDevice = device
        _pairingState.value = PairingState.WaitingForConnection

        // Connect to device
        val connectResult = protocol.connect(device)
        if (connectResult.isFailure) {
            _pairingState.value = PairingState.Failed(
                connectResult.exceptionOrNull()?.message ?: "Connection failed"
            )
            return connectResult
        }

        // Generate keys if needed
        if (!encryption.hasKeys()) {
            encryption.generateKeyPair()
        }

        // Send pairing request
        _pairingState.value = PairingState.SendingRequest

        val payload = mapOf(
            "publicKey" to encryption.getPublicKey(),
            "pin" to currentPin,
            "deviceName" to android.os.Build.MODEL,
            "deviceType" to "phone"
        )

        val message = WinuxMessage(type = MessageType.PAIR_REQUEST, payload = payload)
        val sendResult = protocol.sendMessage(message)

        if (sendResult.isFailure) {
            _pairingState.value = PairingState.Failed(
                sendResult.exceptionOrNull()?.message ?: "Failed to send pairing request"
            )
            return sendResult
        }

        _pairingState.value = PairingState.WaitingForConfirmation
        return Result.success(Unit)
    }

    /**
     * Handle pairing response from desktop
     */
    suspend fun handlePairingResponse(message: WinuxMessage): Boolean {
        when (message.type) {
            MessageType.PAIR_RESPONSE -> {
                val accepted = message.payload["accepted"] as? Boolean ?: false

                if (accepted) {
                    val peerPublicKey = message.payload["publicKey"] as? String ?: ""

                    if (peerPublicKey.isNotEmpty()) {
                        // Setup encryption with peer key
                        encryption.setPeerPublicKey(peerPublicKey)

                        // Send confirmation
                        val confirmMessage = WinuxMessage(
                            type = MessageType.PAIR_CONFIRM,
                            payload = mapOf("success" to true)
                        )
                        protocol.sendMessage(confirmMessage)

                        // Save paired device
                        currentDevice?.let { device ->
                            deviceRepository.markAsPaired(device.id, peerPublicKey)
                        }

                        _pairingState.value = PairingState.Paired
                        return true
                    }
                }

                val reason = message.payload["reason"] as? String ?: "Pairing rejected"
                _pairingState.value = PairingState.Failed(reason)
                return false
            }
            else -> return false
        }
    }

    /**
     * Verify PIN entered by user
     */
    fun verifyPin(enteredPin: String): Boolean {
        return currentPin == enteredPin
    }

    /**
     * Cancel current pairing process
     */
    fun cancelPairing() {
        currentDevice = null
        currentPin = null
        _pairingState.value = PairingState.Idle
        protocol.disconnect()
    }

    /**
     * Reset pairing state
     */
    fun reset() {
        currentDevice = null
        currentPin = null
        _pairingState.value = PairingState.Idle
    }

    /**
     * Unpair a device
     */
    suspend fun unpairDevice(device: Device) {
        deviceRepository.unpairDevice(device.id)

        if (protocol.connectedDevice.value?.id == device.id) {
            val message = WinuxMessage(type = MessageType.DISCONNECT)
            protocol.sendMessage(message)
            protocol.disconnect()
        }
    }
}

/**
 * Pairing state
 */
sealed class PairingState {
    data object Idle : PairingState()
    data object WaitingForConnection : PairingState()
    data object SendingRequest : PairingState()
    data object WaitingForConfirmation : PairingState()
    data object VerifyingPin : PairingState()
    data object Paired : PairingState()
    data class Failed(val reason: String) : PairingState()
}
