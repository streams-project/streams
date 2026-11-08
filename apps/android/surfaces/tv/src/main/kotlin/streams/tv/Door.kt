package streams.tv

import android.view.View
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.keyframes
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.DontMemoize
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.graphics.drawscope.scale
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.scale
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.viewinterop.AndroidView
import androidx.compose.ui.unit.dp
import contract.Address
import streams.shared.Raised
import contract.contractJson
import contract.TitleDetail
import contract.PublicTitles
import contract.ImageRole
import coil3.request.ImageRequest
import coil3.compose.AsyncImage
import coil3.SingletonImageLoader
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.layout.ContentScale
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.key
import androidx.compose.foundation.layout.sizeIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.height
import androidx.compose.animation.Crossfade
import android.content.Context
import kotlinx.coroutines.delay
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.Back
import streams.shared.Join
import streams.shared.R
import streams.shared.ref
import streams.shared.refType

private const val EMPTY_CARD_MS = 333L
private const val ROW_FADE_MS = 240
/** The lockup's mark height at the top left of the landing, measured on the reference: 58 of 1080. */
private const val LOCKUP_HIGH = 58f
private const val FRAME_MS = 42
private const val STAGE = 1920

// Share of the picture width the card still has to travel, one value per frame, as measured.
private val Slide = listOf(0.97f, 0.647f, 0.323f, 0.106f, 0.077f, 0.039f, 0.017f, 0.005f, 0.001f, 0f)

private val CardWash = Brush.verticalGradient(
    0f to Color(0xFF2B3A78),
    0.7f to Color(0xFF42549C),
    1f to Color(0xFF141730),
)
private val CardEdge = Color(0xFFC4E4FB)

/**
 * The violet light in the top left corner behind the card, as the approved opening renders it:
 * strength (1 - (x / 1500 + y / 620)) ^ 1.5 of 5F28EB, on a 1920 by 1080 page.
 */
private val Glow = Brush.linearGradient(
    0f to Color(0xFF5F28EB),
    0.25f to Color(0xFF5F28EB).copy(alpha = 0.65f),
    0.5f to Color(0xFF5F28EB).copy(alpha = 0.354f),
    0.75f to Color(0xFF5F28EB).copy(alpha = 0.125f),
    1f to Color.Transparent,
    start = Offset.Zero,
    end = Offset(219f, 530f),
)

private fun DrawScope.glow() {
    scale(size.width / STAGE, size.height / (STAGE * 9f / 16f), Offset.Zero) {
        drawRect(Glow, Offset.Zero, Size(STAGE.toFloat(), STAGE * 9f / 16f))
    }
}

/** Where the focus can be on the landing: the top row, the showreel card's buttons, a title card's button. */
private enum class Stop { HOME, TAB, TOP_SIGN_IN, HELP, SIGN_IN, JOIN, WATCH }

/** Cards sit this far apart: 1542 wide and 39 between, so the next one shows at 1653. */
private const val CARD_STEP = 1581

/** The page margin: the lockup, the top row and the card all start here, and Sign In ends this far from the right. */
private const val EDGE = 72
private const val CARD_TOP = 169

/** The top row's centre line and height: the lockup, the home circle, the tab and Sign In share both. */
private const val ROW_MID = 104
private const val ROW_HIGH = 72

/** Inside a card, words and buttons keep this far from its left and right edges. */
private const val INSET = 74
private const val CARD_MOVE_MS = 320
private const val SLIDE_EVERY_MS = 4_500L
private const val SLIDE_FADE_MS = 700

private val TitleSide = arrayOf(0f to Color.Black.copy(alpha = 0.8f), 0.62f to Color.Transparent)
private val TitleFoot = Brush.verticalGradient(0.45f to Color.Transparent, 1f to Color.Black.copy(alpha = 0.75f))

/** The pictures of a title card in turn: its backdrop, then its episodes. */
private fun slidesOf(title: TitleDetail): List<String> =
    listOfNotNull(wide(title.images)) + title.images.orEmpty().filter { it.role == ImageRole.STILL }.map { it.url }

/** One title's pictures in turn while its card is the one in view, each fading into the next. */
@Composable
private fun Slideshow(slides: List<String>, running: Boolean) {
    var at by remember { mutableIntStateOf(0) }
    LaunchedEffect(running, slides) {
        while (running && slides.size > 1) {
            delay(SLIDE_EVERY_MS)
            at = (at + 1) % slides.size
        }
    }
    Crossfade(slides.getOrNull(at), Modifier.fillMaxSize(), tween(SLIDE_FADE_MS, easing = LinearEasing), label = "slides") { url ->
        Art(url, Modifier.fillMaxSize())
    }
}

