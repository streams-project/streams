// The line of facts about a title, in one order on every screen: genre, year, length, age.
(function () {
  "use strict";
  var tv = window.tv;
  var kit = tv.kit;

  function words(detail) {
    var parts = [];
    if (detail.genres && detail.genres.length) parts.push(detail.genres[0]);
    if (detail.year) parts.push(String(detail.year));
    if (detail.seasons && detail.seasons.length) {
      parts.push(detail.seasons.length + (detail.seasons.length > 1 ? " Seasons" : " Season"));
    } else if (detail.runtime_min) {
      parts.push(Math.floor(detail.runtime_min / 60) + "h " + detail.runtime_min % 60 + "m");
    }
    return parts;
  }

  tv.parts.facts = function (node, detail) {
    kit.empty(node);
    var parts = words(detail);
    for (var i = 0; i < parts.length; i += 1) {
      if (i) node.appendChild(kit.make("span", "facts-dot"));
      node.appendChild(kit.make("span", "", parts[i]));
    }
    var age = tv.parts.age(detail.age, "text");
    if (age) {
      if (parts.length) node.appendChild(kit.make("span", "facts-dot"));
      node.appendChild(age);
    }
  };
})();
