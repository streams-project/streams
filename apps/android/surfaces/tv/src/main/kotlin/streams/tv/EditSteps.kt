package streams.tv

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DontMemoize
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import contract.Language
import contract.Profile
import streams.shared.Back
import streams.shared.R
import streams.shared.ref
import streams.shared.refType

private fun Map<String, String>.say(key: String) = this[key].orEmpty()

/** A small pill low on the left of a page, for Done or Cancel. */
@Composable
private fun LowPill(word: String, on: Boolean, top: Int) {
    Box(
        Modifier
            .offset(127.ref, top.ref)
            .size(224.ref, 61.ref)
            .background(if (on) Color.White else Color.White.copy(alpha = 0.16f), CircleShape),
        contentAlignment = Alignment.Center,
    ) {
        Text(word, color = if (on) Back else Color.White, fontSize = 30.refType, fontWeight = FontWeight.Bold)
    }
}

/** What this profile may watch. The server holds the rule, and this only says it. */
@Composable
private fun AgeNote(words: Map<String, String>, person: Profile, limit: String, top: Int = 690) {
    val key = if (person.kids) "kids" else limit
    val chip = words["profile.age.$key"] ?: words.say("profile.age.limit").replace("{age}", limit)
    val line = words["profile.age.$key.help"] ?: words.say("profile.age.limit.help")
    Box(Modifier.offset(157.ref, top.ref)) {
        Text(
            chip,
            Modifier.background(Color.White.copy(alpha = 0.2f), RoundedCornerShape(6.ref)).padding(horizontal = 10.ref),
            fontSize = 24.refType,
            fontWeight = FontWeight.Bold,
        )
    }
    Text(line, Modifier.offset(157.ref, (top + 48).ref).width(900.ref), fontSize = 30.refType)
}

/** The first page: what can be changed. */
@Composable
fun EditMenu(
    words: Map<String, String>,
    person: Profile,
    limit: String,
    language: String?,
    pictures: Boolean,
    server: String,
    at: Int,
    move: (Int) -> Unit,
    open: (EditStep) -> Unit,
    leave: () -> Unit,
) {
    var asking by remember { mutableStateOf(false) }
    var askAt by remember { mutableIntStateOf(1) }
    val lines = buildList {
        add(EditStep.NAME to MenuLine(words.say("profile.name"), R.drawable.icon_person))
        if (pictures) add(EditStep.ICON to MenuLine(words.say("profile.icon"), R.drawable.icon_face))
        add(EditStep.LANGUAGE to MenuLine(words.say("profile.language"), R.drawable.icon_language))
        if (!person.main) add(EditStep.DELETE to MenuLine(words.say("profile.delete")))
    }
    val askLines = listOf(
        MenuLine(words.say("profile.delete")),
        MenuLine(words.say("common.cancel")),
    )
    Box(
        Modifier.fillMaxSize().remote(@DontMemoize { word ->
            if (asking) {
                when (word) {
                    Word.UP -> if (askAt > 0) askAt -= 1
                    Word.DOWN -> if (askAt < askLines.lastIndex) askAt += 1
                    Word.OK -> if (askAt == 0) open(EditStep.DELETE) else asking = false
                    Word.BACK -> asking = false
                    else -> return@remote false
                }
                return@remote true
            }
            when (word) {
                Word.UP -> if (at > 0) move(at - 1)
                Word.DOWN -> if (at < lines.size) move(at + 1)
                Word.OK -> {
                    val chosen = lines.getOrNull(at)?.first
                    when (chosen) {
                        null -> leave()
                        EditStep.DELETE -> {
                            asking = true
                            askAt = 1
                        }
                        else -> open(chosen)
                    }
                }
                Word.BACK -> leave()
                else -> return@remote false
            }
            true
        }),
    ) {
        PageHead(
            words.say("profile.edit"),
            if (asking) words.say("profile.delete.ask") else words.say("profile.edit.help"),
        )
        if (asking) {
            Menu(askLines, askAt, Modifier.offset(127.ref, 397.ref))
        } else {
            Menu(lines.map { it.second }, at, Modifier.offset(127.ref, 397.ref))
            AgeNote(words, person, limit, if (lines.size > 4) 750 else 690)
            LowPill(words.say("common.done"), at == lines.size, 897)
        }
        ProfileCard(person, server, { CardName(person.name) }, language)
    }
}

/** The keyboard, with the name growing under the picture. */
@Composable
fun EditName(words: Map<String, String>, person: Profile, server: String, refused: Boolean, finish: (String?) -> Unit) {
    val keys = remember { KeyboardState(person.name) }
    val cancel = words.say("common.cancel")
    val done = words.say("common.done")
    Box(Modifier.fillMaxSize().remote { word -> keys.press(word, cancel, done, finish) }) {
        PageHead(words.say("profile.edit"), words.say("profile.name.help"))
        Keyboard(keys, cancel, done, Modifier.offset(126.ref, 302.ref))
        ProfileCard(
            person,
            server,
            {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    CardName(keys.text)
                    Box(Modifier.padding(start = 3.ref).width(3.ref).height(44.ref).background(Color.White))
                }
            },
            if (refused) words.say("error.name") else null,
        )
    }
}

/** Rows of pictures by theme. */
@Composable
fun EditIcon(
    words: Map<String, String>,
    person: Profile,
    rows: List<PictureRow>,
    server: String,
    finish: (String?) -> Unit,
) {
    val focus = remember { PictureFocus() }
    Box(
        Modifier.fillMaxSize().remote { word ->
            when (word) {
                Word.OK -> finish(focus.chosen(rows))
                Word.BACK -> finish(null)
                else -> focus.press(word, rows)
            }
            true
        },
    ) {
        PictureRows(rows, focus, server)
        PageHead(words.say("profile.edit"), words.say("profile.icon.help"))
        ProfileCorner(person, server)
    }
}

/** Every language this server has words for. */
@Composable
fun EditLanguage(words: Map<String, String>, list: List<Language>, current: String?, finish: (String?) -> Unit) {
    val ticked = list.indexOfFirst { it.code == current }.coerceAtLeast(0)
    var at by remember { mutableStateOf(ticked) }
    val cancelAt = list.size
    Box(
        Modifier.fillMaxSize().remote { word ->
            when (word) {
                Word.UP -> when {
                    at == cancelAt -> at = minOf(GRID_LINES, list.size) - 1
                    at % GRID_LINES > 0 -> at -= 1
                }
                Word.DOWN -> when {
                    at == cancelAt -> Unit
                    at % GRID_LINES == GRID_LINES - 1 || at == list.lastIndex -> at = cancelAt
                    else -> at += 1
                }
                Word.LEFT -> if (at != cancelAt && at >= GRID_LINES) at -= GRID_LINES
                Word.RIGHT -> if (at != cancelAt && at + GRID_LINES <= list.lastIndex) at += GRID_LINES
                Word.OK -> finish(list.getOrNull(at)?.code)
                Word.BACK -> finish(null)
            }
            true
        },
    ) {
        PageHead(words.say("profile.edit"), words.say("profile.language.help"))
        TickGrid(list.map { it.name }, ticked, at)
        LowPill(words.say("common.cancel"), at == cancelAt, 900)
    }
}
