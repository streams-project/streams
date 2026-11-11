// The top bar of every signed in page: who is watching, search, the tabs and the mark.
(function () {
  "use strict";
  var tv = window.tv;
  var kit = tv.kit;
  var TABS = 5;

  function address(item) {
    return item.screen === "home" ? "/home" : "/" + item.screen;
  }

  tv.parts.topbar = function (navItems, active, person) {
    var bar = kit.make("div", "top");
    var who = kit.make("span", "top-who");
    // 48 is the bar's face (note 188); its corner is the same 16% share as every other face.
    who.appendChild(tv.parts.avatar(person, 48));
    who.appendChild(tv.parts.icon("caret-down", 22));
    bar.appendChild(who);
    var search = kit.make("span", "top-search");
    search.appendChild(tv.parts.icon("search", 34));
    bar.appendChild(search);
    var line = kit.make("div", "top-tabs");
    var nav = [];
    var tabs = [];
    for (var i = 0; i < (navItems || []).length && nav.length < TABS; i += 1) {
      if (navItems[i].screen === "genre") continue;
      nav.push(navItems[i]);
      tabs.push(kit.make("span", "top-tab", navItems[i].label));
      line.appendChild(tabs[tabs.length - 1]);
    }
    bar.appendChild(line);
    var mark = kit.make("img", "top-mark");
    mark.src = "/mark.svg";
    mark.alt = "";
    bar.appendChild(mark);
    var at = 0;
    for (var n = 0; n < nav.length; n += 1) {
      if (nav[n].screen === active) at = n;
    }
    return {
      node: bar,
      home: at,
      count: tabs.length,
      draw: function (focus) {
        for (var t = 0; t < tabs.length; t += 1) {
          var name = "top-tab";
          if (t === at) name += " active";
          if (t === focus) name += " on";
          tabs[t].className = name;
        }
      },
      open: function (index) {
        if (nav[index]) kit.go(address(nav[index]));
      }
    };
  };
})();
