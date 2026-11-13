#include "paint.h"

#include <cmath>

namespace streams::paint {

namespace {

// floor(x + 0.5) for a positive value, which is what `Math.round(float)` and Compose's `Dp` to
// pixel conversion both do. `std::lround` rounds half away from zero, which is the same thing on
// the positive side and the side every measurement here is on.
int rounded(float value) { return static_cast<int>(std::lround(value)); }

int clampByte(int v) { return v < 0 ? 0 : (v > 255 ? 255 : v); }

// One colour at one share, rounded as Compose rounds an alpha: to the nearest 255th.
// `Ink.java:shade`.
std::uint32_t shade(std::uint32_t rgb, int hundredths) {
    int alpha = clampByte(rounded(255.0f * hundredths / 100.0f));
    return (static_cast<std::uint32_t>(alpha) << 24) | (rgb & 0x00FFFFFFu);
}

} // namespace

namespace Ref {

int px(int ref, float density) { return rounded(dp(ref) * density); }

} // namespace Ref

namespace Ink {

std::uint32_t white(int hundredths) { return shade(0xFFFFFFu, hundredths); }

std::uint32_t black(int hundredths) { return shade(0x000000u, hundredths); }

std::uint32_t dim(std::uint32_t color, int hundredths) {
    int alpha = clampByte(rounded(static_cast<float>(alphaOf(color)) * hundredths / 100.0f));
    return (static_cast<std::uint32_t>(alpha) << 24) | (color & 0x00FFFFFFu);
}

} // namespace Ink

namespace Corner {

// The five corners are the brand's own, scaled. They are functions and not constants because the
// brand numbers are read out of `brand.json` by the build on every other platform, and a constant
// here would be a fourth copy of a number that already has three homes. Until a C++ side of the
// brand generator exists they are written out, and `paint_test.cpp` holds each one against
// `brand.json` directly.
int tv(int brand) { return rounded(brand * TV_SCALE); }

int card() { return tv(14); }   // shape.card_radius
int panel() { return tv(24); }  // shape.panel_radius
int field() { return tv(12); }  // shape.field_radius

int key() { return field() * 2 / 3; }
int chip() { return field() / 2; }

int inside(int outer, int inset) {
    int inner = outer - inset;
    return inner > chip() ? inner : chip();
}

} // namespace Corner

namespace Safe {

int over(int x, int y, int wide, int high) {
    int worst = 0;
    int edges[4] = {
        LEFT - x,                      // how far past the left line the part starts
        TOP - y,                       // past the top line
        (x + wide) - (Ref::W - RIGHT), // past the right line
        (y + high) - (Ref::H - BOTTOM) // past the bottom line
    };
    for (int edge : edges) {
        if (edge > worst) {
            worst = edge;
        }
    }
    return worst;
}

bool holds(int x, int y, int wide, int high) { return over(x, y, wide, high) == 0; }

} // namespace Safe

namespace Glide {

namespace {

// x and y on the unit cubic Bezier from (0,0) to (1,1) at parameter t. The two end points are
// fixed, so only the two control points appear.
float bezier(float t, float a, float b) {
    float u = 1.0f - t;
    return 3.0f * u * u * t * a + 3.0f * u * t * t * b + t * t * t;
}

} // namespace

float at(float x) {
    if (x <= 0.0f) {
        return 0.0f;
    }
    if (x >= 1.0f) {
        return 1.0f;
    }
    // x is monotone in t for control points inside [0, 1], so bisection cannot miss. Sixteen
    // halvings take the parameter to 1/65536, which is finer than a reference pixel is in a 1920
    // wide canvas by a factor of 34.
    float low = 0.0f;
    float high = 1.0f;
    for (int i = 0; i < 16; ++i) {
        float mid = (low + high) * 0.5f;
        if (bezier(mid, X1, X2) < x) {
            low = mid;
        } else {
            high = mid;
        }
    }
    return bezier((low + high) * 0.5f, Y1, Y2);
}

float remaining(int elapsedMs) {
    if (elapsedMs <= 0) {
        return static_cast<float>(RISE);
    }
    if (elapsedMs >= RISE_MS) {
        return 0.0f;
    }
    float done = at(static_cast<float>(elapsedMs) / static_cast<float>(RISE_MS));
    return static_cast<float>(RISE) * (1.0f - done);
}

} // namespace Glide

} // namespace streams::paint