/** A trending title as a card: its pictures, logo, facts, story and a way in. */
@Composable
private fun TitleCard(title: TitleDetail, words: Map<String, String>, current: Boolean, on: Boolean) {
    val shape = shapeOf(Round.panel)
    Box(Modifier.size(1542.ref, 864.ref).border(3.ref, CardEdge, shape).clip(shape).background(Raised)) {
        Slideshow(slidesOf(title), current)
        Box(Modifier.fillMaxSize().background(fromStart(*TitleSide)))
        Box(Modifier.fillMaxSize().background(TitleFoot))
        val logo = title.images?.firstOrNull { it.role == ImageRole.LOGO }?.url
        Box(Modifier.offset(INSET.ref, 424.ref).size(520.ref, 180.ref), contentAlignment = Alignment.BottomStart) {
            if (logo != null) {
                AsyncImage(
                    logo,
                    title.name,
                    Modifier.sizeIn(maxWidth = 520.ref, maxHeight = 180.ref),
                    alignment = Alignment.BottomStart,
                    contentScale = ContentScale.Fit,
                )
            } else {
                Text(title.name, fontSize = 64.refType, lineHeight = 70.refType, fontWeight = FontWeight.ExtraBold, maxLines = 2)
            }
        }
        Facts(title, words, Modifier.offset(INSET.ref, 628.ref))
        Text(
            title.overview.orEmpty(),
            Modifier.offset(INSET.ref, 676.ref).width(720.ref),
            color = Color.White.copy(alpha = 0.85f),
            fontSize = 30.refType,
            lineHeight = 39.refType,
            maxLines = 3,
            overflow = TextOverflow.Ellipsis,
        )
        Box(Modifier.align(Alignment.TopEnd).offset((-INSET).ref, 720.ref)) { Ask(words["door.watch"].orEmpty(), on, Join, Color.White) }
    }
}

/** Fetches a picture ahead, so a card already has it when it comes into view. */
private fun fetchAhead(context: Context, url: String, done: () -> Unit = {}) {
    SingletonImageLoader.get(context).enqueue(
        ImageRequest.Builder(context).data(url).listener(onSuccess = { _, _ -> done() }, onError = { _, _ -> done() }).build(),
    )
}

/** A brand film or sound. It plays while `on`, and is handed back when it leaves the screen. */
@Composable
fun Film(
    shell: Shell,
    url: String,
    loop: Boolean,
    on: Boolean,
    modifier: Modifier,
    ready: () -> Unit = {},
    ended: () -> Unit,
) {
    val picture = remember { arrayOfNulls<Any>(1) }
    val readyNow by rememberUpdatedState(ready)
    val endedNow by rememberUpdatedState(ended)
    AndroidView(
        { context ->
            (shell.reel.picture(context, url, loop, { readyNow() }, { endedNow() }) as View).also { picture[0] = it }
        },
        modifier,
        update = { view -> shell.reel.play(view, on) },
    )
    DisposableEffect(Unit) { onDispose { picture[0]?.let(shell.reel::release) } }
}

@Composable
private fun Ask(word: String, on: Boolean, lit: Color, ink: Color) {
    Box(
        Modifier
            .scale(if (on) 1.04f else 1f)
            .size(268.ref, 80.ref)
            .background(if (on) lit else Color.White.copy(alpha = 0.22f), CircleShape),
        contentAlignment = Alignment.Center,
    ) {
        Text(word, color = if (on) ink else Color.White, fontSize = 28.refType, fontWeight = FontWeight.Bold)
    }
}

/**
 * The landing. After the opening it is already in place under it: the card does not slide, the
 * showreel is there from the first frame, says when it is `ready`, and plays once `roll` is true.
 * Coming back to it later, the card slides in and the showreel starts after it.
 */
