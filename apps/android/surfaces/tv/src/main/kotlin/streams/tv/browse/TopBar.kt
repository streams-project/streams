package streams.tv

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import coil3.compose.AsyncImage
import contract.NavItem
import contract.Profile
import streams.shared.Back
import streams.shared.R
import streams.shared.ref
import streams.shared.refType

/** The place of the search button, to the left of the first tab. */
const val SEARCH_TAB = -2

/** The place of the profile, to the left of search. */
const val WHO_TAB = -3

/** The bar's centre line and the height of its round parts: face, search, tabs and mark share them. */
private const val BAR_MID = 68
private const val BAR_HIGH = 64
private const val FACE = 48

/** The tabs that fit the bar. Genres live on their own pages. */
fun tabsOf(nav: List<NavItem>): List<NavItem> = nav.filter { it.screen.value != "genre" }.take(6)

@Composable
private fun Tab(item: NavItem, active: Boolean, on: Boolean) {
    val back = when {
        on -> Color.White
        active -> Color.White.copy(alpha = 0.16f)
        else -> Color.Transparent
    }
    Box(
        Modifier
            .height(BAR_HIGH.ref)
            .background(back, CircleShape)
            .padding(horizontal = 32.ref)
            .saysItsWords(on),
        contentAlignment = Alignment.Center,
    ) {
        Text(item.label, color = if (on) Back else Color.White, fontSize = 26.refType, fontWeight = FontWeight.Medium)
    }
}

/**
 * Who is watching, search, the tabs and the mark, at the top of every signed in page.
 *
 * `words` is here for the two parts that are a picture and nothing else - the profile and search -
 * because a picture with no words is unreachable from a remote. The tabs carry the names the server
 * sent with them, and the mark is decoration: it is the same on every page and says nothing a
 * viewer needs.
 */
@Composable
fun TopBar(
    tabs: List<NavItem>,
    active: Int,
    focus: Int,
    person: Profile?,
    server: String,
    words: Map<String, String> = emptyMap(),
) {
    val searchOn = focus == SEARCH_TAB
    val whoOn = focus == WHO_TAB
    val searchHere = active == SEARCH_TAB
    Box(Modifier.offset(PAGE_EDGE.ref, (BAR_MID - FACE / 2).ref).says(person?.name, whoOn)) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Face(person, FACE, server, if (whoOn) Modifier.size(FACE.ref).border(3.ref, Color.White, Round.face) else Modifier.size(FACE.ref))
            Icon(painterResource(R.drawable.icon_caret_down), null, Modifier.size(22.ref), tint = Color.White)
        }
    }
    Box(
        Modifier
            .offset(539.ref, (BAR_MID - BAR_HIGH / 2).ref)
            .size(BAR_HIGH.ref)
            .background(
                when {
                    searchOn -> Color.White
                    searchHere -> Color.White.copy(alpha = 0.16f)
                    else -> Color.Transparent
                },
                CircleShape,
            )
            .says(words["nav.search"], searchOn),
        contentAlignment = Alignment.Center,
    ) {
        Icon(
            painterResource(R.drawable.icon_search),
            null,
            Modifier.size(34.ref),
            tint = if (searchOn) Back else Color.White,
        )
    }
    Row(Modifier.offset(609.ref, (BAR_MID - BAR_HIGH / 2).ref)) {
        tabs.forEachIndexed { index, item ->
            Tab(item, index == active, index == focus)
            Spacer(Modifier.width(8.ref))
        }
    }
    AsyncImage("$server/mark.svg", null, Modifier.offset((1920 - PAGE_EDGE - BAR_HIGH).ref, (BAR_MID - BAR_HIGH / 2).ref).size(BAR_HIGH.ref))
}
