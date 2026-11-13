// The numbers every drawn thing on the remote surface needs, before any Windows is involved.
//
// Note 223 settles how this app draws on an Xbox: a UWP C++/WinRT app, one window, one swap chain,
// and Win2D's immediate-mode drawing (`CanvasDrawingSession`) over Direct2D. Win2D gives us a
// filled rounded rectangle, a stroked one, a linear gradient with its stops where we put them, a
// text layout and a bitmap -- which is the whole vocabulary of
// `apps/android/surfaces/tv/src/main/java/streams/tv/views/`. What Win2D does *not* give us is
// anything above that: no layout, no reference scale, no easing curve, no brand. A Views surface
// gets those from Android (`GradientDrawable`, `ValueAnimator`, `PathInterpolator`, `density`); an
// immediate-mode surface has to own them.
//
// This file is that ownership, and it is deliberately the part that needs no platform at all:
// integer and float arithmetic over measured numbers, so it is built and checked on whatever
// machine builds the tree. No WinRT, no Win2D, no header that only exists on Windows. The Win2D
// painter above it is five calls per part and is written once a Windows machine exists; it cannot
// be written here and is not guessed at here.
//
// The Java twins, file by file:
//
//     Ref    views/Ref.java        the half-dp reference scale
//     Ink    views/Ink.java        the six brand colours and the three shares
//     Corner views/Round.java      the brand's corners at television size
//     Glide  browse/RowsView.java  the easing curve, and browse/BrowseView.java's 320 ms rise
//     Safe   -- new --             Microsoft's TV-safe band, which Android gave us for free
//
// Every number here is either measured on the Netflix reference, read out of `brand.json`, or
// quoted from a Microsoft page with its date. `paint_test.cpp` holds the brand numbers against
// `brand.json` itself rather than against a second copy, so a brand change cannot pass unnoticed.
#ifndef STREAMS_XBOX_PAINT_H
#define STREAMS_XBOX_PAINT_H

#include <cstdint>

