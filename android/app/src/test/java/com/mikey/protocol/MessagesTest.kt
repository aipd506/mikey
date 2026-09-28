package com.mikey.protocol

import org.junit.Assert.assertEquals
import org.junit.Test

class MessagesTest {

    @Test
    fun videoIsOfferedWithACameraExceptOverBluetooth() {
        assertEquals(listOf("audio", "video"), phoneCaps(camera = true, level = 1))
        assertEquals(listOf("audio", "video"), phoneCaps(camera = true, level = 4))
        assertEquals(listOf("audio"), phoneCaps(camera = true, level = 3))
        assertEquals(listOf("audio"), phoneCaps(camera = false, level = 1))
    }
}
