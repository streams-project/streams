package streams.phone

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Popup
import coil3.compose.AsyncImage
import contract.ActionCommand
import contract.Address
import contract.CommandList
import contract.CommandRate
import contract.CommandRemind
import contract.ImageRole
import contract.Rating
import contract.Season
import contract.TitleDetail
import contract.contractJson
import streams.phone.reminder.ReminderScheduler
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.Accent
import streams.shared.Art
import streams.shared.Back
import streams.shared.Corner
import streams.shared.Muted
import streams.shared.Plate
import streams.shared.R
import streams.shared.Raised
import streams.shared.counted
import streams.shared.fill
import streams.shared.number
import streams.shared.wide

private enum class TitleTab {
    EPISODES,
    MORE_LIKE_THIS,
}

private fun playTitle(shell: Shell, data: TitleDetail) {
    val isSeries = data.title.value.startsWith("tv:")
    val path = data.title.value.replace(':', '/')
    if (!isSeries) {
        shell.go(Address.WATCH + path)
        return
    }
    val next = data.resume?.episode
    if (next != null) {
        shell.go("${Address.WATCH}$path/${next.season}/${next.episode}")
        return
    }
    val number = data.seasons?.firstOrNull()?.number ?: 1
    shell.ask("/v1/titles/$path/seasons/$number") { status, text ->
        val first = if (status in 200..299) {
            contractJson.decodeFromString(Season.serializer(), text).episodes.firstOrNull()?.number ?: 1
        } else {
            1
        }
        shell.go("${Address.WATCH}$path/$number/$first")
    }
}

/**
 * The title page on phones: 16:9 art under a 52 dp bar, title or logo, facts, white play button,
 * expandable synopsis, cast/directors line, action buttons row, divider, and Episodes / More Like
 * This tabs.
 */
