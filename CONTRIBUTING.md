# Contributing

1. Read `platform/contract/README.md`. The contract is the one written promise between the parts.
2. Run the checks before you open a change: `./check`. Its first comment says what it does not
   cover, which is mostly the device app builds: build those yourself when you touch them.
3. Keep a rule in one place. If two parts need the same rule, it belongs in the contract, the core or
   the server, never in two copies.
4. Screens hold no rules. The server decides what appears and in what order. The device decides
   pixels, motion and focus.
5. A test must be able to fail. If the work were undone, the test should go red.

Contributions are under the repository's licences, MIT or Apache 2.0.
