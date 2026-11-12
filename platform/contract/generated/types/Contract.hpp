// Generated from the contract. Do not edit.

#ifndef STREAMS_CONTRACT_HPP
#define STREAMS_CONTRACT_HPP

#include <cstddef>
#include <cstdint>
#include <map>
#include <optional>
#include <regex>
#include <stdexcept>
#include <string>
#include <string_view>
#include <utility>
#include <variant>
#include <vector>

/// Every type the contract publishes, as plain aggregates a screen reads, each
/// with the reader and the writer the wire needs. Construction stays in :core
/// and :ports; a surface reads what the core returns.
namespace contract {

inline constexpr std::string_view wire_version = "2026-11-05";
inline constexpr int abi_version = 1;

/// Every address the apps open.
struct Address {
    static constexpr std::string_view start = "/";
    static constexpr std::string_view home = "/browse";
    static constexpr std::string_view kids = "/kids";
    static constexpr std::string_view series = "/browse/genre/83";
    static constexpr std::string_view films = "/browse/genre/34399";
    static constexpr std::string_view new_ = "/latest";
    static constexpr std::string_view games = "/games";
    static constexpr std::string_view my_list = "/browse/my-list";
    static constexpr std::string_view mine = "/mine";
    static constexpr std::string_view language = "/browse/original-audio";
    static constexpr std::string_view search = "/search";
    static constexpr std::string_view notifications = "/notifications";
    static constexpr std::string_view activity = "/settings/viewed";
    static constexpr std::string_view clips = "/clips";
    static constexpr std::string_view help = "/help";
    static constexpr std::string_view settings = "/settings";
    static constexpr std::string_view profiles = "/profiles";
    static constexpr std::string_view sign_in = "/login";
    static constexpr std::string_view activate = "/activate";
    static constexpr std::string_view genre = "/browse/genre/";
    static constexpr std::string_view title = "/title/";
    static constexpr std::string_view watch = "/watch/";

    /// Where a tab lives, by the screen's word.
    static std::string_view of(std::string_view screen) {
        if (screen == "home") {
            return "/browse";
        }
        if (screen == "kids") {
            return "/kids";
        }
        if (screen == "series") {
            return "/browse/genre/83";
        }
        if (screen == "films") {
            return "/browse/genre/34399";
        }
        if (screen == "new") {
            return "/latest";
        }
        if (screen == "games") {
            return "/games";
        }
        if (screen == "my-list") {
            return "/browse/my-list";
        }
        if (screen == "mine") {
            return "/mine";
        }
        if (screen == "language") {
            return "/browse/original-audio";
        }
        if (screen == "search") {
            return "/search";
        }
        if (screen == "notifications") {
            return "/notifications";
        }
        if (screen == "activity") {
            return "/settings/viewed";
        }
        if (screen == "clips") {
            return "/clips";
        }
        if (screen == "help") {
            return "/help";
        }
        if (screen == "settings") {
            return "/settings";
        }
        return home;
    }
};

/// A value the contract refuses, named by what refused it. Every generated `read` throws this and
/// nothing else, so one `catch` covers the contract.
class Refused : public std::runtime_error {
public:
    explicit Refused(const std::string& why) : std::runtime_error(why) {}
};

/// A JSON reader and writer that depends on nothing but the standard library, because C++ has no
/// JSON of its own and a generated header must not add a dependency on one.
struct Json {
    enum class Kind { Null, Bool, Whole, Real, Text, Array, Object };

    /// One JSON value.
    ///
    /// An object holds its keys and its values in two vectors side by side, and not in a
    /// `std::map<std::string, Value>`, because inside its own definition `Value` is incomplete and
    /// `std::vector` is the one standard container allowed to hold an incomplete type. The order
    /// written is the order read, and a repeated key keeps the value written last.
    struct Value {
        Kind kind = Kind::Null;
        bool boolean = false;
        std::int64_t whole = 0;
        double real = 0;
        std::string text;
        std::vector<std::string> keys;
        std::vector<Value> items;
    };

    /// A document as a tree of values.
    static Value parse(const std::string& text) {
        std::size_t index = 0;
        space(text, index);
        Value found = read_value(text, index);
        space(text, index);
        if (index != text.size()) {
            throw Refused("trailing text at " + std::to_string(index));
        }
        return found;
    }

    /// What a value writes itself as.
    template <class T>
    static std::string render(const T& held) {
        std::string out;
        held.write(out);
        return out;
    }

    /// Whether two documents say the same thing. Object fields are compared by key and not by the
    /// order they were written in, which is what `Map.equals` gives the Java arm of the same check.
    static bool same(const Value& left, const Value& right) {
        if (left.kind != right.kind) {
            return false;
        }
        switch (left.kind) {
            case Kind::Null:
                return true;
            case Kind::Bool:
                return left.boolean == right.boolean;
            case Kind::Whole:
                return left.whole == right.whole;
            case Kind::Real:
                return left.real >= right.real && left.real <= right.real;
            case Kind::Text:
                return left.text == right.text;
            case Kind::Array:
                if (left.items.size() != right.items.size()) {
                    return false;
                }
                for (std::size_t index = 0; index < left.items.size(); index++) {
                    if (!same(left.items[index], right.items[index])) {
                        return false;
                    }
                }
                return true;
            case Kind::Object:
                if (left.keys.size() != right.keys.size()) {
                    return false;
                }
                for (std::size_t index = 0; index < left.keys.size(); index++) {
                    const Value* found = held(right, left.keys[index]);
                    if (found == nullptr || !same(left.items[index], *found)) {
                        return false;
                    }
                }
                return true;
        }
        return false;
    }

    /// The object a message is read from.
    static const Value& object(const Value& value) {
        if (value.kind != Kind::Object) {
            throw Refused("not an object");
        }
        return value;
    }

    /// The entry under a key, whether it was written as null or not.
    static const Value* held(const Value& fields, std::string_view key) {
        for (std::size_t index = 0; index < fields.keys.size(); index++) {
            if (fields.keys[index] == key) {
                return &fields.items[index];
            }
        }
        return nullptr;
    }

    /// A field that may be absent. A written null counts as absent, as it does in Kotlin.
    static const Value* at(const Value& fields, std::string_view key) {
        const Value* found = held(fields, key);
        if (found == nullptr || found->kind == Kind::Null) {
            return nullptr;
        }
        return found;
    }

    /// A field that must be there. Never null, so the generated reader dereferences it.
    static const Value* need(const Value& fields, std::string_view key, std::string_view type) {
        const Value* found = at(fields, key);
        if (found == nullptr) {
            throw Refused(std::string(type) + " has no " + std::string(key));
        }
        return found;
    }

    static const std::string& as_text(const Value& value) {
        if (value.kind != Kind::Text) {
            throw Refused("not text");
        }
        return value.text;
    }

    static std::int64_t as_long(const Value& value) {
        if (value.kind != Kind::Whole) {
            throw Refused("not a whole number");
        }
        return value.whole;
    }

    static bool as_bool(const Value& value) {
        if (value.kind != Kind::Bool) {
            throw Refused("not a boolean");
        }
        return value.boolean;
    }

    template <class T, class Each>
    static std::vector<T> list(const Value& value, Each each) {
        if (value.kind != Kind::Array) {
            throw Refused("not a list");
        }
        std::vector<T> found;
        found.reserve(value.items.size());
        for (const Value& item : value.items) {
            found.push_back(each(item));
        }
        return found;
    }

    template <class T, class Each>
    static std::map<std::string, T> map(const Value& value, Each each) {
        const Value& fields = object(value);
        std::map<std::string, T> found;
        for (std::size_t index = 0; index < fields.keys.size(); index++) {
            found.emplace(fields.keys[index], each(fields.items[index]));
        }
        return found;
    }

    /// Opens a field, with the comma the field before it needs.
    static int key(std::string& out, int written, std::string_view name) {
        if (written > 0) {
            out += ',';
        }
        text(out, name);
        out += ':';
        return written + 1;
    }

    /// A JSON string literal. Bytes at or above 0x80 pass through, because a `std::string` here
    /// holds UTF-8 and JSON carries UTF-8.
    static void text(std::string& out, std::string_view value) {
        out += '"';
        for (const char here : value) {
            const unsigned char code = static_cast<unsigned char>(here);
            if (here == '"' || here == '\\') {
                out += '\\';
                out += here;
            } else if (here == '\n') {
                out += "\\n";
            } else if (here == '\r') {
                out += "\\r";
            } else if (here == '\t') {
                out += "\\t";
            } else if (code < 0x20) {
                const char* digits = "0123456789abcdef";
                out += "\\u00";
                out += digits[code >> 4];
                out += digits[code & 0x0F];
            } else {
                out += here;
            }
        }
        out += '"';
    }

private:
    static void space(const std::string& text, std::size_t& index) {
        while (index < text.size()) {
            const char here = text[index];
            if (here != ' ' && here != '\t' && here != '\n' && here != '\r') {
                return;
            }
            index++;
        }
    }

    static Value read_value(const std::string& text, std::size_t& index) {
        if (index >= text.size()) {
            throw Refused("no value at " + std::to_string(index));
        }
        Value found;
        const char head = text[index];
        if (head == '{') {
            return read_object(text, index);
        }
        if (head == '[') {
            return read_array(text, index);
        }
        if (head == '"') {
            found.kind = Kind::Text;
            found.text = read_text(text, index);
            return found;
        }
        if (head == 't' || head == 'f') {
            word(text, index, head == 't' ? "true" : "false");
            found.kind = Kind::Bool;
            found.boolean = head == 't';
            return found;
        }
        if (head == 'n') {
            word(text, index, "null");
            return found;
        }
        return read_number(text, index);
    }

    static void word(const std::string& text, std::size_t& index, std::string_view wanted) {
        if (text.compare(index, wanted.size(), wanted) != 0) {
            throw Refused("not " + std::string(wanted) + " at " + std::to_string(index));
        }
        index += wanted.size();
    }

    static Value read_object(const std::string& text, std::size_t& index) {
        Value found;
        found.kind = Kind::Object;
        index++;
        space(text, index);
        if (index < text.size() && text[index] == '}') {
            index++;
            return found;
        }
        while (true) {
            space(text, index);
            std::string name = read_text(text, index);
            space(text, index);
            if (index >= text.size() || text[index] != ':') {
                throw Refused("no colon at " + std::to_string(index));
            }
            index++;
            space(text, index);
            Value value = read_value(text, index);
            const Value* already = held(found, name);
            if (already == nullptr) {
                found.keys.push_back(std::move(name));
                found.items.push_back(std::move(value));
            } else {
                found.items[static_cast<std::size_t>(already - found.items.data())] =
                        std::move(value);
            }
            space(text, index);
            if (index >= text.size()) {
                throw Refused("unclosed object");
            }
            const char next = text[index++];
            if (next == '}') {
                return found;
            }
            if (next != ',') {
                throw Refused("no comma at " + std::to_string(index - 1));
            }
        }
    }

    static Value read_array(const std::string& text, std::size_t& index) {
        Value found;
        found.kind = Kind::Array;
        index++;
        space(text, index);
        if (index < text.size() && text[index] == ']') {
            index++;
            return found;
        }
        while (true) {
            space(text, index);
            found.items.push_back(read_value(text, index));
            space(text, index);
            if (index >= text.size()) {
                throw Refused("unclosed list");
            }
            const char next = text[index++];
            if (next == ']') {
                return found;
            }
            if (next != ',') {
                throw Refused("no comma at " + std::to_string(index - 1));
            }
        }
    }

    static std::string read_text(const std::string& text, std::size_t& index) {
        if (index >= text.size() || text[index] != '"') {
            throw Refused("no text at " + std::to_string(index));
        }
        index++;
        std::string out;
        while (true) {
            if (index >= text.size()) {
                throw Refused("unclosed text");
            }
            const char here = text[index++];
            if (here == '"') {
                return out;
            }
            if (here != '\\') {
                out += here;
                continue;
            }
            if (index >= text.size()) {
                throw Refused("unclosed escape");
            }
            const char mark = text[index++];
            if (mark == 'u') {
                append_point(out, text, index);
            } else if (mark == 'n') {
                out += '\n';
            } else if (mark == 't') {
                out += '\t';
            } else if (mark == 'r') {
                out += '\r';
            } else if (mark == 'b') {
                out += '\b';
            } else if (mark == 'f') {
                out += '\f';
            } else if (mark == '"' || mark == '\\' || mark == '/') {
                out += mark;
            } else {
                throw Refused("bad escape at " + std::to_string(index - 1));
            }
        }
    }

    /// Four hex digits as UTF-8, joining a surrogate pair when one follows. A `std::string` holds
    /// bytes, not UTF-16 units, so Java's `(char) point` has no counterpart here.
    static void append_point(std::string& out, const std::string& text, std::size_t& index) {
        unsigned long point = hex(text, index);
        if (point >= 0xD800 && point <= 0xDBFF) {
            if (text.compare(index, 2, "\\u") != 0) {
                throw Refused("a lone high surrogate at " + std::to_string(index));
            }
            index += 2;
            const unsigned long low = hex(text, index);
            if (low < 0xDC00 || low > 0xDFFF) {
                throw Refused("not a low surrogate at " + std::to_string(index));
            }
            point = 0x10000 + ((point - 0xD800) << 10) + (low - 0xDC00);
        } else if (point >= 0xDC00 && point <= 0xDFFF) {
            throw Refused("a lone low surrogate at " + std::to_string(index));
        }
        if (point < 0x80) {
            out += static_cast<char>(point);
            return;
        }
        if (point < 0x800) {
            out += static_cast<char>(0xC0 | (point >> 6));
        } else if (point < 0x10000) {
            out += static_cast<char>(0xE0 | (point >> 12));
            out += static_cast<char>(0x80 | ((point >> 6) & 0x3F));
        } else {
            out += static_cast<char>(0xF0 | (point >> 18));
            out += static_cast<char>(0x80 | ((point >> 12) & 0x3F));
            out += static_cast<char>(0x80 | ((point >> 6) & 0x3F));
        }
        out += static_cast<char>(0x80 | (point & 0x3F));
    }

    static unsigned long hex(const std::string& text, std::size_t& index) {
        if (index + 4 > text.size()) {
            throw Refused("a short escape at " + std::to_string(index));
        }
        unsigned long found = 0;
        for (std::size_t step = 0; step < 4; step++) {
            const char here = text[index + step];
            const std::size_t digit = std::string_view("0123456789abcdef").find(
                    static_cast<char>(here >= 'A' && here <= 'F' ? here + ('a' - 'A') : here));
            if (digit == std::string_view::npos) {
                throw Refused("not a hex digit at " + std::to_string(index + step));
            }
            found = (found << 4) + digit;
        }
        index += 4;
        return found;
    }

    static Value read_number(const std::string& text, std::size_t& index) {
        const std::size_t from = index;
        while (index < text.size()
                && std::string_view("+-.eE0123456789").find(text[index])
                        != std::string_view::npos) {
            index++;
        }
        const std::string found = text.substr(from, index - from);
        if (found.empty()) {
            throw Refused("no value at " + std::to_string(from));
        }
        Value out;
        const bool whole = found.find('.') == std::string::npos
                && found.find('e') == std::string::npos
                && found.find('E') == std::string::npos;
        try {
            if (whole) {
                out.kind = Kind::Whole;
                out.whole = static_cast<std::int64_t>(std::stoll(found));
            } else {
                out.kind = Kind::Real;
                out.real = std::stod(found);
            }
        } catch (const std::exception&) {
            throw Refused("not a number: " + found);
        }
        return out;
    }
};

struct Account {
    std::string email;
    std::string first_name;
    std::string id;
    std::string last_name;
    std::optional<std::string> phone;

    static Account read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "email", "Account");
        const Json::Value* raw1 = Json::need(fields, "first_name", "Account");
        const Json::Value* raw2 = Json::need(fields, "id", "Account");
        const Json::Value* raw3 = Json::need(fields, "last_name", "Account");
        const Json::Value* raw4 = Json::at(fields, "phone");
        return Account{
                Json::as_text(*raw0),
                Json::as_text(*raw1),
                Json::as_text(*raw2),
                Json::as_text(*raw3),
                raw4 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw4))};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "email");
        Json::text(out, this->email);
        count = Json::key(out, count, "first_name");
        Json::text(out, this->first_name);
        count = Json::key(out, count, "id");
        Json::text(out, this->id);
        count = Json::key(out, count, "last_name");
        Json::text(out, this->last_name);
        if (this->phone.has_value()) {
            count = Json::key(out, count, "phone");
            Json::text(out, (*this->phone));
        }
        out += '}';
    }
};

/// Client class. Minimum versions, start deadlines and reachability are kept per class.
struct DeviceClass {
    std::string value;

    static constexpr std::string_view web = "web";
    static constexpr std::string_view phone = "phone";
    static constexpr std::string_view tablet = "tablet";
    static constexpr std::string_view desktop = "desktop";
    static constexpr std::string_view tv = "tv";
    static constexpr std::string_view box = "box";
    static constexpr std::string_view car = "car";

    static DeviceClass read(const Json::Value& document) {
        return DeviceClass{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// One signed in device.
struct DeviceSession {
    DeviceClass class_;
    bool current = false;
    std::string id;
    std::int64_t last_seen_ms = 0;
    std::string name;

    static DeviceSession read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "class", "DeviceSession");
        const Json::Value* raw1 = Json::need(fields, "current", "DeviceSession");
        const Json::Value* raw2 = Json::need(fields, "id", "DeviceSession");
        const Json::Value* raw3 = Json::need(fields, "last_seen_ms", "DeviceSession");
        const Json::Value* raw4 = Json::need(fields, "name", "DeviceSession");
        return DeviceSession{
                DeviceClass::read(*raw0),
                Json::as_bool(*raw1),
                Json::as_text(*raw2),
                Json::as_long(*raw3),
                Json::as_text(*raw4)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "class");
        this->class_.write(out);
        count = Json::key(out, count, "current");
        out += this->current ? "true" : "false";
        count = Json::key(out, count, "id");
        Json::text(out, this->id);
        count = Json::key(out, count, "last_seen_ms");
        out += std::to_string(this->last_seen_ms);
        count = Json::key(out, count, "name");
        Json::text(out, this->name);
        out += '}';
    }
};

/// One profile.
struct Profile {
    std::optional<std::string> avatar;
    bool has_pin = false;
    std::string id;
    bool kids = false;
    /// A language code from `/v1/languages`, absent until one is chosen.
    std::optional<std::string> language;
    bool main = false;
    std::string name;

    static Profile read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "avatar");
        const Json::Value* raw1 = Json::need(fields, "has_pin", "Profile");
        const Json::Value* raw2 = Json::need(fields, "id", "Profile");
        const Json::Value* raw3 = Json::need(fields, "kids", "Profile");
        const Json::Value* raw4 = Json::at(fields, "language");
        const Json::Value* raw5 = Json::need(fields, "main", "Profile");
        const Json::Value* raw6 = Json::need(fields, "name", "Profile");
        return Profile{
                raw0 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw0)),
                Json::as_bool(*raw1),
                Json::as_text(*raw2),
                Json::as_bool(*raw3),
                raw4 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw4)),
                Json::as_bool(*raw5),
                Json::as_text(*raw6)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->avatar.has_value()) {
            count = Json::key(out, count, "avatar");
            Json::text(out, (*this->avatar));
        }
        count = Json::key(out, count, "has_pin");
        out += this->has_pin ? "true" : "false";
        count = Json::key(out, count, "id");
        Json::text(out, this->id);
        count = Json::key(out, count, "kids");
        out += this->kids ? "true" : "false";
        if (this->language.has_value()) {
            count = Json::key(out, count, "language");
            Json::text(out, (*this->language));
        }
        count = Json::key(out, count, "main");
        out += this->main ? "true" : "false";
        count = Json::key(out, count, "name");
        Json::text(out, this->name);
        out += '}';
    }
};

/// Everything held about an account, in plain JSON.
struct AccountExport {
    Account account;
    std::vector<DeviceSession> devices;
    std::vector<Profile> profiles;

    static AccountExport read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "account", "AccountExport");
        const Json::Value* raw1 = Json::need(fields, "devices", "AccountExport");
        const Json::Value* raw2 = Json::need(fields, "profiles", "AccountExport");
        return AccountExport{
                Account::read(*raw0),
                Json::list<DeviceSession>(*raw1, [](const Json::Value& x0) { return DeviceSession::read(x0); }),
                Json::list<Profile>(*raw2, [](const Json::Value& x0) { return Profile::read(x0); })};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "account");
        this->account.write(out);
        count = Json::key(out, count, "devices");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->devices.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->devices[index0].write(out);
        }
        out += ']';
        count = Json::key(out, count, "profiles");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->profiles.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->profiles[index0].write(out);
        }
        out += ']';
        out += '}';
    }
};

struct AccountUpdate {
    std::optional<std::string> first_name;
    std::optional<std::string> last_name;

    static AccountUpdate read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "first_name");
        const Json::Value* raw1 = Json::at(fields, "last_name");
        return AccountUpdate{
                raw0 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw0)),
                raw1 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw1))};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->first_name.has_value()) {
            count = Json::key(out, count, "first_name");
            Json::text(out, (*this->first_name));
        }
        if (this->last_name.has_value()) {
            count = Json::key(out, count, "last_name");
            Json::text(out, (*this->last_name));
        }
        out += '}';
    }
};

/// One episode of a series. A film has none.
struct EpisodeRef {
    std::int64_t episode = 0;
    std::int64_t season = 0;

    static EpisodeRef read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "episode", "EpisodeRef");
        const Json::Value* raw1 = Json::need(fields, "season", "EpisodeRef");
        return EpisodeRef{
                Json::as_long(*raw0),
                Json::as_long(*raw1)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "episode");
        out += std::to_string(this->episode);
        count = Json::key(out, count, "season");
        out += std::to_string(this->season);
        out += '}';
    }
};

/// Signals a viewer gives.
struct FeedbackEvent {
    std::string value;

    static constexpr std::string_view click = "click";
    static constexpr std::string_view start = "start";
    static constexpr std::string_view complete = "complete";
    static constexpr std::string_view like = "like";
    static constexpr std::string_view skip = "skip";

    static FeedbackEvent read(const Json::Value& document) {
        return FeedbackEvent{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// Rating words on the wire. Screens use the words like, love and dislike.
struct Rating {
    std::string value;

    static constexpr std::string_view down = "down";
    static constexpr std::string_view up = "up";
    static constexpr std::string_view love = "love";

    static Rating read(const Json::Value& document) {
        return Rating{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// Settings a queued command may change. Age rating and blocked titles are absent on purpose: they need a step up, so a queued write can never touch them.
struct SettingKey {
    std::string value;

    static constexpr std::string_view autoplay_next = "autoplay_next";
    static constexpr std::string_view autoplay_previews = "autoplay_previews";
    static constexpr std::string_view audio_language = "audio_language";
    static constexpr std::string_view subtitle_language = "subtitle_language";
    static constexpr std::string_view subtitle_style = "subtitle_style";
    static constexpr std::string_view ui_language = "ui_language";
    static constexpr std::string_view handle = "handle";
    static constexpr std::string_view gender = "gender";

    static SettingKey read(const Json::Value& document) {
        return SettingKey{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// movie:<id> or tv:<id>. A bare id does not parse.
struct TitleKey {
    std::string value;

    explicit TitleKey(std::string text) : value(std::move(text)) {
        if (!std::regex_match(this->value, shape())) {
            throw Refused("not a TitleKey");
        }
    }

    static TitleKey read(const Json::Value& document) {
        return TitleKey(Json::as_text(document));
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }

private:
    static const std::regex& shape() {
        static const std::regex compiled("^(movie|tv):(?:[1-9][0-9]{0,8}|1[0-9]{9}|20[0-9]{8}|21[0-3][0-9]{7}|214[0-6][0-9]{6}|2147[0-3][0-9]{5}|21474[0-7][0-9]{4}|214748[0-2][0-9]{3}|2147483[0-5][0-9]{2}|21474836[0-3][0-9]|214748364[0-7])$", std::regex::ECMAScript);
        return compiled;
    }
};

/// Add to or remove from the saved list. Leaves the rating alone.
struct CommandList {
    bool in_list = false;
    TitleKey title;

    /// The word this variant carries in `command`.
    std::string_view word() const {
        return "list";
    }

    static CommandList read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "in_list", "Command");
        const Json::Value* raw1 = Json::need(fields, "title", "Command");
        return CommandList{
                Json::as_bool(*raw0),
                TitleKey::read(*raw1)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "in_list");
        out += this->in_list ? "true" : "false";
        count = Json::key(out, count, "title");
        this->title.write(out);
    }
};

/// Set or clear a rating. Leaves the list entry alone.
struct CommandRate {
    std::optional<Rating> rating;
    TitleKey title;

    /// The word this variant carries in `command`.
    std::string_view word() const {
        return "rate";
    }

    static CommandRate read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::at(fields, "rating");
        const Json::Value* raw1 = Json::need(fields, "title", "Command");
        return CommandRate{
                raw0 == nullptr ? std::optional<Rating>() : std::optional<Rating>(Rating::read(*raw0)),
                TitleKey::read(*raw1)};
    }

    void write_fields(std::string& out, int& count) const {
        if (this->rating.has_value()) {
            count = Json::key(out, count, "rating");
            (*this->rating).write(out);
        }
        count = Json::key(out, count, "title");
        this->title.write(out);
    }
};

/// Hide a title from history. For a series this covers every episode.
struct CommandHide {
    TitleKey title;

    /// The word this variant carries in `command`.
    std::string_view word() const {
        return "hide";
    }

    static CommandHide read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "title", "Command");
        return CommandHide{
                TitleKey::read(*raw0)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "title");
        this->title.write(out);
    }
};

/// Clear all history of this profile, as one operation.
struct CommandClearHistory {
    /// The word this variant carries in `command`.
    std::string_view word() const {
        return "clear_history";
    }

    static CommandClearHistory read_fields(const Json::Value&) {
        return CommandClearHistory{};
    }

    void write_fields(std::string&, int&) const {
    }
};

/// Save a position, in whole seconds.
struct CommandPosition {
    std::int64_t duration_s = 0;
    std::optional<EpisodeRef> episode;
    std::int64_t position_s = 0;
    TitleKey title;

    /// The word this variant carries in `command`.
    std::string_view word() const {
        return "position";
    }

    static CommandPosition read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "duration_s", "Command");
        const Json::Value* raw1 = Json::at(fields, "episode");
        const Json::Value* raw2 = Json::need(fields, "position_s", "Command");
        const Json::Value* raw3 = Json::need(fields, "title", "Command");
        return CommandPosition{
                Json::as_long(*raw0),
                raw1 == nullptr ? std::optional<EpisodeRef>() : std::optional<EpisodeRef>(EpisodeRef::read(*raw1)),
                Json::as_long(*raw2),
                TitleKey::read(*raw3)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "duration_s");
        out += std::to_string(this->duration_s);
        if (this->episode.has_value()) {
            count = Json::key(out, count, "episode");
            (*this->episode).write(out);
        }
        count = Json::key(out, count, "position_s");
        out += std::to_string(this->position_s);
        count = Json::key(out, count, "title");
        this->title.write(out);
    }
};

/// Change one profile setting. Booleans as true or false, style as JSON text.
struct CommandSetting {
    SettingKey key;
    std::string value;

    /// The word this variant carries in `command`.
    std::string_view word() const {
        return "setting";
    }

    static CommandSetting read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "key", "Command");
        const Json::Value* raw1 = Json::need(fields, "value", "Command");
        return CommandSetting{
                SettingKey::read(*raw0),
                Json::as_text(*raw1)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "key");
        this->key.write(out);
        count = Json::key(out, count, "value");
        Json::text(out, this->value);
    }
};

/// A viewer signal. Discarded by the server when consent is off.
struct CommandFeedback {
    FeedbackEvent event;
    TitleKey title;

    /// The word this variant carries in `command`.
    std::string_view word() const {
        return "feedback";
    }

    static CommandFeedback read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "event", "Command");
        const Json::Value* raw1 = Json::need(fields, "title", "Command");
        return CommandFeedback{
                FeedbackEvent::read(*raw0),
                TitleKey::read(*raw1)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "event");
        this->event.write(out);
        count = Json::key(out, count, "title");
        this->title.write(out);
    }
};

/// A trailer failed to play for this title.
struct CommandTrailerRefused {
    TitleKey title;

    /// The word this variant carries in `command`.
    std::string_view word() const {
        return "trailer_refused";
    }

    static CommandTrailerRefused read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "title", "Command");
        return CommandTrailerRefused{
                TitleKey::read(*raw0)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "title");
        this->title.write(out);
    }
};

/// Set or clear a reminder on a coming title.
struct CommandRemind {
    bool on = false;
    TitleKey title;

    /// The word this variant carries in `command`.
    std::string_view word() const {
        return "remind";
    }

    static CommandRemind read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "on", "Command");
        const Json::Value* raw1 = Json::need(fields, "title", "Command");
        return CommandRemind{
                Json::as_bool(*raw0),
                TitleKey::read(*raw1)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "on");
        out += this->on ? "true" : "false";
        count = Json::key(out, count, "title");
        this->title.write(out);
    }
};

/// A tag this build does not know.
struct CommandUnknown {
    /// The tag that was read.
    std::string tag;

    std::string_view word() const {
        return this->tag;
    }

    void write_fields(std::string&, int&) const {
    }
};

using CommandBody = std::variant<CommandList, CommandRate, CommandHide, CommandClearHistory, CommandPosition, CommandSetting, CommandFeedback, CommandTrailerRefused, CommandRemind, CommandUnknown>;

