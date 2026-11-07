package streams.surfaces

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import contract.Address
import contract.Image
import contract.ImageRole
import contract.Section
import contract.SectionKind
import contract.Tile
import contract.TitleKey
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.Back
import streams.shared.Muted
import streams.shared.Raised

fun Modifier.focusRing(changed: (Boolean) -> Unit): Modifier = onFocusChanged { changed(it.isFocused) }

fun picture(images: List<Image>?): String? {
    val list = images.orEmpty()
    return listOf(ImageRole.BACKDROP, ImageRole.STILL, ImageRole.POSTER)
        .firstNotNullOfOrNull { role -> list.firstOrNull { it.role == role } }?.url
}

fun titleAddress(key: TitleKey): String = Address.TITLE + key.value.replace(':', '-')

@Composable
fun Browse(shell: Shell, seen: Seen, tv: Boolean) {
    val sections = seen.screen?.sections.orEmpty()
    val lead = sections.firstOrNull { it.kind == SectionKind.HERO }?.tiles?.firstOrNull()
        ?: sections.firstOrNull { !it.tiles.isNullOrEmpty() }?.tiles?.firstOrNull()
    val rows = sections.filter { it.kind != SectionKind.HERO && !it.tiles.isNullOrEmpty() }
    var focused by remember { mutableStateOf<Tile?>(null) }
    val shown = focused ?: lead
    LazyColumn(Modifier.fillMaxSize()) {
        item { Hero(shell, shown, tv) }
        items(rows, key = { it.id }) { section -> RowOf(shell, section, tv) { focused = it } }
        item { Spacer(Modifier.height(48.dp)) }
    }
}

@Composable
private fun Hero(shell: Shell, tile: Tile?, tv: Boolean) {
    if (tile == null) return
    val first = remember { FocusRequester() }
    Box(Modifier.fillMaxWidth().height(if (tv) 340.dp else 420.dp)) {
        AsyncImage(
            model = picture(tile.images),
            contentDescription = null,
            contentScale = ContentScale.Crop,
            modifier = Modifier.fillMaxSize(),
        )
        Box(Modifier.fillMaxSize().background(Brush.verticalGradient(listOf(Color.Transparent, Back))))
        Column(Modifier.align(Alignment.BottomStart).padding(if (tv) 48.dp else 20.dp)) {
            Text(tile.name, fontSize = if (tv) 44.sp else 32.sp, fontWeight = FontWeight.Black)
            Spacer(Modifier.height(4.dp))
            Text(tile.tags.orEmpty().joinToString(" · "), color = Muted)
            Spacer(Modifier.height(12.dp))
            Button({ shell.go(titleAddress(tile.title)) }, Modifier.focusRequester(first)) { Text("More info") }
        }
    }
    LaunchedEffect(Unit) { if (tv) first.requestFocus() }
}

@Composable
private fun RowOf(shell: Shell, section: Section, tv: Boolean, onFocus: (Tile) -> Unit) {
    Column(Modifier.padding(top = 16.dp)) {
        Text(
            section.title.orEmpty(),
            Modifier.padding(horizontal = if (tv) 48.dp else 20.dp),
            fontSize = 18.sp,
            fontWeight = FontWeight.Bold,
        )
        Spacer(Modifier.height(8.dp))
        LazyRow(
            horizontalArrangement = Arrangement.spacedBy(10.dp),
            contentPadding = androidx.compose.foundation.layout.PaddingValues(horizontal = if (tv) 48.dp else 20.dp),
        ) {
            items(section.tiles.orEmpty(), key = { it.title.value }) { tile ->
                Card(tile, if (tv) 220 else 150, { onFocus(tile) }) { shell.go(titleAddress(tile.title)) }
            }
        }
    }
}

@Composable
private fun Card(tile: Tile, width: Int, focused: () -> Unit, open: () -> Unit) {
    var on by remember { mutableStateOf(false) }
    val url = picture(tile.images)
    Box(
        Modifier
            .width(width.dp)
            .aspectRatio(16f / 9f)
            .clip(RoundedCornerShape(12.dp))
            .border(if (on) 3.dp else 0.dp, Color.White, RoundedCornerShape(12.dp))
            .background(Raised)
            .focusRing {
                on = it
                if (it) focused()
            }
            .clickable(onClick = open),
    ) {
        if (url != null) {
            AsyncImage(url, tile.name, Modifier.fillMaxSize(), contentScale = ContentScale.Crop)
        } else {
            Text(tile.name, Modifier.align(Alignment.Center).padding(8.dp), fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
        }
    }
}
