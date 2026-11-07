package streams.shared

import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

/**
 * Every television lays out at 960 by 540 dp, and the screen references measure 1920 by 1080, so a
 * reference pixel is half a dp. These keep the numbers in the code the same as the measurements.
 */
val Int.ref: Dp get() = (this / 2f).dp

/** A reference font size. */
val Int.refType: TextUnit get() = (this / 2f).sp