/// One change to a profile's facts.
struct Command {
    CommandBody body;

    static Command read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        return Command{
                read_body(fields)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "command");
        Json::text(out, std::visit([](const auto& held) -> std::string_view { return held.word(); }, this->body));
        std::visit([&out, &count](const auto& held) { held.write_fields(out, count); }, this->body);
        out += '}';
    }

private:
    static CommandBody read_body(const Json::Value& fields) {
        const std::string& word = Json::as_text(*Json::need(fields, "command", "Command"));
        if (word == "list") {
            return CommandList::read_fields(fields);
        }
        if (word == "rate") {
            return CommandRate::read_fields(fields);
        }
        if (word == "hide") {
            return CommandHide::read_fields(fields);
        }
        if (word == "clear_history") {
            return CommandClearHistory::read_fields(fields);
        }
        if (word == "position") {
            return CommandPosition::read_fields(fields);
        }
        if (word == "setting") {
            return CommandSetting::read_fields(fields);
        }
        if (word == "feedback") {
            return CommandFeedback::read_fields(fields);
        }
        if (word == "trailer_refused") {
            return CommandTrailerRefused::read_fields(fields);
        }
        if (word == "remind") {
            return CommandRemind::read_fields(fields);
        }
        return CommandUnknown{word};
    }
};

/// Methods of an http effect.
struct HttpMethod {
    std::string value;

    static constexpr std::string_view get = "GET";
    static constexpr std::string_view post = "POST";
    static constexpr std::string_view put = "PUT";
    static constexpr std::string_view patch = "PATCH";
    static constexpr std::string_view delete_ = "DELETE";

    static HttpMethod read(const Json::Value& document) {
        return HttpMethod{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

struct ActionSignIn {
    std::string email;
    std::string password;

    /// The word this variant carries in `action`.
    std::string_view word() const {
        return "sign_in";
    }

    static ActionSignIn read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "email", "Action");
        const Json::Value* raw1 = Json::need(fields, "password", "Action");
        return ActionSignIn{
                Json::as_text(*raw0),
                Json::as_text(*raw1)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "email");
        Json::text(out, this->email);
        count = Json::key(out, count, "password");
        Json::text(out, this->password);
    }
};

struct ActionSendCode {
    std::string email;

    /// The word this variant carries in `action`.
    std::string_view word() const {
        return "send_code";
    }

    static ActionSendCode read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "email", "Action");
        return ActionSendCode{
                Json::as_text(*raw0)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "email");
        Json::text(out, this->email);
    }
};

struct ActionSignInCode {
    std::string code;
    std::string email;

    /// The word this variant carries in `action`.
    std::string_view word() const {
        return "sign_in_code";
    }

    static ActionSignInCode read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "code", "Action");
        const Json::Value* raw1 = Json::need(fields, "email", "Action");
        return ActionSignInCode{
                Json::as_text(*raw0),
                Json::as_text(*raw1)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "code");
        Json::text(out, this->code);
        count = Json::key(out, count, "email");
        Json::text(out, this->email);
    }
};

/// Start device sign in. The core asks for a code, polls and renews it, and the screen draws `View.device_sign_in`. While a code is live this changes nothing.
struct ActionDeviceSignIn {
    std::string device_name;

    /// The word this variant carries in `action`.
    std::string_view word() const {
        return "device_sign_in";
    }

    static ActionDeviceSignIn read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "device_name", "Action");
        return ActionDeviceSignIn{
                Json::as_text(*raw0)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "device_name");
        Json::text(out, this->device_name);
    }
};

struct ActionDeviceSignInStop {
    /// The word this variant carries in `action`.
    std::string_view word() const {
        return "device_sign_in_stop";
    }

    static ActionDeviceSignInStop read_fields(const Json::Value&) {
        return ActionDeviceSignInStop{};
    }

    void write_fields(std::string&, int&) const {
    }
};

struct ActionSignOut {
    bool everywhere = false;

    /// The word this variant carries in `action`.
    std::string_view word() const {
        return "sign_out";
    }

    static ActionSignOut read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "everywhere", "Action");
        return ActionSignOut{
                Json::as_bool(*raw0)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "everywhere");
        out += this->everywhere ? "true" : "false";
    }
};

struct ActionEnterProfile {
    std::optional<std::string> pin;
    std::string profile;

    /// The word this variant carries in `action`.
    std::string_view word() const {
        return "enter_profile";
    }

    static ActionEnterProfile read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::at(fields, "pin");
        const Json::Value* raw1 = Json::need(fields, "profile", "Action");
        return ActionEnterProfile{
                raw0 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw0)),
                Json::as_text(*raw1)};
    }

    void write_fields(std::string& out, int& count) const {
        if (this->pin.has_value()) {
            count = Json::key(out, count, "pin");
            Json::text(out, (*this->pin));
        }
        count = Json::key(out, count, "profile");
        Json::text(out, this->profile);
    }
};

struct ActionPlay {
    std::optional<EpisodeRef> episode;
    TitleKey title;

    /// The word this variant carries in `action`.
    std::string_view word() const {
        return "play";
    }

    static ActionPlay read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::at(fields, "episode");
        const Json::Value* raw1 = Json::need(fields, "title", "Action");
        return ActionPlay{
                raw0 == nullptr ? std::optional<EpisodeRef>() : std::optional<EpisodeRef>(EpisodeRef::read(*raw0)),
                TitleKey::read(*raw1)};
    }

    void write_fields(std::string& out, int& count) const {
        if (this->episode.has_value()) {
            count = Json::key(out, count, "episode");
            (*this->episode).write(out);
        }
        count = Json::key(out, count, "title");
        this->title.write(out);
    }
};

struct ActionPause {
    /// The word this variant carries in `action`.
    std::string_view word() const {
        return "pause";
    }

    static ActionPause read_fields(const Json::Value&) {
        return ActionPause{};
    }

    void write_fields(std::string&, int&) const {
    }
};

struct ActionUnpause {
    /// The word this variant carries in `action`.
    std::string_view word() const {
        return "unpause";
    }

    static ActionUnpause read_fields(const Json::Value&) {
        return ActionUnpause{};
    }

    void write_fields(std::string&, int&) const {
    }
};

struct ActionSeek {
    std::int64_t to_ms = 0;

    /// The word this variant carries in `action`.
    std::string_view word() const {
        return "seek";
    }

    static ActionSeek read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "to_ms", "Action");
        return ActionSeek{
                Json::as_long(*raw0)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "to_ms");
        out += std::to_string(this->to_ms);
    }
};

struct ActionSelectAudio {
    std::string track;

    /// The word this variant carries in `action`.
    std::string_view word() const {
        return "select_audio";
    }

    static ActionSelectAudio read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "track", "Action");
        return ActionSelectAudio{
                Json::as_text(*raw0)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "track");
        Json::text(out, this->track);
    }
};

struct ActionSelectText {
    std::optional<std::string> track;

    /// The word this variant carries in `action`.
    std::string_view word() const {
        return "select_text";
    }

    static ActionSelectText read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::at(fields, "track");
        return ActionSelectText{
                raw0 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw0))};
    }

    void write_fields(std::string& out, int& count) const {
        if (this->track.has_value()) {
            count = Json::key(out, count, "track");
            Json::text(out, (*this->track));
        }
    }
};

struct ActionNextSubtitleFile {
    /// The word this variant carries in `action`.
    std::string_view word() const {
        return "next_subtitle_file";
    }

    static ActionNextSubtitleFile read_fields(const Json::Value&) {
        return ActionNextSubtitleFile{};
    }

    void write_fields(std::string&, int&) const {
    }
};

struct ActionStop {
    /// The word this variant carries in `action`.
    std::string_view word() const {
        return "stop";
    }

    static ActionStop read_fields(const Json::Value&) {
        return ActionStop{};
    }

    void write_fields(std::string&, int&) const {
    }
};

struct ActionCommand {
    Command command;

    /// The word this variant carries in `action`.
    std::string_view word() const {
        return "command";
    }

    static ActionCommand read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "command", "Action");
        return ActionCommand{
                Command::read(*raw0)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "command");
        this->command.write(out);
    }
};

/// Send a prepared request with this session's token. The http effect takes the id of this event and its class is always plain, so whoever prepared it gets the answer under that id. Tokens stay in the core.
struct ActionRequest {
    std::optional<std::string> body;
    HttpMethod method;
    std::string path;

    /// The word this variant carries in `action`.
    std::string_view word() const {
        return "request";
    }

    static ActionRequest read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::at(fields, "body");
        const Json::Value* raw1 = Json::need(fields, "method", "Action");
        const Json::Value* raw2 = Json::need(fields, "path", "Action");
        return ActionRequest{
                raw0 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw0)),
                HttpMethod::read(*raw1),
                Json::as_text(*raw2)};
    }

    void write_fields(std::string& out, int& count) const {
        if (this->body.has_value()) {
            count = Json::key(out, count, "body");
            Json::text(out, (*this->body));
        }
        count = Json::key(out, count, "method");
        this->method.write(out);
        count = Json::key(out, count, "path");
        Json::text(out, this->path);
    }
};

/// A tag this build does not know.
struct ActionUnknown {
    /// The tag that was read.
    std::string tag;

    std::string_view word() const {
        return this->tag;
    }

    void write_fields(std::string&, int&) const {
    }
};

using ActionBody = std::variant<ActionSignIn, ActionSendCode, ActionSignInCode, ActionDeviceSignIn, ActionDeviceSignInStop, ActionSignOut, ActionEnterProfile, ActionPlay, ActionPause, ActionUnpause, ActionSeek, ActionSelectAudio, ActionSelectText, ActionNextSubtitleFile, ActionStop, ActionCommand, ActionRequest, ActionUnknown>;

/// What a screen asks for.
struct Action {
    ActionBody body;

    static Action read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        return Action{
                read_body(fields)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "action");
        Json::text(out, std::visit([](const auto& held) -> std::string_view { return held.word(); }, this->body));
        std::visit([&out, &count](const auto& held) { held.write_fields(out, count); }, this->body);
        out += '}';
    }

private:
    static ActionBody read_body(const Json::Value& fields) {
        const std::string& word = Json::as_text(*Json::need(fields, "action", "Action"));
        if (word == "sign_in") {
            return ActionSignIn::read_fields(fields);
        }
        if (word == "send_code") {
            return ActionSendCode::read_fields(fields);
        }
        if (word == "sign_in_code") {
            return ActionSignInCode::read_fields(fields);
        }
        if (word == "device_sign_in") {
            return ActionDeviceSignIn::read_fields(fields);
        }
        if (word == "device_sign_in_stop") {
            return ActionDeviceSignInStop::read_fields(fields);
        }
        if (word == "sign_out") {
            return ActionSignOut::read_fields(fields);
        }
        if (word == "enter_profile") {
            return ActionEnterProfile::read_fields(fields);
        }
        if (word == "play") {
            return ActionPlay::read_fields(fields);
        }
        if (word == "pause") {
            return ActionPause::read_fields(fields);
        }
        if (word == "unpause") {
            return ActionUnpause::read_fields(fields);
        }
        if (word == "seek") {
            return ActionSeek::read_fields(fields);
        }
        if (word == "select_audio") {
            return ActionSelectAudio::read_fields(fields);
        }
        if (word == "select_text") {
            return ActionSelectText::read_fields(fields);
        }
        if (word == "next_subtitle_file") {
            return ActionNextSubtitleFile::read_fields(fields);
        }
        if (word == "stop") {
            return ActionStop::read_fields(fields);
        }
        if (word == "command") {
            return ActionCommand::read_fields(fields);
        }
        if (word == "request") {
            return ActionRequest::read_fields(fields);
        }
        return ActionUnknown{word};
    }
};

/// Age card shown once at the start of play.
struct AgeCard {
    std::string label;
    std::optional<std::vector<std::string>> reasons;

    static AgeCard read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "label", "AgeCard");
        const Json::Value* raw1 = Json::at(fields, "reasons");
        return AgeCard{
                Json::as_text(*raw0),
                raw1 == nullptr ? std::optional<std::vector<std::string>>() : std::optional<std::vector<std::string>>(Json::list<std::string>(*raw1, [](const Json::Value& x0) { return Json::as_text(x0); }))};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "label");
        Json::text(out, this->label);
        if (this->reasons.has_value()) {
            count = Json::key(out, count, "reasons");
            out += '[';
            for (std::size_t index0 = 0; index0 < (*this->reasons).size(); index0++) {
                if (index0 > 0) {
                    out += ',';
                }
                Json::text(out, (*this->reasons)[index0]);
            }
            out += ']';
        }
        out += '}';
    }
};

/// The colour family of an age mark. The server picks it and each app paints it.
struct AgeTone {
    std::string value;

    static constexpr std::string_view green = "green";
    static constexpr std::string_view yellow = "yellow";
    static constexpr std::string_view orange = "orange";
    static constexpr std::string_view pink = "pink";
    static constexpr std::string_view red = "red";
    static constexpr std::string_view dark_red = "dark_red";
    static constexpr std::string_view blue = "blue";
    static constexpr std::string_view grey = "grey";

    static AgeTone read(const Json::Value& document) {
        return AgeTone{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// An age mark, as this viewer reads it. Each screen picks its shape.
struct AgeMark {
    std::string label;
    AgeTone tone;

    static AgeMark read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "label", "AgeMark");
        const Json::Value* raw1 = Json::need(fields, "tone", "AgeMark");
        return AgeMark{
                Json::as_text(*raw0),
                AgeTone::read(*raw1)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "label");
        Json::text(out, this->label);
        count = Json::key(out, count, "tone");
        this->tone.write(out);
        out += '}';
    }
};

/// How an attempt is played.
struct AttemptKind {
    std::string value;

    static constexpr std::string_view hls = "hls";
    static constexpr std::string_view file = "file";
    static constexpr std::string_view embed = "embed";

    static AttemptKind read(const Json::Value& document) {
        return AttemptKind{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// One way to play. Attempts arrive in the order to try, ranked once on the server. `headers` is required, even when empty, and no provider name is carried.
struct Attempt {
    std::optional<std::vector<std::string>> audio;
    std::optional<std::vector<std::string>> codecs;
    std::optional<std::string> container;
    /// Wall time, ms since 1970, after which the address stops working.
    std::optional<std::int64_t> expires_at_ms;
    /// Opaque group. After a failure the core skips other attempts of this group.
    std::string group;
    /// Headers the player must send, such as Referer, Origin, User-Agent.
    std::map<std::string, std::string> headers;
    std::optional<std::int64_t> height;
    std::string id;
    AttemptKind kind;
    std::string url;
    /// True when the finder fetched and read this address.
    bool verified = false;

    static Attempt read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "audio");
        const Json::Value* raw1 = Json::at(fields, "codecs");
        const Json::Value* raw2 = Json::at(fields, "container");
        const Json::Value* raw3 = Json::at(fields, "expires_at_ms");
        const Json::Value* raw4 = Json::need(fields, "group", "Attempt");
        const Json::Value* raw5 = Json::need(fields, "headers", "Attempt");
        const Json::Value* raw6 = Json::at(fields, "height");
        const Json::Value* raw7 = Json::need(fields, "id", "Attempt");
        const Json::Value* raw8 = Json::need(fields, "kind", "Attempt");
        const Json::Value* raw9 = Json::need(fields, "url", "Attempt");
        const Json::Value* raw10 = Json::need(fields, "verified", "Attempt");
        return Attempt{
                raw0 == nullptr ? std::optional<std::vector<std::string>>() : std::optional<std::vector<std::string>>(Json::list<std::string>(*raw0, [](const Json::Value& x0) { return Json::as_text(x0); })),
                raw1 == nullptr ? std::optional<std::vector<std::string>>() : std::optional<std::vector<std::string>>(Json::list<std::string>(*raw1, [](const Json::Value& x0) { return Json::as_text(x0); })),
                raw2 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw2)),
                raw3 == nullptr ? std::optional<std::int64_t>() : std::optional<std::int64_t>(Json::as_long(*raw3)),
                Json::as_text(*raw4),
                Json::map<std::string>(*raw5, [](const Json::Value& x0) { return Json::as_text(x0); }),
                raw6 == nullptr ? std::optional<std::int64_t>() : std::optional<std::int64_t>(Json::as_long(*raw6)),
                Json::as_text(*raw7),
                AttemptKind::read(*raw8),
                Json::as_text(*raw9),
                Json::as_bool(*raw10)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->audio.has_value()) {
            count = Json::key(out, count, "audio");
            out += '[';
            for (std::size_t index0 = 0; index0 < (*this->audio).size(); index0++) {
                if (index0 > 0) {
                    out += ',';
                }
                Json::text(out, (*this->audio)[index0]);
            }
            out += ']';
        }
        if (this->codecs.has_value()) {
            count = Json::key(out, count, "codecs");
            out += '[';
            for (std::size_t index0 = 0; index0 < (*this->codecs).size(); index0++) {
                if (index0 > 0) {
                    out += ',';
                }
                Json::text(out, (*this->codecs)[index0]);
            }
            out += ']';
        }
        if (this->container.has_value()) {
            count = Json::key(out, count, "container");
            Json::text(out, (*this->container));
        }
        if (this->expires_at_ms.has_value()) {
            count = Json::key(out, count, "expires_at_ms");
            out += std::to_string((*this->expires_at_ms));
        }
        count = Json::key(out, count, "group");
        Json::text(out, this->group);
        count = Json::key(out, count, "headers");
        out += '{';
        int n0 = 0;
        for (const auto& entry0 : this->headers) {
            n0 = Json::key(out, n0, entry0.first);
            Json::text(out, entry0.second);
        }
        out += '}';
        if (this->height.has_value()) {
            count = Json::key(out, count, "height");
            out += std::to_string((*this->height));
        }
        count = Json::key(out, count, "id");
        Json::text(out, this->id);
        count = Json::key(out, count, "kind");
        this->kind.write(out);
        count = Json::key(out, count, "url");
        Json::text(out, this->url);
        count = Json::key(out, count, "verified");
        out += this->verified ? "true" : "false";
        out += '}';
    }
};

/// One picture a profile can wear.
struct AvatarPicture {
    std::string name;
    /// Where the picture sits on this server.
    std::string path;

    static AvatarPicture read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "name", "AvatarPicture");
        const Json::Value* raw1 = Json::need(fields, "path", "AvatarPicture");
        return AvatarPicture{
                Json::as_text(*raw0),
                Json::as_text(*raw1)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "name");
        Json::text(out, this->name);
        count = Json::key(out, count, "path");
        Json::text(out, this->path);
        out += '}';
    }
};

/// A row of pictures on one theme. `kids` says a Kids profile may wear them.
struct AvatarSet {
    bool kids = false;
    std::string name;
    std::vector<AvatarPicture> pictures;

    static AvatarSet read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "kids", "AvatarSet");
        const Json::Value* raw1 = Json::need(fields, "name", "AvatarSet");
        const Json::Value* raw2 = Json::need(fields, "pictures", "AvatarSet");
        return AvatarSet{
                Json::as_bool(*raw0),
                Json::as_text(*raw1),
                Json::list<AvatarPicture>(*raw2, [](const Json::Value& x0) { return AvatarPicture::read(x0); })};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "kids");
        out += this->kids ? "true" : "false";
        count = Json::key(out, count, "name");
        Json::text(out, this->name);
        count = Json::key(out, count, "pictures");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->pictures.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->pictures[index0].write(out);
        }
        out += ']';
        out += '}';
    }
};

struct AvatarSets {
    std::vector<AvatarSet> sets;

    static AvatarSets read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "sets", "AvatarSets");
        return AvatarSets{
                Json::list<AvatarSet>(*raw0, [](const Json::Value& x0) { return AvatarSet::read(x0); })};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "sets");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->sets.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->sets[index0].write(out);
        }
        out += ']';
        out += '}';
    }
};

/// Marks on a tile.
struct BadgeKind {
    std::string value;

    static constexpr std::string_view ranked = "ranked";
    static constexpr std::string_view new_ = "new";

    static BadgeKind read(const Json::Value& document) {
        return BadgeKind{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// A mark on a tile.
struct Badge {
    BadgeKind kind;
    std::optional<std::int64_t> rank;

    static Badge read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "kind", "Badge");
        const Json::Value* raw1 = Json::at(fields, "rank");
        return Badge{
                BadgeKind::read(*raw0),
                raw1 == nullptr ? std::optional<std::int64_t>() : std::optional<std::int64_t>(Json::as_long(*raw1))};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "kind");
        this->kind.write(out);
        if (this->rank.has_value()) {
            count = Json::key(out, count, "rank");
            out += std::to_string((*this->rank));
        }
        out += '}';
    }
};

/// Where the body of an answer goes. An answer outside 200 to 299 always comes to the core, because the core reads its error code.
struct BodyTarget {
    std::string value;

    static constexpr std::string_view core = "core";
    static constexpr std::string_view screen = "screen";

    static BodyTarget read(const Json::Value& document) {
        return BodyTarget{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// Name and contact, filled into text at render time.
struct BrandValues {
    std::string name;
    std::string support_email;

    static BrandValues read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "name", "BrandValues");
        const Json::Value* raw1 = Json::need(fields, "support_email", "BrandValues");
        return BrandValues{
                Json::as_text(*raw0),
                Json::as_text(*raw1)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "name");
        Json::text(out, this->name);
        count = Json::key(out, count, "support_email");
        Json::text(out, this->support_email);
        out += '}';
    }
};

/// A day of the calendar, YYYY-MM-DD. The app writes it out in the viewer's own words.
struct CalendarDay {
    std::string value;

    explicit CalendarDay(std::string text) : value(std::move(text)) {
        if (!std::regex_match(this->value, shape())) {
            throw Refused("not a CalendarDay");
        }
    }

    static CalendarDay read(const Json::Value& document) {
        return CalendarDay(Json::as_text(document));
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }

private:
    static const std::regex& shape() {
        static const std::regex compiled("^[0-9]{4}-(0[1-9]|1[0-2])-(0[1-9]|[12][0-9]|3[01])$", std::regex::ECMAScript);
        return compiled;
    }
};

/// The form a client draws in. A browser on a phone is class web, form phone.
struct Form {
    std::string value;

    static constexpr std::string_view phone = "phone";
    static constexpr std::string_view tablet = "tablet";
    static constexpr std::string_view desktop = "desktop";
    static constexpr std::string_view tv = "tv";

    static Form read(const Json::Value& document) {
        return Form{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// Input a device has.
struct InputKind {
    std::string value;

    static constexpr std::string_view touch = "touch";
    static constexpr std::string_view pointer = "pointer";
    static constexpr std::string_view keyboard = "keyboard";
    static constexpr std::string_view remote = "remote";
    static constexpr std::string_view gamepad = "gamepad";
    static constexpr std::string_view rotary = "rotary";

    static InputKind read(const Json::Value& document) {
        return InputKind{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// Section kinds. A kind this build does not draw falls back to a plain row.
struct SectionKind {
    std::string value;

    static constexpr std::string_view hero = "hero";
    static constexpr std::string_view row = "row";
    static constexpr std::string_view ranked = "ranked";
    static constexpr std::string_view continue_ = "continue";
    static constexpr std::string_view grid = "grid";
    static constexpr std::string_view notice = "notice";

    static SectionKind read(const Json::Value& document) {
        return SectionKind{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// What a device can do. The server filters attempts and sections by this card.
struct CapabilityCard {
    /// True when the player switches variants during play.
    bool adaptive = false;
    std::string app;
    /// Attempt kinds this player opens. An embed appears only when listed here.
    std::vector<AttemptKind> attempt_kinds;
    DeviceClass class_;
    /// Codecs this player opens, such as avc1, hvc1, mp4a.
    std::vector<std::string> codecs;
    /// Containers this player opens, such as ts, fmp4, mp4.
    std::vector<std::string> containers;
    Form form;
    std::vector<InputKind> input;
    std::vector<std::string> languages;
    std::int64_t max_height = 0;
    /// Request headers the player can set on media, such as Referer.
    std::vector<std::string> media_headers;
    std::string os;
    std::string os_version;
    /// True when the driver reports video width or has_picture.
    bool reports_picture = false;
    std::int64_t screen_height = 0;
    std::int64_t screen_width = 0;
    /// True when the device has a durable secret store.
    bool secret_store = false;
    /// Section kinds this build draws. Others fall back to a plain row.
    std::vector<SectionKind> section_kinds;
    /// True when the player can load a side subtitle file.
    bool side_subtitles = false;
    /// Bytes this device gives to kept commands and screens. 0 means none.
    std::int64_t storage_bytes = 0;
    std::string version;

    static CapabilityCard read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "adaptive", "CapabilityCard");
        const Json::Value* raw1 = Json::need(fields, "app", "CapabilityCard");
        const Json::Value* raw2 = Json::need(fields, "attempt_kinds", "CapabilityCard");
        const Json::Value* raw3 = Json::need(fields, "class", "CapabilityCard");
        const Json::Value* raw4 = Json::need(fields, "codecs", "CapabilityCard");
        const Json::Value* raw5 = Json::need(fields, "containers", "CapabilityCard");
        const Json::Value* raw6 = Json::need(fields, "form", "CapabilityCard");
        const Json::Value* raw7 = Json::need(fields, "input", "CapabilityCard");
        const Json::Value* raw8 = Json::need(fields, "languages", "CapabilityCard");
        const Json::Value* raw9 = Json::need(fields, "max_height", "CapabilityCard");
        const Json::Value* raw10 = Json::need(fields, "media_headers", "CapabilityCard");
        const Json::Value* raw11 = Json::need(fields, "os", "CapabilityCard");
        const Json::Value* raw12 = Json::need(fields, "os_version", "CapabilityCard");
        const Json::Value* raw13 = Json::need(fields, "reports_picture", "CapabilityCard");
        const Json::Value* raw14 = Json::need(fields, "screen_height", "CapabilityCard");
        const Json::Value* raw15 = Json::need(fields, "screen_width", "CapabilityCard");
        const Json::Value* raw16 = Json::need(fields, "secret_store", "CapabilityCard");
        const Json::Value* raw17 = Json::need(fields, "section_kinds", "CapabilityCard");
        const Json::Value* raw18 = Json::need(fields, "side_subtitles", "CapabilityCard");
        const Json::Value* raw19 = Json::need(fields, "storage_bytes", "CapabilityCard");
        const Json::Value* raw20 = Json::need(fields, "version", "CapabilityCard");
        return CapabilityCard{
                Json::as_bool(*raw0),
                Json::as_text(*raw1),
                Json::list<AttemptKind>(*raw2, [](const Json::Value& x0) { return AttemptKind::read(x0); }),
                DeviceClass::read(*raw3),
                Json::list<std::string>(*raw4, [](const Json::Value& x0) { return Json::as_text(x0); }),
                Json::list<std::string>(*raw5, [](const Json::Value& x0) { return Json::as_text(x0); }),
                Form::read(*raw6),
                Json::list<InputKind>(*raw7, [](const Json::Value& x0) { return InputKind::read(x0); }),
                Json::list<std::string>(*raw8, [](const Json::Value& x0) { return Json::as_text(x0); }),
                Json::as_long(*raw9),
                Json::list<std::string>(*raw10, [](const Json::Value& x0) { return Json::as_text(x0); }),
                Json::as_text(*raw11),
                Json::as_text(*raw12),
                Json::as_bool(*raw13),
                Json::as_long(*raw14),
                Json::as_long(*raw15),
                Json::as_bool(*raw16),
                Json::list<SectionKind>(*raw17, [](const Json::Value& x0) { return SectionKind::read(x0); }),
                Json::as_bool(*raw18),
                Json::as_long(*raw19),
                Json::as_text(*raw20)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "adaptive");
        out += this->adaptive ? "true" : "false";
        count = Json::key(out, count, "app");
        Json::text(out, this->app);
        count = Json::key(out, count, "attempt_kinds");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->attempt_kinds.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->attempt_kinds[index0].write(out);
        }
        out += ']';
        count = Json::key(out, count, "class");
        this->class_.write(out);
        count = Json::key(out, count, "codecs");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->codecs.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            Json::text(out, this->codecs[index0]);
        }
        out += ']';
        count = Json::key(out, count, "containers");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->containers.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            Json::text(out, this->containers[index0]);
        }
        out += ']';
        count = Json::key(out, count, "form");
        this->form.write(out);
        count = Json::key(out, count, "input");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->input.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->input[index0].write(out);
        }
        out += ']';
        count = Json::key(out, count, "languages");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->languages.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            Json::text(out, this->languages[index0]);
        }
        out += ']';
        count = Json::key(out, count, "max_height");
        out += std::to_string(this->max_height);
        count = Json::key(out, count, "media_headers");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->media_headers.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            Json::text(out, this->media_headers[index0]);
        }
        out += ']';
        count = Json::key(out, count, "os");
        Json::text(out, this->os);
        count = Json::key(out, count, "os_version");
        Json::text(out, this->os_version);
        count = Json::key(out, count, "reports_picture");
        out += this->reports_picture ? "true" : "false";
        count = Json::key(out, count, "screen_height");
        out += std::to_string(this->screen_height);
        count = Json::key(out, count, "screen_width");
        out += std::to_string(this->screen_width);
        count = Json::key(out, count, "secret_store");
        out += this->secret_store ? "true" : "false";
        count = Json::key(out, count, "section_kinds");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->section_kinds.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->section_kinds[index0].write(out);
        }
        out += ']';
        count = Json::key(out, count, "side_subtitles");
        out += this->side_subtitles ? "true" : "false";
        count = Json::key(out, count, "storage_bytes");
        out += std::to_string(this->storage_bytes);
        count = Json::key(out, count, "version");
        Json::text(out, this->version);
        out += '}';
    }
};

/// List state of a title, versioned apart from its rating.
struct ListState {
    std::int64_t changed_at_ms = 0;
    bool in_list = false;
    std::int64_t version = 0;

