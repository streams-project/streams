package streams.surfaces

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.background
import androidx.compose.material3.Button
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import contract.Season
import contract.TitleDetail
import contract.contractJson
import streams.shared.Back
import streams.shared.Muted
import streams.ports.Seen
import streams.ports.Shell

/** A series always plays an episode, so Play starts the first one of the first season. */
private fun play(shell: Shell, data: TitleDetail) {
    val (kind, id) = data.title.value.split(":")
    if (kind != "tv") {
        shell.go("/watch/$kind/$id")
        return
    }
    val number = data.seasons?.firstOrNull()?.number ?: 1
    shell.ask("/v1/titles/$kind/$id/seasons/$number") { status, text ->
        val first = if (status in 200..299) {
            contractJson.decodeFromString(Season.serializer(), text).episodes.firstOrNull()?.number ?: 1
        } else {
            1
        }
        shell.go("/watch/$kind/$id/$number/$first")
    }
}

@Composable
fun Title(shell: Shell, seen: Seen, tv: Boolean) {
    val data = seen.detail?.takeIf { it.title == seen.route?.title } ?: return
    val first = remember { FocusRequester() }
    Box(Modifier.fillMaxSize()) {
        AsyncImage(picture(data.images), null, Modifier.fillMaxSize(), contentScale = ContentScale.Crop)
        Box(Modifier.fillMaxSize().background(Brush.verticalGradient(listOf(Color.Transparent, Back))))
        Column(
            Modifier.align(Alignment.BottomStart).padding(if (tv) 48.dp else 20.dp).widthIn(max = 720.dp).fillMaxWidth(),
        ) {
            Text(data.name, fontSize = if (tv) 48.sp else 34.sp, fontWeight = FontWeight.Black)
            Spacer(Modifier.height(8.dp))
            Text(listOfNotNull(data.year?.toString()).joinToString(" · "), color = Muted)
            Spacer(Modifier.height(10.dp))
            Text(data.overview.orEmpty(), maxLines = 5)
            Spacer(Modifier.height(16.dp))
            Button({ play(shell, data) }, Modifier.focusRequester(first)) { Text("Play") }
        }
    }
    LaunchedEffect(Unit) { first.requestFocus() }
}
