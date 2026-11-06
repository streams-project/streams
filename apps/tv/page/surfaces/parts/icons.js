// One icon, by name, from the shared set. It takes the text colour of whatever holds it.
(function () {
  "use strict";
  window.tv.parts.icon = function (name, size) {
    var holder = document.createElement("span");
    holder.className = "icon";
    holder.style.width = size + "px";
    holder.style.height = size + "px";
    holder.innerHTML = '<svg viewBox="0 0 24 24" width="' + size + '" height="' + size + '">' + (window.tvIcons[name] || "") + "</svg>";
    return holder;
  };
})();