    static ListState read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "changed_at_ms", "ListState");
        const Json::Value* raw1 = Json::need(fields, "in_list", "ListState");
        const Json::Value* raw2 = Json::need(fields, "version", "ListState");
        return ListState{
                Json::as_long(*raw0),
                Json::as_bool(*raw1),
                Json::as_long(*raw2)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "changed_at_ms");
        out += std::to_string(this->changed_at_ms);
        count = Json::key(out, count, "in_list");
        out += this->in_list ? "true" : "false";
        count = Json::key(out, count, "version");
        out += std::to_string(this->version);
        out += '}';
    }
};

/// A position, key by title and episode.
struct ProgressState {
    std::int64_t changed_at_ms = 0;
    bool completed = false;
    std::int64_t duration_s = 0;
    std::optional<EpisodeRef> episode;
    bool hidden = false;
    std::int64_t position_s = 0;
    TitleKey title;
    std::int64_t version = 0;

    static ProgressState read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "changed_at_ms", "ProgressState");
        const Json::Value* raw1 = Json::need(fields, "completed", "ProgressState");
        const Json::Value* raw2 = Json::need(fields, "duration_s", "ProgressState");
        const Json::Value* raw3 = Json::at(fields, "episode");
        const Json::Value* raw4 = Json::need(fields, "hidden", "ProgressState");
        const Json::Value* raw5 = Json::need(fields, "position_s", "ProgressState");
        const Json::Value* raw6 = Json::need(fields, "title", "ProgressState");
        const Json::Value* raw7 = Json::need(fields, "version", "ProgressState");
        return ProgressState{
                Json::as_long(*raw0),
                Json::as_bool(*raw1),
                Json::as_long(*raw2),
                raw3 == nullptr ? std::optional<EpisodeRef>() : std::optional<EpisodeRef>(EpisodeRef::read(*raw3)),
                Json::as_bool(*raw4),
                Json::as_long(*raw5),
                TitleKey::read(*raw6),
                Json::as_long(*raw7)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "changed_at_ms");
        out += std::to_string(this->changed_at_ms);
        count = Json::key(out, count, "completed");
        out += this->completed ? "true" : "false";
        count = Json::key(out, count, "duration_s");
        out += std::to_string(this->duration_s);
        if (this->episode.has_value()) {
            count = Json::key(out, count, "episode");
            (*this->episode).write(out);
        }
        count = Json::key(out, count, "hidden");
        out += this->hidden ? "true" : "false";
        count = Json::key(out, count, "position_s");
        out += std::to_string(this->position_s);
        count = Json::key(out, count, "title");
        this->title.write(out);
        count = Json::key(out, count, "version");
        out += std::to_string(this->version);
        out += '}';
    }
};

/// Rating state of a title, versioned apart from its list state.
struct RatingState {
    std::int64_t changed_at_ms = 0;
    /// Absent when no rating is set.
    std::optional<Rating> rating;
    std::int64_t version = 0;

    static RatingState read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "changed_at_ms", "RatingState");
        const Json::Value* raw1 = Json::at(fields, "rating");
        const Json::Value* raw2 = Json::need(fields, "version", "RatingState");
        return RatingState{
                Json::as_long(*raw0),
                raw1 == nullptr ? std::optional<Rating>() : std::optional<Rating>(Rating::read(*raw1)),
                Json::as_long(*raw2)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "changed_at_ms");
        out += std::to_string(this->changed_at_ms);
        if (this->rating.has_value()) {
            count = Json::key(out, count, "rating");
            (*this->rating).write(out);
        }
        count = Json::key(out, count, "version");
        out += std::to_string(this->version);
        out += '}';
    }
};

struct ChangeTitle {
    ListState list;
    RatingState rating;
    TitleKey title;

    /// The word this variant carries in `change`.
    std::string_view word() const {
        return "title";
    }

    static ChangeTitle read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "list", "Change");
        const Json::Value* raw1 = Json::need(fields, "rating", "Change");
        const Json::Value* raw2 = Json::need(fields, "title", "Change");
        return ChangeTitle{
                ListState::read(*raw0),
                RatingState::read(*raw1),
                TitleKey::read(*raw2)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "list");
        this->list.write(out);
        count = Json::key(out, count, "rating");
        this->rating.write(out);
        count = Json::key(out, count, "title");
        this->title.write(out);
    }
};

struct ChangeProgress {
    ProgressState state;

    /// The word this variant carries in `change`.
    std::string_view word() const {
        return "progress";
    }

    static ChangeProgress read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "state", "Change");
        return ChangeProgress{
                ProgressState::read(*raw0)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "state");
        this->state.write(out);
    }
};

/// Hidden as a whole, every episode of a series included.
struct ChangeHidden {
    TitleKey title;

    /// The word this variant carries in `change`.
    std::string_view word() const {
        return "hidden";
    }

    static ChangeHidden read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "title", "Change");
        return ChangeHidden{
                TitleKey::read(*raw0)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "title");
        this->title.write(out);
    }
};

struct ChangeHistoryCleared {
    std::int64_t changed_at_ms = 0;

    /// The word this variant carries in `change`.
    std::string_view word() const {
        return "history_cleared";
    }

    static ChangeHistoryCleared read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "changed_at_ms", "Change");
        return ChangeHistoryCleared{
                Json::as_long(*raw0)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "changed_at_ms");
        out += std::to_string(this->changed_at_ms);
    }
};

struct ChangeSetting {
    SettingKey key;
    std::string value;

    /// The word this variant carries in `change`.
    std::string_view word() const {
        return "setting";
    }

    static ChangeSetting read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "key", "Change");
        const Json::Value* raw1 = Json::need(fields, "value", "Change");
        return ChangeSetting{
                SettingKey::read(*raw0),
                Json::as_text(*raw1)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "key");
        this->key.write(out);
        count = Json::key(out, count, "value");
        Json::text(out, this->value);
    }
};

/// Reminder set or cleared on a coming title.
struct ChangeRemind {
    bool on = false;
    TitleKey title;

    /// The word this variant carries in `change`.
    std::string_view word() const {
        return "remind";
    }

    static ChangeRemind read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "on", "Change");
        const Json::Value* raw1 = Json::need(fields, "title", "Change");
        return ChangeRemind{
                Json::as_bool(*raw0),
                TitleKey::read(*raw1)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "on");
        out += this->on ? "true" : "false";
        count = Json::key(out, count, "title");
        this->title.write(out);
    }
};

/// A tag this build does not know.
struct ChangeUnknown {
    /// The tag that was read.
    std::string tag;

    std::string_view word() const {
        return this->tag;
    }

    void write_fields(std::string&, int&) const {
    }
};

using ChangeBody = std::variant<ChangeTitle, ChangeProgress, ChangeHidden, ChangeHistoryCleared, ChangeSetting, ChangeRemind, ChangeUnknown>;

/// One change a device reads.
struct Change {
    ChangeBody body;

    static Change read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        return Change{
                read_body(fields)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "change");
        Json::text(out, std::visit([](const auto& held) -> std::string_view { return held.word(); }, this->body));
        std::visit([&out, &count](const auto& held) { held.write_fields(out, count); }, this->body);
        out += '}';
    }

private:
    static ChangeBody read_body(const Json::Value& fields) {
        const std::string& word = Json::as_text(*Json::need(fields, "change", "Change"));
        if (word == "title") {
            return ChangeTitle::read_fields(fields);
        }
        if (word == "progress") {
            return ChangeProgress::read_fields(fields);
        }
        if (word == "hidden") {
            return ChangeHidden::read_fields(fields);
        }
        if (word == "history_cleared") {
            return ChangeHistoryCleared::read_fields(fields);
        }
        if (word == "setting") {
            return ChangeSetting::read_fields(fields);
        }
        if (word == "remind") {
            return ChangeRemind::read_fields(fields);
        }
        return ChangeUnknown{word};
    }
};

/// `GET /v1/changes`.
struct Changes {
    std::vector<Change> changes;
    std::string cursor;
    bool has_more = false;

    static Changes read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "changes", "Changes");
        const Json::Value* raw1 = Json::need(fields, "cursor", "Changes");
        const Json::Value* raw2 = Json::need(fields, "has_more", "Changes");
        return Changes{
                Json::list<Change>(*raw0, [](const Json::Value& x0) { return Change::read(x0); }),
                Json::as_text(*raw1),
                Json::as_bool(*raw2)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "changes");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->changes.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->changes[index0].write(out);
        }
        out += ']';
        count = Json::key(out, count, "cursor");
        Json::text(out, this->cursor);
        count = Json::key(out, count, "has_more");
        out += this->has_more ? "true" : "false";
        out += '}';
    }
};

/// One value per client class.
struct ClassValue {
    DeviceClass class_;
    std::int64_t value = 0;

    static ClassValue read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "class", "ClassValue");
        const Json::Value* raw1 = Json::need(fields, "value", "ClassValue");
        return ClassValue{
                DeviceClass::read(*raw0),
                Json::as_long(*raw1)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "class");
        this->class_.write(out);
        count = Json::key(out, count, "value");
        out += std::to_string(this->value);
        out += '}';
    }
};

/// Lowest app version a class may run.
struct ClientFloor {
    DeviceClass class_;
    std::string version;

    static ClientFloor read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "class", "ClientFloor");
        const Json::Value* raw1 = Json::need(fields, "version", "ClientFloor");
        return ClientFloor{
                DeviceClass::read(*raw0),
                Json::as_text(*raw1)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "class");
        this->class_.write(out);
        count = Json::key(out, count, "version");
        Json::text(out, this->version);
        out += '}';
    }
};

struct CodeSend {
    std::string email;

    static CodeSend read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "email", "CodeSend");
        return CodeSend{
                Json::as_text(*raw0)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "email");
        Json::text(out, this->email);
        out += '}';
    }
};

/// A code was sent, or would have been. The answer never says whether the account exists.
struct CodeSent {
    std::int64_t expires_in = 0;

    static CodeSent read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "expires_in", "CodeSent");
        return CodeSent{
                Json::as_long(*raw0)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "expires_in");
        out += std::to_string(this->expires_in);
        out += '}';
    }
};

struct CodeSignIn {
    std::string code;
    std::string email;

    static CodeSignIn read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "code", "CodeSignIn");
        const Json::Value* raw1 = Json::need(fields, "email", "CodeSignIn");
        return CodeSignIn{
                Json::as_text(*raw0),
                Json::as_text(*raw1)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "code");
        Json::text(out, this->code);
        count = Json::key(out, count, "email");
        Json::text(out, this->email);
        out += '}';
    }
};

/// Answer per op.
struct Outcome {
    std::string value;

    static constexpr std::string_view applied = "applied";
    static constexpr std::string_view superseded = "superseded";
    static constexpr std::string_view rejected = "rejected";

    static Outcome read(const Json::Value& document) {
        return Outcome{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// Answer for one op.
struct OpAnswer {
    std::string op_id;
    Outcome outcome;

    static OpAnswer read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "op_id", "OpAnswer");
        const Json::Value* raw1 = Json::need(fields, "outcome", "OpAnswer");
        return OpAnswer{
                Json::as_text(*raw0),
                Outcome::read(*raw1)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "op_id");
        Json::text(out, this->op_id);
        count = Json::key(out, count, "outcome");
        this->outcome.write(out);
        out += '}';
    }
};

/// Answer for a batch, one entry per op.
struct CommandAnswer {
    std::vector<OpAnswer> answers;

    static CommandAnswer read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "answers", "CommandAnswer");
        return CommandAnswer{
                Json::list<OpAnswer>(*raw0, [](const Json::Value& x0) { return OpAnswer::read(x0); })};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "answers");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->answers.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->answers[index0].write(out);
        }
        out += ']';
        out += '}';
    }
};

/// Add to or remove from the saved list. Leaves the rating alone.
struct CommandOpList {
    bool in_list = false;
    TitleKey title;

    /// The word this variant carries in `command`.
    std::string_view word() const {
        return "list";
    }

    static CommandOpList read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "in_list", "CommandOp");
        const Json::Value* raw1 = Json::need(fields, "title", "CommandOp");
        return CommandOpList{
                Json::as_bool(*raw0),
                TitleKey::read(*raw1)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "in_list");
        out += this->in_list ? "true" : "false";
        count = Json::key(out, count, "title");
        this->title.write(out);
    }
};

/// Set or clear a rating. Leaves the list entry alone.
struct CommandOpRate {
    std::optional<Rating> rating;
    TitleKey title;

    /// The word this variant carries in `command`.
    std::string_view word() const {
        return "rate";
    }

    static CommandOpRate read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::at(fields, "rating");
        const Json::Value* raw1 = Json::need(fields, "title", "CommandOp");
        return CommandOpRate{
                raw0 == nullptr ? std::optional<Rating>() : std::optional<Rating>(Rating::read(*raw0)),
                TitleKey::read(*raw1)};
    }

    void write_fields(std::string& out, int& count) const {
        if (this->rating.has_value()) {
            count = Json::key(out, count, "rating");
            (*this->rating).write(out);
        }
        count = Json::key(out, count, "title");
        this->title.write(out);
    }
};

/// Hide a title from history. For a series this covers every episode.
struct CommandOpHide {
    TitleKey title;

    /// The word this variant carries in `command`.
    std::string_view word() const {
        return "hide";
    }

    static CommandOpHide read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "title", "CommandOp");
        return CommandOpHide{
                TitleKey::read(*raw0)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "title");
        this->title.write(out);
    }
};

/// Clear all history of this profile, as one operation.
struct CommandOpClearHistory {
    /// The word this variant carries in `command`.
    std::string_view word() const {
        return "clear_history";
    }

    static CommandOpClearHistory read_fields(const Json::Value&) {
        return CommandOpClearHistory{};
    }

    void write_fields(std::string&, int&) const {
    }
};

/// Save a position, in whole seconds.
struct CommandOpPosition {
    std::int64_t duration_s = 0;
    std::optional<EpisodeRef> episode;
    std::int64_t position_s = 0;
    TitleKey title;

    /// The word this variant carries in `command`.
    std::string_view word() const {
        return "position";
    }

    static CommandOpPosition read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "duration_s", "CommandOp");
        const Json::Value* raw1 = Json::at(fields, "episode");
        const Json::Value* raw2 = Json::need(fields, "position_s", "CommandOp");
        const Json::Value* raw3 = Json::need(fields, "title", "CommandOp");
        return CommandOpPosition{
                Json::as_long(*raw0),
                raw1 == nullptr ? std::optional<EpisodeRef>() : std::optional<EpisodeRef>(EpisodeRef::read(*raw1)),
                Json::as_long(*raw2),
                TitleKey::read(*raw3)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "duration_s");
        out += std::to_string(this->duration_s);
        if (this->episode.has_value()) {
            count = Json::key(out, count, "episode");
            (*this->episode).write(out);
        }
        count = Json::key(out, count, "position_s");
        out += std::to_string(this->position_s);
        count = Json::key(out, count, "title");
        this->title.write(out);
    }
};

/// Change one profile setting. Booleans as true or false, style as JSON text.
struct CommandOpSetting {
    SettingKey key;
    std::string value;

    /// The word this variant carries in `command`.
    std::string_view word() const {
        return "setting";
    }

    static CommandOpSetting read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "key", "CommandOp");
        const Json::Value* raw1 = Json::need(fields, "value", "CommandOp");
        return CommandOpSetting{
                SettingKey::read(*raw0),
                Json::as_text(*raw1)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "key");
        this->key.write(out);
        count = Json::key(out, count, "value");
        Json::text(out, this->value);
    }
};

/// A viewer signal. Discarded by the server when consent is off.
struct CommandOpFeedback {
    FeedbackEvent event;
    TitleKey title;

    /// The word this variant carries in `command`.
    std::string_view word() const {
        return "feedback";
    }

    static CommandOpFeedback read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "event", "CommandOp");
        const Json::Value* raw1 = Json::need(fields, "title", "CommandOp");
        return CommandOpFeedback{
                FeedbackEvent::read(*raw0),
                TitleKey::read(*raw1)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "event");
        this->event.write(out);
        count = Json::key(out, count, "title");
        this->title.write(out);
    }
};

/// A trailer failed to play for this title.
struct CommandOpTrailerRefused {
    TitleKey title;

    /// The word this variant carries in `command`.
    std::string_view word() const {
        return "trailer_refused";
    }

    static CommandOpTrailerRefused read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "title", "CommandOp");
        return CommandOpTrailerRefused{
                TitleKey::read(*raw0)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "title");
        this->title.write(out);
    }
};

/// Set or clear a reminder on a coming title.
struct CommandOpRemind {
    bool on = false;
    TitleKey title;

    /// The word this variant carries in `command`.
    std::string_view word() const {
        return "remind";
    }

    static CommandOpRemind read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "on", "CommandOp");
        const Json::Value* raw1 = Json::need(fields, "title", "CommandOp");
        return CommandOpRemind{
                Json::as_bool(*raw0),
                TitleKey::read(*raw1)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "on");
        out += this->on ? "true" : "false";
        count = Json::key(out, count, "title");
        this->title.write(out);
    }
};

/// A tag this build does not know.
struct CommandOpUnknown {
    /// The tag that was read.
    std::string tag;

    std::string_view word() const {
        return this->tag;
    }

    void write_fields(std::string&, int&) const {
    }
};

using CommandOpBody = std::variant<CommandOpList, CommandOpRate, CommandOpHide, CommandOpClearHistory, CommandOpPosition, CommandOpSetting, CommandOpFeedback, CommandOpTrailerRefused, CommandOpRemind, CommandOpUnknown>;

/// One queued op. A missing `age_ms` means age unknown, and the server orders by arrival.
struct CommandOp {
    std::optional<std::int64_t> age_ms;
    std::string op_id;
    CommandOpBody body;

    static CommandOp read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "age_ms");
        const Json::Value* raw1 = Json::need(fields, "op_id", "CommandOp");
        return CommandOp{
                raw0 == nullptr ? std::optional<std::int64_t>() : std::optional<std::int64_t>(Json::as_long(*raw0)),
                Json::as_text(*raw1),
                read_body(fields)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "command");
        Json::text(out, std::visit([](const auto& held) -> std::string_view { return held.word(); }, this->body));
        if (this->age_ms.has_value()) {
            count = Json::key(out, count, "age_ms");
            out += std::to_string((*this->age_ms));
        }
        count = Json::key(out, count, "op_id");
        Json::text(out, this->op_id);
        std::visit([&out, &count](const auto& held) { held.write_fields(out, count); }, this->body);
        out += '}';
    }

private:
    static CommandOpBody read_body(const Json::Value& fields) {
        const std::string& word = Json::as_text(*Json::need(fields, "command", "CommandOp"));
        if (word == "list") {
            return CommandOpList::read_fields(fields);
        }
        if (word == "rate") {
            return CommandOpRate::read_fields(fields);
        }
        if (word == "hide") {
            return CommandOpHide::read_fields(fields);
        }
        if (word == "clear_history") {
            return CommandOpClearHistory::read_fields(fields);
        }
        if (word == "position") {
            return CommandOpPosition::read_fields(fields);
        }
        if (word == "setting") {
            return CommandOpSetting::read_fields(fields);
        }
        if (word == "feedback") {
            return CommandOpFeedback::read_fields(fields);
        }
        if (word == "trailer_refused") {
            return CommandOpTrailerRefused::read_fields(fields);
        }
        if (word == "remind") {
            return CommandOpRemind::read_fields(fields);
        }
        return CommandOpUnknown{word};
    }
};

/// `POST /v1/commands`. Idempotent by op id.
struct CommandBatch {
    std::vector<CommandOp> ops;

    static CommandBatch read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "ops", "CommandBatch");
        return CommandBatch{
                Json::list<CommandOp>(*raw0, [](const Json::Value& x0) { return CommandOp::read(x0); })};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "ops");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->ops.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->ops[index0].write(out);
        }
        out += ']';
        out += '}';
    }
};

/// Step up. Send a password or an emailed code.
struct Confirm {
    std::optional<std::string> code;
    std::optional<std::string> password;

    static Confirm read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "code");
        const Json::Value* raw1 = Json::at(fields, "password");
        return Confirm{
                raw0 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw0)),
                raw1 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw1))};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->code.has_value()) {
            count = Json::key(out, count, "code");
            Json::text(out, (*this->code));
        }
        if (this->password.has_value()) {
            count = Json::key(out, count, "password");
            Json::text(out, (*this->password));
        }
        out += '}';
    }
};

/// A step up holds for this many seconds and a refresh never renews it.
struct Confirmed {
    std::int64_t valid_s = 0;

    static Confirmed read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "valid_s", "Confirmed");
        return Confirmed{
                Json::as_long(*raw0)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "valid_s");
        out += std::to_string(this->valid_s);
        out += '}';
    }
};

/// Consent for personal rows.
struct Consent {
    bool personal = false;

    static Consent read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "personal", "Consent");
        return Consent{
                Json::as_bool(*raw0)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "personal");
        out += this->personal ? "true" : "false";
        out += '}';
    }
};

/// Type tag of `CoreConfig`.
struct CoreConfigTag {
    static constexpr std::string_view word = "core_config";

    static CoreConfigTag read(const Json::Value& document) {
        const std::string& found = Json::as_text(document);
        if (found != word) {
            throw Refused("not a CoreConfigTag: " + found);
        }
        return CoreConfigTag{};
    }

    void write(std::string& out) const {
        Json::text(out, word);
    }
};

/// Where the core runs for this device.
struct CoreMode {
    std::string value;

    static constexpr std::string_view local = "local";
    static constexpr std::string_view hosted = "hosted";

    static CoreMode read(const Json::Value& document) {
        return CoreMode{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// A wire date, YYYY-MM-DD.
struct WireDate {
    std::string value;

    explicit WireDate(std::string text) : value(std::move(text)) {
        if (!std::regex_match(this->value, shape())) {
            throw Refused("not a WireDate");
        }
    }

    static WireDate read(const Json::Value& document) {
        return WireDate(Json::as_text(document));
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }

private:
    static const std::regex& shape() {
        static const std::regex compiled("^[0-9]{4}-[0-9]{2}-[0-9]{2}$", std::regex::ECMAScript);
        return compiled;
    }
};

/// Given to `core_open`, once. The seed is the only random input the core ever gets.
struct CoreConfig {
    std::string app;
    /// What this device can do. The core sends its hash after sign in.
    std::optional<CapabilityCard> card;
    DeviceClass class_;
    WireDate contract;
    Form form;
    CoreMode mode;
    std::string os;
    /// Hex digits, at least 32.
    std::string seed;
    CoreConfigTag type;
    std::string version;

    static CoreConfig read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "app", "CoreConfig");
        const Json::Value* raw1 = Json::at(fields, "card");
        const Json::Value* raw2 = Json::need(fields, "class", "CoreConfig");
        const Json::Value* raw3 = Json::need(fields, "contract", "CoreConfig");
        const Json::Value* raw4 = Json::need(fields, "form", "CoreConfig");
        const Json::Value* raw5 = Json::need(fields, "mode", "CoreConfig");
        const Json::Value* raw6 = Json::need(fields, "os", "CoreConfig");
        const Json::Value* raw7 = Json::need(fields, "seed", "CoreConfig");
        const Json::Value* raw8 = Json::need(fields, "type", "CoreConfig");
        const Json::Value* raw9 = Json::need(fields, "version", "CoreConfig");
        return CoreConfig{
                Json::as_text(*raw0),
                raw1 == nullptr ? std::optional<CapabilityCard>() : std::optional<CapabilityCard>(CapabilityCard::read(*raw1)),
                DeviceClass::read(*raw2),
                WireDate::read(*raw3),
                Form::read(*raw4),
                CoreMode::read(*raw5),
                Json::as_text(*raw6),
                Json::as_text(*raw7),
                CoreConfigTag::read(*raw8),
                Json::as_text(*raw9)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "app");
        Json::text(out, this->app);
        if (this->card.has_value()) {
            count = Json::key(out, count, "card");
            (*this->card).write(out);
        }
        count = Json::key(out, count, "class");
        this->class_.write(out);
        count = Json::key(out, count, "contract");
        this->contract.write(out);
        count = Json::key(out, count, "form");
        this->form.write(out);
        count = Json::key(out, count, "mode");
        this->mode.write(out);
        count = Json::key(out, count, "os");
        Json::text(out, this->os);
        count = Json::key(out, count, "seed");
        Json::text(out, this->seed);
        count = Json::key(out, count, "type");
        this->type.write(out);
        count = Json::key(out, count, "version");
        Json::text(out, this->version);
        out += '}';
    }
};

/// Type tag of `CoreIn`.
struct CoreInTag {
    static constexpr std::string_view word = "core_in";

    static CoreInTag read(const Json::Value& document) {
        const std::string& found = Json::as_text(document);
        if (found != word) {
            throw Refused("not a CoreInTag: " + found);
        }
        return CoreInTag{};
    }

    void write(std::string& out) const {
        Json::text(out, word);
    }
};

/// Failure classes a driver reports. The core picks the next attempt from these.
struct ErrorClass {
    std::string value;

    static constexpr std::string_view network = "network";
    static constexpr std::string_view manifest = "manifest";
    static constexpr std::string_view segment = "segment";
    static constexpr std::string_view decode = "decode";
    static constexpr std::string_view key = "key";
    static constexpr std::string_view format = "format";
    static constexpr std::string_view refused = "refused";

    static ErrorClass read(const Json::Value& document) {
        return ErrorClass{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// One audio or text track, with the tag the manifest gave it.
struct Track {
    std::string id;
    std::optional<std::string> label;
    std::optional<std::string> lang;
    /// True for the track playing now.
    std::optional<bool> selected;

    static Track read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "id", "Track");
        const Json::Value* raw1 = Json::at(fields, "label");
        const Json::Value* raw2 = Json::at(fields, "lang");
        const Json::Value* raw3 = Json::at(fields, "selected");
        return Track{
                Json::as_text(*raw0),
                raw1 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw1)),
                raw2 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw2)),
                raw3 == nullptr ? std::optional<bool>() : std::optional<bool>(Json::as_bool(*raw3))};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "id");
        Json::text(out, this->id);
        if (this->label.has_value()) {
            count = Json::key(out, count, "label");
            Json::text(out, (*this->label));
        }
        if (this->lang.has_value()) {
            count = Json::key(out, count, "lang");
            Json::text(out, (*this->lang));
        }
        if (this->selected.has_value()) {
            count = Json::key(out, count, "selected");
            out += (*this->selected) ? "true" : "false";
        }
        out += '}';
    }
};

struct DriverEventFirstFrame {
    /// The word this variant carries in `kind`.
    std::string_view word() const {
        return "first_frame";
    }

    static DriverEventFirstFrame read_fields(const Json::Value&) {
        return DriverEventFirstFrame{};
    }

    void write_fields(std::string&, int&) const {
    }
};

struct DriverEventPosition {
    /// 0 or absent when not known.
    std::optional<std::int64_t> duration_ms;
    /// Absent means unknown, and the no picture rule stays off.
    std::optional<bool> has_picture;
    std::int64_t time_ms = 0;
    std::optional<std::int64_t> video_width;

    /// The word this variant carries in `kind`.
    std::string_view word() const {
        return "position";
    }

    static DriverEventPosition read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::at(fields, "duration_ms");
        const Json::Value* raw1 = Json::at(fields, "has_picture");
        const Json::Value* raw2 = Json::need(fields, "time_ms", "DriverEvent");
        const Json::Value* raw3 = Json::at(fields, "video_width");
        return DriverEventPosition{
                raw0 == nullptr ? std::optional<std::int64_t>() : std::optional<std::int64_t>(Json::as_long(*raw0)),
                raw1 == nullptr ? std::optional<bool>() : std::optional<bool>(Json::as_bool(*raw1)),
                Json::as_long(*raw2),
                raw3 == nullptr ? std::optional<std::int64_t>() : std::optional<std::int64_t>(Json::as_long(*raw3))};
    }

    void write_fields(std::string& out, int& count) const {
        if (this->duration_ms.has_value()) {
            count = Json::key(out, count, "duration_ms");
            out += std::to_string((*this->duration_ms));
        }
        if (this->has_picture.has_value()) {
            count = Json::key(out, count, "has_picture");
            out += (*this->has_picture) ? "true" : "false";
        }
        count = Json::key(out, count, "time_ms");
        out += std::to_string(this->time_ms);
        if (this->video_width.has_value()) {
            count = Json::key(out, count, "video_width");
            out += std::to_string((*this->video_width));
        }
    }
};

struct DriverEventStalled {
    /// The word this variant carries in `kind`.
    std::string_view word() const {
        return "stalled";
    }

    static DriverEventStalled read_fields(const Json::Value&) {
        return DriverEventStalled{};
    }

    void write_fields(std::string&, int&) const {
    }
};

struct DriverEventEnded {
    /// The word this variant carries in `kind`.
    std::string_view word() const {
        return "ended";
    }

    static DriverEventEnded read_fields(const Json::Value&) {
        return DriverEventEnded{};
    }

    void write_fields(std::string&, int&) const {
    }
};

struct DriverEventTracks {
    std::vector<Track> audio;
    std::vector<Track> text;

    /// The word this variant carries in `kind`.
    std::string_view word() const {
        return "tracks";
    }

