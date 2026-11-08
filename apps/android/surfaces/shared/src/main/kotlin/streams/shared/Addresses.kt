package streams.shared

import contract.Address
import contract.Tile
import contract.TitleKey

/** A title's page. */
fun titleAddress(key: TitleKey) = Address.TITLE + key.value.replace(':', '-')

/** Where a title plays. */
fun watchAddress(key: TitleKey) = Address.WATCH + key.value.replace(':', '/')

/** A tab's page, from the screen name the server gives it. */
fun tabAddress(screen: String) = Address.of(screen)

/** True for a series. */
fun Tile.isSeries() = title.value.startsWith("tv:")
