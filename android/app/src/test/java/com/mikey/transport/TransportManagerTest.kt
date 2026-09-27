package com.mikey.transport

import android.bluetooth.BluetoothClass
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class TransportManagerTest {

    @Test
    fun usbFirstThenWifiAndBluetoothLast() {
        assertEquals(listOf(1, 2, 4, 3), listOf(1, 2, 3, 4).sortedBy(::rank))
        assertTrue("no level ranks below every level", rank(0) > rank(3))
    }

    @Test
    fun onlyTheKnownComputerOnceWeHaveOneOtherwiseAnyComputer() {
        val computer = BluetoothClass.Device.Major.COMPUTER
        val phone = BluetoothClass.Device.Major.PHONE

        assertTrue(worthTrying("AA:BB", computer, cachedAddress = null))
        assertFalse(worthTrying("AA:BB", phone, cachedAddress = null))
        assertFalse(worthTrying("AA:BB", null, cachedAddress = null))
        assertTrue(worthTrying("aa:bb", phone, cachedAddress = "AA:BB"))
        assertFalse(worthTrying("CC:DD", computer, cachedAddress = "AA:BB"))
    }
}
