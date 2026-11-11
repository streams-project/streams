#include "core_host.h"

#include <cstddef>
#include <utility>

namespace streams {

Core::~Core() { close(); }

Core::Core(Core &&other) noexcept : handle_(other.handle_) { other.handle_ = nullptr; }

Core &Core::operator=(Core &&other) noexcept {
    if (this != &other) {
        close();
        handle_ = other.handle_;
        other.handle_ = nullptr;
    }
    return *this;
}

std::uint32_t Core::abi() { return core_abi(); }

bool Core::open(const std::string &config) {
    close();
    handle_ = core_open(reinterpret_cast<const std::uint8_t *>(config.data()), config.size());
    return handle_ != nullptr;
}

Stepped Core::step(const std::string &input) {
    Stepped answer;
    if (handle_ == nullptr) {
        answer.status = Status::BadCall;
        return answer;
    }
    std::uint8_t *out = nullptr;
    std::size_t len = 0;
    const std::int32_t status = core_step(
        handle_, reinterpret_cast<const std::uint8_t *>(input.data()), input.size(), &out, &len);
    answer.status = static_cast<Status>(status);
    if (out != nullptr) {
        // The bytes belong to the core's allocator, so they are copied once and given straight
        // back. Nothing above this line ever holds a pointer into the core.
        if (len > 0) {
            answer.out.assign(reinterpret_cast<const char *>(out), len);
        }
        core_free(out, len);
    }
    return answer;
}

void Core::close() {
    if (handle_ != nullptr) {
        core_close(handle_);
        handle_ = nullptr;
    }
}

} // namespace streams
