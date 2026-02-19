package com.winux.connect.services

import android.app.Notification
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.ContentResolver
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Binder
import android.os.Environment
import android.os.IBinder
import android.provider.OpenableColumns
import android.util.Log
import androidx.core.app.NotificationCompat
import com.winux.connect.MainActivity
import com.winux.connect.R
import com.winux.connect.WinuxConnectApp
import com.winux.connect.data.*
import com.winux.connect.protocol.WinuxProtocol
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.*
import java.io.*
import java.net.ServerSocket
import java.net.Socket
import java.security.MessageDigest
import javax.inject.Inject

/**
 * Service for handling file transfers between phone and desktop
 */
@AndroidEntryPoint
class FileTransferService : Service() {

    @Inject
    lateinit var protocol: WinuxProtocol

    private val serviceScope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    private val binder = LocalBinder()

    private val _transfers = MutableStateFlow<List<FileTransfer>>(emptyList())
    val transfers: StateFlow<List<FileTransfer>> = _transfers.asStateFlow()

    private var serverSocket: ServerSocket? = null
    private var isServerRunning = false

    companion object {
        private const val TAG = "WinuxFileTransfer"
        const val NOTIFICATION_ID = 1002
        const val FILE_TRANSFER_PORT = 51821
        const val BUFFER_SIZE = 8192
        const val ACTION_SEND_FILE = "com.winux.connect.ACTION_SEND_FILE"
        const val EXTRA_FILE_URI = "file_uri"
    }

    inner class LocalBinder : Binder() {
        fun getService(): FileTransferService = this@FileTransferService
    }

    override fun onBind(intent: Intent?): IBinder = binder

    override fun onCreate() {
        super.onCreate()
        startFileServer()
        observeIncomingTransfers()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            ACTION_SEND_FILE -> {
                val uriString = intent.getStringExtra(EXTRA_FILE_URI)
                uriString?.let { Uri.parse(it) }?.let { uri ->
                    sendFile(uri)
                }
            }
        }

