package com.mikey.protocol

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class ControlTest {
    private val current = AudioSettings(ns = true, nsStrength = 0.8f, aec = true, gateDb = -45f)

    @Test
    fun anEmptyUpdateChangesNothing() {
        assertEquals(current, ControlUpdate().applyTo(current))
    }

    @Test
    fun onlyTheFieldsSentChange() {
        val updated = ControlUpdate(aec = false, nsStrength = 0.3f).applyTo(current)

        assertEquals(AudioSettings(ns = true, nsStrength = 0.3f, aec = false, gateDb = -45f), updated)
    }

    @Test
    fun aNullGateSwitchesItOffAndAValueSetsIt() {
        assertNull(ControlUpdate(gateOff = true).applyTo(current).gateDb)
        assertEquals(-30f, ControlUpdate(gateDb = -30f).applyTo(current.copy(gateDb = null)).gateDb)
    }
}
