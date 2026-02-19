package com.winux.connect.ui.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.winux.connect.data.*
import com.winux.connect.protocol.Discovery
import com.winux.connect.protocol.DiscoveryEvent
import com.winux.connect.protocol.WinuxProtocol
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class DevicesViewModel @Inject constructor(
    private val protocol: WinuxProtocol,
    private val discovery: Discovery,
    private val deviceRepository: DeviceRepository
) : ViewModel() {

    val connectionState: StateFlow<ConnectionState> = protocol.connectionState

    val connectedDeviceId: StateFlow<String?> = protocol.connectedDevice
        .map { it?.id }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), null)

    val pairedDevices: StateFlow<List<Device>> = deviceRepository.pairedDevices
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), emptyList())

    private val _discoveredDevices = MutableStateFlow<List<Device>>(emptyList())
    val discoveredDevices: StateFlow<List<Device>> = _discoveredDevices.asStateFlow()

    private val _isDiscovering = MutableStateFlow(false)
    val isDiscovering: StateFlow<Boolean> = _isDiscovering.asStateFlow()

    fun startDiscovery() {
        if (_isDiscovering.value) return

        _isDiscovering.value = true
        _discoveredDevices.value = emptyList()

        viewModelScope.launch {
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

    fun stopDiscovery() {
        discovery.stopDiscovery()
        _isDiscovering.value = false
    }

    fun connectToDevice(device: Device) {
        viewModelScope.launch {
            protocol.connect(device)
        }
    }

    fun disconnect() {
        protocol.disconnect()
    }

    fun unpairDevice(device: Device) {
        viewModelScope.launch {
            deviceRepository.unpairDevice(device.id)
        }
    }
}
