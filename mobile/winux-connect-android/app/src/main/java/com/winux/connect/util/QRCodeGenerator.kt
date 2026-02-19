package com.winux.connect.util

import android.graphics.Bitmap
import android.graphics.Color
import com.google.zxing.BarcodeFormat
import com.google.zxing.EncodeHintType
import com.google.zxing.qrcode.QRCodeWriter
import com.google.zxing.qrcode.decoder.ErrorCorrectionLevel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import javax.inject.Inject
import javax.inject.Singleton

/**
 * Utility class for generating QR codes for device pairing
 */
@Singleton
class QRCodeGenerator @Inject constructor() {

    companion object {
        private const val DEFAULT_SIZE = 512
        private const val WINUX_CYAN = 0xFF00D4FF.toInt()
        private const val WINUX_DARK_BG = 0xFF121218.toInt()
    }

    /**
     * Generate a QR code bitmap from the given content
     *
     * @param content The content to encode in the QR code
     * @param size The size (width and height) of the generated bitmap
     * @param useWinuxColors Whether to use Winux brand colors
     * @return The generated QR code as a Bitmap, or null if generation fails
     */
    suspend fun generate(
        content: String,
        size: Int = DEFAULT_SIZE,
        useWinuxColors: Boolean = true
    ): Bitmap? = withContext(Dispatchers.Default) {
        try {
            val hints = mapOf(
                EncodeHintType.ERROR_CORRECTION to ErrorCorrectionLevel.M,
                EncodeHintType.CHARACTER_SET to "UTF-8",
                EncodeHintType.MARGIN to 1
            )

            val writer = QRCodeWriter()
            val bitMatrix = writer.encode(content, BarcodeFormat.QR_CODE, size, size, hints)

            val width = bitMatrix.width
            val height = bitMatrix.height
            val pixels = IntArray(width * height)

            val foregroundColor = if (useWinuxColors) WINUX_CYAN else Color.BLACK
            val backgroundColor = if (useWinuxColors) WINUX_DARK_BG else Color.WHITE

            for (y in 0 until height) {
                for (x in 0 until width) {
                    pixels[y * width + x] = if (bitMatrix[x, y]) {
                        foregroundColor
                    } else {
                        backgroundColor
                    }
                }
            }

            Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888).apply {
                setPixels(pixels, 0, width, 0, 0, width, height)
            }
        } catch (e: Exception) {
            null
        }
    }

    /**
     * Parse QR code data for pairing
     *
     * @param qrData The raw QR code data
     * @return Parsed pairing data or null if invalid
     */
    fun parsePairingData(qrData: String): PairingData? {
        return try {
            if (!qrData.startsWith("winux://pair?")) {
                return null
            }

            val params = qrData.substringAfter("?")
                .split("&")
                .associate {
                    val (key, value) = it.split("=", limit = 2)
                    key to java.net.URLDecoder.decode(value, "UTF-8")
                }

            PairingData(
                pin = params["pin"] ?: return null,
                publicKey = params["key"] ?: return null,
                deviceName = params["device"] ?: "Unknown",
                deviceType = params["type"] ?: "desktop"
            )
        } catch (e: Exception) {
            null
        }
    }
}

/**
 * Data class representing parsed pairing information from QR code
 */
data class PairingData(
    val pin: String,
    val publicKey: String,
    val deviceName: String,
    val deviceType: String
)
