// The core's C functions, as C++ sees them. Five of `core.h`'s six are used: `core_abi`,
// `core_open`, `core_step`, `core_free` and `core_close`. `core_alloc` is not, and is not surfaced,
// because every buffer handed in belongs to a `std::string` this host already owns. Bytes go in and
// bytes come out, and no rules live here. This is the same boundary
// `apps/android/core/src/main/cpp/bridge.c` crosses for Java and `apps/apple/Sources/CoreBridge`
// crosses for Swift: one static library, one opaque handle.
#ifndef STREAMS_XBOX_CORE_HOST_H
#define STREAMS_XBOX_CORE_HOST_H

#include <cstdint>
#include <string>

#include "core.h"

namespace streams {

// The four statuses of `core_step`, named instead of numbered. They are core.h's values.
enum class Status : std::int32_t {
    Ok = CORE_OK,
    BadMessage = CORE_BAD_MESSAGE,
    Panicked = CORE_PANICKED,
    BadCall = CORE_BAD_CALL,
};

// What one step gives back. `out` holds a core_out message when the status is Ok, and nothing
// otherwise: a refused step changed nothing, so there is nothing to read.
struct Stepped {
    Status status = Status::BadCall;
    std::string out;
};

// One running core. A host opens it with a core config, steps it with a core_in message, performs
// what the message asks, and closes it. After a Panicked status the handle stays refused: close it
// and open a new one from what was stored.
class Core {
public:
    Core() = default;
    ~Core();
    Core(const Core &) = delete;
    Core &operator=(const Core &) = delete;
    Core(Core &&other) noexcept;
    Core &operator=(Core &&other) noexcept;

    // The ABI number the linked library carries. A host checks it against CORE_ABI before opening.
    static std::uint32_t abi();

    // True when the config read and a handle exists. A config that does not read gives no handle,
    // which is the one failure a host must report to a viewer rather than retry.
    bool open(const std::string &config);
    bool isOpen() const { return handle_ != nullptr; }

    Stepped step(const std::string &input);
    void close();

private:
    ::CoreHandle *handle_ = nullptr;
};

} // namespace streams

#endif
