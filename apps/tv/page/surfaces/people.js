// Who is watching, and the PIN of a locked profile. The PIN goes to the core and is never kept.
(function () {
  "use strict";
  var tv = window.tv;
  var kit = tv.kit;
  var make = kit.make;

  function pin(root, person, leave) {
    kit.empty(root);
    var digits = [0, 0, 0, 0];
    var at = 0;
    var panel = make("div", "people");
    panel.appendChild(make("h1", "", person.name));
    panel.appendChild(make("p", "", "Enter the PIN. Up and down change a number."));
    var row = make("div", "pin");
    var cells = [];
    for (var i = 0; i < digits.length; i += 1) {
      cells.push(make("span", "pin-digit", "0"));
      row.appendChild(cells[i]);
    }
    panel.appendChild(row);
    root.appendChild(panel);
    function mark() {
      for (var n = 0; n < cells.length; n += 1) {
        cells[n].textContent = String(digits[n]);
        cells[n].className = n === at ? "pin-digit on" : "pin-digit";
      }
    }
    mark();
    return function (word) {
      if (word === "left" && at > 0) at -= 1;
      else if (word === "right" && at < digits.length - 1) at += 1;
      else if (word === "up") digits[at] = (digits[at] + 1) % 10;
      else if (word === "down") digits[at] = (digits[at] + 9) % 10;
      else if (/^[0-9]$/.test(word)) {
        digits[at] = Number(word);
        if (at < digits.length - 1) at += 1;
      } else if (word === "ok") {
        kit.act({ action: "enter_profile", profile: person.id, pin: digits.join("") });
        return true;
      } else if (word === "back") {
        leave();
        return true;
      } else return false;
      mark();
      return true;
    };
  }

  tv.surfaces.people = function people(root, list, setKeys) {
    var at = 0;
    var panel = make("div", "people");
    panel.appendChild(make("h1", "", "Who's watching?"));
    var row = make("div", "people-row");
    var faces = [];
    for (var i = 0; i < list.length; i += 1) {
      var person = make("div", "person");
      var letter = list[i].name ? list[i].name.charAt(0).toUpperCase() : "?";
      person.appendChild(make("span", list[i].kids ? "face kids" : "face", letter));
      person.appendChild(make("span", "person-name", list[i].name));
      row.appendChild(person);
      faces.push(person);
    }
    panel.appendChild(row);
    root.appendChild(panel);
    function mark() {
      for (var n = 0; n < faces.length; n += 1) faces[n].className = n === at ? "person on" : "person";
    }
    mark();
    return function (word) {
      if (word === "left" && at > 0) at -= 1;
      else if (word === "right" && at < faces.length - 1) at += 1;
      else if (word === "ok" && list[at]) {
        if (list[at].has_pin) {
          setKeys(pin(root, list[at], function () {
            kit.empty(root);
            setKeys(people(root, list, setKeys));
          }));
        } else {
          kit.act({ action: "enter_profile", profile: list[at].id });
        }
        return true;
      } else return false;
      mark();
      return true;
    };
  };
})();
