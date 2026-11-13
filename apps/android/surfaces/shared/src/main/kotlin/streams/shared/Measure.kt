package streams.shared

import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

/**
 * A measurement as it was taken off the capture, in the unit Compose lays out in.
 *
 * The derivation is stated once, in `Scale.java` beside this file, and the divisor is read from it
 * rather than written here: Kotlin needs a `Dp` and Java needs a float, so the two cannot be one
 * function, but the number they divide by is one number. Until 2026-11-13 it was four.
 */
val Int.ref: Dp get() = (this / Scale.PER_DP).dp

/** A reference font size, on the same scale: `Measure.kt`'s twin of `Scale.sp`. */
val Int.refType: TextUnit get() = (this / Scale.PER_DP).sp
