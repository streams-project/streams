package streams.phone

import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalConfiguration

/**
 * The width a screen has to work with, by class, never by device name: compact phones, medium
 * (large phones on their side, small tablets), expanded tablets. Every page states its layout per
 * class. These equal Sizes.swift.
 */
enum class WidthClass { COMPACT, MEDIUM, EXPANDED }

/** The class of the current window. */
@Composable
fun widthClass(): WidthClass {
    val wide = LocalConfiguration.current.screenWidthDp
    return when {
        wide < 600 -> WidthClass.COMPACT
        wide < 840 -> WidthClass.MEDIUM
        else -> WidthClass.EXPANDED
    }
}

/** The side margin of a page: 12 dp on a phone, wider on tablets. */
@Composable
fun pageEdge(): Int = when (widthClass()) {
    WidthClass.COMPACT -> 12
    WidthClass.MEDIUM -> 24
    WidthClass.EXPANDED -> 40
}
