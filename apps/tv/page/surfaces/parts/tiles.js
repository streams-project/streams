// What a title looks like in a row: the wide picture and the tall poster.
(function () {
  "use strict";
  var tv = window.tv;
  var kit = tv.kit;

  function image(images, role) {
    for (var i = 0; images && i < images.length; i += 1) {
      if (images[i].role === role) return images[i].url;
    }
    return null;
  }

  tv.parts.wide = function (tile) {
    return image(tile.images, "backdrop") || image(tile.images, "still") || image(tile.images, "poster");
  };

  tv.parts.poster = function (tile) {
    var node = kit.make("div", "poster");
    var url = image(tile.images, "poster") || tv.parts.wide(tile);
    kit.art(node, url);
    if (!url) node.appendChild(kit.make("span", "poster-name", tile.name));
    return node;
  };
})();
