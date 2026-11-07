package streams.tv

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import com.google.zxing.BarcodeFormat
import com.google.zxing.qrcode.QRCodeWriter
import streams.shared.ref

/** A code a phone can scan, on white. */
@Composable
fun QrCode(text: String, modifier: Modifier = Modifier) {
    val grid = remember(text) { QRCodeWriter().encode(text, BarcodeFormat.QR_CODE, 0, 0) }
    Box(modifier.background(Color.White, RoundedCornerShape(12.ref)).padding(24.ref)) {
        Canvas(Modifier.fillMaxSize()) {
            val cell = size.width / grid.width
            for (y in 0 until grid.height) {
                for (x in 0 until grid.width) {
                    if (grid.get(x, y)) {
                        drawRect(Color.Black, Offset(x * cell, y * cell), Size(cell + 0.5f, cell + 0.5f))
                    }
                }
            }
        }
    }
}
