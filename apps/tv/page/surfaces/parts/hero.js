// The rounded hero card at the top of a page.
(function () {
  "use strict";
  var tv = window.tv;
  var kit = tv.kit;
  var make = kit.make;

  tv.parts.hero = function (lead) {
    var card = make("div", "hero-card");
    var art = make("div", "hero-card-art");
    kit.art(art, tv.parts.wide(lead));
    card.appendChild(art);
    card.appendChild(make("div", "hero-card-shade"));
    var words = make("div", "hero-card-text");
    words.appendChild(make("h1", "", lead.name));
    var facts = make("p", "facts", "");
    var text = make("p", "overview", "");
    var play = make("span", "button large");
    play.appendChild(tv.parts.icon("play", 34));
    play.appendChild(make("span", "button-word", "Play"));
    var info = make("span", "button large quiet", "More Info");
    words.appendChild(facts);
    words.appendChild(text);
    words.appendChild(play);
    words.appendChild(info);
    card.appendChild(words);
    tv.parts.details(lead, function (detail) {
      tv.parts.facts(facts, detail);
      text.textContent = detail.overview || "";
    });
    return {
      node: card,
      draw: function (shown, focus) {
        card.className = shown ? "hero-card" : "hero-card off";
        play.className = focus === 0 ? "button large on" : "button large";
        info.className = focus === 1 ? "button large quiet on" : "button large quiet";
      }
    };
  };
})();
