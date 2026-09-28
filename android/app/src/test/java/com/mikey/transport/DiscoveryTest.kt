package com.mikey.transport

import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.InetAddress
import kotlin.concurrent.thread

class DiscoveryTest {
    private val loopback: InetAddress = InetAddress.getLoopbackAddress()
    private val reply = "MIKEY!1".toByteArray() + ByteArray(16) { 2 } + byteArrayOf(0x1D, 0xE5.toByte(), 1, 8) + "Mikey-PC".toByteArray()

    /** Needs UDP 7654 free on this machine: a fake PC answers on loopback, twice, to check duplicates are dropped. */
    @Test
    fun findReturnsTheAnsweringPcAndStopsEarlyForThePairedOne() {
        val pcId = "02".repeat(16)
        val responder = DatagramSocket(7654, loopback)
        thread {
            val probe = DatagramPacket(ByteArray(64), 64)
            responder.receive(probe)
            assertArrayEquals(probePayload("00".repeat(16), "Pixel 7"), probe.data.copyOf(probe.length))
            repeat(2) { responder.send(DatagramPacket(reply, reply.size, probe.socketAddress)) }
            responder.close()
        }
        val lo = NetInterface("lo", loopback, 8, loopback)
        val started = System.nanoTime()

        val found = Discovery("00".repeat(16), "Pixel 7").find(preferredPcId = pcId, interfaces = listOf(lo))

        val elapsedMs = (System.nanoTime() - started) / 1_000_000
        assertEquals(1, found.size)
        assertEquals(pcId, found[0].id)
        assertEquals(7653, found[0].port)
        assertEquals(loopback, found[0].address)
        assertEquals("lo", found[0].via?.name)
        assertTrue("returned early, not after the full wait: $elapsedMs ms", elapsedMs < 800)
    }

    @Test
    fun probeIsMagicThenIdBytesThenLengthThenName() {
        val probe = probePayload("000102030405060708090a0b0c0d0e0f", "Pixel 7")

        assertArrayEquals(
            "MIKEY?1".toByteArray() + ByteArray(16) { it.toByte() } + byteArrayOf(7) + "Pixel 7".toByteArray(),
            probe,
        )
    }

    @Test
    fun probeCutsLongNamesTo255Bytes() {
        val probe = probePayload("00".repeat(16), "x".repeat(300))

        assertEquals(7 + 16 + 1 + 255, probe.size)
        assertEquals(255, probe[23].toInt() and 0xFF)
    }

    @Test
    fun parsesTheReplyThePcSends() {
        // The same bytes as the PC's own test in pc/src/transport/beacon.rs.
        val pc = parseReply(reply, reply.size)!!

        assertEquals("02".repeat(16), pc.id)
        assertEquals(7653, pc.port)
        assertEquals(1, pc.proto)
        assertEquals("Mikey-PC", pc.name)
    }

    @Test
    fun ignoresOtherPacketsAndCutOffReplies() {
        assertNull(parseReply("MIKEY?1".toByteArray() + ByteArray(20), 27))
        assertNull(parseReply(reply, reply.size - 1))
        assertNull(parseReply(reply, 10))
    }

    @Test
    fun tetherInterfacesAreLevel2AndTheRestLevel4() {
        assertEquals(2, levelFor("rndis0"))
        assertEquals(2, levelFor("usb0"))
        assertEquals(2, levelFor("ncm0"))
        assertEquals(4, levelFor("wlan0"))
        assertEquals(4, levelFor("eth0"))
    }

    @Test
    fun subnetMatchingHonoursThePrefixLength() {
        val network = byteArrayOf(192.toByte(), 168.toByte(), 1, 0)

        assertTrue(inSubnet(byteArrayOf(192.toByte(), 168.toByte(), 1, 20), network, 24))
        assertFalse(inSubnet(byteArrayOf(192.toByte(), 168.toByte(), 2, 20), network, 24))
        assertTrue(inSubnet(byteArrayOf(192.toByte(), 168.toByte(), 2, 20), network, 16))
        assertFalse(inSubnet(byteArrayOf(192.toByte(), 168.toByte(), 1, 129.toByte()), network, 25))
        assertTrue(inSubnet(byteArrayOf(192.toByte(), 168.toByte(), 1, 129.toByte()), byteArrayOf(192.toByte(), 168.toByte(), 1, 128.toByte()), 25))
        assertFalse(inSubnet(ByteArray(16), network, 24))
    }
}
