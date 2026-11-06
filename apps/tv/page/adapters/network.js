// The network port, on XMLHttpRequest, because old sets have no fetch.
(function () {
  "use strict";
  window.tv.plug("network", {
    send: function (effect, done) {
      var xhr = new XMLHttpRequest();
      xhr.onload = function () { done(xhr.status, xhr.responseText || ""); };
      xhr.onerror = function () { done(0, "", "offline"); };
      xhr.ontimeout = function () { done(0, "", "timeout"); };
      xhr.onabort = function () { done(0, "", "cancelled"); };
      try {
        xhr.open(effect.method, effect.url, true);
        xhr.timeout = effect.deadline_ms;
        for (var name in effect.headers) xhr.setRequestHeader(name, effect.headers[name]);
        xhr.send(effect.body === undefined ? null : effect.body);
      } catch (thrown) {
        console.log("a request did not start", String(thrown));
        done(0, "", "offline");
      }
    }
  });
})();
