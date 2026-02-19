package com.winux.connect.ui.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.winux.connect.data.*
import com.winux.connect.protocol.WinuxProtocol
import com.winux.connect.services.ClipboardService
import com.winux.connect.util.BatteryMonitor
import com.winux.connect.util.BatteryStatus
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class HomeViewModel @Inject constructor(
    private val protocol: WinuxProtocol,
    private val clipboardService: ClipboardService,
    private val batteryMonitor: BatteryMonitor,
    private val deviceRepository: DeviceRepository
) : ViewModel() {

    val connectionState: StateFlow<ConnectionState> = protocol.connectionState
    val connectedDevice: StateFlow<Device?> = protocol.connectedDevice
    val batteryStatus: StateFlow<BatteryStatus> = batteryMonitor.batteryStatus

    init {
        // Start battery monitoring
        batteryMonitor.startMonitoring()
    }

    fun disconnect() {
        protocol.disconnect()
    }

    fun syncClipboard() {
        viewModelScope.launch {
            clipboardService.requestFromDesktop()
        }
    }

    fun ringPC() {
        viewModelScope.launch {
            val message = WinuxMessage(type = MessageType.COMMAND_RING)
            protocol.sendMessage(message)
        }
    }

    fun requestScreenshot() {
        viewModelScope.launch {
            val message = WinuxMessage(type = MessageType.COMMAND_SCREENSHOT)
            protocol.sendMessage(message)
        }
    }

    fun lockPC() {
        viewModelScope.launch {
            val message = WinuxMessage(type = MessageType.COMMAND_LOCK)
            protocol.sendMessage(message)
        }
    }

    fun sendMediaControl(action: MediaAction) {
        viewModelScope.launch {
            protocol.sendMediaControl(action)
        }
    }

    override fun onCleared() {
        super.onCleared()
        batteryMonitor.stopMonitoring()
    }
}
