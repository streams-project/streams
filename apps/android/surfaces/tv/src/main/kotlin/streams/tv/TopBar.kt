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
import androidx.compose.foundation.shape.RoundedCornerShape
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
        Modifier.height(62.ref).background(back, CircleShape).padding(horizontal = 32.ref),
        contentAlignment = Alignment.Center,
    ) {
        Text(item.label, color = if (on) Back else Color.White, fontSize = 26.refType, fontWeight = FontWeight.Medium)
    }
}

/** Who is watching, search, the tabs and the mark, at the top of every signed in page. */
@Composable
fun TopBar(tabs: List<NavItem>, active: Int, focus: Int, person: Profile?, server: String) {
    val searchOn = focus == SEARCH_TAB
    val whoOn = focus == WHO_TAB
    val searchHere = active == SEARCH_TAB
    Box(Modifier.offset(68.ref, 46.ref)) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Face(person, 48, 8, server, if (whoOn) Modifier.size(48.ref).border(3.ref, Color.White, RoundedCornerShape(8.ref)) else Modifier.size(48.ref))
            Icon(painterResource(R.drawable.icon_caret_down), null, Modifier.size(22.ref), tint = Color.White)
        }
    }
    Box(
        Modifier
            .offset(539.ref, 35.ref)
            .size(64.ref)
            .background(
                when {
                    searchOn -> Color.White
                    searchHere -> Color.White.copy(alpha = 0.16f)
                    else -> Color.Transparent
                },
                CircleShape,
            ),
        contentAlignment = Alignment.Center,
    ) {
        Icon(
            painterResource(R.drawable.icon_search),
            null,
            Modifier.size(34.ref),
            tint = if (searchOn) Back else Color.White,
        )
    }
    Row(Modifier.offset(609.ref, 36.ref)) {
        tabs.forEachIndexed { index, item ->
            Tab(item, index == active, index == focus)
            Spacer(Modifier.width(8.ref))
        }
    }
    AsyncImage("$server/mark.svg", null, Modifier.offset(1796.ref, 40.ref).size(64.ref))
}
