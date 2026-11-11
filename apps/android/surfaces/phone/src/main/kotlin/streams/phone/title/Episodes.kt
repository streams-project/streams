package streams.phone

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Canvas
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import contract.Address
import contract.Episode
import contract.Season
import contract.TitleDetail
import contract.contractJson
import kotlin.coroutines.resume
import kotlinx.coroutines.suspendCancellableCoroutine
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.Accent
import streams.shared.Art
import streams.shared.Corner
import streams.shared.Muted
import streams.shared.Plate
import streams.shared.R
import streams.shared.decoration
import streams.shared.fill
import streams.shared.says
import streams.shared.saysItsWords
import streams.shared.number
import streams.shared.wide

suspend fun Shell.read(path: String): String? = suspendCancellableCoroutine { waiting ->
    ask(path) { status, text ->
        if (waiting.isActive) waiting.resume(text.takeIf { status in 200..299 })
    }
}

/**
 * A series' episodes tab: a season chip that opens a full-screen selector, and rows of episodes.
 */
@Composable
fun Episodes(shell: Shell, seen: Seen, data: TitleDetail) {
    val words = seen.words
    val seasons = data.seasons.orEmpty()
    val resumeSeason = data.resume?.episode?.season
    var chosenSeasonIndex by remember(data.title) {
        val idx = seasons.indexOfFirst { it.number == resumeSeason }
        mutableIntStateOf(if (idx >= 0) idx else 0)
    }
    var showSeasonPicker by remember { mutableStateOf(false) }

    val currentSeason = seasons.getOrNull(chosenSeasonIndex)
    val seasonNumber = currentSeason?.number ?: 1

    val episodes by produceState(emptyList<Episode>(), data.title, seasonNumber) {
        val path = data.title.value.replace(':', '/')
        value = shell.read("/v1/titles/$path/seasons/$seasonNumber")
            ?.let { contractJson.decodeFromString(Season.serializer(), it).episodes }
            .orEmpty()
    }

    Column(modifier = Modifier.fillMaxWidth().padding(horizontal = 10.dp)) {
        if (seasons.size > 1) {
            Row(
                modifier = Modifier
                    .padding(vertical = 12.dp)
                    .height(36.dp)
                    .clip(RoundedCornerShape(Corner.CHIP.dp))
                    .background(Color(0xFF212121))
                    .clickable { showSeasonPicker = true }
                    // The season it is on is drawn here; what tapping does is not, so it is said.
                    .says(words["say.season_pick"].orEmpty(), words["title.season.number"].orEmpty().fill("n", seasonNumber))
                    .padding(horizontal = 14.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(6.dp),
            ) {
                Text(
                    text = words["title.season.number"].orEmpty().fill("n", seasonNumber),
                    color = Color.White,
                    fontSize = 14.sp,
                    fontWeight = FontWeight.Bold,
                )
                Icon(
                    painter = painterResource(R.drawable.icon_caret_down),
                    contentDescription = null,
                    modifier = Modifier.size(16.dp),
                    tint = Color.White,
                )
            }
        } else if (seasons.size == 1) {
            Text(
                text = words["title.season.number"].orEmpty().fill("n", seasonNumber),
                color = Color.White,
                fontSize = 14.sp,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.padding(vertical = 12.dp),
            )
        }

        Column(
            modifier = Modifier.fillMaxWidth(),
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            episodes.forEach { episode ->
                EpisodeRow(
                    episode = episode,
                    seasonNumber = seasonNumber,
                    dataPath = data.title.value.replace(':', '/'),
                    shell = shell,
                )
            }
        }
    }

    if (showSeasonPicker) {
        Dialog(
            onDismissRequest = { showSeasonPicker = false },
            properties = DialogProperties(usePlatformDefaultWidth = false),
        ) {
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .background(Color.Black.copy(alpha = 0.92f)),
            ) {
                Column(
                    modifier = Modifier
                        .align(Alignment.Center)
                        .verticalScroll(rememberScrollState())
                        .padding(top = 90.dp, bottom = 110.dp),
                    horizontalAlignment = Alignment.CenterHorizontally,
                ) {
                    seasons.forEachIndexed { index, s ->
                        val isChosen = index == chosenSeasonIndex
                        Box(
                            modifier = Modifier
                                .width(248.dp)
                                .height(52.dp)
                                .clickable {
                                    chosenSeasonIndex = index
                                    showSeasonPicker = false
                                }
                                .saysItsWords(),
                            contentAlignment = Alignment.Center,
                        ) {
                            Text(
                                text = words["title.season.number"].orEmpty().fill("n", s.number),
                                color = if (isChosen) Color.White else Muted,
                                fontSize = if (isChosen) 22.sp else 18.sp,
                                fontWeight = if (isChosen) FontWeight.Bold else FontWeight.Normal,
                            )
                        }
                    }
                }

                Box(
                    modifier = Modifier
                        .align(Alignment.BottomCenter)
                        .padding(bottom = 20.dp)
                        .size(64.dp)
                        .clip(CircleShape)
                        .background(Color.White)
                        .clickable { showSeasonPicker = false }
                        .says(words["say.close"].orEmpty()),
                    contentAlignment = Alignment.Center,
                ) {
                    androidx.compose.foundation.Canvas(modifier = Modifier.size(20.dp)) {
                        val stroke = 2.5.dp.toPx()
                        drawLine(Color.Black, Offset(0f, 0f), Offset(size.width, size.height), strokeWidth = stroke, cap = StrokeCap.Round)
                        drawLine(Color.Black, Offset(0f, size.height), Offset(size.width, 0f), strokeWidth = stroke, cap = StrokeCap.Round)
                    }
                }
            }
        }
    }
}

