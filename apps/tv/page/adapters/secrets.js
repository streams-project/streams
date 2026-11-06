// The secrets port, on local storage. The prefix is not the website's, so this page cannot inherit
// a session from the same browser.
(function () {
  "use strict";
  var PREFIX = "tv:secret:";
  window.tv.plug("secrets", {
    get: function (name) { return window.localStorage.getItem(PREFIX + name); },
    put: function (name, value) { window.localStorage.setItem(PREFIX + name, value); },
    drop: function (name) { window.localStorage.removeItem(PREFIX + name); }
  });
})();
