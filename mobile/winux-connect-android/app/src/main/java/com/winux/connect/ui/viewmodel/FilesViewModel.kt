package com.winux.connect.ui.viewmodel

import android.content.Context
import android.content.Intent
import android.net.Uri
import androidx.core.content.FileProvider
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.winux.connect.data.ConnectionState
import com.winux.connect.protocol.WinuxProtocol
import com.winux.connect.services.FileTransfer
import com.winux.connect.services.FileTransferService
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch
import java.io.File
import javax.inject.Inject

@HiltViewModel
class FilesViewModel @Inject constructor(
    @ApplicationContext private val context: Context,
    private val protocol: WinuxProtocol
) : ViewModel() {

    val connectionState: StateFlow<ConnectionState> = protocol.connectionState

    // In a real app, this would come from FileTransferService
    private val _transfers = MutableStateFlow<List<FileTransfer>>(emptyList())
    val transfers: StateFlow<List<FileTransfer>> = _transfers.asStateFlow()

    fun sendFile(uri: Uri) {
        viewModelScope.launch {
            // Start file transfer service
            val intent = Intent(context, FileTransferService::class.java).apply {
                action = FileTransferService.ACTION_SEND_FILE
                putExtra(FileTransferService.EXTRA_FILE_URI, uri.toString())
            }
            context.startService(intent)
        }
    }

    fun cancelTransfer(transferId: String) {
        viewModelScope.launch {
            // TODO: Implement transfer cancellation
        }
    }

    fun openFile(path: String) {
        try {
            val file = File(path)
            if (!file.exists()) return

            val uri = FileProvider.getUriForFile(
                context,
                "${context.packageName}.fileprovider",
                file
            )

            val intent = Intent(Intent.ACTION_VIEW).apply {
                setDataAndType(uri, getMimeType(file))
                addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            }

            context.startActivity(intent)
        } catch (e: Exception) {
            // Handle error
        }
    }

    private fun getMimeType(file: File): String {
        val extension = file.extension.lowercase()
        return when (extension) {
            "jpg", "jpeg" -> "image/jpeg"
            "png" -> "image/png"
            "gif" -> "image/gif"
            "pdf" -> "application/pdf"
            "mp4" -> "video/mp4"
            "mp3" -> "audio/mpeg"
            "txt" -> "text/plain"
            "zip" -> "application/zip"
            else -> "application/octet-stream"
        }
    }
}
