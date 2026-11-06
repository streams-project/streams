// A small QR code encoder: byte mode, error correction level L, versions 1 to 9, which holds 230
// bytes and more than any address this page shows. Plain enough for the oldest sets.
(function () {
  "use strict";
  var EC_PER_BLOCK = [0, 7, 10, 15, 20, 26, 18, 20, 24, 30];
  var BLOCKS = [0, 1, 1, 1, 1, 1, 2, 2, 2, 2];
  var DATA_PER_BLOCK = [0, 19, 34, 55, 80, 108, 68, 78, 97, 116];
  var ALIGN = [[], [], [6, 18], [6, 22], [6, 26], [6, 30], [6, 34], [6, 22, 38], [6, 24, 42], [6, 26, 46]];
  var VERSION_BITS = [0, 0, 0, 0, 0, 0, 0, 0x07c94, 0x085bc, 0x09a99];

  var EXP = [];
  var LOG = [];
  (function () {
    var value = 1;
    for (var i = 0; i < 255; i += 1) {
      EXP[i] = value;
      LOG[value] = i;
      value <<= 1;
      if (value & 256) value ^= 0x11d;
    }
  })();

  function times(a, b) {
    return a === 0 || b === 0 ? 0 : EXP[(LOG[a] + LOG[b]) % 255];
  }

  function generator(degree) {
    var poly = [1];
    for (var i = 0; i < degree; i += 1) {
      var next = [];
      for (var j = 0; j <= poly.length; j += 1) next[j] = 0;
      for (var k = 0; k < poly.length; k += 1) {
        next[k] ^= poly[k];
        next[k + 1] ^= times(poly[k], EXP[i]);
      }
      poly = next;
    }
    return poly;
  }

  function remainder(data, degree) {
    var poly = generator(degree);
    var rest = [];
    for (var i = 0; i < degree; i += 1) rest[i] = 0;
    for (var d = 0; d < data.length; d += 1) {
      var factor = data[d] ^ rest[0];
      rest.shift();
      rest.push(0);
      for (var j = 0; j < degree; j += 1) rest[j] ^= times(poly[j + 1], factor);
    }
    return rest;
  }

  function bytesOf(text) {
    var raw = unescape(encodeURIComponent(text));
    var bytes = [];
    for (var i = 0; i < raw.length; i += 1) bytes.push(raw.charCodeAt(i));
    return bytes;
  }

  function codewords(bytes, version) {
    var capacity = BLOCKS[version] * DATA_PER_BLOCK[version];
    var bits = [];
    function put(value, length) {
      for (var i = length - 1; i >= 0; i -= 1) bits.push((value >>> i) & 1);
    }
    put(4, 4);
    put(bytes.length, 8);
    for (var i = 0; i < bytes.length; i += 1) put(bytes[i], 8);
    put(0, Math.min(4, capacity * 8 - bits.length));
    while (bits.length % 8) bits.push(0);
    var data = [];
    for (var b = 0; b < bits.length; b += 8) {
      var byte = 0;
      for (var n = 0; n < 8; n += 1) byte = (byte << 1) | bits[b + n];
      data.push(byte);
    }
    for (var pad = 0xec; data.length < capacity; pad ^= 0xec ^ 0x11) data.push(pad);
    var blocks = [];
    var checks = [];
    for (var k = 0; k < BLOCKS[version]; k += 1) {
      var piece = data.slice(k * DATA_PER_BLOCK[version], (k + 1) * DATA_PER_BLOCK[version]);
      blocks.push(piece);
      checks.push(remainder(piece, EC_PER_BLOCK[version]));
    }
    var out = [];
    for (var c = 0; c < DATA_PER_BLOCK[version]; c += 1) {
      for (var d = 0; d < blocks.length; d += 1) out.push(blocks[d][c]);
    }
    for (var e = 0; e < EC_PER_BLOCK[version]; e += 1) {
      for (var f = 0; f < checks.length; f += 1) out.push(checks[f][e]);
    }
    return out;
  }

  function masked(mask, x, y) {
    switch (mask) {
      case 0: return (x + y) % 2 === 0;
      case 1: return y % 2 === 0;
      case 2: return x % 3 === 0;
      case 3: return (x + y) % 3 === 0;
      case 4: return (Math.floor(x / 3) + Math.floor(y / 2)) % 2 === 0;
      case 5: return (x * y) % 2 + (x * y) % 3 === 0;
      case 6: return ((x * y) % 2 + (x * y) % 3) % 2 === 0;
      default: return ((x + y) % 2 + (x * y) % 3) % 2 === 0;
    }
  }

  function blank(size) {
    var grid = [];
    for (var y = 0; y < size; y += 1) {
      grid[y] = [];
      for (var x = 0; x < size; x += 1) grid[y][x] = false;
    }
    return grid;
  }

  function build(version, words, mask) {
    var size = 17 + version * 4;
    var dark = blank(size);
    var fixed = blank(size);
    function set(x, y, value) {
      dark[y][x] = !!value;
      fixed[y][x] = true;
    }
    function finder(cx, cy) {
      for (var dy = -4; dy <= 4; dy += 1) {
        for (var dx = -4; dx <= 4; dx += 1) {
          var x = cx + dx;
          var y = cy + dy;
          if (x < 0 || y < 0 || x >= size || y >= size) continue;
          var ring = Math.max(Math.abs(dx), Math.abs(dy));
          set(x, y, ring !== 2 && ring !== 4);
        }
      }
    }
    for (var i = 0; i < size; i += 1) {
      set(6, i, i % 2 === 0);
      set(i, 6, i % 2 === 0);
    }
    finder(3, 3);
    finder(size - 4, 3);
    finder(3, size - 4);
    var spots = ALIGN[version];
    for (var a = 0; a < spots.length; a += 1) {
      for (var b = 0; b < spots.length; b += 1) {
        var first = a === 0 && b === 0;
        var second = a === 0 && b === spots.length - 1;
        var third = a === spots.length - 1 && b === 0;
        if (first || second || third) continue;
        for (var dy = -2; dy <= 2; dy += 1) {
          for (var dx = -2; dx <= 2; dx += 1) {
            set(spots[a] + dx, spots[b] + dy, Math.max(Math.abs(dx), Math.abs(dy)) !== 1);
          }
        }
      }
    }
    function format() {
      var value = (1 << 3) | mask;
      var rest = value;
      for (var n = 0; n < 10; n += 1) rest = (rest << 1) ^ ((rest >>> 9) * 0x537);
      var bits = ((value << 10) | rest) ^ 0x5412;
      function bit(n) { return (bits >>> n) & 1; }
      for (var i = 0; i <= 5; i += 1) set(8, i, bit(i));
      set(8, 7, bit(6));
      set(8, 8, bit(7));
      set(7, 8, bit(8));
      for (var j = 9; j < 15; j += 1) set(14 - j, 8, bit(j));
      for (var k = 0; k < 8; k += 1) set(size - 1 - k, 8, bit(k));
      for (var l = 8; l < 15; l += 1) set(8, size - 15 + l, bit(l));
      set(8, size - 8, 1);
    }
    format();
    if (version >= 7) {
      for (var v = 0; v < 18; v += 1) {
        var on = (VERSION_BITS[version] >>> v) & 1;
        var far = size - 11 + v % 3;
        var near = Math.floor(v / 3);
        set(far, near, on);
        set(near, far, on);
      }
    }
    var index = 0;
    var total = words.length * 8;
    for (var right = size - 1; right >= 1; right -= 2) {
      if (right === 6) right = 5;
      for (var step = 0; step < size; step += 1) {
        for (var j = 0; j < 2; j += 1) {
          var x = right - j;
          var up = ((right + 1) & 2) === 0;
          var y = up ? size - 1 - step : step;
          if (fixed[y][x]) continue;
          var value = index < total && ((words[index >>> 3] >>> (7 - (index & 7))) & 1) === 1;
          index += 1;
          dark[y][x] = masked(mask, x, y) ? !value : value;
        }
      }
    }
    return dark;
  }

  // The runs and two by two blocks of the standard's penalty, and how far dark is from half.
  function penalty(grid) {
    var size = grid.length;
    var score = 0;
    var count = 0;
    for (var a = 0; a < size; a += 1) {
      var rowRun = 1;
      var colRun = 1;
      for (var b = 0; b < size; b += 1) {
        if (grid[a][b]) count += 1;
        if (b > 0) {
          rowRun = grid[a][b] === grid[a][b - 1] ? rowRun + 1 : 1;
          colRun = grid[b][a] === grid[b - 1][a] ? colRun + 1 : 1;
          if (rowRun === 5) score += 3;
          else if (rowRun > 5) score += 1;
          if (colRun === 5) score += 3;
          else if (colRun > 5) score += 1;
        }
        if (a > 0 && b > 0 && grid[a][b] === grid[a - 1][b] && grid[a][b] === grid[a][b - 1] &&
            grid[a][b] === grid[a - 1][b - 1]) score += 3;
      }
    }
    return score + Math.floor(Math.abs(count * 20 / (size * size) - 10)) * 10;
  }

  // The dark modules as rows of booleans, or null when the text is too long.
  window.tvQr = function (text) {
    var bytes = bytesOf(text);
    var version = 1;
    while (version <= 9 && BLOCKS[version] * DATA_PER_BLOCK[version] < bytes.length + 2) version += 1;
    if (version > 9) return null;
    var words = codewords(bytes, version);
    var best = null;
    var lowest = Infinity;
    for (var mask = 0; mask < 8; mask += 1) {
      var grid = build(version, words, mask);
      var score = penalty(grid);
      if (score < lowest) {
        lowest = score;
        best = grid;
      }
    }
    return best;
  };
})();
