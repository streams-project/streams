// An age mark. The server decides what it says and its tone, and the screen decides the shape:
// plain text, a chip, or the ticket, which stands where a round badge would otherwise go.
(function () {
  "use strict";
  var kit = window.tv.kit;
  window.tv.parts.age = function (mark, shape) {
    if (!mark) return null;
    return kit.make("span", "age age-" + shape + " tone-" + mark.tone, mark.label);
  };
})();
