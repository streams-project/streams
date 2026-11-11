// Does the skeleton really drive the core? Open, step twice, refuse a broken message, close.
//
// It is not an Xbox test: this folder has no harness, no console and no Windows toolchain, so this
// runs on whatever machine builds the C bridge and proves one thing only - that `core_host.cpp`
// crosses the boundary correctly. The Xbox-shaped part of it is the config: class `box`, form `tv`.
//
//     cargo build --locked --profile native -p bridge-c --target <this machine's target>
//     c++ -std=c++17 -I platform/bridges/c apps/xbox/host/core_host.cpp \
//         apps/xbox/host/smoke_test.cpp target/<target>/native/libbridge_c.a -o smoke && ./smoke
#include <cstdio>
#include <string>

#include "core_host.h"

namespace {

// The contract version the core in this tree speaks. A mismatch here fails the first step, which
// is the point: the host carries one version and the core refuses anything else.
const char *const WIRE = "2026-11-05";

// A console under a television. `class` decides the floors the server hands this device and `form`
// decides what it draws, so the pair is the one thing a host must get right before any screen
// exists. Which class an Xbox reports is still open - see this folder's README.
const std::string CONFIG = std::string(R"({"app":"xbox","class":"box","contract":")") + WIRE +
                           R"(","form":"tv","mode":"local","os":"xbox",)"
                           R"("seed":"00112233445566778899aabbccddeeff",)"
                           R"("type":"core_config","version":"0.0.0"})";

std::string tick(long long ms) {
    return std::string(R"({"contract":")") + WIRE + R"(","events":[],"mono_ms":)" +
           std::to_string(ms) + R"(,"now_ms":)" + std::to_string(1000 + ms) +
           R"(,"type":"core_in"})";
}

int checks = 0;
int failures = 0;

void ok(bool held, const char *what) {
    checks += 1;
    std::printf("%s %s\n", held ? "ok  " : "FAIL", what);
    if (!held) {
        failures += 1;
    }
}

} // namespace

int main() {
    ok(streams::Core::abi() == CORE_ABI, "the linked core carries the ABI this host was built for");

    streams::Core core;
    ok(!core.isOpen(), "a core that was never opened holds no handle");
    ok(core.step(tick(0)).status == streams::Status::BadCall, "a step before open is refused");
    ok(!core.open("not json"), "a config that does not read gives no handle");

    ok(core.open(CONFIG), "a console config opens the core");

    const streams::Stepped first = core.step(tick(0));
    ok(first.status == streams::Status::Ok, "the first step runs");
    const std::string head = R"({"contract":")" + std::string(WIRE) + R"(","type":"core_out")";
    ok(first.out.rfind(head, 0) == 0, "the first step gives a core_out message");

    const streams::Stepped broken = core.step("{");
    ok(broken.status == streams::Status::BadMessage, "a message that does not read is refused");
    ok(broken.out.empty(), "a refused step hands back nothing");

    const streams::Stepped second = core.step(tick(5));
    ok(second.status == streams::Status::Ok, "the core carries on after a refused message");

    core.close();
    ok(!core.isOpen(), "a closed core holds no handle");

    // Counted rather than written out, so adding a check cannot leave a wrong total behind.
    if (failures > 0) {
        std::printf("%d of %d failed\n", failures, checks);
        return 1;
    }
    std::printf("%d of %d held\n", checks, checks);
    return 0;
}
