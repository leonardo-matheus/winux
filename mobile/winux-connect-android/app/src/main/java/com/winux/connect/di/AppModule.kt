package com.winux.connect.di

import android.content.Context
import androidx.room.Room
import com.winux.connect.data.DeviceDao
import com.winux.connect.data.DeviceRepository
import com.winux.connect.data.WinuxDatabase
import com.winux.connect.protocol.Discovery
import com.winux.connect.protocol.Encryption
import com.winux.connect.protocol.Pairing
import com.winux.connect.protocol.WinuxProtocol
import com.winux.connect.services.ClipboardService
import com.winux.connect.util.BatteryMonitor
import com.winux.connect.util.QRCodeGenerator
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

/**
 * Dagger Hilt module providing application-wide dependencies
 */
@Module
@InstallIn(SingletonComponent::class)
object AppModule {

    @Provides
    @Singleton
    fun provideDatabase(@ApplicationContext context: Context): WinuxDatabase {
        return Room.databaseBuilder(
            context,
            WinuxDatabase::class.java,
            "winux_database"
        )
            .fallbackToDestructiveMigration()
            .build()
    }

    @Provides
    @Singleton
    fun provideDeviceDao(database: WinuxDatabase): DeviceDao {
        return database.deviceDao()
    }

    @Provides
    @Singleton
    fun provideDeviceRepository(deviceDao: DeviceDao): DeviceRepository {
        return DeviceRepository(deviceDao)
    }

    @Provides
    @Singleton
    fun provideEncryption(): Encryption {
        return Encryption()
    }

    @Provides
    @Singleton
    fun provideWinuxProtocol(encryption: Encryption): WinuxProtocol {
        return WinuxProtocol(encryption)
    }

    @Provides
    @Singleton
    fun provideDiscovery(@ApplicationContext context: Context): Discovery {
        return Discovery(context)
    }

    @Provides
    @Singleton
    fun provideQRCodeGenerator(): QRCodeGenerator {
        return QRCodeGenerator()
    }

    @Provides
    @Singleton
    fun providePairing(
        protocol: WinuxProtocol,
        encryption: Encryption,
        qrCodeGenerator: QRCodeGenerator,
        deviceRepository: DeviceRepository
    ): Pairing {
        return Pairing(protocol, encryption, qrCodeGenerator, deviceRepository)
    }

    @Provides
    @Singleton
    fun provideClipboardService(
        @ApplicationContext context: Context,
        protocol: WinuxProtocol
    ): ClipboardService {
        return ClipboardService(context, protocol)
    }

    @Provides
    @Singleton
    fun provideBatteryMonitor(@ApplicationContext context: Context): BatteryMonitor {
        return BatteryMonitor(context)
    }
}
