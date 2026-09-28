package com.mikey.transport

import android.bluetooth.BluetoothAdapter
import android.bluetooth.BluetoothDevice
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.net.ConnectivityManager
import android.net.LinkProperties
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import android.os.SystemClock
import android.util.Log
import com.mikey.settings.Settings
import java.io.IOException
import java.net.InetAddress

/**
 * Picks the best way to the PC that works right now, and knows when a better one may have
 * appeared. In order: USB debugging, USB tethering, Wi-Fi, and Bluetooth as the last resort
 * (connection-levels.md).
 *
 * Probing is driven by events, not timers: a cable plugged in, a network coming or going,
 * Bluetooth turned on or a new pairing. The one exception is a cheap tick while a USB cable is
 * in and we're not on a USB level, for the localhost probe and a look for a tether interface.
 * [onCableHint] is told when a cable to a computer has been in for 3 s without either USB level
 * working, so the UI can suggest tethering.
 */
class TransportManager(
    private val context: Context,
    private val settings: Settings,
    private val discovery: Discovery,
    private val onCableHint: (Boolean) -> Unit,
) {
    private val lock = Object()

    /** Which Network each interface belongs to, so sockets can be pinned to it. Guarded by [lock]. */
    private val networksByInterface = HashMap<String, Network>()

    /** Per level: don't try it again before this time (elapsedRealtime). Guarded by [lock]. */
    private val deadUntilMs = LongArray(5)

    /** When a USB cable to a computer was plugged in (elapsedRealtime), or 0 while there is none. */
    @Volatile private var usbPluggedAtMs = 0L

    private var cableHintOn = false

    private val networkCallback = object : ConnectivityManager.NetworkCallback() {
        override fun onLinkPropertiesChanged(network: Network, properties: LinkProperties) {
            val name = properties.interfaceName ?: return
            synchronized(lock) {
                networksByInterface[name] = network
                lock.notifyAll()
            }
        }

        override fun onLost(network: Network) {
            synchronized(lock) {
                networksByInterface.values.remove(network)
                lock.notifyAll()
            }
        }
    }

    /** USB and Bluetooth events. Any of them is a reason to look again. */
    private val eventReceiver = object : BroadcastReceiver() {
        override fun onReceive(context: Context, intent: Intent) {
            when (intent.action) {
                ACTION_USB_STATE -> readUsbState(intent)
                else -> wake()
            }
        }
    }

    fun start() {
        val request = NetworkRequest.Builder().removeCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET).build()
        context.getSystemService(ConnectivityManager::class.java)?.registerNetworkCallback(request, networkCallback)
        val events = IntentFilter().apply {
            addAction(ACTION_USB_STATE)
            addAction(BluetoothAdapter.ACTION_STATE_CHANGED)
            addAction(BluetoothDevice.ACTION_BOND_STATE_CHANGED)
        }
        // USB_STATE is sticky, so registering hands us where the cable stands right now.
        readUsbState(context.registerReceiver(eventReceiver, events))
    }

    fun stop() {
        context.getSystemService(ConnectivityManager::class.java)?.unregisterNetworkCallback(networkCallback)
        context.unregisterReceiver(eventReceiver)
        wake()
    }

    /** The best way that works right now. Throws when none does. */
    fun open(): Connection = openFrom(candidates(betterThan = NONE))

    /** A way better than [level]. Throws when there is none. */
    fun openBetterThan(level: Int): Connection = openFrom(candidates(betterThan = level))

    /** A level that just died isn't tried again for 10 s, so a bad cable can't cause flapping. */
    fun markDead(level: Int) {
        synchronized(lock) { deadUntilMs[level] = SystemClock.elapsedRealtime() + DEAD_MS }
    }

    /**
     * Blocks until something suggests a level better than [current] may have appeared, or until
     * [wake]. While a USB cable is in, that is a tick: every 1 s for the first 4 s after plugging
     * in (the PC's adb reverse takes a moment), then every 5 s.
     */
    fun waitForBetterChance(current: Int) {
        synchronized(lock) {
            val plugged = usbPluggedAtMs
            val timeout = when {
                rank(current) == 0 || plugged == 0L -> Long.MAX_VALUE
                SystemClock.elapsedRealtime() - plugged < EARLY_PROBES_MS -> 1_000L
                else -> USB_TICK_MS
            }
            try {
                lock.wait(timeout)
            } catch (e: InterruptedException) {
                // The caller checks whether it should still be running.
            }
        }
    }

    fun wake() {
        synchronized(lock) { lock.notifyAll() }
    }

    /**
     * Tells the manager which level we're on now (0 = none), so it can decide about the cable hint:
     * a cable to a computer in for 3 s or more, and no USB level working.
     */
    fun noteLevel(level: Int) {
        val plugged = usbPluggedAtMs
        val now = SystemClock.elapsedRealtime()
        // A USB level that just died is only resting, so that is no reason to suggest tethering.
        val resting = synchronized(lock) { deadUntilMs[1] > now || deadUntilMs[2] > now }
        reportCableHint(level !in 1..2 && !resting && plugged != 0L && now - plugged >= CABLE_HINT_AFTER_MS)
    }

    private fun openFrom(candidates: Sequence<Transport>): Connection {
        for (transport in candidates) {
            try {
                return transport.open()
            } catch (e: IOException) {
                Log.d(TAG, "Not via level ${transport.level} at ${transport.host}: $e")
            }
        }
        throw IOException("No PC reachable")
    }

    // Lazy on purpose: discovery only broadcasts, and Bluetooth only connects, when nothing cheaper worked.
    private fun candidates(betterThan: Int): Sequence<Transport> = sequence {
        val now = SystemClock.elapsedRealtime()
        val enabled = settings.enabledLevels
        fun usable(level: Int) = rank(level) < rank(betterThan) && level in enabled && synchronized(lock) { deadUntilMs[level] } <= now

        if (usable(1)) yield(TcpTransport.adb())
        val interfaces = discovery.interfaces()
        val found by lazy { discovery.find(settings.pairedPc?.id, interfaces) }
        if (usable(2) && interfaces.any { it.level == 2 }) {
            for (pc in found.filter { it.level == 2 }) yield(TcpTransport.discovered(pc, networkFor(pc.via)))
        }
        if (usable(3)) {
            settings.manualPcAddress?.let { yield(TcpTransport.manual(it, networkFor(interfaces, it))) }
            settings.lastPcAddress?.let { yield(TcpTransport.lastKnown(it, networkFor(interfaces, it))) }
            for (pc in found.filter { it.level == 3 }) yield(TcpTransport.discovered(pc, networkFor(pc.via)))
        }
        if (usable(4)) yieldAll(BluetoothTransport.candidates(context, settings.pcBtAddress))
    }

    private fun networkFor(via: NetInterface?): Network? =
        via?.let { synchronized(lock) { networksByInterface[it.name] } }

    /** The network whose interface owns the subnet [address] is in, if any. */
    private fun networkFor(interfaces: List<NetInterface>, address: String): Network? {
        val ip = try {
            InetAddress.getByName(address) // A literal address: no lookup happens.
        } catch (e: IOException) {
            return null
        }
        return networkFor(interfaces.firstOrNull { it.contains(ip) })
    }

    /**
     * Whether a USB cable to a computer is in, from USB_STATE. Not from the charger type: many
     * laptop ports (USB-C, charging ports) make the battery report a wall charger.
     */
    private fun readUsbState(intent: Intent?) {
        val usb = intent?.getBooleanExtra(USB_CONNECTED, false) == true
        synchronized(lock) {
            if (usb && usbPluggedAtMs == 0L) usbPluggedAtMs = SystemClock.elapsedRealtime()
            if (!usb) usbPluggedAtMs = 0L
            lock.notifyAll()
        }
        if (!usb) reportCableHint(false)
    }

    private fun reportCableHint(on: Boolean) {
        synchronized(lock) {
            if (on == cableHintOn) return
            cableHintOn = on
        }
        Log.i(TAG, if (on) "USB cable in, but no USB level works: suggest tethering" else "USB cable hint cleared")
        onCableHint(on)
    }

    private companion object {
        const val TAG = "TransportManager"

        /**
         * Sent on plug, unplug and USB mode changes (debugging, tethering). Hidden in UsbManager,
         * but every Android version sends it and the system's own USB settings rely on it.
         */
        const val ACTION_USB_STATE = "android.hardware.usb.action.USB_STATE"
        const val USB_CONNECTED = "connected"

        /** "Better than nothing": every level counts. */
        const val NONE = 0
        const val DEAD_MS = 10_000L
        const val EARLY_PROBES_MS = 4_000L
        const val USB_TICK_MS = 5_000L
        const val CABLE_HINT_AFTER_MS = 3_000L
    }
}

/**
 * Where a level stands in the order of preference, 0 being best. The levels are numbered in that
 * order: USB debugging, USB tethering, Wi-Fi, then Bluetooth, which comes last because it carries
 * narrower audio and no video. Anything else (like 0 for "no level") ranks below them all.
 */
internal fun rank(level: Int): Int = if (level in 1..4) level - 1 else 4
