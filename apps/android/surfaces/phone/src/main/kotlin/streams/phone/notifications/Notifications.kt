package streams.phone.notifications

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalLayoutDirection
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import contract.ImageRole
import contract.NoticeItem
import contract.NoticeKind
import contract.Notifications
import contract.contractJson
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.Accent
import streams.shared.decoration
import streams.shared.says
import streams.shared.saysItsWords
import streams.shared.Back
import streams.shared.Corner
import streams.shared.Muted
import streams.shared.Plate
import streams.shared.R
import streams.shared.titleAddress

@Composable
fun NotificationsScreen(shell: Shell, seen: Seen) {
    val words = seen.words
    var items by remember { mutableStateOf<List<NoticeItem>>(emptyList()) }
    var cursor by remember { mutableStateOf<String?>(null) }
    var hasMore by remember { mutableStateOf(false) }
    var loading by remember { mutableStateOf(true) }
    var loadingMore by remember { mutableStateOf(false) }

    fun loadPage(c: String?, append: Boolean) {
        val path = if (c != null) "/v1/notifications?cursor=$c" else "/v1/notifications"
        shell.ask(path) { status, text ->
            loading = false
            loadingMore = false
            if (status in 200..299) {
                val parsed = contractJson.decodeFromString(Notifications.serializer(), text)
                cursor = parsed.cursor
                hasMore = parsed.hasMore
                items = if (append) items + parsed.items else parsed.items
            }
        }
    }

    LaunchedEffect(Unit) {
        loadPage(null, false)
    }

    BackHandler { shell.back() }

    val top = WindowInsets.statusBars.asPaddingValues().calculateTopPadding()
    val listState = rememberLazyListState()

    val shouldLoadMore by remember(listState, hasMore, loadingMore) {
        derivedStateOf {
            if (!hasMore || loadingMore || items.isEmpty()) false
            else {
                val lastVisible = listState.layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: 0
                lastVisible >= items.size - 3
            }
        }
    }

    LaunchedEffect(shouldLoadMore) {
        if (shouldLoadMore && cursor != null) {
            loadingMore = true
            loadPage(cursor, true)
        }
    }

    Column(
        Modifier
            .fillMaxSize()
            .background(Back)
            .padding(top = top),
    ) {
        val layoutDirection = LocalLayoutDirection.current
        Row(
            Modifier
                .fillMaxWidth()
                .height(52.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Box(
                Modifier
                    .padding(start = 4.dp)
                    .size(48.dp)
                    .clickable(onClick = { shell.back() })
                    .says(words["common.back"].orEmpty()),
                contentAlignment = Alignment.Center,
            ) {
                Icon(
                    painterResource(R.drawable.icon_back),
                    contentDescription = null,
                    modifier = Modifier
                        .size(24.dp)
                        .graphicsLayer {
                            if (layoutDirection == LayoutDirection.Rtl) scaleX = -1f
                        },
                    tint = Color.White,
                )
            }
            Text(
                words["notifications.title"] ?: "Notifications",
                color = Color.White,
                fontSize = 22.sp,
                fontWeight = FontWeight.Bold,
                maxLines = 1,
            )
        }

        if (loading && items.isEmpty()) {
            Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                CircularProgressIndicator(color = Accent)
            }
        } else if (items.isEmpty()) {
            Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    // A picture beside words that already say there is nothing here.
                    Icon(
                        painterResource(R.drawable.icon_bell),
                        contentDescription = null,
                        modifier = Modifier.size(48.dp).decoration(),
                        tint = Muted,
                    )
                    Spacer(Modifier.height(16.dp))
                    Text(
                        words["notifications.empty"] ?: "No notifications yet.",
                        color = Muted,
                        fontSize = 16.sp,
                    )
                }
            }
        } else {
            LazyColumn(
                state = listState,
                modifier = Modifier.fillMaxSize(),
                contentPadding = PaddingValues(vertical = 8.dp),
            ) {
                items(items, key = { it.id }) { item ->
                    NotificationRow(
                        item = item,
                        words = words,
                        onClick = {
                            if (item.unread) {
                                shell.send(
                                    "POST",
                                    "/v1/notifications/read",
                                    """{"up_to":"${item.id}"}""",
                                ) { _, _ ->
                                    shell.refreshNotifications()
                                }
                                items = items.map {
                                    if (it.id == item.id) it.copy(unread = false) else it
                                }
                            }
                            shell.go(titleAddress(item.tile.title))
                        },
                    )
                }
                if (loadingMore) {
                    item {
                        Box(
                            Modifier
                                .fillMaxWidth()
                                .padding(16.dp),
                            contentAlignment = Alignment.Center,
                        ) {
                            CircularProgressIndicator(modifier = Modifier.size(24.dp), color = Accent)
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun NotificationRow(
    item: NoticeItem,
    words: Map<String, String>,
    onClick: () -> Unit,
) {
    val poster = item.tile.images?.firstOrNull { it.role == ImageRole.POSTER }
    val kindLabel = when (item.kind) {
        NoticeKind.COMING -> words["notification.coming"] ?: "Coming Soon"
        NoticeKind.REMINDER_LIVE -> words["notification.reminder_live"] ?: "Now Available"
        NoticeKind.NEW_ARRIVAL -> words["notification.new_arrival"] ?: "New Arrival"
        NoticeKind.NEW_SEASON -> words["notification.new_season"] ?: "New Season"
        NoticeKind.NEW_EPISODE -> words["notification.new_episode"] ?: "New Episode"
        else -> ""
    }

    Row(
        Modifier
            .fillMaxWidth()
            .clickable(onClick = onClick)
            // The notice draws its kind, the title's name and its date, which is what a reader
            // wants in that order. Whether it has been read is a dot with no words, so it is said
            // as the control's state.
            .saysItsWords(if (item.unread) words["say.unread"].orEmpty() else "")
            .padding(horizontal = 16.dp, vertical = 10.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(
            Modifier
                .size(72.dp, 104.dp)
                .clip(RoundedCornerShape(Corner.CARD.dp))
                .background(Plate),
        ) {
            if (poster != null) {
                AsyncImage(
                    model = poster.url,
                    // The name is drawn as words beside it, so the poster is decoration.
                    contentDescription = null,
                    contentScale = ContentScale.Crop,
                    modifier = Modifier.fillMaxSize(),
                )
            }
        }

        Spacer(Modifier.width(16.dp))

        Column(
            Modifier.weight(1f),
            verticalArrangement = Arrangement.Center,
        ) {
            if (kindLabel.isNotEmpty()) {
                val isAccentKind = item.kind == NoticeKind.REMINDER_LIVE || item.kind == NoticeKind.NEW_ARRIVAL
                Box(
                    Modifier
                        .background(
                            if (isAccentKind) Accent else Plate,
                            RoundedCornerShape(Corner.CHIP.dp),
                        )
                        .padding(horizontal = 8.dp, vertical = 2.dp),
                ) {
                    Text(
                        kindLabel,
                        color = Color.White,
                        fontSize = 11.sp,
                        fontWeight = FontWeight.SemiBold,
                    )
                }
                Spacer(Modifier.height(6.dp))
            }

            Text(
                item.tile.name,
                color = Color.White,
                fontSize = 16.sp,
                fontWeight = FontWeight.Bold,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
            )

            Spacer(Modifier.height(4.dp))

            Text(
                item.date.value,
                color = Muted,
                fontSize = 13.sp,
            )
        }

        if (item.unread) {
            Spacer(Modifier.width(12.dp))
            Box(
                Modifier
                    .size(8.dp)
                    .background(Accent, CircleShape)
                    .decoration(),
            )
        }
    }
}
