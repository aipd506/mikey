package com.mikey.service

import org.junit.Assert.assertEquals
import org.junit.Test
import java.util.concurrent.ArrayBlockingQueue

class SessionControllerTest {

    @Test
    fun reconnectWaitsDoubleFromHalfASecondAndStopAtFive() {
        val waits = listOf(0, 1, 2, 3, 4, 5, 100).map(::reconnectDelayMs)

        assertEquals(listOf(500L, 1_000L, 2_000L, 4_000L, 5_000L, 5_000L, 5_000L), waits)
    }

    @Test
    fun rejectPolicyRetriesOnlyWhatCanClearUpByItself() {
        assertEquals(Reaction.RETRY, rejectPolicy("timeout"))
        assertEquals(Reaction.RETRY, rejectPolicy("busy"))
        assertEquals(Reaction.FORGET_AND_RETRY, rejectPolicy("bad_token"))
        assertEquals(Reaction.GIVE_UP, rejectPolicy("denied"))
        assertEquals(Reaction.GIVE_UP, rejectPolicy("version"))
        assertEquals(Reaction.GIVE_UP, rejectPolicy("something-new"))
    }

    @Test
    fun fullQueueDropsTheOldestItem() {
        val queue = ArrayBlockingQueue<Int>(2)

        listOf(1, 2, 3).forEach { queue.offerDroppingOldest(it) }

        assertEquals(listOf(2, 3), queue.toList())
    }
}
