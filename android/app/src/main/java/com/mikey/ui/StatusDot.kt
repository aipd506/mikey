package com.mikey.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import com.mikey.R
import com.mikey.service.Link

/** Green while streaming to the PC, amber while the PC asks its user, red otherwise. */
@Composable
fun StatusDot(link: Link, modifier: Modifier = Modifier) {
    val color = when (link) {
        is Link.Live -> Palette.statusOk
        Link.Waiting -> Palette.statusWait
        else -> Palette.statusErr
    }
    val description = stringResource(
        when (link) {
            is Link.Live -> R.string.cd_status_connected
            Link.Waiting -> R.string.cd_status_waiting
            is Link.Refused -> R.string.cd_status_refused
            Link.Searching -> R.string.cd_status_no_pc
        },
    )
    Box(
        modifier
            .size(10.dp)
            .background(color, RoundedCornerShape(3.dp))
            .semantics { contentDescription = description },
    )
}