        return START_STICKY
    }

    override fun onDestroy() {
        super.onDestroy()
        serviceScope.cancel()
        stopFileServer()
    }

    /**
     * Start file server to receive files
     */
    private fun startFileServer() {
        if (isServerRunning) return

        serviceScope.launch {
            try {
                serverSocket = ServerSocket(FILE_TRANSFER_PORT)
                isServerRunning = true
                Log.d(TAG, "File server started on port $FILE_TRANSFER_PORT")

                while (isActive && isServerRunning) {
                    try {
                        val clientSocket = serverSocket?.accept() ?: break
                        launch { handleIncomingFile(clientSocket) }
                    } catch (e: Exception) {
                        if (isServerRunning) {
                            Log.e(TAG, "Error accepting connection", e)
                        }
                    }
                }
            } catch (e: Exception) {
                Log.e(TAG, "Failed to start file server", e)
            }
        }
    }

    /**
     * Stop file server
     */
    private fun stopFileServer() {
        isServerRunning = false
        serverSocket?.close()
        serverSocket = null
    }

    /**
     * Handle incoming file transfer
     */
    private suspend fun handleIncomingFile(socket: Socket) {
        try {
            val inputStream = socket.getInputStream()
            val dataInput = DataInputStream(inputStream)

            // Read file metadata
            val transferId = dataInput.readUTF()
            val fileName = dataInput.readUTF()
            val fileSize = dataInput.readLong()
            val mimeType = dataInput.readUTF()

            Log.d(TAG, "Receiving file: $fileName ($fileSize bytes)")

            // Create transfer record
            val transfer = FileTransfer(
                id = transferId,
                fileName = fileName,
                fileSize = fileSize,
                mimeType = mimeType,
                direction = TransferDirection.INCOMING,
                state = TransferState.IN_PROGRESS
            )
            addTransfer(transfer)
            showTransferNotification(transfer)

            // Determine download location
            val downloadsDir = Environment.getExternalStoragePublicDirectory(
                Environment.DIRECTORY_DOWNLOADS
            )
            val targetFile = File(downloadsDir, "Winux/$fileName")
            targetFile.parentFile?.mkdirs()

            // Receive file with progress
            val outputStream = FileOutputStream(targetFile)
            val buffer = ByteArray(BUFFER_SIZE)
            var totalBytesReceived = 0L

            while (totalBytesReceived < fileSize) {
                val bytesToRead = minOf(BUFFER_SIZE.toLong(), fileSize - totalBytesReceived).toInt()
                val bytesRead = inputStream.read(buffer, 0, bytesToRead)

                if (bytesRead == -1) break

                outputStream.write(buffer, 0, bytesRead)
                totalBytesReceived += bytesRead

                // Update progress
                val progress = ((totalBytesReceived * 100) / fileSize).toInt()
                updateTransferProgress(transferId, progress, totalBytesReceived)
            }

            outputStream.close()
            socket.close()

            // Mark transfer as complete
            updateTransferState(transferId, TransferState.COMPLETED, targetFile.absolutePath)
            updateTransferNotification(transfer.copy(
                state = TransferState.COMPLETED,
                localPath = targetFile.absolutePath
            ))

            Log.d(TAG, "File received: ${targetFile.absolutePath}")

        } catch (e: Exception) {
            Log.e(TAG, "Error receiving file", e)
        }
    }

    /**
     * Send a file to the connected desktop
     */
    fun sendFile(uri: Uri) {
        serviceScope.launch {
            try {
                if (!protocol.isConnected()) {
                    Log.e(TAG, "Cannot send file: not connected")
                    return@launch
                }

                val contentResolver = applicationContext.contentResolver

                // Get file info
                val fileInfo = getFileInfo(contentResolver, uri) ?: run {
                    Log.e(TAG, "Failed to get file info")
                    return@launch
                }

                // Request transfer
                val result = protocol.requestFileTransfer(
                    fileName = fileInfo.name,
                    fileSize = fileInfo.size,
                    mimeType = fileInfo.mimeType
                )

                if (result.isFailure) {
                    Log.e(TAG, "Failed to request file transfer")
                    return@launch
                }

                val transferId = result.getOrThrow()

                // Create transfer record
                val transfer = FileTransfer(
                    id = transferId,
                    fileName = fileInfo.name,
                    fileSize = fileInfo.size,
                    mimeType = fileInfo.mimeType,
                    direction = TransferDirection.OUTGOING,
                    state = TransferState.PENDING,
                    localPath = uri.toString()
                )
                addTransfer(transfer)
                showTransferNotification(transfer)

                // Wait for acceptance (handled by message listener)

            } catch (e: Exception) {
                Log.e(TAG, "Error initiating file transfer", e)
            }
        }
    }

    /**
     * Actually send file data after acceptance
     */
    suspend fun sendFileData(transferId: String, targetIp: String, targetPort: Int) {
        val transfer = _transfers.value.find { it.id == transferId } ?: return

        try {
            val uri = Uri.parse(transfer.localPath)
            val contentResolver = applicationContext.contentResolver
            val inputStream = contentResolver.openInputStream(uri) ?: return

            updateTransferState(transferId, TransferState.IN_PROGRESS)

            // Connect to desktop
            val socket = Socket(targetIp, targetPort)
            val outputStream = socket.getOutputStream()
            val dataOutput = DataOutputStream(outputStream)

            // Send metadata
            dataOutput.writeUTF(transfer.id)
            dataOutput.writeUTF(transfer.fileName)
            dataOutput.writeLong(transfer.fileSize)
            dataOutput.writeUTF(transfer.mimeType)

            // Send file data with progress
            val buffer = ByteArray(BUFFER_SIZE)
            var totalBytesSent = 0L

            while (true) {
                val bytesRead = inputStream.read(buffer)
                if (bytesRead == -1) break

                outputStream.write(buffer, 0, bytesRead)
                totalBytesSent += bytesRead

                val progress = ((totalBytesSent * 100) / transfer.fileSize).toInt()
                updateTransferProgress(transferId, progress, totalBytesSent)
            }

            inputStream.close()
            socket.close()

            updateTransferState(transferId, TransferState.COMPLETED)
            Log.d(TAG, "File sent: ${transfer.fileName}")

        } catch (e: Exception) {
            Log.e(TAG, "Error sending file", e)
            updateTransferState(transferId, TransferState.FAILED, errorMessage = e.message)
        }
    }

    /**
     * Observe incoming transfer messages
     */
    private fun observeIncomingTransfers() {
        serviceScope.launch {
            protocol.incomingMessages.collect { message ->
                when (message.type) {
                    MessageType.FILE_TRANSFER_ACCEPT -> {
                        val transferId = message.payload["transferId"] as? String
                        val targetIp = message.payload["ip"] as? String
                        val targetPort = (message.payload["port"] as? Number)?.toInt()

                        if (transferId != null && targetIp != null && targetPort != null) {
                            sendFileData(transferId, targetIp, targetPort)
                        }
                    }
                    MessageType.FILE_TRANSFER_REJECT -> {
                        val transferId = message.payload["transferId"] as? String
                        val reason = message.payload["reason"] as? String
                        transferId?.let {
                            updateTransferState(it, TransferState.FAILED, errorMessage = reason)
                        }
                    }
                    else -> {}
                }
            }
        }
    }

    /**
     * Get file info from URI
     */
    private fun getFileInfo(contentResolver: ContentResolver, uri: Uri): FileInfo? {
        return try {
            val cursor = contentResolver.query(uri, null, null, null, null)
            cursor?.use {
                if (it.moveToFirst()) {
                    val nameIndex = it.getColumnIndex(OpenableColumns.DISPLAY_NAME)
                    val sizeIndex = it.getColumnIndex(OpenableColumns.SIZE)

                    val name = if (nameIndex >= 0) it.getString(nameIndex) else "file"
                    val size = if (sizeIndex >= 0) it.getLong(sizeIndex) else 0L
                    val mimeType = contentResolver.getType(uri) ?: "application/octet-stream"

                    FileInfo(name, size, mimeType)
                } else null
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error getting file info", e)
            null
        }
    }

    private fun addTransfer(transfer: FileTransfer) {
        _transfers.value = _transfers.value + transfer
    }

    private fun updateTransferProgress(transferId: String, progress: Int, bytesTransferred: Long) {
        _transfers.value = _transfers.value.map {
            if (it.id == transferId) {
                it.copy(progress = progress, bytesTransferred = bytesTransferred)
            } else it
        }
    }

    private fun updateTransferState(
        transferId: String,
        state: TransferState,
        localPath: String? = null,
        errorMessage: String? = null
    ) {
        _transfers.value = _transfers.value.map {
            if (it.id == transferId) {
                it.copy(
                    state = state,
                    localPath = localPath ?: it.localPath,
                    errorMessage = errorMessage
                )
            } else it
        }
    }

    private fun showTransferNotification(transfer: FileTransfer) {
        val intent = Intent(this, MainActivity::class.java)
        val pendingIntent = PendingIntent.getActivity(
            this, 0, intent, PendingIntent.FLAG_IMMUTABLE
        )

        val title = when (transfer.direction) {
            TransferDirection.INCOMING -> getString(R.string.receiving_file)
            TransferDirection.OUTGOING -> getString(R.string.sending_file)
        }

        val notification = NotificationCompat.Builder(this, WinuxConnectApp.CHANNEL_FILE_TRANSFER)
            .setContentTitle(title)
            .setContentText(transfer.fileName)
            .setSmallIcon(R.drawable.ic_notification)
            .setProgress(100, 0, true)
            .setContentIntent(pendingIntent)
            .setOngoing(true)
            .build()

        val notificationManager = getSystemService(NOTIFICATION_SERVICE) as NotificationManager
        notificationManager.notify(transfer.id.hashCode(), notification)
    }

    private fun updateTransferNotification(transfer: FileTransfer) {
        val notificationManager = getSystemService(NOTIFICATION_SERVICE) as NotificationManager

        if (transfer.state == TransferState.COMPLETED || transfer.state == TransferState.FAILED) {
            notificationManager.cancel(transfer.id.hashCode())

            // Show completion notification
            val title = when {
                transfer.state == TransferState.COMPLETED && transfer.direction == TransferDirection.INCOMING ->
                    getString(R.string.file_received)
                transfer.state == TransferState.COMPLETED && transfer.direction == TransferDirection.OUTGOING ->
                    getString(R.string.file_sent)
                else -> getString(R.string.transfer_failed)
            }

            val notification = NotificationCompat.Builder(this, WinuxConnectApp.CHANNEL_FILE_TRANSFER)
                .setContentTitle(title)
                .setContentText(transfer.fileName)
                .setSmallIcon(R.drawable.ic_notification)
                .setAutoCancel(true)
                .build()

            notificationManager.notify(transfer.id.hashCode() + 10000, notification)
        }
    }
}

data class FileInfo(
    val name: String,
    val size: Long,
    val mimeType: String
)

data class FileTransfer(
    val id: String,
    val fileName: String,
    val fileSize: Long,
    val mimeType: String,
    val direction: TransferDirection,
    val state: TransferState,
    val progress: Int = 0,
    val bytesTransferred: Long = 0,
    val localPath: String? = null,
    val errorMessage: String? = null
)

enum class TransferDirection {
    INCOMING,
    OUTGOING
}

enum class TransferState {
    PENDING,
    IN_PROGRESS,
    COMPLETED,
    FAILED,
    CANCELLED
}