    static DriverEventTracks read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "audio", "DriverEvent");
        const Json::Value* raw1 = Json::need(fields, "text", "DriverEvent");
        return DriverEventTracks{
                Json::list<Track>(*raw0, [](const Json::Value& x0) { return Track::read(x0); }),
                Json::list<Track>(*raw1, [](const Json::Value& x0) { return Track::read(x0); })};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "audio");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->audio.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->audio[index0].write(out);
        }
        out += ']';
        count = Json::key(out, count, "text");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->text.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->text[index0].write(out);
        }
        out += ']';
    }
};

struct DriverEventError {
    ErrorClass class_;

    /// The word this variant carries in `kind`.
    std::string_view word() const {
        return "error";
    }

    static DriverEventError read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "class", "DriverEvent");
        return DriverEventError{
                ErrorClass::read(*raw0)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "class");
        this->class_.write(out);
    }
};

/// A tag this build does not know.
struct DriverEventUnknown {
    /// The tag that was read.
    std::string tag;

    std::string_view word() const {
        return this->tag;
    }

    void write_fields(std::string&, int&) const {
    }
};

using DriverEventBody = std::variant<DriverEventFirstFrame, DriverEventPosition, DriverEventStalled, DriverEventEnded, DriverEventTracks, DriverEventError, DriverEventUnknown>;

/// What the driver reports. No driver decides failover or no picture by itself.
struct DriverEvent {
    DriverEventBody body;

    static DriverEvent read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        return DriverEvent{
                read_body(fields)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "kind");
        Json::text(out, std::visit([](const auto& held) -> std::string_view { return held.word(); }, this->body));
        std::visit([&out, &count](const auto& held) { held.write_fields(out, count); }, this->body);
        out += '}';
    }

private:
    static DriverEventBody read_body(const Json::Value& fields) {
        const std::string& word = Json::as_text(*Json::need(fields, "kind", "DriverEvent"));
        if (word == "first_frame") {
            return DriverEventFirstFrame::read_fields(fields);
        }
        if (word == "position") {
            return DriverEventPosition::read_fields(fields);
        }
        if (word == "stalled") {
            return DriverEventStalled::read_fields(fields);
        }
        if (word == "ended") {
            return DriverEventEnded::read_fields(fields);
        }
        if (word == "tracks") {
            return DriverEventTracks::read_fields(fields);
        }
        if (word == "error") {
            return DriverEventError::read_fields(fields);
        }
        return DriverEventUnknown{word};
    }
};

/// Why an http effect got no answer. Any status at all is an answer.
struct NoAnswerReason {
    std::string value;

    static constexpr std::string_view offline = "offline";
    static constexpr std::string_view timeout = "timeout";
    static constexpr std::string_view tls = "tls";
    static constexpr std::string_view clock_wrong = "clock_wrong";
    static constexpr std::string_view cancelled = "cancelled";

    static NoAnswerReason read(const Json::Value& document) {
        return NoAnswerReason{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// An http effect got an answer. Every status is data.
struct EventHttpDone {
    /// Absent when the body went to the screen.
    std::optional<std::string> body;
    std::optional<std::map<std::string, std::string>> headers;
    std::int64_t status = 0;

    /// The word this variant carries in `type`.
    std::string_view word() const {
        return "http_done";
    }

    static EventHttpDone read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::at(fields, "body");
        const Json::Value* raw1 = Json::at(fields, "headers");
        const Json::Value* raw2 = Json::need(fields, "status", "Event");
        return EventHttpDone{
                raw0 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw0)),
                raw1 == nullptr ? std::optional<std::map<std::string, std::string>>() : std::optional<std::map<std::string, std::string>>(Json::map<std::string>(*raw1, [](const Json::Value& x0) { return Json::as_text(x0); })),
                Json::as_long(*raw2)};
    }

    void write_fields(std::string& out, int& count) const {
        if (this->body.has_value()) {
            count = Json::key(out, count, "body");
            Json::text(out, (*this->body));
        }
        if (this->headers.has_value()) {
            count = Json::key(out, count, "headers");
            out += '{';
            int n0 = 0;
            for (const auto& entry0 : (*this->headers)) {
                n0 = Json::key(out, n0, entry0.first);
                Json::text(out, entry0.second);
            }
            out += '}';
        }
        count = Json::key(out, count, "status");
        out += std::to_string(this->status);
    }
};

/// An http effect got no answer at all.
struct EventHttpFailed {
    NoAnswerReason reason;

    /// The word this variant carries in `type`.
    std::string_view word() const {
        return "http_failed";
    }

    static EventHttpFailed read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "reason", "Event");
        return EventHttpFailed{
                NoAnswerReason::read(*raw0)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "reason");
        this->reason.write(out);
    }
};

struct EventStoreDone {
    std::optional<std::vector<std::string>> keys;
    std::optional<std::string> value;

    /// The word this variant carries in `type`.
    std::string_view word() const {
        return "store_done";
    }

    static EventStoreDone read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::at(fields, "keys");
        const Json::Value* raw1 = Json::at(fields, "value");
        return EventStoreDone{
                raw0 == nullptr ? std::optional<std::vector<std::string>>() : std::optional<std::vector<std::string>>(Json::list<std::string>(*raw0, [](const Json::Value& x0) { return Json::as_text(x0); })),
                raw1 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw1))};
    }

    void write_fields(std::string& out, int& count) const {
        if (this->keys.has_value()) {
            count = Json::key(out, count, "keys");
            out += '[';
            for (std::size_t index0 = 0; index0 < (*this->keys).size(); index0++) {
                if (index0 > 0) {
                    out += ',';
                }
                Json::text(out, (*this->keys)[index0]);
            }
            out += ']';
        }
        if (this->value.has_value()) {
            count = Json::key(out, count, "value");
            Json::text(out, (*this->value));
        }
    }
};

struct EventSecretDone {
    std::optional<std::string> value;

    /// The word this variant carries in `type`.
    std::string_view word() const {
        return "secret_done";
    }

    static EventSecretDone read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::at(fields, "value");
        return EventSecretDone{
                raw0 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw0))};
    }

    void write_fields(std::string& out, int& count) const {
        if (this->value.has_value()) {
            count = Json::key(out, count, "value");
            Json::text(out, (*this->value));
        }
    }
};

struct EventTimerFired {
    /// The word this variant carries in `type`.
    std::string_view word() const {
        return "timer_fired";
    }

    static EventTimerFired read_fields(const Json::Value&) {
        return EventTimerFired{};
    }

    void write_fields(std::string&, int&) const {
    }
};

/// The host cannot perform this effect.
struct EventUnsupported {
    /// The word this variant carries in `type`.
    std::string_view word() const {
        return "unsupported";
    }

    static EventUnsupported read_fields(const Json::Value&) {
        return EventUnsupported{};
    }

    void write_fields(std::string&, int&) const {
    }
};

/// The effect with this id was dropped.
struct EventCancelled {
    /// The word this variant carries in `type`.
    std::string_view word() const {
        return "cancelled";
    }

    static EventCancelled read_fields(const Json::Value&) {
        return EventCancelled{};
    }

    void write_fields(std::string&, int&) const {
    }
};

struct EventSuspend {
    /// The word this variant carries in `type`.
    std::string_view word() const {
        return "suspend";
    }

    static EventSuspend read_fields(const Json::Value&) {
        return EventSuspend{};
    }

    void write_fields(std::string&, int&) const {
    }
};

struct EventResume {
    /// The word this variant carries in `type`.
    std::string_view word() const {
        return "resume";
    }

    static EventResume read_fields(const Json::Value&) {
        return EventResume{};
    }

    void write_fields(std::string&, int&) const {
    }
};

struct EventVisibility {
    bool visible = false;

    /// The word this variant carries in `type`.
    std::string_view word() const {
        return "visibility";
    }

    static EventVisibility read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "visible", "Event");
        return EventVisibility{
                Json::as_bool(*raw0)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "visible");
        out += this->visible ? "true" : "false";
    }
};

struct EventNetwork {
    bool online = false;

    /// The word this variant carries in `type`.
    std::string_view word() const {
        return "network";
    }

    static EventNetwork read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "online", "Event");
        return EventNetwork{
                Json::as_bool(*raw0)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "online");
        out += this->online ? "true" : "false";
    }
};

/// Touch, key, wheel or playing video in view.
struct EventActivity {
    /// The word this variant carries in `type`.
    std::string_view word() const {
        return "activity";
    }

    static EventActivity read_fields(const Json::Value&) {
        return EventActivity{};
    }

    void write_fields(std::string&, int&) const {
    }
};

/// Another tab of this browser ended the session. The server has heard already.
struct EventEndedElsewhere {
    /// The word this variant carries in `type`.
    std::string_view word() const {
        return "ended_elsewhere";
    }

    static EventEndedElsewhere read_fields(const Json::Value&) {
        return EventEndedElsewhere{};
    }

    void write_fields(std::string&, int&) const {
    }
};

/// An address or deep link to open.
struct EventAddress {
    std::string address;

    /// The word this variant carries in `type`.
    std::string_view word() const {
        return "address";
    }

    static EventAddress read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "address", "Event");
        return EventAddress{
                Json::as_text(*raw0)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "address");
        Json::text(out, this->address);
    }
};

struct EventAction {
    Action action;

    /// The word this variant carries in `type`.
    std::string_view word() const {
        return "action";
    }

    static EventAction read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "action", "Event");
        return EventAction{
                Action::read(*raw0)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "action");
        this->action.write(out);
    }
};

/// A driver event, stamped with the generation of the open that made it.
struct EventDriver {
    DriverEvent event;
    std::int64_t generation = 0;

    /// The word this variant carries in `type`.
    std::string_view word() const {
        return "driver";
    }

    static EventDriver read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "event", "Event");
        const Json::Value* raw1 = Json::need(fields, "generation", "Event");
        return EventDriver{
                DriverEvent::read(*raw0),
                Json::as_long(*raw1)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "event");
        this->event.write(out);
        count = Json::key(out, count, "generation");
        out += std::to_string(this->generation);
    }
};

/// A tag this build does not know.
struct EventUnknown {
    /// The tag that was read.
    std::string tag;

    std::string_view word() const {
        return this->tag;
    }

    void write_fields(std::string&, int&) const {
    }
};

using EventBody = std::variant<EventHttpDone, EventHttpFailed, EventStoreDone, EventSecretDone, EventTimerFired, EventUnsupported, EventCancelled, EventSuspend, EventResume, EventVisibility, EventNetwork, EventActivity, EventEndedElsewhere, EventAddress, EventAction, EventDriver, EventUnknown>;

/// One event. `id` is the id of the effect it answers, when it answers one.
struct Event {
    std::optional<std::string> id;
    EventBody body;

    static Event read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "id");
        return Event{
                raw0 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw0)),
                read_body(fields)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "type");
        Json::text(out, std::visit([](const auto& held) -> std::string_view { return held.word(); }, this->body));
        if (this->id.has_value()) {
            count = Json::key(out, count, "id");
            Json::text(out, (*this->id));
        }
        std::visit([&out, &count](const auto& held) { held.write_fields(out, count); }, this->body);
        out += '}';
    }

private:
    static EventBody read_body(const Json::Value& fields) {
        const std::string& word = Json::as_text(*Json::need(fields, "type", "Event"));
        if (word == "http_done") {
            return EventHttpDone::read_fields(fields);
        }
        if (word == "http_failed") {
            return EventHttpFailed::read_fields(fields);
        }
        if (word == "store_done") {
            return EventStoreDone::read_fields(fields);
        }
        if (word == "secret_done") {
            return EventSecretDone::read_fields(fields);
        }
        if (word == "timer_fired") {
            return EventTimerFired::read_fields(fields);
        }
        if (word == "unsupported") {
            return EventUnsupported::read_fields(fields);
        }
        if (word == "cancelled") {
            return EventCancelled::read_fields(fields);
        }
        if (word == "suspend") {
            return EventSuspend::read_fields(fields);
        }
        if (word == "resume") {
            return EventResume::read_fields(fields);
        }
        if (word == "visibility") {
            return EventVisibility::read_fields(fields);
        }
        if (word == "network") {
            return EventNetwork::read_fields(fields);
        }
        if (word == "activity") {
            return EventActivity::read_fields(fields);
        }
        if (word == "ended_elsewhere") {
            return EventEndedElsewhere::read_fields(fields);
        }
        if (word == "address") {
            return EventAddress::read_fields(fields);
        }
        if (word == "action") {
            return EventAction::read_fields(fields);
        }
        if (word == "driver") {
            return EventDriver::read_fields(fields);
        }
        return EventUnknown{word};
    }
};

/// One step of input. Every input carries wall time and monotonic time.
struct CoreIn {
    WireDate contract;
    std::vector<Event> events;
    /// Monotonic time, ms. Restarts at boot.
    std::int64_t mono_ms = 0;
    /// Wall time, ms since 1970.
    std::int64_t now_ms = 0;
    CoreInTag type;

    static CoreIn read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "contract", "CoreIn");
        const Json::Value* raw1 = Json::need(fields, "events", "CoreIn");
        const Json::Value* raw2 = Json::need(fields, "mono_ms", "CoreIn");
        const Json::Value* raw3 = Json::need(fields, "now_ms", "CoreIn");
        const Json::Value* raw4 = Json::need(fields, "type", "CoreIn");
        return CoreIn{
                WireDate::read(*raw0),
                Json::list<Event>(*raw1, [](const Json::Value& x0) { return Event::read(x0); }),
                Json::as_long(*raw2),
                Json::as_long(*raw3),
                CoreInTag::read(*raw4)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "contract");
        this->contract.write(out);
        count = Json::key(out, count, "events");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->events.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->events[index0].write(out);
        }
        out += ']';
        count = Json::key(out, count, "mono_ms");
        out += std::to_string(this->mono_ms);
        count = Json::key(out, count, "now_ms");
        out += std::to_string(this->now_ms);
        count = Json::key(out, count, "type");
        this->type.write(out);
        out += '}';
    }
};

/// Type tag of `CoreOut`.
struct CoreOutTag {
    static constexpr std::string_view word = "core_out";

    static CoreOutTag read(const Json::Value& document) {
        const std::string& found = Json::as_text(document);
        if (found != word) {
            throw Refused("not a CoreOutTag: " + found);
        }
        return CoreOutTag{};
    }

    void write(std::string& out) const {
        Json::text(out, word);
    }
};

struct DriverCommandOpen {
    std::int64_t generation = 0;
    std::map<std::string, std::string> headers;
    std::optional<std::int64_t> max_bitrate;
    std::optional<std::int64_t> max_height;
    std::int64_t start_at_ms = 0;
    std::string url;

    /// The word this variant carries in `op`.
    std::string_view word() const {
        return "open";
    }

    static DriverCommandOpen read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "generation", "DriverCommand");
        const Json::Value* raw1 = Json::need(fields, "headers", "DriverCommand");
        const Json::Value* raw2 = Json::at(fields, "max_bitrate");
        const Json::Value* raw3 = Json::at(fields, "max_height");
        const Json::Value* raw4 = Json::need(fields, "start_at_ms", "DriverCommand");
        const Json::Value* raw5 = Json::need(fields, "url", "DriverCommand");
        return DriverCommandOpen{
                Json::as_long(*raw0),
                Json::map<std::string>(*raw1, [](const Json::Value& x0) { return Json::as_text(x0); }),
                raw2 == nullptr ? std::optional<std::int64_t>() : std::optional<std::int64_t>(Json::as_long(*raw2)),
                raw3 == nullptr ? std::optional<std::int64_t>() : std::optional<std::int64_t>(Json::as_long(*raw3)),
                Json::as_long(*raw4),
                Json::as_text(*raw5)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "generation");
        out += std::to_string(this->generation);
        count = Json::key(out, count, "headers");
        out += '{';
        int n0 = 0;
        for (const auto& entry0 : this->headers) {
            n0 = Json::key(out, n0, entry0.first);
            Json::text(out, entry0.second);
        }
        out += '}';
        if (this->max_bitrate.has_value()) {
            count = Json::key(out, count, "max_bitrate");
            out += std::to_string((*this->max_bitrate));
        }
        if (this->max_height.has_value()) {
            count = Json::key(out, count, "max_height");
            out += std::to_string((*this->max_height));
        }
        count = Json::key(out, count, "start_at_ms");
        out += std::to_string(this->start_at_ms);
        count = Json::key(out, count, "url");
        Json::text(out, this->url);
    }
};

struct DriverCommandPlay {
    /// The word this variant carries in `op`.
    std::string_view word() const {
        return "play";
    }

    static DriverCommandPlay read_fields(const Json::Value&) {
        return DriverCommandPlay{};
    }

    void write_fields(std::string&, int&) const {
    }
};

struct DriverCommandPause {
    /// The word this variant carries in `op`.
    std::string_view word() const {
        return "pause";
    }

    static DriverCommandPause read_fields(const Json::Value&) {
        return DriverCommandPause{};
    }

    void write_fields(std::string&, int&) const {
    }
};

struct DriverCommandSeek {
    std::int64_t to_ms = 0;

    /// The word this variant carries in `op`.
    std::string_view word() const {
        return "seek";
    }

    static DriverCommandSeek read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "to_ms", "DriverCommand");
        return DriverCommandSeek{
                Json::as_long(*raw0)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "to_ms");
        out += std::to_string(this->to_ms);
    }
};

struct DriverCommandRate {
    std::int64_t percent = 0;

    /// The word this variant carries in `op`.
    std::string_view word() const {
        return "rate";
    }

    static DriverCommandRate read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "percent", "DriverCommand");
        return DriverCommandRate{
                Json::as_long(*raw0)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "percent");
        out += std::to_string(this->percent);
    }
};

struct DriverCommandVolume {
    std::int64_t percent = 0;

    /// The word this variant carries in `op`.
    std::string_view word() const {
        return "volume";
    }

    static DriverCommandVolume read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "percent", "DriverCommand");
        return DriverCommandVolume{
                Json::as_long(*raw0)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "percent");
        out += std::to_string(this->percent);
    }
};

struct DriverCommandSelectAudio {
    std::string track;

    /// The word this variant carries in `op`.
    std::string_view word() const {
        return "select_audio";
    }

    static DriverCommandSelectAudio read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "track", "DriverCommand");
        return DriverCommandSelectAudio{
                Json::as_text(*raw0)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "track");
        Json::text(out, this->track);
    }
};

struct DriverCommandSelectText {
    std::optional<std::string> track;

    /// The word this variant carries in `op`.
    std::string_view word() const {
        return "select_text";
    }

    static DriverCommandSelectText read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::at(fields, "track");
        return DriverCommandSelectText{
                raw0 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw0))};
    }

    void write_fields(std::string& out, int& count) const {
        if (this->track.has_value()) {
            count = Json::key(out, count, "track");
            Json::text(out, (*this->track));
        }
    }
};

struct DriverCommandClose {
    /// The word this variant carries in `op`.
    std::string_view word() const {
        return "close";
    }

    static DriverCommandClose read_fields(const Json::Value&) {
        return DriverCommandClose{};
    }

    void write_fields(std::string&, int&) const {
    }
};

/// A tag this build does not know.
struct DriverCommandUnknown {
    /// The tag that was read.
    std::string tag;

    std::string_view word() const {
        return this->tag;
    }

    void write_fields(std::string&, int&) const {
    }
};

using DriverCommandBody = std::variant<DriverCommandOpen, DriverCommandPlay, DriverCommandPause, DriverCommandSeek, DriverCommandRate, DriverCommandVolume, DriverCommandSelectAudio, DriverCommandSelectText, DriverCommandClose, DriverCommandUnknown>;

/// A command to the player. Attempts carry their headers here.
struct DriverCommand {
    DriverCommandBody body;

    static DriverCommand read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        return DriverCommand{
                read_body(fields)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "op");
        Json::text(out, std::visit([](const auto& held) -> std::string_view { return held.word(); }, this->body));
        std::visit([&out, &count](const auto& held) { held.write_fields(out, count); }, this->body);
        out += '}';
    }

private:
    static DriverCommandBody read_body(const Json::Value& fields) {
        const std::string& word = Json::as_text(*Json::need(fields, "op", "DriverCommand"));
        if (word == "open") {
            return DriverCommandOpen::read_fields(fields);
        }
        if (word == "play") {
            return DriverCommandPlay::read_fields(fields);
        }
        if (word == "pause") {
            return DriverCommandPause::read_fields(fields);
        }
        if (word == "seek") {
            return DriverCommandSeek::read_fields(fields);
        }
        if (word == "rate") {
            return DriverCommandRate::read_fields(fields);
        }
        if (word == "volume") {
            return DriverCommandVolume::read_fields(fields);
        }
        if (word == "select_audio") {
            return DriverCommandSelectAudio::read_fields(fields);
        }
        if (word == "select_text") {
            return DriverCommandSelectText::read_fields(fields);
        }
        if (word == "close") {
            return DriverCommandClose::read_fields(fields);
        }
        return DriverCommandUnknown{word};
    }
};

/// How an http effect may be handled.
struct HttpClass {
    std::string value;

    static constexpr std::string_view command = "command";
    static constexpr std::string_view cacheable = "cacheable";
    static constexpr std::string_view plain = "plain";

    static HttpClass read(const Json::Value& document) {
        return HttpClass{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// Log levels.
struct LogLevel {
    std::string value;

    static constexpr std::string_view debug = "debug";
    static constexpr std::string_view info = "info";
    static constexpr std::string_view warn = "warn";
    static constexpr std::string_view error = "error";

    static LogLevel read(const Json::Value& document) {
        return LogLevel{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// Text a client can draw with no other knowledge, including a client below the floor.
struct Notice {
    std::optional<std::string> action_address;
    std::optional<std::string> action_label;
    /// Label key, for clients with the strings.
    std::string label;
    /// Plain text in the viewer's language.
    std::string text;

    static Notice read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "action_address");
        const Json::Value* raw1 = Json::at(fields, "action_label");
        const Json::Value* raw2 = Json::need(fields, "label", "Notice");
        const Json::Value* raw3 = Json::need(fields, "text", "Notice");
        return Notice{
                raw0 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw0)),
                raw1 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw1)),
                Json::as_text(*raw2),
                Json::as_text(*raw3)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->action_address.has_value()) {
            count = Json::key(out, count, "action_address");
            Json::text(out, (*this->action_address));
        }
        if (this->action_label.has_value()) {
            count = Json::key(out, count, "action_label");
            Json::text(out, (*this->action_label));
        }
        count = Json::key(out, count, "label");
        Json::text(out, this->label);
        count = Json::key(out, count, "text");
        Json::text(out, this->text);
        out += '}';
    }
};

/// The screens the server builds.
struct ScreenName {
    std::string value;

    static constexpr std::string_view home = "home";
    static constexpr std::string_view films = "films";
    static constexpr std::string_view series = "series";
    static constexpr std::string_view genre = "genre";
    static constexpr std::string_view new_ = "new";
    static constexpr std::string_view my_list = "my-list";
    static constexpr std::string_view kids = "kids";
    static constexpr std::string_view search = "search";
    static constexpr std::string_view notifications = "notifications";
    static constexpr std::string_view activity = "activity";
    static constexpr std::string_view games = "games";
    static constexpr std::string_view clips = "clips";
    static constexpr std::string_view language = "language";
    static constexpr std::string_view mine = "mine";
    static constexpr std::string_view help = "help";
    static constexpr std::string_view settings = "settings";

    static ScreenName read(const Json::Value& document) {
        return ScreenName{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// A semantic target. Each device maps it to one of its own screens.
struct Target {
    std::string value;

    static constexpr std::string_view start = "start";
    static constexpr std::string_view sign_in = "sign_in";
    static constexpr std::string_view profiles = "profiles";
    static constexpr std::string_view home = "home";
    static constexpr std::string_view kids_home = "kids_home";
    static constexpr std::string_view screen = "screen";
    static constexpr std::string_view title = "title";
    static constexpr std::string_view watch = "watch";
    static constexpr std::string_view not_found = "not_found";
    static constexpr std::string_view wait = "wait";
    static constexpr std::string_view upgrade = "upgrade";
    static constexpr std::string_view activate = "activate";

    static Target read(const Json::Value& document) {
        return Target{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// What an address opens.
struct RouteTarget {
    /// The code an approve address carried, as it stood in the address.
    std::optional<std::string> code;
    std::optional<EpisodeRef> episode;
    /// The genre, for the genre screen.
    std::optional<std::string> genre;
    /// A safe address to return to after sign in.
    std::optional<std::string> next;
    std::optional<ScreenName> screen;
    Target target;
    std::optional<TitleKey> title;

    static RouteTarget read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "code");
        const Json::Value* raw1 = Json::at(fields, "episode");
        const Json::Value* raw2 = Json::at(fields, "genre");
        const Json::Value* raw3 = Json::at(fields, "next");
        const Json::Value* raw4 = Json::at(fields, "screen");
        const Json::Value* raw5 = Json::need(fields, "target", "RouteTarget");
        const Json::Value* raw6 = Json::at(fields, "title");
        return RouteTarget{
                raw0 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw0)),
                raw1 == nullptr ? std::optional<EpisodeRef>() : std::optional<EpisodeRef>(EpisodeRef::read(*raw1)),
                raw2 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw2)),
                raw3 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw3)),
                raw4 == nullptr ? std::optional<ScreenName>() : std::optional<ScreenName>(ScreenName::read(*raw4)),
                Target::read(*raw5),
                raw6 == nullptr ? std::optional<TitleKey>() : std::optional<TitleKey>(TitleKey::read(*raw6))};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->code.has_value()) {
            count = Json::key(out, count, "code");
            Json::text(out, (*this->code));
        }
        if (this->episode.has_value()) {
            count = Json::key(out, count, "episode");
            (*this->episode).write(out);
        }
        if (this->genre.has_value()) {
            count = Json::key(out, count, "genre");
            Json::text(out, (*this->genre));
        }
        if (this->next.has_value()) {
            count = Json::key(out, count, "next");
            Json::text(out, (*this->next));
        }
        if (this->screen.has_value()) {
            count = Json::key(out, count, "screen");
            (*this->screen).write(out);
        }
        count = Json::key(out, count, "target");
        this->target.write(out);
        if (this->title.has_value()) {
            count = Json::key(out, count, "title");
            (*this->title).write(out);
        }
        out += '}';
    }
};

/// A page body handed to a screen. It stands in for the answer to an http effect whose body was going to the screen.
struct ScreenBody {
    std::string body;
    /// Id of the http effect this answers.
    std::string request;
    /// True when this copy is past its ttl and was kept for when the network is gone.
    std::optional<bool> stale;

    static ScreenBody read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "body", "ScreenBody");
        const Json::Value* raw1 = Json::need(fields, "request", "ScreenBody");
        const Json::Value* raw2 = Json::at(fields, "stale");
        return ScreenBody{
                Json::as_text(*raw0),
                Json::as_text(*raw1),
                raw2 == nullptr ? std::optional<bool>() : std::optional<bool>(Json::as_bool(*raw2))};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "body");
        Json::text(out, this->body);
        count = Json::key(out, count, "request");
        Json::text(out, this->request);
        if (this->stale.has_value()) {
            count = Json::key(out, count, "stale");
            out += (*this->stale) ? "true" : "false";
        }
        out += '}';
    }
};

/// Secret operations.
struct SecretOp {
    std::string value;

    static constexpr std::string_view get = "get";
    static constexpr std::string_view put = "put";
    static constexpr std::string_view delete_ = "delete";

    static SecretOp read(const Json::Value& document) {
        return SecretOp{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// Store operations. Only the offline layer uses store.
struct StoreOp {
    std::string value;

    static constexpr std::string_view get = "get";
    static constexpr std::string_view put = "put";
    static constexpr std::string_view delete_ = "delete";
    static constexpr std::string_view list = "list";

    static StoreOp read(const Json::Value& document) {
        return StoreOp{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// Timer operations. A clear uses the id of the timer it clears.
struct TimerOp {
    std::string value;

    static constexpr std::string_view set = "set";
    static constexpr std::string_view clear = "clear";

    static TimerOp read(const Json::Value& document) {
        return TimerOp{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// A path, never an address. The host holds the base address.
struct EffectHttp {
    std::optional<std::string> body;
    HttpClass class_;
    std::int64_t deadline_ms = 0;
    std::optional<std::map<std::string, std::string>> headers;
    HttpMethod method;
    std::string path;
    /// Opaque scope of an account and profile, for kept copies.
    std::optional<std::string> scope;
    BodyTarget to;

    /// The word this variant carries in `kind`.
    std::string_view word() const {
        return "http";
    }

    static EffectHttp read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::at(fields, "body");
        const Json::Value* raw1 = Json::need(fields, "class", "Effect");
        const Json::Value* raw2 = Json::need(fields, "deadline_ms", "Effect");
        const Json::Value* raw3 = Json::at(fields, "headers");
        const Json::Value* raw4 = Json::need(fields, "method", "Effect");
        const Json::Value* raw5 = Json::need(fields, "path", "Effect");
        const Json::Value* raw6 = Json::at(fields, "scope");
        const Json::Value* raw7 = Json::need(fields, "to", "Effect");
        return EffectHttp{
                raw0 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw0)),
                HttpClass::read(*raw1),
                Json::as_long(*raw2),
                raw3 == nullptr ? std::optional<std::map<std::string, std::string>>() : std::optional<std::map<std::string, std::string>>(Json::map<std::string>(*raw3, [](const Json::Value& x0) { return Json::as_text(x0); })),
                HttpMethod::read(*raw4),
                Json::as_text(*raw5),
                raw6 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw6)),
                BodyTarget::read(*raw7)};
    }

    void write_fields(std::string& out, int& count) const {
        if (this->body.has_value()) {
            count = Json::key(out, count, "body");
            Json::text(out, (*this->body));
        }
        count = Json::key(out, count, "class");
        this->class_.write(out);
        count = Json::key(out, count, "deadline_ms");
        out += std::to_string(this->deadline_ms);
        if (this->headers.has_value()) {
            count = Json::key(out, count, "headers");
            out += '{';
            int n0 = 0;
            for (const auto& entry0 : (*this->headers)) {
                n0 = Json::key(out, n0, entry0.first);
                Json::text(out, entry0.second);
            }
            out += '}';
        }
        count = Json::key(out, count, "method");
        this->method.write(out);
        count = Json::key(out, count, "path");
        Json::text(out, this->path);
        if (this->scope.has_value()) {
            count = Json::key(out, count, "scope");
            Json::text(out, (*this->scope));
        }
        count = Json::key(out, count, "to");
        this->to.write(out);
    }
};

struct EffectStore {
    std::optional<std::string> key;
    StoreOp op;
    std::optional<std::string> value;

    /// The word this variant carries in `kind`.
    std::string_view word() const {
        return "store";
    }

    static EffectStore read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::at(fields, "key");
        const Json::Value* raw1 = Json::need(fields, "op", "Effect");
        const Json::Value* raw2 = Json::at(fields, "value");
        return EffectStore{
                raw0 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw0)),
                StoreOp::read(*raw1),
                raw2 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw2))};
    }

    void write_fields(std::string& out, int& count) const {
        if (this->key.has_value()) {
            count = Json::key(out, count, "key");
            Json::text(out, (*this->key));
        }
        count = Json::key(out, count, "op");
        this->op.write(out);
        if (this->value.has_value()) {
            count = Json::key(out, count, "value");
            Json::text(out, (*this->value));
        }
    }
};

