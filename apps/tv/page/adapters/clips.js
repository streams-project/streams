// The clips port, with nothing behind it yet. Trailers get their own index, and until then a focused
// title shows its picture. Plugging in the real source changes this file and nothing else.
(function () {
  "use strict";
  window.tv.plug("clips", {
    find: function () {}
  });
})();
