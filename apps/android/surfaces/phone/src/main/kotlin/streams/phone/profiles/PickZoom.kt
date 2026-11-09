package streams.phone.profiles

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import contract.Profile
import kotlinx.coroutines.delay
import streams.shared.Face
import kotlin.math.roundToInt

/**
 * The A3 pick animation:
 * Grows the tapped avatar to 176 dp centered at (width/2, 44% height), holds for ~400 ms,
 * and completes transition into the home screen.
 */
@Composable
fun PickZoom(
    profile: Profile,
    server: String,
    onDone: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val configuration = LocalConfiguration.current
    val density = LocalDensity.current
    val screenWidth = configuration.screenWidthDp.dp
    val screenHeight = configuration.screenHeightDp.dp

    val sizeAnim = remember { Animatable(72f) }
    val yCenterAnim = remember { Animatable(with(density) { (screenHeight * 0.70f).toPx() }) }
    val targetCenterY = with(density) { (screenHeight * 0.44f).toPx() }

    LaunchedEffect(profile) {
        // Phase 1: Grow to 176 dp and move to center
        sizeAnim.animateTo(
            targetValue = 176f,
            animationSpec = tween(durationMillis = 400, easing = FastOutSlowInEasing),
        )
        yCenterAnim.animateTo(
            targetValue = targetCenterY,
            animationSpec = tween(durationMillis = 400, easing = FastOutSlowInEasing),
        )

        // Phase 2: Hold at full size
        delay(400)

        // Phase 3: Transition completes
        onDone()
    }

    Box(modifier = modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.65f))) {
        val currentSizeDp = sizeAnim.value.dp
        val halfSizePx = with(density) { (currentSizeDp / 2).toPx() }
        val screenHalfWidthPx = with(density) { (screenWidth / 2).toPx() }

        Box(
            Modifier
                .offset {
                    IntOffset(
                        x = (screenHalfWidthPx - halfSizePx).roundToInt(),
                        y = (yCenterAnim.value - halfSizePx).roundToInt(),
                    )
                }
                .size(currentSizeDp)
                .clip(RoundedCornerShape(10.8.dp))
        ) {
            Face(
                person = profile,
                server = server,
                modifier = Modifier.fillMaxSize(),
                shape = RoundedCornerShape(10.8.dp),
                fontSize = 48.sp,
            )
        }
    }
}