struct EffectSecret {
    std::string name;
    SecretOp op;
    std::optional<std::string> value;

    /// The word this variant carries in `kind`.
    std::string_view word() const {
        return "secret";
    }

    static EffectSecret read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "name", "Effect");
        const Json::Value* raw1 = Json::need(fields, "op", "Effect");
        const Json::Value* raw2 = Json::at(fields, "value");
        return EffectSecret{
                Json::as_text(*raw0),
                SecretOp::read(*raw1),
                raw2 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw2))};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "name");
        Json::text(out, this->name);
        count = Json::key(out, count, "op");
        this->op.write(out);
        if (this->value.has_value()) {
            count = Json::key(out, count, "value");
            Json::text(out, (*this->value));
        }
    }
};

struct EffectTimer {
    std::optional<std::int64_t> after_ms;
    TimerOp op;

    /// The word this variant carries in `kind`.
    std::string_view word() const {
        return "timer";
    }

    static EffectTimer read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::at(fields, "after_ms");
        const Json::Value* raw1 = Json::need(fields, "op", "Effect");
        return EffectTimer{
                raw0 == nullptr ? std::optional<std::int64_t>() : std::optional<std::int64_t>(Json::as_long(*raw0)),
                TimerOp::read(*raw1)};
    }

    void write_fields(std::string& out, int& count) const {
        if (this->after_ms.has_value()) {
            count = Json::key(out, count, "after_ms");
            out += std::to_string((*this->after_ms));
        }
        count = Json::key(out, count, "op");
        this->op.write(out);
    }
};

struct EffectPlayer {
    DriverCommand command;

    /// The word this variant carries in `kind`.
    std::string_view word() const {
        return "player";
    }

    static EffectPlayer read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "command", "Effect");
        return EffectPlayer{
                DriverCommand::read(*raw0)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "command");
        this->command.write(out);
    }
};

/// A one time notice or a kept page. Lasting state travels in `view`.
struct EffectView {
    std::optional<ScreenBody> body;
    std::optional<Notice> notice;

    /// The word this variant carries in `kind`.
    std::string_view word() const {
        return "view";
    }

    static EffectView read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::at(fields, "body");
        const Json::Value* raw1 = Json::at(fields, "notice");
        return EffectView{
                raw0 == nullptr ? std::optional<ScreenBody>() : std::optional<ScreenBody>(ScreenBody::read(*raw0)),
                raw1 == nullptr ? std::optional<Notice>() : std::optional<Notice>(Notice::read(*raw1))};
    }

    void write_fields(std::string& out, int& count) const {
        if (this->body.has_value()) {
            count = Json::key(out, count, "body");
            (*this->body).write(out);
        }
        if (this->notice.has_value()) {
            count = Json::key(out, count, "notice");
            (*this->notice).write(out);
        }
    }
};

struct EffectOpen {
    RouteTarget target;

    /// The word this variant carries in `kind`.
    std::string_view word() const {
        return "open";
    }

    static EffectOpen read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "target", "Effect");
        return EffectOpen{
                RouteTarget::read(*raw0)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "target");
        this->target.write(out);
    }
};

struct EffectLog {
    std::string code;
    std::optional<std::string> detail;
    LogLevel level;

    /// The word this variant carries in `kind`.
    std::string_view word() const {
        return "log";
    }

    static EffectLog read_fields(const Json::Value& fields) {
        const Json::Value* raw0 = Json::need(fields, "code", "Effect");
        const Json::Value* raw1 = Json::at(fields, "detail");
        const Json::Value* raw2 = Json::need(fields, "level", "Effect");
        return EffectLog{
                Json::as_text(*raw0),
                raw1 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw1)),
                LogLevel::read(*raw2)};
    }

    void write_fields(std::string& out, int& count) const {
        count = Json::key(out, count, "code");
        Json::text(out, this->code);
        if (this->detail.has_value()) {
            count = Json::key(out, count, "detail");
            Json::text(out, (*this->detail));
        }
        count = Json::key(out, count, "level");
        this->level.write(out);
    }
};

/// A tag this build does not know.
struct EffectUnknown {
    /// The tag that was read.
    std::string tag;

    std::string_view word() const {
        return this->tag;
    }

    void write_fields(std::string&, int&) const {
    }
};

using EffectBody = std::variant<EffectHttp, EffectStore, EffectSecret, EffectTimer, EffectPlayer, EffectView, EffectOpen, EffectLog, EffectUnknown>;

/// One effect. The result comes back as an event with this id.
struct Effect {
    std::string id;
    EffectBody body;

    static Effect read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "id", "Effect");
        return Effect{
                Json::as_text(*raw0),
                read_body(fields)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "kind");
        Json::text(out, std::visit([](const auto& held) -> std::string_view { return held.word(); }, this->body));
        count = Json::key(out, count, "id");
        Json::text(out, this->id);
        std::visit([&out, &count](const auto& held) { held.write_fields(out, count); }, this->body);
        out += '}';
    }

private:
    static EffectBody read_body(const Json::Value& fields) {
        const std::string& word = Json::as_text(*Json::need(fields, "kind", "Effect"));
        if (word == "http") {
            return EffectHttp::read_fields(fields);
        }
        if (word == "store") {
            return EffectStore::read_fields(fields);
        }
        if (word == "secret") {
            return EffectSecret::read_fields(fields);
        }
        if (word == "timer") {
            return EffectTimer::read_fields(fields);
        }
        if (word == "player") {
            return EffectPlayer::read_fields(fields);
        }
        if (word == "view") {
            return EffectView::read_fields(fields);
        }
        if (word == "open") {
            return EffectOpen::read_fields(fields);
        }
        if (word == "log") {
            return EffectLog::read_fields(fields);
        }
        return EffectUnknown{word};
    }
};

/// Where a device sign in stands.
struct DeviceSignInState {
    std::string value;

    static constexpr std::string_view idle = "idle";
    static constexpr std::string_view waiting = "waiting";
    static constexpr std::string_view denied = "denied";
    static constexpr std::string_view expired = "expired";
    static constexpr std::string_view off = "off";

    static DeviceSignInState read(const Json::Value& document) {
        return DeviceSignInState{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// Device sign in state for screens. A dead code is left out, so no screen can show one, and the device code has no field here.
struct DeviceSignInView {
    /// Seconds this code lives, counted from when it reached this view.
    std::optional<std::int64_t> expires_in;
    DeviceSignInState state;
    /// The screen shows this even when it shows a QR code.
    std::optional<std::string> user_code;
    std::optional<std::string> verification_uri;
    /// The same address with the code in it, for a QR code.
    std::optional<std::string> verification_uri_complete;

    static DeviceSignInView read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "expires_in");
        const Json::Value* raw1 = Json::need(fields, "state", "DeviceSignInView");
        const Json::Value* raw2 = Json::at(fields, "user_code");
        const Json::Value* raw3 = Json::at(fields, "verification_uri");
        const Json::Value* raw4 = Json::at(fields, "verification_uri_complete");
        return DeviceSignInView{
                raw0 == nullptr ? std::optional<std::int64_t>() : std::optional<std::int64_t>(Json::as_long(*raw0)),
                DeviceSignInState::read(*raw1),
                raw2 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw2)),
                raw3 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw3)),
                raw4 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw4))};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->expires_in.has_value()) {
            count = Json::key(out, count, "expires_in");
            out += std::to_string((*this->expires_in));
        }
        count = Json::key(out, count, "state");
        this->state.write(out);
        if (this->user_code.has_value()) {
            count = Json::key(out, count, "user_code");
            Json::text(out, (*this->user_code));
        }
        if (this->verification_uri.has_value()) {
            count = Json::key(out, count, "verification_uri");
            Json::text(out, (*this->verification_uri));
        }
        if (this->verification_uri_complete.has_value()) {
            count = Json::key(out, count, "verification_uri_complete");
            Json::text(out, (*this->verification_uri_complete));
        }
        out += '}';
    }
};

/// True while the who is watching prompt should show.
struct GateView {
    bool ask = false;

    static GateView read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "ask", "GateView");
        return GateView{
                Json::as_bool(*raw0)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "ask");
        out += this->ask ? "true" : "false";
        out += '}';
    }
};

/// Feature values. With no answer from the server, accounts are required.
struct Features {
    std::optional<bool> accounts_required;
    std::optional<bool> clips;
    std::optional<bool> games;
    std::optional<bool> original_audio;
    /// Tier for visitors. Present only when accounts are not required.
    std::optional<std::string> visitor_tier;

    static Features read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "accounts_required");
        const Json::Value* raw1 = Json::at(fields, "clips");
        const Json::Value* raw2 = Json::at(fields, "games");
        const Json::Value* raw3 = Json::at(fields, "original_audio");
        const Json::Value* raw4 = Json::at(fields, "visitor_tier");
        return Features{
                raw0 == nullptr ? std::optional<bool>() : std::optional<bool>(Json::as_bool(*raw0)),
                raw1 == nullptr ? std::optional<bool>() : std::optional<bool>(Json::as_bool(*raw1)),
                raw2 == nullptr ? std::optional<bool>() : std::optional<bool>(Json::as_bool(*raw2)),
                raw3 == nullptr ? std::optional<bool>() : std::optional<bool>(Json::as_bool(*raw3)),
                raw4 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw4))};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->accounts_required.has_value()) {
            count = Json::key(out, count, "accounts_required");
            out += (*this->accounts_required) ? "true" : "false";
        }
        if (this->clips.has_value()) {
            count = Json::key(out, count, "clips");
            out += (*this->clips) ? "true" : "false";
        }
        if (this->games.has_value()) {
            count = Json::key(out, count, "games");
            out += (*this->games) ? "true" : "false";
        }
        if (this->original_audio.has_value()) {
            count = Json::key(out, count, "original_audio");
            out += (*this->original_audio) ? "true" : "false";
        }
        if (this->visitor_tier.has_value()) {
            count = Json::key(out, count, "visitor_tier");
            Json::text(out, (*this->visitor_tier));
        }
        out += '}';
    }
};

/// Limits served from one place, so no client holds a copy.
struct Limits {
    std::int64_t blocked_title_max = 0;
    std::int64_t blocked_titles_max = 0;
    std::int64_t code_digits = 0;
    std::int64_t code_valid_s = 0;
    std::int64_t command_batch_max = 0;
    std::int64_t handle_max = 0;
    std::int64_t handle_min = 0;
    std::int64_t name_max = 0;
    std::int64_t password_min = 0;
    std::int64_t phone_digits_max = 0;
    std::int64_t phone_digits_min = 0;
    std::int64_t pin_digits = 0;
    std::int64_t profile_name_max = 0;
    std::int64_t profiles_max = 0;
    std::int64_t step_up_s = 0;

    static Limits read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "blocked_title_max", "Limits");
        const Json::Value* raw1 = Json::need(fields, "blocked_titles_max", "Limits");
        const Json::Value* raw2 = Json::need(fields, "code_digits", "Limits");
        const Json::Value* raw3 = Json::need(fields, "code_valid_s", "Limits");
        const Json::Value* raw4 = Json::need(fields, "command_batch_max", "Limits");
        const Json::Value* raw5 = Json::need(fields, "handle_max", "Limits");
        const Json::Value* raw6 = Json::need(fields, "handle_min", "Limits");
        const Json::Value* raw7 = Json::need(fields, "name_max", "Limits");
        const Json::Value* raw8 = Json::need(fields, "password_min", "Limits");
        const Json::Value* raw9 = Json::need(fields, "phone_digits_max", "Limits");
        const Json::Value* raw10 = Json::need(fields, "phone_digits_min", "Limits");
        const Json::Value* raw11 = Json::need(fields, "pin_digits", "Limits");
        const Json::Value* raw12 = Json::need(fields, "profile_name_max", "Limits");
        const Json::Value* raw13 = Json::need(fields, "profiles_max", "Limits");
        const Json::Value* raw14 = Json::need(fields, "step_up_s", "Limits");
        return Limits{
                Json::as_long(*raw0),
                Json::as_long(*raw1),
                Json::as_long(*raw2),
                Json::as_long(*raw3),
                Json::as_long(*raw4),
                Json::as_long(*raw5),
                Json::as_long(*raw6),
                Json::as_long(*raw7),
                Json::as_long(*raw8),
                Json::as_long(*raw9),
                Json::as_long(*raw10),
                Json::as_long(*raw11),
                Json::as_long(*raw12),
                Json::as_long(*raw13),
                Json::as_long(*raw14)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "blocked_title_max");
        out += std::to_string(this->blocked_title_max);
        count = Json::key(out, count, "blocked_titles_max");
        out += std::to_string(this->blocked_titles_max);
        count = Json::key(out, count, "code_digits");
        out += std::to_string(this->code_digits);
        count = Json::key(out, count, "code_valid_s");
        out += std::to_string(this->code_valid_s);
        count = Json::key(out, count, "command_batch_max");
        out += std::to_string(this->command_batch_max);
        count = Json::key(out, count, "handle_max");
        out += std::to_string(this->handle_max);
        count = Json::key(out, count, "handle_min");
        out += std::to_string(this->handle_min);
        count = Json::key(out, count, "name_max");
        out += std::to_string(this->name_max);
        count = Json::key(out, count, "password_min");
        out += std::to_string(this->password_min);
        count = Json::key(out, count, "phone_digits_max");
        out += std::to_string(this->phone_digits_max);
        count = Json::key(out, count, "phone_digits_min");
        out += std::to_string(this->phone_digits_min);
        count = Json::key(out, count, "pin_digits");
        out += std::to_string(this->pin_digits);
        count = Json::key(out, count, "profile_name_max");
        out += std::to_string(this->profile_name_max);
        count = Json::key(out, count, "profiles_max");
        out += std::to_string(this->profiles_max);
        count = Json::key(out, count, "step_up_s");
        out += std::to_string(this->step_up_s);
        out += '}';
    }
};

/// One row of the moved address table.
struct MovedAddress {
    std::string from;
    std::string to;

    static MovedAddress read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "from", "MovedAddress");
        const Json::Value* raw1 = Json::need(fields, "to", "MovedAddress");
        return MovedAddress{
                Json::as_text(*raw0),
                Json::as_text(*raw1)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "from");
        Json::text(out, this->from);
        count = Json::key(out, count, "to");
        Json::text(out, this->to);
        out += '}';
    }
};

/// A staged rollout. The bucket comes from the household, so a home moves together.
struct Rollout {
    std::string name;
    std::int64_t percent = 0;

    static Rollout read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "name", "Rollout");
        const Json::Value* raw1 = Json::need(fields, "percent", "Rollout");
        return Rollout{
                Json::as_text(*raw0),
                Json::as_long(*raw1)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "name");
        Json::text(out, this->name);
        count = Json::key(out, count, "percent");
        out += std::to_string(this->percent);
        out += '}';
    }
};

/// The wire dates a server answers. Never compared for equality.
struct VersionRange {
    WireDate max;
    WireDate min;

    static VersionRange read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "max", "VersionRange");
        const Json::Value* raw1 = Json::need(fields, "min", "VersionRange");
        return VersionRange{
                WireDate::read(*raw0),
                WireDate::read(*raw1)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "max");
        this->max.write(out);
        count = Json::key(out, count, "min");
        this->min.write(out);
        out += '}';
    }
};

/// First answer a client reads.
struct Hello {
    BrandValues brand;
    std::vector<ClientFloor> client_min;
    /// Wire dates this server answers.
    VersionRange contract;
    Features features;
    /// Builds refused by name.
    std::optional<std::vector<std::string>> killed_builds;
    Limits limits;
    std::optional<std::vector<MovedAddress>> moved;
    /// False stops playback everywhere with a notice.
    bool playback = false;
    std::optional<std::vector<Rollout>> rollouts;
    std::vector<ClassValue> start_deadline_ms;
    /// Seconds this answer stays good.
    std::int64_t ttl_s = 0;

    static Hello read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "brand", "Hello");
        const Json::Value* raw1 = Json::need(fields, "client_min", "Hello");
        const Json::Value* raw2 = Json::need(fields, "contract", "Hello");
        const Json::Value* raw3 = Json::need(fields, "features", "Hello");
        const Json::Value* raw4 = Json::at(fields, "killed_builds");
        const Json::Value* raw5 = Json::need(fields, "limits", "Hello");
        const Json::Value* raw6 = Json::at(fields, "moved");
        const Json::Value* raw7 = Json::need(fields, "playback", "Hello");
        const Json::Value* raw8 = Json::at(fields, "rollouts");
        const Json::Value* raw9 = Json::need(fields, "start_deadline_ms", "Hello");
        const Json::Value* raw10 = Json::need(fields, "ttl_s", "Hello");
        return Hello{
                BrandValues::read(*raw0),
                Json::list<ClientFloor>(*raw1, [](const Json::Value& x0) { return ClientFloor::read(x0); }),
                VersionRange::read(*raw2),
                Features::read(*raw3),
                raw4 == nullptr ? std::optional<std::vector<std::string>>() : std::optional<std::vector<std::string>>(Json::list<std::string>(*raw4, [](const Json::Value& x0) { return Json::as_text(x0); })),
                Limits::read(*raw5),
                raw6 == nullptr ? std::optional<std::vector<MovedAddress>>() : std::optional<std::vector<MovedAddress>>(Json::list<MovedAddress>(*raw6, [](const Json::Value& x0) { return MovedAddress::read(x0); })),
                Json::as_bool(*raw7),
                raw8 == nullptr ? std::optional<std::vector<Rollout>>() : std::optional<std::vector<Rollout>>(Json::list<Rollout>(*raw8, [](const Json::Value& x0) { return Rollout::read(x0); })),
                Json::list<ClassValue>(*raw9, [](const Json::Value& x0) { return ClassValue::read(x0); }),
                Json::as_long(*raw10)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "brand");
        this->brand.write(out);
        count = Json::key(out, count, "client_min");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->client_min.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->client_min[index0].write(out);
        }
        out += ']';
        count = Json::key(out, count, "contract");
        this->contract.write(out);
        count = Json::key(out, count, "features");
        this->features.write(out);
        if (this->killed_builds.has_value()) {
            count = Json::key(out, count, "killed_builds");
            out += '[';
            for (std::size_t index0 = 0; index0 < (*this->killed_builds).size(); index0++) {
                if (index0 > 0) {
                    out += ',';
                }
                Json::text(out, (*this->killed_builds)[index0]);
            }
            out += ']';
        }
        count = Json::key(out, count, "limits");
        this->limits.write(out);
        if (this->moved.has_value()) {
            count = Json::key(out, count, "moved");
            out += '[';
            for (std::size_t index0 = 0; index0 < (*this->moved).size(); index0++) {
                if (index0 > 0) {
                    out += ',';
                }
                (*this->moved)[index0].write(out);
            }
            out += ']';
        }
        count = Json::key(out, count, "playback");
        out += this->playback ? "true" : "false";
        if (this->rollouts.has_value()) {
            count = Json::key(out, count, "rollouts");
            out += '[';
            for (std::size_t index0 = 0; index0 < (*this->rollouts).size(); index0++) {
                if (index0 > 0) {
                    out += ',';
                }
                (*this->rollouts)[index0].write(out);
            }
            out += ']';
        }
        count = Json::key(out, count, "start_deadline_ms");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->start_deadline_ms.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->start_deadline_ms[index0].write(out);
        }
        out += ']';
        count = Json::key(out, count, "ttl_s");
        out += std::to_string(this->ttl_s);
        out += '}';
    }
};

/// Where the play session stands.
struct PlayState {
    std::string value;

    static constexpr std::string_view resolving = "resolving";
    static constexpr std::string_view starting = "starting";
    static constexpr std::string_view playing = "playing";
    static constexpr std::string_view paused = "paused";
    static constexpr std::string_view stalled = "stalled";
    static constexpr std::string_view switching = "switching";
    static constexpr std::string_view ended = "ended";
    static constexpr std::string_view failed = "failed";

    static PlayState read(const Json::Value& document) {
        return PlayState{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// Windows of time inside a video.
struct WindowKind {
    std::string value;

    static constexpr std::string_view recap = "recap";
    static constexpr std::string_view intro = "intro";
    static constexpr std::string_view credits = "credits";
    static constexpr std::string_view age_card = "age_card";
    static constexpr std::string_view cue = "cue";

    static WindowKind read(const Json::Value& document) {
        return WindowKind{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// A span of a video the host compares to player time. Nothing ticks across the boundary.
struct TimeWindow {
    std::int64_t end_ms = 0;
    WindowKind kind;
    /// Where a skip button moves to.
    std::optional<std::int64_t> seek_to_ms;
    std::int64_t start_ms = 0;
    std::optional<std::string> text;

    static TimeWindow read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "end_ms", "TimeWindow");
        const Json::Value* raw1 = Json::need(fields, "kind", "TimeWindow");
        const Json::Value* raw2 = Json::at(fields, "seek_to_ms");
        const Json::Value* raw3 = Json::need(fields, "start_ms", "TimeWindow");
        const Json::Value* raw4 = Json::at(fields, "text");
        return TimeWindow{
                Json::as_long(*raw0),
                WindowKind::read(*raw1),
                raw2 == nullptr ? std::optional<std::int64_t>() : std::optional<std::int64_t>(Json::as_long(*raw2)),
                Json::as_long(*raw3),
                raw4 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw4))};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "end_ms");
        out += std::to_string(this->end_ms);
        count = Json::key(out, count, "kind");
        this->kind.write(out);
        if (this->seek_to_ms.has_value()) {
            count = Json::key(out, count, "seek_to_ms");
            out += std::to_string((*this->seek_to_ms));
        }
        count = Json::key(out, count, "start_ms");
        out += std::to_string(this->start_ms);
        if (this->text.has_value()) {
            count = Json::key(out, count, "text");
            Json::text(out, (*this->text));
        }
        out += '}';
    }
};

/// Play session state for screens.
struct PlayView {
    std::optional<std::string> attempt;
    std::optional<EpisodeRef> episode;
    /// Absent on the last episode, so no screen offers one that does not exist.
    std::optional<EpisodeRef> next_episode;
    PlayState state;
    /// Several episodes started by themselves with nobody touching anything, so the next one waits until the viewer says they are still there.
    std::optional<bool> still_asking;
    TitleKey title;
    std::optional<std::vector<TimeWindow>> windows;

    static PlayView read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "attempt");
        const Json::Value* raw1 = Json::at(fields, "episode");
        const Json::Value* raw2 = Json::at(fields, "next_episode");
        const Json::Value* raw3 = Json::need(fields, "state", "PlayView");
        const Json::Value* raw4 = Json::at(fields, "still_asking");
        const Json::Value* raw5 = Json::need(fields, "title", "PlayView");
        const Json::Value* raw6 = Json::at(fields, "windows");
        return PlayView{
                raw0 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw0)),
                raw1 == nullptr ? std::optional<EpisodeRef>() : std::optional<EpisodeRef>(EpisodeRef::read(*raw1)),
                raw2 == nullptr ? std::optional<EpisodeRef>() : std::optional<EpisodeRef>(EpisodeRef::read(*raw2)),
                PlayState::read(*raw3),
                raw4 == nullptr ? std::optional<bool>() : std::optional<bool>(Json::as_bool(*raw4)),
                TitleKey::read(*raw5),
                raw6 == nullptr ? std::optional<std::vector<TimeWindow>>() : std::optional<std::vector<TimeWindow>>(Json::list<TimeWindow>(*raw6, [](const Json::Value& x0) { return TimeWindow::read(x0); }))};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->attempt.has_value()) {
            count = Json::key(out, count, "attempt");
            Json::text(out, (*this->attempt));
        }
        if (this->episode.has_value()) {
            count = Json::key(out, count, "episode");
            (*this->episode).write(out);
        }
        if (this->next_episode.has_value()) {
            count = Json::key(out, count, "next_episode");
            (*this->next_episode).write(out);
        }
        count = Json::key(out, count, "state");
        this->state.write(out);
        if (this->still_asking.has_value()) {
            count = Json::key(out, count, "still_asking");
            out += (*this->still_asking) ? "true" : "false";
        }
        count = Json::key(out, count, "title");
        this->title.write(out);
        if (this->windows.has_value()) {
            count = Json::key(out, count, "windows");
            out += '[';
            for (std::size_t index0 = 0; index0 < (*this->windows).size(); index0++) {
                if (index0 > 0) {
                    out += ',';
                }
                (*this->windows)[index0].write(out);
            }
            out += ']';
        }
        out += '}';
    }
};

struct ProfileList {
    std::vector<Profile> profiles;

    static ProfileList read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "profiles", "ProfileList");
        return ProfileList{
                Json::list<Profile>(*raw0, [](const Json::Value& x0) { return Profile::read(x0); })};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "profiles");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->profiles.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->profiles[index0].write(out);
        }
        out += ']';
        out += '}';
    }
};

/// Why a session ended.
struct EndReason {
    std::string value;

    static constexpr std::string_view signed_out = "signed_out";
    static constexpr std::string_view everywhere = "everywhere";
    static constexpr std::string_view refused = "refused";
    static constexpr std::string_view elsewhere = "elsewhere";

    static EndReason read(const Json::Value& document) {
        return EndReason{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// Where sign in stands.
struct SessionState {
    std::string value;

    static constexpr std::string_view starting = "starting";
    static constexpr std::string_view signed_out = "signed_out";
    static constexpr std::string_view signed_in = "signed_in";

    static SessionState read(const Json::Value& document) {
        return SessionState{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// Sign in state for screens. Tokens never appear here.
struct SessionView {
    std::optional<std::string> email;
    std::optional<EndReason> ended;
    std::optional<std::int64_t> expires_at_ms;
    std::optional<std::string> profile;
    /// Opaque scope of this account and profile, the same value http effects carry.
    std::optional<std::string> scope;
    SessionState state;
    std::optional<std::string> user_id;

    static SessionView read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "email");
        const Json::Value* raw1 = Json::at(fields, "ended");
        const Json::Value* raw2 = Json::at(fields, "expires_at_ms");
        const Json::Value* raw3 = Json::at(fields, "profile");
        const Json::Value* raw4 = Json::at(fields, "scope");
        const Json::Value* raw5 = Json::need(fields, "state", "SessionView");
        const Json::Value* raw6 = Json::at(fields, "user_id");
        return SessionView{
                raw0 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw0)),
                raw1 == nullptr ? std::optional<EndReason>() : std::optional<EndReason>(EndReason::read(*raw1)),
                raw2 == nullptr ? std::optional<std::int64_t>() : std::optional<std::int64_t>(Json::as_long(*raw2)),
                raw3 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw3)),
                raw4 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw4)),
                SessionState::read(*raw5),
                raw6 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw6))};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->email.has_value()) {
            count = Json::key(out, count, "email");
            Json::text(out, (*this->email));
        }
        if (this->ended.has_value()) {
            count = Json::key(out, count, "ended");
            (*this->ended).write(out);
        }
        if (this->expires_at_ms.has_value()) {
            count = Json::key(out, count, "expires_at_ms");
            out += std::to_string((*this->expires_at_ms));
        }
        if (this->profile.has_value()) {
            count = Json::key(out, count, "profile");
            Json::text(out, (*this->profile));
        }
        if (this->scope.has_value()) {
            count = Json::key(out, count, "scope");
            Json::text(out, (*this->scope));
        }
        count = Json::key(out, count, "state");
        this->state.write(out);
        if (this->user_id.has_value()) {
            count = Json::key(out, count, "user_id");
            Json::text(out, (*this->user_id));
        }
        out += '}';
    }
};

/// Changed screen state. A part that is absent did not change.
struct View {
    std::optional<DeviceSignInView> device_sign_in;
    std::optional<GateView> gate;
    /// The last hello read, so a screen takes limits and features from one place.
    std::optional<Hello> hello;
    std::optional<PlayView> play;
    /// The account's profiles, for the picker. No PIN is ever held here.
    std::optional<ProfileList> profiles;
    std::optional<RouteTarget> route;
    std::optional<SessionView> session;

