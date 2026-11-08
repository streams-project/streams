package streams.tv

import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DontMemoize
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import contract.Profile
import contract.ProfileCreate
import contract.ProfileList
import contract.contractJson
import kotlinx.coroutines.launch
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.R
import streams.shared.Raised
import streams.shared.ref
import streams.shared.refType

private const val ADD_SMALL = 130
private const val ADD_GROW_MS = 200

private enum class AddStep { NAME, KIDS }

/** The last row in the profile column when another profile can be added. */
@Composable
fun AddProfileRow(word: String, on: Boolean) {
    val time = tween<androidx.compose.ui.unit.Dp>(ADD_GROW_MS, easing = Glide)
    val size by animateDpAsState((if (on) FACE_LARGE else ADD_SMALL).ref, time, label = "size")
    val left by animateDpAsState((if (on) 193 else 225).ref, time, label = "left")
    val iconSize = if (on) 64 else 44
    Box(Modifier.fillMaxWidth().height(size)) {
        Box(
            Modifier.offset(left, 0.ref).size(size).background(Raised, Round.face),
            contentAlignment = Alignment.Center,
        ) {
            Icon(painterResource(R.drawable.icon_plus), null, Modifier.size(iconSize.ref), tint = Color.White)
            if (on) {
                Box(Modifier.size(size).border(4.ref, Color.White, Round.face))
            }
        }
        if (on) {
            Text(
                word,
                Modifier.offset(420.ref, 74.ref),
                fontSize = 40.refType,
                fontWeight = FontWeight.Bold,
            )
        }
    }
}

/** Two-step flow to create a new profile: name first, then Kids choice. */
@Composable
fun AddProfile(shell: Shell, seen: Seen, nameMax: Int, finish: (String?) -> Unit) {
    val words = seen.words
    val scope = rememberCoroutineScope()
    val keys = remember { KeyboardState("") }
    var step by remember { mutableStateOf(AddStep.NAME) }
    var refused by remember { mutableStateOf(false) }
    var kidsAt by remember { mutableIntStateOf(0) }
    var busy by remember { mutableStateOf(false) }

    if (keys.text.length > nameMax) {
        keys.text = keys.text.take(nameMax)
    }

    val preview = Profile(
        hasPin = false,
        id = "",
        kids = step == AddStep.KIDS && kidsAt == 1,
        main = false,
        name = keys.text.ifBlank { "+" },
    )

    fun create(kids: Boolean) {
        if (busy) return
        busy = true
        scope.launch {
            val trimmed = keys.text.trim().take(nameMax)
            val body = contractJson.encodeToString(
                ProfileCreate.serializer(),
                ProfileCreate(kids = kids, name = trimmed),
            )
            val answer = shell.write("POST", "/v1/profiles", body)
            if (answer == null) {
                refused = true
                busy = false
                step = AddStep.NAME
            } else {
                val created = contractJson.decodeFromString(Profile.serializer(), answer)
                val listText = shell.read("/v1/profiles")
                val updated = listText
                    ?.let { contractJson.decodeFromString(ProfileList.serializer(), it).profiles }
                    ?: (seen.profiles + created)
                shell.keepAll(updated)
                finish(created.id)
            }
        }
    }

    when (step) {
        AddStep.NAME -> {
            val cancel = words["common.cancel"].orEmpty()
            val done = words["common.next"].orEmpty()
            Box(
                Modifier.fillMaxSize().remote(@DontMemoize { word ->
                    val handled = keys.press(word, cancel, done) { chosen ->
                        when {
                            chosen == null -> finish(null)
                            chosen.trim().isEmpty() -> refused = true
                            else -> {
                                keys.text = chosen.trim().take(nameMax)
                                refused = false
                                step = AddStep.KIDS
                            }
                        }
                    }
                    if (keys.text.length > nameMax) {
                        keys.text = keys.text.take(nameMax)
                    }
                    handled
                }),
            ) {
                PageHead(words["profile.add"].orEmpty(), words["profile.add.help"].orEmpty())
                Keyboard(keys, cancel, done, Modifier.offset(SUB_EDGE.ref, 302.ref))
                ProfileCard(
                    preview,
                    seen.server,
                    {
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            CardName(keys.text)
                            Box(Modifier.padding(start = 3.ref).width(3.ref).height(44.ref).background(Color.White))
                        }
                    },
                    if (refused) words["error.name"].orEmpty() else null,
                )
            }
        }
        AddStep.KIDS -> {
            val lines = listOf(
                MenuLine(words["profile.add.kids.no"].orEmpty()),
                MenuLine(words["profile.add.kids.yes"].orEmpty()),
            )
            Box(
                Modifier.fillMaxSize().remote(@DontMemoize { word ->
                    when (word) {
                        Word.UP -> if (kidsAt > 0) kidsAt -= 1
                        Word.DOWN -> if (kidsAt < lines.lastIndex) kidsAt += 1
                        Word.OK -> create(kidsAt == 1)
                        Word.BACK -> finish(null)
                        else -> return@remote false
                    }
                    true
                }),
            ) {
                PageHead(words["profile.add"].orEmpty(), words["profile.add.kids"].orEmpty())
                Menu(lines, kidsAt, Modifier.offset((SUB_EDGE - MENU_WORDS).ref, 397.ref), width = 460)
                ProfileCard(preview, seen.server, { CardName(keys.text) }, null)
            }
        }
    }
}
