// Small pure curve helpers. No device, screen or clock in here.

export function clamp01(x: number): number {
  return x < 0 ? 0 : x > 1 ? 1 : x;
}

export function lerp(a: number, b: number, t: number): number {
  return a + (b - a) * t;
}

export function smooth(edge0: number, edge1: number, x: number): number {
  var t = clamp01((x - edge0) / (edge1 - edge0));
  return t * t * (3 - 2 * t);
}

// Piecewise linear lookup on a flat list: x0, y0, x1, y1, ... with x rising.
export function table(points: number[], x: number): number {
  var n = points.length;
  if (x <= points[0]) return points[1];
  if (x >= points[n - 2]) return points[n - 1];
  for (var i = 2; i < n; i += 2) {
    if (x <= points[i]) {
      var x0 = points[i - 2];
      var x1 = points[i];
      return lerp(points[i - 1], points[i + 1], (x - x0) / (x1 - x0));
    }
  }
  return points[n - 1];
}

// Cubic Bezier easing with fixed ends (0, 0) and (1, 1), solved on x.
export function bezier(x1: number, y1: number, x2: number, y2: number): (x: number) => number {
  function at(a: number, b: number, t: number): number {
    var u = 1 - t;
    return 3 * u * u * t * a + 3 * u * t * t * b + t * t * t;
  }
  function slope(a: number, b: number, t: number): number {
    var u = 1 - t;
    return 3 * u * u * a + 6 * u * t * (b - a) + 3 * t * t * (1 - b);
  }
  return function (x: number): number {
    if (x <= 0) return 0;
    if (x >= 1) return 1;
    var t = x;
    for (var i = 0; i < 8; i++) {
      var d = slope(x1, x2, t);
      if (d > -1e-5 && d < 1e-5) break;
      t -= (at(x1, x2, t) - x) / d;
      if (t < 0) t = 0;
      if (t > 1) t = 1;
    }
    return at(y1, y2, t);
  };
}

// Small seeded generator so a seed gives the same output on every device.
export function rng(seed: number): () => number {
  var s = seed >>> 0;
  return function (): number {
    s = (s + 0x6d2b79f5) >>> 0;
    var t = s;
    t = mul(t ^ (t >>> 15), t | 1);
    t ^= t + mul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

function mul(a: number, b: number): number {
  var ah = (a >>> 16) & 0xffff;
  var al = a & 0xffff;
  var bh = (b >>> 16) & 0xffff;
  var bl = b & 0xffff;
  return (al * bl + (((ah * bl + al * bh) << 16) >>> 0)) | 0;
}