    static View read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "device_sign_in");
        const Json::Value* raw1 = Json::at(fields, "gate");
        const Json::Value* raw2 = Json::at(fields, "hello");
        const Json::Value* raw3 = Json::at(fields, "play");
        const Json::Value* raw4 = Json::at(fields, "profiles");
        const Json::Value* raw5 = Json::at(fields, "route");
        const Json::Value* raw6 = Json::at(fields, "session");
        return View{
                raw0 == nullptr ? std::optional<DeviceSignInView>() : std::optional<DeviceSignInView>(DeviceSignInView::read(*raw0)),
                raw1 == nullptr ? std::optional<GateView>() : std::optional<GateView>(GateView::read(*raw1)),
                raw2 == nullptr ? std::optional<Hello>() : std::optional<Hello>(Hello::read(*raw2)),
                raw3 == nullptr ? std::optional<PlayView>() : std::optional<PlayView>(PlayView::read(*raw3)),
                raw4 == nullptr ? std::optional<ProfileList>() : std::optional<ProfileList>(ProfileList::read(*raw4)),
                raw5 == nullptr ? std::optional<RouteTarget>() : std::optional<RouteTarget>(RouteTarget::read(*raw5)),
                raw6 == nullptr ? std::optional<SessionView>() : std::optional<SessionView>(SessionView::read(*raw6))};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->device_sign_in.has_value()) {
            count = Json::key(out, count, "device_sign_in");
            (*this->device_sign_in).write(out);
        }
        if (this->gate.has_value()) {
            count = Json::key(out, count, "gate");
            (*this->gate).write(out);
        }
        if (this->hello.has_value()) {
            count = Json::key(out, count, "hello");
            (*this->hello).write(out);
        }
        if (this->play.has_value()) {
            count = Json::key(out, count, "play");
            (*this->play).write(out);
        }
        if (this->profiles.has_value()) {
            count = Json::key(out, count, "profiles");
            (*this->profiles).write(out);
        }
        if (this->route.has_value()) {
            count = Json::key(out, count, "route");
            (*this->route).write(out);
        }
        if (this->session.has_value()) {
            count = Json::key(out, count, "session");
            (*this->session).write(out);
        }
        out += '}';
    }
};

/// One step of output.
struct CoreOut {
    WireDate contract;
    std::vector<Effect> effects;
    /// Monotonic time at which the host steps the core again. Absent means no wake.
    std::optional<std::int64_t> next_wake_ms;
    CoreOutTag type;
    /// Ids of in events of a type this core does not know.
    std::optional<std::vector<std::string>> unsupported;
    std::optional<View> view;

    static CoreOut read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "contract", "CoreOut");
        const Json::Value* raw1 = Json::need(fields, "effects", "CoreOut");
        const Json::Value* raw2 = Json::at(fields, "next_wake_ms");
        const Json::Value* raw3 = Json::need(fields, "type", "CoreOut");
        const Json::Value* raw4 = Json::at(fields, "unsupported");
        const Json::Value* raw5 = Json::at(fields, "view");
        return CoreOut{
                WireDate::read(*raw0),
                Json::list<Effect>(*raw1, [](const Json::Value& x0) { return Effect::read(x0); }),
                raw2 == nullptr ? std::optional<std::int64_t>() : std::optional<std::int64_t>(Json::as_long(*raw2)),
                CoreOutTag::read(*raw3),
                raw4 == nullptr ? std::optional<std::vector<std::string>>() : std::optional<std::vector<std::string>>(Json::list<std::string>(*raw4, [](const Json::Value& x0) { return Json::as_text(x0); })),
                raw5 == nullptr ? std::optional<View>() : std::optional<View>(View::read(*raw5))};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "contract");
        this->contract.write(out);
        count = Json::key(out, count, "effects");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->effects.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->effects[index0].write(out);
        }
        out += ']';
        if (this->next_wake_ms.has_value()) {
            count = Json::key(out, count, "next_wake_ms");
            out += std::to_string((*this->next_wake_ms));
        }
        count = Json::key(out, count, "type");
        this->type.write(out);
        if (this->unsupported.has_value()) {
            count = Json::key(out, count, "unsupported");
            out += '[';
            for (std::size_t index0 = 0; index0 < (*this->unsupported).size(); index0++) {
                if (index0 > 0) {
                    out += ',';
                }
                Json::text(out, (*this->unsupported)[index0]);
            }
            out += ']';
        }
        if (this->view.has_value()) {
            count = Json::key(out, count, "view");
            (*this->view).write(out);
        }
        out += '}';
    }
};

/// One cue.
struct Cue {
    std::int64_t end_ms = 0;
    std::int64_t start_ms = 0;
    std::string text;

    static Cue read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "end_ms", "Cue");
        const Json::Value* raw1 = Json::need(fields, "start_ms", "Cue");
        const Json::Value* raw2 = Json::need(fields, "text", "Cue");
        return Cue{
                Json::as_long(*raw0),
                Json::as_long(*raw1),
                Json::as_text(*raw2)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "end_ms");
        out += std::to_string(this->end_ms);
        count = Json::key(out, count, "start_ms");
        out += std::to_string(this->start_ms);
        count = Json::key(out, count, "text");
        Json::text(out, this->text);
        out += '}';
    }
};

/// An opaque position in a list. Clients hand it back and never read it.
using Cursor = std::string;

/// True when the server wants the whole card.
struct DeviceCardAnswer {
    bool send_card = false;

    static DeviceCardAnswer read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "send_card", "DeviceCardAnswer");
        return DeviceCardAnswer{
                Json::as_bool(*raw0)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "send_card");
        out += this->send_card ? "true" : "false";
        out += '}';
    }
};

/// Sent at each start. The card travels only when the server asks or the app or OS changed.
struct DeviceCardRequest {
    std::optional<CapabilityCard> card;
    std::string card_hash;
    std::string device_key;

    static DeviceCardRequest read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "card");
        const Json::Value* raw1 = Json::need(fields, "card_hash", "DeviceCardRequest");
        const Json::Value* raw2 = Json::need(fields, "device_key", "DeviceCardRequest");
        return DeviceCardRequest{
                raw0 == nullptr ? std::optional<CapabilityCard>() : std::optional<CapabilityCard>(CapabilityCard::read(*raw0)),
                Json::as_text(*raw1),
                Json::as_text(*raw2)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->card.has_value()) {
            count = Json::key(out, count, "card");
            (*this->card).write(out);
        }
        count = Json::key(out, count, "card_hash");
        Json::text(out, this->card_hash);
        count = Json::key(out, count, "device_key");
        Json::text(out, this->device_key);
        out += '}';
    }
};

/// Device sign in. The screen shows `user_code` even when it shows a QR code.
struct DeviceCode {
    std::string device_code;
    std::int64_t expires_in = 0;
    std::int64_t interval = 0;
    std::string user_code;
    std::string verification_uri;
    std::string verification_uri_complete;

    static DeviceCode read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "device_code", "DeviceCode");
        const Json::Value* raw1 = Json::need(fields, "expires_in", "DeviceCode");
        const Json::Value* raw2 = Json::need(fields, "interval", "DeviceCode");
        const Json::Value* raw3 = Json::need(fields, "user_code", "DeviceCode");
        const Json::Value* raw4 = Json::need(fields, "verification_uri", "DeviceCode");
        const Json::Value* raw5 = Json::need(fields, "verification_uri_complete", "DeviceCode");
        return DeviceCode{
                Json::as_text(*raw0),
                Json::as_long(*raw1),
                Json::as_long(*raw2),
                Json::as_text(*raw3),
                Json::as_text(*raw4),
                Json::as_text(*raw5)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "device_code");
        Json::text(out, this->device_code);
        count = Json::key(out, count, "expires_in");
        out += std::to_string(this->expires_in);
        count = Json::key(out, count, "interval");
        out += std::to_string(this->interval);
        count = Json::key(out, count, "user_code");
        Json::text(out, this->user_code);
        count = Json::key(out, count, "verification_uri");
        Json::text(out, this->verification_uri);
        count = Json::key(out, count, "verification_uri_complete");
        Json::text(out, this->verification_uri_complete);
        out += '}';
    }
};

struct DeviceCodeRequest {
    std::string device_name;

    static DeviceCodeRequest read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "device_name", "DeviceCodeRequest");
        return DeviceCodeRequest{
                Json::as_text(*raw0)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "device_name");
        Json::text(out, this->device_name);
        out += '}';
    }
};

struct DeviceDecision {
    std::string user_code;

    static DeviceDecision read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "user_code", "DeviceDecision");
        return DeviceDecision{
                Json::as_text(*raw0)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "user_code");
        Json::text(out, this->user_code);
        out += '}';
    }
};

struct DeviceList {
    std::vector<DeviceSession> devices;

    static DeviceList read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "devices", "DeviceList");
        return DeviceList{
                Json::list<DeviceSession>(*raw0, [](const Json::Value& x0) { return DeviceSession::read(x0); })};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "devices");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->devices.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->devices[index0].write(out);
        }
        out += ']';
        out += '}';
    }
};

/// Answers while a device waits for approval.
struct DevicePoll {
    std::string value;

    static constexpr std::string_view authorization_pending = "authorization_pending";
    static constexpr std::string_view slow_down = "slow_down";
    static constexpr std::string_view access_denied = "access_denied";
    static constexpr std::string_view expired_token = "expired_token";

    static DevicePoll read(const Json::Value& document) {
        return DevicePoll{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// Opaque tokens. `expires_in` is seconds, so no device clock decides anything.
struct SignedIn {
    std::string access_token;
    std::string email;
    std::int64_t expires_in = 0;
    std::string refresh_token;
    std::string user_id;

    static SignedIn read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "access_token", "SignedIn");
        const Json::Value* raw1 = Json::need(fields, "email", "SignedIn");
        const Json::Value* raw2 = Json::need(fields, "expires_in", "SignedIn");
        const Json::Value* raw3 = Json::need(fields, "refresh_token", "SignedIn");
        const Json::Value* raw4 = Json::need(fields, "user_id", "SignedIn");
        return SignedIn{
                Json::as_text(*raw0),
                Json::as_text(*raw1),
                Json::as_long(*raw2),
                Json::as_text(*raw3),
                Json::as_text(*raw4)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "access_token");
        Json::text(out, this->access_token);
        count = Json::key(out, count, "email");
        Json::text(out, this->email);
        count = Json::key(out, count, "expires_in");
        out += std::to_string(this->expires_in);
        count = Json::key(out, count, "refresh_token");
        Json::text(out, this->refresh_token);
        count = Json::key(out, count, "user_id");
        Json::text(out, this->user_id);
        out += '}';
    }
};

/// Either tokens or a word saying to wait or stop.
struct DeviceTokenAnswer {
    std::optional<DevicePoll> poll;
    std::optional<SignedIn> signed_in;

    static DeviceTokenAnswer read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "poll");
        const Json::Value* raw1 = Json::at(fields, "signed_in");
        return DeviceTokenAnswer{
                raw0 == nullptr ? std::optional<DevicePoll>() : std::optional<DevicePoll>(DevicePoll::read(*raw0)),
                raw1 == nullptr ? std::optional<SignedIn>() : std::optional<SignedIn>(SignedIn::read(*raw1))};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->poll.has_value()) {
            count = Json::key(out, count, "poll");
            (*this->poll).write(out);
        }
        if (this->signed_in.has_value()) {
            count = Json::key(out, count, "signed_in");
            (*this->signed_in).write(out);
        }
        out += '}';
    }
};

struct DeviceTokenRequest {
    std::string device_code;

    static DeviceTokenRequest read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "device_code", "DeviceTokenRequest");
        return DeviceTokenRequest{
                Json::as_text(*raw0)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "device_code");
        Json::text(out, this->device_code);
        out += '}';
    }
};

/// Answer with no data.
struct Done {
    bool done = false;

    static Done read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "done", "Done");
        return Done{
                Json::as_bool(*raw0)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "done");
        out += this->done ? "true" : "false";
        out += '}';
    }
};

struct EmailChange {
    std::string email;

    static EmailChange read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "email", "EmailChange");
        return EmailChange{
                Json::as_text(*raw0)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "email");
        Json::text(out, this->email);
        out += '}';
    }
};

/// What an image is for.
struct ImageRole {
    std::string value;

    static constexpr std::string_view poster = "poster";
    static constexpr std::string_view backdrop = "backdrop";
    static constexpr std::string_view logo = "logo";
    static constexpr std::string_view still = "still";
    static constexpr std::string_view avatar = "avatar";
    static constexpr std::string_view art = "art";

    static ImageRole read(const Json::Value& document) {
        return ImageRole{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// One image address. Images go straight from the host to the viewer.
struct Image {
    ImageRole role;
    std::string url;
    std::optional<std::int64_t> width;

    static Image read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "role", "Image");
        const Json::Value* raw1 = Json::need(fields, "url", "Image");
        const Json::Value* raw2 = Json::at(fields, "width");
        return Image{
                ImageRole::read(*raw0),
                Json::as_text(*raw1),
                raw2 == nullptr ? std::optional<std::int64_t>() : std::optional<std::int64_t>(Json::as_long(*raw2))};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "role");
        this->role.write(out);
        count = Json::key(out, count, "url");
        Json::text(out, this->url);
        if (this->width.has_value()) {
            count = Json::key(out, count, "width");
            out += std::to_string((*this->width));
        }
        out += '}';
    }
};

/// One episode.
struct Episode {
    std::optional<std::vector<Image>> images;
    std::string name;
    std::int64_t number = 0;
    std::optional<std::string> overview;
    /// Whole percent watched.
    std::optional<std::int64_t> percent;
    std::optional<std::int64_t> runtime_min;

    static Episode read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "images");
        const Json::Value* raw1 = Json::need(fields, "name", "Episode");
        const Json::Value* raw2 = Json::need(fields, "number", "Episode");
        const Json::Value* raw3 = Json::at(fields, "overview");
        const Json::Value* raw4 = Json::at(fields, "percent");
        const Json::Value* raw5 = Json::at(fields, "runtime_min");
        return Episode{
                raw0 == nullptr ? std::optional<std::vector<Image>>() : std::optional<std::vector<Image>>(Json::list<Image>(*raw0, [](const Json::Value& x0) { return Image::read(x0); })),
                Json::as_text(*raw1),
                Json::as_long(*raw2),
                raw3 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw3)),
                raw4 == nullptr ? std::optional<std::int64_t>() : std::optional<std::int64_t>(Json::as_long(*raw4)),
                raw5 == nullptr ? std::optional<std::int64_t>() : std::optional<std::int64_t>(Json::as_long(*raw5))};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->images.has_value()) {
            count = Json::key(out, count, "images");
            out += '[';
            for (std::size_t index0 = 0; index0 < (*this->images).size(); index0++) {
                if (index0 > 0) {
                    out += ',';
                }
                (*this->images)[index0].write(out);
            }
            out += ']';
        }
        count = Json::key(out, count, "name");
        Json::text(out, this->name);
        count = Json::key(out, count, "number");
        out += std::to_string(this->number);
        if (this->overview.has_value()) {
            count = Json::key(out, count, "overview");
            Json::text(out, (*this->overview));
        }
        if (this->percent.has_value()) {
            count = Json::key(out, count, "percent");
            out += std::to_string((*this->percent));
        }
        if (this->runtime_min.has_value()) {
            count = Json::key(out, count, "runtime_min");
            out += std::to_string((*this->runtime_min));
        }
        out += '}';
    }
};

/// Stable error code. A code this build does not know reads as `Unknown`.
struct ErrorCode {
    std::string value;

    static constexpr std::string_view bad_request = "bad_request";
    static constexpr std::string_view unauthorized = "unauthorized";
    static constexpr std::string_view session_ended = "session_ended";
    static constexpr std::string_view profile_required = "profile_required";
    static constexpr std::string_view pin_required = "pin_required";
    static constexpr std::string_view step_up_required = "step_up_required";
    static constexpr std::string_view refused = "refused";
    static constexpr std::string_view not_found = "not_found";
    static constexpr std::string_view no_longer_available = "no_longer_available";
    static constexpr std::string_view conflict = "conflict";
    static constexpr std::string_view rate_limited = "rate_limited";
    static constexpr std::string_view overloaded = "overloaded";
    static constexpr std::string_view deadline_exceeded = "deadline_exceeded";
    static constexpr std::string_view upgrade_required = "upgrade_required";
    static constexpr std::string_view unsupported = "unsupported";
    static constexpr std::string_view no_source = "no_source";
    static constexpr std::string_view clock_wrong = "clock_wrong";
    static constexpr std::string_view internal = "internal";

    static ErrorCode read(const Json::Value& document) {
        return ErrorCode{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// Body of every error answer.
struct ErrorBody {
    /// The build that answered.
    std::string build;
    ErrorCode code;
    /// Label key for this error.
    std::string label;
    std::optional<Notice> notice;
    std::optional<std::int64_t> retry_after_s;

    static ErrorBody read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "build", "ErrorBody");
        const Json::Value* raw1 = Json::need(fields, "code", "ErrorBody");
        const Json::Value* raw2 = Json::need(fields, "label", "ErrorBody");
        const Json::Value* raw3 = Json::at(fields, "notice");
        const Json::Value* raw4 = Json::at(fields, "retry_after_s");
        return ErrorBody{
                Json::as_text(*raw0),
                ErrorCode::read(*raw1),
                Json::as_text(*raw2),
                raw3 == nullptr ? std::optional<Notice>() : std::optional<Notice>(Notice::read(*raw3)),
                raw4 == nullptr ? std::optional<std::int64_t>() : std::optional<std::int64_t>(Json::as_long(*raw4))};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "build");
        Json::text(out, this->build);
        count = Json::key(out, count, "code");
        this->code.write(out);
        count = Json::key(out, count, "label");
        Json::text(out, this->label);
        if (this->notice.has_value()) {
            count = Json::key(out, count, "notice");
            (*this->notice).write(out);
        }
        if (this->retry_after_s.has_value()) {
            count = Json::key(out, count, "retry_after_s");
            out += std::to_string((*this->retry_after_s));
        }
        out += '}';
    }
};

/// How a search for sources ended.
struct FindOutcome {
    std::string value;

    static constexpr std::string_view found = "found";
    static constexpr std::string_view partial = "partial";
    static constexpr std::string_view not_found = "not_found";
    static constexpr std::string_view pending = "pending";

    static FindOutcome read(const Json::Value& document) {
        return FindOutcome{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// One source as facts. The server turns these into ranked attempts.
struct FoundSource {
    std::optional<std::vector<std::string>> audio;
    std::optional<std::string> container;
    std::string group;
    std::map<std::string, std::string> headers;
    std::optional<std::int64_t> height;
    AttemptKind kind;
    std::string url;
    bool verified = false;

    static FoundSource read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "audio");
        const Json::Value* raw1 = Json::at(fields, "container");
        const Json::Value* raw2 = Json::need(fields, "group", "FoundSource");
        const Json::Value* raw3 = Json::need(fields, "headers", "FoundSource");
        const Json::Value* raw4 = Json::at(fields, "height");
        const Json::Value* raw5 = Json::need(fields, "kind", "FoundSource");
        const Json::Value* raw6 = Json::need(fields, "url", "FoundSource");
        const Json::Value* raw7 = Json::need(fields, "verified", "FoundSource");
        return FoundSource{
                raw0 == nullptr ? std::optional<std::vector<std::string>>() : std::optional<std::vector<std::string>>(Json::list<std::string>(*raw0, [](const Json::Value& x0) { return Json::as_text(x0); })),
                raw1 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw1)),
                Json::as_text(*raw2),
                Json::map<std::string>(*raw3, [](const Json::Value& x0) { return Json::as_text(x0); }),
                raw4 == nullptr ? std::optional<std::int64_t>() : std::optional<std::int64_t>(Json::as_long(*raw4)),
                AttemptKind::read(*raw5),
                Json::as_text(*raw6),
                Json::as_bool(*raw7)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->audio.has_value()) {
            count = Json::key(out, count, "audio");
            out += '[';
            for (std::size_t index0 = 0; index0 < (*this->audio).size(); index0++) {
                if (index0 > 0) {
                    out += ',';
                }
                Json::text(out, (*this->audio)[index0]);
            }
            out += ']';
        }
        if (this->container.has_value()) {
            count = Json::key(out, count, "container");
            Json::text(out, (*this->container));
        }
        count = Json::key(out, count, "group");
        Json::text(out, this->group);
        count = Json::key(out, count, "headers");
        out += '{';
        int n0 = 0;
        for (const auto& entry0 : this->headers) {
            n0 = Json::key(out, n0, entry0.first);
            Json::text(out, entry0.second);
        }
        out += '}';
        if (this->height.has_value()) {
            count = Json::key(out, count, "height");
            out += std::to_string((*this->height));
        }
        count = Json::key(out, count, "kind");
        this->kind.write(out);
        count = Json::key(out, count, "url");
        Json::text(out, this->url);
        count = Json::key(out, count, "verified");
        out += this->verified ? "true" : "false";
        out += '}';
    }
};

/// Answer, possibly partial when the budget ran out.
struct FindAnswer {
    /// Read the rest through `/v1/find/{job}`.
    std::optional<std::string> job;
    FindOutcome outcome;
    std::vector<FoundSource> sources;

    static FindAnswer read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "job");
        const Json::Value* raw1 = Json::need(fields, "outcome", "FindAnswer");
        const Json::Value* raw2 = Json::need(fields, "sources", "FindAnswer");
        return FindAnswer{
                raw0 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw0)),
                FindOutcome::read(*raw1),
                Json::list<FoundSource>(*raw2, [](const Json::Value& x0) { return FoundSource::read(x0); })};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->job.has_value()) {
            count = Json::key(out, count, "job");
            Json::text(out, (*this->job));
        }
        count = Json::key(out, count, "outcome");
        this->outcome.write(out);
        count = Json::key(out, count, "sources");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->sources.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->sources[index0].write(out);
        }
        out += ']';
        out += '}';
    }
};

/// Title facts come from the server, so a finder never looks them up.
struct FindRequest {
    /// Total time this search may take.
    std::int64_t budget_ms = 0;
    std::optional<EpisodeRef> episode;
    std::string name;
    TitleKey title;
    std::optional<std::int64_t> year;

    static FindRequest read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "budget_ms", "FindRequest");
        const Json::Value* raw1 = Json::at(fields, "episode");
        const Json::Value* raw2 = Json::need(fields, "name", "FindRequest");
        const Json::Value* raw3 = Json::need(fields, "title", "FindRequest");
        const Json::Value* raw4 = Json::at(fields, "year");
        return FindRequest{
                Json::as_long(*raw0),
                raw1 == nullptr ? std::optional<EpisodeRef>() : std::optional<EpisodeRef>(EpisodeRef::read(*raw1)),
                Json::as_text(*raw2),
                TitleKey::read(*raw3),
                raw4 == nullptr ? std::optional<std::int64_t>() : std::optional<std::int64_t>(Json::as_long(*raw4))};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "budget_ms");
        out += std::to_string(this->budget_ms);
        if (this->episode.has_value()) {
            count = Json::key(out, count, "episode");
            (*this->episode).write(out);
        }
        count = Json::key(out, count, "name");
        Json::text(out, this->name);
        count = Json::key(out, count, "title");
        this->title.write(out);
        if (this->year.has_value()) {
            count = Json::key(out, count, "year");
            out += std::to_string((*this->year));
        }
        out += '}';
    }
};

/// State of a part of the service.
struct ServiceState {
    std::string value;

    static constexpr std::string_view ready = "ready";
    static constexpr std::string_view degraded = "degraded";
    static constexpr std::string_view down = "down";

    static ServiceState read(const Json::Value& document) {
        return ServiceState{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// `/v1/health` of a finder.
struct FinderHealth {
    std::string build;
    ServiceState state;

    static FinderHealth read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "build", "FinderHealth");
        const Json::Value* raw1 = Json::need(fields, "state", "FinderHealth");
        return FinderHealth{
                Json::as_text(*raw0),
                ServiceState::read(*raw1)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "build");
        Json::text(out, this->build);
        count = Json::key(out, count, "state");
        this->state.write(out);
        out += '}';
    }
};

/// What the line under a title's logo on the profile chooser tells. The app writes the words.
struct GateLineKind {
    std::string value;

    static constexpr std::string_view coming = "coming";
    static constexpr std::string_view seasons_coming = "seasons_coming";
    static constexpr std::string_view ranked = "ranked";
    static constexpr std::string_view none = "none";

    static GateLineKind read(const Json::Value& document) {
        return GateLineKind{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// The two kinds of title. Closed on purpose: a new kind reaches old clients only through a dated transform, so a key always names a kind this build can open.
struct TitleKind {
    std::string value;

    static constexpr std::string_view movie = "movie";
    static constexpr std::string_view tv = "tv";

    static TitleKind read(const Json::Value& document) {
        return TitleKind{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// The facts of the line under a title's logo on the profile chooser. Never words: the app writes them in the viewer's language, with the date in its own month names and digits.
struct GateLine {
    /// The day, for `coming` and `seasons_coming`.
    std::optional<CalendarDay> date;
    GateLineKind kind;
    /// Films or series, for `ranked`.
    std::optional<TitleKind> media;
    /// The place from 1, for `ranked`.
    std::optional<std::int64_t> rank;

    static GateLine read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "date");
        const Json::Value* raw1 = Json::need(fields, "kind", "GateLine");
        const Json::Value* raw2 = Json::at(fields, "media");
        const Json::Value* raw3 = Json::at(fields, "rank");
        return GateLine{
                raw0 == nullptr ? std::optional<CalendarDay>() : std::optional<CalendarDay>(CalendarDay::read(*raw0)),
                GateLineKind::read(*raw1),
                raw2 == nullptr ? std::optional<TitleKind>() : std::optional<TitleKind>(TitleKind::read(*raw2)),
                raw3 == nullptr ? std::optional<std::int64_t>() : std::optional<std::int64_t>(Json::as_long(*raw3))};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->date.has_value()) {
            count = Json::key(out, count, "date");
            (*this->date).write(out);
        }
        count = Json::key(out, count, "kind");
        this->kind.write(out);
        if (this->media.has_value()) {
            count = Json::key(out, count, "media");
            (*this->media).write(out);
        }
        if (this->rank.has_value()) {
            count = Json::key(out, count, "rank");
            out += std::to_string((*this->rank));
        }
        out += '}';
    }
};

/// One title in a section.
struct Tile {
    std::optional<std::vector<Badge>> badges;
    std::optional<EpisodeRef> episode;
    std::optional<std::vector<Image>> images;
    std::string name;
    /// Whole percent watched.
    std::optional<std::int64_t> percent;
    std::optional<std::vector<std::string>> tags;
    TitleKey title;

    static Tile read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "badges");
        const Json::Value* raw1 = Json::at(fields, "episode");
        const Json::Value* raw2 = Json::at(fields, "images");
        const Json::Value* raw3 = Json::need(fields, "name", "Tile");
        const Json::Value* raw4 = Json::at(fields, "percent");
        const Json::Value* raw5 = Json::at(fields, "tags");
        const Json::Value* raw6 = Json::need(fields, "title", "Tile");
        return Tile{
                raw0 == nullptr ? std::optional<std::vector<Badge>>() : std::optional<std::vector<Badge>>(Json::list<Badge>(*raw0, [](const Json::Value& x0) { return Badge::read(x0); })),
                raw1 == nullptr ? std::optional<EpisodeRef>() : std::optional<EpisodeRef>(EpisodeRef::read(*raw1)),
                raw2 == nullptr ? std::optional<std::vector<Image>>() : std::optional<std::vector<Image>>(Json::list<Image>(*raw2, [](const Json::Value& x0) { return Image::read(x0); })),
                Json::as_text(*raw3),
                raw4 == nullptr ? std::optional<std::int64_t>() : std::optional<std::int64_t>(Json::as_long(*raw4)),
                raw5 == nullptr ? std::optional<std::vector<std::string>>() : std::optional<std::vector<std::string>>(Json::list<std::string>(*raw5, [](const Json::Value& x0) { return Json::as_text(x0); })),
                TitleKey::read(*raw6)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->badges.has_value()) {
            count = Json::key(out, count, "badges");
            out += '[';
            for (std::size_t index0 = 0; index0 < (*this->badges).size(); index0++) {
                if (index0 > 0) {
                    out += ',';
                }
                (*this->badges)[index0].write(out);
            }
            out += ']';
        }
        if (this->episode.has_value()) {
            count = Json::key(out, count, "episode");
            (*this->episode).write(out);
        }
        if (this->images.has_value()) {
            count = Json::key(out, count, "images");
            out += '[';
            for (std::size_t index0 = 0; index0 < (*this->images).size(); index0++) {
                if (index0 > 0) {
                    out += ',';
                }
                (*this->images)[index0].write(out);
            }
            out += ']';
        }
        count = Json::key(out, count, "name");
        Json::text(out, this->name);
        if (this->percent.has_value()) {
            count = Json::key(out, count, "percent");
            out += std::to_string((*this->percent));
        }
        if (this->tags.has_value()) {
            count = Json::key(out, count, "tags");
            out += '[';
            for (std::size_t index0 = 0; index0 < (*this->tags).size(); index0++) {
                if (index0 > 0) {
                    out += ',';
                }
                Json::text(out, (*this->tags)[index0]);
            }
            out += ']';
        }
        count = Json::key(out, count, "title");
        this->title.write(out);
        out += '}';
    }
};

/// One title shown over the profile chooser: its tile, with art, logo and poster, and its line.
struct GateTitle {
    GateLine line;
    Tile tile;

    static GateTitle read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "line", "GateTitle");
        const Json::Value* raw1 = Json::need(fields, "tile", "GateTitle");
        return GateTitle{
                GateLine::read(*raw0),
                Tile::read(*raw1)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "line");
        this->line.write(out);
        count = Json::key(out, count, "tile");
        this->tile.write(out);
        out += '}';
    }
};

