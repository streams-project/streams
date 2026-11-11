// A profile's picture, or its first letter when it has none. Profiles, the top bar and Settings all
// draw this one part, so a change of avatar shows everywhere.
(function () {
  "use strict";
  var kit = window.tv.kit;
  window.tv.parts.avatar = function (person, size, name) {
    var node = kit.make("span", "avatar" + (name ? " " + name : ""));
    node.style.width = size + "px";
    node.style.height = size + "px";
    // A face keeps its shape as it grows: the corner is a share of the size, not a fixed number.
    node.style.borderRadius = Math.round(size * 0.16) + "px";
    if (person && person.kids) node.className += " kids";
    if (person && person.avatar && /^(https?:|\/)/.test(person.avatar)) {
      kit.art(node, person.avatar);
    } else {
      node.style.lineHeight = size + "px";
      node.style.fontSize = Math.round(size * 0.46) + "px";
      node.textContent = person && person.name ? person.name.charAt(0).toUpperCase() : "";
    }
    return node;
  };
})();
