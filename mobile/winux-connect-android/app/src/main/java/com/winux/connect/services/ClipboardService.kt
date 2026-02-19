package com.winux.connect.services

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.os.Build
import android.util.Log
import com.winux.connect.protocol.WinuxProtocol
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.*
import javax.inject.Inject
import javax.inject.Singleton

/**
 * Service for clipboard synchronization between phone and desktop
 */
@Singleton
class ClipboardService @Inject constructor(
    @ApplicationContext private val context: Context,
    private val protocol: WinuxProtocol
) {
    private val clipboardManager: ClipboardManager by lazy {
        context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
    }

    private val _clipboardContent = MutableStateFlow<String?>(null)
    val clipboardContent: StateFlow<String?> = _clipboardContent.asStateFlow()

    private var lastSentContent: String? = null
    private var lastReceivedContent: String? = null
    private var isMonitoring = false
    private var monitorJob: Job? = null

    private val scope = CoroutineScope(Dispatchers.Main + SupervisorJob())

    companion object {
        private const val TAG = "WinuxClipboard"
        private const val CLIPBOARD_POLL_INTERVAL = 1000L // 1 second
        private const val MAX_CLIPBOARD_SIZE = 1024 * 1024 // 1MB limit
    }

    /**
     * Start monitoring clipboard changes
     */
    fun startMonitoring() {
        if (isMonitoring) return
        isMonitoring = true

        // Register clipboard listener
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            clipboardManager.addPrimaryClipChangedListener(clipboardListener)
        } else {
            // For older versions, poll clipboard
            startPolling()
        }

        Log.d(TAG, "Clipboard monitoring started")
    }

    /**
     * Stop monitoring clipboard changes
     */
    fun stopMonitoring() {
        isMonitoring = false
        monitorJob?.cancel()

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            clipboardManager.removePrimaryClipChangedListener(clipboardListener)
        }

        Log.d(TAG, "Clipboard monitoring stopped")
    }

    /**
     * Clipboard change listener
     */
    private val clipboardListener = ClipboardManager.OnPrimaryClipChangedListener {
        scope.launch {
            handleClipboardChange()
        }
    }

    /**
     * Start polling clipboard for older Android versions
     */
    private fun startPolling() {
        monitorJob = scope.launch {
            var lastContent: String? = null

            while (isActive && isMonitoring) {
                val content = getClipboardText()
                if (content != lastContent) {
                    lastContent = content
                    handleClipboardChange()
                }
                delay(CLIPBOARD_POLL_INTERVAL)
            }
        }
    }

    /**
     * Handle clipboard content change
     */
    private suspend fun handleClipboardChange() {
        val content = getClipboardText() ?: return

        // Skip if this is content we just received from desktop
        if (content == lastReceivedContent) return

        // Skip if content hasn't changed
        if (content == lastSentContent) return

        // Skip if content is too large
        if (content.length > MAX_CLIPBOARD_SIZE) {
            Log.w(TAG, "Clipboard content too large to sync")
            return
        }

        lastSentContent = content
        _clipboardContent.value = content

        // Send to desktop if connected
        if (protocol.isConnected()) {
            val result = protocol.sendClipboard(content)
            if (result.isSuccess) {
                Log.d(TAG, "Clipboard synced to desktop")
            } else {
                Log.e(TAG, "Failed to sync clipboard: ${result.exceptionOrNull()?.message}")
            }
        }
    }

    /**
     * Get current clipboard text
     */
    fun getClipboardText(): String? {
        return try {
            if (!clipboardManager.hasPrimaryClip()) return null

            val clip = clipboardManager.primaryClip ?: return null
            if (clip.itemCount == 0) return null

            val item = clip.getItemAt(0)
            item.text?.toString()
        } catch (e: Exception) {
            Log.e(TAG, "Failed to get clipboard content", e)
            null
        }
    }

    /**
     * Set clipboard content (from desktop)
     */
    fun setClipboard(content: String) {
        try {
            if (content.length > MAX_CLIPBOARD_SIZE) {
                Log.w(TAG, "Clipboard content from desktop too large")
                return
            }

            lastReceivedContent = content

            val clip = ClipData.newPlainText("Winux Connect", content)
            clipboardManager.setPrimaryClip(clip)

            _clipboardContent.value = content
            Log.d(TAG, "Clipboard set from desktop")
        } catch (e: Exception) {
            Log.e(TAG, "Failed to set clipboard content", e)
        }
    }

    /**
     * Clear clipboard
     */
    fun clearClipboard() {
        try {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
                clipboardManager.clearPrimaryClip()
            } else {
                val clip = ClipData.newPlainText("", "")
                clipboardManager.setPrimaryClip(clip)
            }
            _clipboardContent.value = null
        } catch (e: Exception) {
            Log.e(TAG, "Failed to clear clipboard", e)
        }
    }

    /**
     * Request clipboard content from desktop
     */
    suspend fun requestFromDesktop(): Result<Unit> {
        return withContext(Dispatchers.IO) {
            try {
                if (!protocol.isConnected()) {
                    return@withContext Result.failure(Exception("Not connected"))
                }

                val message = com.winux.connect.data.WinuxMessage(
                    type = com.winux.connect.data.MessageType.CLIPBOARD_REQUEST
                )
                protocol.sendMessage(message)
            } catch (e: Exception) {
                Result.failure(e)
            }
        }
    }
}