/// An article as a list shows it.
struct HelpLink {
    std::string slug;
    std::optional<std::string> summary;
    std::string title;

    static HelpLink read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "slug", "HelpLink");
        const Json::Value* raw1 = Json::at(fields, "summary");
        const Json::Value* raw2 = Json::need(fields, "title", "HelpLink");
        return HelpLink{
                Json::as_text(*raw0),
                raw1 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw1)),
                Json::as_text(*raw2)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "slug");
        Json::text(out, this->slug);
        if (this->summary.has_value()) {
            count = Json::key(out, count, "summary");
            Json::text(out, (*this->summary));
        }
        count = Json::key(out, count, "title");
        Json::text(out, this->title);
        out += '}';
    }
};

/// What a block of an article is.
struct HelpBlockKind {
    std::string value;

    static constexpr std::string_view paragraph = "paragraph";
    static constexpr std::string_view heading = "heading";
    static constexpr std::string_view steps = "steps";
    static constexpr std::string_view list = "list";
    static constexpr std::string_view note = "note";
    static constexpr std::string_view tool = "tool";

    static HelpBlockKind read(const Json::Value& document) {
        return HelpBlockKind{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// Some words of a line, with how they are marked.
struct HelpRun {
    std::optional<bool> bold;
    /// Where the words lead: an address in the app, a web page or a mail address.
    std::optional<std::string> link;
    std::string text;

    static HelpRun read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "bold");
        const Json::Value* raw1 = Json::at(fields, "link");
        const Json::Value* raw2 = Json::need(fields, "text", "HelpRun");
        return HelpRun{
                raw0 == nullptr ? std::optional<bool>() : std::optional<bool>(Json::as_bool(*raw0)),
                raw1 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw1)),
                Json::as_text(*raw2)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->bold.has_value()) {
            count = Json::key(out, count, "bold");
            out += (*this->bold) ? "true" : "false";
        }
        if (this->link.has_value()) {
            count = Json::key(out, count, "link");
            Json::text(out, (*this->link));
        }
        count = Json::key(out, count, "text");
        Json::text(out, this->text);
        out += '}';
    }
};

/// One line of a block. A paragraph has one, a list has one per item.
struct HelpLine {
    std::vector<HelpRun> runs;

    static HelpLine read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "runs", "HelpLine");
        return HelpLine{
                Json::list<HelpRun>(*raw0, [](const Json::Value& x0) { return HelpRun::read(x0); })};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "runs");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->runs.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->runs[index0].write(out);
        }
        out += ']';
        out += '}';
    }
};

struct HelpBlock {
    HelpBlockKind kind;
    std::optional<std::vector<HelpLine>> lines;
    /// The action of a tool block.
    std::optional<std::string> tool;

    static HelpBlock read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "kind", "HelpBlock");
        const Json::Value* raw1 = Json::at(fields, "lines");
        const Json::Value* raw2 = Json::at(fields, "tool");
        return HelpBlock{
                HelpBlockKind::read(*raw0),
                raw1 == nullptr ? std::optional<std::vector<HelpLine>>() : std::optional<std::vector<HelpLine>>(Json::list<HelpLine>(*raw1, [](const Json::Value& x0) { return HelpLine::read(x0); })),
                raw2 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw2))};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "kind");
        this->kind.write(out);
        if (this->lines.has_value()) {
            count = Json::key(out, count, "lines");
            out += '[';
            for (std::size_t index0 = 0; index0 < (*this->lines).size(); index0++) {
                if (index0 > 0) {
                    out += ',';
                }
                (*this->lines)[index0].write(out);
            }
            out += ']';
        }
        if (this->tool.has_value()) {
            count = Json::key(out, count, "tool");
            Json::text(out, (*this->tool));
        }
        out += '}';
    }
};

/// A part of a page. A folded one starts closed where the app can open and close it.
struct HelpSection {
    std::vector<HelpBlock> blocks;
    std::optional<bool> folded;
    std::optional<std::string> title;

    static HelpSection read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "blocks", "HelpSection");
        const Json::Value* raw1 = Json::at(fields, "folded");
        const Json::Value* raw2 = Json::at(fields, "title");
        return HelpSection{
                Json::list<HelpBlock>(*raw0, [](const Json::Value& x0) { return HelpBlock::read(x0); }),
                raw1 == nullptr ? std::optional<bool>() : std::optional<bool>(Json::as_bool(*raw1)),
                raw2 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw2))};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "blocks");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->blocks.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->blocks[index0].write(out);
        }
        out += ']';
        if (this->folded.has_value()) {
            count = Json::key(out, count, "folded");
            out += (*this->folded) ? "true" : "false";
        }
        if (this->title.has_value()) {
            count = Json::key(out, count, "title");
            Json::text(out, (*this->title));
        }
        out += '}';
    }
};

/// `/v1/help/articles/{slug}`.
struct HelpArticle {
    std::optional<std::vector<HelpLink>> related;
    std::vector<HelpSection> sections;
    std::string slug;
    std::string title;

    static HelpArticle read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "related");
        const Json::Value* raw1 = Json::need(fields, "sections", "HelpArticle");
        const Json::Value* raw2 = Json::need(fields, "slug", "HelpArticle");
        const Json::Value* raw3 = Json::need(fields, "title", "HelpArticle");
        return HelpArticle{
                raw0 == nullptr ? std::optional<std::vector<HelpLink>>() : std::optional<std::vector<HelpLink>>(Json::list<HelpLink>(*raw0, [](const Json::Value& x0) { return HelpLink::read(x0); })),
                Json::list<HelpSection>(*raw1, [](const Json::Value& x0) { return HelpSection::read(x0); }),
                Json::as_text(*raw2),
                Json::as_text(*raw3)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->related.has_value()) {
            count = Json::key(out, count, "related");
            out += '[';
            for (std::size_t index0 = 0; index0 < (*this->related).size(); index0++) {
                if (index0 > 0) {
                    out += ',';
                }
                (*this->related)[index0].write(out);
            }
            out += ']';
        }
        count = Json::key(out, count, "sections");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->sections.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->sections[index0].write(out);
        }
        out += ']';
        count = Json::key(out, count, "slug");
        Json::text(out, this->slug);
        count = Json::key(out, count, "title");
        Json::text(out, this->title);
        out += '}';
    }
};

struct HelpGroup {
    std::vector<HelpLink> articles;
    std::string id;
    std::string title;

    static HelpGroup read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "articles", "HelpGroup");
        const Json::Value* raw1 = Json::need(fields, "id", "HelpGroup");
        const Json::Value* raw2 = Json::need(fields, "title", "HelpGroup");
        return HelpGroup{
                Json::list<HelpLink>(*raw0, [](const Json::Value& x0) { return HelpLink::read(x0); }),
                Json::as_text(*raw1),
                Json::as_text(*raw2)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "articles");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->articles.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->articles[index0].write(out);
        }
        out += ']';
        count = Json::key(out, count, "id");
        Json::text(out, this->id);
        count = Json::key(out, count, "title");
        Json::text(out, this->title);
        out += '}';
    }
};

struct HelpPolicyLink {
    std::string id;
    std::string title;

    static HelpPolicyLink read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "id", "HelpPolicyLink");
        const Json::Value* raw1 = Json::need(fields, "title", "HelpPolicyLink");
        return HelpPolicyLink{
                Json::as_text(*raw0),
                Json::as_text(*raw1)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "id");
        Json::text(out, this->id);
        count = Json::key(out, count, "title");
        Json::text(out, this->title);
        out += '}';
    }
};

struct HelpTopic {
    std::vector<HelpGroup> groups;
    std::string id;
    std::string title;

    static HelpTopic read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "groups", "HelpTopic");
        const Json::Value* raw1 = Json::need(fields, "id", "HelpTopic");
        const Json::Value* raw2 = Json::need(fields, "title", "HelpTopic");
        return HelpTopic{
                Json::list<HelpGroup>(*raw0, [](const Json::Value& x0) { return HelpGroup::read(x0); }),
                Json::as_text(*raw1),
                Json::as_text(*raw2)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "groups");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->groups.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->groups[index0].write(out);
        }
        out += ']';
        count = Json::key(out, count, "id");
        Json::text(out, this->id);
        count = Json::key(out, count, "title");
        Json::text(out, this->title);
        out += '}';
    }
};

/// `/v1/help`.
struct HelpHome {
    std::vector<HelpPolicyLink> policies;
    std::vector<HelpLink> popular;
    std::vector<HelpTopic> topics;

    static HelpHome read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "policies", "HelpHome");
        const Json::Value* raw1 = Json::need(fields, "popular", "HelpHome");
        const Json::Value* raw2 = Json::need(fields, "topics", "HelpHome");
        return HelpHome{
                Json::list<HelpPolicyLink>(*raw0, [](const Json::Value& x0) { return HelpPolicyLink::read(x0); }),
                Json::list<HelpLink>(*raw1, [](const Json::Value& x0) { return HelpLink::read(x0); }),
                Json::list<HelpTopic>(*raw2, [](const Json::Value& x0) { return HelpTopic::read(x0); })};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "policies");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->policies.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->policies[index0].write(out);
        }
        out += ']';
        count = Json::key(out, count, "popular");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->popular.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->popular[index0].write(out);
        }
        out += ']';
        count = Json::key(out, count, "topics");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->topics.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->topics[index0].write(out);
        }
        out += ']';
        out += '}';
    }
};

/// `/v1/help/policies/{id}`.
struct HelpPolicy {
    std::string id;
    std::vector<HelpSection> sections;
    std::string title;
    std::optional<std::string> updated;

    static HelpPolicy read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "id", "HelpPolicy");
        const Json::Value* raw1 = Json::need(fields, "sections", "HelpPolicy");
        const Json::Value* raw2 = Json::need(fields, "title", "HelpPolicy");
        const Json::Value* raw3 = Json::at(fields, "updated");
        return HelpPolicy{
                Json::as_text(*raw0),
                Json::list<HelpSection>(*raw1, [](const Json::Value& x0) { return HelpSection::read(x0); }),
                Json::as_text(*raw2),
                raw3 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw3))};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "id");
        Json::text(out, this->id);
        count = Json::key(out, count, "sections");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->sections.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->sections[index0].write(out);
        }
        out += ']';
        count = Json::key(out, count, "title");
        Json::text(out, this->title);
        if (this->updated.has_value()) {
            count = Json::key(out, count, "updated");
            Json::text(out, (*this->updated));
        }
        out += '}';
    }
};

/// `/v1/help/search`.
struct HelpResults {
    std::vector<HelpLink> articles;

    static HelpResults read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "articles", "HelpResults");
        return HelpResults{
                Json::list<HelpLink>(*raw0, [](const Json::Value& x0) { return HelpLink::read(x0); })};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "articles");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->articles.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->articles[index0].write(out);
        }
        out += ']';
        out += '}';
    }
};

/// `POST /v1/core/step`. `config` is sent on the first step of a hosted session.
struct HostedStep {
    std::optional<CoreConfig> config;
    CoreIn input;

    static HostedStep read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "config");
        const Json::Value* raw1 = Json::need(fields, "input", "HostedStep");
        return HostedStep{
                raw0 == nullptr ? std::optional<CoreConfig>() : std::optional<CoreConfig>(CoreConfig::read(*raw0)),
                CoreIn::read(*raw1)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->config.has_value()) {
            count = Json::key(out, count, "config");
            (*this->config).write(out);
        }
        count = Json::key(out, count, "input");
        this->input.write(out);
        out += '}';
    }
};

/// Which way a language's lines run. The apps lay out, mirror and move focus by it; the player's timeline and the brand's art never turn.
struct TextDirection {
    std::string value;

    static constexpr std::string_view ltr = "ltr";
    static constexpr std::string_view rtl = "rtl";

    static TextDirection read(const Json::Value& document) {
        return TextDirection{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// A language this server has text for, named in itself.
struct Language {
    std::string code;
    std::optional<TextDirection> dir;
    std::string name;

    static Language read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "code", "Language");
        const Json::Value* raw1 = Json::at(fields, "dir");
        const Json::Value* raw2 = Json::need(fields, "name", "Language");
        return Language{
                Json::as_text(*raw0),
                raw1 == nullptr ? std::optional<TextDirection>() : std::optional<TextDirection>(TextDirection::read(*raw1)),
                Json::as_text(*raw2)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "code");
        Json::text(out, this->code);
        if (this->dir.has_value()) {
            count = Json::key(out, count, "dir");
            (*this->dir).write(out);
        }
        count = Json::key(out, count, "name");
        Json::text(out, this->name);
        out += '}';
    }
};

struct LanguageList {
    std::vector<Language> languages;

    static LanguageList read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "languages", "LanguageList");
        return LanguageList{
                Json::list<Language>(*raw0, [](const Json::Value& x0) { return Language::read(x0); })};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "languages");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->languages.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->languages[index0].write(out);
        }
        out += ']';
        out += '}';
    }
};

/// Asks `core_open` for the offline layer. A host sets it as the `layer` key of the text it gives `core_open`, next to the `CoreConfig` keys. With no `layer` key the core runs alone, and the core itself never reads this key.
struct LayerConfig {
    /// Names this boot. It changes when monotonic time restarts.
    std::string boot_id;
    /// The byte cap from the capability card. 0 means nothing is kept across a restart.
    std::int64_t storage_bytes = 0;

    static LayerConfig read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "boot_id", "LayerConfig");
        const Json::Value* raw1 = Json::need(fields, "storage_bytes", "LayerConfig");
        return LayerConfig{
                Json::as_text(*raw0),
                Json::as_long(*raw1)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "boot_id");
        Json::text(out, this->boot_id);
        count = Json::key(out, count, "storage_bytes");
        out += std::to_string(this->storage_bytes);
        out += '}';
    }
};

/// Skip marks.
struct Marks {
    std::vector<TimeWindow> windows;

    static Marks read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "windows", "Marks");
        return Marks{
                Json::list<TimeWindow>(*raw0, [](const Json::Value& x0) { return TimeWindow::read(x0); })};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "windows");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->windows.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->windows[index0].write(out);
        }
        out += ']';
        out += '}';
    }
};

/// Ask for skip marks, after first frame, series only.
struct MarksRequest {
    std::string attempt;
    std::optional<std::int64_t> delay_ms;
    std::int64_t duration_ms = 0;
    std::optional<std::string> file;

    static MarksRequest read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "attempt", "MarksRequest");
        const Json::Value* raw1 = Json::at(fields, "delay_ms");
        const Json::Value* raw2 = Json::need(fields, "duration_ms", "MarksRequest");
        const Json::Value* raw3 = Json::at(fields, "file");
        return MarksRequest{
                Json::as_text(*raw0),
                raw1 == nullptr ? std::optional<std::int64_t>() : std::optional<std::int64_t>(Json::as_long(*raw1)),
                Json::as_long(*raw2),
                raw3 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw3))};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "attempt");
        Json::text(out, this->attempt);
        if (this->delay_ms.has_value()) {
            count = Json::key(out, count, "delay_ms");
            out += std::to_string((*this->delay_ms));
        }
        count = Json::key(out, count, "duration_ms");
        out += std::to_string(this->duration_ms);
        if (this->file.has_value()) {
            count = Json::key(out, count, "file");
            Json::text(out, (*this->file));
        }
        out += '}';
    }
};

/// One row, hero, grid or notice of a screen.
struct Section {
    /// Reads more of this section through `/v1/more/{cursor}`.
    std::optional<std::string> cursor;
    std::string id;
    SectionKind kind;
    std::optional<Notice> notice;
    std::optional<std::vector<Tile>> tiles;
    /// Translated title.
    std::optional<std::string> title;

    static Section read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "cursor");
        const Json::Value* raw1 = Json::need(fields, "id", "Section");
        const Json::Value* raw2 = Json::need(fields, "kind", "Section");
        const Json::Value* raw3 = Json::at(fields, "notice");
        const Json::Value* raw4 = Json::at(fields, "tiles");
        const Json::Value* raw5 = Json::at(fields, "title");
        return Section{
                raw0 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw0)),
                Json::as_text(*raw1),
                SectionKind::read(*raw2),
                raw3 == nullptr ? std::optional<Notice>() : std::optional<Notice>(Notice::read(*raw3)),
                raw4 == nullptr ? std::optional<std::vector<Tile>>() : std::optional<std::vector<Tile>>(Json::list<Tile>(*raw4, [](const Json::Value& x0) { return Tile::read(x0); })),
                raw5 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw5))};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->cursor.has_value()) {
            count = Json::key(out, count, "cursor");
            Json::text(out, (*this->cursor));
        }
        count = Json::key(out, count, "id");
        Json::text(out, this->id);
        count = Json::key(out, count, "kind");
        this->kind.write(out);
        if (this->notice.has_value()) {
            count = Json::key(out, count, "notice");
            (*this->notice).write(out);
        }
        if (this->tiles.has_value()) {
            count = Json::key(out, count, "tiles");
            out += '[';
            for (std::size_t index0 = 0; index0 < (*this->tiles).size(); index0++) {
                if (index0 > 0) {
                    out += ',';
                }
                (*this->tiles)[index0].write(out);
            }
            out += ']';
        }
        if (this->title.has_value()) {
            count = Json::key(out, count, "title");
            Json::text(out, (*this->title));
        }
        out += '}';
    }
};

/// The next part of a screen or a section.
struct More {
    std::optional<std::string> cursor;
    std::optional<std::vector<Section>> sections;
    std::optional<std::vector<Tile>> tiles;

    static More read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "cursor");
        const Json::Value* raw1 = Json::at(fields, "sections");
        const Json::Value* raw2 = Json::at(fields, "tiles");
        return More{
                raw0 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw0)),
                raw1 == nullptr ? std::optional<std::vector<Section>>() : std::optional<std::vector<Section>>(Json::list<Section>(*raw1, [](const Json::Value& x0) { return Section::read(x0); })),
                raw2 == nullptr ? std::optional<std::vector<Tile>>() : std::optional<std::vector<Tile>>(Json::list<Tile>(*raw2, [](const Json::Value& x0) { return Tile::read(x0); }))};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->cursor.has_value()) {
            count = Json::key(out, count, "cursor");
            Json::text(out, (*this->cursor));
        }
        if (this->sections.has_value()) {
            count = Json::key(out, count, "sections");
            out += '[';
            for (std::size_t index0 = 0; index0 < (*this->sections).size(); index0++) {
                if (index0 > 0) {
                    out += ',';
                }
                (*this->sections)[index0].write(out);
            }
            out += ']';
        }
        if (this->tiles.has_value()) {
            count = Json::key(out, count, "tiles");
            out += '[';
            for (std::size_t index0 = 0; index0 < (*this->tiles).size(); index0++) {
                if (index0 > 0) {
                    out += ',';
                }
                (*this->tiles)[index0].write(out);
            }
            out += ']';
        }
        out += '}';
    }
};

/// Late attempts.
struct MoreAttempts {
    std::vector<Attempt> attempts;
    bool more = false;

    static MoreAttempts read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "attempts", "MoreAttempts");
        const Json::Value* raw1 = Json::need(fields, "more", "MoreAttempts");
        return MoreAttempts{
                Json::list<Attempt>(*raw0, [](const Json::Value& x0) { return Attempt::read(x0); }),
                Json::as_bool(*raw1)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "attempts");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->attempts.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->attempts[index0].write(out);
        }
        out += ']';
        count = Json::key(out, count, "more");
        out += this->more ? "true" : "false";
        out += '}';
    }
};

/// One entry of the menu.
struct NavItem {
    std::optional<std::string> genre;
    std::string id;
    std::string label;
    ScreenName screen;

    static NavItem read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "genre");
        const Json::Value* raw1 = Json::need(fields, "id", "NavItem");
        const Json::Value* raw2 = Json::need(fields, "label", "NavItem");
        const Json::Value* raw3 = Json::need(fields, "screen", "NavItem");
        return NavItem{
                raw0 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw0)),
                Json::as_text(*raw1),
                Json::as_text(*raw2),
                ScreenName::read(*raw3)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->genre.has_value()) {
            count = Json::key(out, count, "genre");
            Json::text(out, (*this->genre));
        }
        count = Json::key(out, count, "id");
        Json::text(out, this->id);
        count = Json::key(out, count, "label");
        Json::text(out, this->label);
        count = Json::key(out, count, "screen");
        this->screen.write(out);
        out += '}';
    }
};

/// The menu for this class and profile.
struct Nav {
    std::vector<NavItem> items;
    std::int64_t ttl_s = 0;

    static Nav read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "items", "Nav");
        const Json::Value* raw1 = Json::need(fields, "ttl_s", "Nav");
        return Nav{
                Json::list<NavItem>(*raw0, [](const Json::Value& x0) { return NavItem::read(x0); }),
                Json::as_long(*raw1)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "items");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->items.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->items[index0].write(out);
        }
        out += ']';
        count = Json::key(out, count, "ttl_s");
        out += std::to_string(this->ttl_s);
        out += '}';
    }
};

/// What kind of notice this is.
struct NoticeKind {
    std::string value;

    static constexpr std::string_view new_arrival = "new_arrival";
    static constexpr std::string_view coming = "coming";
    static constexpr std::string_view reminder_live = "reminder_live";
    static constexpr std::string_view new_season = "new_season";
    static constexpr std::string_view new_episode = "new_episode";

    static NoticeKind read(const Json::Value& document) {
        return NoticeKind{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// One item in the notification feed.
struct NoticeItem {
    CalendarDay date;
    std::string id;
    NoticeKind kind;
    Tile tile;
    bool unread = false;

    static NoticeItem read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "date", "NoticeItem");
        const Json::Value* raw1 = Json::need(fields, "id", "NoticeItem");
        const Json::Value* raw2 = Json::need(fields, "kind", "NoticeItem");
        const Json::Value* raw3 = Json::need(fields, "tile", "NoticeItem");
        const Json::Value* raw4 = Json::need(fields, "unread", "NoticeItem");
        return NoticeItem{
                CalendarDay::read(*raw0),
                Json::as_text(*raw1),
                NoticeKind::read(*raw2),
                Tile::read(*raw3),
                Json::as_bool(*raw4)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "date");
        this->date.write(out);
        count = Json::key(out, count, "id");
        Json::text(out, this->id);
        count = Json::key(out, count, "kind");
        this->kind.write(out);
        count = Json::key(out, count, "tile");
        this->tile.write(out);
        count = Json::key(out, count, "unread");
        out += this->unread ? "true" : "false";
        out += '}';
    }
};

/// `GET /v1/notifications/count`.
struct NotificationCount {
    std::int64_t unread = 0;

    static NotificationCount read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "unread", "NotificationCount");
        return NotificationCount{
                Json::as_long(*raw0)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "unread");
        out += std::to_string(this->unread);
        out += '}';
    }
};

/// `POST /v1/notifications/read`.
struct NotificationRead {
    std::string up_to;

    static NotificationRead read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "up_to", "NotificationRead");
        return NotificationRead{
                Json::as_text(*raw0)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "up_to");
        Json::text(out, this->up_to);
        out += '}';
    }
};

/// `GET /v1/notifications`.
struct Notifications {
    std::optional<std::string> cursor;
    bool has_more = false;
    std::vector<NoticeItem> items;

    static Notifications read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "cursor");
        const Json::Value* raw1 = Json::need(fields, "has_more", "Notifications");
        const Json::Value* raw2 = Json::need(fields, "items", "Notifications");
        return Notifications{
                raw0 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw0)),
                Json::as_bool(*raw1),
                Json::list<NoticeItem>(*raw2, [](const Json::Value& x0) { return NoticeItem::read(x0); })};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->cursor.has_value()) {
            count = Json::key(out, count, "cursor");
            Json::text(out, (*this->cursor));
        }
        count = Json::key(out, count, "has_more");
        out += this->has_more ? "true" : "false";
        count = Json::key(out, count, "items");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->items.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->items[index0].write(out);
        }
        out += ']';
        out += '}';
    }
};

struct PasswordChange {
    std::string new_password;

    static PasswordChange read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "new_password", "PasswordChange");
        return PasswordChange{
                Json::as_text(*raw0)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "new_password");
        Json::text(out, this->new_password);
        out += '}';
    }
};

struct PasswordSignIn {
    std::string email;
    std::string password;

    static PasswordSignIn read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "email", "PasswordSignIn");
        const Json::Value* raw1 = Json::need(fields, "password", "PasswordSignIn");
        return PasswordSignIn{
                Json::as_text(*raw0),
                Json::as_text(*raw1)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "email");
        Json::text(out, this->email);
        count = Json::key(out, count, "password");
        Json::text(out, this->password);
        out += '}';
    }
};

struct PhoneChange {
    std::string phone;

    static PhoneChange read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "phone", "PhoneChange");
        return PhoneChange{
                Json::as_text(*raw0)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "phone");
        Json::text(out, this->phone);
        out += '}';
    }
};

/// Absent clears the PIN.
struct PinSet {
    std::optional<std::string> pin;

    static PinSet read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "pin");
        return PinSet{
                raw0 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw0))};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->pin.has_value()) {
            count = Json::key(out, count, "pin");
            Json::text(out, (*this->pin));
        }
        out += '}';
    }
};

/// Answer to a play request.
struct PlayAnswer {
    std::optional<AgeCard> age_card;
    std::vector<Attempt> attempts;
    std::optional<std::string> audio_default;
    /// This profile's setting: the next episode starts by itself at the end of play.
    std::optional<bool> autoplay_next;
    /// Attempts removed by the capability card.
    std::int64_t dropped = 0;
    /// True when more attempts may arrive through `/v1/play/{id}/more`.
    bool more = false;
    /// The next episode in the server's order, across seasons. Absent on the last one.
    std::optional<EpisodeRef> next_episode;
    std::string play_id;
    /// Where to start.
    std::int64_t start_at_ms = 0;
    std::optional<std::string> subtitle_default;

    static PlayAnswer read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "age_card");
        const Json::Value* raw1 = Json::need(fields, "attempts", "PlayAnswer");
        const Json::Value* raw2 = Json::at(fields, "audio_default");
        const Json::Value* raw3 = Json::at(fields, "autoplay_next");
        const Json::Value* raw4 = Json::need(fields, "dropped", "PlayAnswer");
        const Json::Value* raw5 = Json::need(fields, "more", "PlayAnswer");
        const Json::Value* raw6 = Json::at(fields, "next_episode");
        const Json::Value* raw7 = Json::need(fields, "play_id", "PlayAnswer");
        const Json::Value* raw8 = Json::need(fields, "start_at_ms", "PlayAnswer");
        const Json::Value* raw9 = Json::at(fields, "subtitle_default");
        return PlayAnswer{
                raw0 == nullptr ? std::optional<AgeCard>() : std::optional<AgeCard>(AgeCard::read(*raw0)),
                Json::list<Attempt>(*raw1, [](const Json::Value& x0) { return Attempt::read(x0); }),
                raw2 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw2)),
                raw3 == nullptr ? std::optional<bool>() : std::optional<bool>(Json::as_bool(*raw3)),
                Json::as_long(*raw4),
                Json::as_bool(*raw5),
                raw6 == nullptr ? std::optional<EpisodeRef>() : std::optional<EpisodeRef>(EpisodeRef::read(*raw6)),
                Json::as_text(*raw7),
                Json::as_long(*raw8),
                raw9 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw9))};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->age_card.has_value()) {
            count = Json::key(out, count, "age_card");
            (*this->age_card).write(out);
        }
        count = Json::key(out, count, "attempts");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->attempts.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->attempts[index0].write(out);
        }
        out += ']';
        if (this->audio_default.has_value()) {
            count = Json::key(out, count, "audio_default");
            Json::text(out, (*this->audio_default));
        }
        if (this->autoplay_next.has_value()) {
            count = Json::key(out, count, "autoplay_next");
            out += (*this->autoplay_next) ? "true" : "false";
        }
        count = Json::key(out, count, "dropped");
        out += std::to_string(this->dropped);
        count = Json::key(out, count, "more");
        out += this->more ? "true" : "false";
        if (this->next_episode.has_value()) {
            count = Json::key(out, count, "next_episode");
            (*this->next_episode).write(out);
        }
        count = Json::key(out, count, "play_id");
        Json::text(out, this->play_id);
        count = Json::key(out, count, "start_at_ms");
        out += std::to_string(this->start_at_ms);
        if (this->subtitle_default.has_value()) {
            count = Json::key(out, count, "subtitle_default");
            Json::text(out, (*this->subtitle_default));
        }
        out += '}';
    }
};

/// What a play report says happened.
struct PlayEventName {
    std::string value;

    static constexpr std::string_view first_frame = "first_frame";
    static constexpr std::string_view rebuffer = "rebuffer";
    static constexpr std::string_view failure = "failure";
    static constexpr std::string_view ended = "ended";