@Composable
fun Title(shell: Shell, seen: Seen, tv: Boolean = false) {
    val data = seen.detail?.takeIf { it.title == seen.route?.title } ?: return
    val words = seen.words
    val isSeries = data.title.value.startsWith("tv:")

    val context = LocalContext.current
    var selectedTab by remember(data.title) {
        mutableStateOf(if (isSeries) TitleTab.EPISODES else TitleTab.MORE_LIKE_THIS)
    }
    var saved by remember(data.title, data.inList) { mutableStateOf(data.inList == true) }
    var reminded by remember(data.title, data.reminderSet) { mutableStateOf(data.reminderSet == true) }
    var rating by remember(data.title, data.rating) { mutableStateOf(data.rating) }
    var rateMenuOpen by remember { mutableStateOf(false) }
    var synopsisExpanded by remember { mutableStateOf(false) }
    var castExpanded by remember { mutableStateOf(false) }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(Back),
    ) {
        // Pinned 52 dp top bar with back arrow and cast/close buttons
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .height(52.dp)
                .background(Back)
                .padding(horizontal = 4.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Box(
                modifier = Modifier
                    .size(48.dp)
                    .clickable(onClick = { shell.back() }),
                contentAlignment = Alignment.Center,
            ) {
                Icon(
                    painter = painterResource(R.drawable.icon_back),
                    contentDescription = words["player.back"] ?: "Back",
                    modifier = Modifier.size(24.dp),
                    tint = Color.White,
                )
            }
            Spacer(modifier = Modifier.weight(1f))
            Box(
                modifier = Modifier
                    .size(48.dp)
                    .clickable { /* cast action */ },
                contentAlignment = Alignment.Center,
            ) {
                Icon(
                    painter = painterResource(R.drawable.icon_cast),
                    contentDescription = "Cast",
                    modifier = Modifier.size(24.dp),
                    tint = Color.White,
                )
            }
            Box(
                modifier = Modifier
                    .size(48.dp)
                    .clickable(onClick = { shell.back() }),
                contentAlignment = Alignment.Center,
            ) {
                androidx.compose.foundation.Canvas(modifier = Modifier.size(16.dp)) {
                    val stroke = 2.dp.toPx()
                    drawLine(Color.White, Offset(0f, 0f), Offset(size.width, size.height), strokeWidth = stroke, cap = StrokeCap.Round)
                    drawLine(Color.White, Offset(0f, size.height), Offset(size.width, 0f), strokeWidth = stroke, cap = StrokeCap.Round)
                }
            }
        }

        // Scrollable content
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .weight(1f)
                .verticalScroll(rememberScrollState()),
        ) {
            // Top: 16:9 backdrop art
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .aspectRatio(16f / 9f)
                    .background(Plate),
            ) {
                Art(wide(data.images), Modifier.fillMaxSize())
            }

            // Body content with 10 dp margins
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 10.dp, vertical = 10.dp),
            ) {
                // Title / logo
                val logoUrl = data.images?.firstOrNull { it.role == ImageRole.LOGO }?.url
                if (logoUrl != null) {
                    AsyncImage(
                        model = logoUrl,
                        contentDescription = data.name,
                        modifier = Modifier
                            .fillMaxWidth(0.7f)
                            .heightIn(max = 80.dp),
                        alignment = Alignment.CenterStart,
                        contentScale = ContentScale.Fit,
                    )
                } else {
                    Text(
                        text = data.name,
                        fontSize = 24.sp,
                        fontWeight = FontWeight.Bold,
                        color = Color.White,
                        lineHeight = 28.sp,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis,
                    )
                }

                // Meta line: year, age badge, seasons/length, HD badge
                Row(
                    modifier = Modifier
                        .padding(top = 8.dp)
                        .fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    data.year?.let { y ->
                        Text(
                            text = number(y),
                            color = Muted,
                            fontSize = 14.sp,
                        )
                    }

                    data.age?.label?.let { age ->
                        Box(
                            modifier = Modifier
                                .clip(RoundedCornerShape(Corner.FIELD.dp))
                                .background(Color.White.copy(alpha = 0.15f))
                                .padding(horizontal = 6.dp, vertical = 2.dp),
                        ) {
                            Text(
                                text = age,
                                color = Color.White.copy(alpha = 0.9f),
                                fontSize = 12.sp,
                                fontWeight = FontWeight.Medium,
                            )
                        }
                    }

                    val seasonsCount = data.seasons?.size ?: 0
                    val minutes = data.runtimeMin
                    val lengthStr = when {
                        seasonsCount > 0 -> words.counted("title.season", seasonsCount)
                        minutes != null && minutes >= 60 ->
                            words["title.length.hours"].orEmpty().fill("h", minutes / 60).fill("m", minutes % 60)
                        minutes != null ->
                            words["title.length.minutes"].orEmpty().fill("m", minutes)
                        else -> null
                    }
                    lengthStr?.let { len ->
                        Text(
                            text = len,
                            color = Muted,
                            fontSize = 14.sp,
                        )
                    }

                    Box(
                        modifier = Modifier
                            .border(1.dp, Color.White.copy(alpha = 0.4f), RoundedCornerShape(3.dp))
                            .padding(horizontal = 5.dp, vertical = 1.dp),
                    ) {
                        Text(
                            text = "HD",
                            color = Color.White.copy(alpha = 0.9f),
                            fontSize = 10.sp,
                            fontWeight = FontWeight.Bold,
                        )
                    }
                }

                val availableFrom = data.availableFrom
                if (availableFrom != null) {
                    val availableDate = availableFrom.value
                    val comingLabel = words["gate.coming"]?.replace("{date}", availableDate)
                        ?: "Coming on $availableDate"
                    Text(
                        text = comingLabel,
                        color = Color.White,
                        fontSize = 15.sp,
                        fontWeight = FontWeight.SemiBold,
                        modifier = Modifier.padding(top = 10.dp),
                    )
                    Button(
                        onClick = {
                            val next = !reminded
                            reminded = next
                            shell.act(ActionCommand(CommandRemind(on = next, title = data.title)))
                            if (next) {
                                ReminderScheduler.schedule(context, data.title, data.name, availableDate)
                            } else {
                                ReminderScheduler.cancel(context, data.title)
                            }
                        },
                        modifier = Modifier
                            .padding(top = 8.dp)
                            .fillMaxWidth()
                            .height(48.dp),
                        shape = RoundedCornerShape(Corner.BUTTON.dp),
                        colors = ButtonDefaults.buttonColors(
                            containerColor = if (reminded) Raised else Accent,
                            contentColor = Color.White,
                        ),
                        contentPadding = PaddingValues(horizontal = 16.dp),
                    ) {
                        Icon(
                            painter = painterResource(if (reminded) R.drawable.icon_check else R.drawable.icon_bell),
                            contentDescription = null,
                            modifier = Modifier.size(22.dp),
                            tint = Color.White,
                        )
                        Spacer(modifier = Modifier.width(8.dp))
                        Text(
                            text = words[if (reminded) "remind.set" else "remind.me"]
                                ?: (if (reminded) "Reminder Set" else "Remind Me"),
                            fontSize = 16.sp,
                            fontWeight = FontWeight.Bold,
                            color = Color.White,
                        )
                    }
                } else {
                    // Play button: white, full width (10 dp margins), 48 dp high, Corner.BUTTON radius
                    Button(
                        onClick = { playTitle(shell, data) },
                        modifier = Modifier
                            .padding(top = 10.dp)
                            .fillMaxWidth()
                            .height(48.dp),
                        shape = RoundedCornerShape(Corner.BUTTON.dp),
                        colors = ButtonDefaults.buttonColors(
                            containerColor = Color.White,
                            contentColor = Color.Black,
                        ),
                        contentPadding = PaddingValues(horizontal = 16.dp),
                    ) {
                        Icon(
                            painter = painterResource(R.drawable.icon_play),
                            contentDescription = null,
                            modifier = Modifier.size(22.dp),
                            tint = Color.Black,
                        )
                        Spacer(modifier = Modifier.width(8.dp))
                        Text(
                            text = words["common.play"] ?: "Play",
                            fontSize = 16.sp,
                            fontWeight = FontWeight.Bold,
                            color = Color.Black,
                        )
                    }
                }

                // Synopsis: 16 sp, expandable (3 lines with "more")
                val overview = data.overview.orEmpty()
                if (overview.isNotEmpty()) {
                    Column(
                        modifier = Modifier
                            .padding(top = 10.dp)
                            .fillMaxWidth()
                            .clickable { synopsisExpanded = !synopsisExpanded },
                    ) {
                        Text(
                            text = overview,
                            color = Color.White,
                            fontSize = 16.sp,
                            lineHeight = 21.sp,
                            maxLines = if (synopsisExpanded) Int.MAX_VALUE else 3,
                            overflow = TextOverflow.Ellipsis,
                        )
                        if (!synopsisExpanded && overview.length > 120) {
                            Text(
                                text = "… more",
                                color = Color(0xFFE7E7E7),
                                fontSize = 14.sp,
                                fontWeight = FontWeight.Bold,
                                modifier = Modifier.padding(top = 2.dp),
                            )
                        }
                    }
                }

                // Cast / director line: muted color with "more" expandable
                val genres = data.genres.orEmpty()
                if (genres.isNotEmpty()) {
                    val displayGenres = if (castExpanded) genres else genres.take(3)
                    Row(
                        modifier = Modifier
                            .padding(top = 8.dp)
                            .fillMaxWidth()
                            .clickable { if (genres.size > 3) castExpanded = !castExpanded },
                    ) {
                        Text(
                            text = displayGenres.joinToString(", "),
                            color = Muted,
                            fontSize = 13.sp,
                            lineHeight = 18.sp,
                        )
                        if (!castExpanded && genres.size > 3) {
                            Text(
                                text = "… more",
                                color = Color(0xFFE7E7E7),
                                fontSize = 13.sp,
                                fontWeight = FontWeight.Medium,
                                modifier = Modifier.padding(start = 4.dp),
                            )
                        }
                    }
                }

                // Action buttons: My List, Rate, Share in a row (about 96 dp wide, 8 dp apart)
                Row(
                    modifier = Modifier
                        .padding(top = 16.dp, start = 4.dp)
                        .fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    // My List
                    Column(
                        modifier = Modifier
                            .width(96.dp)
                            .clickable {
                                saved = !saved
                                shell.act(ActionCommand(CommandList(inList = saved, title = data.title)))
                            }
                            .padding(vertical = 4.dp),
                        horizontalAlignment = Alignment.CenterHorizontally,
                    ) {
                        Icon(
                            painter = painterResource(if (saved) R.drawable.icon_check else R.drawable.icon_plus),
                            contentDescription = null,
                            modifier = Modifier.size(24.dp),
                            tint = Color.White,
                        )
                        Spacer(modifier = Modifier.height(6.dp))
                        Text(
                            text = words[if (saved) "list.added" else "list.add"] ?: "My List",
                            color = Muted,
                            fontSize = 12.sp,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                        )
                    }

                    // Rate
                    Box {
                        Column(
                            modifier = Modifier
                                .width(96.dp)
                                .clickable { rateMenuOpen = !rateMenuOpen }
                                .padding(vertical = 4.dp),
                            horizontalAlignment = Alignment.CenterHorizontally,
                        ) {
                            val rateIcon = when (rating) {
                                Rating.DOWN -> R.drawable.icon_thumb_down_filled
                                Rating.UP -> R.drawable.icon_thumb_up_filled
                                Rating.LOVE -> R.drawable.icon_love_filled
                                else -> R.drawable.icon_thumb_up
                            }
                            Icon(
                                painter = painterResource(rateIcon),
                                contentDescription = null,
                                modifier = Modifier.size(24.dp),
                                tint = Color.White,
                            )
                            Spacer(modifier = Modifier.height(6.dp))
                            Text(
                                text = "Rate",
                                color = Muted,
                                fontSize = 12.sp,
                                maxLines = 1,
                            )
                        }

                        if (rateMenuOpen) {
                            Popup(
                                alignment = Alignment.TopCenter,
                                offset = IntOffset(0, -110),
                                onDismissRequest = { rateMenuOpen = false },
                            ) {
                                Row(
                                    modifier = Modifier
                                        .clip(RoundedCornerShape(Corner.BUTTON.dp))
                                        .background(Raised)
                                        .padding(horizontal = 8.dp, vertical = 4.dp),
                                    horizontalArrangement = Arrangement.spacedBy(4.dp),
                                    verticalAlignment = Alignment.CenterVertically,
                                ) {
                                    IconButton(onClick = {
                                        val next = if (rating == Rating.DOWN) null else Rating.DOWN
                                        rating = next
                                        shell.act(ActionCommand(CommandRate(rating = next, title = data.title)))
                                        rateMenuOpen = false
                                    }) {
                                        Icon(painterResource(if (rating == Rating.DOWN) R.drawable.icon_thumb_down_filled else R.drawable.icon_thumb_down), null, tint = Color.White)
                                    }
                                    IconButton(onClick = {
                                        val next = if (rating == Rating.UP) null else Rating.UP
                                        rating = next
                                        shell.act(ActionCommand(CommandRate(rating = next, title = data.title)))
                                        rateMenuOpen = false
                                    }) {
                                        Icon(painterResource(if (rating == Rating.UP) R.drawable.icon_thumb_up_filled else R.drawable.icon_thumb_up), null, tint = Color.White)
                                    }
                                    IconButton(onClick = {
                                        val next = if (rating == Rating.LOVE) null else Rating.LOVE
                                        rating = next
                                        shell.act(ActionCommand(CommandRate(rating = next, title = data.title)))
                                        rateMenuOpen = false
                                    }) {
                                        Icon(painterResource(if (rating == Rating.LOVE) R.drawable.icon_love_filled else R.drawable.icon_love), null, tint = Color.White)
                                    }
                                }
                            }
                        }
                    }

                    // Share
                    Column(
                        modifier = Modifier
                            .width(96.dp)
                            .clickable { /* share */ }
                            .padding(vertical = 4.dp),
                        horizontalAlignment = Alignment.CenterHorizontally,
                    ) {
                        Icon(
                            painter = painterResource(R.drawable.icon_share),
                            contentDescription = null,
                            modifier = Modifier.size(24.dp),
                            tint = Color.White,
                        )
                        Spacer(modifier = Modifier.height(6.dp))
                        Text(
                            text = "Share",
                            color = Muted,
                            fontSize = 12.sp,
                            maxLines = 1,
                        )
                    }
                }
            }

            // Divider
            Box(
                modifier = Modifier
                    .padding(top = 16.dp)
                    .fillMaxWidth()
                    .height(1.dp)
                    .background(Color.White.copy(alpha = 0.15f)),
            )

            // Tabs: "Episodes" (for series) and "More Like This" (48 dp boxes, 4 dp accent line on chosen tab)
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(48.dp)
                    .padding(horizontal = 10.dp),
                horizontalArrangement = Arrangement.Start,
            ) {
                if (isSeries) {
                    val epChosen = selectedTab == TitleTab.EPISODES
                    Box(
                        modifier = Modifier
                            .height(48.dp)
                            .clickable { selectedTab = TitleTab.EPISODES }
                            .padding(horizontal = 12.dp),
                        contentAlignment = Alignment.Center,
                    ) {
                        if (epChosen) {
                            Box(
                                modifier = Modifier
                                    .align(Alignment.TopCenter)
                                    .fillMaxWidth()
                                    .height(4.dp)
                                    .background(Accent),
                            )
                        }
                        Text(
                            text = words["title.episodes_more"] ?: "Episodes",
                            color = if (epChosen) Color.White else Muted,
                            fontSize = 14.sp,
                            fontWeight = FontWeight.Bold,
                        )
                    }
                }

                val moreChosen = selectedTab == TitleTab.MORE_LIKE_THIS
                Box(
                    modifier = Modifier
                        .height(48.dp)
                        .clickable { selectedTab = TitleTab.MORE_LIKE_THIS }
                        .padding(horizontal = 12.dp),
                    contentAlignment = Alignment.Center,
                ) {
                    if (moreChosen) {
                        Box(
                            modifier = Modifier
                                .align(Alignment.TopCenter)
                                .fillMaxWidth()
                                .height(4.dp)
                                .background(Accent),
                        )
                    }
                    Text(
                        text = words["title.more_like"] ?: "More Like This",
                        color = if (moreChosen) Color.White else Muted,
                        fontSize = 14.sp,
                        fontWeight = FontWeight.Bold,
                    )
                }
            }

            // Tab Content
            if (isSeries && selectedTab == TitleTab.EPISODES) {
                Episodes(shell = shell, seen = seen, data = data)
            } else {
                MoreLike(shell = shell, seen = seen, data = data)
            }

            // Bottom space for floating bottom bar
            Spacer(modifier = Modifier.height(96.dp))
        }
    }
}