@Composable
private fun Landing(shell: Shell, seen: Seen, opening: Boolean, roll: Boolean, ready: () -> Unit) {
    val words = seen.words
    val context = LocalContext.current
    // As in the reference, Get started holds the focus when the landing first shows.
    var at by remember { mutableStateOf(Stop.JOIN) }
    // Which card is in view: 0 is the showreel, then the trending titles.
    var page by remember { mutableIntStateOf(0) }
    var cards by remember { mutableStateOf<List<TitleDetail>?>(null) }
    var filmReady by remember { mutableStateOf(false) }
    var peekReady by remember { mutableStateOf(false) }
    // The brand's lockup sits top left until the viewer first goes up, or along to the titles.
    var raised by remember { mutableStateOf(false) }
    if (at == Stop.HOME || at == Stop.TAB || at == Stop.TOP_SIGN_IN || page > 0) raised = true
    val row by animateFloatAsState(if (raised) 1f else 0f, tween(ROW_FADE_MS, easing = LinearEasing), label = "row")
    var film by remember { mutableStateOf(opening) }
    val slide = remember { Animatable(if (opening) 0f else Slide.first()) }
    val width = with(LocalDensity.current) { STAGE.ref.toPx() }
    // In a right to left language the cards run leftwards: every move along them turns.
    val way = along()
    val step = with(LocalDensity.current) { CARD_STEP.ref.toPx() } * way
    val moved by animateFloatAsState(page.toFloat(), tween(CARD_MOVE_MS, easing = Glide), label = "cards")

    // The titles and all their pictures load at once, under the opening when there is one, so every
    // card is there as soon as the landing shows. The opening waits for the showreel and the next card.
    LaunchedEffect(Unit) {
        val list = shell.read("/v1/public/trending")
            ?.let { runCatching { contractJson.decodeFromString(PublicTitles.serializer(), it).titles }.getOrNull() }
            .orEmpty()
        list.forEachIndexed { index, title ->
            slidesOf(title).forEachIndexed { which, url ->
                fetchAhead(context, url) { if (index == 0 && which == 0) peekReady = true }
            }
            title.images?.firstOrNull { it.role == ImageRole.LOGO }?.let { fetchAhead(context, it.url) }
        }
        if (list.firstOrNull()?.let(::slidesOf).isNullOrEmpty()) peekReady = true
        cards = list
    }
    LaunchedEffect(filmReady, cards, peekReady) {
        if (filmReady && cards != null && peekReady) ready()
    }
    LaunchedEffect(Unit) {
        if (!opening) {
            slide.animateTo(
                0f,
                keyframes {
                    durationMillis = (Slide.size - 1) * FRAME_MS
                    Slide.forEachIndexed { index, left -> left at index * FRAME_MS }
                },
            )
            delay(EMPTY_CARD_MS)
            film = true
        }
    }

    val titles = cards.orEmpty()
    fun cardStop() = if (page == 0) Stop.JOIN else Stop.WATCH
    fun show(card: Int) {
        page = card
        at = cardStop()
    }

    fun press(word: Word): Boolean {
        val onTop = at == Stop.HOME || at == Stop.TAB || at == Stop.TOP_SIGN_IN
        when {
            // Back walks out the way the viewer came in: the row to its card, a title to the showreel,
            // and only the showreel's buttons leave the app.
            word == Word.BACK -> when {
                onTop -> at = cardStop()
                page > 0 -> show(0)
                else -> return false
            }
            onTop -> when (word) {
                Word.LEFT -> at = if (at == Stop.TOP_SIGN_IN) Stop.TAB else Stop.HOME
                Word.RIGHT -> at = when {
                    at == Stop.HOME && titles.isNotEmpty() -> Stop.TAB
                    at == Stop.TAB && page > 0 -> Stop.TOP_SIGN_IN
                    else -> at
                }
                Word.DOWN -> at = cardStop()
                Word.OK -> when (at) {
                    Stop.HOME -> show(0)
                    Stop.TAB -> show(maxOf(page, 1))
                    else -> shell.go(Address.SIGN_IN)
                }
                else -> Unit
            }
            page == 0 -> when (word) {
                Word.LEFT -> at = if (at == Stop.JOIN) Stop.SIGN_IN else Stop.HELP
                // Past Get Started the next card comes in, as on the reference.
                Word.RIGHT -> when (at) {
                    Stop.HELP -> at = Stop.SIGN_IN
                    Stop.SIGN_IN -> at = Stop.JOIN
                    else -> if (titles.isNotEmpty()) show(1)
                }
                Word.UP -> at = Stop.HOME
                Word.OK -> shell.go(if (at == Stop.HELP) Address.HELP else Address.SIGN_IN)
                else -> Unit
            }
            else -> when (word) {
                Word.LEFT -> show(page - 1)
                Word.RIGHT -> if (page < titles.size) show(page + 1)
                Word.UP -> at = Stop.TAB
                Word.OK -> shell.go(Address.SIGN_IN)
                else -> Unit
            }
        }
        return true
    }

    val shape = shapeOf(Round.panel)
    Box(Modifier.fillMaxSize().background(Back).drawBehind { if (way < 0f) scale(-1f, 1f) { glow() } else glow() }.remote(@DontMemoize { press(it) })) {
        if (row < 1f) BrandLockup(LOCKUP_HIGH, Modifier.offset(EDGE.ref, (ROW_MID - LOCKUP_HIGH.toInt() / 2).ref).alpha(1 - row))
        if (row > 0f) {
            Row(Modifier.offset(EDGE.ref, (ROW_MID - ROW_HIGH / 2).ref).alpha(row), verticalAlignment = Alignment.CenterVertically) {
                Box(
                    Modifier
                        .size(ROW_HIGH.ref)
                        .background(if (at == Stop.HOME) Color.White else Color.White.copy(alpha = 0.12f), CircleShape),
                    contentAlignment = Alignment.Center,
                ) {
                    Icon(
                        painterResource(R.drawable.icon_home),
                        null,
                        Modifier.size(34.ref),
                        tint = if (at == Stop.HOME) Back else Color.White,
                    )
                }
                Spacer(Modifier.width(16.ref))
                if (titles.isNotEmpty()) {
                    // The tab of the titles: grey while a title card is in view, white in focus.
                    Box(
                        Modifier
                            .height(ROW_HIGH.ref)
                            .background(
                                when {
                                    at == Stop.TAB -> Color.White
                                    page > 0 -> Color.White.copy(alpha = 0.16f)
                                    else -> Color.Transparent
                                },
                                CircleShape,
                            )
                            .padding(horizontal = 32.ref),
                        contentAlignment = Alignment.Center,
                    ) {
                        Text(
                            words["door.trending"].orEmpty(),
                            color = if (at == Stop.TAB) Back else Color.White,
                            fontSize = 28.refType,
                            fontWeight = FontWeight.SemiBold,
                        )
                    }
                }
            }
            if (page > 0) {
                // On a title card the way to sign in moves up to the top right, outlined until in focus.
                val on = at == Stop.TOP_SIGN_IN
                Box(
                    Modifier
                        .align(Alignment.TopEnd)
                        .offset((-EDGE).ref, (ROW_MID - ROW_HIGH / 2).ref)
                        .height(ROW_HIGH.ref)
                        .widthIn(min = 200.ref)
                        .alpha(row)
                        .border(3.ref, if (on) Color.Transparent else Color.White.copy(alpha = 0.5f), CircleShape)
                        .background(if (on) Color.White else Color.Transparent, CircleShape)
                        .padding(horizontal = 40.ref),
                    contentAlignment = Alignment.Center,
                ) {
                    Text(words["door.sign_in"].orEmpty(), color = if (on) Back else Color.White, fontSize = 28.refType, fontWeight = FontWeight.SemiBold)
                }
            }
        }
        Box(
            Modifier
                .offset(EDGE.ref, CARD_TOP.ref)
                // A card that leaves to the left fades as it goes, so no sliver of it waits at the edge.
                .graphicsLayer {
                    translationX = slide.value * width * way - moved * step
                    alpha = (1f - moved).coerceIn(0f, 1f)
                }
                .size(1542.ref, 864.ref)
                .border(3.ref, CardEdge, shape)
                .clip(shape)
                .background(CardWash),
        ) {
            // The showreel rests while another card is in view.
            if (film) {
                Film(shell, "${seen.server}/tv/showreel.mp4", true, roll && page == 0, Modifier.fillMaxSize(), { filmReady = true }) {
                    filmReady = true
                }
            }
            // Get Help sits on the buttons' centre line; its words keep the card's inset, the pill grows around them.
            Box(
                Modifier
                    .offset((INSET - 24).ref, 732.ref)
                    .height(56.ref)
                    .background(if (at == Stop.HELP) Color.White else Color.Transparent, CircleShape)
                    .padding(horizontal = 24.ref),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    words["door.help"].orEmpty(),
                    color = if (at == Stop.HELP) Back else Color.White.copy(alpha = 0.85f),
                    fontSize = 24.refType,
                    fontWeight = FontWeight.SemiBold,
                )
            }
            Row(Modifier.align(Alignment.TopEnd).offset((-INSET).ref, 720.ref)) {
                Ask(words["door.sign_in"].orEmpty(), at == Stop.SIGN_IN, Color.White, Back)
                Spacer(Modifier.width(32.ref))
                Ask(words["door.join"].orEmpty(), at == Stop.JOIN, Join, Color.White)
            }
        }
        // Each title card is built when it is near: the one in view and one each side.
        titles.forEachIndexed { index, title ->
            val card = index + 1
            if (card in page - 1..page + 1) {
                key(title.title) {
                    Box(Modifier.offset(EDGE.ref, CARD_TOP.ref).graphicsLayer {
                        translationX = (card - moved) * step
                        alpha = (1f - (moved - card)).coerceIn(0f, 1f)
                    }) {
                        TitleCard(title, words, current = card == page, on = card == page && at == Stop.WATCH)
                    }
                }
            }
        }
    }
}

/**
 * The way in: the landing. The first time, the opening is drawn over it (see `OpeningLayer`), and it
 * loads its showreel underneath so the arrow opens onto the film.
 */
@Composable
fun Door(shell: Shell, seen: Seen, state: OpeningState) {
    val opening = remember { !state.opened }
    Landing(shell, seen, opening, state.roll) { state.landingReady = true }
}
