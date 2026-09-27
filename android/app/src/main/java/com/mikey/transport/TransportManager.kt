package com.mikey.transport

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.net.ConnectivityManager
import android.net.LinkProperties
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import android.os.BatteryManager
import android.os.SystemClock
import android.util.Log
import com.mikey.settings.Settings
import java.io.IOException
import java.net.InetAddress
import java.net.Socket

/**
 * Picks the best way to the PC that works right now, and knows when a better one may have
 * appeared: USB debugging, then USB tethering, then Wi-Fi (connection-levels.md).
 *
 * Probing is driven by events, not timers: a cable plugged in, a network coming or going. The
 * one exception is a cheap tick while a USB cable is in and we're not on a USB level, for the
 * localhost probe and a look for a tether interface. [onCableHint] is told when a cable to a
 * computer has been in for 3 s without either USB level working, so the UI can suggest tethering.
 */
class TransportManager(
    private val context: Context,
    private val settings: Settings,
    private val discovery: Discovery,
    private val onCableHint: (Boolean) -> Unit,
) {
    class Connection(val socket: Socket, val transport: TcpTransport)

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

    private val powerReceiver = object : BroadcastReceiver() {
        override fun onReceive(context: Context, intent: Intent) = readUsbPower()
    }

    fun start() {
        val request = NetworkRequest.Builder().removeCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET).build()
        context.getSystemService(ConnectivityManager::class.java)?.registerNetworkCallback(request, networkCallback)
        val power = IntentFilter().apply {
            addAction(Intent.ACTION_POWER_CONNECTED)
            addAction(Intent.ACTION_POWER_DISCONNECTED)
        }
        context.registerReceiver(powerReceiver, power)
        readUsbPower()
    }

    fun stop() {
        context.getSystemService(ConnectivityManager::class.java)?.unregisterNetworkCallback(networkCallback)
        context.unregisterReceiver(powerReceiver)
        wake()
    }

    /** The best way that works right now. Throws when none does. */
    fun open(): Connection = openFrom(candidates(betterThan = 5))

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
                current <= 1 || plugged == 0L -> Long.MAX_VALUE
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

    private fun openFrom(candidates: Sequence<TcpTransport>): Connection {
        for (transport in candidates) {
            try {
                return Connection(transport.open(), transport)
            } catch (e: IOException) {
                Log.d(TAG, "Not via level ${transport.level} at ${transport.host}: $e")
            }
        }
        throw IOException("No PC reachable")
    }

    // Lazy on purpose: discovery only broadcasts when nothing cheaper worked.
    private fun candidates(betterThan: Int): Sequence<TcpTransport> = sequence {
        val now = SystemClock.elapsedRealtime()
        val enabled = settings.enabledLevels
        fun usable(level: Int) = level < betterThan && level in enabled && synchronized(lock) { deadUntilMs[level] } <= now

        if (usable(1)) yield(TcpTransport.adb())
        val interfaces = discovery.interfaces()
        val found by lazy { discovery.find(settings.pairedPc?.id, interfaces) }
        if (usable(2) && interfaces.any { it.level == 2 }) {
            for (pc in found.filter { it.level == 2 }) yield(TcpTransport.discovered(pc, networkFor(pc.via)))
        }
        if (usable(4)) {
            settings.manualPcAddress?.let { yield(TcpTransport.manual(it, networkFor(interfaces, it))) }
            settings.lastPcAddress?.let { yield(TcpTransport.lastKnown(it, networkFor(interfaces, it))) }
            for (pc in found.filter { it.level == 4 }) yield(TcpTransport.discovered(pc, networkFor(pc.via)))
        }
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

    private fun readUsbPower() {
        val battery = context.registerReceiver(null, IntentFilter(Intent.ACTION_BATTERY_CHANGED))
        val usb = battery?.getIntExtra(BatteryManager.EXTRA_PLUGGED, 0) == BatteryManager.BATTERY_PLUGGED_USB
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
        const val DEAD_MS = 10_000L
        const val EARLY_PROBES_MS = 4_000L
        const val USB_TICK_MS = 5_000L
        const val CABLE_HINT_AFTER_MS = 3_000L
    }
}