/**
 * An episode row: 130x80 dp still thumbnail with play circle overlay and progress bar in accent blue,
 * episode number and name bold, runtime, synopsis below.
 */
@Composable
private fun EpisodeRow(episode: Episode, seasonNumber: Long, dataPath: String, shell: Shell) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .clickable {
                shell.go("${Address.WATCH}$dataPath/$seasonNumber/${episode.number}")
            }
            // The row draws the episode's number, its name, how long it runs and what happens in
            // it. All of that is what a reader wants, so the row gathers its own words; the play
            // circle over the thumbnail is marked decoration below.
            .saysItsWords()
            .padding(vertical = 4.dp),
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Box(
                modifier = Modifier
                    .size(130.dp, 80.dp)
                    .clip(RoundedCornerShape(Corner.CARD.dp))
                    .background(Plate),
            ) {
                // The reference's episode thumbnail, 130.0 x 80.0 dp, asked for at that size.
                Art(wide(episode.images), Modifier.fillMaxSize(), 130.dp, 80.dp)
                // The play circle over the thumbnail: the row it sits in is already the control
                // that plays the episode, so this is a picture of what tapping does, not a control.
                Box(
                    modifier = Modifier
                        .align(Alignment.Center)
                        .size(36.dp)
                        .background(Color.Black.copy(alpha = 0.5f), CircleShape)
                        .border(1.5.dp, Color.White.copy(alpha = 0.8f), CircleShape)
                        .decoration(),
                    contentAlignment = Alignment.Center,
                ) {
                    Icon(
                        painter = painterResource(R.drawable.icon_play),
                        contentDescription = null,
                        modifier = Modifier.size(16.dp),
                        tint = Color.White,
                    )
                }
                val percent = episode.percent
                if (percent != null && percent > 0) {
                    val fraction = (percent / 100f).coerceIn(0f, 1f)
                    Box(
                        modifier = Modifier
                            .align(Alignment.BottomStart)
                            .fillMaxWidth()
                            .height(3.dp)
                            .background(Color.Black.copy(alpha = 0.6f)),
                    ) {
                        Box(
                            modifier = Modifier
                                .fillMaxHeight()
                                .fillMaxWidth(fraction)
                                .background(Accent),
                        )
                    }
                }
            }

            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = "${episode.number}. ${episode.name}",
                    color = Color.White,
                    fontSize = 14.sp,
                    fontWeight = FontWeight.Bold,
                    lineHeight = 18.sp,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                )
                episode.runtimeMin?.takeIf { it > 0 }?.let { rm ->
                    Text(
                        text = "${number(rm)}m",
                        color = Muted,
                        fontSize = 12.sp,
                        modifier = Modifier.padding(top = 2.dp),
                    )
                }
            }
        }

        val overview = episode.overview.orEmpty()
        if (overview.isNotEmpty()) {
            Text(
                text = overview,
                color = Muted,
                fontSize = 13.sp,
                lineHeight = 18.sp,
                maxLines = 4,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.padding(top = 8.dp),
            )
        }
    }
}
