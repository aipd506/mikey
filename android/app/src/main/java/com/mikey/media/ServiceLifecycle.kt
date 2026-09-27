package com.mikey.media

import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleOwner
import androidx.lifecycle.LifecycleRegistry

/** A lifecycle for CameraX to bind to inside a service, which has none of its own. Use on the main thread. */
class ServiceLifecycle : LifecycleOwner {
    private val registry = LifecycleRegistry(this)

    override val lifecycle: Lifecycle get() = registry

    fun start() {
        registry.currentState = Lifecycle.State.STARTED
    }

    /** Final: CameraX unbinds everything. Make a new one to start again. */
    fun destroy() {
        registry.currentState = Lifecycle.State.DESTROYED
    }
}
