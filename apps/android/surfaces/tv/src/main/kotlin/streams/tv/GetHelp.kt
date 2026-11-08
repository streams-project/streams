package streams.tv

import streams.shared.fill
import android.app.Activity
import android.os.Build
import android.os.SystemClock
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DontMemoize
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import contract.Account
import contract.ActionSignOut
import contract.Address
import contract.Hello
import contract.contractJson
import java.util.Locale
import kotlinx.coroutines.launch
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.Muted
import streams.shared.ref
import streams.shared.refType

private enum class Line(val word: String) {
    BACK("help.back"),
    MEMBER("help.member"),
    DEVICE("help.device"),
    NETWORK("help.network"),
    CENTRE("help.centre"),
    SETTINGS("help.settings"),
    CONTACT("help.contact"),
    CHECK("help.check"),
    RELOAD("help.reload"),
    EXIT("menu.exit"),
    SIGN_OUT("menu.sign_out"),
}

/** Lines that need an account. */
private val MemberOnly = setOf(Line.MEMBER, Line.SIGN_OUT)

@Composable
private fun Fact(head: String, value: String) {
    Column(Modifier.padding(bottom = 22.ref)) {
        Text(head, fontSize = 30.refType, fontWeight = FontWeight.Bold)
        Text(value, fontSize = 28.refType, color = Muted)
    }
}

@Composable
private fun Hint(text: String) {
    Text(text, Modifier.width(900.ref), fontSize = 30.refType, color = Color.White.copy(alpha = 0.85f))
}

/** Who and what this is, and the ways to get unstuck. The panel follows the focus. */
@Composable
fun GetHelp(shell: Shell, seen: Seen) {
    val words = seen.words
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    val person = seen.profiles.firstOrNull { it.id == seen.active } ?: seen.profiles.firstOrNull()
    val signedIn = seen.profiles.isNotEmpty()
    val lines = Line.entries.filter { line ->
        when {
            line in MemberOnly -> signedIn
            line == Line.SETTINGS -> signedIn && person?.kids != true
            else -> true
        }
    }
    var at by remember { mutableIntStateOf(1) }
    var centre by remember { mutableStateOf(false) }
    var asking by remember { mutableStateOf(false) }
    var answer by remember { mutableIntStateOf(1) }
    var checked by remember { mutableStateOf<String?>(null) }
    val account by produceState<Account?>(null, signedIn) {
        if (signedIn) {
            value = shell.read("/v1/account")?.let { contractJson.decodeFromString(Account.serializer(), it) }
        }
    }
    val hello by produceState<Hello?>(null) {
        value = shell.read("/v1/hello")?.let { contractJson.decodeFromString(Hello.serializer(), it) }
    }
    if (centre) {
        HelpCentre(shell, seen) { centre = false }
        return
    }
    val line = lines.getOrElse(at) { Line.BACK }

    fun check() {
        val start = SystemClock.elapsedRealtime()
        scope.launch {
            val answered = shell.read("/v1/ready") != null
            val took = SystemClock.elapsedRealtime() - start
            checked = if (answered) {
                words["help.check.ok"].orEmpty().fill("ms", took.toString())
            } else {
                words["help.check.fail"].orEmpty()
            }
        }
    }

    fun choose() {
        when (line) {
            Line.BACK -> shell.back()
            Line.CENTRE -> centre = true
            Line.SETTINGS -> shell.go(Address.SETTINGS)
            Line.CHECK -> check()
            Line.RELOAD -> shell.go(Address.START)
            Line.EXIT -> (context as? Activity)?.finish()
            Line.SIGN_OUT -> {
                asking = true
                answer = 1
            }
            else -> Unit
        }
    }

    fun press(word: Word): Boolean {
        if (asking) {
            when (word) {
                Word.LEFT -> answer = 0
                Word.RIGHT -> answer = 1
                Word.BACK -> asking = false
                Word.OK -> if (answer == 0) shell.act(ActionSignOut(everywhere = false)) else asking = false
                else -> Unit
            }
            return true
        }
        when (word) {
            Word.UP -> if (at > 0) at -= 1
            Word.DOWN -> if (at < lines.lastIndex) at += 1
            Word.OK -> choose()
            // The menu is the left edge of the page: Left goes back, as Back does.
            Word.BACK, Word.LEFT -> shell.back()
            else -> Unit
        }
        return true
    }

    Box(Modifier.fillMaxSize().background(Color.Black).remote(@DontMemoize { press(it) })) {
        Text(words["menu.help"].orEmpty(), Modifier.offset(SUB_EDGE.ref, 60.ref), fontSize = 44.refType, fontWeight = FontWeight.Bold)
        Menu(
            lines.map { MenuLine(words[it.word].orEmpty()) },
            if (asking) -1 else at,
            Modifier.offset((SUB_EDGE - MENU_WORDS).ref, 136.ref),
            width = 560,
            kept = if (asking) at else -1,
        )
        Column(Modifier.offset(720.ref, 136.ref).width(1040.ref)) {
            when (line) {
                Line.MEMBER -> {
                    Fact(words["help.profile_name"].orEmpty(), person?.name.orEmpty())
                    Fact(words["help.account_email"].orEmpty(), account?.email.orEmpty())
                    Fact(words["help.language"].orEmpty(), Locale.forLanguageTag(seen.language).getDisplayLanguage(Locale.forLanguageTag(seen.language)))
                }
                Line.DEVICE -> {
                    val version = context.packageManager.getPackageInfo(context.packageName, 0).versionName
                    Fact(words["help.app_version"].orEmpty(), version.orEmpty())
                    Fact(words["help.software"].orEmpty(), "Android ${Build.VERSION.RELEASE}")
                    Fact(words["help.model"].orEmpty(), Build.MODEL)
                }
                Line.NETWORK -> Fact(words["help.server"].orEmpty(), seen.server)
                Line.CENTRE -> Hint(words["help.centre.hint"].orEmpty())
                Line.SETTINGS -> Hint(words["help.settings.hint"].orEmpty())
                Line.CONTACT -> {
                    val address = hello?.brand?.supportEmail.orEmpty()
                    if (address.isEmpty()) {
                        Hint(words["help.contact.none"].orEmpty())
                    } else {
                        Fact(words["help.contact.hint"].orEmpty(), address)
                    }
                }
                Line.CHECK -> Hint(checked ?: words["help.check.hint"].orEmpty())
                Line.RELOAD -> Hint(words["help.reload.hint"].orEmpty())
                Line.EXIT -> Hint(words["help.exit.hint"].orEmpty())
                Line.SIGN_OUT -> {
                    Hint(words["help.sign_out.hint"].orEmpty())
                    if (asking) {
                        Row(Modifier.padding(top = 36.ref), horizontalArrangement = Arrangement.spacedBy(20.ref)) {
                            Pill(words["menu.sign_out"].orEmpty(), answer == 0)
                            Pill(words["common.cancel"].orEmpty(), answer == 1)
                        }
                    }
                }
                Line.BACK -> Unit
            }
        }
    }
}
