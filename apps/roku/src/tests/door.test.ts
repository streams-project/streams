// Where this app's server comes from, and which addresses it will take.
//
// The second half of this file is a mirror of the launcher's bind rule, so it is held the way
// every mirror in this folder is held: `launcher/src/address.rs` is read as **text** and its
// `own_network` and `check_bind` rules are compared with the ones here. Change the Rust and this
// folder goes red, naming that file. (The technique is `src/tests/tier.test.ts`; the bargain is
// the same one: brittle to a rewrite of that file, and better than no guard.)
//
// The first half is this family's own decision, so it is held to what the decision says it does.

import { readFileSync } from "node:fs";
import { join } from "node:path";

import {
  SERVER_KEY,
  SERVER_PARAMETER,
  encryptionOf,
  hostOf,
  ownNetwork,
  serverFrom,
  tidy,
} from "../door";

const ROOT = join(__dirname, "..", "..", "..", "..");
const ADDRESS_RULES = readFileSync(join(ROOT, "launcher", "src", "address.rs"), "utf8");

describe("the address this run starts with", () => {
  it("is the one the channel was launched with", () => {
    const given = serverFrom({ [SERVER_PARAMETER]: "https://one.example" }, null);
    expect(given).toEqual({ server: "https://one.example", from: "launch" });
  });

  it("is the kept one when the launch carried none", () => {
    expect(serverFrom({}, "https://two.example", "https://bound.example")).toEqual({
      server: "https://two.example",
      from: "kept",
    });
  });

  it("is the launched one even when there is a kept or bound one, so a device can be re-pointed", () => {
    // This is the whole reason the order is this way round. A kept or bound value that won would
    // make a television impossible to move to another server without clearing its registry or
    // rebuilding, and clearing a registry is not something a person with a remote can do.
    const given = serverFrom(
      { [SERVER_PARAMETER]: "https://new.example" },
      "https://old.example",
      "https://bound.example",
    );
    expect(given.server).toBe("https://new.example");
    expect(given.from).toBe("launch");
  });

  it("falls back to the bound build address when neither launch nor kept carried one", () => {
    expect(serverFrom({}, null, "  https://bound.example  ")).toEqual({
      server: "  https://bound.example  ".trim(),
      from: "bound",
    });
    expect(serverFrom({ [SERVER_PARAMETER]: "  " }, " ", "https://bound.example")).toEqual({
      server: "https://bound.example",
      from: "bound",
    });
  });

  it("is nothing when no place has one, and says so rather than guessing", () => {
    expect(serverFrom({}, null)).toEqual({ server: "", from: "none" });
    expect(serverFrom({ [SERVER_PARAMETER]: "   " }, "  ", "   ")).toEqual({
      server: "",
      from: "none",
    });
  });

  it("ignores every other launch parameter", () => {
    expect(serverFrom({ contentId: "movie:1", mediaType: "movie" }, null).from).toBe("none");
  });

  it("keeps the address under the shared store key name", () => {
    expect(SERVER_KEY).toBe("app:server");
  });
});

describe("an address is tidied before anything looks at it", () => {
  it("of its spaces and its trailing slashes", () => {
    expect(tidy("  https://one.example/  ")).toBe("https://one.example");
    expect(tidy("https://one.example///")).toBe("https://one.example");
  });
});

describe("the launcher rule this mirrors has not moved", () => {
  it("is where this suite thinks it is", () => {
    expect(ADDRESS_RULES).toContain("pub fn own_network(host: &str) -> bool");
    expect(ADDRESS_RULES).toContain("pub fn check_bind(server: &str, family: Family) -> Result<String>");
  });

  it("names the same networks as its own hop", () => {
    const start = ADDRESS_RULES.indexOf("pub fn own_network");
    const end = ADDRESS_RULES.indexOf("pub fn check_bind", start);
    const body = ADDRESS_RULES.slice(start, end);
    for (const named of [
      '"localhost"',
      '".local"',
      "is_loopback()",
      "is_private()",
      "is_link_local()",
      "is_unique_local()",
      "0xfe80",
    ]) {
      expect(body).toContain(named);
    }
    const bindStart = ADDRESS_RULES.indexOf("pub fn check_bind");
    const bindEnd = ADDRESS_RULES.indexOf("#[must_use]", bindStart);
    const bindBody = ADDRESS_RULES.slice(bindStart, bindEnd);
    expect(bindBody).toContain('"http://"');
    expect(bindBody).toContain("!own_network(host)");
  });
});

describe("plain http reaches only the viewer's own network", () => {
  it("because it carries the session's tokens where anyone on the hop can read them", () => {
    expect(encryptionOf("https://streams.example")).toBe(false);
    expect(encryptionOf("http://streams.example")).toBeNull();
  });

  it("and the viewer's own machine is their own hop", () => {
    expect(encryptionOf("http://localhost:8080")).toBe(true);
    expect(encryptionOf("http://127.0.0.1")).toBe(true);
    expect(encryptionOf("http://nas.local")).toBe(true);
  });

  it("as is every private range, and nothing next to one", () => {
    expect(ownNetwork("10.0.0.5")).toBe(true);
    expect(ownNetwork("192.168.1.4")).toBe(true);
    expect(ownNetwork("169.254.3.3")).toBe(true);
    expect(ownNetwork("172.16.0.1")).toBe(true);
    expect(ownNetwork("172.31.255.255")).toBe(true);
    expect(ownNetwork("172.15.0.1")).toBe(false);
    expect(ownNetwork("172.32.0.1")).toBe(false);
    expect(ownNetwork("11.0.0.1")).toBe(false);
    expect(ownNetwork("192.169.1.4")).toBe(false);
  });

  it("refuses an address that is neither, which is every address a stranger could serve", () => {
    expect(encryptionOf("streams.example")).toBeNull();
    expect(encryptionOf("ftp://streams.example")).toBeNull();
    expect(encryptionOf("")).toBeNull();
  });

  it("reads the host out of an address with a port, a path or brackets round it", () => {
    expect(hostOf("http://10.0.0.5:8080/v1/hello")).toBe("10.0.0.5");
    expect(hostOf("http://[::1]:8080")).toBe("::1");
    expect(hostOf("http://localhost")).toBe("localhost");
  });

  it("is not fooled by a name that only looks like a number", () => {
    expect(ownNetwork("10.0.0")).toBe(false);
    expect(ownNetwork("10.0.0.x")).toBe(false);
    expect(ownNetwork("300.1.1.1")).toBe(false);
  });

  it("takes https whatever case it was typed in", () => {
    expect(encryptionOf("HTTPS://Example.Test")).toBe(false);
    expect(encryptionOf("HTTP://LOCALHOST")).toBe(true);
  });
});
