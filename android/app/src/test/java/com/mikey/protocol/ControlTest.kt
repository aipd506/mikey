package com.mikey.protocol

import org.junit.Assert.assertEquals
import org.junit.Test

class ControlTest {
    private val current = AudioSettings(ns = true, nsStrength = 0.8f)

    @Test
    fun anEmptyUpdateChangesNothing() {
        assertEquals(current, ControlUpdate().applyTo(current))
    }

    @Test
    fun onlyTheFieldsSentChange() {
        assertEquals(AudioSettings(ns = true, nsStrength = 0.3f), ControlUpdate(nsStrength = 0.3f).applyTo(current))
        assertEquals(AudioSettings(ns = false, nsStrength = 0.8f), ControlUpdate(ns = false).applyTo(current))
    }
}
