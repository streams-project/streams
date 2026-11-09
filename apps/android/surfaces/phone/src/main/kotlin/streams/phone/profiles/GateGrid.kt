package streams.phone.profiles

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import contract.Profile
import streams.ports.Seen
import streams.shared.Corner
import streams.shared.Face
import streams.shared.Join
import streams.shared.Muted
import streams.shared.R

// Measured reference grey for edit/action tiles on chooser screen
private val TileGrey = Color(0xFF333333)

/** A 3-column grid of profile tiles on the chooser screen. */
@Composable
fun GateGrid(
    seen: Seen,
    profiles: List<Profile>,
    manageMode: Boolean,
    profilesMax: Int,
    onEnterManage: () -> Unit,
    onExitManage: () -> Unit,
    onEditProfile: (Profile) -> Unit,
    onAddProfile: () -> Unit,
    onPickPin: (Profile) -> Unit,
    onEnterProfile: (Profile) -> Unit,
    modifier: Modifier = Modifier,
    editTileColor: Color = TileGrey,
) {
    val words = seen.words
    val avatarShape = RoundedCornerShape(10.8.dp)
    val canAdd = profiles.size < profilesMax

    // Cells to display in the 3-column grid:
    // When manageMode:
    //   profiles + (if canAdd AddTile) + DoneTile
    // When normal:
    //   profiles + EditTile
    val itemsCount = profiles.size + (if (manageMode && canAdd) 1 else 0) + 1
    val rows = (itemsCount + 2) / 3

    Column(
        modifier = modifier.fillMaxWidth(),
        verticalArrangement = Arrangement.spacedBy(49.2.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        for (rowIndex in 0 until rows) {
            Row(
                horizontalArrangement = Arrangement.spacedBy(40.dp),
                verticalAlignment = Alignment.Top,
            ) {
                for (colIndex in 0 until 3) {
                    val index = rowIndex * 3 + colIndex
                    if (index < itemsCount) {
                        when {
                            index < profiles.size -> {
                                val person = profiles[index]
                                ProfileTile(
                                    seen = seen,
                                    person = person,
                                    manageMode = manageMode,
                                    avatarShape = avatarShape,
                                    onClick = {
                                        if (manageMode) {
                                            onEditProfile(person)
                                        } else if (person.hasPin) {
                                            onPickPin(person)
                                        } else {
                                            onEnterProfile(person)
                                        }
                                    },
                                )
                            }
                            manageMode && canAdd && index == profiles.size -> {
                                // Add Profile tile
                                ActionTile(
                                    avatarShape = avatarShape,
                                    iconRes = R.drawable.icon_plus,
                                    label = words["profile.add"].orEmpty(),
                                    iconSize = 28,
                                    onClick = onAddProfile,
                                )
                            }
                            else -> {
                                // Edit / Done tile
                                if (manageMode) {
                                    ActionTile(
                                        avatarShape = avatarShape,
                                        iconRes = R.drawable.icon_check,
                                        label = words["common.done"].orEmpty(),
                                        iconSize = 24,
                                        tileColor = editTileColor,
                                        onClick = onExitManage,
                                    )
                                } else {
                                    ActionTile(
                                        avatarShape = avatarShape,
                                        iconRes = R.drawable.icon_pencil,
                                        label = words["profiles.edit"].orEmpty(),
                                        iconSize = 23,
                                        tileColor = editTileColor,
                                        onClick = onEnterManage,
                                    )
                                }
                            }
                        }
                    } else {
                        // Invisible spacer cell to maintain 3-column alignment
                        Spacer(Modifier.width(72.dp))
                    }
                }
            }
        }
    }
}

/** One profile avatar tile with its name, kids badge, lock icon and manage overlay. */
@Composable
private fun ProfileTile(
    seen: Seen,
    person: Profile,
    manageMode: Boolean,
    avatarShape: RoundedCornerShape,
    onClick: () -> Unit,
) {
    val words = seen.words
    Column(
        modifier = Modifier
            .width(72.dp)
            .clickable(onClick = onClick),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Box(
            Modifier
                .size(72.dp)
                .clip(avatarShape),
            contentAlignment = Alignment.Center,
        ) {
            Face(
                person = person,
                server = seen.server,
                modifier = Modifier.fillMaxSize(),
                shape = avatarShape,
                fontSize = 32.sp,
            )

            // Kids badge in normal mode
            if (!manageMode && person.kids) {
                Box(
                    Modifier
                        .align(Alignment.BottomEnd)
                        .offset(x = (-4).dp, y = (-4).dp)
                        .background(Join, RoundedCornerShape(2.dp))
                        .padding(horizontal = 4.dp, vertical = 1.dp),
                ) {
                    Text(
                        words["profile.badge.kids"] ?: "kids",
                        color = Color.White,
                        fontSize = 9.sp,
                        fontWeight = FontWeight.ExtraBold,
                    )
                }
            }

            // In manage mode: dim avatar and show white pencil icon in center
            if (manageMode) {
                Box(
                    Modifier
                        .fillMaxSize()
                        .background(Color.Black.copy(alpha = 0.5f)),
                    contentAlignment = Alignment.Center,
                ) {
                    Icon(
                        painterResource(R.drawable.icon_pencil),
                        null,
                        Modifier.size(26.dp),
                        tint = Color.White,
                    )
                }
            }
        }

        Spacer(Modifier.height(14.8.dp))

        Text(
            person.name,
            color = Color.White,
            fontSize = 14.sp,
            fontWeight = FontWeight.Bold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            textAlign = TextAlign.Center,
            modifier = Modifier.fillMaxWidth(),
        )

        if (!manageMode && person.hasPin) {
            Spacer(Modifier.height(4.dp))
            Icon(
                painterResource(R.drawable.icon_locked),
                null,
                Modifier.size(14.dp),
                tint = Color.White,
            )
        }
    }
}

/** Action tile for Add Profile or Edit / Done. */
@Composable
private fun ActionTile(
    avatarShape: RoundedCornerShape,
    iconRes: Int,
    label: String,
    iconSize: Int,
    tileColor: Color = TileGrey,
    onClick: () -> Unit,
) {
    Column(
        modifier = Modifier
            .width(72.dp)
            .clickable(onClick = onClick),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Box(
            Modifier
                .size(72.dp)
                .clip(avatarShape)
                .background(tileColor),
            contentAlignment = Alignment.Center,
        ) {
            Icon(
                painterResource(iconRes),
                label,
                Modifier.size(iconSize.dp),
                tint = Color.White,
            )
        }

        Spacer(Modifier.height(14.8.dp))

        Text(
            label,
            color = Color.White,
            fontSize = 14.sp,
            fontWeight = FontWeight.Normal,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            textAlign = TextAlign.Center,
            modifier = Modifier.fillMaxWidth(),
        )
    }
}