namespace streams::paint {

// The half-dp reference scale. The twin of `views/Ref.java` and of
// `apps/apple/Sources/SurfacesTelevision/Measure.swift`.
//
// Every television lays out at 960 by 540 dp and the screen references measure 1920 by 1080, so a
// reference pixel is half a dp and every measurement in the code stays the number that was
// measured off the capture.
namespace Ref {

// The reference canvas.
constexpr int W = 1920;
constexpr int H = 1080;

// The layout it really draws in, in dp. Not used in the arithmetic below; it is here so the
// derivation reads off the code, as `Ref.java` has it.
constexpr int LAYOUT_W = 960;
constexpr int LAYOUT_H = 540;

// A reference measurement in dp: half of it, exactly, keeping the half.
constexpr float dp(int ref) { return ref / 2.0f; }

// A reference font size, in dp. The twin of `shared/Measure.kt:15`'s `refType`. On Android this is
// sp, so a language can ask for larger text; a Win2D `CanvasTextFormat.FontSize` is in DIPs and
// carries no such multiplier, which is note 223's open item 6.
constexpr float type(int ref) { return ref / 2.0f; }

// A reference measurement in pixels on a screen of this density, rounded as Compose rounds a `Dp`
// and as `Ref.java:px` rounds it: floor(x + 0.5) for a positive value. A sum is converted once,
// never summed after converting -- `px(72 + 904 + 19, d)`, not three roundings added.
int px(int ref, float density);

// What one reference pixel is on an Xbox, which is the number that made this surface portable.
//
// Microsoft: "Xbox One renders your app at 1080p (1920 x 1080 pixels)" and "UWP apps will always
// scale to 1080p", and "they are scaled at a default of 200% for XAML apps". A UWP app therefore
// lays out in 960 x 540 effective pixels over a 1920 x 1080 panel. Our reference pixel is half a
// dp and an effective pixel is a dp, so:
//
//     one reference pixel  =  one physical pixel on an Xbox, exactly
//     one effective pixel  =  two reference pixels
//
// Source: Designing for Xbox and TV,
// https://learn.microsoft.com/en-us/windows/uwp/xbox-apps/designing-for-tv (ms.date 2020-09-24,
// updated_at 2026-03-13, read 2026-11-12). The page is archived (`is_archived: true`) and is the
// only Microsoft page that states the figure.
constexpr float XBOX_SCALE = 2.0f;

// A reference measurement in Win2D's own unit, device-independent pixels, at Xbox's 200% scale.
// This is `dp`, named for where it is used: a `CanvasDrawingSession` takes DIPs.
constexpr float dip(int ref) { return dp(ref); }

} // namespace Ref

// The brand's colours, as the 0xAARRGGBB integers a painter draws with. The twin of
// `views/Ink.java`, whose own comment explains why a second home is safe when a test holds it: the
// test here reads `brand.json` and no Java at all, so this is a reading of the brand rather than a
// copy of a copy.
namespace Ink {

// The page behind everything. `brand.json` colors.background.
constexpr std::uint32_t BACK = 0xFF0B0B10u;

// A panel lifted off the page: the hero. colors.surface.
constexpr std::uint32_t PLATE = 0xFF16161Du;

// A tile before its artwork arrives. colors.raised.
constexpr std::uint32_t RAISED = 0xFF24242Eu;

// Words that are not the point: a fact, a dot between facts. colors.muted.
constexpr std::uint32_t MUTED = 0xFFA6A6B3u;

// A profile's plate. colors.accent.
constexpr std::uint32_t ACCENT = 0xFF2F6BFFu;

// A kid's profile plate. colors.join.
constexpr std::uint32_t JOIN = 0xFFE5352Bu;

// White at a share of its strength, the share given in hundredths. `Ink.java:white`.
//
// Hundredths and not a float for the reason `Ink.java` gives: `white(28)` is the 0.28 written at
// `tv/browse/HeroCard.kt:57`, so a reader finds the measurement on the line. The rounding is
// Compose's, to the nearest 255th.
std::uint32_t white(int hundredths);

// Black at a share of its strength: the two shades over the hero, the plate behind a badge.
std::uint32_t black(int hundredths);

// A colour at a share of the strength it already has, for a whole row or card drawn dim.
std::uint32_t dim(std::uint32_t color, int hundredths);

// The four channels, for a painter that wants floats. Win2D takes a `Windows::UI::Color`, which is
// four bytes, so these exist for a Direct2D `D2D1_COLOR_F` and for a test, not for Win2D itself.
constexpr std::uint8_t alphaOf(std::uint32_t c) { return static_cast<std::uint8_t>(c >> 24); }
constexpr std::uint8_t redOf(std::uint32_t c) { return static_cast<std::uint8_t>(c >> 16); }
constexpr std::uint8_t greenOf(std::uint32_t c) { return static_cast<std::uint8_t>(c >> 8); }
constexpr std::uint8_t blueOf(std::uint32_t c) { return static_cast<std::uint8_t>(c); }

} // namespace Ink

// The brand's corners at television size, in reference pixels. The twin of `views/Round.java`,
// which reads them from `BuildConfig` and scales them; here they are read from `brand.json` by the
// test and scaled by the same rule.
namespace Corner {

// How much larger a corner is on a television than on a phone. `tv/app/Round.kt:18`.
constexpr float TV_SCALE = 1.5f;

// A brand corner at television size, rounded as `Round.kt:20` rounds it.
int tv(int brand);

// Posters, stills, episode pictures, the picture grid. brand.json shape.card_radius.
int card();

// The hero, the landing cards, menus, panels. shape.panel_radius.
int panel();

// Fields, boxes of text, the code plate, PIN boxes. shape.field_radius.
int field();

// Keys and letters: smaller than a field. `Round.kt:37`'s two thirds.
int key();

// Badges and small labels. `Round.kt:40`'s half.
int chip();

// A corner that sits inside another, `inset` from its edge. `Round.kt:43`.
int inside(int outer, int inset);

// The page margin on a television, left and right, in reference pixels. `Round.kt:50`. One number,
// written once, which is what makes it changeable.
constexpr int PAGE_EDGE = 72;

// Pages without the top bar keep their words on this column. `Round.kt:53`.
constexpr int SUB_EDGE = 130;

} // namespace Corner

// Microsoft's TV-safe band, in reference pixels, and whether a part is inside it.
//
// This is the one part of the drawing that Android handed us and an Xbox will not. On a television
// Android gives an app the whole panel and the launcher's own overscan guidance; a UWP app instead
// gets a window that the system has already inset, unless it asks for the whole panel with
// `ApplicationView::SetDesiredBoundsMode(ApplicationViewBoundsMode::UseCoreWindow)` -- and having
// asked, it owns the margin itself. So the margin becomes arithmetic in the surface, and it is
// here.
//
// Microsoft, on the same archived page as `Ref::XBOX_SCALE`: "we recommend that you draw only
// non-essential visuals within 5% of the screen edges ... you should avoid putting essential UI in
// the following areas: 27 epx from the top and bottom, 48 epx from the left and right sides".
// An effective pixel is two reference pixels, so:
namespace Safe {

// 27 effective pixels, doubled. Essential UI should stay below this from the top and above it from
// the bottom.
constexpr int TOP = 54;
constexpr int BOTTOM = 54;

// 48 effective pixels, doubled.
constexpr int LEFT = 96;
constexpr int RIGHT = 96;

// Is a part, given in reference pixels on the 1920 by 1080 canvas, clear of the band?
bool holds(int x, int y, int wide, int high);

// How far into the band a part reaches, or 0 when it is clear. Positive means over the line, in
// reference pixels; a surface that chooses to overhang (a row drawn to the edge) can say by how
// much rather than pretend it does not.
int over(int x, int y, int wide, int high);

} // namespace Safe

// The surface's one easing curve, and the row rise it drives.
//
// `browse/RowsView.java:57` is `PathInterpolator(0.22f, 0.61f, 0.2f, 1f)` and
// `tv/browse/Browse.kt` holds the same four numbers as a Compose `CubicBezierEasing`. Both are the
// same primitive: a unit cubic Bezier from (0,0) to (1,1) with those two control points, read as
// y for a given x.
//
// It is here because Win2D has no easing and no animator. `CanvasDrawingSession` draws what it is
// told, now; every frame of the 320 ms rise is a position this function computes. On Android the
// toolkit owned the clock and the curve, and on an immediate-mode surface the surface owns both.
namespace Glide {

// The four control-point coordinates, as the two files hold them.
constexpr float X1 = 0.22f;
constexpr float Y1 = 0.61f;
constexpr float X2 = 0.20f;
constexpr float Y2 = 1.00f;

// y at x, for x in [0, 1] and clamped outside it.
//
// Solved by bisection on the curve's own x, to 1/65536 of the parameter, which is a stated method
// rather than a borrowed one: `PathInterpolator` flattens the path into line segments and
// Compose's `CubicBezierEasing` runs a Newton iteration, so the three agree on the shape and may
// differ in the last fraction of a pixel. Note 223 records that as an assumption, not a
// measurement: nothing in this tree has compared them on a screen.
float at(float x);

// How far a row has still to travel, in reference pixels, `elapsed` milliseconds into the rise.
//
// `browse/BrowseView.java:47` holds RISE = 810 and `:50` holds RISE_MS = 320: the row arriving
// slides up from 810 reference pixels below its place and the row leaving slides out the same way,
// both on this curve. The sign is the caller's -- `BrowseView:196` decides which way from which
// way the focus went -- so this returns the distance remaining and never a direction.
constexpr int RISE = 810;
constexpr int RISE_MS = 320;

// At 0 ms this is RISE and at RISE_MS or later it is 0, so a painter can translate by it directly.
float remaining(int elapsedMs);

} // namespace Glide

} // namespace streams::paint

#endif // STREAMS_XBOX_PAINT_H
