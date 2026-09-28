package com.mikey.transport

import android.Manifest
import android.annotation.SuppressLint
import android.bluetooth.BluetoothClass
import android.bluetooth.BluetoothDevice
import android.bluetooth.BluetoothManager
import android.content.Context
import android.content.pm.PackageManager
import android.os.Build
import java.io.IOException
import java.util.UUID
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import kotlin.concurrent.thread

/**
 * Level 4: an RFCOMM link to a bonded computer that runs Mikey (connection-levels.md). Only
 * already-bonded devices are tried, so there is no scanning and no location permission.
 */
class BluetoothTransport private constructor(private val device: BluetoothDevice) : Transport {
    override val level = 4
    override val host: String = device.address

    /** Connects, giving up after [CONNECT_TIMEOUT_MS]: a BluetoothSocket has no timeout of its own. */
    @SuppressLint("MissingPermission") // Only made by [candidates], which checks the permission first.
    override fun open(): Connection {
        val socket = device.createRfcommSocketToServiceRecord(MIKEY_UUID)
        val done = CountDownLatch(1)
        thread(name = "mikey-bt-connect", isDaemon = true) {
            if (!done.await(CONNECT_TIMEOUT_MS, TimeUnit.MILLISECONDS)) socket.close()
        }
        try {
            socket.connect()
        } catch (e: IOException) {
            socket.close()
            throw e
        } finally {
            done.countDown()
        }
        val input = TimedInput(socket.inputStream) { socket.close() }
        return Connection(level, host, input, socket.outputStream, setReadTimeout = { input.timeoutMs = it }) { socket.close() }
    }

    companion object {
        /** The service the PC publishes (pc/src/transport/bt/mod.rs). */
        val MIKEY_UUID: UUID = UUID.fromString("6d696b65-7900-4000-8000-00805f9b34fb")

        private const val CONNECT_TIMEOUT_MS = 4_000L

        /** Android 12+ asks the user for this; before that it comes with the install. */
        fun hasPermission(context: Context): Boolean =
            Build.VERSION.SDK_INT < Build.VERSION_CODES.S ||
                context.checkSelfPermission(Manifest.permission.BLUETOOTH_CONNECT) == PackageManager.PERMISSION_GRANTED

        /**
         * The bonded computers worth a try: only the one that answered last time once we know it
         * ([cachedAddress]), otherwise every bonded computer. Empty without the permission or with
         * Bluetooth off.
         */
        @SuppressLint("MissingPermission")
        fun candidates(context: Context, cachedAddress: String?): List<BluetoothTransport> {
            if (!hasPermission(context)) return emptyList()
            val adapter = context.getSystemService(BluetoothManager::class.java)?.adapter ?: return emptyList()
            if (!adapter.isEnabled) return emptyList()
            return adapter.bondedDevices.orEmpty()
                .filter { worthTrying(it.address, it.bluetoothClass?.majorDeviceClass, cachedAddress) }
                .map { BluetoothTransport(it) }
        }
    }
}

/** The device that answered before, or, while we don't know one yet, any computer. */
internal fun worthTrying(address: String, majorClass: Int?, cachedAddress: String?): Boolean =
    if (cachedAddress != null) address.equals(cachedAddress, ignoreCase = true) else majorClass == BluetoothClass.Device.Major.COMPUTER
