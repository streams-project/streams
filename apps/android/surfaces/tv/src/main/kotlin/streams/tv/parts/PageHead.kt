package streams.tv

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.offset
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import streams.shared.ref
import streams.shared.refType

/** The title and one line of help at the top left of a settings style page. */
@Composable
fun PageHead(title: String, help: String) {
    Box {
        Text(title, Modifier.offset(SUB_EDGE.ref, 74.ref), fontSize = 44.refType, fontWeight = FontWeight.Bold)
        Text(help, Modifier.offset(SUB_EDGE.ref, 148.ref), fontSize = 30.refType)
    }
}
