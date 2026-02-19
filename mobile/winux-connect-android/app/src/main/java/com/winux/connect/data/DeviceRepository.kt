package com.winux.connect.data

import androidx.room.*
import kotlinx.coroutines.flow.Flow
import javax.inject.Inject
import javax.inject.Singleton

/**
 * Room Database for Winux Connect
 */
@Database(
    entities = [Device::class],
    version = 1,
    exportSchema = false
)
@TypeConverters(Converters::class)
abstract class WinuxDatabase : RoomDatabase() {
    abstract fun deviceDao(): DeviceDao
}

/**
 * Type converters for Room
 */
class Converters {
    @TypeConverter
    fun fromStringList(value: List<String>): String {
        return value.joinToString(",")
    }

    @TypeConverter
    fun toStringList(value: String): List<String> {
        return if (value.isEmpty()) emptyList() else value.split(",")
    }

    @TypeConverter
    fun fromDeviceType(value: DeviceType): String {
        return value.name
    }

    @TypeConverter
    fun toDeviceType(value: String): DeviceType {
        return DeviceType.valueOf(value)
    }
}

/**
 * Data Access Object for Device
 */
@Dao
interface DeviceDao {
    @Query("SELECT * FROM devices ORDER BY lastConnected DESC")
    fun getAllDevices(): Flow<List<Device>>

    @Query("SELECT * FROM devices WHERE isPaired = 1 ORDER BY lastConnected DESC")
    fun getPairedDevices(): Flow<List<Device>>

    @Query("SELECT * FROM devices WHERE id = :id")
    suspend fun getDeviceById(id: String): Device?

    @Query("SELECT * FROM devices WHERE hostname = :hostname OR ipAddress = :ipAddress LIMIT 1")
    suspend fun getDeviceByHostnameOrIp(hostname: String, ipAddress: String): Device?

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertDevice(device: Device)

    @Update
    suspend fun updateDevice(device: Device)

    @Delete
    suspend fun deleteDevice(device: Device)

    @Query("DELETE FROM devices WHERE id = :id")
    suspend fun deleteDeviceById(id: String)

    @Query("UPDATE devices SET lastConnected = :timestamp WHERE id = :id")
    suspend fun updateLastConnected(id: String, timestamp: Long = System.currentTimeMillis())

    @Query("UPDATE devices SET lastSeen = :timestamp WHERE id = :id")
    suspend fun updateLastSeen(id: String, timestamp: Long = System.currentTimeMillis())

    @Query("UPDATE devices SET isPaired = :isPaired, publicKey = :publicKey WHERE id = :id")
    suspend fun updatePairingStatus(id: String, isPaired: Boolean, publicKey: String)
}

/**
 * Repository for Device operations
 */
@Singleton
class DeviceRepository @Inject constructor(
    private val deviceDao: DeviceDao
) {
    val allDevices: Flow<List<Device>> = deviceDao.getAllDevices()
    val pairedDevices: Flow<List<Device>> = deviceDao.getPairedDevices()

    suspend fun getDeviceById(id: String): Device? = deviceDao.getDeviceById(id)

    suspend fun getDeviceByHostnameOrIp(hostname: String, ipAddress: String): Device? =
        deviceDao.getDeviceByHostnameOrIp(hostname, ipAddress)

    suspend fun addDevice(device: Device) = deviceDao.insertDevice(device)

    suspend fun updateDevice(device: Device) = deviceDao.updateDevice(device)

    suspend fun deleteDevice(device: Device) = deviceDao.deleteDevice(device)

    suspend fun deleteDeviceById(id: String) = deviceDao.deleteDeviceById(id)

    suspend fun updateLastConnected(id: String) = deviceDao.updateLastConnected(id)

    suspend fun updateLastSeen(id: String) = deviceDao.updateLastSeen(id)

    suspend fun markAsPaired(id: String, publicKey: String) =
        deviceDao.updatePairingStatus(id, true, publicKey)

    suspend fun unpairDevice(id: String) =
        deviceDao.updatePairingStatus(id, false, "")

    /**
     * Add or update a discovered device
     */
    suspend fun upsertDiscoveredDevice(
        name: String,
        hostname: String,
        ipAddress: String,
        port: Int,
        deviceType: DeviceType = DeviceType.DESKTOP
    ): Device {
        val existingDevice = getDeviceByHostnameOrIp(hostname, ipAddress)

        return if (existingDevice != null) {
            val updated = existingDevice.copy(
                name = name,
                hostname = hostname,
                ipAddress = ipAddress,
                port = port,
                lastSeen = System.currentTimeMillis()
            )
            updateDevice(updated)
            updated
        } else {
            val newDevice = Device(
                name = name,
                hostname = hostname,
                ipAddress = ipAddress,
                port = port,
                deviceType = deviceType,
                lastSeen = System.currentTimeMillis()
            )
            addDevice(newDevice)
            newDevice
        }
    }
}