    static PlayEventName read(const Json::Value& document) {
        return PlayEventName{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// One thing that happened during play.
struct PlayEvent {
    std::int64_t at_ms = 0;
    std::optional<std::string> attempt;
    std::optional<ErrorClass> failure;
    std::optional<std::int64_t> first_frame_ms;
    std::optional<std::string> host;
    PlayEventName name;
    std::optional<CoreMode> rung;

    static PlayEvent read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "at_ms", "PlayEvent");
        const Json::Value* raw1 = Json::at(fields, "attempt");
        const Json::Value* raw2 = Json::at(fields, "failure");
        const Json::Value* raw3 = Json::at(fields, "first_frame_ms");
        const Json::Value* raw4 = Json::at(fields, "host");
        const Json::Value* raw5 = Json::need(fields, "name", "PlayEvent");
        const Json::Value* raw6 = Json::at(fields, "rung");
        return PlayEvent{
                Json::as_long(*raw0),
                raw1 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw1)),
                raw2 == nullptr ? std::optional<ErrorClass>() : std::optional<ErrorClass>(ErrorClass::read(*raw2)),
                raw3 == nullptr ? std::optional<std::int64_t>() : std::optional<std::int64_t>(Json::as_long(*raw3)),
                raw4 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw4)),
                PlayEventName::read(*raw5),
                raw6 == nullptr ? std::optional<CoreMode>() : std::optional<CoreMode>(CoreMode::read(*raw6))};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "at_ms");
        out += std::to_string(this->at_ms);
        if (this->attempt.has_value()) {
            count = Json::key(out, count, "attempt");
            Json::text(out, (*this->attempt));
        }
        if (this->failure.has_value()) {
            count = Json::key(out, count, "failure");
            (*this->failure).write(out);
        }
        if (this->first_frame_ms.has_value()) {
            count = Json::key(out, count, "first_frame_ms");
            out += std::to_string((*this->first_frame_ms));
        }
        if (this->host.has_value()) {
            count = Json::key(out, count, "host");
            Json::text(out, (*this->host));
        }
        count = Json::key(out, count, "name");
        this->name.write(out);
        if (this->rung.has_value()) {
            count = Json::key(out, count, "rung");
            (*this->rung).write(out);
        }
        out += '}';
    }
};

/// `/v1/play/{id}/events`.
struct PlayEvents {
    std::vector<PlayEvent> events;

    static PlayEvents read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "events", "PlayEvents");
        return PlayEvents{
                Json::list<PlayEvent>(*raw0, [](const Json::Value& x0) { return PlayEvent::read(x0); })};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "events");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->events.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->events[index0].write(out);
        }
        out += ']';
        out += '}';
    }
};

/// Ask to play a title.
struct PlayRequest {
    std::optional<EpisodeRef> episode;
    TitleKey title;

    static PlayRequest read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "episode");
        const Json::Value* raw1 = Json::need(fields, "title", "PlayRequest");
        return PlayRequest{
                raw0 == nullptr ? std::optional<EpisodeRef>() : std::optional<EpisodeRef>(EpisodeRef::read(*raw0)),
                TitleKey::read(*raw1)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->episode.has_value()) {
            count = Json::key(out, count, "episode");
            (*this->episode).write(out);
        }
        count = Json::key(out, count, "title");
        this->title.write(out);
        out += '}';
    }
};

struct ProfileCreate {
    std::optional<bool> kids;
    std::string name;

    static ProfileCreate read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "kids");
        const Json::Value* raw1 = Json::need(fields, "name", "ProfileCreate");
        return ProfileCreate{
                raw0 == nullptr ? std::optional<bool>() : std::optional<bool>(Json::as_bool(*raw0)),
                Json::as_text(*raw1)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->kids.has_value()) {
            count = Json::key(out, count, "kids");
            out += (*this->kids) ? "true" : "false";
        }
        count = Json::key(out, count, "name");
        Json::text(out, this->name);
        out += '}';
    }
};

struct ProfileEnter {
    std::optional<std::string> pin;

    static ProfileEnter read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "pin");
        return ProfileEnter{
                raw0 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw0))};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->pin.has_value()) {
            count = Json::key(out, count, "pin");
            Json::text(out, (*this->pin));
        }
        out += '}';
    }
};

/// The profile answer names its home, so the core can map it to a target.
struct ProfileEntered {
    ScreenName home;
    Profile profile;

    static ProfileEntered read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "home", "ProfileEntered");
        const Json::Value* raw1 = Json::need(fields, "profile", "ProfileEntered");
        return ProfileEntered{
                ScreenName::read(*raw0),
                Profile::read(*raw1)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "home");
        this->home.write(out);
        count = Json::key(out, count, "profile");
        this->profile.write(out);
        out += '}';
    }
};

/// Titles shown over the profile chooser before anyone has chosen a profile, at most three, in order. Chosen for the whole account: never above the most open tier among its profiles.
struct ProfileGate {
    std::vector<GateTitle> titles;
    /// Seconds this answer stays good.
    std::int64_t ttl_s = 0;

    static ProfileGate read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "titles", "ProfileGate");
        const Json::Value* raw1 = Json::need(fields, "ttl_s", "ProfileGate");
        return ProfileGate{
                Json::list<GateTitle>(*raw0, [](const Json::Value& x0) { return GateTitle::read(x0); }),
                Json::as_long(*raw1)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "titles");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->titles.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->titles[index0].write(out);
        }
        out += ']';
        count = Json::key(out, count, "ttl_s");
        out += std::to_string(this->ttl_s);
        out += '}';
    }
};

/// Needs a step up. Never reachable through a queued command.
struct ProfileLimits {
    std::string age_rating;
    std::vector<std::string> blocked_titles;

    static ProfileLimits read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "age_rating", "ProfileLimits");
        const Json::Value* raw1 = Json::need(fields, "blocked_titles", "ProfileLimits");
        return ProfileLimits{
                Json::as_text(*raw0),
                Json::list<std::string>(*raw1, [](const Json::Value& x0) { return Json::as_text(x0); })};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "age_rating");
        Json::text(out, this->age_rating);
        count = Json::key(out, count, "blocked_titles");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->blocked_titles.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            Json::text(out, this->blocked_titles[index0]);
        }
        out += ']';
        out += '}';
    }
};

struct ProfileSettings {
    std::optional<std::string> audio_language;
    bool autoplay_next = false;
    bool autoplay_previews = false;
    std::optional<std::string> gender;
    std::optional<std::string> handle;
    std::optional<std::string> subtitle_language;
    std::optional<std::map<std::string, std::string>> subtitle_style;
    std::optional<std::string> ui_language;

    static ProfileSettings read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "audio_language");
        const Json::Value* raw1 = Json::need(fields, "autoplay_next", "ProfileSettings");
        const Json::Value* raw2 = Json::need(fields, "autoplay_previews", "ProfileSettings");
        const Json::Value* raw3 = Json::at(fields, "gender");
        const Json::Value* raw4 = Json::at(fields, "handle");
        const Json::Value* raw5 = Json::at(fields, "subtitle_language");
        const Json::Value* raw6 = Json::at(fields, "subtitle_style");
        const Json::Value* raw7 = Json::at(fields, "ui_language");
        return ProfileSettings{
                raw0 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw0)),
                Json::as_bool(*raw1),
                Json::as_bool(*raw2),
                raw3 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw3)),
                raw4 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw4)),
                raw5 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw5)),
                raw6 == nullptr ? std::optional<std::map<std::string, std::string>>() : std::optional<std::map<std::string, std::string>>(Json::map<std::string>(*raw6, [](const Json::Value& x0) { return Json::as_text(x0); })),
                raw7 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw7))};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->audio_language.has_value()) {
            count = Json::key(out, count, "audio_language");
            Json::text(out, (*this->audio_language));
        }
        count = Json::key(out, count, "autoplay_next");
        out += this->autoplay_next ? "true" : "false";
        count = Json::key(out, count, "autoplay_previews");
        out += this->autoplay_previews ? "true" : "false";
        if (this->gender.has_value()) {
            count = Json::key(out, count, "gender");
            Json::text(out, (*this->gender));
        }
        if (this->handle.has_value()) {
            count = Json::key(out, count, "handle");
            Json::text(out, (*this->handle));
        }
        if (this->subtitle_language.has_value()) {
            count = Json::key(out, count, "subtitle_language");
            Json::text(out, (*this->subtitle_language));
        }
        if (this->subtitle_style.has_value()) {
            count = Json::key(out, count, "subtitle_style");
            out += '{';
            int n0 = 0;
            for (const auto& entry0 : (*this->subtitle_style)) {
                n0 = Json::key(out, n0, entry0.first);
                Json::text(out, entry0.second);
            }
            out += '}';
        }
        if (this->ui_language.has_value()) {
            count = Json::key(out, count, "ui_language");
            Json::text(out, (*this->ui_language));
        }
        out += '}';
    }
};

struct ProfileTransfer {
    std::string to_account_email;

    static ProfileTransfer read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "to_account_email", "ProfileTransfer");
        return ProfileTransfer{
                Json::as_text(*raw0)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "to_account_email");
        Json::text(out, this->to_account_email);
        out += '}';
    }
};

struct ProfileUpdate {
    std::optional<std::string> avatar;
    std::optional<bool> kids;
    std::optional<std::string> language;
    std::optional<std::string> name;

    static ProfileUpdate read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "avatar");
        const Json::Value* raw1 = Json::at(fields, "kids");
        const Json::Value* raw2 = Json::at(fields, "language");
        const Json::Value* raw3 = Json::at(fields, "name");
        return ProfileUpdate{
                raw0 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw0)),
                raw1 == nullptr ? std::optional<bool>() : std::optional<bool>(Json::as_bool(*raw1)),
                raw2 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw2)),
                raw3 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw3))};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->avatar.has_value()) {
            count = Json::key(out, count, "avatar");
            Json::text(out, (*this->avatar));
        }
        if (this->kids.has_value()) {
            count = Json::key(out, count, "kids");
            out += (*this->kids) ? "true" : "false";
        }
        if (this->language.has_value()) {
            count = Json::key(out, count, "language");
            Json::text(out, (*this->language));
        }
        if (this->name.has_value()) {
            count = Json::key(out, count, "name");
            Json::text(out, (*this->name));
        }
        out += '}';
    }
};

/// Where play picks up: the episode of a series, and 0 for one not started yet.
struct Resume {
    std::optional<EpisodeRef> episode;
    std::int64_t position_s = 0;

    static Resume read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "episode");
        const Json::Value* raw1 = Json::need(fields, "position_s", "Resume");
        return Resume{
                raw0 == nullptr ? std::optional<EpisodeRef>() : std::optional<EpisodeRef>(EpisodeRef::read(*raw0)),
                Json::as_long(*raw1)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->episode.has_value()) {
            count = Json::key(out, count, "episode");
            (*this->episode).write(out);
        }
        count = Json::key(out, count, "position_s");
        out += std::to_string(this->position_s);
        out += '}';
    }
};

/// A season named on a title page.
struct SeasonSummary {
    std::int64_t episode_count = 0;
    std::int64_t number = 0;

    static SeasonSummary read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "episode_count", "SeasonSummary");
        const Json::Value* raw1 = Json::need(fields, "number", "SeasonSummary");
        return SeasonSummary{
                Json::as_long(*raw0),
                Json::as_long(*raw1)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "episode_count");
        out += std::to_string(this->episode_count);
        count = Json::key(out, count, "number");
        out += std::to_string(this->number);
        out += '}';
    }
};

/// Facts of one title.
struct TitleDetail {
    /// Absent when the title has no rating in either system.
    std::optional<AgeMark> age;
    /// When this title becomes available to watch. Absent when already available.
    std::optional<CalendarDay> available_from;
    std::optional<std::vector<std::string>> genres;
    std::optional<std::vector<Image>> images;
    /// True when this profile has saved the title.
    std::optional<bool> in_list;
    /// Hidden when empty.
    std::optional<std::vector<Tile>> more_like_this;
    std::string name;
    std::optional<std::string> overview;
    /// What this profile said about the title.
    std::optional<Rating> rating;
    /// True when this profile set a reminder for when this title arrives.
    std::optional<bool> reminder_set;
    /// Where this profile left off. Absent when there is nothing to go back to.
    std::optional<Resume> resume;
    std::optional<std::int64_t> runtime_min;
    std::optional<std::vector<SeasonSummary>> seasons;
    TitleKey title;
    std::optional<std::int64_t> year;

    static TitleDetail read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "age");
        const Json::Value* raw1 = Json::at(fields, "available_from");
        const Json::Value* raw2 = Json::at(fields, "genres");
        const Json::Value* raw3 = Json::at(fields, "images");
        const Json::Value* raw4 = Json::at(fields, "in_list");
        const Json::Value* raw5 = Json::at(fields, "more_like_this");
        const Json::Value* raw6 = Json::need(fields, "name", "TitleDetail");
        const Json::Value* raw7 = Json::at(fields, "overview");
        const Json::Value* raw8 = Json::at(fields, "rating");
        const Json::Value* raw9 = Json::at(fields, "reminder_set");
        const Json::Value* raw10 = Json::at(fields, "resume");
        const Json::Value* raw11 = Json::at(fields, "runtime_min");
        const Json::Value* raw12 = Json::at(fields, "seasons");
        const Json::Value* raw13 = Json::need(fields, "title", "TitleDetail");
        const Json::Value* raw14 = Json::at(fields, "year");
        return TitleDetail{
                raw0 == nullptr ? std::optional<AgeMark>() : std::optional<AgeMark>(AgeMark::read(*raw0)),
                raw1 == nullptr ? std::optional<CalendarDay>() : std::optional<CalendarDay>(CalendarDay::read(*raw1)),
                raw2 == nullptr ? std::optional<std::vector<std::string>>() : std::optional<std::vector<std::string>>(Json::list<std::string>(*raw2, [](const Json::Value& x0) { return Json::as_text(x0); })),
                raw3 == nullptr ? std::optional<std::vector<Image>>() : std::optional<std::vector<Image>>(Json::list<Image>(*raw3, [](const Json::Value& x0) { return Image::read(x0); })),
                raw4 == nullptr ? std::optional<bool>() : std::optional<bool>(Json::as_bool(*raw4)),
                raw5 == nullptr ? std::optional<std::vector<Tile>>() : std::optional<std::vector<Tile>>(Json::list<Tile>(*raw5, [](const Json::Value& x0) { return Tile::read(x0); })),
                Json::as_text(*raw6),
                raw7 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw7)),
                raw8 == nullptr ? std::optional<Rating>() : std::optional<Rating>(Rating::read(*raw8)),
                raw9 == nullptr ? std::optional<bool>() : std::optional<bool>(Json::as_bool(*raw9)),
                raw10 == nullptr ? std::optional<Resume>() : std::optional<Resume>(Resume::read(*raw10)),
                raw11 == nullptr ? std::optional<std::int64_t>() : std::optional<std::int64_t>(Json::as_long(*raw11)),
                raw12 == nullptr ? std::optional<std::vector<SeasonSummary>>() : std::optional<std::vector<SeasonSummary>>(Json::list<SeasonSummary>(*raw12, [](const Json::Value& x0) { return SeasonSummary::read(x0); })),
                TitleKey::read(*raw13),
                raw14 == nullptr ? std::optional<std::int64_t>() : std::optional<std::int64_t>(Json::as_long(*raw14))};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->age.has_value()) {
            count = Json::key(out, count, "age");
            (*this->age).write(out);
        }
        if (this->available_from.has_value()) {
            count = Json::key(out, count, "available_from");
            (*this->available_from).write(out);
        }
        if (this->genres.has_value()) {
            count = Json::key(out, count, "genres");
            out += '[';
            for (std::size_t index0 = 0; index0 < (*this->genres).size(); index0++) {
                if (index0 > 0) {
                    out += ',';
                }
                Json::text(out, (*this->genres)[index0]);
            }
            out += ']';
        }
        if (this->images.has_value()) {
            count = Json::key(out, count, "images");
            out += '[';
            for (std::size_t index0 = 0; index0 < (*this->images).size(); index0++) {
                if (index0 > 0) {
                    out += ',';
                }
                (*this->images)[index0].write(out);
            }
            out += ']';
        }
        if (this->in_list.has_value()) {
            count = Json::key(out, count, "in_list");
            out += (*this->in_list) ? "true" : "false";
        }
        if (this->more_like_this.has_value()) {
            count = Json::key(out, count, "more_like_this");
            out += '[';
            for (std::size_t index0 = 0; index0 < (*this->more_like_this).size(); index0++) {
                if (index0 > 0) {
                    out += ',';
                }
                (*this->more_like_this)[index0].write(out);
            }
            out += ']';
        }
        count = Json::key(out, count, "name");
        Json::text(out, this->name);
        if (this->overview.has_value()) {
            count = Json::key(out, count, "overview");
            Json::text(out, (*this->overview));
        }
        if (this->rating.has_value()) {
            count = Json::key(out, count, "rating");
            (*this->rating).write(out);
        }
        if (this->reminder_set.has_value()) {
            count = Json::key(out, count, "reminder_set");
            out += (*this->reminder_set) ? "true" : "false";
        }
        if (this->resume.has_value()) {
            count = Json::key(out, count, "resume");
            (*this->resume).write(out);
        }
        if (this->runtime_min.has_value()) {
            count = Json::key(out, count, "runtime_min");
            out += std::to_string((*this->runtime_min));
        }
        if (this->seasons.has_value()) {
            count = Json::key(out, count, "seasons");
            out += '[';
            for (std::size_t index0 = 0; index0 < (*this->seasons).size(); index0++) {
                if (index0 > 0) {
                    out += ',';
                }
                (*this->seasons)[index0].write(out);
            }
            out += ']';
        }
        count = Json::key(out, count, "title");
        this->title.write(out);
        if (this->year.has_value()) {
            count = Json::key(out, count, "year");
            out += std::to_string((*this->year));
        }
        out += '}';
    }
};

/// Titles a visitor may look at before signing in, most watched first. A series also carries pictures of its first episodes, so a television can show the title as a slideshow.
struct PublicTitles {
    std::vector<TitleDetail> titles;

    static PublicTitles read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "titles", "PublicTitles");
        return PublicTitles{
                Json::list<TitleDetail>(*raw0, [](const Json::Value& x0) { return TitleDetail::read(x0); })};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "titles");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->titles.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->titles[index0].write(out);
        }
        out += ']';
        out += '}';
    }
};

/// `/v1/ready`, for outside checks.
struct Ready {
    std::string build;
    std::int64_t data_age_s = 0;
    std::string data_hash;
    ServiceState finder;
    ServiceState index;

    static Ready read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "build", "Ready");
        const Json::Value* raw1 = Json::need(fields, "data_age_s", "Ready");
        const Json::Value* raw2 = Json::need(fields, "data_hash", "Ready");
        const Json::Value* raw3 = Json::need(fields, "finder", "Ready");
        const Json::Value* raw4 = Json::need(fields, "index", "Ready");
        return Ready{
                Json::as_text(*raw0),
                Json::as_long(*raw1),
                Json::as_text(*raw2),
                ServiceState::read(*raw3),
                ServiceState::read(*raw4)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "build");
        Json::text(out, this->build);
        count = Json::key(out, count, "data_age_s");
        out += std::to_string(this->data_age_s);
        count = Json::key(out, count, "data_hash");
        Json::text(out, this->data_hash);
        count = Json::key(out, count, "finder");
        this->finder.write(out);
        count = Json::key(out, count, "index");
        this->index.write(out);
        out += '}';
    }
};

struct RecoveryFinish {
    std::string code;
    std::string email;
    std::string new_password;

    static RecoveryFinish read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "code", "RecoveryFinish");
        const Json::Value* raw1 = Json::need(fields, "email", "RecoveryFinish");
        const Json::Value* raw2 = Json::need(fields, "new_password", "RecoveryFinish");
        return RecoveryFinish{
                Json::as_text(*raw0),
                Json::as_text(*raw1),
                Json::as_text(*raw2)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "code");
        Json::text(out, this->code);
        count = Json::key(out, count, "email");
        Json::text(out, this->email);
        count = Json::key(out, count, "new_password");
        Json::text(out, this->new_password);
        out += '}';
    }
};

struct RecoveryStart {
    std::string email;

    static RecoveryStart read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "email", "RecoveryStart");
        return RecoveryStart{
                Json::as_text(*raw0)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "email");
        Json::text(out, this->email);
        out += '}';
    }
};

struct RefreshRequest {
    std::string refresh_token;

    static RefreshRequest read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "refresh_token", "RefreshRequest");
        return RefreshRequest{
                Json::as_text(*raw0)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "refresh_token");
        Json::text(out, this->refresh_token);
        out += '}';
    }
};

/// Events after which a screen should be read again.
struct RefreshTrigger {
    std::string value;

    static constexpr std::string_view focus = "focus";
    static constexpr std::string_view sign_in = "sign_in";
    static constexpr std::string_view profile_change = "profile_change";
    static constexpr std::string_view list_change = "list_change";
    static constexpr std::string_view history_change = "history_change";

    static RefreshTrigger read(const Json::Value& document) {
        return RefreshTrigger{Json::as_text(document)};
    }

    void write(std::string& out) const {
        Json::text(out, this->value);
    }
};

/// `/v1/reports`.
struct Report {
    std::string build;
    std::string code;
    std::optional<std::string> detail;

    static Report read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "build", "Report");
        const Json::Value* raw1 = Json::need(fields, "code", "Report");
        const Json::Value* raw2 = Json::at(fields, "detail");
        return Report{
                Json::as_text(*raw0),
                Json::as_text(*raw1),
                raw2 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw2))};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "build");
        Json::text(out, this->build);
        count = Json::key(out, count, "code");
        Json::text(out, this->code);
        if (this->detail.has_value()) {
            count = Json::key(out, count, "detail");
            Json::text(out, (*this->detail));
        }
        out += '}';
    }
};

/// A built page.
struct Screen {
    /// Reads more sections through `/v1/more/{cursor}`.
    std::optional<std::string> cursor;
    ScreenName name;
    std::optional<std::vector<RefreshTrigger>> refresh_on;
    std::vector<Section> sections;
    /// Seconds this answer stays good.
    std::int64_t ttl_s = 0;

    static Screen read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "cursor");
        const Json::Value* raw1 = Json::need(fields, "name", "Screen");
        const Json::Value* raw2 = Json::at(fields, "refresh_on");
        const Json::Value* raw3 = Json::need(fields, "sections", "Screen");
        const Json::Value* raw4 = Json::need(fields, "ttl_s", "Screen");
        return Screen{
                raw0 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw0)),
                ScreenName::read(*raw1),
                raw2 == nullptr ? std::optional<std::vector<RefreshTrigger>>() : std::optional<std::vector<RefreshTrigger>>(Json::list<RefreshTrigger>(*raw2, [](const Json::Value& x0) { return RefreshTrigger::read(x0); })),
                Json::list<Section>(*raw3, [](const Json::Value& x0) { return Section::read(x0); }),
                Json::as_long(*raw4)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->cursor.has_value()) {
            count = Json::key(out, count, "cursor");
            Json::text(out, (*this->cursor));
        }
        count = Json::key(out, count, "name");
        this->name.write(out);
        if (this->refresh_on.has_value()) {
            count = Json::key(out, count, "refresh_on");
            out += '[';
            for (std::size_t index0 = 0; index0 < (*this->refresh_on).size(); index0++) {
                if (index0 > 0) {
                    out += ',';
                }
                (*this->refresh_on)[index0].write(out);
            }
            out += ']';
        }
        count = Json::key(out, count, "sections");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->sections.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->sections[index0].write(out);
        }
        out += ']';
        count = Json::key(out, count, "ttl_s");
        out += std::to_string(this->ttl_s);
        out += '}';
    }
};

/// One season with its episodes.
struct Season {
    std::vector<Episode> episodes;
    std::optional<std::string> name;
    std::int64_t number = 0;

    static Season read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "episodes", "Season");
        const Json::Value* raw1 = Json::at(fields, "name");
        const Json::Value* raw2 = Json::need(fields, "number", "Season");
        return Season{
                Json::list<Episode>(*raw0, [](const Json::Value& x0) { return Episode::read(x0); }),
                raw1 == nullptr ? std::optional<std::string>() : std::optional<std::string>(Json::as_text(*raw1)),
                Json::as_long(*raw2)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "episodes");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->episodes.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->episodes[index0].write(out);
        }
        out += ']';
        if (this->name.has_value()) {
            count = Json::key(out, count, "name");
            Json::text(out, (*this->name));
        }
        count = Json::key(out, count, "number");
        out += std::to_string(this->number);
        out += '}';
    }
};

struct SignOut {
    /// True ends every session of this account.
    bool everywhere = false;

    static SignOut read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "everywhere", "SignOut");
        return SignOut{
                Json::as_bool(*raw0)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "everywhere");
        out += this->everywhere ? "true" : "false";
        out += '}';
    }
};

struct SignUp {
    std::string email;
    std::string first_name;
    std::string last_name;
    std::string password;

    static SignUp read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "email", "SignUp");
        const Json::Value* raw1 = Json::need(fields, "first_name", "SignUp");
        const Json::Value* raw2 = Json::need(fields, "last_name", "SignUp");
        const Json::Value* raw3 = Json::need(fields, "password", "SignUp");
        return SignUp{
                Json::as_text(*raw0),
                Json::as_text(*raw1),
                Json::as_text(*raw2),
                Json::as_text(*raw3)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "email");
        Json::text(out, this->email);
        count = Json::key(out, count, "first_name");
        Json::text(out, this->first_name);
        count = Json::key(out, count, "last_name");
        Json::text(out, this->last_name);
        count = Json::key(out, count, "password");
        Json::text(out, this->password);
        out += '}';
    }
};

/// `/v1/strings/{lang}`. Text holds the placeholders below.
struct Strings {
    /// Which way this language runs. Absent from older servers, which only spoke left to right.
    std::optional<TextDirection> dir;
    std::string lang;
    std::map<std::string, std::string> texts;

    static Strings read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "dir");
        const Json::Value* raw1 = Json::need(fields, "lang", "Strings");
        const Json::Value* raw2 = Json::need(fields, "texts", "Strings");
        return Strings{
                raw0 == nullptr ? std::optional<TextDirection>() : std::optional<TextDirection>(TextDirection::read(*raw0)),
                Json::as_text(*raw1),
                Json::map<std::string>(*raw2, [](const Json::Value& x0) { return Json::as_text(x0); })};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->dir.has_value()) {
            count = Json::key(out, count, "dir");
            (*this->dir).write(out);
        }
        count = Json::key(out, count, "lang");
        Json::text(out, this->lang);
        count = Json::key(out, count, "texts");
        out += '{';
        int n0 = 0;
        for (const auto& entry0 : this->texts) {
            n0 = Json::key(out, n0, entry0.first);
            Json::text(out, entry0.second);
        }
        out += '}';
        out += '}';
    }
};

/// `/v1/subtitles/{file}`. The core checks `last_end_ms` against the driver's duration.
struct SubtitleBody {
    std::vector<Cue> cues;
    std::int64_t last_end_ms = 0;

    static SubtitleBody read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "cues", "SubtitleBody");
        const Json::Value* raw1 = Json::need(fields, "last_end_ms", "SubtitleBody");
        return SubtitleBody{
                Json::list<Cue>(*raw0, [](const Json::Value& x0) { return Cue::read(x0); }),
                Json::as_long(*raw1)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "cues");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->cues.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->cues[index0].write(out);
        }
        out += ']';
        count = Json::key(out, count, "last_end_ms");
        out += std::to_string(this->last_end_ms);
        out += '}';
    }
};

/// One subtitle file of a language, in the order to try.
struct SubtitleFile {
    std::optional<bool> hearing_impaired;
    std::string id;

    static SubtitleFile read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::at(fields, "hearing_impaired");
        const Json::Value* raw1 = Json::need(fields, "id", "SubtitleFile");
        return SubtitleFile{
                raw0 == nullptr ? std::optional<bool>() : std::optional<bool>(Json::as_bool(*raw0)),
                Json::as_text(*raw1)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        if (this->hearing_impaired.has_value()) {
            count = Json::key(out, count, "hearing_impaired");
            out += (*this->hearing_impaired) ? "true" : "false";
        }
        count = Json::key(out, count, "id");
        Json::text(out, this->id);
        out += '}';
    }
};

/// One language.
struct SubtitleLanguage {
    std::vector<SubtitleFile> files;
    std::string label;
    std::string lang;

    static SubtitleLanguage read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "files", "SubtitleLanguage");
        const Json::Value* raw1 = Json::need(fields, "label", "SubtitleLanguage");
        const Json::Value* raw2 = Json::need(fields, "lang", "SubtitleLanguage");
        return SubtitleLanguage{
                Json::list<SubtitleFile>(*raw0, [](const Json::Value& x0) { return SubtitleFile::read(x0); }),
                Json::as_text(*raw1),
                Json::as_text(*raw2)};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "files");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->files.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->files[index0].write(out);
        }
        out += ']';
        count = Json::key(out, count, "label");
        Json::text(out, this->label);
        count = Json::key(out, count, "lang");
        Json::text(out, this->lang);
        out += '}';
    }
};

/// `/v1/subtitles`.
struct SubtitleList {
    std::vector<SubtitleLanguage> languages;

    static SubtitleList read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "languages", "SubtitleList");
        return SubtitleList{
                Json::list<SubtitleLanguage>(*raw0, [](const Json::Value& x0) { return SubtitleLanguage::read(x0); })};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "languages");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->languages.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            this->languages[index0].write(out);
        }
        out += ']';
        out += '}';
    }
};

/// Search suggestions.
struct Suggestions {
    std::vector<std::string> items;

    static Suggestions read(const Json::Value& document) {
        const Json::Value& fields = Json::object(document);
        const Json::Value* raw0 = Json::need(fields, "items", "Suggestions");
        return Suggestions{
                Json::list<std::string>(*raw0, [](const Json::Value& x0) { return Json::as_text(x0); })};
    }

    void write(std::string& out) const {
        out += '{';
        int count = 0;
        count = Json::key(out, count, "items");
        out += '[';
        for (std::size_t index0 = 0; index0 < this->items.size(); index0++) {
            if (index0 > 0) {
                out += ',';
            }
            Json::text(out, this->items[index0]);
        }
        out += ']';
        out += '}';
    }
};

}  // namespace contract

#endif  // STREAMS_CONTRACT_HPP
