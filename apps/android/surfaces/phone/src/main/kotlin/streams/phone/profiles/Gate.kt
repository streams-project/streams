package streams.phone.profiles

import android.graphics.Bitmap
import androidx.activity.compose.BackHandler
import androidx.compose.animation.Crossfade
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.SingletonImageLoader
import coil3.request.ImageRequest
import coil3.request.SuccessResult
import coil3.toBitmap
import contract.ActionEnterProfile
import contract.Hello
import contract.Profile
import contract.ProfileGate
import contract.contractJson
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.Back
import streams.shared.read

private const val DEFAULT_PROFILES_MAX = 5
private const val CAROUSEL_INTERVAL_MS = 3850L
private const val KEN_BURNS_ZOOM_TARGET = 1.08f

/**
 * The profile chooser screen with background art carousel, Ken Burns zoom, title logo, localized
 * gate lines, arc sheet with dynamic tint model, manage mode, and PIN flow.
 */
@Composable
fun Gate(shell: Shell, seen: Seen) {
    var manageMode by remember { mutableStateOf(false) }
    var editingProfile by remember { mutableStateOf<Profile?>(null) }
    var addingProfile by remember { mutableStateOf(false) }
    var pinProfile by remember { mutableStateOf<Profile?>(null) }
    var pickingProfile by remember { mutableStateOf<Profile?>(null) }

    val hello by produceState<Hello?>(null) {
        value = shell.read("/v1/hello")?.let { contractJson.decodeFromString(Hello.serializer(), it) }
    }
    val gate by produceState<ProfileGate?>(null) {
        value = shell.read("/v1/profiles/gate")?.let { contractJson.decodeFromString(ProfileGate.serializer(), it) }
    }

    val profilesMax = hello?.limits?.profilesMax?.toInt() ?: DEFAULT_PROFILES_MAX
    val words = seen.words

    // Carousel state
    var currentIndex by remember { mutableIntStateOf(0) }
    val zoomAnim = remember { Animatable(1.0f) }
    var tintState by remember { mutableStateOf(GateTint.Neutral) }
    val context = LocalContext.current

    val titlesCount = gate?.titles?.size ?: 0
    val activeTitle = if (titlesCount > 0) gate?.titles?.getOrNull(currentIndex % titlesCount) else null
    val artUrl = remember(activeTitle) { gateArtUrl(activeTitle?.tile) }

    // Ken Burns zoom and Carousel timer
    LaunchedEffect(titlesCount) {
        if (titlesCount <= 0) return@LaunchedEffect
        while (isActive) {
            zoomAnim.snapTo(1.0f)
            zoomAnim.animateTo(
                targetValue = KEN_BURNS_ZOOM_TARGET,
                animationSpec = tween(durationMillis = CAROUSEL_INTERVAL_MS.toInt(), easing = LinearEasing),
            )
            if (titlesCount > 1) {
                currentIndex = (currentIndex + 1) % minOf(titlesCount, 3)
            }
        }
    }

    // Extract dynamic tint from active title art
    LaunchedEffect(artUrl) {
        if (artUrl.isNullOrEmpty()) {
            tintState = GateTint.Neutral
            return@LaunchedEffect
        }
        // The app's one loader, so the picture the grid has already read is not read again.
        val result = SingletonImageLoader.get(context).execute(
            ImageRequest.Builder(context).data(artUrl).build(),
        )
        if (result is SuccessResult) {
            val bitmap = result.image.toBitmap()
            tintState = GateTint.fromBitmap(bitmap)
        }
    }

    BackHandler(enabled = pinProfile != null || editingProfile != null || addingProfile || manageMode) {
        when {
            pinProfile != null -> pinProfile = null
            editingProfile != null -> editingProfile = null
            addingProfile -> addingProfile = false
            manageMode -> manageMode = false
        }
    }

    when {
        editingProfile != null -> {
            EditProfile(
                shell = shell,
                seen = seen,
                start = editingProfile!!,
                leave = { editingProfile = null },
            )
        }
        addingProfile -> {
            AddProfile(
                shell = shell,
                seen = seen,
                leave = { addingProfile = false },
            )
        }
        else -> {
            val configuration = LocalConfiguration.current
            val screenHeight = configuration.screenHeightDp.dp
            val screenWidth = configuration.screenWidthDp.dp
            val density = LocalDensity.current

            // Responsive positioning matching Pixel 8
            val defaultRow1Top = 669.7.dp
            val maxRow1Top = maxOf(340.dp, screenHeight - 240.dp)
            val row1Top = minOf(defaultRow1Top, maxRow1Top)
            val sheetApex = row1Top + 15.9.dp
            val blockBottom = row1Top - 65.2.dp
            val artHeight = row1Top

            Box(Modifier.fillMaxSize().background(Back)) {
                // 1. Background Art Area with Ken Burns Zoom & Fade
                GateArtArea(
                    artUrl = artUrl,
                    tint = tintState,
                    zoomScale = zoomAnim.value,
                    height = artHeight,
                )

                // 2. Title Block: Logo & Gate Line
                Crossfade(
                    targetState = activeTitle,
                    animationSpec = tween(durationMillis = 350),
                    label = "gateTitleCrossfade",
                    modifier = Modifier
                        .fillMaxWidth()
                        .offset(y = blockBottom - 45.dp),
                ) { title ->
                    GateTitleBlock(
                        title = title,
                        seen = seen,
                    )
                }

                // 3. Arc Sheet Background
                val sheetHeightPx = with(density) { (screenHeight - sheetApex).toPx() }
                Box(
                    Modifier
                        .fillMaxWidth()
                        .height(screenHeight - sheetApex)
                        .offset(y = sheetApex)
                        .clip(ArcSheetShape(sagDp = 15.2.dp, radiusDp = 1465.dp))
                        .background(tintState.sheetBrush(sheetHeightPx))
                ) {
                    // 10% white overlay matching spec
                    Box(Modifier.fillMaxSize().background(Color.White.copy(alpha = 0.10f)))
                }

                // 4. Who's watching / Manage label
                val headerKey = if (manageMode) "profiles.manage" else "profiles.choose.phone"
                val fallbackKey = if (manageMode) "profiles.manage" else "profiles.choose"
                val labelText = words[headerKey] ?: words[fallbackKey].orEmpty()
                Text(
                    text = labelText,
                    color = Color.White.copy(alpha = 0.70f),
                    fontSize = 14.sp,
                    fontWeight = FontWeight.Normal,
                    textAlign = TextAlign.Center,
                    modifier = Modifier
                        .fillMaxWidth()
                        .offset(y = row1Top - 25.dp)
                        .padding(horizontal = 24.dp),
                )

                // 5. Profile Grid
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .offset(y = row1Top + 8.dp),
                    horizontalAlignment = Alignment.CenterHorizontally,
                ) {
                    GateGrid(
                        seen = seen,
                        profiles = seen.profiles,
                        manageMode = manageMode,
                        profilesMax = profilesMax,
                        onEnterManage = { manageMode = true },
                        onExitManage = { manageMode = false },
                        onEditProfile = { person -> editingProfile = person },
                        onAddProfile = { addingProfile = true },
                        onPickPin = { person -> pinProfile = person },
                        onEnterProfile = { person ->
                            pickingProfile = person
                            shell.speak(person.language)
                            shell.act(ActionEnterProfile(profile = person.id))
                        },
                        editTileColor = tintState.editTileColor,
                    )
                }

                // 6. Pick animation overlay
                if (pickingProfile != null) {
                    PickZoom(
                        profile = pickingProfile!!,
                        server = seen.server,
                        onDone = {
                            pickingProfile = null
                        },
                    )
                }

                // 7. PIN Sheet
                if (pinProfile != null) {
                    PinSheet(
                        shell = shell,
                        seen = seen,
                        person = pinProfile!!,
                        onDismiss = { pinProfile = null },
                    )
                }
            }
        }
    }
}
