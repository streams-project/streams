// Generated from the contract. Do not edit.

package contract;

import java.util.ArrayList;
import java.util.Collections;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.function.Function;
import java.util.regex.Pattern;

/**
 * Every type the contract publishes, with protobuf shaped accessor names so
 * note 204's P5 can swap protoc's output in without touching a call site.
 * Read a value through an accessor; construction stays in :core and :ports.
 */
public final class Contract {
    private Contract() {
    }

    public static final String WIRE_VERSION = "2026-11-05";
    public static final int ABI_VERSION = 1;

    /** Every address the apps open. */
    public static final class Address {
        private Address() {
        }

        public static final String START = "/";
        public static final String HOME = "/browse";
        public static final String KIDS = "/kids";
        public static final String SERIES = "/browse/genre/83";
        public static final String FILMS = "/browse/genre/34399";
        public static final String NEW = "/latest";
        public static final String GAMES = "/games";
        public static final String MY_LIST = "/browse/my-list";
        public static final String MINE = "/mine";
        public static final String LANGUAGE = "/browse/original-audio";
        public static final String SEARCH = "/search";
        public static final String NOTIFICATIONS = "/notifications";
        public static final String ACTIVITY = "/settings/viewed";
        public static final String CLIPS = "/clips";
        public static final String HELP = "/help";
        public static final String SETTINGS = "/settings";
        public static final String PROFILES = "/profiles";
        public static final String SIGN_IN = "/login";
        public static final String ACTIVATE = "/activate";
        public static final String GENRE = "/browse/genre/";
        public static final String TITLE = "/title/";
        public static final String WATCH = "/watch/";

        /** Where a tab lives, by the screen's word. */
        public static String of(String screen) {
            switch (screen) {
                case "home":
                    return "/browse";
                case "kids":
                    return "/kids";
                case "series":
                    return "/browse/genre/83";
                case "films":
                    return "/browse/genre/34399";
                case "new":
                    return "/latest";
                case "games":
                    return "/games";
                case "my-list":
                    return "/browse/my-list";
                case "mine":
                    return "/mine";
                case "language":
                    return "/browse/original-audio";
                case "search":
                    return "/search";
                case "notifications":
                    return "/notifications";
                case "activity":
                    return "/settings/viewed";
                case "clips":
                    return "/clips";
                case "help":
                    return "/help";
                case "settings":
                    return "/settings";
                default:
                    return HOME;
            }
        }
    }

    /** What every generated type can do: write itself back as JSON. */
    public interface Wire {
        void write(StringBuilder out);
    }

    /**
     * A JSON reader and writer that depends on nothing but the JDK, because Java has no
     * kotlinx.serialization and `org.json` is in `android.jar` but not in a desktop JDK, which
     * would leave the Java arm of check_types.py unable to run.
     */
    public static final class Json {
        private final String text;
        private int at;

        private Json(String text) {
            this.text = text;
        }

        /** A document as Map, List, String, Long, Double, Boolean or null. */
        public static Object parse(String text) {
            Json json = new Json(text);
            json.space();
            Object found = json.value();
            json.space();
            if (json.at != text.length()) {
                throw new IllegalArgumentException("trailing text at " + json.at);
            }
            return found;
        }

        /** What a value writes itself as. */
        public static String render(Wire value) {
            StringBuilder out = new StringBuilder();
            value.write(out);
            return out.toString();
        }

        private void space() {
            while (at < text.length()) {
                char here = text.charAt(at);
                if (here != ' ' && here != '\t' && here != '\n' && here != '\r') {
                    return;
                }
                at++;
            }
        }

        private Object value() {
            if (at >= text.length()) {
                throw new IllegalArgumentException("no value at " + at);
            }
            char head = text.charAt(at);
            if (head == '{') {
                return readObject();
            }
            if (head == '[') {
                return readArray();
            }
            if (head == '"') {
                return readText();
            }
            if (head == 't') {
                word("true");
                return Boolean.TRUE;
            }
            if (head == 'f') {
                word("false");
                return Boolean.FALSE;
            }
            if (head == 'n') {
                word("null");
                return null;
            }
            return readNumber();
        }

        private void word(String wanted) {
            if (!text.startsWith(wanted, at)) {
                throw new IllegalArgumentException("not " + wanted + " at " + at);
            }
            at += wanted.length();
        }

        private Map<String, Object> readObject() {
            Map<String, Object> found = new LinkedHashMap<String, Object>();
            at++;
            space();
            if (at < text.length() && text.charAt(at) == '}') {
                at++;
                return found;
            }
            while (true) {
                space();
                String key = readText();
                space();
                if (at >= text.length() || text.charAt(at) != ':') {
                    throw new IllegalArgumentException("no colon at " + at);
                }
                at++;
                space();
                found.put(key, value());
                space();
                if (at >= text.length()) {
                    throw new IllegalArgumentException("unclosed object");
                }
                char next = text.charAt(at++);
                if (next == '}') {
                    return found;
                }
                if (next != ',') {
                    throw new IllegalArgumentException("no comma at " + (at - 1));
                }
            }
        }

        private List<Object> readArray() {
            List<Object> found = new ArrayList<Object>();
            at++;
            space();
            if (at < text.length() && text.charAt(at) == ']') {
                at++;
                return found;
            }
            while (true) {
                space();
                found.add(value());
                space();
                if (at >= text.length()) {
                    throw new IllegalArgumentException("unclosed list");
                }
                char next = text.charAt(at++);
                if (next == ']') {
                    return found;
                }
                if (next != ',') {
                    throw new IllegalArgumentException("no comma at " + (at - 1));
                }
            }
        }

        private String readText() {
            if (at >= text.length() || text.charAt(at) != '"') {
                throw new IllegalArgumentException("no text at " + at);
            }
            at++;
            StringBuilder out = new StringBuilder();
            while (true) {
                if (at >= text.length()) {
                    throw new IllegalArgumentException("unclosed text");
                }
                char here = text.charAt(at++);
                if (here == '"') {
                    return out.toString();
                }
                if (here != '\\') {
                    out.append(here);
                    continue;
                }
                if (at >= text.length()) {
                    throw new IllegalArgumentException("unclosed escape");
                }
                char mark = text.charAt(at++);
                if (mark == 'u') {
                    out.append((char) Integer.parseInt(text.substring(at, at + 4), 16));
                    at += 4;
                } else if (mark == 'n') {
                    out.append('\n');
                } else if (mark == 't') {
                    out.append('\t');
                } else if (mark == 'r') {
                    out.append('\r');
                } else if (mark == 'b') {
                    out.append('\b');
                } else if (mark == 'f') {
                    out.append('\f');
                } else if (mark == '"' || mark == '\\' || mark == '/') {
                    out.append(mark);
                } else {
                    throw new IllegalArgumentException("bad escape at " + (at - 1));
                }
            }
        }

        private Object readNumber() {
            int from = at;
            while (at < text.length() && "+-.eE0123456789".indexOf(text.charAt(at)) >= 0) {
                at++;
            }
            String found = text.substring(from, at);
            if (found.isEmpty()) {
                throw new IllegalArgumentException("no value at " + from);
            }
            if (found.indexOf('.') < 0 && found.indexOf('e') < 0 && found.indexOf('E') < 0) {
                return Long.valueOf(Long.parseLong(found));
            }
            return Double.valueOf(Double.parseDouble(found));
        }

        /** The object a message is read from. */
        @SuppressWarnings("unchecked")
        public static Map<String, Object> object(Object value) {
            if (value instanceof Map) {
                return (Map<String, Object>) value;
            }
            throw new IllegalArgumentException("not an object: " + value);
        }

        /** A field that must be there. A written null counts as absent, as it does in Kotlin. */
        public static Object need(Map<String, Object> from, String key, String type) {
            Object found = from.get(key);
            if (found == null) {
                throw new IllegalArgumentException(type + " has no " + key);
            }
            return found;
        }

        /** A field that may be absent. */
        public static Object at(Map<String, Object> from, String key) {
            return from.get(key);
        }

        public static String asText(Object value) {
            if (value instanceof String) {
                return (String) value;
            }
            throw new IllegalArgumentException("not text: " + value);
        }

        public static long asLong(Object value) {
            if (value instanceof Long) {
                return ((Long) value).longValue();
            }
            throw new IllegalArgumentException("not a whole number: " + value);
        }

        public static boolean asBool(Object value) {
            if (value instanceof Boolean) {
                return ((Boolean) value).booleanValue();
            }
            throw new IllegalArgumentException("not a boolean: " + value);
        }

        public static <T> List<T> list(Object value, Function<Object, T> each) {
            if (!(value instanceof List)) {
                throw new IllegalArgumentException("not a list: " + value);
            }
            List<?> from = (List<?>) value;
            List<T> found = new ArrayList<T>(from.size());
            for (Object item : from) {
                found.add(each.apply(item));
            }
            return Collections.unmodifiableList(found);
        }

        public static <T> Map<String, T> map(Object value, Function<Object, T> each) {
            Map<String, T> found = new LinkedHashMap<String, T>();
            for (Map.Entry<String, Object> entry : object(value).entrySet()) {
                found.put(entry.getKey(), each.apply(entry.getValue()));
            }
            return Collections.unmodifiableMap(found);
        }

        /** Opens a field, with the comma the field before it needs. */
        public static int key(StringBuilder out, int written, String name) {
            if (written > 0) {
                out.append(',');
            }
            text(out, name);
            out.append(':');
            return written + 1;
        }

        /** A JSON string literal. */
        public static void text(StringBuilder out, String value) {
            out.append('"');
            for (int index = 0; index < value.length(); index++) {
                char here = value.charAt(index);
                if (here == '"' || here == '\\') {
                    out.append('\\').append(here);
                } else if (here == '\n') {
                    out.append("\\n");
                } else if (here == '\r') {
                    out.append("\\r");
                } else if (here == '\t') {
                    out.append("\\t");
                } else if (here < 0x20) {
                    out.append(String.format("\\u%04x", Integer.valueOf(here)));
                } else {
                    out.append(here);
                }
            }
            out.append('"');
        }
    }

    public static final class Account implements Wire {
        private final String email;
        private final String firstName;
        private final String id;
        private final String lastName;
        private final String phone;

        public Account(String email, String firstName, String id, String lastName, String phone) {
            this.email = email;
            this.firstName = firstName;
            this.id = id;
            this.lastName = lastName;
            this.phone = phone;
        }

        public String getEmail() {
            return email;
        }

        public String getFirstName() {
            return firstName;
        }

        public String getId() {
            return id;
        }

        public String getLastName() {
            return lastName;
        }

        public boolean hasPhone() {
            return phone != null;
        }

        public String getPhone() {
            return phone == null ? "" : phone;
        }

        public static Account read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "email", "Account");
            Object $raw1 = Json.need($fields, "first_name", "Account");
            Object $raw2 = Json.need($fields, "id", "Account");
            Object $raw3 = Json.need($fields, "last_name", "Account");
            Object $raw4 = Json.at($fields, "phone");
            return new Account(
                    Json.asText($raw0),
                    Json.asText($raw1),
                    Json.asText($raw2),
                    Json.asText($raw3),
                    $raw4 == null ? null : Json.asText($raw4));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "email");
            Json.text($out, email);
            $count = Json.key($out, $count, "first_name");
            Json.text($out, firstName);
            $count = Json.key($out, $count, "id");
            Json.text($out, id);
            $count = Json.key($out, $count, "last_name");
            Json.text($out, lastName);
            if (phone != null) {
                $count = Json.key($out, $count, "phone");
                Json.text($out, phone);
            }
            $out.append('}');
        }
    }

    /** Everything held about an account, in plain JSON. */
    public static final class AccountExport implements Wire {
        private final Account account;
        private final List<DeviceSession> devices;
        private final List<Profile> profiles;

        public AccountExport(Account account, List<DeviceSession> devices, List<Profile> profiles) {
            this.account = account;
            this.devices = devices;
            this.profiles = profiles;
        }

        public Account getAccount() {
            return account;
        }

        public List<DeviceSession> getDevicesList() {
            return devices == null ? Collections.<DeviceSession>emptyList() : devices;
        }

        public int getDevicesCount() {
            return devices == null ? 0 : devices.size();
        }

        public DeviceSession getDevices(int $index) {
            return devices.get($index);
        }

        public List<Profile> getProfilesList() {
            return profiles == null ? Collections.<Profile>emptyList() : profiles;
        }

        public int getProfilesCount() {
            return profiles == null ? 0 : profiles.size();
        }

        public Profile getProfiles(int $index) {
            return profiles.get($index);
        }

        public static AccountExport read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "account", "AccountExport");
            Object $raw1 = Json.need($fields, "devices", "AccountExport");
            Object $raw2 = Json.need($fields, "profiles", "AccountExport");
            return new AccountExport(
                    Account.read($raw0),
                    Json.list($raw1, $x0 -> DeviceSession.read($x0)),
                    Json.list($raw2, $x0 -> Profile.read($x0)));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "account");
            account.write($out);
            $count = Json.key($out, $count, "devices");
            $out.append('[');
            for (int $i0 = 0; $i0 < devices.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                devices.get($i0).write($out);
            }
            $out.append(']');
            $count = Json.key($out, $count, "profiles");
            $out.append('[');
            for (int $i0 = 0; $i0 < profiles.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                profiles.get($i0).write($out);
            }
            $out.append(']');
            $out.append('}');
        }
    }

    public static final class AccountUpdate implements Wire {
        private final String firstName;
        private final String lastName;

        public AccountUpdate(String firstName, String lastName) {
            this.firstName = firstName;
            this.lastName = lastName;
        }

        public boolean hasFirstName() {
            return firstName != null;
        }

        public String getFirstName() {
            return firstName == null ? "" : firstName;
        }

        public boolean hasLastName() {
            return lastName != null;
        }

        public String getLastName() {
            return lastName == null ? "" : lastName;
        }

        public static AccountUpdate read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "first_name");
            Object $raw1 = Json.at($fields, "last_name");
            return new AccountUpdate(
                    $raw0 == null ? null : Json.asText($raw0),
                    $raw1 == null ? null : Json.asText($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (firstName != null) {
                $count = Json.key($out, $count, "first_name");
                Json.text($out, firstName);
            }
            if (lastName != null) {
                $count = Json.key($out, $count, "last_name");
                Json.text($out, lastName);
            }
            $out.append('}');
        }
    }

    /** What a screen asks for. */
    public abstract static class Action implements Wire {
        public static Action read(Object $json) {
            String $tag = Json.asText(Json.need(Json.object($json), "action", "Action"));
            if ("sign_in".equals($tag)) {
                return ActionSignIn.read($json);
            }
            if ("send_code".equals($tag)) {
                return ActionSendCode.read($json);
            }
            if ("sign_in_code".equals($tag)) {
                return ActionSignInCode.read($json);
            }
            if ("device_sign_in".equals($tag)) {
                return ActionDeviceSignIn.read($json);
            }
            if ("device_sign_in_stop".equals($tag)) {
                return ActionDeviceSignInStop.read($json);
            }
            if ("sign_out".equals($tag)) {
                return ActionSignOut.read($json);
            }
            if ("enter_profile".equals($tag)) {
                return ActionEnterProfile.read($json);
            }
            if ("play".equals($tag)) {
                return ActionPlay.read($json);
            }
            if ("pause".equals($tag)) {
                return ActionPause.read($json);
            }
            if ("unpause".equals($tag)) {
                return ActionUnpause.read($json);
            }
            if ("seek".equals($tag)) {
                return ActionSeek.read($json);
            }
            if ("select_audio".equals($tag)) {
                return ActionSelectAudio.read($json);
            }
            if ("select_text".equals($tag)) {
                return ActionSelectText.read($json);
            }
            if ("next_subtitle_file".equals($tag)) {
                return ActionNextSubtitleFile.read($json);
            }
            if ("stop".equals($tag)) {
                return ActionStop.read($json);
            }
            if ("command".equals($tag)) {
                return ActionCommand.read($json);
            }
            if ("request".equals($tag)) {
                return ActionRequest.read($json);
            }
            return ActionUnknown.read($json);
        }
    }

    public static final class ActionSignIn extends Action {
        private final String email;
        private final String password;

        public ActionSignIn(String email, String password) {
            this.email = email;
            this.password = password;
        }

        public String getEmail() {
            return email;
        }

        public String getPassword() {
            return password;
        }

        public static ActionSignIn read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"sign_in".equals(Json.asText(Json.need($fields, "action", "Action")))) {
                throw new IllegalArgumentException("not a ActionSignIn");
            }
            Object $raw0 = Json.need($fields, "email", "Action");
            Object $raw1 = Json.need($fields, "password", "Action");
            return new ActionSignIn(
                    Json.asText($raw0),
                    Json.asText($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "action");
            Json.text($out, "sign_in");
            $count = Json.key($out, $count, "email");
            Json.text($out, email);
            $count = Json.key($out, $count, "password");
            Json.text($out, password);
            $out.append('}');
        }
    }

    public static final class ActionSendCode extends Action {
        private final String email;

        public ActionSendCode(String email) {
            this.email = email;
        }

        public String getEmail() {
            return email;
        }

        public static ActionSendCode read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"send_code".equals(Json.asText(Json.need($fields, "action", "Action")))) {
                throw new IllegalArgumentException("not a ActionSendCode");
            }
            Object $raw0 = Json.need($fields, "email", "Action");
            return new ActionSendCode(
                    Json.asText($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "action");
            Json.text($out, "send_code");
            $count = Json.key($out, $count, "email");
            Json.text($out, email);
            $out.append('}');
        }
    }

    public static final class ActionSignInCode extends Action {
        private final String code;
        private final String email;

        public ActionSignInCode(String code, String email) {
            this.code = code;
            this.email = email;
        }

        public String getCode() {
            return code;
        }

        public String getEmail() {
            return email;
        }

        public static ActionSignInCode read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"sign_in_code".equals(Json.asText(Json.need($fields, "action", "Action")))) {
                throw new IllegalArgumentException("not a ActionSignInCode");
            }
            Object $raw0 = Json.need($fields, "code", "Action");
            Object $raw1 = Json.need($fields, "email", "Action");
            return new ActionSignInCode(
                    Json.asText($raw0),
                    Json.asText($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "action");
            Json.text($out, "sign_in_code");
            $count = Json.key($out, $count, "code");
            Json.text($out, code);
            $count = Json.key($out, $count, "email");
            Json.text($out, email);
            $out.append('}');
        }
    }

    /** Start device sign in. The core asks for a code, polls and renews it, and the screen draws `View.device_sign_in`. While a code is live this changes nothing. */
    public static final class ActionDeviceSignIn extends Action {
        private final String deviceName;

        public ActionDeviceSignIn(String deviceName) {
            this.deviceName = deviceName;
        }

        public String getDeviceName() {
            return deviceName;
        }

        public static ActionDeviceSignIn read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"device_sign_in".equals(Json.asText(Json.need($fields, "action", "Action")))) {
                throw new IllegalArgumentException("not a ActionDeviceSignIn");
            }
            Object $raw0 = Json.need($fields, "device_name", "Action");
            return new ActionDeviceSignIn(
                    Json.asText($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "action");
            Json.text($out, "device_sign_in");
            $count = Json.key($out, $count, "device_name");
            Json.text($out, deviceName);
            $out.append('}');
        }
    }

    public static final class ActionDeviceSignInStop extends Action {
        public ActionDeviceSignInStop() {
        }

        public static ActionDeviceSignInStop read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"device_sign_in_stop".equals(Json.asText(Json.need($fields, "action", "Action")))) {
                throw new IllegalArgumentException("not a ActionDeviceSignInStop");
            }
            return new ActionDeviceSignInStop();
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "action");
            Json.text($out, "device_sign_in_stop");
            $out.append('}');
        }
    }

    public static final class ActionSignOut extends Action {
        private final boolean everywhere;

        public ActionSignOut(boolean everywhere) {
            this.everywhere = everywhere;
        }

        public boolean getEverywhere() {
            return everywhere;
        }

        public static ActionSignOut read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"sign_out".equals(Json.asText(Json.need($fields, "action", "Action")))) {
                throw new IllegalArgumentException("not a ActionSignOut");
            }
            Object $raw0 = Json.need($fields, "everywhere", "Action");
            return new ActionSignOut(
                    Json.asBool($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "action");
            Json.text($out, "sign_out");
            $count = Json.key($out, $count, "everywhere");
            $out.append(everywhere);
            $out.append('}');
        }
    }

    public static final class ActionEnterProfile extends Action {
        private final String pin;
        private final String profile;

        public ActionEnterProfile(String pin, String profile) {
            this.pin = pin;
            this.profile = profile;
        }

        public boolean hasPin() {
            return pin != null;
        }

        public String getPin() {
            return pin == null ? "" : pin;
        }

        public String getProfile() {
            return profile;
        }

        public static ActionEnterProfile read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"enter_profile".equals(Json.asText(Json.need($fields, "action", "Action")))) {
                throw new IllegalArgumentException("not a ActionEnterProfile");
            }
            Object $raw0 = Json.at($fields, "pin");
            Object $raw1 = Json.need($fields, "profile", "Action");
            return new ActionEnterProfile(
                    $raw0 == null ? null : Json.asText($raw0),
                    Json.asText($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "action");
            Json.text($out, "enter_profile");
            if (pin != null) {
                $count = Json.key($out, $count, "pin");
                Json.text($out, pin);
            }
            $count = Json.key($out, $count, "profile");
            Json.text($out, profile);
            $out.append('}');
        }
    }

    public static final class ActionPlay extends Action {
        private final EpisodeRef episode;
        private final TitleKey title;

        public ActionPlay(EpisodeRef episode, TitleKey title) {
            this.episode = episode;
            this.title = title;
        }

        public boolean hasEpisode() {
            return episode != null;
        }

        public EpisodeRef getEpisode() {
            return episode == null ? null : episode;
        }

        public TitleKey getTitle() {
            return title;
        }

        public static ActionPlay read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"play".equals(Json.asText(Json.need($fields, "action", "Action")))) {
                throw new IllegalArgumentException("not a ActionPlay");
            }
            Object $raw0 = Json.at($fields, "episode");
            Object $raw1 = Json.need($fields, "title", "Action");
            return new ActionPlay(
                    $raw0 == null ? null : EpisodeRef.read($raw0),
                    TitleKey.read($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "action");
            Json.text($out, "play");
            if (episode != null) {
                $count = Json.key($out, $count, "episode");
                episode.write($out);
            }
            $count = Json.key($out, $count, "title");
            title.write($out);
            $out.append('}');
        }
    }

    public static final class ActionPause extends Action {
        public ActionPause() {
        }

        public static ActionPause read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"pause".equals(Json.asText(Json.need($fields, "action", "Action")))) {
                throw new IllegalArgumentException("not a ActionPause");
            }
            return new ActionPause();
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "action");
            Json.text($out, "pause");
            $out.append('}');
        }
    }

    public static final class ActionUnpause extends Action {
        public ActionUnpause() {
        }

        public static ActionUnpause read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"unpause".equals(Json.asText(Json.need($fields, "action", "Action")))) {
                throw new IllegalArgumentException("not a ActionUnpause");
            }
            return new ActionUnpause();
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "action");
            Json.text($out, "unpause");
            $out.append('}');
        }
    }

    public static final class ActionSeek extends Action {
        private final long toMs;

        public ActionSeek(long toMs) {
            this.toMs = toMs;
        }

        public long getToMs() {
            return toMs;
        }

        public static ActionSeek read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"seek".equals(Json.asText(Json.need($fields, "action", "Action")))) {
                throw new IllegalArgumentException("not a ActionSeek");
            }
            Object $raw0 = Json.need($fields, "to_ms", "Action");
            return new ActionSeek(
                    Json.asLong($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "action");
            Json.text($out, "seek");
            $count = Json.key($out, $count, "to_ms");
            $out.append(toMs);
            $out.append('}');
        }
    }

    public static final class ActionSelectAudio extends Action {
        private final String track;

        public ActionSelectAudio(String track) {
            this.track = track;
        }

        public String getTrack() {
            return track;
        }

        public static ActionSelectAudio read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"select_audio".equals(Json.asText(Json.need($fields, "action", "Action")))) {
                throw new IllegalArgumentException("not a ActionSelectAudio");
            }
            Object $raw0 = Json.need($fields, "track", "Action");
            return new ActionSelectAudio(
                    Json.asText($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "action");
            Json.text($out, "select_audio");
            $count = Json.key($out, $count, "track");
            Json.text($out, track);
            $out.append('}');
        }
    }

    public static final class ActionSelectText extends Action {
        private final String track;

        public ActionSelectText(String track) {
            this.track = track;
        }

        public boolean hasTrack() {
            return track != null;
        }

        public String getTrack() {
            return track == null ? "" : track;
        }

        public static ActionSelectText read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"select_text".equals(Json.asText(Json.need($fields, "action", "Action")))) {
                throw new IllegalArgumentException("not a ActionSelectText");
            }
            Object $raw0 = Json.at($fields, "track");
            return new ActionSelectText(
                    $raw0 == null ? null : Json.asText($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "action");
            Json.text($out, "select_text");
            if (track != null) {
                $count = Json.key($out, $count, "track");
                Json.text($out, track);
            }
            $out.append('}');
        }
    }

    public static final class ActionNextSubtitleFile extends Action {
        public ActionNextSubtitleFile() {
        }

        public static ActionNextSubtitleFile read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"next_subtitle_file".equals(Json.asText(Json.need($fields, "action", "Action")))) {
                throw new IllegalArgumentException("not a ActionNextSubtitleFile");
            }
            return new ActionNextSubtitleFile();
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "action");
            Json.text($out, "next_subtitle_file");
            $out.append('}');
        }
    }

    public static final class ActionStop extends Action {
        public ActionStop() {
        }

        public static ActionStop read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"stop".equals(Json.asText(Json.need($fields, "action", "Action")))) {
                throw new IllegalArgumentException("not a ActionStop");
            }
            return new ActionStop();
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "action");
            Json.text($out, "stop");
            $out.append('}');
        }
    }

    public static final class ActionCommand extends Action {
        private final Command command;

        public ActionCommand(Command command) {
            this.command = command;
        }

        public Command getCommand() {
            return command;
        }

        public static ActionCommand read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"command".equals(Json.asText(Json.need($fields, "action", "Action")))) {
                throw new IllegalArgumentException("not a ActionCommand");
            }
            Object $raw0 = Json.need($fields, "command", "Action");
            return new ActionCommand(
                    Command.read($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "action");
            Json.text($out, "command");
            $count = Json.key($out, $count, "command");
            command.write($out);
            $out.append('}');
        }
    }

    /** Send a prepared request with this session's token. The http effect takes the id of this event and its class is always plain, so whoever prepared it gets the answer under that id. Tokens stay in the core. */
    public static final class ActionRequest extends Action {
        private final String body;
        private final HttpMethod method;
        private final String path;

        public ActionRequest(String body, HttpMethod method, String path) {
            this.body = body;
            this.method = method;
            this.path = path;
        }

        public boolean hasBody() {
            return body != null;
        }

        public String getBody() {
            return body == null ? "" : body;
        }

        public HttpMethod getMethod() {
            return method;
        }

        public String getPath() {
            return path;
        }

        public static ActionRequest read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"request".equals(Json.asText(Json.need($fields, "action", "Action")))) {
                throw new IllegalArgumentException("not a ActionRequest");
            }
            Object $raw0 = Json.at($fields, "body");
            Object $raw1 = Json.need($fields, "method", "Action");
            Object $raw2 = Json.need($fields, "path", "Action");
            return new ActionRequest(
                    $raw0 == null ? null : Json.asText($raw0),
                    HttpMethod.read($raw1),
                    Json.asText($raw2));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "action");
            Json.text($out, "request");
            if (body != null) {
                $count = Json.key($out, $count, "body");
                Json.text($out, body);
            }
            $count = Json.key($out, $count, "method");
            method.write($out);
            $count = Json.key($out, $count, "path");
            Json.text($out, path);
            $out.append('}');
        }
    }

    /** A tag this build does not know. */
    public static final class ActionUnknown extends Action {
        private final String tag;

        public ActionUnknown(String tag) {
            this.tag = tag;
        }

        /** The tag that was read. */
        public String getTag() {
            return tag;
        }

        public static ActionUnknown read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "action");
            return new ActionUnknown(
                    $raw0 == null ? null : Json.asText($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "action");
            Json.text($out, tag);
            $out.append('}');
        }
    }

    /** Age card shown once at the start of play. */
    public static final class AgeCard implements Wire {
        private final String label;
        private final List<String> reasons;

        public AgeCard(String label, List<String> reasons) {
            this.label = label;
            this.reasons = reasons;
        }

        public String getLabel() {
            return label;
        }

        public List<String> getReasonsList() {
            return reasons == null ? Collections.<String>emptyList() : reasons;
        }

        public int getReasonsCount() {
            return reasons == null ? 0 : reasons.size();
        }

        public String getReasons(int $index) {
            return reasons.get($index);
        }

        public static AgeCard read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "label", "AgeCard");
            Object $raw1 = Json.at($fields, "reasons");
            return new AgeCard(
                    Json.asText($raw0),
                    $raw1 == null ? null : Json.list($raw1, $x0 -> Json.asText($x0)));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "label");
            Json.text($out, label);
            if (reasons != null) {
                $count = Json.key($out, $count, "reasons");
                $out.append('[');
                for (int $i0 = 0; $i0 < reasons.size(); $i0++) {
                    if ($i0 > 0) {
                        $out.append(',');
                    }
                    Json.text($out, reasons.get($i0));
                }
                $out.append(']');
            }
            $out.append('}');
        }
    }

    /** An age mark, as this viewer reads it. Each screen picks its shape. */
    public static final class AgeMark implements Wire {
        private final String label;
        private final AgeTone tone;

        public AgeMark(String label, AgeTone tone) {
            this.label = label;
            this.tone = tone;
        }

        public String getLabel() {
            return label;
        }

        public AgeTone getTone() {
            return tone;
        }

        public static AgeMark read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "label", "AgeMark");
            Object $raw1 = Json.need($fields, "tone", "AgeMark");
            return new AgeMark(
                    Json.asText($raw0),
                    AgeTone.read($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "label");
            Json.text($out, label);
            $count = Json.key($out, $count, "tone");
            tone.write($out);
            $out.append('}');
        }
    }

    /** The colour family of an age mark. The server picks it and each app paints it. */
    public static final class AgeTone implements Wire {
        public static final AgeTone GREEN = new AgeTone("green");
        public static final AgeTone YELLOW = new AgeTone("yellow");
        public static final AgeTone ORANGE = new AgeTone("orange");
        public static final AgeTone PINK = new AgeTone("pink");
        public static final AgeTone RED = new AgeTone("red");
        public static final AgeTone DARK_RED = new AgeTone("dark_red");
        public static final AgeTone BLUE = new AgeTone("blue");
        public static final AgeTone GREY = new AgeTone("grey");

        private final String value;

        public AgeTone(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static AgeTone read(Object $json) {
            return new AgeTone(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof AgeTone && (((AgeTone) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** One way to play. Attempts arrive in the order to try, ranked once on the server. `headers` is required, even when empty, and no provider name is carried. */
    public static final class Attempt implements Wire {
        private final List<String> audio;
        private final List<String> codecs;
        private final String container;
        private final Long expiresAtMs;
        private final String group;
        private final Map<String, String> headers;
        private final Long height;
        private final String id;
        private final AttemptKind kind;
        private final String url;
        private final boolean verified;

        public Attempt(List<String> audio, List<String> codecs, String container, Long expiresAtMs, String group, Map<String, String> headers, Long height, String id, AttemptKind kind, String url, boolean verified) {
            this.audio = audio;
            this.codecs = codecs;
            this.container = container;
            this.expiresAtMs = expiresAtMs;
            this.group = group;
            this.headers = headers;
            this.height = height;
            this.id = id;
            this.kind = kind;
            this.url = url;
            this.verified = verified;
        }

        public List<String> getAudioList() {
            return audio == null ? Collections.<String>emptyList() : audio;
        }

        public int getAudioCount() {
            return audio == null ? 0 : audio.size();
        }

        public String getAudio(int $index) {
            return audio.get($index);
        }

        public List<String> getCodecsList() {
            return codecs == null ? Collections.<String>emptyList() : codecs;
        }

        public int getCodecsCount() {
            return codecs == null ? 0 : codecs.size();
        }

        public String getCodecs(int $index) {
            return codecs.get($index);
        }

        public boolean hasContainer() {
            return container != null;
        }

        public String getContainer() {
            return container == null ? "" : container;
        }

        /** Wall time, ms since 1970, after which the address stops working. */
        public boolean hasExpiresAtMs() {
            return expiresAtMs != null;
        }

        public long getExpiresAtMs() {
            return expiresAtMs == null ? 0L : expiresAtMs;
        }

        /** Opaque group. After a failure the core skips other attempts of this group. */
        public String getGroup() {
            return group;
        }

        /** Headers the player must send, such as Referer, Origin, User-Agent. */
        public Map<String, String> getHeadersMap() {
            return headers == null ? Collections.<String, String>emptyMap() : headers;
        }

        public int getHeadersCount() {
            return headers == null ? 0 : headers.size();
        }

        public boolean hasHeight() {
            return height != null;
        }

        public long getHeight() {
            return height == null ? 0L : height;
        }

        public String getId() {
            return id;
        }

        public AttemptKind getKind() {
            return kind;
        }

        public String getUrl() {
            return url;
        }

        /** True when the finder fetched and read this address. */
        public boolean getVerified() {
            return verified;
        }

        public static Attempt read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "audio");
            Object $raw1 = Json.at($fields, "codecs");
            Object $raw2 = Json.at($fields, "container");
            Object $raw3 = Json.at($fields, "expires_at_ms");
            Object $raw4 = Json.need($fields, "group", "Attempt");
            Object $raw5 = Json.need($fields, "headers", "Attempt");
            Object $raw6 = Json.at($fields, "height");
            Object $raw7 = Json.need($fields, "id", "Attempt");
            Object $raw8 = Json.need($fields, "kind", "Attempt");
            Object $raw9 = Json.need($fields, "url", "Attempt");
            Object $raw10 = Json.need($fields, "verified", "Attempt");
            return new Attempt(
                    $raw0 == null ? null : Json.list($raw0, $x0 -> Json.asText($x0)),
                    $raw1 == null ? null : Json.list($raw1, $x0 -> Json.asText($x0)),
                    $raw2 == null ? null : Json.asText($raw2),
                    $raw3 == null ? null : Long.valueOf(Json.asLong($raw3)),
                    Json.asText($raw4),
                    Json.map($raw5, $x0 -> Json.asText($x0)),
                    $raw6 == null ? null : Long.valueOf(Json.asLong($raw6)),
                    Json.asText($raw7),
                    AttemptKind.read($raw8),
                    Json.asText($raw9),
                    Json.asBool($raw10));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (audio != null) {
                $count = Json.key($out, $count, "audio");
                $out.append('[');
                for (int $i0 = 0; $i0 < audio.size(); $i0++) {
                    if ($i0 > 0) {
                        $out.append(',');
                    }
                    Json.text($out, audio.get($i0));
                }
                $out.append(']');
            }
            if (codecs != null) {
                $count = Json.key($out, $count, "codecs");
                $out.append('[');
                for (int $i0 = 0; $i0 < codecs.size(); $i0++) {
                    if ($i0 > 0) {
                        $out.append(',');
                    }
                    Json.text($out, codecs.get($i0));
                }
                $out.append(']');
            }
            if (container != null) {
                $count = Json.key($out, $count, "container");
                Json.text($out, container);
            }
            if (expiresAtMs != null) {
                $count = Json.key($out, $count, "expires_at_ms");
                $out.append(expiresAtMs);
            }
            $count = Json.key($out, $count, "group");
            Json.text($out, group);
            $count = Json.key($out, $count, "headers");
            $out.append('{');
            int $n0 = 0;
            for (Map.Entry<String, String> $e0 : headers.entrySet()) {
                $n0 = Json.key($out, $n0, $e0.getKey());
                Json.text($out, $e0.getValue());
            }
            $out.append('}');
            if (height != null) {
                $count = Json.key($out, $count, "height");
                $out.append(height);
            }
            $count = Json.key($out, $count, "id");
            Json.text($out, id);
            $count = Json.key($out, $count, "kind");
            kind.write($out);
            $count = Json.key($out, $count, "url");
            Json.text($out, url);
            $count = Json.key($out, $count, "verified");
            $out.append(verified);
            $out.append('}');
        }
    }

    /** How an attempt is played. */
    public static final class AttemptKind implements Wire {
        public static final AttemptKind HLS = new AttemptKind("hls");
        public static final AttemptKind FILE = new AttemptKind("file");
        public static final AttemptKind EMBED = new AttemptKind("embed");

        private final String value;

        public AttemptKind(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static AttemptKind read(Object $json) {
            return new AttemptKind(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof AttemptKind && (((AttemptKind) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** One picture a profile can wear. */
    public static final class AvatarPicture implements Wire {
        private final String name;
        private final String path;

        public AvatarPicture(String name, String path) {
            this.name = name;
            this.path = path;
        }

        public String getName() {
            return name;
        }

        /** Where the picture sits on this server. */
        public String getPath() {
            return path;
        }

        public static AvatarPicture read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "name", "AvatarPicture");
            Object $raw1 = Json.need($fields, "path", "AvatarPicture");
            return new AvatarPicture(
                    Json.asText($raw0),
                    Json.asText($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "name");
            Json.text($out, name);
            $count = Json.key($out, $count, "path");
            Json.text($out, path);
            $out.append('}');
        }
    }

    /** A row of pictures on one theme. `kids` says a Kids profile may wear them. */
    public static final class AvatarSet implements Wire {
        private final boolean kids;
        private final String name;
        private final List<AvatarPicture> pictures;

        public AvatarSet(boolean kids, String name, List<AvatarPicture> pictures) {
            this.kids = kids;
            this.name = name;
            this.pictures = pictures;
        }

        public boolean getKids() {
            return kids;
        }

        public String getName() {
            return name;
        }

        public List<AvatarPicture> getPicturesList() {
            return pictures == null ? Collections.<AvatarPicture>emptyList() : pictures;
        }

        public int getPicturesCount() {
            return pictures == null ? 0 : pictures.size();
        }

        public AvatarPicture getPictures(int $index) {
            return pictures.get($index);
        }

        public static AvatarSet read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "kids", "AvatarSet");
            Object $raw1 = Json.need($fields, "name", "AvatarSet");
            Object $raw2 = Json.need($fields, "pictures", "AvatarSet");
            return new AvatarSet(
                    Json.asBool($raw0),
                    Json.asText($raw1),
                    Json.list($raw2, $x0 -> AvatarPicture.read($x0)));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "kids");
            $out.append(kids);
            $count = Json.key($out, $count, "name");
            Json.text($out, name);
            $count = Json.key($out, $count, "pictures");
            $out.append('[');
            for (int $i0 = 0; $i0 < pictures.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                pictures.get($i0).write($out);
            }
            $out.append(']');
            $out.append('}');
        }
    }

    public static final class AvatarSets implements Wire {
        private final List<AvatarSet> sets;

        public AvatarSets(List<AvatarSet> sets) {
            this.sets = sets;
        }

        public List<AvatarSet> getSetsList() {
            return sets == null ? Collections.<AvatarSet>emptyList() : sets;
        }

        public int getSetsCount() {
            return sets == null ? 0 : sets.size();
        }

        public AvatarSet getSets(int $index) {
            return sets.get($index);
        }

        public static AvatarSets read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "sets", "AvatarSets");
            return new AvatarSets(
                    Json.list($raw0, $x0 -> AvatarSet.read($x0)));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "sets");
            $out.append('[');
            for (int $i0 = 0; $i0 < sets.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                sets.get($i0).write($out);
            }
            $out.append(']');
            $out.append('}');
        }
    }

    /** A mark on a tile. */
    public static final class Badge implements Wire {
        private final BadgeKind kind;
        private final Long rank;

        public Badge(BadgeKind kind, Long rank) {
            this.kind = kind;
            this.rank = rank;
        }

        public BadgeKind getKind() {
            return kind;
        }

        public boolean hasRank() {
            return rank != null;
        }

        public long getRank() {
            return rank == null ? 0L : rank;
        }

        public static Badge read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "kind", "Badge");
            Object $raw1 = Json.at($fields, "rank");
            return new Badge(
                    BadgeKind.read($raw0),
                    $raw1 == null ? null : Long.valueOf(Json.asLong($raw1)));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "kind");
            kind.write($out);
            if (rank != null) {
                $count = Json.key($out, $count, "rank");
                $out.append(rank);
            }
            $out.append('}');
        }
    }

    /** Marks on a tile. */
    public static final class BadgeKind implements Wire {
        public static final BadgeKind RANKED = new BadgeKind("ranked");
        public static final BadgeKind NEW = new BadgeKind("new");

        private final String value;

        public BadgeKind(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static BadgeKind read(Object $json) {
            return new BadgeKind(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof BadgeKind && (((BadgeKind) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** Where the body of an answer goes. An answer outside 200 to 299 always comes to the core, because the core reads its error code. */
    public static final class BodyTarget implements Wire {
        public static final BodyTarget CORE = new BodyTarget("core");
        public static final BodyTarget SCREEN = new BodyTarget("screen");

        private final String value;

        public BodyTarget(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static BodyTarget read(Object $json) {
            return new BodyTarget(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof BodyTarget && (((BodyTarget) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** Name and contact, filled into text at render time. */
    public static final class BrandValues implements Wire {
        private final String name;
        private final String supportEmail;

        public BrandValues(String name, String supportEmail) {
            this.name = name;
            this.supportEmail = supportEmail;
        }

        public String getName() {
            return name;
        }

        public String getSupportEmail() {
            return supportEmail;
        }

        public static BrandValues read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "name", "BrandValues");
            Object $raw1 = Json.need($fields, "support_email", "BrandValues");
            return new BrandValues(
                    Json.asText($raw0),
                    Json.asText($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "name");
            Json.text($out, name);
            $count = Json.key($out, $count, "support_email");
            Json.text($out, supportEmail);
            $out.append('}');
        }
    }

    /** A day of the calendar, YYYY-MM-DD. The app writes it out in the viewer's own words. */
    public static final class CalendarDay implements Wire {
        private static final Pattern SHAPE = Pattern.compile("^[0-9]{4}-(0[1-9]|1[0-2])-(0[1-9]|[12][0-9]|3[01])$");

        private final String value;

        public CalendarDay(String value) {
            if (!SHAPE.matcher(value).matches()) {
                throw new IllegalArgumentException("not a CalendarDay");
            }
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static CalendarDay read(Object $json) {
            return new CalendarDay(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof CalendarDay && (((CalendarDay) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** What a device can do. The server filters attempts and sections by this card. */
    public static final class CapabilityCard implements Wire {
        private final boolean adaptive;
        private final String app;
        private final List<AttemptKind> attemptKinds;
        private final DeviceClass class_;
        private final List<String> codecs;
        private final List<String> containers;
        private final Form form;
        private final List<InputKind> input;
        private final List<String> languages;
        private final long maxHeight;
        private final List<String> mediaHeaders;
        private final String os;
        private final String osVersion;
        private final boolean reportsPicture;
        private final long screenHeight;
        private final long screenWidth;
        private final boolean secretStore;
        private final List<SectionKind> sectionKinds;
        private final boolean sideSubtitles;
        private final long storageBytes;
        private final String version;

        public CapabilityCard(boolean adaptive, String app, List<AttemptKind> attemptKinds, DeviceClass class_, List<String> codecs, List<String> containers, Form form, List<InputKind> input, List<String> languages, long maxHeight, List<String> mediaHeaders, String os, String osVersion, boolean reportsPicture, long screenHeight, long screenWidth, boolean secretStore, List<SectionKind> sectionKinds, boolean sideSubtitles, long storageBytes, String version) {
            this.adaptive = adaptive;
            this.app = app;
            this.attemptKinds = attemptKinds;
            this.class_ = class_;
            this.codecs = codecs;
            this.containers = containers;
            this.form = form;
            this.input = input;
            this.languages = languages;
            this.maxHeight = maxHeight;
            this.mediaHeaders = mediaHeaders;
            this.os = os;
            this.osVersion = osVersion;
            this.reportsPicture = reportsPicture;
            this.screenHeight = screenHeight;
            this.screenWidth = screenWidth;
            this.secretStore = secretStore;
            this.sectionKinds = sectionKinds;
            this.sideSubtitles = sideSubtitles;
            this.storageBytes = storageBytes;
            this.version = version;
        }

        /** True when the player switches variants during play. */
        public boolean getAdaptive() {
            return adaptive;
        }

        public String getApp() {
            return app;
        }

        /** Attempt kinds this player opens. An embed appears only when listed here. */
        public List<AttemptKind> getAttemptKindsList() {
            return attemptKinds == null ? Collections.<AttemptKind>emptyList() : attemptKinds;
        }

        public int getAttemptKindsCount() {
            return attemptKinds == null ? 0 : attemptKinds.size();
        }

        public AttemptKind getAttemptKinds(int $index) {
            return attemptKinds.get($index);
        }

        public DeviceClass getClass_() {
            return class_;
        }

        /** Codecs this player opens, such as avc1, hvc1, mp4a. */
        public List<String> getCodecsList() {
            return codecs == null ? Collections.<String>emptyList() : codecs;
        }

        public int getCodecsCount() {
            return codecs == null ? 0 : codecs.size();
        }

        public String getCodecs(int $index) {
            return codecs.get($index);
        }

        /** Containers this player opens, such as ts, fmp4, mp4. */
        public List<String> getContainersList() {
            return containers == null ? Collections.<String>emptyList() : containers;
        }

        public int getContainersCount() {
            return containers == null ? 0 : containers.size();
        }

        public String getContainers(int $index) {
            return containers.get($index);
        }

        public Form getForm() {
            return form;
        }

        public List<InputKind> getInputList() {
            return input == null ? Collections.<InputKind>emptyList() : input;
        }

        public int getInputCount() {
            return input == null ? 0 : input.size();
        }

        public InputKind getInput(int $index) {
            return input.get($index);
        }

        public List<String> getLanguagesList() {
            return languages == null ? Collections.<String>emptyList() : languages;
        }

        public int getLanguagesCount() {
            return languages == null ? 0 : languages.size();
        }

        public String getLanguages(int $index) {
            return languages.get($index);
        }

        public long getMaxHeight() {
            return maxHeight;
        }

        /** Request headers the player can set on media, such as Referer. */
        public List<String> getMediaHeadersList() {
            return mediaHeaders == null ? Collections.<String>emptyList() : mediaHeaders;
        }

        public int getMediaHeadersCount() {
            return mediaHeaders == null ? 0 : mediaHeaders.size();
        }

        public String getMediaHeaders(int $index) {
            return mediaHeaders.get($index);
        }

        public String getOs() {
            return os;
        }

        public String getOsVersion() {
            return osVersion;
        }

        /** True when the driver reports video width or has_picture. */
        public boolean getReportsPicture() {
            return reportsPicture;
        }

        public long getScreenHeight() {
            return screenHeight;
        }

        public long getScreenWidth() {
            return screenWidth;
        }

        /** True when the device has a durable secret store. */
        public boolean getSecretStore() {
            return secretStore;
        }

        /** Section kinds this build draws. Others fall back to a plain row. */
        public List<SectionKind> getSectionKindsList() {
            return sectionKinds == null ? Collections.<SectionKind>emptyList() : sectionKinds;
        }

        public int getSectionKindsCount() {
            return sectionKinds == null ? 0 : sectionKinds.size();
        }

        public SectionKind getSectionKinds(int $index) {
            return sectionKinds.get($index);
        }

        /** True when the player can load a side subtitle file. */
        public boolean getSideSubtitles() {
            return sideSubtitles;
        }

        /** Bytes this device gives to kept commands and screens. 0 means none. */
        public long getStorageBytes() {
            return storageBytes;
        }

        public String getVersion() {
            return version;
        }

        public static CapabilityCard read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "adaptive", "CapabilityCard");
            Object $raw1 = Json.need($fields, "app", "CapabilityCard");
            Object $raw2 = Json.need($fields, "attempt_kinds", "CapabilityCard");
            Object $raw3 = Json.need($fields, "class", "CapabilityCard");
            Object $raw4 = Json.need($fields, "codecs", "CapabilityCard");
            Object $raw5 = Json.need($fields, "containers", "CapabilityCard");
            Object $raw6 = Json.need($fields, "form", "CapabilityCard");
            Object $raw7 = Json.need($fields, "input", "CapabilityCard");
            Object $raw8 = Json.need($fields, "languages", "CapabilityCard");
            Object $raw9 = Json.need($fields, "max_height", "CapabilityCard");
            Object $raw10 = Json.need($fields, "media_headers", "CapabilityCard");
            Object $raw11 = Json.need($fields, "os", "CapabilityCard");
            Object $raw12 = Json.need($fields, "os_version", "CapabilityCard");
            Object $raw13 = Json.need($fields, "reports_picture", "CapabilityCard");
            Object $raw14 = Json.need($fields, "screen_height", "CapabilityCard");
            Object $raw15 = Json.need($fields, "screen_width", "CapabilityCard");
            Object $raw16 = Json.need($fields, "secret_store", "CapabilityCard");
            Object $raw17 = Json.need($fields, "section_kinds", "CapabilityCard");
            Object $raw18 = Json.need($fields, "side_subtitles", "CapabilityCard");
            Object $raw19 = Json.need($fields, "storage_bytes", "CapabilityCard");
            Object $raw20 = Json.need($fields, "version", "CapabilityCard");
            return new CapabilityCard(
                    Json.asBool($raw0),
                    Json.asText($raw1),
                    Json.list($raw2, $x0 -> AttemptKind.read($x0)),
                    DeviceClass.read($raw3),
                    Json.list($raw4, $x0 -> Json.asText($x0)),
                    Json.list($raw5, $x0 -> Json.asText($x0)),
                    Form.read($raw6),
                    Json.list($raw7, $x0 -> InputKind.read($x0)),
                    Json.list($raw8, $x0 -> Json.asText($x0)),
                    Json.asLong($raw9),
                    Json.list($raw10, $x0 -> Json.asText($x0)),
                    Json.asText($raw11),
                    Json.asText($raw12),
                    Json.asBool($raw13),
                    Json.asLong($raw14),
                    Json.asLong($raw15),
                    Json.asBool($raw16),
                    Json.list($raw17, $x0 -> SectionKind.read($x0)),
                    Json.asBool($raw18),
                    Json.asLong($raw19),
                    Json.asText($raw20));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "adaptive");
            $out.append(adaptive);
            $count = Json.key($out, $count, "app");
            Json.text($out, app);
            $count = Json.key($out, $count, "attempt_kinds");
            $out.append('[');
            for (int $i0 = 0; $i0 < attemptKinds.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                attemptKinds.get($i0).write($out);
            }
            $out.append(']');
            $count = Json.key($out, $count, "class");
            class_.write($out);
            $count = Json.key($out, $count, "codecs");
            $out.append('[');
            for (int $i0 = 0; $i0 < codecs.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                Json.text($out, codecs.get($i0));
            }
            $out.append(']');
            $count = Json.key($out, $count, "containers");
            $out.append('[');
            for (int $i0 = 0; $i0 < containers.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                Json.text($out, containers.get($i0));
            }
            $out.append(']');
            $count = Json.key($out, $count, "form");
            form.write($out);
            $count = Json.key($out, $count, "input");
            $out.append('[');
            for (int $i0 = 0; $i0 < input.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                input.get($i0).write($out);
            }
            $out.append(']');
            $count = Json.key($out, $count, "languages");
            $out.append('[');
            for (int $i0 = 0; $i0 < languages.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                Json.text($out, languages.get($i0));
            }
            $out.append(']');
            $count = Json.key($out, $count, "max_height");
            $out.append(maxHeight);
            $count = Json.key($out, $count, "media_headers");
            $out.append('[');
            for (int $i0 = 0; $i0 < mediaHeaders.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                Json.text($out, mediaHeaders.get($i0));
            }
            $out.append(']');
            $count = Json.key($out, $count, "os");
            Json.text($out, os);
            $count = Json.key($out, $count, "os_version");
            Json.text($out, osVersion);
            $count = Json.key($out, $count, "reports_picture");
            $out.append(reportsPicture);
            $count = Json.key($out, $count, "screen_height");
            $out.append(screenHeight);
            $count = Json.key($out, $count, "screen_width");
            $out.append(screenWidth);
            $count = Json.key($out, $count, "secret_store");
            $out.append(secretStore);
            $count = Json.key($out, $count, "section_kinds");
            $out.append('[');
            for (int $i0 = 0; $i0 < sectionKinds.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                sectionKinds.get($i0).write($out);
            }
            $out.append(']');
            $count = Json.key($out, $count, "side_subtitles");
            $out.append(sideSubtitles);
            $count = Json.key($out, $count, "storage_bytes");
            $out.append(storageBytes);
            $count = Json.key($out, $count, "version");
            Json.text($out, version);
            $out.append('}');
        }
    }

    /** One change a device reads. */
    public abstract static class Change implements Wire {
        public static Change read(Object $json) {
            String $tag = Json.asText(Json.need(Json.object($json), "change", "Change"));
            if ("title".equals($tag)) {
                return ChangeTitle.read($json);
            }
            if ("progress".equals($tag)) {
                return ChangeProgress.read($json);
            }
            if ("hidden".equals($tag)) {
                return ChangeHidden.read($json);
            }
            if ("history_cleared".equals($tag)) {
                return ChangeHistoryCleared.read($json);
            }
            if ("setting".equals($tag)) {
                return ChangeSetting.read($json);
            }
            if ("remind".equals($tag)) {
                return ChangeRemind.read($json);
            }
            return ChangeUnknown.read($json);
        }
    }

    public static final class ChangeTitle extends Change {
        private final ListState list;
        private final RatingState rating;
        private final TitleKey title;

        public ChangeTitle(ListState list, RatingState rating, TitleKey title) {
            this.list = list;
            this.rating = rating;
            this.title = title;
        }

        public ListState getList() {
            return list;
        }

        public RatingState getRating() {
            return rating;
        }

        public TitleKey getTitle() {
            return title;
        }

        public static ChangeTitle read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"title".equals(Json.asText(Json.need($fields, "change", "Change")))) {
                throw new IllegalArgumentException("not a ChangeTitle");
            }
            Object $raw0 = Json.need($fields, "list", "Change");
            Object $raw1 = Json.need($fields, "rating", "Change");
            Object $raw2 = Json.need($fields, "title", "Change");
            return new ChangeTitle(
                    ListState.read($raw0),
                    RatingState.read($raw1),
                    TitleKey.read($raw2));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "change");
            Json.text($out, "title");
            $count = Json.key($out, $count, "list");
            list.write($out);
            $count = Json.key($out, $count, "rating");
            rating.write($out);
            $count = Json.key($out, $count, "title");
            title.write($out);
            $out.append('}');
        }
    }

    public static final class ChangeProgress extends Change {
        private final ProgressState state;

        public ChangeProgress(ProgressState state) {
            this.state = state;
        }

        public ProgressState getState() {
            return state;
        }

        public static ChangeProgress read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"progress".equals(Json.asText(Json.need($fields, "change", "Change")))) {
                throw new IllegalArgumentException("not a ChangeProgress");
            }
            Object $raw0 = Json.need($fields, "state", "Change");
            return new ChangeProgress(
                    ProgressState.read($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "change");
            Json.text($out, "progress");
            $count = Json.key($out, $count, "state");
            state.write($out);
            $out.append('}');
        }
    }

    /** Hidden as a whole, every episode of a series included. */
    public static final class ChangeHidden extends Change {
        private final TitleKey title;

        public ChangeHidden(TitleKey title) {
            this.title = title;
        }

        public TitleKey getTitle() {
            return title;
        }

        public static ChangeHidden read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"hidden".equals(Json.asText(Json.need($fields, "change", "Change")))) {
                throw new IllegalArgumentException("not a ChangeHidden");
            }
            Object $raw0 = Json.need($fields, "title", "Change");
            return new ChangeHidden(
                    TitleKey.read($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "change");
            Json.text($out, "hidden");
            $count = Json.key($out, $count, "title");
            title.write($out);
            $out.append('}');
        }
    }

    public static final class ChangeHistoryCleared extends Change {
        private final long changedAtMs;

        public ChangeHistoryCleared(long changedAtMs) {
            this.changedAtMs = changedAtMs;
        }

        public long getChangedAtMs() {
            return changedAtMs;
        }

        public static ChangeHistoryCleared read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"history_cleared".equals(Json.asText(Json.need($fields, "change", "Change")))) {
                throw new IllegalArgumentException("not a ChangeHistoryCleared");
            }
            Object $raw0 = Json.need($fields, "changed_at_ms", "Change");
            return new ChangeHistoryCleared(
                    Json.asLong($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "change");
            Json.text($out, "history_cleared");
            $count = Json.key($out, $count, "changed_at_ms");
            $out.append(changedAtMs);
            $out.append('}');
        }
    }

    public static final class ChangeSetting extends Change {
        private final SettingKey key;
        private final String value;

        public ChangeSetting(SettingKey key, String value) {
            this.key = key;
            this.value = value;
        }

        public SettingKey getKey() {
            return key;
        }

        public String getValue() {
            return value;
        }

        public static ChangeSetting read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"setting".equals(Json.asText(Json.need($fields, "change", "Change")))) {
                throw new IllegalArgumentException("not a ChangeSetting");
            }
            Object $raw0 = Json.need($fields, "key", "Change");
            Object $raw1 = Json.need($fields, "value", "Change");
            return new ChangeSetting(
                    SettingKey.read($raw0),
                    Json.asText($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "change");
            Json.text($out, "setting");
            $count = Json.key($out, $count, "key");
            key.write($out);
            $count = Json.key($out, $count, "value");
            Json.text($out, value);
            $out.append('}');
        }
    }

    /** Reminder set or cleared on a coming title. */
    public static final class ChangeRemind extends Change {
        private final boolean on;
        private final TitleKey title;

        public ChangeRemind(boolean on, TitleKey title) {
            this.on = on;
            this.title = title;
        }

        public boolean getOn() {
            return on;
        }

        public TitleKey getTitle() {
            return title;
        }

        public static ChangeRemind read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"remind".equals(Json.asText(Json.need($fields, "change", "Change")))) {
                throw new IllegalArgumentException("not a ChangeRemind");
            }
            Object $raw0 = Json.need($fields, "on", "Change");
            Object $raw1 = Json.need($fields, "title", "Change");
            return new ChangeRemind(
                    Json.asBool($raw0),
                    TitleKey.read($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "change");
            Json.text($out, "remind");
            $count = Json.key($out, $count, "on");
            $out.append(on);
            $count = Json.key($out, $count, "title");
            title.write($out);
            $out.append('}');
        }
    }

    /** A tag this build does not know. */
    public static final class ChangeUnknown extends Change {
        private final String tag;

        public ChangeUnknown(String tag) {
            this.tag = tag;
        }

        /** The tag that was read. */
        public String getTag() {
            return tag;
        }

        public static ChangeUnknown read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "change");
            return new ChangeUnknown(
                    $raw0 == null ? null : Json.asText($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "change");
            Json.text($out, tag);
            $out.append('}');
        }
    }

    /** `GET /v1/changes`. */
    public static final class Changes implements Wire {
        private final List<Change> changes;
        private final String cursor;
        private final boolean hasMore;

        public Changes(List<Change> changes, String cursor, boolean hasMore) {
            this.changes = changes;
            this.cursor = cursor;
            this.hasMore = hasMore;
        }

        public List<Change> getChangesList() {
            return changes == null ? Collections.<Change>emptyList() : changes;
        }

        public int getChangesCount() {
            return changes == null ? 0 : changes.size();
        }

        public Change getChanges(int $index) {
            return changes.get($index);
        }

        public String getCursor() {
            return cursor;
        }

        public boolean getHasMore() {
            return hasMore;
        }

        public static Changes read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "changes", "Changes");
            Object $raw1 = Json.need($fields, "cursor", "Changes");
            Object $raw2 = Json.need($fields, "has_more", "Changes");
            return new Changes(
                    Json.list($raw0, $x0 -> Change.read($x0)),
                    Json.asText($raw1),
                    Json.asBool($raw2));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "changes");
            $out.append('[');
            for (int $i0 = 0; $i0 < changes.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                changes.get($i0).write($out);
            }
            $out.append(']');
            $count = Json.key($out, $count, "cursor");
            Json.text($out, cursor);
            $count = Json.key($out, $count, "has_more");
            $out.append(hasMore);
            $out.append('}');
        }
    }

    /** One value per client class. */
    public static final class ClassValue implements Wire {
        private final DeviceClass class_;
        private final long value;

        public ClassValue(DeviceClass class_, long value) {
            this.class_ = class_;
            this.value = value;
        }

        public DeviceClass getClass_() {
            return class_;
        }

        public long getValue() {
            return value;
        }

        public static ClassValue read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "class", "ClassValue");
            Object $raw1 = Json.need($fields, "value", "ClassValue");
            return new ClassValue(
                    DeviceClass.read($raw0),
                    Json.asLong($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "class");
            class_.write($out);
            $count = Json.key($out, $count, "value");
            $out.append(value);
            $out.append('}');
        }
    }

    /** Lowest app version a class may run. */
    public static final class ClientFloor implements Wire {
        private final DeviceClass class_;
        private final String version;

        public ClientFloor(DeviceClass class_, String version) {
            this.class_ = class_;
            this.version = version;
        }

        public DeviceClass getClass_() {
            return class_;
        }

        public String getVersion() {
            return version;
        }

        public static ClientFloor read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "class", "ClientFloor");
            Object $raw1 = Json.need($fields, "version", "ClientFloor");
            return new ClientFloor(
                    DeviceClass.read($raw0),
                    Json.asText($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "class");
            class_.write($out);
            $count = Json.key($out, $count, "version");
            Json.text($out, version);
            $out.append('}');
        }
    }

    public static final class CodeSend implements Wire {
        private final String email;

        public CodeSend(String email) {
            this.email = email;
        }

        public String getEmail() {
            return email;
        }

        public static CodeSend read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "email", "CodeSend");
            return new CodeSend(
                    Json.asText($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "email");
            Json.text($out, email);
            $out.append('}');
        }
    }

    /** A code was sent, or would have been. The answer never says whether the account exists. */
    public static final class CodeSent implements Wire {
        private final long expiresIn;

        public CodeSent(long expiresIn) {
            this.expiresIn = expiresIn;
        }

        public long getExpiresIn() {
            return expiresIn;
        }

        public static CodeSent read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "expires_in", "CodeSent");
            return new CodeSent(
                    Json.asLong($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "expires_in");
            $out.append(expiresIn);
            $out.append('}');
        }
    }

    public static final class CodeSignIn implements Wire {
        private final String code;
        private final String email;

        public CodeSignIn(String code, String email) {
            this.code = code;
            this.email = email;
        }

        public String getCode() {
            return code;
        }

        public String getEmail() {
            return email;
        }

        public static CodeSignIn read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "code", "CodeSignIn");
            Object $raw1 = Json.need($fields, "email", "CodeSignIn");
            return new CodeSignIn(
                    Json.asText($raw0),
                    Json.asText($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "code");
            Json.text($out, code);
            $count = Json.key($out, $count, "email");
            Json.text($out, email);
            $out.append('}');
        }
    }

    /** One change to a profile's facts. */
    public abstract static class Command implements Wire {
        public static Command read(Object $json) {
            String $tag = Json.asText(Json.need(Json.object($json), "command", "Command"));
            if ("list".equals($tag)) {
                return CommandList.read($json);
            }
            if ("rate".equals($tag)) {
                return CommandRate.read($json);
            }
            if ("hide".equals($tag)) {
                return CommandHide.read($json);
            }
            if ("clear_history".equals($tag)) {
                return CommandClearHistory.read($json);
            }
            if ("position".equals($tag)) {
                return CommandPosition.read($json);
            }
            if ("setting".equals($tag)) {
                return CommandSetting.read($json);
            }
            if ("feedback".equals($tag)) {
                return CommandFeedback.read($json);
            }
            if ("trailer_refused".equals($tag)) {
                return CommandTrailerRefused.read($json);
            }
            if ("remind".equals($tag)) {
                return CommandRemind.read($json);
            }
            return CommandUnknown.read($json);
        }
    }

    /** Add to or remove from the saved list. Leaves the rating alone. */
    public static final class CommandList extends Command {
        private final boolean inList;
        private final TitleKey title;

        public CommandList(boolean inList, TitleKey title) {
            this.inList = inList;
            this.title = title;
        }

        public boolean getInList() {
            return inList;
        }

        public TitleKey getTitle() {
            return title;
        }

        public static CommandList read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"list".equals(Json.asText(Json.need($fields, "command", "Command")))) {
                throw new IllegalArgumentException("not a CommandList");
            }
            Object $raw0 = Json.need($fields, "in_list", "Command");
            Object $raw1 = Json.need($fields, "title", "Command");
            return new CommandList(
                    Json.asBool($raw0),
                    TitleKey.read($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "command");
            Json.text($out, "list");
            $count = Json.key($out, $count, "in_list");
            $out.append(inList);
            $count = Json.key($out, $count, "title");
            title.write($out);
            $out.append('}');
        }
    }

    /** Set or clear a rating. Leaves the list entry alone. */
    public static final class CommandRate extends Command {
        private final Rating rating;
        private final TitleKey title;

        public CommandRate(Rating rating, TitleKey title) {
            this.rating = rating;
            this.title = title;
        }

        public boolean hasRating() {
            return rating != null;
        }

        public Rating getRating() {
            return rating == null ? null : rating;
        }

        public TitleKey getTitle() {
            return title;
        }

        public static CommandRate read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"rate".equals(Json.asText(Json.need($fields, "command", "Command")))) {
                throw new IllegalArgumentException("not a CommandRate");
            }
            Object $raw0 = Json.at($fields, "rating");
            Object $raw1 = Json.need($fields, "title", "Command");
            return new CommandRate(
                    $raw0 == null ? null : Rating.read($raw0),
                    TitleKey.read($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "command");
            Json.text($out, "rate");
            if (rating != null) {
                $count = Json.key($out, $count, "rating");
                rating.write($out);
            }
            $count = Json.key($out, $count, "title");
            title.write($out);
            $out.append('}');
        }
    }

    /** Hide a title from history. For a series this covers every episode. */
    public static final class CommandHide extends Command {
        private final TitleKey title;

        public CommandHide(TitleKey title) {
            this.title = title;
        }

        public TitleKey getTitle() {
            return title;
        }

        public static CommandHide read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"hide".equals(Json.asText(Json.need($fields, "command", "Command")))) {
                throw new IllegalArgumentException("not a CommandHide");
            }
            Object $raw0 = Json.need($fields, "title", "Command");
            return new CommandHide(
                    TitleKey.read($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "command");
            Json.text($out, "hide");
            $count = Json.key($out, $count, "title");
            title.write($out);
            $out.append('}');
        }
    }

    /** Clear all history of this profile, as one operation. */
    public static final class CommandClearHistory extends Command {
        public CommandClearHistory() {
        }

        public static CommandClearHistory read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"clear_history".equals(Json.asText(Json.need($fields, "command", "Command")))) {
                throw new IllegalArgumentException("not a CommandClearHistory");
            }
            return new CommandClearHistory();
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "command");
            Json.text($out, "clear_history");
            $out.append('}');
        }
    }

    /** Save a position, in whole seconds. */
    public static final class CommandPosition extends Command {
        private final long durationS;
        private final EpisodeRef episode;
        private final long positionS;
        private final TitleKey title;

        public CommandPosition(long durationS, EpisodeRef episode, long positionS, TitleKey title) {
            this.durationS = durationS;
            this.episode = episode;
            this.positionS = positionS;
            this.title = title;
        }

        public long getDurationS() {
            return durationS;
        }

        public boolean hasEpisode() {
            return episode != null;
        }

        public EpisodeRef getEpisode() {
            return episode == null ? null : episode;
        }

        public long getPositionS() {
            return positionS;
        }

        public TitleKey getTitle() {
            return title;
        }

        public static CommandPosition read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"position".equals(Json.asText(Json.need($fields, "command", "Command")))) {
                throw new IllegalArgumentException("not a CommandPosition");
            }
            Object $raw0 = Json.need($fields, "duration_s", "Command");
            Object $raw1 = Json.at($fields, "episode");
            Object $raw2 = Json.need($fields, "position_s", "Command");
            Object $raw3 = Json.need($fields, "title", "Command");
            return new CommandPosition(
                    Json.asLong($raw0),
                    $raw1 == null ? null : EpisodeRef.read($raw1),
                    Json.asLong($raw2),
                    TitleKey.read($raw3));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "command");
            Json.text($out, "position");
            $count = Json.key($out, $count, "duration_s");
            $out.append(durationS);
            if (episode != null) {
                $count = Json.key($out, $count, "episode");
                episode.write($out);
            }
            $count = Json.key($out, $count, "position_s");
            $out.append(positionS);
            $count = Json.key($out, $count, "title");
            title.write($out);
            $out.append('}');
        }
    }

    /** Change one profile setting. Booleans as true or false, style as JSON text. */
    public static final class CommandSetting extends Command {
        private final SettingKey key;
        private final String value;

        public CommandSetting(SettingKey key, String value) {
            this.key = key;
            this.value = value;
        }

        public SettingKey getKey() {
            return key;
        }

        public String getValue() {
            return value;
        }

        public static CommandSetting read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"setting".equals(Json.asText(Json.need($fields, "command", "Command")))) {
                throw new IllegalArgumentException("not a CommandSetting");
            }
            Object $raw0 = Json.need($fields, "key", "Command");
            Object $raw1 = Json.need($fields, "value", "Command");
            return new CommandSetting(
                    SettingKey.read($raw0),
                    Json.asText($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "command");
            Json.text($out, "setting");
            $count = Json.key($out, $count, "key");
            key.write($out);
            $count = Json.key($out, $count, "value");
            Json.text($out, value);
            $out.append('}');
        }
    }

    /** A viewer signal. Discarded by the server when consent is off. */
    public static final class CommandFeedback extends Command {
        private final FeedbackEvent event;
        private final TitleKey title;

        public CommandFeedback(FeedbackEvent event, TitleKey title) {
            this.event = event;
            this.title = title;
        }

        public FeedbackEvent getEvent() {
            return event;
        }

        public TitleKey getTitle() {
            return title;
        }

        public static CommandFeedback read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"feedback".equals(Json.asText(Json.need($fields, "command", "Command")))) {
                throw new IllegalArgumentException("not a CommandFeedback");
            }
            Object $raw0 = Json.need($fields, "event", "Command");
            Object $raw1 = Json.need($fields, "title", "Command");
            return new CommandFeedback(
                    FeedbackEvent.read($raw0),
                    TitleKey.read($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "command");
            Json.text($out, "feedback");
            $count = Json.key($out, $count, "event");
            event.write($out);
            $count = Json.key($out, $count, "title");
            title.write($out);
            $out.append('}');
        }
    }

    /** A trailer failed to play for this title. */
    public static final class CommandTrailerRefused extends Command {
        private final TitleKey title;

        public CommandTrailerRefused(TitleKey title) {
            this.title = title;
        }

        public TitleKey getTitle() {
            return title;
        }

        public static CommandTrailerRefused read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"trailer_refused".equals(Json.asText(Json.need($fields, "command", "Command")))) {
                throw new IllegalArgumentException("not a CommandTrailerRefused");
            }
            Object $raw0 = Json.need($fields, "title", "Command");
            return new CommandTrailerRefused(
                    TitleKey.read($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "command");
            Json.text($out, "trailer_refused");
            $count = Json.key($out, $count, "title");
            title.write($out);
            $out.append('}');
        }
    }

    /** Set or clear a reminder on a coming title. */
    public static final class CommandRemind extends Command {
        private final boolean on;
        private final TitleKey title;

        public CommandRemind(boolean on, TitleKey title) {
            this.on = on;
            this.title = title;
        }

        public boolean getOn() {
            return on;
        }

        public TitleKey getTitle() {
            return title;
        }

        public static CommandRemind read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"remind".equals(Json.asText(Json.need($fields, "command", "Command")))) {
                throw new IllegalArgumentException("not a CommandRemind");
            }
            Object $raw0 = Json.need($fields, "on", "Command");
            Object $raw1 = Json.need($fields, "title", "Command");
            return new CommandRemind(
                    Json.asBool($raw0),
                    TitleKey.read($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "command");
            Json.text($out, "remind");
            $count = Json.key($out, $count, "on");
            $out.append(on);
            $count = Json.key($out, $count, "title");
            title.write($out);
            $out.append('}');
        }
    }

    /** A tag this build does not know. */
    public static final class CommandUnknown extends Command {
        private final String tag;

        public CommandUnknown(String tag) {
            this.tag = tag;
        }

        /** The tag that was read. */
        public String getTag() {
            return tag;
        }

        public static CommandUnknown read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "command");
            return new CommandUnknown(
                    $raw0 == null ? null : Json.asText($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "command");
            Json.text($out, tag);
            $out.append('}');
        }
    }

    /** Answer for a batch, one entry per op. */
    public static final class CommandAnswer implements Wire {
        private final List<OpAnswer> answers;

        public CommandAnswer(List<OpAnswer> answers) {
            this.answers = answers;
        }

        public List<OpAnswer> getAnswersList() {
            return answers == null ? Collections.<OpAnswer>emptyList() : answers;
        }

        public int getAnswersCount() {
            return answers == null ? 0 : answers.size();
        }

        public OpAnswer getAnswers(int $index) {
            return answers.get($index);
        }

        public static CommandAnswer read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "answers", "CommandAnswer");
            return new CommandAnswer(
                    Json.list($raw0, $x0 -> OpAnswer.read($x0)));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "answers");
            $out.append('[');
            for (int $i0 = 0; $i0 < answers.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                answers.get($i0).write($out);
            }
            $out.append(']');
            $out.append('}');
        }
    }

    /** `POST /v1/commands`. Idempotent by op id. */
    public static final class CommandBatch implements Wire {
        private final List<CommandOp> ops;

        public CommandBatch(List<CommandOp> ops) {
            this.ops = ops;
        }

        public List<CommandOp> getOpsList() {
            return ops == null ? Collections.<CommandOp>emptyList() : ops;
        }

        public int getOpsCount() {
            return ops == null ? 0 : ops.size();
        }

        public CommandOp getOps(int $index) {
            return ops.get($index);
        }

        public static CommandBatch read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "ops", "CommandBatch");
            return new CommandBatch(
                    Json.list($raw0, $x0 -> CommandOp.read($x0)));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "ops");
            $out.append('[');
            for (int $i0 = 0; $i0 < ops.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                ops.get($i0).write($out);
            }
            $out.append(']');
            $out.append('}');
        }
    }

    /** One queued op. A missing `age_ms` means age unknown, and the server orders by arrival. */
    public abstract static class CommandOp implements Wire {
        public abstract boolean hasAgeMs();

        public abstract long getAgeMs();

        public abstract String getOpId();

        public static CommandOp read(Object $json) {
            String $tag = Json.asText(Json.need(Json.object($json), "command", "CommandOp"));
            if ("list".equals($tag)) {
                return CommandOpList.read($json);
            }
            if ("rate".equals($tag)) {
                return CommandOpRate.read($json);
            }
            if ("hide".equals($tag)) {
                return CommandOpHide.read($json);
            }
            if ("clear_history".equals($tag)) {
                return CommandOpClearHistory.read($json);
            }
            if ("position".equals($tag)) {
                return CommandOpPosition.read($json);
            }
            if ("setting".equals($tag)) {
                return CommandOpSetting.read($json);
            }
            if ("feedback".equals($tag)) {
                return CommandOpFeedback.read($json);
            }
            if ("trailer_refused".equals($tag)) {
                return CommandOpTrailerRefused.read($json);
            }
            if ("remind".equals($tag)) {
                return CommandOpRemind.read($json);
            }
            return CommandOpUnknown.read($json);
        }
    }

    /** Add to or remove from the saved list. Leaves the rating alone. */
    public static final class CommandOpList extends CommandOp {
        private final Long ageMs;
        private final String opId;
        private final boolean inList;
        private final TitleKey title;

        public CommandOpList(Long ageMs, String opId, boolean inList, TitleKey title) {
            this.ageMs = ageMs;
            this.opId = opId;
            this.inList = inList;
            this.title = title;
        }

        @Override
        public boolean hasAgeMs() {
            return ageMs != null;
        }

        @Override
        public long getAgeMs() {
            return ageMs == null ? 0L : ageMs;
        }

        @Override
        public String getOpId() {
            return opId;
        }

        public boolean getInList() {
            return inList;
        }

        public TitleKey getTitle() {
            return title;
        }

        public static CommandOpList read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"list".equals(Json.asText(Json.need($fields, "command", "CommandOp")))) {
                throw new IllegalArgumentException("not a CommandOpList");
            }
            Object $raw0 = Json.at($fields, "age_ms");
            Object $raw1 = Json.need($fields, "op_id", "CommandOp");
            Object $raw2 = Json.need($fields, "in_list", "CommandOp");
            Object $raw3 = Json.need($fields, "title", "CommandOp");
            return new CommandOpList(
                    $raw0 == null ? null : Long.valueOf(Json.asLong($raw0)),
                    Json.asText($raw1),
                    Json.asBool($raw2),
                    TitleKey.read($raw3));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "command");
            Json.text($out, "list");
            if (ageMs != null) {
                $count = Json.key($out, $count, "age_ms");
                $out.append(ageMs);
            }
            $count = Json.key($out, $count, "op_id");
            Json.text($out, opId);
            $count = Json.key($out, $count, "in_list");
            $out.append(inList);
            $count = Json.key($out, $count, "title");
            title.write($out);
            $out.append('}');
        }
    }

    /** Set or clear a rating. Leaves the list entry alone. */
    public static final class CommandOpRate extends CommandOp {
        private final Long ageMs;
        private final String opId;
        private final Rating rating;
        private final TitleKey title;

        public CommandOpRate(Long ageMs, String opId, Rating rating, TitleKey title) {
            this.ageMs = ageMs;
            this.opId = opId;
            this.rating = rating;
            this.title = title;
        }

        @Override
        public boolean hasAgeMs() {
            return ageMs != null;
        }

        @Override
        public long getAgeMs() {
            return ageMs == null ? 0L : ageMs;
        }

        @Override
        public String getOpId() {
            return opId;
        }

        public boolean hasRating() {
            return rating != null;
        }

        public Rating getRating() {
            return rating == null ? null : rating;
        }

        public TitleKey getTitle() {
            return title;
        }

        public static CommandOpRate read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"rate".equals(Json.asText(Json.need($fields, "command", "CommandOp")))) {
                throw new IllegalArgumentException("not a CommandOpRate");
            }
            Object $raw0 = Json.at($fields, "age_ms");
            Object $raw1 = Json.need($fields, "op_id", "CommandOp");
            Object $raw2 = Json.at($fields, "rating");
            Object $raw3 = Json.need($fields, "title", "CommandOp");
            return new CommandOpRate(
                    $raw0 == null ? null : Long.valueOf(Json.asLong($raw0)),
                    Json.asText($raw1),
                    $raw2 == null ? null : Rating.read($raw2),
                    TitleKey.read($raw3));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "command");
            Json.text($out, "rate");
            if (ageMs != null) {
                $count = Json.key($out, $count, "age_ms");
                $out.append(ageMs);
            }
            $count = Json.key($out, $count, "op_id");
            Json.text($out, opId);
            if (rating != null) {
                $count = Json.key($out, $count, "rating");
                rating.write($out);
            }
            $count = Json.key($out, $count, "title");
            title.write($out);
            $out.append('}');
        }
    }

    /** Hide a title from history. For a series this covers every episode. */
    public static final class CommandOpHide extends CommandOp {
        private final Long ageMs;
        private final String opId;
        private final TitleKey title;

        public CommandOpHide(Long ageMs, String opId, TitleKey title) {
            this.ageMs = ageMs;
            this.opId = opId;
            this.title = title;
        }

        @Override
        public boolean hasAgeMs() {
            return ageMs != null;
        }

        @Override
        public long getAgeMs() {
            return ageMs == null ? 0L : ageMs;
        }

        @Override
        public String getOpId() {
            return opId;
        }

        public TitleKey getTitle() {
            return title;
        }

        public static CommandOpHide read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"hide".equals(Json.asText(Json.need($fields, "command", "CommandOp")))) {
                throw new IllegalArgumentException("not a CommandOpHide");
            }
            Object $raw0 = Json.at($fields, "age_ms");
            Object $raw1 = Json.need($fields, "op_id", "CommandOp");
            Object $raw2 = Json.need($fields, "title", "CommandOp");
            return new CommandOpHide(
                    $raw0 == null ? null : Long.valueOf(Json.asLong($raw0)),
                    Json.asText($raw1),
                    TitleKey.read($raw2));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "command");
            Json.text($out, "hide");
            if (ageMs != null) {
                $count = Json.key($out, $count, "age_ms");
                $out.append(ageMs);
            }
            $count = Json.key($out, $count, "op_id");
            Json.text($out, opId);
            $count = Json.key($out, $count, "title");
            title.write($out);
            $out.append('}');
        }
    }

    /** Clear all history of this profile, as one operation. */
    public static final class CommandOpClearHistory extends CommandOp {
        private final Long ageMs;
        private final String opId;

        public CommandOpClearHistory(Long ageMs, String opId) {
            this.ageMs = ageMs;
            this.opId = opId;
        }

        @Override
        public boolean hasAgeMs() {
            return ageMs != null;
        }

        @Override
        public long getAgeMs() {
            return ageMs == null ? 0L : ageMs;
        }

        @Override
        public String getOpId() {
            return opId;
        }

        public static CommandOpClearHistory read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"clear_history".equals(Json.asText(Json.need($fields, "command", "CommandOp")))) {
                throw new IllegalArgumentException("not a CommandOpClearHistory");
            }
            Object $raw0 = Json.at($fields, "age_ms");
            Object $raw1 = Json.need($fields, "op_id", "CommandOp");
            return new CommandOpClearHistory(
                    $raw0 == null ? null : Long.valueOf(Json.asLong($raw0)),
                    Json.asText($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "command");
            Json.text($out, "clear_history");
            if (ageMs != null) {
                $count = Json.key($out, $count, "age_ms");
                $out.append(ageMs);
            }
            $count = Json.key($out, $count, "op_id");
            Json.text($out, opId);
            $out.append('}');
        }
    }

    /** Save a position, in whole seconds. */
    public static final class CommandOpPosition extends CommandOp {
        private final Long ageMs;
        private final String opId;
        private final long durationS;
        private final EpisodeRef episode;
        private final long positionS;
        private final TitleKey title;

        public CommandOpPosition(Long ageMs, String opId, long durationS, EpisodeRef episode, long positionS, TitleKey title) {
            this.ageMs = ageMs;
            this.opId = opId;
            this.durationS = durationS;
            this.episode = episode;
            this.positionS = positionS;
            this.title = title;
        }

        @Override
        public boolean hasAgeMs() {
            return ageMs != null;
        }

        @Override
        public long getAgeMs() {
            return ageMs == null ? 0L : ageMs;
        }

        @Override
        public String getOpId() {
            return opId;
        }

        public long getDurationS() {
            return durationS;
        }

        public boolean hasEpisode() {
            return episode != null;
        }

        public EpisodeRef getEpisode() {
            return episode == null ? null : episode;
        }

        public long getPositionS() {
            return positionS;
        }

        public TitleKey getTitle() {
            return title;
        }

        public static CommandOpPosition read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"position".equals(Json.asText(Json.need($fields, "command", "CommandOp")))) {
                throw new IllegalArgumentException("not a CommandOpPosition");
            }
            Object $raw0 = Json.at($fields, "age_ms");
            Object $raw1 = Json.need($fields, "op_id", "CommandOp");
            Object $raw2 = Json.need($fields, "duration_s", "CommandOp");
            Object $raw3 = Json.at($fields, "episode");
            Object $raw4 = Json.need($fields, "position_s", "CommandOp");
            Object $raw5 = Json.need($fields, "title", "CommandOp");
            return new CommandOpPosition(
                    $raw0 == null ? null : Long.valueOf(Json.asLong($raw0)),
                    Json.asText($raw1),
                    Json.asLong($raw2),
                    $raw3 == null ? null : EpisodeRef.read($raw3),
                    Json.asLong($raw4),
                    TitleKey.read($raw5));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "command");
            Json.text($out, "position");
            if (ageMs != null) {
                $count = Json.key($out, $count, "age_ms");
                $out.append(ageMs);
            }
            $count = Json.key($out, $count, "op_id");
            Json.text($out, opId);
            $count = Json.key($out, $count, "duration_s");
            $out.append(durationS);
            if (episode != null) {
                $count = Json.key($out, $count, "episode");
                episode.write($out);
            }
            $count = Json.key($out, $count, "position_s");
            $out.append(positionS);
            $count = Json.key($out, $count, "title");
            title.write($out);
            $out.append('}');
        }
    }

    /** Change one profile setting. Booleans as true or false, style as JSON text. */
    public static final class CommandOpSetting extends CommandOp {
        private final Long ageMs;
        private final String opId;
        private final SettingKey key;
        private final String value;

        public CommandOpSetting(Long ageMs, String opId, SettingKey key, String value) {
            this.ageMs = ageMs;
            this.opId = opId;
            this.key = key;
            this.value = value;
        }

        @Override
        public boolean hasAgeMs() {
            return ageMs != null;
        }

        @Override
        public long getAgeMs() {
            return ageMs == null ? 0L : ageMs;
        }

        @Override
        public String getOpId() {
            return opId;
        }

        public SettingKey getKey() {
            return key;
        }

        public String getValue() {
            return value;
        }

        public static CommandOpSetting read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"setting".equals(Json.asText(Json.need($fields, "command", "CommandOp")))) {
                throw new IllegalArgumentException("not a CommandOpSetting");
            }
            Object $raw0 = Json.at($fields, "age_ms");
            Object $raw1 = Json.need($fields, "op_id", "CommandOp");
            Object $raw2 = Json.need($fields, "key", "CommandOp");
            Object $raw3 = Json.need($fields, "value", "CommandOp");
            return new CommandOpSetting(
                    $raw0 == null ? null : Long.valueOf(Json.asLong($raw0)),
                    Json.asText($raw1),
                    SettingKey.read($raw2),
                    Json.asText($raw3));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "command");
            Json.text($out, "setting");
            if (ageMs != null) {
                $count = Json.key($out, $count, "age_ms");
                $out.append(ageMs);
            }
            $count = Json.key($out, $count, "op_id");
            Json.text($out, opId);
            $count = Json.key($out, $count, "key");
            key.write($out);
            $count = Json.key($out, $count, "value");
            Json.text($out, value);
            $out.append('}');
        }
    }

    /** A viewer signal. Discarded by the server when consent is off. */
    public static final class CommandOpFeedback extends CommandOp {
        private final Long ageMs;
        private final String opId;
        private final FeedbackEvent event;
        private final TitleKey title;

        public CommandOpFeedback(Long ageMs, String opId, FeedbackEvent event, TitleKey title) {
            this.ageMs = ageMs;
            this.opId = opId;
            this.event = event;
            this.title = title;
        }

        @Override
        public boolean hasAgeMs() {
            return ageMs != null;
        }

        @Override
        public long getAgeMs() {
            return ageMs == null ? 0L : ageMs;
        }

        @Override
        public String getOpId() {
            return opId;
        }

        public FeedbackEvent getEvent() {
            return event;
        }

        public TitleKey getTitle() {
            return title;
        }

        public static CommandOpFeedback read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"feedback".equals(Json.asText(Json.need($fields, "command", "CommandOp")))) {
                throw new IllegalArgumentException("not a CommandOpFeedback");
            }
            Object $raw0 = Json.at($fields, "age_ms");
            Object $raw1 = Json.need($fields, "op_id", "CommandOp");
            Object $raw2 = Json.need($fields, "event", "CommandOp");
            Object $raw3 = Json.need($fields, "title", "CommandOp");
            return new CommandOpFeedback(
                    $raw0 == null ? null : Long.valueOf(Json.asLong($raw0)),
                    Json.asText($raw1),
                    FeedbackEvent.read($raw2),
                    TitleKey.read($raw3));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "command");
            Json.text($out, "feedback");
            if (ageMs != null) {
                $count = Json.key($out, $count, "age_ms");
                $out.append(ageMs);
            }
            $count = Json.key($out, $count, "op_id");
            Json.text($out, opId);
            $count = Json.key($out, $count, "event");
            event.write($out);
            $count = Json.key($out, $count, "title");
            title.write($out);
            $out.append('}');
        }
    }

    /** A trailer failed to play for this title. */
    public static final class CommandOpTrailerRefused extends CommandOp {
        private final Long ageMs;
        private final String opId;
        private final TitleKey title;

        public CommandOpTrailerRefused(Long ageMs, String opId, TitleKey title) {
            this.ageMs = ageMs;
            this.opId = opId;
            this.title = title;
        }

        @Override
        public boolean hasAgeMs() {
            return ageMs != null;
        }

        @Override
        public long getAgeMs() {
            return ageMs == null ? 0L : ageMs;
        }

        @Override
        public String getOpId() {
            return opId;
        }

        public TitleKey getTitle() {
            return title;
        }

        public static CommandOpTrailerRefused read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"trailer_refused".equals(Json.asText(Json.need($fields, "command", "CommandOp")))) {
                throw new IllegalArgumentException("not a CommandOpTrailerRefused");
            }
            Object $raw0 = Json.at($fields, "age_ms");
            Object $raw1 = Json.need($fields, "op_id", "CommandOp");
            Object $raw2 = Json.need($fields, "title", "CommandOp");
            return new CommandOpTrailerRefused(
                    $raw0 == null ? null : Long.valueOf(Json.asLong($raw0)),
                    Json.asText($raw1),
                    TitleKey.read($raw2));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "command");
            Json.text($out, "trailer_refused");
            if (ageMs != null) {
                $count = Json.key($out, $count, "age_ms");
                $out.append(ageMs);
            }
            $count = Json.key($out, $count, "op_id");
            Json.text($out, opId);
            $count = Json.key($out, $count, "title");
            title.write($out);
            $out.append('}');
        }
    }

    /** Set or clear a reminder on a coming title. */
    public static final class CommandOpRemind extends CommandOp {
        private final Long ageMs;
        private final String opId;
        private final boolean on;
        private final TitleKey title;

        public CommandOpRemind(Long ageMs, String opId, boolean on, TitleKey title) {
            this.ageMs = ageMs;
            this.opId = opId;
            this.on = on;
            this.title = title;
        }

        @Override
        public boolean hasAgeMs() {
            return ageMs != null;
        }

        @Override
        public long getAgeMs() {
            return ageMs == null ? 0L : ageMs;
        }

        @Override
        public String getOpId() {
            return opId;
        }

        public boolean getOn() {
            return on;
        }

        public TitleKey getTitle() {
            return title;
        }

        public static CommandOpRemind read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"remind".equals(Json.asText(Json.need($fields, "command", "CommandOp")))) {
                throw new IllegalArgumentException("not a CommandOpRemind");
            }
            Object $raw0 = Json.at($fields, "age_ms");
            Object $raw1 = Json.need($fields, "op_id", "CommandOp");
            Object $raw2 = Json.need($fields, "on", "CommandOp");
            Object $raw3 = Json.need($fields, "title", "CommandOp");
            return new CommandOpRemind(
                    $raw0 == null ? null : Long.valueOf(Json.asLong($raw0)),
                    Json.asText($raw1),
                    Json.asBool($raw2),
                    TitleKey.read($raw3));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "command");
            Json.text($out, "remind");
            if (ageMs != null) {
                $count = Json.key($out, $count, "age_ms");
                $out.append(ageMs);
            }
            $count = Json.key($out, $count, "op_id");
            Json.text($out, opId);
            $count = Json.key($out, $count, "on");
            $out.append(on);
            $count = Json.key($out, $count, "title");
            title.write($out);
            $out.append('}');
        }
    }

    /** A tag this build does not know. */
    public static final class CommandOpUnknown extends CommandOp {
        private final String tag;
        private final Long rawAgeMs;
        private final String rawOpId;

        public CommandOpUnknown(String tag, Long rawAgeMs, String rawOpId) {
            this.tag = tag;
            this.rawAgeMs = rawAgeMs;
            this.rawOpId = rawOpId;
        }

        /** The tag that was read. */
        public String getTag() {
            return tag;
        }

        @Override
        public boolean hasAgeMs() {
            return rawAgeMs != null;
        }

        @Override
        public long getAgeMs() {
            return rawAgeMs == null ? 0L : rawAgeMs;
        }

        @Override
        public String getOpId() {
            return rawOpId == null ? "" : rawOpId;
        }

        public static CommandOpUnknown read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "command");
            Object $raw1 = Json.at($fields, "age_ms");
            Object $raw2 = Json.at($fields, "op_id");
            return new CommandOpUnknown(
                    $raw0 == null ? null : Json.asText($raw0),
                    $raw1 == null ? null : Long.valueOf(Json.asLong($raw1)),
                    $raw2 == null ? null : Json.asText($raw2));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "command");
            Json.text($out, tag);
            if (rawAgeMs != null) {
                $count = Json.key($out, $count, "age_ms");
                $out.append(rawAgeMs);
            }
            if (rawOpId != null) {
                $count = Json.key($out, $count, "op_id");
                Json.text($out, rawOpId);
            }
            $out.append('}');
        }
    }

    /** Step up. Send a password or an emailed code. */
    public static final class Confirm implements Wire {
        private final String code;
        private final String password;

        public Confirm(String code, String password) {
            this.code = code;
            this.password = password;
        }

        public boolean hasCode() {
            return code != null;
        }

        public String getCode() {
            return code == null ? "" : code;
        }

        public boolean hasPassword() {
            return password != null;
        }

        public String getPassword() {
            return password == null ? "" : password;
        }

        public static Confirm read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "code");
            Object $raw1 = Json.at($fields, "password");
            return new Confirm(
                    $raw0 == null ? null : Json.asText($raw0),
                    $raw1 == null ? null : Json.asText($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (code != null) {
                $count = Json.key($out, $count, "code");
                Json.text($out, code);
            }
            if (password != null) {
                $count = Json.key($out, $count, "password");
                Json.text($out, password);
            }
            $out.append('}');
        }
    }

    /** A step up holds for this many seconds and a refresh never renews it. */
    public static final class Confirmed implements Wire {
        private final long validS;

        public Confirmed(long validS) {
            this.validS = validS;
        }

        public long getValidS() {
            return validS;
        }

        public static Confirmed read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "valid_s", "Confirmed");
            return new Confirmed(
                    Json.asLong($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "valid_s");
            $out.append(validS);
            $out.append('}');
        }
    }

    /** Consent for personal rows. */
    public static final class Consent implements Wire {
        private final boolean personal;

        public Consent(boolean personal) {
            this.personal = personal;
        }

        public boolean getPersonal() {
            return personal;
        }

        public static Consent read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "personal", "Consent");
            return new Consent(
                    Json.asBool($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "personal");
            $out.append(personal);
            $out.append('}');
        }
    }

    /** Given to `core_open`, once. The seed is the only random input the core ever gets. */
    public static final class CoreConfig implements Wire {
        private final String app;
        private final CapabilityCard card;
        private final DeviceClass class_;
        private final WireDate contract;
        private final Form form;
        private final CoreMode mode;
        private final String os;
        private final String seed;
        private final CoreConfigTag type;
        private final String version;

        public CoreConfig(String app, CapabilityCard card, DeviceClass class_, WireDate contract, Form form, CoreMode mode, String os, String seed, CoreConfigTag type, String version) {
            this.app = app;
            this.card = card;
            this.class_ = class_;
            this.contract = contract;
            this.form = form;
            this.mode = mode;
            this.os = os;
            this.seed = seed;
            this.type = type;
            this.version = version;
        }

        public String getApp() {
            return app;
        }

        /** What this device can do. The core sends its hash after sign in. */
        public boolean hasCard() {
            return card != null;
        }

        public CapabilityCard getCard() {
            return card == null ? null : card;
        }

        public DeviceClass getClass_() {
            return class_;
        }

        public WireDate getContract() {
            return contract;
        }

        public Form getForm() {
            return form;
        }

        public CoreMode getMode() {
            return mode;
        }

        public String getOs() {
            return os;
        }

        /** Hex digits, at least 32. */
        public String getSeed() {
            return seed;
        }

        public CoreConfigTag getType() {
            return type;
        }

        public String getVersion() {
            return version;
        }

        public static CoreConfig read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "app", "CoreConfig");
            Object $raw1 = Json.at($fields, "card");
            Object $raw2 = Json.need($fields, "class", "CoreConfig");
            Object $raw3 = Json.need($fields, "contract", "CoreConfig");
            Object $raw4 = Json.need($fields, "form", "CoreConfig");
            Object $raw5 = Json.need($fields, "mode", "CoreConfig");
            Object $raw6 = Json.need($fields, "os", "CoreConfig");
            Object $raw7 = Json.need($fields, "seed", "CoreConfig");
            Object $raw8 = Json.need($fields, "type", "CoreConfig");
            Object $raw9 = Json.need($fields, "version", "CoreConfig");
            return new CoreConfig(
                    Json.asText($raw0),
                    $raw1 == null ? null : CapabilityCard.read($raw1),
                    DeviceClass.read($raw2),
                    WireDate.read($raw3),
                    Form.read($raw4),
                    CoreMode.read($raw5),
                    Json.asText($raw6),
                    Json.asText($raw7),
                    CoreConfigTag.read($raw8),
                    Json.asText($raw9));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "app");
            Json.text($out, app);
            if (card != null) {
                $count = Json.key($out, $count, "card");
                card.write($out);
            }
            $count = Json.key($out, $count, "class");
            class_.write($out);
            $count = Json.key($out, $count, "contract");
            contract.write($out);
            $count = Json.key($out, $count, "form");
            form.write($out);
            $count = Json.key($out, $count, "mode");
            mode.write($out);
            $count = Json.key($out, $count, "os");
            Json.text($out, os);
            $count = Json.key($out, $count, "seed");
            Json.text($out, seed);
            $count = Json.key($out, $count, "type");
            type.write($out);
            $count = Json.key($out, $count, "version");
            Json.text($out, version);
            $out.append('}');
        }
    }

    /** Type tag of `CoreConfig`. */
    public enum CoreConfigTag implements Wire {
        VALUE("core_config");

        private final String wire;

        CoreConfigTag(String wire) {
            this.wire = wire;
        }

        public String wire() {
            return wire;
        }

        public static CoreConfigTag read(Object $json) {
            String $word = Json.asText($json);
            if (!"core_config".equals($word)) {
                throw new IllegalArgumentException("not a CoreConfigTag: " + $word);
            }
            return VALUE;
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, wire);
        }
    }

    /** One step of input. Every input carries wall time and monotonic time. */
    public static final class CoreIn implements Wire {
        private final WireDate contract;
        private final List<Event> events;
        private final long monoMs;
        private final long nowMs;
        private final CoreInTag type;

        public CoreIn(WireDate contract, List<Event> events, long monoMs, long nowMs, CoreInTag type) {
            this.contract = contract;
            this.events = events;
            this.monoMs = monoMs;
            this.nowMs = nowMs;
            this.type = type;
        }

        public WireDate getContract() {
            return contract;
        }

        public List<Event> getEventsList() {
            return events == null ? Collections.<Event>emptyList() : events;
        }

        public int getEventsCount() {
            return events == null ? 0 : events.size();
        }

        public Event getEvents(int $index) {
            return events.get($index);
        }

        /** Monotonic time, ms. Restarts at boot. */
        public long getMonoMs() {
            return monoMs;
        }

        /** Wall time, ms since 1970. */
        public long getNowMs() {
            return nowMs;
        }

        public CoreInTag getType() {
            return type;
        }

        public static CoreIn read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "contract", "CoreIn");
            Object $raw1 = Json.need($fields, "events", "CoreIn");
            Object $raw2 = Json.need($fields, "mono_ms", "CoreIn");
            Object $raw3 = Json.need($fields, "now_ms", "CoreIn");
            Object $raw4 = Json.need($fields, "type", "CoreIn");
            return new CoreIn(
                    WireDate.read($raw0),
                    Json.list($raw1, $x0 -> Event.read($x0)),
                    Json.asLong($raw2),
                    Json.asLong($raw3),
                    CoreInTag.read($raw4));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "contract");
            contract.write($out);
            $count = Json.key($out, $count, "events");
            $out.append('[');
            for (int $i0 = 0; $i0 < events.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                events.get($i0).write($out);
            }
            $out.append(']');
            $count = Json.key($out, $count, "mono_ms");
            $out.append(monoMs);
            $count = Json.key($out, $count, "now_ms");
            $out.append(nowMs);
            $count = Json.key($out, $count, "type");
            type.write($out);
            $out.append('}');
        }
    }

    /** Type tag of `CoreIn`. */
    public enum CoreInTag implements Wire {
        VALUE("core_in");

        private final String wire;

        CoreInTag(String wire) {
            this.wire = wire;
        }

        public String wire() {
            return wire;
        }

        public static CoreInTag read(Object $json) {
            String $word = Json.asText($json);
            if (!"core_in".equals($word)) {
                throw new IllegalArgumentException("not a CoreInTag: " + $word);
            }
            return VALUE;
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, wire);
        }
    }

    /** Where the core runs for this device. */
    public static final class CoreMode implements Wire {
        public static final CoreMode LOCAL = new CoreMode("local");
        public static final CoreMode HOSTED = new CoreMode("hosted");

        private final String value;

        public CoreMode(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static CoreMode read(Object $json) {
            return new CoreMode(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof CoreMode && (((CoreMode) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** One step of output. */
    public static final class CoreOut implements Wire {
        private final WireDate contract;
        private final List<Effect> effects;
        private final Long nextWakeMs;
        private final CoreOutTag type;
        private final List<String> unsupported;
        private final View view;

        public CoreOut(WireDate contract, List<Effect> effects, Long nextWakeMs, CoreOutTag type, List<String> unsupported, View view) {
            this.contract = contract;
            this.effects = effects;
            this.nextWakeMs = nextWakeMs;
            this.type = type;
            this.unsupported = unsupported;
            this.view = view;
        }

        public WireDate getContract() {
            return contract;
        }

        public List<Effect> getEffectsList() {
            return effects == null ? Collections.<Effect>emptyList() : effects;
        }

        public int getEffectsCount() {
            return effects == null ? 0 : effects.size();
        }

        public Effect getEffects(int $index) {
            return effects.get($index);
        }

        /** Monotonic time at which the host steps the core again. Absent means no wake. */
        public boolean hasNextWakeMs() {
            return nextWakeMs != null;
        }

        public long getNextWakeMs() {
            return nextWakeMs == null ? 0L : nextWakeMs;
        }

        public CoreOutTag getType() {
            return type;
        }

        /** Ids of in events of a type this core does not know. */
        public List<String> getUnsupportedList() {
            return unsupported == null ? Collections.<String>emptyList() : unsupported;
        }

        public int getUnsupportedCount() {
            return unsupported == null ? 0 : unsupported.size();
        }

        public String getUnsupported(int $index) {
            return unsupported.get($index);
        }

        public boolean hasView() {
            return view != null;
        }

        public View getView() {
            return view == null ? null : view;
        }

        public static CoreOut read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "contract", "CoreOut");
            Object $raw1 = Json.need($fields, "effects", "CoreOut");
            Object $raw2 = Json.at($fields, "next_wake_ms");
            Object $raw3 = Json.need($fields, "type", "CoreOut");
            Object $raw4 = Json.at($fields, "unsupported");
            Object $raw5 = Json.at($fields, "view");
            return new CoreOut(
                    WireDate.read($raw0),
                    Json.list($raw1, $x0 -> Effect.read($x0)),
                    $raw2 == null ? null : Long.valueOf(Json.asLong($raw2)),
                    CoreOutTag.read($raw3),
                    $raw4 == null ? null : Json.list($raw4, $x0 -> Json.asText($x0)),
                    $raw5 == null ? null : View.read($raw5));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "contract");
            contract.write($out);
            $count = Json.key($out, $count, "effects");
            $out.append('[');
            for (int $i0 = 0; $i0 < effects.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                effects.get($i0).write($out);
            }
            $out.append(']');
            if (nextWakeMs != null) {
                $count = Json.key($out, $count, "next_wake_ms");
                $out.append(nextWakeMs);
            }
            $count = Json.key($out, $count, "type");
            type.write($out);
            if (unsupported != null) {
                $count = Json.key($out, $count, "unsupported");
                $out.append('[');
                for (int $i0 = 0; $i0 < unsupported.size(); $i0++) {
                    if ($i0 > 0) {
                        $out.append(',');
                    }
                    Json.text($out, unsupported.get($i0));
                }
                $out.append(']');
            }
            if (view != null) {
                $count = Json.key($out, $count, "view");
                view.write($out);
            }
            $out.append('}');
        }
    }

    /** Type tag of `CoreOut`. */
    public enum CoreOutTag implements Wire {
        VALUE("core_out");

        private final String wire;

        CoreOutTag(String wire) {
            this.wire = wire;
        }

        public String wire() {
            return wire;
        }

        public static CoreOutTag read(Object $json) {
            String $word = Json.asText($json);
            if (!"core_out".equals($word)) {
                throw new IllegalArgumentException("not a CoreOutTag: " + $word);
            }
            return VALUE;
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, wire);
        }
    }

    /** One cue. */
    public static final class Cue implements Wire {
        private final long endMs;
        private final long startMs;
        private final String text;

        public Cue(long endMs, long startMs, String text) {
            this.endMs = endMs;
            this.startMs = startMs;
            this.text = text;
        }

        public long getEndMs() {
            return endMs;
        }

        public long getStartMs() {
            return startMs;
        }

        public String getText() {
            return text;
        }

        public static Cue read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "end_ms", "Cue");
            Object $raw1 = Json.need($fields, "start_ms", "Cue");
            Object $raw2 = Json.need($fields, "text", "Cue");
            return new Cue(
                    Json.asLong($raw0),
                    Json.asLong($raw1),
                    Json.asText($raw2));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "end_ms");
            $out.append(endMs);
            $count = Json.key($out, $count, "start_ms");
            $out.append(startMs);
            $count = Json.key($out, $count, "text");
            Json.text($out, text);
            $out.append('}');
        }
    }

    // An opaque position in a list. Clients hand it back and never read it.
    // Cursor is String. Java has no type alias, so every use of it reads String.

    /** True when the server wants the whole card. */
    public static final class DeviceCardAnswer implements Wire {
        private final boolean sendCard;

        public DeviceCardAnswer(boolean sendCard) {
            this.sendCard = sendCard;
        }

        public boolean getSendCard() {
            return sendCard;
        }

        public static DeviceCardAnswer read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "send_card", "DeviceCardAnswer");
            return new DeviceCardAnswer(
                    Json.asBool($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "send_card");
            $out.append(sendCard);
            $out.append('}');
        }
    }

    /** Sent at each start. The card travels only when the server asks or the app or OS changed. */
    public static final class DeviceCardRequest implements Wire {
        private final CapabilityCard card;
        private final String cardHash;
        private final String deviceKey;

        public DeviceCardRequest(CapabilityCard card, String cardHash, String deviceKey) {
            this.card = card;
            this.cardHash = cardHash;
            this.deviceKey = deviceKey;
        }

        public boolean hasCard() {
            return card != null;
        }

        public CapabilityCard getCard() {
            return card == null ? null : card;
        }

        public String getCardHash() {
            return cardHash;
        }

        public String getDeviceKey() {
            return deviceKey;
        }

        public static DeviceCardRequest read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "card");
            Object $raw1 = Json.need($fields, "card_hash", "DeviceCardRequest");
            Object $raw2 = Json.need($fields, "device_key", "DeviceCardRequest");
            return new DeviceCardRequest(
                    $raw0 == null ? null : CapabilityCard.read($raw0),
                    Json.asText($raw1),
                    Json.asText($raw2));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (card != null) {
                $count = Json.key($out, $count, "card");
                card.write($out);
            }
            $count = Json.key($out, $count, "card_hash");
            Json.text($out, cardHash);
            $count = Json.key($out, $count, "device_key");
            Json.text($out, deviceKey);
            $out.append('}');
        }
    }

    /** Client class. Minimum versions, start deadlines and reachability are kept per class. */
    public static final class DeviceClass implements Wire {
        public static final DeviceClass WEB = new DeviceClass("web");
        public static final DeviceClass PHONE = new DeviceClass("phone");
        public static final DeviceClass TABLET = new DeviceClass("tablet");
        public static final DeviceClass DESKTOP = new DeviceClass("desktop");
        public static final DeviceClass TV = new DeviceClass("tv");
        public static final DeviceClass BOX = new DeviceClass("box");
        public static final DeviceClass CAR = new DeviceClass("car");

        private final String value;

        public DeviceClass(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static DeviceClass read(Object $json) {
            return new DeviceClass(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof DeviceClass && (((DeviceClass) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** Device sign in. The screen shows `user_code` even when it shows a QR code. */
    public static final class DeviceCode implements Wire {
        private final String deviceCode;
        private final long expiresIn;
        private final long interval;
        private final String userCode;
        private final String verificationUri;
        private final String verificationUriComplete;

        public DeviceCode(String deviceCode, long expiresIn, long interval, String userCode, String verificationUri, String verificationUriComplete) {
            this.deviceCode = deviceCode;
            this.expiresIn = expiresIn;
            this.interval = interval;
            this.userCode = userCode;
            this.verificationUri = verificationUri;
            this.verificationUriComplete = verificationUriComplete;
        }

        public String getDeviceCode() {
            return deviceCode;
        }

        public long getExpiresIn() {
            return expiresIn;
        }

        public long getInterval() {
            return interval;
        }

        public String getUserCode() {
            return userCode;
        }

        public String getVerificationUri() {
            return verificationUri;
        }

        public String getVerificationUriComplete() {
            return verificationUriComplete;
        }

        public static DeviceCode read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "device_code", "DeviceCode");
            Object $raw1 = Json.need($fields, "expires_in", "DeviceCode");
            Object $raw2 = Json.need($fields, "interval", "DeviceCode");
            Object $raw3 = Json.need($fields, "user_code", "DeviceCode");
            Object $raw4 = Json.need($fields, "verification_uri", "DeviceCode");
            Object $raw5 = Json.need($fields, "verification_uri_complete", "DeviceCode");
            return new DeviceCode(
                    Json.asText($raw0),
                    Json.asLong($raw1),
                    Json.asLong($raw2),
                    Json.asText($raw3),
                    Json.asText($raw4),
                    Json.asText($raw5));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "device_code");
            Json.text($out, deviceCode);
            $count = Json.key($out, $count, "expires_in");
            $out.append(expiresIn);
            $count = Json.key($out, $count, "interval");
            $out.append(interval);
            $count = Json.key($out, $count, "user_code");
            Json.text($out, userCode);
            $count = Json.key($out, $count, "verification_uri");
            Json.text($out, verificationUri);
            $count = Json.key($out, $count, "verification_uri_complete");
            Json.text($out, verificationUriComplete);
            $out.append('}');
        }
    }

    public static final class DeviceCodeRequest implements Wire {
        private final String deviceName;

        public DeviceCodeRequest(String deviceName) {
            this.deviceName = deviceName;
        }

        public String getDeviceName() {
            return deviceName;
        }

        public static DeviceCodeRequest read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "device_name", "DeviceCodeRequest");
            return new DeviceCodeRequest(
                    Json.asText($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "device_name");
            Json.text($out, deviceName);
            $out.append('}');
        }
    }

    public static final class DeviceDecision implements Wire {
        private final String userCode;

        public DeviceDecision(String userCode) {
            this.userCode = userCode;
        }

        public String getUserCode() {
            return userCode;
        }

        public static DeviceDecision read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "user_code", "DeviceDecision");
            return new DeviceDecision(
                    Json.asText($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "user_code");
            Json.text($out, userCode);
            $out.append('}');
        }
    }

    public static final class DeviceList implements Wire {
        private final List<DeviceSession> devices;

        public DeviceList(List<DeviceSession> devices) {
            this.devices = devices;
        }

        public List<DeviceSession> getDevicesList() {
            return devices == null ? Collections.<DeviceSession>emptyList() : devices;
        }

        public int getDevicesCount() {
            return devices == null ? 0 : devices.size();
        }

        public DeviceSession getDevices(int $index) {
            return devices.get($index);
        }

        public static DeviceList read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "devices", "DeviceList");
            return new DeviceList(
                    Json.list($raw0, $x0 -> DeviceSession.read($x0)));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "devices");
            $out.append('[');
            for (int $i0 = 0; $i0 < devices.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                devices.get($i0).write($out);
            }
            $out.append(']');
            $out.append('}');
        }
    }

    /** Answers while a device waits for approval. */
    public static final class DevicePoll implements Wire {
        public static final DevicePoll AUTHORIZATION_PENDING = new DevicePoll("authorization_pending");
        public static final DevicePoll SLOW_DOWN = new DevicePoll("slow_down");
        public static final DevicePoll ACCESS_DENIED = new DevicePoll("access_denied");
        public static final DevicePoll EXPIRED_TOKEN = new DevicePoll("expired_token");

        private final String value;

        public DevicePoll(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static DevicePoll read(Object $json) {
            return new DevicePoll(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof DevicePoll && (((DevicePoll) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** One signed in device. */
    public static final class DeviceSession implements Wire {
        private final DeviceClass class_;
        private final boolean current;
        private final String id;
        private final long lastSeenMs;
        private final String name;

        public DeviceSession(DeviceClass class_, boolean current, String id, long lastSeenMs, String name) {
            this.class_ = class_;
            this.current = current;
            this.id = id;
            this.lastSeenMs = lastSeenMs;
            this.name = name;
        }

        public DeviceClass getClass_() {
            return class_;
        }

        public boolean getCurrent() {
            return current;
        }

        public String getId() {
            return id;
        }

        public long getLastSeenMs() {
            return lastSeenMs;
        }

        public String getName() {
            return name;
        }

        public static DeviceSession read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "class", "DeviceSession");
            Object $raw1 = Json.need($fields, "current", "DeviceSession");
            Object $raw2 = Json.need($fields, "id", "DeviceSession");
            Object $raw3 = Json.need($fields, "last_seen_ms", "DeviceSession");
            Object $raw4 = Json.need($fields, "name", "DeviceSession");
            return new DeviceSession(
                    DeviceClass.read($raw0),
                    Json.asBool($raw1),
                    Json.asText($raw2),
                    Json.asLong($raw3),
                    Json.asText($raw4));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "class");
            class_.write($out);
            $count = Json.key($out, $count, "current");
            $out.append(current);
            $count = Json.key($out, $count, "id");
            Json.text($out, id);
            $count = Json.key($out, $count, "last_seen_ms");
            $out.append(lastSeenMs);
            $count = Json.key($out, $count, "name");
            Json.text($out, name);
            $out.append('}');
        }
    }

    /** Where a device sign in stands. */
    public static final class DeviceSignInState implements Wire {
        public static final DeviceSignInState IDLE = new DeviceSignInState("idle");
        public static final DeviceSignInState WAITING = new DeviceSignInState("waiting");
        public static final DeviceSignInState DENIED = new DeviceSignInState("denied");
        public static final DeviceSignInState EXPIRED = new DeviceSignInState("expired");
        public static final DeviceSignInState OFF = new DeviceSignInState("off");

        private final String value;

        public DeviceSignInState(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static DeviceSignInState read(Object $json) {
            return new DeviceSignInState(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof DeviceSignInState && (((DeviceSignInState) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** Device sign in state for screens. A dead code is left out, so no screen can show one, and the device code has no field here. */
    public static final class DeviceSignInView implements Wire {
        private final Long expiresIn;
        private final DeviceSignInState state;
        private final String userCode;
        private final String verificationUri;
        private final String verificationUriComplete;

        public DeviceSignInView(Long expiresIn, DeviceSignInState state, String userCode, String verificationUri, String verificationUriComplete) {
            this.expiresIn = expiresIn;
            this.state = state;
            this.userCode = userCode;
            this.verificationUri = verificationUri;
            this.verificationUriComplete = verificationUriComplete;
        }

        /** Seconds this code lives, counted from when it reached this view. */
        public boolean hasExpiresIn() {
            return expiresIn != null;
        }

        public long getExpiresIn() {
            return expiresIn == null ? 0L : expiresIn;
        }

        public DeviceSignInState getState() {
            return state;
        }

        /** The screen shows this even when it shows a QR code. */
        public boolean hasUserCode() {
            return userCode != null;
        }

        public String getUserCode() {
            return userCode == null ? "" : userCode;
        }

        public boolean hasVerificationUri() {
            return verificationUri != null;
        }

        public String getVerificationUri() {
            return verificationUri == null ? "" : verificationUri;
        }

        /** The same address with the code in it, for a QR code. */
        public boolean hasVerificationUriComplete() {
            return verificationUriComplete != null;
        }

        public String getVerificationUriComplete() {
            return verificationUriComplete == null ? "" : verificationUriComplete;
        }

        public static DeviceSignInView read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "expires_in");
            Object $raw1 = Json.need($fields, "state", "DeviceSignInView");
            Object $raw2 = Json.at($fields, "user_code");
            Object $raw3 = Json.at($fields, "verification_uri");
            Object $raw4 = Json.at($fields, "verification_uri_complete");
            return new DeviceSignInView(
                    $raw0 == null ? null : Long.valueOf(Json.asLong($raw0)),
                    DeviceSignInState.read($raw1),
                    $raw2 == null ? null : Json.asText($raw2),
                    $raw3 == null ? null : Json.asText($raw3),
                    $raw4 == null ? null : Json.asText($raw4));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (expiresIn != null) {
                $count = Json.key($out, $count, "expires_in");
                $out.append(expiresIn);
            }
            $count = Json.key($out, $count, "state");
            state.write($out);
            if (userCode != null) {
                $count = Json.key($out, $count, "user_code");
                Json.text($out, userCode);
            }
            if (verificationUri != null) {
                $count = Json.key($out, $count, "verification_uri");
                Json.text($out, verificationUri);
            }
            if (verificationUriComplete != null) {
                $count = Json.key($out, $count, "verification_uri_complete");
                Json.text($out, verificationUriComplete);
            }
            $out.append('}');
        }
    }

    /** Either tokens or a word saying to wait or stop. */
    public static final class DeviceTokenAnswer implements Wire {
        private final DevicePoll poll;
        private final SignedIn signedIn;

        public DeviceTokenAnswer(DevicePoll poll, SignedIn signedIn) {
            this.poll = poll;
            this.signedIn = signedIn;
        }

        public boolean hasPoll() {
            return poll != null;
        }

        public DevicePoll getPoll() {
            return poll == null ? null : poll;
        }

        public boolean hasSignedIn() {
            return signedIn != null;
        }

        public SignedIn getSignedIn() {
            return signedIn == null ? null : signedIn;
        }

        public static DeviceTokenAnswer read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "poll");
            Object $raw1 = Json.at($fields, "signed_in");
            return new DeviceTokenAnswer(
                    $raw0 == null ? null : DevicePoll.read($raw0),
                    $raw1 == null ? null : SignedIn.read($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (poll != null) {
                $count = Json.key($out, $count, "poll");
                poll.write($out);
            }
            if (signedIn != null) {
                $count = Json.key($out, $count, "signed_in");
                signedIn.write($out);
            }
            $out.append('}');
        }
    }

    public static final class DeviceTokenRequest implements Wire {
        private final String deviceCode;

        public DeviceTokenRequest(String deviceCode) {
            this.deviceCode = deviceCode;
        }

        public String getDeviceCode() {
            return deviceCode;
        }

        public static DeviceTokenRequest read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "device_code", "DeviceTokenRequest");
            return new DeviceTokenRequest(
                    Json.asText($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "device_code");
            Json.text($out, deviceCode);
            $out.append('}');
        }
    }

    /** Answer with no data. */
    public static final class Done implements Wire {
        private final boolean done;

        public Done(boolean done) {
            this.done = done;
        }

        public boolean getDone() {
            return done;
        }

        public static Done read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "done", "Done");
            return new Done(
                    Json.asBool($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "done");
            $out.append(done);
            $out.append('}');
        }
    }

    /** A command to the player. Attempts carry their headers here. */
    public abstract static class DriverCommand implements Wire {
        public static DriverCommand read(Object $json) {
            String $tag = Json.asText(Json.need(Json.object($json), "op", "DriverCommand"));
            if ("open".equals($tag)) {
                return DriverCommandOpen.read($json);
            }
            if ("play".equals($tag)) {
                return DriverCommandPlay.read($json);
            }
            if ("pause".equals($tag)) {
                return DriverCommandPause.read($json);
            }
            if ("seek".equals($tag)) {
                return DriverCommandSeek.read($json);
            }
            if ("rate".equals($tag)) {
                return DriverCommandRate.read($json);
            }
            if ("volume".equals($tag)) {
                return DriverCommandVolume.read($json);
            }
            if ("select_audio".equals($tag)) {
                return DriverCommandSelectAudio.read($json);
            }
            if ("select_text".equals($tag)) {
                return DriverCommandSelectText.read($json);
            }
            if ("close".equals($tag)) {
                return DriverCommandClose.read($json);
            }
            return DriverCommandUnknown.read($json);
        }
    }

    public static final class DriverCommandOpen extends DriverCommand {
        private final long generation;
        private final Map<String, String> headers;
        private final Long maxBitrate;
        private final Long maxHeight;
        private final long startAtMs;
        private final String url;

        public DriverCommandOpen(long generation, Map<String, String> headers, Long maxBitrate, Long maxHeight, long startAtMs, String url) {
            this.generation = generation;
            this.headers = headers;
            this.maxBitrate = maxBitrate;
            this.maxHeight = maxHeight;
            this.startAtMs = startAtMs;
            this.url = url;
        }

        public long getGeneration() {
            return generation;
        }

        public Map<String, String> getHeadersMap() {
            return headers == null ? Collections.<String, String>emptyMap() : headers;
        }

        public int getHeadersCount() {
            return headers == null ? 0 : headers.size();
        }

        public boolean hasMaxBitrate() {
            return maxBitrate != null;
        }

        public long getMaxBitrate() {
            return maxBitrate == null ? 0L : maxBitrate;
        }

        public boolean hasMaxHeight() {
            return maxHeight != null;
        }

        public long getMaxHeight() {
            return maxHeight == null ? 0L : maxHeight;
        }

        public long getStartAtMs() {
            return startAtMs;
        }

        public String getUrl() {
            return url;
        }

        public static DriverCommandOpen read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"open".equals(Json.asText(Json.need($fields, "op", "DriverCommand")))) {
                throw new IllegalArgumentException("not a DriverCommandOpen");
            }
            Object $raw0 = Json.need($fields, "generation", "DriverCommand");
            Object $raw1 = Json.need($fields, "headers", "DriverCommand");
            Object $raw2 = Json.at($fields, "max_bitrate");
            Object $raw3 = Json.at($fields, "max_height");
            Object $raw4 = Json.need($fields, "start_at_ms", "DriverCommand");
            Object $raw5 = Json.need($fields, "url", "DriverCommand");
            return new DriverCommandOpen(
                    Json.asLong($raw0),
                    Json.map($raw1, $x0 -> Json.asText($x0)),
                    $raw2 == null ? null : Long.valueOf(Json.asLong($raw2)),
                    $raw3 == null ? null : Long.valueOf(Json.asLong($raw3)),
                    Json.asLong($raw4),
                    Json.asText($raw5));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "op");
            Json.text($out, "open");
            $count = Json.key($out, $count, "generation");
            $out.append(generation);
            $count = Json.key($out, $count, "headers");
            $out.append('{');
            int $n0 = 0;
            for (Map.Entry<String, String> $e0 : headers.entrySet()) {
                $n0 = Json.key($out, $n0, $e0.getKey());
                Json.text($out, $e0.getValue());
            }
            $out.append('}');
            if (maxBitrate != null) {
                $count = Json.key($out, $count, "max_bitrate");
                $out.append(maxBitrate);
            }
            if (maxHeight != null) {
                $count = Json.key($out, $count, "max_height");
                $out.append(maxHeight);
            }
            $count = Json.key($out, $count, "start_at_ms");
            $out.append(startAtMs);
            $count = Json.key($out, $count, "url");
            Json.text($out, url);
            $out.append('}');
        }
    }

    public static final class DriverCommandPlay extends DriverCommand {
        public DriverCommandPlay() {
        }

        public static DriverCommandPlay read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"play".equals(Json.asText(Json.need($fields, "op", "DriverCommand")))) {
                throw new IllegalArgumentException("not a DriverCommandPlay");
            }
            return new DriverCommandPlay();
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "op");
            Json.text($out, "play");
            $out.append('}');
        }
    }

    public static final class DriverCommandPause extends DriverCommand {
        public DriverCommandPause() {
        }

        public static DriverCommandPause read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"pause".equals(Json.asText(Json.need($fields, "op", "DriverCommand")))) {
                throw new IllegalArgumentException("not a DriverCommandPause");
            }
            return new DriverCommandPause();
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "op");
            Json.text($out, "pause");
            $out.append('}');
        }
    }

    public static final class DriverCommandSeek extends DriverCommand {
        private final long toMs;

        public DriverCommandSeek(long toMs) {
            this.toMs = toMs;
        }

        public long getToMs() {
            return toMs;
        }

        public static DriverCommandSeek read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"seek".equals(Json.asText(Json.need($fields, "op", "DriverCommand")))) {
                throw new IllegalArgumentException("not a DriverCommandSeek");
            }
            Object $raw0 = Json.need($fields, "to_ms", "DriverCommand");
            return new DriverCommandSeek(
                    Json.asLong($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "op");
            Json.text($out, "seek");
            $count = Json.key($out, $count, "to_ms");
            $out.append(toMs);
            $out.append('}');
        }
    }

    public static final class DriverCommandRate extends DriverCommand {
        private final long percent;

        public DriverCommandRate(long percent) {
            this.percent = percent;
        }

        public long getPercent() {
            return percent;
        }

        public static DriverCommandRate read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"rate".equals(Json.asText(Json.need($fields, "op", "DriverCommand")))) {
                throw new IllegalArgumentException("not a DriverCommandRate");
            }
            Object $raw0 = Json.need($fields, "percent", "DriverCommand");
            return new DriverCommandRate(
                    Json.asLong($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "op");
            Json.text($out, "rate");
            $count = Json.key($out, $count, "percent");
            $out.append(percent);
            $out.append('}');
        }
    }

    public static final class DriverCommandVolume extends DriverCommand {
        private final long percent;

        public DriverCommandVolume(long percent) {
            this.percent = percent;
        }

        public long getPercent() {
            return percent;
        }

        public static DriverCommandVolume read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"volume".equals(Json.asText(Json.need($fields, "op", "DriverCommand")))) {
                throw new IllegalArgumentException("not a DriverCommandVolume");
            }
            Object $raw0 = Json.need($fields, "percent", "DriverCommand");
            return new DriverCommandVolume(
                    Json.asLong($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "op");
            Json.text($out, "volume");
            $count = Json.key($out, $count, "percent");
            $out.append(percent);
            $out.append('}');
        }
    }

    public static final class DriverCommandSelectAudio extends DriverCommand {
        private final String track;

        public DriverCommandSelectAudio(String track) {
            this.track = track;
        }

        public String getTrack() {
            return track;
        }

        public static DriverCommandSelectAudio read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"select_audio".equals(Json.asText(Json.need($fields, "op", "DriverCommand")))) {
                throw new IllegalArgumentException("not a DriverCommandSelectAudio");
            }
            Object $raw0 = Json.need($fields, "track", "DriverCommand");
            return new DriverCommandSelectAudio(
                    Json.asText($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "op");
            Json.text($out, "select_audio");
            $count = Json.key($out, $count, "track");
            Json.text($out, track);
            $out.append('}');
        }
    }

    public static final class DriverCommandSelectText extends DriverCommand {
        private final String track;

        public DriverCommandSelectText(String track) {
            this.track = track;
        }

        public boolean hasTrack() {
            return track != null;
        }

        public String getTrack() {
            return track == null ? "" : track;
        }

        public static DriverCommandSelectText read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"select_text".equals(Json.asText(Json.need($fields, "op", "DriverCommand")))) {
                throw new IllegalArgumentException("not a DriverCommandSelectText");
            }
            Object $raw0 = Json.at($fields, "track");
            return new DriverCommandSelectText(
                    $raw0 == null ? null : Json.asText($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "op");
            Json.text($out, "select_text");
            if (track != null) {
                $count = Json.key($out, $count, "track");
                Json.text($out, track);
            }
            $out.append('}');
        }
    }

    public static final class DriverCommandClose extends DriverCommand {
        public DriverCommandClose() {
        }

        public static DriverCommandClose read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"close".equals(Json.asText(Json.need($fields, "op", "DriverCommand")))) {
                throw new IllegalArgumentException("not a DriverCommandClose");
            }
            return new DriverCommandClose();
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "op");
            Json.text($out, "close");
            $out.append('}');
        }
    }

    /** A tag this build does not know. */
    public static final class DriverCommandUnknown extends DriverCommand {
        private final String tag;

        public DriverCommandUnknown(String tag) {
            this.tag = tag;
        }

        /** The tag that was read. */
        public String getTag() {
            return tag;
        }

        public static DriverCommandUnknown read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "op");
            return new DriverCommandUnknown(
                    $raw0 == null ? null : Json.asText($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "op");
            Json.text($out, tag);
            $out.append('}');
        }
    }

    /** What the driver reports. No driver decides failover or no picture by itself. */
    public abstract static class DriverEvent implements Wire {
        public static DriverEvent read(Object $json) {
            String $tag = Json.asText(Json.need(Json.object($json), "kind", "DriverEvent"));
            if ("first_frame".equals($tag)) {
                return DriverEventFirstFrame.read($json);
            }
            if ("position".equals($tag)) {
                return DriverEventPosition.read($json);
            }
            if ("stalled".equals($tag)) {
                return DriverEventStalled.read($json);
            }
            if ("ended".equals($tag)) {
                return DriverEventEnded.read($json);
            }
            if ("tracks".equals($tag)) {
                return DriverEventTracks.read($json);
            }
            if ("error".equals($tag)) {
                return DriverEventError.read($json);
            }
            return DriverEventUnknown.read($json);
        }
    }

    public static final class DriverEventFirstFrame extends DriverEvent {
        public DriverEventFirstFrame() {
        }

        public static DriverEventFirstFrame read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"first_frame".equals(Json.asText(Json.need($fields, "kind", "DriverEvent")))) {
                throw new IllegalArgumentException("not a DriverEventFirstFrame");
            }
            return new DriverEventFirstFrame();
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "kind");
            Json.text($out, "first_frame");
            $out.append('}');
        }
    }

    public static final class DriverEventPosition extends DriverEvent {
        private final Long durationMs;
        private final Boolean hasPicture;
        private final long timeMs;
        private final Long videoWidth;

        public DriverEventPosition(Long durationMs, Boolean hasPicture, long timeMs, Long videoWidth) {
            this.durationMs = durationMs;
            this.hasPicture = hasPicture;
            this.timeMs = timeMs;
            this.videoWidth = videoWidth;
        }

        /** 0 or absent when not known. */
        public boolean hasDurationMs() {
            return durationMs != null;
        }

        public long getDurationMs() {
            return durationMs == null ? 0L : durationMs;
        }

        /** Absent means unknown, and the no picture rule stays off. */
        public boolean hasHasPicture() {
            return hasPicture != null;
        }

        public boolean getHasPicture() {
            return hasPicture != null && hasPicture;
        }

        public long getTimeMs() {
            return timeMs;
        }

        public boolean hasVideoWidth() {
            return videoWidth != null;
        }

        public long getVideoWidth() {
            return videoWidth == null ? 0L : videoWidth;
        }

        public static DriverEventPosition read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"position".equals(Json.asText(Json.need($fields, "kind", "DriverEvent")))) {
                throw new IllegalArgumentException("not a DriverEventPosition");
            }
            Object $raw0 = Json.at($fields, "duration_ms");
            Object $raw1 = Json.at($fields, "has_picture");
            Object $raw2 = Json.need($fields, "time_ms", "DriverEvent");
            Object $raw3 = Json.at($fields, "video_width");
            return new DriverEventPosition(
                    $raw0 == null ? null : Long.valueOf(Json.asLong($raw0)),
                    $raw1 == null ? null : Boolean.valueOf(Json.asBool($raw1)),
                    Json.asLong($raw2),
                    $raw3 == null ? null : Long.valueOf(Json.asLong($raw3)));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "kind");
            Json.text($out, "position");
            if (durationMs != null) {
                $count = Json.key($out, $count, "duration_ms");
                $out.append(durationMs);
            }
            if (hasPicture != null) {
                $count = Json.key($out, $count, "has_picture");
                $out.append(hasPicture);
            }
            $count = Json.key($out, $count, "time_ms");
            $out.append(timeMs);
            if (videoWidth != null) {
                $count = Json.key($out, $count, "video_width");
                $out.append(videoWidth);
            }
            $out.append('}');
        }
    }

    public static final class DriverEventStalled extends DriverEvent {
        public DriverEventStalled() {
        }

        public static DriverEventStalled read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"stalled".equals(Json.asText(Json.need($fields, "kind", "DriverEvent")))) {
                throw new IllegalArgumentException("not a DriverEventStalled");
            }
            return new DriverEventStalled();
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "kind");
            Json.text($out, "stalled");
            $out.append('}');
        }
    }

    public static final class DriverEventEnded extends DriverEvent {
        public DriverEventEnded() {
        }

        public static DriverEventEnded read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"ended".equals(Json.asText(Json.need($fields, "kind", "DriverEvent")))) {
                throw new IllegalArgumentException("not a DriverEventEnded");
            }
            return new DriverEventEnded();
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "kind");
            Json.text($out, "ended");
            $out.append('}');
        }
    }

    public static final class DriverEventTracks extends DriverEvent {
        private final List<Track> audio;
        private final List<Track> text;

        public DriverEventTracks(List<Track> audio, List<Track> text) {
            this.audio = audio;
            this.text = text;
        }

        public List<Track> getAudioList() {
            return audio == null ? Collections.<Track>emptyList() : audio;
        }

        public int getAudioCount() {
            return audio == null ? 0 : audio.size();
        }

        public Track getAudio(int $index) {
            return audio.get($index);
        }

        public List<Track> getTextList() {
            return text == null ? Collections.<Track>emptyList() : text;
        }

        public int getTextCount() {
            return text == null ? 0 : text.size();
        }

        public Track getText(int $index) {
            return text.get($index);
        }

        public static DriverEventTracks read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"tracks".equals(Json.asText(Json.need($fields, "kind", "DriverEvent")))) {
                throw new IllegalArgumentException("not a DriverEventTracks");
            }
            Object $raw0 = Json.need($fields, "audio", "DriverEvent");
            Object $raw1 = Json.need($fields, "text", "DriverEvent");
            return new DriverEventTracks(
                    Json.list($raw0, $x0 -> Track.read($x0)),
                    Json.list($raw1, $x0 -> Track.read($x0)));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "kind");
            Json.text($out, "tracks");
            $count = Json.key($out, $count, "audio");
            $out.append('[');
            for (int $i0 = 0; $i0 < audio.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                audio.get($i0).write($out);
            }
            $out.append(']');
            $count = Json.key($out, $count, "text");
            $out.append('[');
            for (int $i0 = 0; $i0 < text.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                text.get($i0).write($out);
            }
            $out.append(']');
            $out.append('}');
        }
    }

    public static final class DriverEventError extends DriverEvent {
        private final ErrorClass class_;

        public DriverEventError(ErrorClass class_) {
            this.class_ = class_;
        }

        public ErrorClass getClass_() {
            return class_;
        }

        public static DriverEventError read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"error".equals(Json.asText(Json.need($fields, "kind", "DriverEvent")))) {
                throw new IllegalArgumentException("not a DriverEventError");
            }
            Object $raw0 = Json.need($fields, "class", "DriverEvent");
            return new DriverEventError(
                    ErrorClass.read($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "kind");
            Json.text($out, "error");
            $count = Json.key($out, $count, "class");
            class_.write($out);
            $out.append('}');
        }
    }

    /** A tag this build does not know. */
    public static final class DriverEventUnknown extends DriverEvent {
        private final String tag;

        public DriverEventUnknown(String tag) {
            this.tag = tag;
        }

        /** The tag that was read. */
        public String getTag() {
            return tag;
        }

        public static DriverEventUnknown read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "kind");
            return new DriverEventUnknown(
                    $raw0 == null ? null : Json.asText($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "kind");
            Json.text($out, tag);
            $out.append('}');
        }
    }

    /** One effect. The result comes back as an event with this id. */
    public abstract static class Effect implements Wire {
        public abstract String getId();

        public static Effect read(Object $json) {
            String $tag = Json.asText(Json.need(Json.object($json), "kind", "Effect"));
            if ("http".equals($tag)) {
                return EffectHttp.read($json);
            }
            if ("store".equals($tag)) {
                return EffectStore.read($json);
            }
            if ("secret".equals($tag)) {
                return EffectSecret.read($json);
            }
            if ("timer".equals($tag)) {
                return EffectTimer.read($json);
            }
            if ("player".equals($tag)) {
                return EffectPlayer.read($json);
            }
            if ("view".equals($tag)) {
                return EffectView.read($json);
            }
            if ("open".equals($tag)) {
                return EffectOpen.read($json);
            }
            if ("log".equals($tag)) {
                return EffectLog.read($json);
            }
            return EffectUnknown.read($json);
        }
    }

    /** A path, never an address. The host holds the base address. */
    public static final class EffectHttp extends Effect {
        private final String id;
        private final String body;
        private final HttpClass class_;
        private final long deadlineMs;
        private final Map<String, String> headers;
        private final HttpMethod method;
        private final String path;
        private final String scope;
        private final BodyTarget to;

        public EffectHttp(String id, String body, HttpClass class_, long deadlineMs, Map<String, String> headers, HttpMethod method, String path, String scope, BodyTarget to) {
            this.id = id;
            this.body = body;
            this.class_ = class_;
            this.deadlineMs = deadlineMs;
            this.headers = headers;
            this.method = method;
            this.path = path;
            this.scope = scope;
            this.to = to;
        }

        @Override
        public String getId() {
            return id;
        }

        public boolean hasBody() {
            return body != null;
        }

        public String getBody() {
            return body == null ? "" : body;
        }

        public HttpClass getClass_() {
            return class_;
        }

        public long getDeadlineMs() {
            return deadlineMs;
        }

        public Map<String, String> getHeadersMap() {
            return headers == null ? Collections.<String, String>emptyMap() : headers;
        }

        public int getHeadersCount() {
            return headers == null ? 0 : headers.size();
        }

        public HttpMethod getMethod() {
            return method;
        }

        public String getPath() {
            return path;
        }

        /** Opaque scope of an account and profile, for kept copies. */
        public boolean hasScope() {
            return scope != null;
        }

        public String getScope() {
            return scope == null ? "" : scope;
        }

        public BodyTarget getTo() {
            return to;
        }

        public static EffectHttp read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"http".equals(Json.asText(Json.need($fields, "kind", "Effect")))) {
                throw new IllegalArgumentException("not a EffectHttp");
            }
            Object $raw0 = Json.need($fields, "id", "Effect");
            Object $raw1 = Json.at($fields, "body");
            Object $raw2 = Json.need($fields, "class", "Effect");
            Object $raw3 = Json.need($fields, "deadline_ms", "Effect");
            Object $raw4 = Json.at($fields, "headers");
            Object $raw5 = Json.need($fields, "method", "Effect");
            Object $raw6 = Json.need($fields, "path", "Effect");
            Object $raw7 = Json.at($fields, "scope");
            Object $raw8 = Json.need($fields, "to", "Effect");
            return new EffectHttp(
                    Json.asText($raw0),
                    $raw1 == null ? null : Json.asText($raw1),
                    HttpClass.read($raw2),
                    Json.asLong($raw3),
                    $raw4 == null ? null : Json.map($raw4, $x0 -> Json.asText($x0)),
                    HttpMethod.read($raw5),
                    Json.asText($raw6),
                    $raw7 == null ? null : Json.asText($raw7),
                    BodyTarget.read($raw8));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "kind");
            Json.text($out, "http");
            $count = Json.key($out, $count, "id");
            Json.text($out, id);
            if (body != null) {
                $count = Json.key($out, $count, "body");
                Json.text($out, body);
            }
            $count = Json.key($out, $count, "class");
            class_.write($out);
            $count = Json.key($out, $count, "deadline_ms");
            $out.append(deadlineMs);
            if (headers != null) {
                $count = Json.key($out, $count, "headers");
                $out.append('{');
                int $n0 = 0;
                for (Map.Entry<String, String> $e0 : headers.entrySet()) {
                    $n0 = Json.key($out, $n0, $e0.getKey());
                    Json.text($out, $e0.getValue());
                }
                $out.append('}');
            }
            $count = Json.key($out, $count, "method");
            method.write($out);
            $count = Json.key($out, $count, "path");
            Json.text($out, path);
            if (scope != null) {
                $count = Json.key($out, $count, "scope");
                Json.text($out, scope);
            }
            $count = Json.key($out, $count, "to");
            to.write($out);
            $out.append('}');
        }
    }

    public static final class EffectStore extends Effect {
        private final String id;
        private final String key;
        private final StoreOp op;
        private final String value;

        public EffectStore(String id, String key, StoreOp op, String value) {
            this.id = id;
            this.key = key;
            this.op = op;
            this.value = value;
        }

        @Override
        public String getId() {
            return id;
        }

        public boolean hasKey() {
            return key != null;
        }

        public String getKey() {
            return key == null ? "" : key;
        }

        public StoreOp getOp() {
            return op;
        }

        public boolean hasValue() {
            return value != null;
        }

        public String getValue() {
            return value == null ? "" : value;
        }

        public static EffectStore read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"store".equals(Json.asText(Json.need($fields, "kind", "Effect")))) {
                throw new IllegalArgumentException("not a EffectStore");
            }
            Object $raw0 = Json.need($fields, "id", "Effect");
            Object $raw1 = Json.at($fields, "key");
            Object $raw2 = Json.need($fields, "op", "Effect");
            Object $raw3 = Json.at($fields, "value");
            return new EffectStore(
                    Json.asText($raw0),
                    $raw1 == null ? null : Json.asText($raw1),
                    StoreOp.read($raw2),
                    $raw3 == null ? null : Json.asText($raw3));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "kind");
            Json.text($out, "store");
            $count = Json.key($out, $count, "id");
            Json.text($out, id);
            if (key != null) {
                $count = Json.key($out, $count, "key");
                Json.text($out, key);
            }
            $count = Json.key($out, $count, "op");
            op.write($out);
            if (value != null) {
                $count = Json.key($out, $count, "value");
                Json.text($out, value);
            }
            $out.append('}');
        }
    }

    public static final class EffectSecret extends Effect {
        private final String id;
        private final String name;
        private final SecretOp op;
        private final String value;

        public EffectSecret(String id, String name, SecretOp op, String value) {
            this.id = id;
            this.name = name;
            this.op = op;
            this.value = value;
        }

        @Override
        public String getId() {
            return id;
        }

        public String getName() {
            return name;
        }

        public SecretOp getOp() {
            return op;
        }

        public boolean hasValue() {
            return value != null;
        }

        public String getValue() {
            return value == null ? "" : value;
        }

        public static EffectSecret read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"secret".equals(Json.asText(Json.need($fields, "kind", "Effect")))) {
                throw new IllegalArgumentException("not a EffectSecret");
            }
            Object $raw0 = Json.need($fields, "id", "Effect");
            Object $raw1 = Json.need($fields, "name", "Effect");
            Object $raw2 = Json.need($fields, "op", "Effect");
            Object $raw3 = Json.at($fields, "value");
            return new EffectSecret(
                    Json.asText($raw0),
                    Json.asText($raw1),
                    SecretOp.read($raw2),
                    $raw3 == null ? null : Json.asText($raw3));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "kind");
            Json.text($out, "secret");
            $count = Json.key($out, $count, "id");
            Json.text($out, id);
            $count = Json.key($out, $count, "name");
            Json.text($out, name);
            $count = Json.key($out, $count, "op");
            op.write($out);
            if (value != null) {
                $count = Json.key($out, $count, "value");
                Json.text($out, value);
            }
            $out.append('}');
        }
    }

    public static final class EffectTimer extends Effect {
        private final String id;
        private final Long afterMs;
        private final TimerOp op;

        public EffectTimer(String id, Long afterMs, TimerOp op) {
            this.id = id;
            this.afterMs = afterMs;
            this.op = op;
        }

        @Override
        public String getId() {
            return id;
        }

        public boolean hasAfterMs() {
            return afterMs != null;
        }

        public long getAfterMs() {
            return afterMs == null ? 0L : afterMs;
        }

        public TimerOp getOp() {
            return op;
        }

        public static EffectTimer read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"timer".equals(Json.asText(Json.need($fields, "kind", "Effect")))) {
                throw new IllegalArgumentException("not a EffectTimer");
            }
            Object $raw0 = Json.need($fields, "id", "Effect");
            Object $raw1 = Json.at($fields, "after_ms");
            Object $raw2 = Json.need($fields, "op", "Effect");
            return new EffectTimer(
                    Json.asText($raw0),
                    $raw1 == null ? null : Long.valueOf(Json.asLong($raw1)),
                    TimerOp.read($raw2));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "kind");
            Json.text($out, "timer");
            $count = Json.key($out, $count, "id");
            Json.text($out, id);
            if (afterMs != null) {
                $count = Json.key($out, $count, "after_ms");
                $out.append(afterMs);
            }
            $count = Json.key($out, $count, "op");
            op.write($out);
            $out.append('}');
        }
    }

    public static final class EffectPlayer extends Effect {
        private final String id;
        private final DriverCommand command;

        public EffectPlayer(String id, DriverCommand command) {
            this.id = id;
            this.command = command;
        }

        @Override
        public String getId() {
            return id;
        }

        public DriverCommand getCommand() {
            return command;
        }

        public static EffectPlayer read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"player".equals(Json.asText(Json.need($fields, "kind", "Effect")))) {
                throw new IllegalArgumentException("not a EffectPlayer");
            }
            Object $raw0 = Json.need($fields, "id", "Effect");
            Object $raw1 = Json.need($fields, "command", "Effect");
            return new EffectPlayer(
                    Json.asText($raw0),
                    DriverCommand.read($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "kind");
            Json.text($out, "player");
            $count = Json.key($out, $count, "id");
            Json.text($out, id);
            $count = Json.key($out, $count, "command");
            command.write($out);
            $out.append('}');
        }
    }

    /** A one time notice or a kept page. Lasting state travels in `view`. */
    public static final class EffectView extends Effect {
        private final String id;
        private final ScreenBody body;
        private final Notice notice;

        public EffectView(String id, ScreenBody body, Notice notice) {
            this.id = id;
            this.body = body;
            this.notice = notice;
        }

        @Override
        public String getId() {
            return id;
        }

        public boolean hasBody() {
            return body != null;
        }

        public ScreenBody getBody() {
            return body == null ? null : body;
        }

        public boolean hasNotice() {
            return notice != null;
        }

        public Notice getNotice() {
            return notice == null ? null : notice;
        }

        public static EffectView read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"view".equals(Json.asText(Json.need($fields, "kind", "Effect")))) {
                throw new IllegalArgumentException("not a EffectView");
            }
            Object $raw0 = Json.need($fields, "id", "Effect");
            Object $raw1 = Json.at($fields, "body");
            Object $raw2 = Json.at($fields, "notice");
            return new EffectView(
                    Json.asText($raw0),
                    $raw1 == null ? null : ScreenBody.read($raw1),
                    $raw2 == null ? null : Notice.read($raw2));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "kind");
            Json.text($out, "view");
            $count = Json.key($out, $count, "id");
            Json.text($out, id);
            if (body != null) {
                $count = Json.key($out, $count, "body");
                body.write($out);
            }
            if (notice != null) {
                $count = Json.key($out, $count, "notice");
                notice.write($out);
            }
            $out.append('}');
        }
    }

    public static final class EffectOpen extends Effect {
        private final String id;
        private final RouteTarget target;

        public EffectOpen(String id, RouteTarget target) {
            this.id = id;
            this.target = target;
        }

        @Override
        public String getId() {
            return id;
        }

        public RouteTarget getTarget() {
            return target;
        }

        public static EffectOpen read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"open".equals(Json.asText(Json.need($fields, "kind", "Effect")))) {
                throw new IllegalArgumentException("not a EffectOpen");
            }
            Object $raw0 = Json.need($fields, "id", "Effect");
            Object $raw1 = Json.need($fields, "target", "Effect");
            return new EffectOpen(
                    Json.asText($raw0),
                    RouteTarget.read($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "kind");
            Json.text($out, "open");
            $count = Json.key($out, $count, "id");
            Json.text($out, id);
            $count = Json.key($out, $count, "target");
            target.write($out);
            $out.append('}');
        }
    }

    public static final class EffectLog extends Effect {
        private final String id;
        private final String code;
        private final String detail;
        private final LogLevel level;

        public EffectLog(String id, String code, String detail, LogLevel level) {
            this.id = id;
            this.code = code;
            this.detail = detail;
            this.level = level;
        }

        @Override
        public String getId() {
            return id;
        }

        public String getCode() {
            return code;
        }

        public boolean hasDetail() {
            return detail != null;
        }

        public String getDetail() {
            return detail == null ? "" : detail;
        }

        public LogLevel getLevel() {
            return level;
        }

        public static EffectLog read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"log".equals(Json.asText(Json.need($fields, "kind", "Effect")))) {
                throw new IllegalArgumentException("not a EffectLog");
            }
            Object $raw0 = Json.need($fields, "id", "Effect");
            Object $raw1 = Json.need($fields, "code", "Effect");
            Object $raw2 = Json.at($fields, "detail");
            Object $raw3 = Json.need($fields, "level", "Effect");
            return new EffectLog(
                    Json.asText($raw0),
                    Json.asText($raw1),
                    $raw2 == null ? null : Json.asText($raw2),
                    LogLevel.read($raw3));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "kind");
            Json.text($out, "log");
            $count = Json.key($out, $count, "id");
            Json.text($out, id);
            $count = Json.key($out, $count, "code");
            Json.text($out, code);
            if (detail != null) {
                $count = Json.key($out, $count, "detail");
                Json.text($out, detail);
            }
            $count = Json.key($out, $count, "level");
            level.write($out);
            $out.append('}');
        }
    }

    /** A tag this build does not know. */
    public static final class EffectUnknown extends Effect {
        private final String tag;
        private final String rawId;

        public EffectUnknown(String tag, String rawId) {
            this.tag = tag;
            this.rawId = rawId;
        }

        /** The tag that was read. */
        public String getTag() {
            return tag;
        }

        @Override
        public String getId() {
            return rawId == null ? "" : rawId;
        }

        public static EffectUnknown read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "kind");
            Object $raw1 = Json.at($fields, "id");
            return new EffectUnknown(
                    $raw0 == null ? null : Json.asText($raw0),
                    $raw1 == null ? null : Json.asText($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "kind");
            Json.text($out, tag);
            if (rawId != null) {
                $count = Json.key($out, $count, "id");
                Json.text($out, rawId);
            }
            $out.append('}');
        }
    }

    public static final class EmailChange implements Wire {
        private final String email;

        public EmailChange(String email) {
            this.email = email;
        }

        public String getEmail() {
            return email;
        }

        public static EmailChange read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "email", "EmailChange");
            return new EmailChange(
                    Json.asText($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "email");
            Json.text($out, email);
            $out.append('}');
        }
    }

    /** Why a session ended. */
    public static final class EndReason implements Wire {
        public static final EndReason SIGNED_OUT = new EndReason("signed_out");
        public static final EndReason EVERYWHERE = new EndReason("everywhere");
        public static final EndReason REFUSED = new EndReason("refused");
        public static final EndReason ELSEWHERE = new EndReason("elsewhere");

        private final String value;

        public EndReason(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static EndReason read(Object $json) {
            return new EndReason(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof EndReason && (((EndReason) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** One episode. */
    public static final class Episode implements Wire {
        private final List<Image> images;
        private final String name;
        private final long number;
        private final String overview;
        private final Long percent;
        private final Long runtimeMin;

        public Episode(List<Image> images, String name, long number, String overview, Long percent, Long runtimeMin) {
            this.images = images;
            this.name = name;
            this.number = number;
            this.overview = overview;
            this.percent = percent;
            this.runtimeMin = runtimeMin;
        }

        public List<Image> getImagesList() {
            return images == null ? Collections.<Image>emptyList() : images;
        }

        public int getImagesCount() {
            return images == null ? 0 : images.size();
        }

        public Image getImages(int $index) {
            return images.get($index);
        }

        public String getName() {
            return name;
        }

        public long getNumber() {
            return number;
        }

        public boolean hasOverview() {
            return overview != null;
        }

        public String getOverview() {
            return overview == null ? "" : overview;
        }

        /** Whole percent watched. */
        public boolean hasPercent() {
            return percent != null;
        }

        public long getPercent() {
            return percent == null ? 0L : percent;
        }

        public boolean hasRuntimeMin() {
            return runtimeMin != null;
        }

        public long getRuntimeMin() {
            return runtimeMin == null ? 0L : runtimeMin;
        }

        public static Episode read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "images");
            Object $raw1 = Json.need($fields, "name", "Episode");
            Object $raw2 = Json.need($fields, "number", "Episode");
            Object $raw3 = Json.at($fields, "overview");
            Object $raw4 = Json.at($fields, "percent");
            Object $raw5 = Json.at($fields, "runtime_min");
            return new Episode(
                    $raw0 == null ? null : Json.list($raw0, $x0 -> Image.read($x0)),
                    Json.asText($raw1),
                    Json.asLong($raw2),
                    $raw3 == null ? null : Json.asText($raw3),
                    $raw4 == null ? null : Long.valueOf(Json.asLong($raw4)),
                    $raw5 == null ? null : Long.valueOf(Json.asLong($raw5)));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (images != null) {
                $count = Json.key($out, $count, "images");
                $out.append('[');
                for (int $i0 = 0; $i0 < images.size(); $i0++) {
                    if ($i0 > 0) {
                        $out.append(',');
                    }
                    images.get($i0).write($out);
                }
                $out.append(']');
            }
            $count = Json.key($out, $count, "name");
            Json.text($out, name);
            $count = Json.key($out, $count, "number");
            $out.append(number);
            if (overview != null) {
                $count = Json.key($out, $count, "overview");
                Json.text($out, overview);
            }
            if (percent != null) {
                $count = Json.key($out, $count, "percent");
                $out.append(percent);
            }
            if (runtimeMin != null) {
                $count = Json.key($out, $count, "runtime_min");
                $out.append(runtimeMin);
            }
            $out.append('}');
        }
    }

    /** One episode of a series. A film has none. */
    public static final class EpisodeRef implements Wire {
        private final long episode;
        private final long season;

        public EpisodeRef(long episode, long season) {
            this.episode = episode;
            this.season = season;
        }

        public long getEpisode() {
            return episode;
        }

        public long getSeason() {
            return season;
        }

        public static EpisodeRef read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "episode", "EpisodeRef");
            Object $raw1 = Json.need($fields, "season", "EpisodeRef");
            return new EpisodeRef(
                    Json.asLong($raw0),
                    Json.asLong($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "episode");
            $out.append(episode);
            $count = Json.key($out, $count, "season");
            $out.append(season);
            $out.append('}');
        }
    }

    /** Body of every error answer. */
    public static final class ErrorBody implements Wire {
        private final String build;
        private final ErrorCode code;
        private final String label;
        private final Notice notice;
        private final Long retryAfterS;

        public ErrorBody(String build, ErrorCode code, String label, Notice notice, Long retryAfterS) {
            this.build = build;
            this.code = code;
            this.label = label;
            this.notice = notice;
            this.retryAfterS = retryAfterS;
        }

        /** The build that answered. */
        public String getBuild() {
            return build;
        }

        public ErrorCode getCode() {
            return code;
        }

        /** Label key for this error. */
        public String getLabel() {
            return label;
        }

        public boolean hasNotice() {
            return notice != null;
        }

        public Notice getNotice() {
            return notice == null ? null : notice;
        }

        public boolean hasRetryAfterS() {
            return retryAfterS != null;
        }

        public long getRetryAfterS() {
            return retryAfterS == null ? 0L : retryAfterS;
        }

        public static ErrorBody read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "build", "ErrorBody");
            Object $raw1 = Json.need($fields, "code", "ErrorBody");
            Object $raw2 = Json.need($fields, "label", "ErrorBody");
            Object $raw3 = Json.at($fields, "notice");
            Object $raw4 = Json.at($fields, "retry_after_s");
            return new ErrorBody(
                    Json.asText($raw0),
                    ErrorCode.read($raw1),
                    Json.asText($raw2),
                    $raw3 == null ? null : Notice.read($raw3),
                    $raw4 == null ? null : Long.valueOf(Json.asLong($raw4)));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "build");
            Json.text($out, build);
            $count = Json.key($out, $count, "code");
            code.write($out);
            $count = Json.key($out, $count, "label");
            Json.text($out, label);
            if (notice != null) {
                $count = Json.key($out, $count, "notice");
                notice.write($out);
            }
            if (retryAfterS != null) {
                $count = Json.key($out, $count, "retry_after_s");
                $out.append(retryAfterS);
            }
            $out.append('}');
        }
    }

    /** Failure classes a driver reports. The core picks the next attempt from these. */
    public static final class ErrorClass implements Wire {
        public static final ErrorClass NETWORK = new ErrorClass("network");
        public static final ErrorClass MANIFEST = new ErrorClass("manifest");
        public static final ErrorClass SEGMENT = new ErrorClass("segment");
        public static final ErrorClass DECODE = new ErrorClass("decode");
        public static final ErrorClass KEY = new ErrorClass("key");
        public static final ErrorClass FORMAT = new ErrorClass("format");
        public static final ErrorClass REFUSED = new ErrorClass("refused");

        private final String value;

        public ErrorClass(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static ErrorClass read(Object $json) {
            return new ErrorClass(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof ErrorClass && (((ErrorClass) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** Stable error code. A code this build does not know reads as `Unknown`. */
    public static final class ErrorCode implements Wire {
        public static final ErrorCode BAD_REQUEST = new ErrorCode("bad_request");
        public static final ErrorCode UNAUTHORIZED = new ErrorCode("unauthorized");
        public static final ErrorCode SESSION_ENDED = new ErrorCode("session_ended");
        public static final ErrorCode PROFILE_REQUIRED = new ErrorCode("profile_required");
        public static final ErrorCode PIN_REQUIRED = new ErrorCode("pin_required");
        public static final ErrorCode STEP_UP_REQUIRED = new ErrorCode("step_up_required");
        public static final ErrorCode REFUSED = new ErrorCode("refused");
        public static final ErrorCode NOT_FOUND = new ErrorCode("not_found");
        public static final ErrorCode NO_LONGER_AVAILABLE = new ErrorCode("no_longer_available");
        public static final ErrorCode CONFLICT = new ErrorCode("conflict");
        public static final ErrorCode RATE_LIMITED = new ErrorCode("rate_limited");
        public static final ErrorCode OVERLOADED = new ErrorCode("overloaded");
        public static final ErrorCode DEADLINE_EXCEEDED = new ErrorCode("deadline_exceeded");
        public static final ErrorCode UPGRADE_REQUIRED = new ErrorCode("upgrade_required");
        public static final ErrorCode UNSUPPORTED = new ErrorCode("unsupported");
        public static final ErrorCode NO_SOURCE = new ErrorCode("no_source");
        public static final ErrorCode CLOCK_WRONG = new ErrorCode("clock_wrong");
        public static final ErrorCode INTERNAL = new ErrorCode("internal");

        private final String value;

        public ErrorCode(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static ErrorCode read(Object $json) {
            return new ErrorCode(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof ErrorCode && (((ErrorCode) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** One event. `id` is the id of the effect it answers, when it answers one. */
    public abstract static class Event implements Wire {
        public abstract boolean hasId();

        public abstract String getId();

        public static Event read(Object $json) {
            String $tag = Json.asText(Json.need(Json.object($json), "type", "Event"));
            if ("http_done".equals($tag)) {
                return EventHttpDone.read($json);
            }
            if ("http_failed".equals($tag)) {
                return EventHttpFailed.read($json);
            }
            if ("store_done".equals($tag)) {
                return EventStoreDone.read($json);
            }
            if ("secret_done".equals($tag)) {
                return EventSecretDone.read($json);
            }
            if ("timer_fired".equals($tag)) {
                return EventTimerFired.read($json);
            }
            if ("unsupported".equals($tag)) {
                return EventUnsupported.read($json);
            }
            if ("cancelled".equals($tag)) {
                return EventCancelled.read($json);
            }
            if ("suspend".equals($tag)) {
                return EventSuspend.read($json);
            }
            if ("resume".equals($tag)) {
                return EventResume.read($json);
            }
            if ("visibility".equals($tag)) {
                return EventVisibility.read($json);
            }
            if ("network".equals($tag)) {
                return EventNetwork.read($json);
            }
            if ("activity".equals($tag)) {
                return EventActivity.read($json);
            }
            if ("ended_elsewhere".equals($tag)) {
                return EventEndedElsewhere.read($json);
            }
            if ("address".equals($tag)) {
                return EventAddress.read($json);
            }
            if ("action".equals($tag)) {
                return EventAction.read($json);
            }
            if ("driver".equals($tag)) {
                return EventDriver.read($json);
            }
            return EventUnknown.read($json);
        }
    }

    /** An http effect got an answer. Every status is data. */
    public static final class EventHttpDone extends Event {
        private final String id;
        private final String body;
        private final Map<String, String> headers;
        private final long status;

        public EventHttpDone(String id, String body, Map<String, String> headers, long status) {
            this.id = id;
            this.body = body;
            this.headers = headers;
            this.status = status;
        }

        @Override
        public boolean hasId() {
            return id != null;
        }

        @Override
        public String getId() {
            return id == null ? "" : id;
        }

        /** Absent when the body went to the screen. */
        public boolean hasBody() {
            return body != null;
        }

        public String getBody() {
            return body == null ? "" : body;
        }

        public Map<String, String> getHeadersMap() {
            return headers == null ? Collections.<String, String>emptyMap() : headers;
        }

        public int getHeadersCount() {
            return headers == null ? 0 : headers.size();
        }

        public long getStatus() {
            return status;
        }

        public static EventHttpDone read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"http_done".equals(Json.asText(Json.need($fields, "type", "Event")))) {
                throw new IllegalArgumentException("not a EventHttpDone");
            }
            Object $raw0 = Json.at($fields, "id");
            Object $raw1 = Json.at($fields, "body");
            Object $raw2 = Json.at($fields, "headers");
            Object $raw3 = Json.need($fields, "status", "Event");
            return new EventHttpDone(
                    $raw0 == null ? null : Json.asText($raw0),
                    $raw1 == null ? null : Json.asText($raw1),
                    $raw2 == null ? null : Json.map($raw2, $x0 -> Json.asText($x0)),
                    Json.asLong($raw3));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "type");
            Json.text($out, "http_done");
            if (id != null) {
                $count = Json.key($out, $count, "id");
                Json.text($out, id);
            }
            if (body != null) {
                $count = Json.key($out, $count, "body");
                Json.text($out, body);
            }
            if (headers != null) {
                $count = Json.key($out, $count, "headers");
                $out.append('{');
                int $n0 = 0;
                for (Map.Entry<String, String> $e0 : headers.entrySet()) {
                    $n0 = Json.key($out, $n0, $e0.getKey());
                    Json.text($out, $e0.getValue());
                }
                $out.append('}');
            }
            $count = Json.key($out, $count, "status");
            $out.append(status);
            $out.append('}');
        }
    }

    /** An http effect got no answer at all. */
    public static final class EventHttpFailed extends Event {
        private final String id;
        private final NoAnswerReason reason;

        public EventHttpFailed(String id, NoAnswerReason reason) {
            this.id = id;
            this.reason = reason;
        }

        @Override
        public boolean hasId() {
            return id != null;
        }

        @Override
        public String getId() {
            return id == null ? "" : id;
        }

        public NoAnswerReason getReason() {
            return reason;
        }

        public static EventHttpFailed read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"http_failed".equals(Json.asText(Json.need($fields, "type", "Event")))) {
                throw new IllegalArgumentException("not a EventHttpFailed");
            }
            Object $raw0 = Json.at($fields, "id");
            Object $raw1 = Json.need($fields, "reason", "Event");
            return new EventHttpFailed(
                    $raw0 == null ? null : Json.asText($raw0),
                    NoAnswerReason.read($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "type");
            Json.text($out, "http_failed");
            if (id != null) {
                $count = Json.key($out, $count, "id");
                Json.text($out, id);
            }
            $count = Json.key($out, $count, "reason");
            reason.write($out);
            $out.append('}');
        }
    }

    public static final class EventStoreDone extends Event {
        private final String id;
        private final List<String> keys;
        private final String value;

        public EventStoreDone(String id, List<String> keys, String value) {
            this.id = id;
            this.keys = keys;
            this.value = value;
        }

        @Override
        public boolean hasId() {
            return id != null;
        }

        @Override
        public String getId() {
            return id == null ? "" : id;
        }

        public List<String> getKeysList() {
            return keys == null ? Collections.<String>emptyList() : keys;
        }

        public int getKeysCount() {
            return keys == null ? 0 : keys.size();
        }

        public String getKeys(int $index) {
            return keys.get($index);
        }

        public boolean hasValue() {
            return value != null;
        }

        public String getValue() {
            return value == null ? "" : value;
        }

        public static EventStoreDone read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"store_done".equals(Json.asText(Json.need($fields, "type", "Event")))) {
                throw new IllegalArgumentException("not a EventStoreDone");
            }
            Object $raw0 = Json.at($fields, "id");
            Object $raw1 = Json.at($fields, "keys");
            Object $raw2 = Json.at($fields, "value");
            return new EventStoreDone(
                    $raw0 == null ? null : Json.asText($raw0),
                    $raw1 == null ? null : Json.list($raw1, $x0 -> Json.asText($x0)),
                    $raw2 == null ? null : Json.asText($raw2));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "type");
            Json.text($out, "store_done");
            if (id != null) {
                $count = Json.key($out, $count, "id");
                Json.text($out, id);
            }
            if (keys != null) {
                $count = Json.key($out, $count, "keys");
                $out.append('[');
                for (int $i0 = 0; $i0 < keys.size(); $i0++) {
                    if ($i0 > 0) {
                        $out.append(',');
                    }
                    Json.text($out, keys.get($i0));
                }
                $out.append(']');
            }
            if (value != null) {
                $count = Json.key($out, $count, "value");
                Json.text($out, value);
            }
            $out.append('}');
        }
    }

    public static final class EventSecretDone extends Event {
        private final String id;
        private final String value;

        public EventSecretDone(String id, String value) {
            this.id = id;
            this.value = value;
        }

        @Override
        public boolean hasId() {
            return id != null;
        }

        @Override
        public String getId() {
            return id == null ? "" : id;
        }

        public boolean hasValue() {
            return value != null;
        }

        public String getValue() {
            return value == null ? "" : value;
        }

        public static EventSecretDone read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"secret_done".equals(Json.asText(Json.need($fields, "type", "Event")))) {
                throw new IllegalArgumentException("not a EventSecretDone");
            }
            Object $raw0 = Json.at($fields, "id");
            Object $raw1 = Json.at($fields, "value");
            return new EventSecretDone(
                    $raw0 == null ? null : Json.asText($raw0),
                    $raw1 == null ? null : Json.asText($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "type");
            Json.text($out, "secret_done");
            if (id != null) {
                $count = Json.key($out, $count, "id");
                Json.text($out, id);
            }
            if (value != null) {
                $count = Json.key($out, $count, "value");
                Json.text($out, value);
            }
            $out.append('}');
        }
    }

    public static final class EventTimerFired extends Event {
        private final String id;

        public EventTimerFired(String id) {
            this.id = id;
        }

        @Override
        public boolean hasId() {
            return id != null;
        }

        @Override
        public String getId() {
            return id == null ? "" : id;
        }

        public static EventTimerFired read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"timer_fired".equals(Json.asText(Json.need($fields, "type", "Event")))) {
                throw new IllegalArgumentException("not a EventTimerFired");
            }
            Object $raw0 = Json.at($fields, "id");
            return new EventTimerFired(
                    $raw0 == null ? null : Json.asText($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "type");
            Json.text($out, "timer_fired");
            if (id != null) {
                $count = Json.key($out, $count, "id");
                Json.text($out, id);
            }
            $out.append('}');
        }
    }

    /** The host cannot perform this effect. */
    public static final class EventUnsupported extends Event {
        private final String id;

        public EventUnsupported(String id) {
            this.id = id;
        }

        @Override
        public boolean hasId() {
            return id != null;
        }

        @Override
        public String getId() {
            return id == null ? "" : id;
        }

        public static EventUnsupported read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"unsupported".equals(Json.asText(Json.need($fields, "type", "Event")))) {
                throw new IllegalArgumentException("not a EventUnsupported");
            }
            Object $raw0 = Json.at($fields, "id");
            return new EventUnsupported(
                    $raw0 == null ? null : Json.asText($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "type");
            Json.text($out, "unsupported");
            if (id != null) {
                $count = Json.key($out, $count, "id");
                Json.text($out, id);
            }
            $out.append('}');
        }
    }

    /** The effect with this id was dropped. */
    public static final class EventCancelled extends Event {
        private final String id;

        public EventCancelled(String id) {
            this.id = id;
        }

        @Override
        public boolean hasId() {
            return id != null;
        }

        @Override
        public String getId() {
            return id == null ? "" : id;
        }

        public static EventCancelled read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"cancelled".equals(Json.asText(Json.need($fields, "type", "Event")))) {
                throw new IllegalArgumentException("not a EventCancelled");
            }
            Object $raw0 = Json.at($fields, "id");
            return new EventCancelled(
                    $raw0 == null ? null : Json.asText($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "type");
            Json.text($out, "cancelled");
            if (id != null) {
                $count = Json.key($out, $count, "id");
                Json.text($out, id);
            }
            $out.append('}');
        }
    }

    public static final class EventSuspend extends Event {
        private final String id;

        public EventSuspend(String id) {
            this.id = id;
        }

        @Override
        public boolean hasId() {
            return id != null;
        }

        @Override
        public String getId() {
            return id == null ? "" : id;
        }

        public static EventSuspend read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"suspend".equals(Json.asText(Json.need($fields, "type", "Event")))) {
                throw new IllegalArgumentException("not a EventSuspend");
            }
            Object $raw0 = Json.at($fields, "id");
            return new EventSuspend(
                    $raw0 == null ? null : Json.asText($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "type");
            Json.text($out, "suspend");
            if (id != null) {
                $count = Json.key($out, $count, "id");
                Json.text($out, id);
            }
            $out.append('}');
        }
    }

    public static final class EventResume extends Event {
        private final String id;

        public EventResume(String id) {
            this.id = id;
        }

        @Override
        public boolean hasId() {
            return id != null;
        }

        @Override
        public String getId() {
            return id == null ? "" : id;
        }

        public static EventResume read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"resume".equals(Json.asText(Json.need($fields, "type", "Event")))) {
                throw new IllegalArgumentException("not a EventResume");
            }
            Object $raw0 = Json.at($fields, "id");
            return new EventResume(
                    $raw0 == null ? null : Json.asText($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "type");
            Json.text($out, "resume");
            if (id != null) {
                $count = Json.key($out, $count, "id");
                Json.text($out, id);
            }
            $out.append('}');
        }
    }

    public static final class EventVisibility extends Event {
        private final String id;
        private final boolean visible;

        public EventVisibility(String id, boolean visible) {
            this.id = id;
            this.visible = visible;
        }

        @Override
        public boolean hasId() {
            return id != null;
        }

        @Override
        public String getId() {
            return id == null ? "" : id;
        }

        public boolean getVisible() {
            return visible;
        }

        public static EventVisibility read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"visibility".equals(Json.asText(Json.need($fields, "type", "Event")))) {
                throw new IllegalArgumentException("not a EventVisibility");
            }
            Object $raw0 = Json.at($fields, "id");
            Object $raw1 = Json.need($fields, "visible", "Event");
            return new EventVisibility(
                    $raw0 == null ? null : Json.asText($raw0),
                    Json.asBool($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "type");
            Json.text($out, "visibility");
            if (id != null) {
                $count = Json.key($out, $count, "id");
                Json.text($out, id);
            }
            $count = Json.key($out, $count, "visible");
            $out.append(visible);
            $out.append('}');
        }
    }

    public static final class EventNetwork extends Event {
        private final String id;
        private final boolean online;

        public EventNetwork(String id, boolean online) {
            this.id = id;
            this.online = online;
        }

        @Override
        public boolean hasId() {
            return id != null;
        }

        @Override
        public String getId() {
            return id == null ? "" : id;
        }

        public boolean getOnline() {
            return online;
        }

        public static EventNetwork read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"network".equals(Json.asText(Json.need($fields, "type", "Event")))) {
                throw new IllegalArgumentException("not a EventNetwork");
            }
            Object $raw0 = Json.at($fields, "id");
            Object $raw1 = Json.need($fields, "online", "Event");
            return new EventNetwork(
                    $raw0 == null ? null : Json.asText($raw0),
                    Json.asBool($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "type");
            Json.text($out, "network");
            if (id != null) {
                $count = Json.key($out, $count, "id");
                Json.text($out, id);
            }
            $count = Json.key($out, $count, "online");
            $out.append(online);
            $out.append('}');
        }
    }

    /** Touch, key, wheel or playing video in view. */
    public static final class EventActivity extends Event {
        private final String id;

        public EventActivity(String id) {
            this.id = id;
        }

        @Override
        public boolean hasId() {
            return id != null;
        }

        @Override
        public String getId() {
            return id == null ? "" : id;
        }

        public static EventActivity read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"activity".equals(Json.asText(Json.need($fields, "type", "Event")))) {
                throw new IllegalArgumentException("not a EventActivity");
            }
            Object $raw0 = Json.at($fields, "id");
            return new EventActivity(
                    $raw0 == null ? null : Json.asText($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "type");
            Json.text($out, "activity");
            if (id != null) {
                $count = Json.key($out, $count, "id");
                Json.text($out, id);
            }
            $out.append('}');
        }
    }

    /** Another tab of this browser ended the session. The server has heard already. */
    public static final class EventEndedElsewhere extends Event {
        private final String id;

        public EventEndedElsewhere(String id) {
            this.id = id;
        }

        @Override
        public boolean hasId() {
            return id != null;
        }

        @Override
        public String getId() {
            return id == null ? "" : id;
        }

        public static EventEndedElsewhere read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"ended_elsewhere".equals(Json.asText(Json.need($fields, "type", "Event")))) {
                throw new IllegalArgumentException("not a EventEndedElsewhere");
            }
            Object $raw0 = Json.at($fields, "id");
            return new EventEndedElsewhere(
                    $raw0 == null ? null : Json.asText($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "type");
            Json.text($out, "ended_elsewhere");
            if (id != null) {
                $count = Json.key($out, $count, "id");
                Json.text($out, id);
            }
            $out.append('}');
        }
    }

    /** An address or deep link to open. */
    public static final class EventAddress extends Event {
        private final String id;
        private final String address;

        public EventAddress(String id, String address) {
            this.id = id;
            this.address = address;
        }

        @Override
        public boolean hasId() {
            return id != null;
        }

        @Override
        public String getId() {
            return id == null ? "" : id;
        }

        public String getAddress() {
            return address;
        }

        public static EventAddress read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"address".equals(Json.asText(Json.need($fields, "type", "Event")))) {
                throw new IllegalArgumentException("not a EventAddress");
            }
            Object $raw0 = Json.at($fields, "id");
            Object $raw1 = Json.need($fields, "address", "Event");
            return new EventAddress(
                    $raw0 == null ? null : Json.asText($raw0),
                    Json.asText($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "type");
            Json.text($out, "address");
            if (id != null) {
                $count = Json.key($out, $count, "id");
                Json.text($out, id);
            }
            $count = Json.key($out, $count, "address");
            Json.text($out, address);
            $out.append('}');
        }
    }

    public static final class EventAction extends Event {
        private final String id;
        private final Action action;

        public EventAction(String id, Action action) {
            this.id = id;
            this.action = action;
        }

        @Override
        public boolean hasId() {
            return id != null;
        }

        @Override
        public String getId() {
            return id == null ? "" : id;
        }

        public Action getAction() {
            return action;
        }

        public static EventAction read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"action".equals(Json.asText(Json.need($fields, "type", "Event")))) {
                throw new IllegalArgumentException("not a EventAction");
            }
            Object $raw0 = Json.at($fields, "id");
            Object $raw1 = Json.need($fields, "action", "Event");
            return new EventAction(
                    $raw0 == null ? null : Json.asText($raw0),
                    Action.read($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "type");
            Json.text($out, "action");
            if (id != null) {
                $count = Json.key($out, $count, "id");
                Json.text($out, id);
            }
            $count = Json.key($out, $count, "action");
            action.write($out);
            $out.append('}');
        }
    }

    /** A driver event, stamped with the generation of the open that made it. */
    public static final class EventDriver extends Event {
        private final String id;
        private final DriverEvent event;
        private final long generation;

        public EventDriver(String id, DriverEvent event, long generation) {
            this.id = id;
            this.event = event;
            this.generation = generation;
        }

        @Override
        public boolean hasId() {
            return id != null;
        }

        @Override
        public String getId() {
            return id == null ? "" : id;
        }

        public DriverEvent getEvent() {
            return event;
        }

        public long getGeneration() {
            return generation;
        }

        public static EventDriver read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            if (!"driver".equals(Json.asText(Json.need($fields, "type", "Event")))) {
                throw new IllegalArgumentException("not a EventDriver");
            }
            Object $raw0 = Json.at($fields, "id");
            Object $raw1 = Json.need($fields, "event", "Event");
            Object $raw2 = Json.need($fields, "generation", "Event");
            return new EventDriver(
                    $raw0 == null ? null : Json.asText($raw0),
                    DriverEvent.read($raw1),
                    Json.asLong($raw2));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "type");
            Json.text($out, "driver");
            if (id != null) {
                $count = Json.key($out, $count, "id");
                Json.text($out, id);
            }
            $count = Json.key($out, $count, "event");
            event.write($out);
            $count = Json.key($out, $count, "generation");
            $out.append(generation);
            $out.append('}');
        }
    }

    /** A tag this build does not know. */
    public static final class EventUnknown extends Event {
        private final String tag;
        private final String rawId;

        public EventUnknown(String tag, String rawId) {
            this.tag = tag;
            this.rawId = rawId;
        }

        /** The tag that was read. */
        public String getTag() {
            return tag;
        }

        @Override
        public boolean hasId() {
            return rawId != null;
        }

        @Override
        public String getId() {
            return rawId == null ? "" : rawId;
        }

        public static EventUnknown read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "type");
            Object $raw1 = Json.at($fields, "id");
            return new EventUnknown(
                    $raw0 == null ? null : Json.asText($raw0),
                    $raw1 == null ? null : Json.asText($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "type");
            Json.text($out, tag);
            if (rawId != null) {
                $count = Json.key($out, $count, "id");
                Json.text($out, rawId);
            }
            $out.append('}');
        }
    }

    /** Feature values. With no answer from the server, accounts are required. */
    public static final class Features implements Wire {
        private final Boolean accountsRequired;
        private final Boolean clips;
        private final Boolean games;
        private final Boolean originalAudio;
        private final String visitorTier;

        public Features(Boolean accountsRequired, Boolean clips, Boolean games, Boolean originalAudio, String visitorTier) {
            this.accountsRequired = accountsRequired;
            this.clips = clips;
            this.games = games;
            this.originalAudio = originalAudio;
            this.visitorTier = visitorTier;
        }

        public boolean hasAccountsRequired() {
            return accountsRequired != null;
        }

        public boolean getAccountsRequired() {
            return accountsRequired != null && accountsRequired;
        }

        public boolean hasClips() {
            return clips != null;
        }

        public boolean getClips() {
            return clips != null && clips;
        }

        public boolean hasGames() {
            return games != null;
        }

        public boolean getGames() {
            return games != null && games;
        }

        public boolean hasOriginalAudio() {
            return originalAudio != null;
        }

        public boolean getOriginalAudio() {
            return originalAudio != null && originalAudio;
        }

        /** Tier for visitors. Present only when accounts are not required. */
        public boolean hasVisitorTier() {
            return visitorTier != null;
        }

        public String getVisitorTier() {
            return visitorTier == null ? "" : visitorTier;
        }

        public static Features read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "accounts_required");
            Object $raw1 = Json.at($fields, "clips");
            Object $raw2 = Json.at($fields, "games");
            Object $raw3 = Json.at($fields, "original_audio");
            Object $raw4 = Json.at($fields, "visitor_tier");
            return new Features(
                    $raw0 == null ? null : Boolean.valueOf(Json.asBool($raw0)),
                    $raw1 == null ? null : Boolean.valueOf(Json.asBool($raw1)),
                    $raw2 == null ? null : Boolean.valueOf(Json.asBool($raw2)),
                    $raw3 == null ? null : Boolean.valueOf(Json.asBool($raw3)),
                    $raw4 == null ? null : Json.asText($raw4));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (accountsRequired != null) {
                $count = Json.key($out, $count, "accounts_required");
                $out.append(accountsRequired);
            }
            if (clips != null) {
                $count = Json.key($out, $count, "clips");
                $out.append(clips);
            }
            if (games != null) {
                $count = Json.key($out, $count, "games");
                $out.append(games);
            }
            if (originalAudio != null) {
                $count = Json.key($out, $count, "original_audio");
                $out.append(originalAudio);
            }
            if (visitorTier != null) {
                $count = Json.key($out, $count, "visitor_tier");
                Json.text($out, visitorTier);
            }
            $out.append('}');
        }
    }

    /** Signals a viewer gives. */
    public static final class FeedbackEvent implements Wire {
        public static final FeedbackEvent CLICK = new FeedbackEvent("click");
        public static final FeedbackEvent START = new FeedbackEvent("start");
        public static final FeedbackEvent COMPLETE = new FeedbackEvent("complete");
        public static final FeedbackEvent LIKE = new FeedbackEvent("like");
        public static final FeedbackEvent SKIP = new FeedbackEvent("skip");

        private final String value;

        public FeedbackEvent(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static FeedbackEvent read(Object $json) {
            return new FeedbackEvent(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof FeedbackEvent && (((FeedbackEvent) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** Answer, possibly partial when the budget ran out. */
    public static final class FindAnswer implements Wire {
        private final String job;
        private final FindOutcome outcome;
        private final List<FoundSource> sources;

        public FindAnswer(String job, FindOutcome outcome, List<FoundSource> sources) {
            this.job = job;
            this.outcome = outcome;
            this.sources = sources;
        }

        /** Read the rest through `/v1/find/{job}`. */
        public boolean hasJob() {
            return job != null;
        }

        public String getJob() {
            return job == null ? "" : job;
        }

        public FindOutcome getOutcome() {
            return outcome;
        }

        public List<FoundSource> getSourcesList() {
            return sources == null ? Collections.<FoundSource>emptyList() : sources;
        }

        public int getSourcesCount() {
            return sources == null ? 0 : sources.size();
        }

        public FoundSource getSources(int $index) {
            return sources.get($index);
        }

        public static FindAnswer read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "job");
            Object $raw1 = Json.need($fields, "outcome", "FindAnswer");
            Object $raw2 = Json.need($fields, "sources", "FindAnswer");
            return new FindAnswer(
                    $raw0 == null ? null : Json.asText($raw0),
                    FindOutcome.read($raw1),
                    Json.list($raw2, $x0 -> FoundSource.read($x0)));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (job != null) {
                $count = Json.key($out, $count, "job");
                Json.text($out, job);
            }
            $count = Json.key($out, $count, "outcome");
            outcome.write($out);
            $count = Json.key($out, $count, "sources");
            $out.append('[');
            for (int $i0 = 0; $i0 < sources.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                sources.get($i0).write($out);
            }
            $out.append(']');
            $out.append('}');
        }
    }

    /** How a search for sources ended. */
    public static final class FindOutcome implements Wire {
        public static final FindOutcome FOUND = new FindOutcome("found");
        public static final FindOutcome PARTIAL = new FindOutcome("partial");
        public static final FindOutcome NOT_FOUND = new FindOutcome("not_found");
        public static final FindOutcome PENDING = new FindOutcome("pending");

        private final String value;

        public FindOutcome(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static FindOutcome read(Object $json) {
            return new FindOutcome(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof FindOutcome && (((FindOutcome) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** Title facts come from the server, so a finder never looks them up. */
    public static final class FindRequest implements Wire {
        private final long budgetMs;
        private final EpisodeRef episode;
        private final String name;
        private final TitleKey title;
        private final Long year;

        public FindRequest(long budgetMs, EpisodeRef episode, String name, TitleKey title, Long year) {
            this.budgetMs = budgetMs;
            this.episode = episode;
            this.name = name;
            this.title = title;
            this.year = year;
        }

        /** Total time this search may take. */
        public long getBudgetMs() {
            return budgetMs;
        }

        public boolean hasEpisode() {
            return episode != null;
        }

        public EpisodeRef getEpisode() {
            return episode == null ? null : episode;
        }

        public String getName() {
            return name;
        }

        public TitleKey getTitle() {
            return title;
        }

        public boolean hasYear() {
            return year != null;
        }

        public long getYear() {
            return year == null ? 0L : year;
        }

        public static FindRequest read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "budget_ms", "FindRequest");
            Object $raw1 = Json.at($fields, "episode");
            Object $raw2 = Json.need($fields, "name", "FindRequest");
            Object $raw3 = Json.need($fields, "title", "FindRequest");
            Object $raw4 = Json.at($fields, "year");
            return new FindRequest(
                    Json.asLong($raw0),
                    $raw1 == null ? null : EpisodeRef.read($raw1),
                    Json.asText($raw2),
                    TitleKey.read($raw3),
                    $raw4 == null ? null : Long.valueOf(Json.asLong($raw4)));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "budget_ms");
            $out.append(budgetMs);
            if (episode != null) {
                $count = Json.key($out, $count, "episode");
                episode.write($out);
            }
            $count = Json.key($out, $count, "name");
            Json.text($out, name);
            $count = Json.key($out, $count, "title");
            title.write($out);
            if (year != null) {
                $count = Json.key($out, $count, "year");
                $out.append(year);
            }
            $out.append('}');
        }
    }

    /** `/v1/health` of a finder. */
    public static final class FinderHealth implements Wire {
        private final String build;
        private final ServiceState state;

        public FinderHealth(String build, ServiceState state) {
            this.build = build;
            this.state = state;
        }

        public String getBuild() {
            return build;
        }

        public ServiceState getState() {
            return state;
        }

        public static FinderHealth read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "build", "FinderHealth");
            Object $raw1 = Json.need($fields, "state", "FinderHealth");
            return new FinderHealth(
                    Json.asText($raw0),
                    ServiceState.read($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "build");
            Json.text($out, build);
            $count = Json.key($out, $count, "state");
            state.write($out);
            $out.append('}');
        }
    }

    /** The form a client draws in. A browser on a phone is class web, form phone. */
    public static final class Form implements Wire {
        public static final Form PHONE = new Form("phone");
        public static final Form TABLET = new Form("tablet");
        public static final Form DESKTOP = new Form("desktop");
        public static final Form TV = new Form("tv");

        private final String value;

        public Form(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static Form read(Object $json) {
            return new Form(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof Form && (((Form) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** One source as facts. The server turns these into ranked attempts. */
    public static final class FoundSource implements Wire {
        private final List<String> audio;
        private final String container;
        private final String group;
        private final Map<String, String> headers;
        private final Long height;
        private final AttemptKind kind;
        private final String url;
        private final boolean verified;

        public FoundSource(List<String> audio, String container, String group, Map<String, String> headers, Long height, AttemptKind kind, String url, boolean verified) {
            this.audio = audio;
            this.container = container;
            this.group = group;
            this.headers = headers;
            this.height = height;
            this.kind = kind;
            this.url = url;
            this.verified = verified;
        }

        public List<String> getAudioList() {
            return audio == null ? Collections.<String>emptyList() : audio;
        }

        public int getAudioCount() {
            return audio == null ? 0 : audio.size();
        }

        public String getAudio(int $index) {
            return audio.get($index);
        }

        public boolean hasContainer() {
            return container != null;
        }

        public String getContainer() {
            return container == null ? "" : container;
        }

        public String getGroup() {
            return group;
        }

        public Map<String, String> getHeadersMap() {
            return headers == null ? Collections.<String, String>emptyMap() : headers;
        }

        public int getHeadersCount() {
            return headers == null ? 0 : headers.size();
        }

        public boolean hasHeight() {
            return height != null;
        }

        public long getHeight() {
            return height == null ? 0L : height;
        }

        public AttemptKind getKind() {
            return kind;
        }

        public String getUrl() {
            return url;
        }

        public boolean getVerified() {
            return verified;
        }

        public static FoundSource read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "audio");
            Object $raw1 = Json.at($fields, "container");
            Object $raw2 = Json.need($fields, "group", "FoundSource");
            Object $raw3 = Json.need($fields, "headers", "FoundSource");
            Object $raw4 = Json.at($fields, "height");
            Object $raw5 = Json.need($fields, "kind", "FoundSource");
            Object $raw6 = Json.need($fields, "url", "FoundSource");
            Object $raw7 = Json.need($fields, "verified", "FoundSource");
            return new FoundSource(
                    $raw0 == null ? null : Json.list($raw0, $x0 -> Json.asText($x0)),
                    $raw1 == null ? null : Json.asText($raw1),
                    Json.asText($raw2),
                    Json.map($raw3, $x0 -> Json.asText($x0)),
                    $raw4 == null ? null : Long.valueOf(Json.asLong($raw4)),
                    AttemptKind.read($raw5),
                    Json.asText($raw6),
                    Json.asBool($raw7));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (audio != null) {
                $count = Json.key($out, $count, "audio");
                $out.append('[');
                for (int $i0 = 0; $i0 < audio.size(); $i0++) {
                    if ($i0 > 0) {
                        $out.append(',');
                    }
                    Json.text($out, audio.get($i0));
                }
                $out.append(']');
            }
            if (container != null) {
                $count = Json.key($out, $count, "container");
                Json.text($out, container);
            }
            $count = Json.key($out, $count, "group");
            Json.text($out, group);
            $count = Json.key($out, $count, "headers");
            $out.append('{');
            int $n0 = 0;
            for (Map.Entry<String, String> $e0 : headers.entrySet()) {
                $n0 = Json.key($out, $n0, $e0.getKey());
                Json.text($out, $e0.getValue());
            }
            $out.append('}');
            if (height != null) {
                $count = Json.key($out, $count, "height");
                $out.append(height);
            }
            $count = Json.key($out, $count, "kind");
            kind.write($out);
            $count = Json.key($out, $count, "url");
            Json.text($out, url);
            $count = Json.key($out, $count, "verified");
            $out.append(verified);
            $out.append('}');
        }
    }

    /** The facts of the line under a title's logo on the profile chooser. Never words: the app writes them in the viewer's language, with the date in its own month names and digits. */
    public static final class GateLine implements Wire {
        private final CalendarDay date;
        private final GateLineKind kind;
        private final TitleKind media;
        private final Long rank;

        public GateLine(CalendarDay date, GateLineKind kind, TitleKind media, Long rank) {
            this.date = date;
            this.kind = kind;
            this.media = media;
            this.rank = rank;
        }

        /** The day, for `coming` and `seasons_coming`. */
        public boolean hasDate() {
            return date != null;
        }

        public CalendarDay getDate() {
            return date == null ? null : date;
        }

        public GateLineKind getKind() {
            return kind;
        }

        /** Films or series, for `ranked`. */
        public boolean hasMedia() {
            return media != null;
        }

        public TitleKind getMedia() {
            return media == null ? null : media;
        }

        /** The place from 1, for `ranked`. */
        public boolean hasRank() {
            return rank != null;
        }

        public long getRank() {
            return rank == null ? 0L : rank;
        }

        public static GateLine read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "date");
            Object $raw1 = Json.need($fields, "kind", "GateLine");
            Object $raw2 = Json.at($fields, "media");
            Object $raw3 = Json.at($fields, "rank");
            return new GateLine(
                    $raw0 == null ? null : CalendarDay.read($raw0),
                    GateLineKind.read($raw1),
                    $raw2 == null ? null : TitleKind.read($raw2),
                    $raw3 == null ? null : Long.valueOf(Json.asLong($raw3)));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (date != null) {
                $count = Json.key($out, $count, "date");
                date.write($out);
            }
            $count = Json.key($out, $count, "kind");
            kind.write($out);
            if (media != null) {
                $count = Json.key($out, $count, "media");
                media.write($out);
            }
            if (rank != null) {
                $count = Json.key($out, $count, "rank");
                $out.append(rank);
            }
            $out.append('}');
        }
    }

    /** What the line under a title's logo on the profile chooser tells. The app writes the words. */
    public static final class GateLineKind implements Wire {
        public static final GateLineKind COMING = new GateLineKind("coming");
        public static final GateLineKind SEASONS_COMING = new GateLineKind("seasons_coming");
        public static final GateLineKind RANKED = new GateLineKind("ranked");
        public static final GateLineKind NONE = new GateLineKind("none");

        private final String value;

        public GateLineKind(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static GateLineKind read(Object $json) {
            return new GateLineKind(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof GateLineKind && (((GateLineKind) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** One title shown over the profile chooser: its tile, with art, logo and poster, and its line. */
    public static final class GateTitle implements Wire {
        private final GateLine line;
        private final Tile tile;

        public GateTitle(GateLine line, Tile tile) {
            this.line = line;
            this.tile = tile;
        }

        public GateLine getLine() {
            return line;
        }

        public Tile getTile() {
            return tile;
        }

        public static GateTitle read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "line", "GateTitle");
            Object $raw1 = Json.need($fields, "tile", "GateTitle");
            return new GateTitle(
                    GateLine.read($raw0),
                    Tile.read($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "line");
            line.write($out);
            $count = Json.key($out, $count, "tile");
            tile.write($out);
            $out.append('}');
        }
    }

    /** True while the who is watching prompt should show. */
    public static final class GateView implements Wire {
        private final boolean ask;

        public GateView(boolean ask) {
            this.ask = ask;
        }

        public boolean getAsk() {
            return ask;
        }

        public static GateView read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "ask", "GateView");
            return new GateView(
                    Json.asBool($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "ask");
            $out.append(ask);
            $out.append('}');
        }
    }

    /** First answer a client reads. */
    public static final class Hello implements Wire {
        private final BrandValues brand;
        private final List<ClientFloor> clientMin;
        private final VersionRange contract;
        private final Features features;
        private final List<String> killedBuilds;
        private final Limits limits;
        private final List<MovedAddress> moved;
        private final boolean playback;
        private final List<Rollout> rollouts;
        private final List<ClassValue> startDeadlineMs;
        private final long ttlS;

        public Hello(BrandValues brand, List<ClientFloor> clientMin, VersionRange contract, Features features, List<String> killedBuilds, Limits limits, List<MovedAddress> moved, boolean playback, List<Rollout> rollouts, List<ClassValue> startDeadlineMs, long ttlS) {
            this.brand = brand;
            this.clientMin = clientMin;
            this.contract = contract;
            this.features = features;
            this.killedBuilds = killedBuilds;
            this.limits = limits;
            this.moved = moved;
            this.playback = playback;
            this.rollouts = rollouts;
            this.startDeadlineMs = startDeadlineMs;
            this.ttlS = ttlS;
        }

        public BrandValues getBrand() {
            return brand;
        }

        public List<ClientFloor> getClientMinList() {
            return clientMin == null ? Collections.<ClientFloor>emptyList() : clientMin;
        }

        public int getClientMinCount() {
            return clientMin == null ? 0 : clientMin.size();
        }

        public ClientFloor getClientMin(int $index) {
            return clientMin.get($index);
        }

        /** Wire dates this server answers. */
        public VersionRange getContract() {
            return contract;
        }

        public Features getFeatures() {
            return features;
        }

        /** Builds refused by name. */
        public List<String> getKilledBuildsList() {
            return killedBuilds == null ? Collections.<String>emptyList() : killedBuilds;
        }

        public int getKilledBuildsCount() {
            return killedBuilds == null ? 0 : killedBuilds.size();
        }

        public String getKilledBuilds(int $index) {
            return killedBuilds.get($index);
        }

        public Limits getLimits() {
            return limits;
        }

        public List<MovedAddress> getMovedList() {
            return moved == null ? Collections.<MovedAddress>emptyList() : moved;
        }

        public int getMovedCount() {
            return moved == null ? 0 : moved.size();
        }

        public MovedAddress getMoved(int $index) {
            return moved.get($index);
        }

        /** False stops playback everywhere with a notice. */
        public boolean getPlayback() {
            return playback;
        }

        public List<Rollout> getRolloutsList() {
            return rollouts == null ? Collections.<Rollout>emptyList() : rollouts;
        }

        public int getRolloutsCount() {
            return rollouts == null ? 0 : rollouts.size();
        }

        public Rollout getRollouts(int $index) {
            return rollouts.get($index);
        }

        public List<ClassValue> getStartDeadlineMsList() {
            return startDeadlineMs == null ? Collections.<ClassValue>emptyList() : startDeadlineMs;
        }

        public int getStartDeadlineMsCount() {
            return startDeadlineMs == null ? 0 : startDeadlineMs.size();
        }

        public ClassValue getStartDeadlineMs(int $index) {
            return startDeadlineMs.get($index);
        }

        /** Seconds this answer stays good. */
        public long getTtlS() {
            return ttlS;
        }

        public static Hello read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "brand", "Hello");
            Object $raw1 = Json.need($fields, "client_min", "Hello");
            Object $raw2 = Json.need($fields, "contract", "Hello");
            Object $raw3 = Json.need($fields, "features", "Hello");
            Object $raw4 = Json.at($fields, "killed_builds");
            Object $raw5 = Json.need($fields, "limits", "Hello");
            Object $raw6 = Json.at($fields, "moved");
            Object $raw7 = Json.need($fields, "playback", "Hello");
            Object $raw8 = Json.at($fields, "rollouts");
            Object $raw9 = Json.need($fields, "start_deadline_ms", "Hello");
            Object $raw10 = Json.need($fields, "ttl_s", "Hello");
            return new Hello(
                    BrandValues.read($raw0),
                    Json.list($raw1, $x0 -> ClientFloor.read($x0)),
                    VersionRange.read($raw2),
                    Features.read($raw3),
                    $raw4 == null ? null : Json.list($raw4, $x0 -> Json.asText($x0)),
                    Limits.read($raw5),
                    $raw6 == null ? null : Json.list($raw6, $x0 -> MovedAddress.read($x0)),
                    Json.asBool($raw7),
                    $raw8 == null ? null : Json.list($raw8, $x0 -> Rollout.read($x0)),
                    Json.list($raw9, $x0 -> ClassValue.read($x0)),
                    Json.asLong($raw10));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "brand");
            brand.write($out);
            $count = Json.key($out, $count, "client_min");
            $out.append('[');
            for (int $i0 = 0; $i0 < clientMin.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                clientMin.get($i0).write($out);
            }
            $out.append(']');
            $count = Json.key($out, $count, "contract");
            contract.write($out);
            $count = Json.key($out, $count, "features");
            features.write($out);
            if (killedBuilds != null) {
                $count = Json.key($out, $count, "killed_builds");
                $out.append('[');
                for (int $i0 = 0; $i0 < killedBuilds.size(); $i0++) {
                    if ($i0 > 0) {
                        $out.append(',');
                    }
                    Json.text($out, killedBuilds.get($i0));
                }
                $out.append(']');
            }
            $count = Json.key($out, $count, "limits");
            limits.write($out);
            if (moved != null) {
                $count = Json.key($out, $count, "moved");
                $out.append('[');
                for (int $i0 = 0; $i0 < moved.size(); $i0++) {
                    if ($i0 > 0) {
                        $out.append(',');
                    }
                    moved.get($i0).write($out);
                }
                $out.append(']');
            }
            $count = Json.key($out, $count, "playback");
            $out.append(playback);
            if (rollouts != null) {
                $count = Json.key($out, $count, "rollouts");
                $out.append('[');
                for (int $i0 = 0; $i0 < rollouts.size(); $i0++) {
                    if ($i0 > 0) {
                        $out.append(',');
                    }
                    rollouts.get($i0).write($out);
                }
                $out.append(']');
            }
            $count = Json.key($out, $count, "start_deadline_ms");
            $out.append('[');
            for (int $i0 = 0; $i0 < startDeadlineMs.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                startDeadlineMs.get($i0).write($out);
            }
            $out.append(']');
            $count = Json.key($out, $count, "ttl_s");
            $out.append(ttlS);
            $out.append('}');
        }
    }

    /** `/v1/help/articles/{slug}`. */
    public static final class HelpArticle implements Wire {
        private final List<HelpLink> related;
        private final List<HelpSection> sections;
        private final String slug;
        private final String title;

        public HelpArticle(List<HelpLink> related, List<HelpSection> sections, String slug, String title) {
            this.related = related;
            this.sections = sections;
            this.slug = slug;
            this.title = title;
        }

        public List<HelpLink> getRelatedList() {
            return related == null ? Collections.<HelpLink>emptyList() : related;
        }

        public int getRelatedCount() {
            return related == null ? 0 : related.size();
        }

        public HelpLink getRelated(int $index) {
            return related.get($index);
        }

        public List<HelpSection> getSectionsList() {
            return sections == null ? Collections.<HelpSection>emptyList() : sections;
        }

        public int getSectionsCount() {
            return sections == null ? 0 : sections.size();
        }

        public HelpSection getSections(int $index) {
            return sections.get($index);
        }

        public String getSlug() {
            return slug;
        }

        public String getTitle() {
            return title;
        }

        public static HelpArticle read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "related");
            Object $raw1 = Json.need($fields, "sections", "HelpArticle");
            Object $raw2 = Json.need($fields, "slug", "HelpArticle");
            Object $raw3 = Json.need($fields, "title", "HelpArticle");
            return new HelpArticle(
                    $raw0 == null ? null : Json.list($raw0, $x0 -> HelpLink.read($x0)),
                    Json.list($raw1, $x0 -> HelpSection.read($x0)),
                    Json.asText($raw2),
                    Json.asText($raw3));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (related != null) {
                $count = Json.key($out, $count, "related");
                $out.append('[');
                for (int $i0 = 0; $i0 < related.size(); $i0++) {
                    if ($i0 > 0) {
                        $out.append(',');
                    }
                    related.get($i0).write($out);
                }
                $out.append(']');
            }
            $count = Json.key($out, $count, "sections");
            $out.append('[');
            for (int $i0 = 0; $i0 < sections.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                sections.get($i0).write($out);
            }
            $out.append(']');
            $count = Json.key($out, $count, "slug");
            Json.text($out, slug);
            $count = Json.key($out, $count, "title");
            Json.text($out, title);
            $out.append('}');
        }
    }

    public static final class HelpBlock implements Wire {
        private final HelpBlockKind kind;
        private final List<HelpLine> lines;
        private final String tool;

        public HelpBlock(HelpBlockKind kind, List<HelpLine> lines, String tool) {
            this.kind = kind;
            this.lines = lines;
            this.tool = tool;
        }

        public HelpBlockKind getKind() {
            return kind;
        }

        public List<HelpLine> getLinesList() {
            return lines == null ? Collections.<HelpLine>emptyList() : lines;
        }

        public int getLinesCount() {
            return lines == null ? 0 : lines.size();
        }

        public HelpLine getLines(int $index) {
            return lines.get($index);
        }

        /** The action of a tool block. */
        public boolean hasTool() {
            return tool != null;
        }

        public String getTool() {
            return tool == null ? "" : tool;
        }

        public static HelpBlock read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "kind", "HelpBlock");
            Object $raw1 = Json.at($fields, "lines");
            Object $raw2 = Json.at($fields, "tool");
            return new HelpBlock(
                    HelpBlockKind.read($raw0),
                    $raw1 == null ? null : Json.list($raw1, $x0 -> HelpLine.read($x0)),
                    $raw2 == null ? null : Json.asText($raw2));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "kind");
            kind.write($out);
            if (lines != null) {
                $count = Json.key($out, $count, "lines");
                $out.append('[');
                for (int $i0 = 0; $i0 < lines.size(); $i0++) {
                    if ($i0 > 0) {
                        $out.append(',');
                    }
                    lines.get($i0).write($out);
                }
                $out.append(']');
            }
            if (tool != null) {
                $count = Json.key($out, $count, "tool");
                Json.text($out, tool);
            }
            $out.append('}');
        }
    }

    /** What a block of an article is. */
    public static final class HelpBlockKind implements Wire {
        public static final HelpBlockKind PARAGRAPH = new HelpBlockKind("paragraph");
        public static final HelpBlockKind HEADING = new HelpBlockKind("heading");
        public static final HelpBlockKind STEPS = new HelpBlockKind("steps");
        public static final HelpBlockKind LIST = new HelpBlockKind("list");
        public static final HelpBlockKind NOTE = new HelpBlockKind("note");
        public static final HelpBlockKind TOOL = new HelpBlockKind("tool");

        private final String value;

        public HelpBlockKind(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static HelpBlockKind read(Object $json) {
            return new HelpBlockKind(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof HelpBlockKind && (((HelpBlockKind) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    public static final class HelpGroup implements Wire {
        private final List<HelpLink> articles;
        private final String id;
        private final String title;

        public HelpGroup(List<HelpLink> articles, String id, String title) {
            this.articles = articles;
            this.id = id;
            this.title = title;
        }

        public List<HelpLink> getArticlesList() {
            return articles == null ? Collections.<HelpLink>emptyList() : articles;
        }

        public int getArticlesCount() {
            return articles == null ? 0 : articles.size();
        }

        public HelpLink getArticles(int $index) {
            return articles.get($index);
        }

        public String getId() {
            return id;
        }

        public String getTitle() {
            return title;
        }

        public static HelpGroup read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "articles", "HelpGroup");
            Object $raw1 = Json.need($fields, "id", "HelpGroup");
            Object $raw2 = Json.need($fields, "title", "HelpGroup");
            return new HelpGroup(
                    Json.list($raw0, $x0 -> HelpLink.read($x0)),
                    Json.asText($raw1),
                    Json.asText($raw2));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "articles");
            $out.append('[');
            for (int $i0 = 0; $i0 < articles.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                articles.get($i0).write($out);
            }
            $out.append(']');
            $count = Json.key($out, $count, "id");
            Json.text($out, id);
            $count = Json.key($out, $count, "title");
            Json.text($out, title);
            $out.append('}');
        }
    }

    /** `/v1/help`. */
    public static final class HelpHome implements Wire {
        private final List<HelpPolicyLink> policies;
        private final List<HelpLink> popular;
        private final List<HelpTopic> topics;

        public HelpHome(List<HelpPolicyLink> policies, List<HelpLink> popular, List<HelpTopic> topics) {
            this.policies = policies;
            this.popular = popular;
            this.topics = topics;
        }

        public List<HelpPolicyLink> getPoliciesList() {
            return policies == null ? Collections.<HelpPolicyLink>emptyList() : policies;
        }

        public int getPoliciesCount() {
            return policies == null ? 0 : policies.size();
        }

        public HelpPolicyLink getPolicies(int $index) {
            return policies.get($index);
        }

        public List<HelpLink> getPopularList() {
            return popular == null ? Collections.<HelpLink>emptyList() : popular;
        }

        public int getPopularCount() {
            return popular == null ? 0 : popular.size();
        }

        public HelpLink getPopular(int $index) {
            return popular.get($index);
        }

        public List<HelpTopic> getTopicsList() {
            return topics == null ? Collections.<HelpTopic>emptyList() : topics;
        }

        public int getTopicsCount() {
            return topics == null ? 0 : topics.size();
        }

        public HelpTopic getTopics(int $index) {
            return topics.get($index);
        }

        public static HelpHome read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "policies", "HelpHome");
            Object $raw1 = Json.need($fields, "popular", "HelpHome");
            Object $raw2 = Json.need($fields, "topics", "HelpHome");
            return new HelpHome(
                    Json.list($raw0, $x0 -> HelpPolicyLink.read($x0)),
                    Json.list($raw1, $x0 -> HelpLink.read($x0)),
                    Json.list($raw2, $x0 -> HelpTopic.read($x0)));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "policies");
            $out.append('[');
            for (int $i0 = 0; $i0 < policies.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                policies.get($i0).write($out);
            }
            $out.append(']');
            $count = Json.key($out, $count, "popular");
            $out.append('[');
            for (int $i0 = 0; $i0 < popular.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                popular.get($i0).write($out);
            }
            $out.append(']');
            $count = Json.key($out, $count, "topics");
            $out.append('[');
            for (int $i0 = 0; $i0 < topics.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                topics.get($i0).write($out);
            }
            $out.append(']');
            $out.append('}');
        }
    }

    /** One line of a block. A paragraph has one, a list has one per item. */
    public static final class HelpLine implements Wire {
        private final List<HelpRun> runs;

        public HelpLine(List<HelpRun> runs) {
            this.runs = runs;
        }

        public List<HelpRun> getRunsList() {
            return runs == null ? Collections.<HelpRun>emptyList() : runs;
        }

        public int getRunsCount() {
            return runs == null ? 0 : runs.size();
        }

        public HelpRun getRuns(int $index) {
            return runs.get($index);
        }

        public static HelpLine read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "runs", "HelpLine");
            return new HelpLine(
                    Json.list($raw0, $x0 -> HelpRun.read($x0)));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "runs");
            $out.append('[');
            for (int $i0 = 0; $i0 < runs.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                runs.get($i0).write($out);
            }
            $out.append(']');
            $out.append('}');
        }
    }

    /** An article as a list shows it. */
    public static final class HelpLink implements Wire {
        private final String slug;
        private final String summary;
        private final String title;

        public HelpLink(String slug, String summary, String title) {
            this.slug = slug;
            this.summary = summary;
            this.title = title;
        }

        public String getSlug() {
            return slug;
        }

        public boolean hasSummary() {
            return summary != null;
        }

        public String getSummary() {
            return summary == null ? "" : summary;
        }

        public String getTitle() {
            return title;
        }

        public static HelpLink read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "slug", "HelpLink");
            Object $raw1 = Json.at($fields, "summary");
            Object $raw2 = Json.need($fields, "title", "HelpLink");
            return new HelpLink(
                    Json.asText($raw0),
                    $raw1 == null ? null : Json.asText($raw1),
                    Json.asText($raw2));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "slug");
            Json.text($out, slug);
            if (summary != null) {
                $count = Json.key($out, $count, "summary");
                Json.text($out, summary);
            }
            $count = Json.key($out, $count, "title");
            Json.text($out, title);
            $out.append('}');
        }
    }

    /** `/v1/help/policies/{id}`. */
    public static final class HelpPolicy implements Wire {
        private final String id;
        private final List<HelpSection> sections;
        private final String title;
        private final String updated;

        public HelpPolicy(String id, List<HelpSection> sections, String title, String updated) {
            this.id = id;
            this.sections = sections;
            this.title = title;
            this.updated = updated;
        }

        public String getId() {
            return id;
        }

        public List<HelpSection> getSectionsList() {
            return sections == null ? Collections.<HelpSection>emptyList() : sections;
        }

        public int getSectionsCount() {
            return sections == null ? 0 : sections.size();
        }

        public HelpSection getSections(int $index) {
            return sections.get($index);
        }

        public String getTitle() {
            return title;
        }

        public boolean hasUpdated() {
            return updated != null;
        }

        public String getUpdated() {
            return updated == null ? "" : updated;
        }

        public static HelpPolicy read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "id", "HelpPolicy");
            Object $raw1 = Json.need($fields, "sections", "HelpPolicy");
            Object $raw2 = Json.need($fields, "title", "HelpPolicy");
            Object $raw3 = Json.at($fields, "updated");
            return new HelpPolicy(
                    Json.asText($raw0),
                    Json.list($raw1, $x0 -> HelpSection.read($x0)),
                    Json.asText($raw2),
                    $raw3 == null ? null : Json.asText($raw3));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "id");
            Json.text($out, id);
            $count = Json.key($out, $count, "sections");
            $out.append('[');
            for (int $i0 = 0; $i0 < sections.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                sections.get($i0).write($out);
            }
            $out.append(']');
            $count = Json.key($out, $count, "title");
            Json.text($out, title);
            if (updated != null) {
                $count = Json.key($out, $count, "updated");
                Json.text($out, updated);
            }
            $out.append('}');
        }
    }

    public static final class HelpPolicyLink implements Wire {
        private final String id;
        private final String title;

        public HelpPolicyLink(String id, String title) {
            this.id = id;
            this.title = title;
        }

        public String getId() {
            return id;
        }

        public String getTitle() {
            return title;
        }

        public static HelpPolicyLink read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "id", "HelpPolicyLink");
            Object $raw1 = Json.need($fields, "title", "HelpPolicyLink");
            return new HelpPolicyLink(
                    Json.asText($raw0),
                    Json.asText($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "id");
            Json.text($out, id);
            $count = Json.key($out, $count, "title");
            Json.text($out, title);
            $out.append('}');
        }
    }

    /** `/v1/help/search`. */
    public static final class HelpResults implements Wire {
        private final List<HelpLink> articles;

        public HelpResults(List<HelpLink> articles) {
            this.articles = articles;
        }

        public List<HelpLink> getArticlesList() {
            return articles == null ? Collections.<HelpLink>emptyList() : articles;
        }

        public int getArticlesCount() {
            return articles == null ? 0 : articles.size();
        }

        public HelpLink getArticles(int $index) {
            return articles.get($index);
        }

        public static HelpResults read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "articles", "HelpResults");
            return new HelpResults(
                    Json.list($raw0, $x0 -> HelpLink.read($x0)));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "articles");
            $out.append('[');
            for (int $i0 = 0; $i0 < articles.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                articles.get($i0).write($out);
            }
            $out.append(']');
            $out.append('}');
        }
    }

    /** Some words of a line, with how they are marked. */
    public static final class HelpRun implements Wire {
        private final Boolean bold;
        private final String link;
        private final String text;

        public HelpRun(Boolean bold, String link, String text) {
            this.bold = bold;
            this.link = link;
            this.text = text;
        }

        public boolean hasBold() {
            return bold != null;
        }

        public boolean getBold() {
            return bold != null && bold;
        }

        /** Where the words lead: an address in the app, a web page or a mail address. */
        public boolean hasLink() {
            return link != null;
        }

        public String getLink() {
            return link == null ? "" : link;
        }

        public String getText() {
            return text;
        }

        public static HelpRun read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "bold");
            Object $raw1 = Json.at($fields, "link");
            Object $raw2 = Json.need($fields, "text", "HelpRun");
            return new HelpRun(
                    $raw0 == null ? null : Boolean.valueOf(Json.asBool($raw0)),
                    $raw1 == null ? null : Json.asText($raw1),
                    Json.asText($raw2));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (bold != null) {
                $count = Json.key($out, $count, "bold");
                $out.append(bold);
            }
            if (link != null) {
                $count = Json.key($out, $count, "link");
                Json.text($out, link);
            }
            $count = Json.key($out, $count, "text");
            Json.text($out, text);
            $out.append('}');
        }
    }

    /** A part of a page. A folded one starts closed where the app can open and close it. */
    public static final class HelpSection implements Wire {
        private final List<HelpBlock> blocks;
        private final Boolean folded;
        private final String title;

        public HelpSection(List<HelpBlock> blocks, Boolean folded, String title) {
            this.blocks = blocks;
            this.folded = folded;
            this.title = title;
        }

        public List<HelpBlock> getBlocksList() {
            return blocks == null ? Collections.<HelpBlock>emptyList() : blocks;
        }

        public int getBlocksCount() {
            return blocks == null ? 0 : blocks.size();
        }

        public HelpBlock getBlocks(int $index) {
            return blocks.get($index);
        }

        public boolean hasFolded() {
            return folded != null;
        }

        public boolean getFolded() {
            return folded != null && folded;
        }

        public boolean hasTitle() {
            return title != null;
        }

        public String getTitle() {
            return title == null ? "" : title;
        }

        public static HelpSection read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "blocks", "HelpSection");
            Object $raw1 = Json.at($fields, "folded");
            Object $raw2 = Json.at($fields, "title");
            return new HelpSection(
                    Json.list($raw0, $x0 -> HelpBlock.read($x0)),
                    $raw1 == null ? null : Boolean.valueOf(Json.asBool($raw1)),
                    $raw2 == null ? null : Json.asText($raw2));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "blocks");
            $out.append('[');
            for (int $i0 = 0; $i0 < blocks.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                blocks.get($i0).write($out);
            }
            $out.append(']');
            if (folded != null) {
                $count = Json.key($out, $count, "folded");
                $out.append(folded);
            }
            if (title != null) {
                $count = Json.key($out, $count, "title");
                Json.text($out, title);
            }
            $out.append('}');
        }
    }

    public static final class HelpTopic implements Wire {
        private final List<HelpGroup> groups;
        private final String id;
        private final String title;

        public HelpTopic(List<HelpGroup> groups, String id, String title) {
            this.groups = groups;
            this.id = id;
            this.title = title;
        }

        public List<HelpGroup> getGroupsList() {
            return groups == null ? Collections.<HelpGroup>emptyList() : groups;
        }

        public int getGroupsCount() {
            return groups == null ? 0 : groups.size();
        }

        public HelpGroup getGroups(int $index) {
            return groups.get($index);
        }

        public String getId() {
            return id;
        }

        public String getTitle() {
            return title;
        }

        public static HelpTopic read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "groups", "HelpTopic");
            Object $raw1 = Json.need($fields, "id", "HelpTopic");
            Object $raw2 = Json.need($fields, "title", "HelpTopic");
            return new HelpTopic(
                    Json.list($raw0, $x0 -> HelpGroup.read($x0)),
                    Json.asText($raw1),
                    Json.asText($raw2));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "groups");
            $out.append('[');
            for (int $i0 = 0; $i0 < groups.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                groups.get($i0).write($out);
            }
            $out.append(']');
            $count = Json.key($out, $count, "id");
            Json.text($out, id);
            $count = Json.key($out, $count, "title");
            Json.text($out, title);
            $out.append('}');
        }
    }

    /** `POST /v1/core/step`. `config` is sent on the first step of a hosted session. */
    public static final class HostedStep implements Wire {
        private final CoreConfig config;
        private final CoreIn input;

        public HostedStep(CoreConfig config, CoreIn input) {
            this.config = config;
            this.input = input;
        }

        public boolean hasConfig() {
            return config != null;
        }

        public CoreConfig getConfig() {
            return config == null ? null : config;
        }

        public CoreIn getInput() {
            return input;
        }

        public static HostedStep read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "config");
            Object $raw1 = Json.need($fields, "input", "HostedStep");
            return new HostedStep(
                    $raw0 == null ? null : CoreConfig.read($raw0),
                    CoreIn.read($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (config != null) {
                $count = Json.key($out, $count, "config");
                config.write($out);
            }
            $count = Json.key($out, $count, "input");
            input.write($out);
            $out.append('}');
        }
    }

    /** How an http effect may be handled. */
    public static final class HttpClass implements Wire {
        public static final HttpClass COMMAND = new HttpClass("command");
        public static final HttpClass CACHEABLE = new HttpClass("cacheable");
        public static final HttpClass PLAIN = new HttpClass("plain");

        private final String value;

        public HttpClass(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static HttpClass read(Object $json) {
            return new HttpClass(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof HttpClass && (((HttpClass) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** Methods of an http effect. */
    public static final class HttpMethod implements Wire {
        public static final HttpMethod GET = new HttpMethod("GET");
        public static final HttpMethod POST = new HttpMethod("POST");
        public static final HttpMethod PUT = new HttpMethod("PUT");
        public static final HttpMethod PATCH = new HttpMethod("PATCH");
        public static final HttpMethod DELETE = new HttpMethod("DELETE");

        private final String value;

        public HttpMethod(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static HttpMethod read(Object $json) {
            return new HttpMethod(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof HttpMethod && (((HttpMethod) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** One image address. Images go straight from the host to the viewer. */
    public static final class Image implements Wire {
        private final ImageRole role;
        private final String url;
        private final Long width;

        public Image(ImageRole role, String url, Long width) {
            this.role = role;
            this.url = url;
            this.width = width;
        }

        public ImageRole getRole() {
            return role;
        }

        public String getUrl() {
            return url;
        }

        public boolean hasWidth() {
            return width != null;
        }

        public long getWidth() {
            return width == null ? 0L : width;
        }

        public static Image read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "role", "Image");
            Object $raw1 = Json.need($fields, "url", "Image");
            Object $raw2 = Json.at($fields, "width");
            return new Image(
                    ImageRole.read($raw0),
                    Json.asText($raw1),
                    $raw2 == null ? null : Long.valueOf(Json.asLong($raw2)));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "role");
            role.write($out);
            $count = Json.key($out, $count, "url");
            Json.text($out, url);
            if (width != null) {
                $count = Json.key($out, $count, "width");
                $out.append(width);
            }
            $out.append('}');
        }
    }

    /** What an image is for. */
    public static final class ImageRole implements Wire {
        public static final ImageRole POSTER = new ImageRole("poster");
        public static final ImageRole BACKDROP = new ImageRole("backdrop");
        public static final ImageRole LOGO = new ImageRole("logo");
        public static final ImageRole STILL = new ImageRole("still");
        public static final ImageRole AVATAR = new ImageRole("avatar");
        public static final ImageRole ART = new ImageRole("art");

        private final String value;

        public ImageRole(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static ImageRole read(Object $json) {
            return new ImageRole(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof ImageRole && (((ImageRole) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** Input a device has. */
    public static final class InputKind implements Wire {
        public static final InputKind TOUCH = new InputKind("touch");
        public static final InputKind POINTER = new InputKind("pointer");
        public static final InputKind KEYBOARD = new InputKind("keyboard");
        public static final InputKind REMOTE = new InputKind("remote");
        public static final InputKind GAMEPAD = new InputKind("gamepad");
        public static final InputKind ROTARY = new InputKind("rotary");

        private final String value;

        public InputKind(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static InputKind read(Object $json) {
            return new InputKind(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof InputKind && (((InputKind) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** A language this server has text for, named in itself. */
    public static final class Language implements Wire {
        private final String code;
        private final TextDirection dir;
        private final String name;

        public Language(String code, TextDirection dir, String name) {
            this.code = code;
            this.dir = dir;
            this.name = name;
        }

        public String getCode() {
            return code;
        }

        public boolean hasDir() {
            return dir != null;
        }

        public TextDirection getDir() {
            return dir == null ? null : dir;
        }

        public String getName() {
            return name;
        }

        public static Language read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "code", "Language");
            Object $raw1 = Json.at($fields, "dir");
            Object $raw2 = Json.need($fields, "name", "Language");
            return new Language(
                    Json.asText($raw0),
                    $raw1 == null ? null : TextDirection.read($raw1),
                    Json.asText($raw2));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "code");
            Json.text($out, code);
            if (dir != null) {
                $count = Json.key($out, $count, "dir");
                dir.write($out);
            }
            $count = Json.key($out, $count, "name");
            Json.text($out, name);
            $out.append('}');
        }
    }

    public static final class LanguageList implements Wire {
        private final List<Language> languages;

        public LanguageList(List<Language> languages) {
            this.languages = languages;
        }

        public List<Language> getLanguagesList() {
            return languages == null ? Collections.<Language>emptyList() : languages;
        }

        public int getLanguagesCount() {
            return languages == null ? 0 : languages.size();
        }

        public Language getLanguages(int $index) {
            return languages.get($index);
        }

        public static LanguageList read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "languages", "LanguageList");
            return new LanguageList(
                    Json.list($raw0, $x0 -> Language.read($x0)));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "languages");
            $out.append('[');
            for (int $i0 = 0; $i0 < languages.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                languages.get($i0).write($out);
            }
            $out.append(']');
            $out.append('}');
        }
    }

    /** Asks `core_open` for the offline layer. A host sets it as the `layer` key of the text it gives `core_open`, next to the `CoreConfig` keys. With no `layer` key the core runs alone, and the core itself never reads this key. */
    public static final class LayerConfig implements Wire {
        private final String bootId;
        private final long storageBytes;

        public LayerConfig(String bootId, long storageBytes) {
            this.bootId = bootId;
            this.storageBytes = storageBytes;
        }

        /** Names this boot. It changes when monotonic time restarts. */
        public String getBootId() {
            return bootId;
        }

        /** The byte cap from the capability card. 0 means nothing is kept across a restart. */
        public long getStorageBytes() {
            return storageBytes;
        }

        public static LayerConfig read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "boot_id", "LayerConfig");
            Object $raw1 = Json.need($fields, "storage_bytes", "LayerConfig");
            return new LayerConfig(
                    Json.asText($raw0),
                    Json.asLong($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "boot_id");
            Json.text($out, bootId);
            $count = Json.key($out, $count, "storage_bytes");
            $out.append(storageBytes);
            $out.append('}');
        }
    }

    /** Limits served from one place, so no client holds a copy. */
    public static final class Limits implements Wire {
        private final long blockedTitleMax;
        private final long blockedTitlesMax;
        private final long codeDigits;
        private final long codeValidS;
        private final long commandBatchMax;
        private final long handleMax;
        private final long handleMin;
        private final long nameMax;
        private final long passwordMin;
        private final long phoneDigitsMax;
        private final long phoneDigitsMin;
        private final long pinDigits;
        private final long profileNameMax;
        private final long profilesMax;
        private final long stepUpS;

        public Limits(long blockedTitleMax, long blockedTitlesMax, long codeDigits, long codeValidS, long commandBatchMax, long handleMax, long handleMin, long nameMax, long passwordMin, long phoneDigitsMax, long phoneDigitsMin, long pinDigits, long profileNameMax, long profilesMax, long stepUpS) {
            this.blockedTitleMax = blockedTitleMax;
            this.blockedTitlesMax = blockedTitlesMax;
            this.codeDigits = codeDigits;
            this.codeValidS = codeValidS;
            this.commandBatchMax = commandBatchMax;
            this.handleMax = handleMax;
            this.handleMin = handleMin;
            this.nameMax = nameMax;
            this.passwordMin = passwordMin;
            this.phoneDigitsMax = phoneDigitsMax;
            this.phoneDigitsMin = phoneDigitsMin;
            this.pinDigits = pinDigits;
            this.profileNameMax = profileNameMax;
            this.profilesMax = profilesMax;
            this.stepUpS = stepUpS;
        }

        public long getBlockedTitleMax() {
            return blockedTitleMax;
        }

        public long getBlockedTitlesMax() {
            return blockedTitlesMax;
        }

        public long getCodeDigits() {
            return codeDigits;
        }

        public long getCodeValidS() {
            return codeValidS;
        }

        public long getCommandBatchMax() {
            return commandBatchMax;
        }

        public long getHandleMax() {
            return handleMax;
        }

        public long getHandleMin() {
            return handleMin;
        }

        public long getNameMax() {
            return nameMax;
        }

        public long getPasswordMin() {
            return passwordMin;
        }

        public long getPhoneDigitsMax() {
            return phoneDigitsMax;
        }

        public long getPhoneDigitsMin() {
            return phoneDigitsMin;
        }

        public long getPinDigits() {
            return pinDigits;
        }

        public long getProfileNameMax() {
            return profileNameMax;
        }

        public long getProfilesMax() {
            return profilesMax;
        }

        public long getStepUpS() {
            return stepUpS;
        }

        public static Limits read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "blocked_title_max", "Limits");
            Object $raw1 = Json.need($fields, "blocked_titles_max", "Limits");
            Object $raw2 = Json.need($fields, "code_digits", "Limits");
            Object $raw3 = Json.need($fields, "code_valid_s", "Limits");
            Object $raw4 = Json.need($fields, "command_batch_max", "Limits");
            Object $raw5 = Json.need($fields, "handle_max", "Limits");
            Object $raw6 = Json.need($fields, "handle_min", "Limits");
            Object $raw7 = Json.need($fields, "name_max", "Limits");
            Object $raw8 = Json.need($fields, "password_min", "Limits");
            Object $raw9 = Json.need($fields, "phone_digits_max", "Limits");
            Object $raw10 = Json.need($fields, "phone_digits_min", "Limits");
            Object $raw11 = Json.need($fields, "pin_digits", "Limits");
            Object $raw12 = Json.need($fields, "profile_name_max", "Limits");
            Object $raw13 = Json.need($fields, "profiles_max", "Limits");
            Object $raw14 = Json.need($fields, "step_up_s", "Limits");
            return new Limits(
                    Json.asLong($raw0),
                    Json.asLong($raw1),
                    Json.asLong($raw2),
                    Json.asLong($raw3),
                    Json.asLong($raw4),
                    Json.asLong($raw5),
                    Json.asLong($raw6),
                    Json.asLong($raw7),
                    Json.asLong($raw8),
                    Json.asLong($raw9),
                    Json.asLong($raw10),
                    Json.asLong($raw11),
                    Json.asLong($raw12),
                    Json.asLong($raw13),
                    Json.asLong($raw14));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "blocked_title_max");
            $out.append(blockedTitleMax);
            $count = Json.key($out, $count, "blocked_titles_max");
            $out.append(blockedTitlesMax);
            $count = Json.key($out, $count, "code_digits");
            $out.append(codeDigits);
            $count = Json.key($out, $count, "code_valid_s");
            $out.append(codeValidS);
            $count = Json.key($out, $count, "command_batch_max");
            $out.append(commandBatchMax);
            $count = Json.key($out, $count, "handle_max");
            $out.append(handleMax);
            $count = Json.key($out, $count, "handle_min");
            $out.append(handleMin);
            $count = Json.key($out, $count, "name_max");
            $out.append(nameMax);
            $count = Json.key($out, $count, "password_min");
            $out.append(passwordMin);
            $count = Json.key($out, $count, "phone_digits_max");
            $out.append(phoneDigitsMax);
            $count = Json.key($out, $count, "phone_digits_min");
            $out.append(phoneDigitsMin);
            $count = Json.key($out, $count, "pin_digits");
            $out.append(pinDigits);
            $count = Json.key($out, $count, "profile_name_max");
            $out.append(profileNameMax);
            $count = Json.key($out, $count, "profiles_max");
            $out.append(profilesMax);
            $count = Json.key($out, $count, "step_up_s");
            $out.append(stepUpS);
            $out.append('}');
        }
    }

    /** List state of a title, versioned apart from its rating. */
    public static final class ListState implements Wire {
        private final long changedAtMs;
        private final boolean inList;
        private final long version;

        public ListState(long changedAtMs, boolean inList, long version) {
            this.changedAtMs = changedAtMs;
            this.inList = inList;
            this.version = version;
        }

        public long getChangedAtMs() {
            return changedAtMs;
        }

        public boolean getInList() {
            return inList;
        }

        public long getVersion() {
            return version;
        }

        public static ListState read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "changed_at_ms", "ListState");
            Object $raw1 = Json.need($fields, "in_list", "ListState");
            Object $raw2 = Json.need($fields, "version", "ListState");
            return new ListState(
                    Json.asLong($raw0),
                    Json.asBool($raw1),
                    Json.asLong($raw2));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "changed_at_ms");
            $out.append(changedAtMs);
            $count = Json.key($out, $count, "in_list");
            $out.append(inList);
            $count = Json.key($out, $count, "version");
            $out.append(version);
            $out.append('}');
        }
    }

    /** Log levels. */
    public static final class LogLevel implements Wire {
        public static final LogLevel DEBUG = new LogLevel("debug");
        public static final LogLevel INFO = new LogLevel("info");
        public static final LogLevel WARN = new LogLevel("warn");
        public static final LogLevel ERROR = new LogLevel("error");

        private final String value;

        public LogLevel(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static LogLevel read(Object $json) {
            return new LogLevel(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof LogLevel && (((LogLevel) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** Skip marks. */
    public static final class Marks implements Wire {
        private final List<TimeWindow> windows;

        public Marks(List<TimeWindow> windows) {
            this.windows = windows;
        }

        public List<TimeWindow> getWindowsList() {
            return windows == null ? Collections.<TimeWindow>emptyList() : windows;
        }

        public int getWindowsCount() {
            return windows == null ? 0 : windows.size();
        }

        public TimeWindow getWindows(int $index) {
            return windows.get($index);
        }

        public static Marks read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "windows", "Marks");
            return new Marks(
                    Json.list($raw0, $x0 -> TimeWindow.read($x0)));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "windows");
            $out.append('[');
            for (int $i0 = 0; $i0 < windows.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                windows.get($i0).write($out);
            }
            $out.append(']');
            $out.append('}');
        }
    }

    /** Ask for skip marks, after first frame, series only. */
    public static final class MarksRequest implements Wire {
        private final String attempt;
        private final Long delayMs;
        private final long durationMs;
        private final String file;

        public MarksRequest(String attempt, Long delayMs, long durationMs, String file) {
            this.attempt = attempt;
            this.delayMs = delayMs;
            this.durationMs = durationMs;
            this.file = file;
        }

        public String getAttempt() {
            return attempt;
        }

        public boolean hasDelayMs() {
            return delayMs != null;
        }

        public long getDelayMs() {
            return delayMs == null ? 0L : delayMs;
        }

        public long getDurationMs() {
            return durationMs;
        }

        public boolean hasFile() {
            return file != null;
        }

        public String getFile() {
            return file == null ? "" : file;
        }

        public static MarksRequest read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "attempt", "MarksRequest");
            Object $raw1 = Json.at($fields, "delay_ms");
            Object $raw2 = Json.need($fields, "duration_ms", "MarksRequest");
            Object $raw3 = Json.at($fields, "file");
            return new MarksRequest(
                    Json.asText($raw0),
                    $raw1 == null ? null : Long.valueOf(Json.asLong($raw1)),
                    Json.asLong($raw2),
                    $raw3 == null ? null : Json.asText($raw3));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "attempt");
            Json.text($out, attempt);
            if (delayMs != null) {
                $count = Json.key($out, $count, "delay_ms");
                $out.append(delayMs);
            }
            $count = Json.key($out, $count, "duration_ms");
            $out.append(durationMs);
            if (file != null) {
                $count = Json.key($out, $count, "file");
                Json.text($out, file);
            }
            $out.append('}');
        }
    }

    /** The next part of a screen or a section. */
    public static final class More implements Wire {
        private final String cursor;
        private final List<Section> sections;
        private final List<Tile> tiles;

        public More(String cursor, List<Section> sections, List<Tile> tiles) {
            this.cursor = cursor;
            this.sections = sections;
            this.tiles = tiles;
        }

        public boolean hasCursor() {
            return cursor != null;
        }

        public String getCursor() {
            return cursor == null ? "" : cursor;
        }

        public List<Section> getSectionsList() {
            return sections == null ? Collections.<Section>emptyList() : sections;
        }

        public int getSectionsCount() {
            return sections == null ? 0 : sections.size();
        }

        public Section getSections(int $index) {
            return sections.get($index);
        }

        public List<Tile> getTilesList() {
            return tiles == null ? Collections.<Tile>emptyList() : tiles;
        }

        public int getTilesCount() {
            return tiles == null ? 0 : tiles.size();
        }

        public Tile getTiles(int $index) {
            return tiles.get($index);
        }

        public static More read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "cursor");
            Object $raw1 = Json.at($fields, "sections");
            Object $raw2 = Json.at($fields, "tiles");
            return new More(
                    $raw0 == null ? null : Json.asText($raw0),
                    $raw1 == null ? null : Json.list($raw1, $x0 -> Section.read($x0)),
                    $raw2 == null ? null : Json.list($raw2, $x0 -> Tile.read($x0)));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (cursor != null) {
                $count = Json.key($out, $count, "cursor");
                Json.text($out, cursor);
            }
            if (sections != null) {
                $count = Json.key($out, $count, "sections");
                $out.append('[');
                for (int $i0 = 0; $i0 < sections.size(); $i0++) {
                    if ($i0 > 0) {
                        $out.append(',');
                    }
                    sections.get($i0).write($out);
                }
                $out.append(']');
            }
            if (tiles != null) {
                $count = Json.key($out, $count, "tiles");
                $out.append('[');
                for (int $i0 = 0; $i0 < tiles.size(); $i0++) {
                    if ($i0 > 0) {
                        $out.append(',');
                    }
                    tiles.get($i0).write($out);
                }
                $out.append(']');
            }
            $out.append('}');
        }
    }

    /** Late attempts. */
    public static final class MoreAttempts implements Wire {
        private final List<Attempt> attempts;
        private final boolean more;

        public MoreAttempts(List<Attempt> attempts, boolean more) {
            this.attempts = attempts;
            this.more = more;
        }

        public List<Attempt> getAttemptsList() {
            return attempts == null ? Collections.<Attempt>emptyList() : attempts;
        }

        public int getAttemptsCount() {
            return attempts == null ? 0 : attempts.size();
        }

        public Attempt getAttempts(int $index) {
            return attempts.get($index);
        }

        public boolean getMore() {
            return more;
        }

        public static MoreAttempts read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "attempts", "MoreAttempts");
            Object $raw1 = Json.need($fields, "more", "MoreAttempts");
            return new MoreAttempts(
                    Json.list($raw0, $x0 -> Attempt.read($x0)),
                    Json.asBool($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "attempts");
            $out.append('[');
            for (int $i0 = 0; $i0 < attempts.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                attempts.get($i0).write($out);
            }
            $out.append(']');
            $count = Json.key($out, $count, "more");
            $out.append(more);
            $out.append('}');
        }
    }

    /** One row of the moved address table. */
    public static final class MovedAddress implements Wire {
        private final String from;
        private final String to;

        public MovedAddress(String from, String to) {
            this.from = from;
            this.to = to;
        }

        public String getFrom() {
            return from;
        }

        public String getTo() {
            return to;
        }

        public static MovedAddress read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "from", "MovedAddress");
            Object $raw1 = Json.need($fields, "to", "MovedAddress");
            return new MovedAddress(
                    Json.asText($raw0),
                    Json.asText($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "from");
            Json.text($out, from);
            $count = Json.key($out, $count, "to");
            Json.text($out, to);
            $out.append('}');
        }
    }

    /** The menu for this class and profile. */
    public static final class Nav implements Wire {
        private final List<NavItem> items;
        private final long ttlS;

        public Nav(List<NavItem> items, long ttlS) {
            this.items = items;
            this.ttlS = ttlS;
        }

        public List<NavItem> getItemsList() {
            return items == null ? Collections.<NavItem>emptyList() : items;
        }

        public int getItemsCount() {
            return items == null ? 0 : items.size();
        }

        public NavItem getItems(int $index) {
            return items.get($index);
        }

        public long getTtlS() {
            return ttlS;
        }

        public static Nav read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "items", "Nav");
            Object $raw1 = Json.need($fields, "ttl_s", "Nav");
            return new Nav(
                    Json.list($raw0, $x0 -> NavItem.read($x0)),
                    Json.asLong($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "items");
            $out.append('[');
            for (int $i0 = 0; $i0 < items.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                items.get($i0).write($out);
            }
            $out.append(']');
            $count = Json.key($out, $count, "ttl_s");
            $out.append(ttlS);
            $out.append('}');
        }
    }

    /** One entry of the menu. */
    public static final class NavItem implements Wire {
        private final String genre;
        private final String id;
        private final String label;
        private final ScreenName screen;

        public NavItem(String genre, String id, String label, ScreenName screen) {
            this.genre = genre;
            this.id = id;
            this.label = label;
            this.screen = screen;
        }

        public boolean hasGenre() {
            return genre != null;
        }

        public String getGenre() {
            return genre == null ? "" : genre;
        }

        public String getId() {
            return id;
        }

        public String getLabel() {
            return label;
        }

        public ScreenName getScreen() {
            return screen;
        }

        public static NavItem read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "genre");
            Object $raw1 = Json.need($fields, "id", "NavItem");
            Object $raw2 = Json.need($fields, "label", "NavItem");
            Object $raw3 = Json.need($fields, "screen", "NavItem");
            return new NavItem(
                    $raw0 == null ? null : Json.asText($raw0),
                    Json.asText($raw1),
                    Json.asText($raw2),
                    ScreenName.read($raw3));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (genre != null) {
                $count = Json.key($out, $count, "genre");
                Json.text($out, genre);
            }
            $count = Json.key($out, $count, "id");
            Json.text($out, id);
            $count = Json.key($out, $count, "label");
            Json.text($out, label);
            $count = Json.key($out, $count, "screen");
            screen.write($out);
            $out.append('}');
        }
    }

    /** Why an http effect got no answer. Any status at all is an answer. */
    public static final class NoAnswerReason implements Wire {
        public static final NoAnswerReason OFFLINE = new NoAnswerReason("offline");
        public static final NoAnswerReason TIMEOUT = new NoAnswerReason("timeout");
        public static final NoAnswerReason TLS = new NoAnswerReason("tls");
        public static final NoAnswerReason CLOCK_WRONG = new NoAnswerReason("clock_wrong");
        public static final NoAnswerReason CANCELLED = new NoAnswerReason("cancelled");

        private final String value;

        public NoAnswerReason(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static NoAnswerReason read(Object $json) {
            return new NoAnswerReason(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof NoAnswerReason && (((NoAnswerReason) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** Text a client can draw with no other knowledge, including a client below the floor. */
    public static final class Notice implements Wire {
        private final String actionAddress;
        private final String actionLabel;
        private final String label;
        private final String text;

        public Notice(String actionAddress, String actionLabel, String label, String text) {
            this.actionAddress = actionAddress;
            this.actionLabel = actionLabel;
            this.label = label;
            this.text = text;
        }

        public boolean hasActionAddress() {
            return actionAddress != null;
        }

        public String getActionAddress() {
            return actionAddress == null ? "" : actionAddress;
        }

        public boolean hasActionLabel() {
            return actionLabel != null;
        }

        public String getActionLabel() {
            return actionLabel == null ? "" : actionLabel;
        }

        /** Label key, for clients with the strings. */
        public String getLabel() {
            return label;
        }

        /** Plain text in the viewer's language. */
        public String getText() {
            return text;
        }

        public static Notice read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "action_address");
            Object $raw1 = Json.at($fields, "action_label");
            Object $raw2 = Json.need($fields, "label", "Notice");
            Object $raw3 = Json.need($fields, "text", "Notice");
            return new Notice(
                    $raw0 == null ? null : Json.asText($raw0),
                    $raw1 == null ? null : Json.asText($raw1),
                    Json.asText($raw2),
                    Json.asText($raw3));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (actionAddress != null) {
                $count = Json.key($out, $count, "action_address");
                Json.text($out, actionAddress);
            }
            if (actionLabel != null) {
                $count = Json.key($out, $count, "action_label");
                Json.text($out, actionLabel);
            }
            $count = Json.key($out, $count, "label");
            Json.text($out, label);
            $count = Json.key($out, $count, "text");
            Json.text($out, text);
            $out.append('}');
        }
    }

    /** One item in the notification feed. */
    public static final class NoticeItem implements Wire {
        private final CalendarDay date;
        private final String id;
        private final NoticeKind kind;
        private final Tile tile;
        private final boolean unread;

        public NoticeItem(CalendarDay date, String id, NoticeKind kind, Tile tile, boolean unread) {
            this.date = date;
            this.id = id;
            this.kind = kind;
            this.tile = tile;
            this.unread = unread;
        }

        public CalendarDay getDate() {
            return date;
        }

        public String getId() {
            return id;
        }

        public NoticeKind getKind() {
            return kind;
        }

        public Tile getTile() {
            return tile;
        }

        public boolean getUnread() {
            return unread;
        }

        public static NoticeItem read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "date", "NoticeItem");
            Object $raw1 = Json.need($fields, "id", "NoticeItem");
            Object $raw2 = Json.need($fields, "kind", "NoticeItem");
            Object $raw3 = Json.need($fields, "tile", "NoticeItem");
            Object $raw4 = Json.need($fields, "unread", "NoticeItem");
            return new NoticeItem(
                    CalendarDay.read($raw0),
                    Json.asText($raw1),
                    NoticeKind.read($raw2),
                    Tile.read($raw3),
                    Json.asBool($raw4));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "date");
            date.write($out);
            $count = Json.key($out, $count, "id");
            Json.text($out, id);
            $count = Json.key($out, $count, "kind");
            kind.write($out);
            $count = Json.key($out, $count, "tile");
            tile.write($out);
            $count = Json.key($out, $count, "unread");
            $out.append(unread);
            $out.append('}');
        }
    }

    /** What kind of notice this is. */
    public static final class NoticeKind implements Wire {
        public static final NoticeKind NEW_ARRIVAL = new NoticeKind("new_arrival");
        public static final NoticeKind COMING = new NoticeKind("coming");
        public static final NoticeKind REMINDER_LIVE = new NoticeKind("reminder_live");
        public static final NoticeKind NEW_SEASON = new NoticeKind("new_season");
        public static final NoticeKind NEW_EPISODE = new NoticeKind("new_episode");

        private final String value;

        public NoticeKind(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static NoticeKind read(Object $json) {
            return new NoticeKind(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof NoticeKind && (((NoticeKind) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** `GET /v1/notifications/count`. */
    public static final class NotificationCount implements Wire {
        private final long unread;

        public NotificationCount(long unread) {
            this.unread = unread;
        }

        public long getUnread() {
            return unread;
        }

        public static NotificationCount read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "unread", "NotificationCount");
            return new NotificationCount(
                    Json.asLong($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "unread");
            $out.append(unread);
            $out.append('}');
        }
    }

    /** `POST /v1/notifications/read`. */
    public static final class NotificationRead implements Wire {
        private final String upTo;

        public NotificationRead(String upTo) {
            this.upTo = upTo;
        }

        public String getUpTo() {
            return upTo;
        }

        public static NotificationRead read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "up_to", "NotificationRead");
            return new NotificationRead(
                    Json.asText($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "up_to");
            Json.text($out, upTo);
            $out.append('}');
        }
    }

    /** `GET /v1/notifications`. */
    public static final class Notifications implements Wire {
        private final String cursor;
        private final boolean hasMore;
        private final List<NoticeItem> items;

        public Notifications(String cursor, boolean hasMore, List<NoticeItem> items) {
            this.cursor = cursor;
            this.hasMore = hasMore;
            this.items = items;
        }

        public boolean hasCursor() {
            return cursor != null;
        }

        public String getCursor() {
            return cursor == null ? "" : cursor;
        }

        public boolean getHasMore() {
            return hasMore;
        }

        public List<NoticeItem> getItemsList() {
            return items == null ? Collections.<NoticeItem>emptyList() : items;
        }

        public int getItemsCount() {
            return items == null ? 0 : items.size();
        }

        public NoticeItem getItems(int $index) {
            return items.get($index);
        }

        public static Notifications read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "cursor");
            Object $raw1 = Json.need($fields, "has_more", "Notifications");
            Object $raw2 = Json.need($fields, "items", "Notifications");
            return new Notifications(
                    $raw0 == null ? null : Json.asText($raw0),
                    Json.asBool($raw1),
                    Json.list($raw2, $x0 -> NoticeItem.read($x0)));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (cursor != null) {
                $count = Json.key($out, $count, "cursor");
                Json.text($out, cursor);
            }
            $count = Json.key($out, $count, "has_more");
            $out.append(hasMore);
            $count = Json.key($out, $count, "items");
            $out.append('[');
            for (int $i0 = 0; $i0 < items.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                items.get($i0).write($out);
            }
            $out.append(']');
            $out.append('}');
        }
    }

    /** Answer for one op. */
    public static final class OpAnswer implements Wire {
        private final String opId;
        private final Outcome outcome;

        public OpAnswer(String opId, Outcome outcome) {
            this.opId = opId;
            this.outcome = outcome;
        }

        public String getOpId() {
            return opId;
        }

        public Outcome getOutcome() {
            return outcome;
        }

        public static OpAnswer read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "op_id", "OpAnswer");
            Object $raw1 = Json.need($fields, "outcome", "OpAnswer");
            return new OpAnswer(
                    Json.asText($raw0),
                    Outcome.read($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "op_id");
            Json.text($out, opId);
            $count = Json.key($out, $count, "outcome");
            outcome.write($out);
            $out.append('}');
        }
    }

    /** Answer per op. */
    public static final class Outcome implements Wire {
        public static final Outcome APPLIED = new Outcome("applied");
        public static final Outcome SUPERSEDED = new Outcome("superseded");
        public static final Outcome REJECTED = new Outcome("rejected");

        private final String value;

        public Outcome(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static Outcome read(Object $json) {
            return new Outcome(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof Outcome && (((Outcome) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    public static final class PasswordChange implements Wire {
        private final String newPassword;

        public PasswordChange(String newPassword) {
            this.newPassword = newPassword;
        }

        public String getNewPassword() {
            return newPassword;
        }

        public static PasswordChange read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "new_password", "PasswordChange");
            return new PasswordChange(
                    Json.asText($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "new_password");
            Json.text($out, newPassword);
            $out.append('}');
        }
    }

    public static final class PasswordSignIn implements Wire {
        private final String email;
        private final String password;

        public PasswordSignIn(String email, String password) {
            this.email = email;
            this.password = password;
        }

        public String getEmail() {
            return email;
        }

        public String getPassword() {
            return password;
        }

        public static PasswordSignIn read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "email", "PasswordSignIn");
            Object $raw1 = Json.need($fields, "password", "PasswordSignIn");
            return new PasswordSignIn(
                    Json.asText($raw0),
                    Json.asText($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "email");
            Json.text($out, email);
            $count = Json.key($out, $count, "password");
            Json.text($out, password);
            $out.append('}');
        }
    }

    public static final class PhoneChange implements Wire {
        private final String phone;

        public PhoneChange(String phone) {
            this.phone = phone;
        }

        public String getPhone() {
            return phone;
        }

        public static PhoneChange read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "phone", "PhoneChange");
            return new PhoneChange(
                    Json.asText($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "phone");
            Json.text($out, phone);
            $out.append('}');
        }
    }

    /** Absent clears the PIN. */
    public static final class PinSet implements Wire {
        private final String pin;

        public PinSet(String pin) {
            this.pin = pin;
        }

        public boolean hasPin() {
            return pin != null;
        }

        public String getPin() {
            return pin == null ? "" : pin;
        }

        public static PinSet read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "pin");
            return new PinSet(
                    $raw0 == null ? null : Json.asText($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (pin != null) {
                $count = Json.key($out, $count, "pin");
                Json.text($out, pin);
            }
            $out.append('}');
        }
    }

    /** Answer to a play request. */
    public static final class PlayAnswer implements Wire {
        private final AgeCard ageCard;
        private final List<Attempt> attempts;
        private final String audioDefault;
        private final Boolean autoplayNext;
        private final long dropped;
        private final boolean more;
        private final EpisodeRef nextEpisode;
        private final String playId;
        private final long startAtMs;
        private final String subtitleDefault;

        public PlayAnswer(AgeCard ageCard, List<Attempt> attempts, String audioDefault, Boolean autoplayNext, long dropped, boolean more, EpisodeRef nextEpisode, String playId, long startAtMs, String subtitleDefault) {
            this.ageCard = ageCard;
            this.attempts = attempts;
            this.audioDefault = audioDefault;
            this.autoplayNext = autoplayNext;
            this.dropped = dropped;
            this.more = more;
            this.nextEpisode = nextEpisode;
            this.playId = playId;
            this.startAtMs = startAtMs;
            this.subtitleDefault = subtitleDefault;
        }

        public boolean hasAgeCard() {
            return ageCard != null;
        }

        public AgeCard getAgeCard() {
            return ageCard == null ? null : ageCard;
        }

        public List<Attempt> getAttemptsList() {
            return attempts == null ? Collections.<Attempt>emptyList() : attempts;
        }

        public int getAttemptsCount() {
            return attempts == null ? 0 : attempts.size();
        }

        public Attempt getAttempts(int $index) {
            return attempts.get($index);
        }

        public boolean hasAudioDefault() {
            return audioDefault != null;
        }

        public String getAudioDefault() {
            return audioDefault == null ? "" : audioDefault;
        }

        /** This profile's setting: the next episode starts by itself at the end of play. */
        public boolean hasAutoplayNext() {
            return autoplayNext != null;
        }

        public boolean getAutoplayNext() {
            return autoplayNext != null && autoplayNext;
        }

        /** Attempts removed by the capability card. */
        public long getDropped() {
            return dropped;
        }

        /** True when more attempts may arrive through `/v1/play/{id}/more`. */
        public boolean getMore() {
            return more;
        }

        /** The next episode in the server's order, across seasons. Absent on the last one. */
        public boolean hasNextEpisode() {
            return nextEpisode != null;
        }

        public EpisodeRef getNextEpisode() {
            return nextEpisode == null ? null : nextEpisode;
        }

        public String getPlayId() {
            return playId;
        }

        /** Where to start. */
        public long getStartAtMs() {
            return startAtMs;
        }

        public boolean hasSubtitleDefault() {
            return subtitleDefault != null;
        }

        public String getSubtitleDefault() {
            return subtitleDefault == null ? "" : subtitleDefault;
        }

        public static PlayAnswer read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "age_card");
            Object $raw1 = Json.need($fields, "attempts", "PlayAnswer");
            Object $raw2 = Json.at($fields, "audio_default");
            Object $raw3 = Json.at($fields, "autoplay_next");
            Object $raw4 = Json.need($fields, "dropped", "PlayAnswer");
            Object $raw5 = Json.need($fields, "more", "PlayAnswer");
            Object $raw6 = Json.at($fields, "next_episode");
            Object $raw7 = Json.need($fields, "play_id", "PlayAnswer");
            Object $raw8 = Json.need($fields, "start_at_ms", "PlayAnswer");
            Object $raw9 = Json.at($fields, "subtitle_default");
            return new PlayAnswer(
                    $raw0 == null ? null : AgeCard.read($raw0),
                    Json.list($raw1, $x0 -> Attempt.read($x0)),
                    $raw2 == null ? null : Json.asText($raw2),
                    $raw3 == null ? null : Boolean.valueOf(Json.asBool($raw3)),
                    Json.asLong($raw4),
                    Json.asBool($raw5),
                    $raw6 == null ? null : EpisodeRef.read($raw6),
                    Json.asText($raw7),
                    Json.asLong($raw8),
                    $raw9 == null ? null : Json.asText($raw9));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (ageCard != null) {
                $count = Json.key($out, $count, "age_card");
                ageCard.write($out);
            }
            $count = Json.key($out, $count, "attempts");
            $out.append('[');
            for (int $i0 = 0; $i0 < attempts.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                attempts.get($i0).write($out);
            }
            $out.append(']');
            if (audioDefault != null) {
                $count = Json.key($out, $count, "audio_default");
                Json.text($out, audioDefault);
            }
            if (autoplayNext != null) {
                $count = Json.key($out, $count, "autoplay_next");
                $out.append(autoplayNext);
            }
            $count = Json.key($out, $count, "dropped");
            $out.append(dropped);
            $count = Json.key($out, $count, "more");
            $out.append(more);
            if (nextEpisode != null) {
                $count = Json.key($out, $count, "next_episode");
                nextEpisode.write($out);
            }
            $count = Json.key($out, $count, "play_id");
            Json.text($out, playId);
            $count = Json.key($out, $count, "start_at_ms");
            $out.append(startAtMs);
            if (subtitleDefault != null) {
                $count = Json.key($out, $count, "subtitle_default");
                Json.text($out, subtitleDefault);
            }
            $out.append('}');
        }
    }

    /** One thing that happened during play. */
    public static final class PlayEvent implements Wire {
        private final long atMs;
        private final String attempt;
        private final ErrorClass failure;
        private final Long firstFrameMs;
        private final String host;
        private final PlayEventName name;
        private final CoreMode rung;

        public PlayEvent(long atMs, String attempt, ErrorClass failure, Long firstFrameMs, String host, PlayEventName name, CoreMode rung) {
            this.atMs = atMs;
            this.attempt = attempt;
            this.failure = failure;
            this.firstFrameMs = firstFrameMs;
            this.host = host;
            this.name = name;
            this.rung = rung;
        }

        public long getAtMs() {
            return atMs;
        }

        public boolean hasAttempt() {
            return attempt != null;
        }

        public String getAttempt() {
            return attempt == null ? "" : attempt;
        }

        public boolean hasFailure() {
            return failure != null;
        }

        public ErrorClass getFailure() {
            return failure == null ? null : failure;
        }

        public boolean hasFirstFrameMs() {
            return firstFrameMs != null;
        }

        public long getFirstFrameMs() {
            return firstFrameMs == null ? 0L : firstFrameMs;
        }

        public boolean hasHost() {
            return host != null;
        }

        public String getHost() {
            return host == null ? "" : host;
        }

        public PlayEventName getName() {
            return name;
        }

        public boolean hasRung() {
            return rung != null;
        }

        public CoreMode getRung() {
            return rung == null ? null : rung;
        }

        public static PlayEvent read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "at_ms", "PlayEvent");
            Object $raw1 = Json.at($fields, "attempt");
            Object $raw2 = Json.at($fields, "failure");
            Object $raw3 = Json.at($fields, "first_frame_ms");
            Object $raw4 = Json.at($fields, "host");
            Object $raw5 = Json.need($fields, "name", "PlayEvent");
            Object $raw6 = Json.at($fields, "rung");
            return new PlayEvent(
                    Json.asLong($raw0),
                    $raw1 == null ? null : Json.asText($raw1),
                    $raw2 == null ? null : ErrorClass.read($raw2),
                    $raw3 == null ? null : Long.valueOf(Json.asLong($raw3)),
                    $raw4 == null ? null : Json.asText($raw4),
                    PlayEventName.read($raw5),
                    $raw6 == null ? null : CoreMode.read($raw6));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "at_ms");
            $out.append(atMs);
            if (attempt != null) {
                $count = Json.key($out, $count, "attempt");
                Json.text($out, attempt);
            }
            if (failure != null) {
                $count = Json.key($out, $count, "failure");
                failure.write($out);
            }
            if (firstFrameMs != null) {
                $count = Json.key($out, $count, "first_frame_ms");
                $out.append(firstFrameMs);
            }
            if (host != null) {
                $count = Json.key($out, $count, "host");
                Json.text($out, host);
            }
            $count = Json.key($out, $count, "name");
            name.write($out);
            if (rung != null) {
                $count = Json.key($out, $count, "rung");
                rung.write($out);
            }
            $out.append('}');
        }
    }

    /** What a play report says happened. */
    public static final class PlayEventName implements Wire {
        public static final PlayEventName FIRST_FRAME = new PlayEventName("first_frame");
        public static final PlayEventName REBUFFER = new PlayEventName("rebuffer");
        public static final PlayEventName FAILURE = new PlayEventName("failure");
        public static final PlayEventName ENDED = new PlayEventName("ended");

        private final String value;

        public PlayEventName(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static PlayEventName read(Object $json) {
            return new PlayEventName(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof PlayEventName && (((PlayEventName) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** `/v1/play/{id}/events`. */
    public static final class PlayEvents implements Wire {
        private final List<PlayEvent> events;

        public PlayEvents(List<PlayEvent> events) {
            this.events = events;
        }

        public List<PlayEvent> getEventsList() {
            return events == null ? Collections.<PlayEvent>emptyList() : events;
        }

        public int getEventsCount() {
            return events == null ? 0 : events.size();
        }

        public PlayEvent getEvents(int $index) {
            return events.get($index);
        }

        public static PlayEvents read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "events", "PlayEvents");
            return new PlayEvents(
                    Json.list($raw0, $x0 -> PlayEvent.read($x0)));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "events");
            $out.append('[');
            for (int $i0 = 0; $i0 < events.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                events.get($i0).write($out);
            }
            $out.append(']');
            $out.append('}');
        }
    }

    /** Ask to play a title. */
    public static final class PlayRequest implements Wire {
        private final EpisodeRef episode;
        private final TitleKey title;

        public PlayRequest(EpisodeRef episode, TitleKey title) {
            this.episode = episode;
            this.title = title;
        }

        public boolean hasEpisode() {
            return episode != null;
        }

        public EpisodeRef getEpisode() {
            return episode == null ? null : episode;
        }

        public TitleKey getTitle() {
            return title;
        }

        public static PlayRequest read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "episode");
            Object $raw1 = Json.need($fields, "title", "PlayRequest");
            return new PlayRequest(
                    $raw0 == null ? null : EpisodeRef.read($raw0),
                    TitleKey.read($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (episode != null) {
                $count = Json.key($out, $count, "episode");
                episode.write($out);
            }
            $count = Json.key($out, $count, "title");
            title.write($out);
            $out.append('}');
        }
    }

    /** Where the play session stands. */
    public static final class PlayState implements Wire {
        public static final PlayState RESOLVING = new PlayState("resolving");
        public static final PlayState STARTING = new PlayState("starting");
        public static final PlayState PLAYING = new PlayState("playing");
        public static final PlayState PAUSED = new PlayState("paused");
        public static final PlayState STALLED = new PlayState("stalled");
        public static final PlayState SWITCHING = new PlayState("switching");
        public static final PlayState ENDED = new PlayState("ended");
        public static final PlayState FAILED = new PlayState("failed");

        private final String value;

        public PlayState(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static PlayState read(Object $json) {
            return new PlayState(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof PlayState && (((PlayState) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** Play session state for screens. */
    public static final class PlayView implements Wire {
        private final String attempt;
        private final EpisodeRef episode;
        private final EpisodeRef nextEpisode;
        private final PlayState state;
        private final Boolean stillAsking;
        private final TitleKey title;
        private final List<TimeWindow> windows;

        public PlayView(String attempt, EpisodeRef episode, EpisodeRef nextEpisode, PlayState state, Boolean stillAsking, TitleKey title, List<TimeWindow> windows) {
            this.attempt = attempt;
            this.episode = episode;
            this.nextEpisode = nextEpisode;
            this.state = state;
            this.stillAsking = stillAsking;
            this.title = title;
            this.windows = windows;
        }

        public boolean hasAttempt() {
            return attempt != null;
        }

        public String getAttempt() {
            return attempt == null ? "" : attempt;
        }

        public boolean hasEpisode() {
            return episode != null;
        }

        public EpisodeRef getEpisode() {
            return episode == null ? null : episode;
        }

        /** Absent on the last episode, so no screen offers one that does not exist. */
        public boolean hasNextEpisode() {
            return nextEpisode != null;
        }

        public EpisodeRef getNextEpisode() {
            return nextEpisode == null ? null : nextEpisode;
        }

        public PlayState getState() {
            return state;
        }

        /** Several episodes started by themselves with nobody touching anything, so the next one waits until the viewer says they are still there. */
        public boolean hasStillAsking() {
            return stillAsking != null;
        }

        public boolean getStillAsking() {
            return stillAsking != null && stillAsking;
        }

        public TitleKey getTitle() {
            return title;
        }

        public List<TimeWindow> getWindowsList() {
            return windows == null ? Collections.<TimeWindow>emptyList() : windows;
        }

        public int getWindowsCount() {
            return windows == null ? 0 : windows.size();
        }

        public TimeWindow getWindows(int $index) {
            return windows.get($index);
        }

        public static PlayView read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "attempt");
            Object $raw1 = Json.at($fields, "episode");
            Object $raw2 = Json.at($fields, "next_episode");
            Object $raw3 = Json.need($fields, "state", "PlayView");
            Object $raw4 = Json.at($fields, "still_asking");
            Object $raw5 = Json.need($fields, "title", "PlayView");
            Object $raw6 = Json.at($fields, "windows");
            return new PlayView(
                    $raw0 == null ? null : Json.asText($raw0),
                    $raw1 == null ? null : EpisodeRef.read($raw1),
                    $raw2 == null ? null : EpisodeRef.read($raw2),
                    PlayState.read($raw3),
                    $raw4 == null ? null : Boolean.valueOf(Json.asBool($raw4)),
                    TitleKey.read($raw5),
                    $raw6 == null ? null : Json.list($raw6, $x0 -> TimeWindow.read($x0)));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (attempt != null) {
                $count = Json.key($out, $count, "attempt");
                Json.text($out, attempt);
            }
            if (episode != null) {
                $count = Json.key($out, $count, "episode");
                episode.write($out);
            }
            if (nextEpisode != null) {
                $count = Json.key($out, $count, "next_episode");
                nextEpisode.write($out);
            }
            $count = Json.key($out, $count, "state");
            state.write($out);
            if (stillAsking != null) {
                $count = Json.key($out, $count, "still_asking");
                $out.append(stillAsking);
            }
            $count = Json.key($out, $count, "title");
            title.write($out);
            if (windows != null) {
                $count = Json.key($out, $count, "windows");
                $out.append('[');
                for (int $i0 = 0; $i0 < windows.size(); $i0++) {
                    if ($i0 > 0) {
                        $out.append(',');
                    }
                    windows.get($i0).write($out);
                }
                $out.append(']');
            }
            $out.append('}');
        }
    }

    /** One profile. */
    public static final class Profile implements Wire {
        private final String avatar;
        private final boolean hasPin;
        private final String id;
        private final boolean kids;
        private final String language;
        private final boolean main;
        private final String name;

        public Profile(String avatar, boolean hasPin, String id, boolean kids, String language, boolean main, String name) {
            this.avatar = avatar;
            this.hasPin = hasPin;
            this.id = id;
            this.kids = kids;
            this.language = language;
            this.main = main;
            this.name = name;
        }

        public boolean hasAvatar() {
            return avatar != null;
        }

        public String getAvatar() {
            return avatar == null ? "" : avatar;
        }

        public boolean getHasPin() {
            return hasPin;
        }

        public String getId() {
            return id;
        }

        public boolean getKids() {
            return kids;
        }

        /** A language code from `/v1/languages`, absent until one is chosen. */
        public boolean hasLanguage() {
            return language != null;
        }

        public String getLanguage() {
            return language == null ? "" : language;
        }

        public boolean getMain() {
            return main;
        }

        public String getName() {
            return name;
        }

        public static Profile read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "avatar");
            Object $raw1 = Json.need($fields, "has_pin", "Profile");
            Object $raw2 = Json.need($fields, "id", "Profile");
            Object $raw3 = Json.need($fields, "kids", "Profile");
            Object $raw4 = Json.at($fields, "language");
            Object $raw5 = Json.need($fields, "main", "Profile");
            Object $raw6 = Json.need($fields, "name", "Profile");
            return new Profile(
                    $raw0 == null ? null : Json.asText($raw0),
                    Json.asBool($raw1),
                    Json.asText($raw2),
                    Json.asBool($raw3),
                    $raw4 == null ? null : Json.asText($raw4),
                    Json.asBool($raw5),
                    Json.asText($raw6));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (avatar != null) {
                $count = Json.key($out, $count, "avatar");
                Json.text($out, avatar);
            }
            $count = Json.key($out, $count, "has_pin");
            $out.append(hasPin);
            $count = Json.key($out, $count, "id");
            Json.text($out, id);
            $count = Json.key($out, $count, "kids");
            $out.append(kids);
            if (language != null) {
                $count = Json.key($out, $count, "language");
                Json.text($out, language);
            }
            $count = Json.key($out, $count, "main");
            $out.append(main);
            $count = Json.key($out, $count, "name");
            Json.text($out, name);
            $out.append('}');
        }
    }

    public static final class ProfileCreate implements Wire {
        private final Boolean kids;
        private final String name;

        public ProfileCreate(Boolean kids, String name) {
            this.kids = kids;
            this.name = name;
        }

        public boolean hasKids() {
            return kids != null;
        }

        public boolean getKids() {
            return kids != null && kids;
        }

        public String getName() {
            return name;
        }

        public static ProfileCreate read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "kids");
            Object $raw1 = Json.need($fields, "name", "ProfileCreate");
            return new ProfileCreate(
                    $raw0 == null ? null : Boolean.valueOf(Json.asBool($raw0)),
                    Json.asText($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (kids != null) {
                $count = Json.key($out, $count, "kids");
                $out.append(kids);
            }
            $count = Json.key($out, $count, "name");
            Json.text($out, name);
            $out.append('}');
        }
    }

    public static final class ProfileEnter implements Wire {
        private final String pin;

        public ProfileEnter(String pin) {
            this.pin = pin;
        }

        public boolean hasPin() {
            return pin != null;
        }

        public String getPin() {
            return pin == null ? "" : pin;
        }

        public static ProfileEnter read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "pin");
            return new ProfileEnter(
                    $raw0 == null ? null : Json.asText($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (pin != null) {
                $count = Json.key($out, $count, "pin");
                Json.text($out, pin);
            }
            $out.append('}');
        }
    }

    /** The profile answer names its home, so the core can map it to a target. */
    public static final class ProfileEntered implements Wire {
        private final ScreenName home;
        private final Profile profile;

        public ProfileEntered(ScreenName home, Profile profile) {
            this.home = home;
            this.profile = profile;
        }

        public ScreenName getHome() {
            return home;
        }

        public Profile getProfile() {
            return profile;
        }

        public static ProfileEntered read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "home", "ProfileEntered");
            Object $raw1 = Json.need($fields, "profile", "ProfileEntered");
            return new ProfileEntered(
                    ScreenName.read($raw0),
                    Profile.read($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "home");
            home.write($out);
            $count = Json.key($out, $count, "profile");
            profile.write($out);
            $out.append('}');
        }
    }

    /** Titles shown over the profile chooser before anyone has chosen a profile, at most three, in order. Chosen for the whole account: never above the most open tier among its profiles. */
    public static final class ProfileGate implements Wire {
        private final List<GateTitle> titles;
        private final long ttlS;

        public ProfileGate(List<GateTitle> titles, long ttlS) {
            this.titles = titles;
            this.ttlS = ttlS;
        }

        public List<GateTitle> getTitlesList() {
            return titles == null ? Collections.<GateTitle>emptyList() : titles;
        }

        public int getTitlesCount() {
            return titles == null ? 0 : titles.size();
        }

        public GateTitle getTitles(int $index) {
            return titles.get($index);
        }

        /** Seconds this answer stays good. */
        public long getTtlS() {
            return ttlS;
        }

        public static ProfileGate read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "titles", "ProfileGate");
            Object $raw1 = Json.need($fields, "ttl_s", "ProfileGate");
            return new ProfileGate(
                    Json.list($raw0, $x0 -> GateTitle.read($x0)),
                    Json.asLong($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "titles");
            $out.append('[');
            for (int $i0 = 0; $i0 < titles.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                titles.get($i0).write($out);
            }
            $out.append(']');
            $count = Json.key($out, $count, "ttl_s");
            $out.append(ttlS);
            $out.append('}');
        }
    }

    /** Needs a step up. Never reachable through a queued command. */
    public static final class ProfileLimits implements Wire {
        private final String ageRating;
        private final List<String> blockedTitles;

        public ProfileLimits(String ageRating, List<String> blockedTitles) {
            this.ageRating = ageRating;
            this.blockedTitles = blockedTitles;
        }

        public String getAgeRating() {
            return ageRating;
        }

        public List<String> getBlockedTitlesList() {
            return blockedTitles == null ? Collections.<String>emptyList() : blockedTitles;
        }

        public int getBlockedTitlesCount() {
            return blockedTitles == null ? 0 : blockedTitles.size();
        }

        public String getBlockedTitles(int $index) {
            return blockedTitles.get($index);
        }

        public static ProfileLimits read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "age_rating", "ProfileLimits");
            Object $raw1 = Json.need($fields, "blocked_titles", "ProfileLimits");
            return new ProfileLimits(
                    Json.asText($raw0),
                    Json.list($raw1, $x0 -> Json.asText($x0)));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "age_rating");
            Json.text($out, ageRating);
            $count = Json.key($out, $count, "blocked_titles");
            $out.append('[');
            for (int $i0 = 0; $i0 < blockedTitles.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                Json.text($out, blockedTitles.get($i0));
            }
            $out.append(']');
            $out.append('}');
        }
    }

    public static final class ProfileList implements Wire {
        private final List<Profile> profiles;

        public ProfileList(List<Profile> profiles) {
            this.profiles = profiles;
        }

        public List<Profile> getProfilesList() {
            return profiles == null ? Collections.<Profile>emptyList() : profiles;
        }

        public int getProfilesCount() {
            return profiles == null ? 0 : profiles.size();
        }

        public Profile getProfiles(int $index) {
            return profiles.get($index);
        }

        public static ProfileList read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "profiles", "ProfileList");
            return new ProfileList(
                    Json.list($raw0, $x0 -> Profile.read($x0)));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "profiles");
            $out.append('[');
            for (int $i0 = 0; $i0 < profiles.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                profiles.get($i0).write($out);
            }
            $out.append(']');
            $out.append('}');
        }
    }

    public static final class ProfileSettings implements Wire {
        private final String audioLanguage;
        private final boolean autoplayNext;
        private final boolean autoplayPreviews;
        private final String gender;
        private final String handle;
        private final String subtitleLanguage;
        private final Map<String, String> subtitleStyle;
        private final String uiLanguage;

        public ProfileSettings(String audioLanguage, boolean autoplayNext, boolean autoplayPreviews, String gender, String handle, String subtitleLanguage, Map<String, String> subtitleStyle, String uiLanguage) {
            this.audioLanguage = audioLanguage;
            this.autoplayNext = autoplayNext;
            this.autoplayPreviews = autoplayPreviews;
            this.gender = gender;
            this.handle = handle;
            this.subtitleLanguage = subtitleLanguage;
            this.subtitleStyle = subtitleStyle;
            this.uiLanguage = uiLanguage;
        }

        public boolean hasAudioLanguage() {
            return audioLanguage != null;
        }

        public String getAudioLanguage() {
            return audioLanguage == null ? "" : audioLanguage;
        }

        public boolean getAutoplayNext() {
            return autoplayNext;
        }

        public boolean getAutoplayPreviews() {
            return autoplayPreviews;
        }

        public boolean hasGender() {
            return gender != null;
        }

        public String getGender() {
            return gender == null ? "" : gender;
        }

        public boolean hasHandle() {
            return handle != null;
        }

        public String getHandle() {
            return handle == null ? "" : handle;
        }

        public boolean hasSubtitleLanguage() {
            return subtitleLanguage != null;
        }

        public String getSubtitleLanguage() {
            return subtitleLanguage == null ? "" : subtitleLanguage;
        }

        public Map<String, String> getSubtitleStyleMap() {
            return subtitleStyle == null ? Collections.<String, String>emptyMap() : subtitleStyle;
        }

        public int getSubtitleStyleCount() {
            return subtitleStyle == null ? 0 : subtitleStyle.size();
        }

        public boolean hasUiLanguage() {
            return uiLanguage != null;
        }

        public String getUiLanguage() {
            return uiLanguage == null ? "" : uiLanguage;
        }

        public static ProfileSettings read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "audio_language");
            Object $raw1 = Json.need($fields, "autoplay_next", "ProfileSettings");
            Object $raw2 = Json.need($fields, "autoplay_previews", "ProfileSettings");
            Object $raw3 = Json.at($fields, "gender");
            Object $raw4 = Json.at($fields, "handle");
            Object $raw5 = Json.at($fields, "subtitle_language");
            Object $raw6 = Json.at($fields, "subtitle_style");
            Object $raw7 = Json.at($fields, "ui_language");
            return new ProfileSettings(
                    $raw0 == null ? null : Json.asText($raw0),
                    Json.asBool($raw1),
                    Json.asBool($raw2),
                    $raw3 == null ? null : Json.asText($raw3),
                    $raw4 == null ? null : Json.asText($raw4),
                    $raw5 == null ? null : Json.asText($raw5),
                    $raw6 == null ? null : Json.map($raw6, $x0 -> Json.asText($x0)),
                    $raw7 == null ? null : Json.asText($raw7));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (audioLanguage != null) {
                $count = Json.key($out, $count, "audio_language");
                Json.text($out, audioLanguage);
            }
            $count = Json.key($out, $count, "autoplay_next");
            $out.append(autoplayNext);
            $count = Json.key($out, $count, "autoplay_previews");
            $out.append(autoplayPreviews);
            if (gender != null) {
                $count = Json.key($out, $count, "gender");
                Json.text($out, gender);
            }
            if (handle != null) {
                $count = Json.key($out, $count, "handle");
                Json.text($out, handle);
            }
            if (subtitleLanguage != null) {
                $count = Json.key($out, $count, "subtitle_language");
                Json.text($out, subtitleLanguage);
            }
            if (subtitleStyle != null) {
                $count = Json.key($out, $count, "subtitle_style");
                $out.append('{');
                int $n0 = 0;
                for (Map.Entry<String, String> $e0 : subtitleStyle.entrySet()) {
                    $n0 = Json.key($out, $n0, $e0.getKey());
                    Json.text($out, $e0.getValue());
                }
                $out.append('}');
            }
            if (uiLanguage != null) {
                $count = Json.key($out, $count, "ui_language");
                Json.text($out, uiLanguage);
            }
            $out.append('}');
        }
    }

    public static final class ProfileTransfer implements Wire {
        private final String toAccountEmail;

        public ProfileTransfer(String toAccountEmail) {
            this.toAccountEmail = toAccountEmail;
        }

        public String getToAccountEmail() {
            return toAccountEmail;
        }

        public static ProfileTransfer read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "to_account_email", "ProfileTransfer");
            return new ProfileTransfer(
                    Json.asText($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "to_account_email");
            Json.text($out, toAccountEmail);
            $out.append('}');
        }
    }

    public static final class ProfileUpdate implements Wire {
        private final String avatar;
        private final Boolean kids;
        private final String language;
        private final String name;

        public ProfileUpdate(String avatar, Boolean kids, String language, String name) {
            this.avatar = avatar;
            this.kids = kids;
            this.language = language;
            this.name = name;
        }

        public boolean hasAvatar() {
            return avatar != null;
        }

        public String getAvatar() {
            return avatar == null ? "" : avatar;
        }

        public boolean hasKids() {
            return kids != null;
        }

        public boolean getKids() {
            return kids != null && kids;
        }

        public boolean hasLanguage() {
            return language != null;
        }

        public String getLanguage() {
            return language == null ? "" : language;
        }

        public boolean hasName() {
            return name != null;
        }

        public String getName() {
            return name == null ? "" : name;
        }

        public static ProfileUpdate read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "avatar");
            Object $raw1 = Json.at($fields, "kids");
            Object $raw2 = Json.at($fields, "language");
            Object $raw3 = Json.at($fields, "name");
            return new ProfileUpdate(
                    $raw0 == null ? null : Json.asText($raw0),
                    $raw1 == null ? null : Boolean.valueOf(Json.asBool($raw1)),
                    $raw2 == null ? null : Json.asText($raw2),
                    $raw3 == null ? null : Json.asText($raw3));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (avatar != null) {
                $count = Json.key($out, $count, "avatar");
                Json.text($out, avatar);
            }
            if (kids != null) {
                $count = Json.key($out, $count, "kids");
                $out.append(kids);
            }
            if (language != null) {
                $count = Json.key($out, $count, "language");
                Json.text($out, language);
            }
            if (name != null) {
                $count = Json.key($out, $count, "name");
                Json.text($out, name);
            }
            $out.append('}');
        }
    }

    /** A position, key by title and episode. */
    public static final class ProgressState implements Wire {
        private final long changedAtMs;
        private final boolean completed;
        private final long durationS;
        private final EpisodeRef episode;
        private final boolean hidden;
        private final long positionS;
        private final TitleKey title;
        private final long version;

        public ProgressState(long changedAtMs, boolean completed, long durationS, EpisodeRef episode, boolean hidden, long positionS, TitleKey title, long version) {
            this.changedAtMs = changedAtMs;
            this.completed = completed;
            this.durationS = durationS;
            this.episode = episode;
            this.hidden = hidden;
            this.positionS = positionS;
            this.title = title;
            this.version = version;
        }

        public long getChangedAtMs() {
            return changedAtMs;
        }

        public boolean getCompleted() {
            return completed;
        }

        public long getDurationS() {
            return durationS;
        }

        public boolean hasEpisode() {
            return episode != null;
        }

        public EpisodeRef getEpisode() {
            return episode == null ? null : episode;
        }

        public boolean getHidden() {
            return hidden;
        }

        public long getPositionS() {
            return positionS;
        }

        public TitleKey getTitle() {
            return title;
        }

        public long getVersion() {
            return version;
        }

        public static ProgressState read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "changed_at_ms", "ProgressState");
            Object $raw1 = Json.need($fields, "completed", "ProgressState");
            Object $raw2 = Json.need($fields, "duration_s", "ProgressState");
            Object $raw3 = Json.at($fields, "episode");
            Object $raw4 = Json.need($fields, "hidden", "ProgressState");
            Object $raw5 = Json.need($fields, "position_s", "ProgressState");
            Object $raw6 = Json.need($fields, "title", "ProgressState");
            Object $raw7 = Json.need($fields, "version", "ProgressState");
            return new ProgressState(
                    Json.asLong($raw0),
                    Json.asBool($raw1),
                    Json.asLong($raw2),
                    $raw3 == null ? null : EpisodeRef.read($raw3),
                    Json.asBool($raw4),
                    Json.asLong($raw5),
                    TitleKey.read($raw6),
                    Json.asLong($raw7));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "changed_at_ms");
            $out.append(changedAtMs);
            $count = Json.key($out, $count, "completed");
            $out.append(completed);
            $count = Json.key($out, $count, "duration_s");
            $out.append(durationS);
            if (episode != null) {
                $count = Json.key($out, $count, "episode");
                episode.write($out);
            }
            $count = Json.key($out, $count, "hidden");
            $out.append(hidden);
            $count = Json.key($out, $count, "position_s");
            $out.append(positionS);
            $count = Json.key($out, $count, "title");
            title.write($out);
            $count = Json.key($out, $count, "version");
            $out.append(version);
            $out.append('}');
        }
    }

    /** Titles a visitor may look at before signing in, most watched first. A series also carries pictures of its first episodes, so a television can show the title as a slideshow. */
    public static final class PublicTitles implements Wire {
        private final List<TitleDetail> titles;

        public PublicTitles(List<TitleDetail> titles) {
            this.titles = titles;
        }

        public List<TitleDetail> getTitlesList() {
            return titles == null ? Collections.<TitleDetail>emptyList() : titles;
        }

        public int getTitlesCount() {
            return titles == null ? 0 : titles.size();
        }

        public TitleDetail getTitles(int $index) {
            return titles.get($index);
        }

        public static PublicTitles read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "titles", "PublicTitles");
            return new PublicTitles(
                    Json.list($raw0, $x0 -> TitleDetail.read($x0)));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "titles");
            $out.append('[');
            for (int $i0 = 0; $i0 < titles.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                titles.get($i0).write($out);
            }
            $out.append(']');
            $out.append('}');
        }
    }

    /** Rating words on the wire. Screens use the words like, love and dislike. */
    public static final class Rating implements Wire {
        public static final Rating DOWN = new Rating("down");
        public static final Rating UP = new Rating("up");
        public static final Rating LOVE = new Rating("love");

        private final String value;

        public Rating(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static Rating read(Object $json) {
            return new Rating(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof Rating && (((Rating) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** Rating state of a title, versioned apart from its list state. */
    public static final class RatingState implements Wire {
        private final long changedAtMs;
        private final Rating rating;
        private final long version;

        public RatingState(long changedAtMs, Rating rating, long version) {
            this.changedAtMs = changedAtMs;
            this.rating = rating;
            this.version = version;
        }

        public long getChangedAtMs() {
            return changedAtMs;
        }

        /** Absent when no rating is set. */
        public boolean hasRating() {
            return rating != null;
        }

        public Rating getRating() {
            return rating == null ? null : rating;
        }

        public long getVersion() {
            return version;
        }

        public static RatingState read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "changed_at_ms", "RatingState");
            Object $raw1 = Json.at($fields, "rating");
            Object $raw2 = Json.need($fields, "version", "RatingState");
            return new RatingState(
                    Json.asLong($raw0),
                    $raw1 == null ? null : Rating.read($raw1),
                    Json.asLong($raw2));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "changed_at_ms");
            $out.append(changedAtMs);
            if (rating != null) {
                $count = Json.key($out, $count, "rating");
                rating.write($out);
            }
            $count = Json.key($out, $count, "version");
            $out.append(version);
            $out.append('}');
        }
    }

    /** `/v1/ready`, for outside checks. */
    public static final class Ready implements Wire {
        private final String build;
        private final long dataAgeS;
        private final String dataHash;
        private final ServiceState finder;
        private final ServiceState index;

        public Ready(String build, long dataAgeS, String dataHash, ServiceState finder, ServiceState index) {
            this.build = build;
            this.dataAgeS = dataAgeS;
            this.dataHash = dataHash;
            this.finder = finder;
            this.index = index;
        }

        public String getBuild() {
            return build;
        }

        public long getDataAgeS() {
            return dataAgeS;
        }

        public String getDataHash() {
            return dataHash;
        }

        public ServiceState getFinder() {
            return finder;
        }

        public ServiceState getIndex() {
            return index;
        }

        public static Ready read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "build", "Ready");
            Object $raw1 = Json.need($fields, "data_age_s", "Ready");
            Object $raw2 = Json.need($fields, "data_hash", "Ready");
            Object $raw3 = Json.need($fields, "finder", "Ready");
            Object $raw4 = Json.need($fields, "index", "Ready");
            return new Ready(
                    Json.asText($raw0),
                    Json.asLong($raw1),
                    Json.asText($raw2),
                    ServiceState.read($raw3),
                    ServiceState.read($raw4));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "build");
            Json.text($out, build);
            $count = Json.key($out, $count, "data_age_s");
            $out.append(dataAgeS);
            $count = Json.key($out, $count, "data_hash");
            Json.text($out, dataHash);
            $count = Json.key($out, $count, "finder");
            finder.write($out);
            $count = Json.key($out, $count, "index");
            index.write($out);
            $out.append('}');
        }
    }

    public static final class RecoveryFinish implements Wire {
        private final String code;
        private final String email;
        private final String newPassword;

        public RecoveryFinish(String code, String email, String newPassword) {
            this.code = code;
            this.email = email;
            this.newPassword = newPassword;
        }

        public String getCode() {
            return code;
        }

        public String getEmail() {
            return email;
        }

        public String getNewPassword() {
            return newPassword;
        }

        public static RecoveryFinish read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "code", "RecoveryFinish");
            Object $raw1 = Json.need($fields, "email", "RecoveryFinish");
            Object $raw2 = Json.need($fields, "new_password", "RecoveryFinish");
            return new RecoveryFinish(
                    Json.asText($raw0),
                    Json.asText($raw1),
                    Json.asText($raw2));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "code");
            Json.text($out, code);
            $count = Json.key($out, $count, "email");
            Json.text($out, email);
            $count = Json.key($out, $count, "new_password");
            Json.text($out, newPassword);
            $out.append('}');
        }
    }

    public static final class RecoveryStart implements Wire {
        private final String email;

        public RecoveryStart(String email) {
            this.email = email;
        }

        public String getEmail() {
            return email;
        }

        public static RecoveryStart read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "email", "RecoveryStart");
            return new RecoveryStart(
                    Json.asText($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "email");
            Json.text($out, email);
            $out.append('}');
        }
    }

    public static final class RefreshRequest implements Wire {
        private final String refreshToken;

        public RefreshRequest(String refreshToken) {
            this.refreshToken = refreshToken;
        }

        public String getRefreshToken() {
            return refreshToken;
        }

        public static RefreshRequest read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "refresh_token", "RefreshRequest");
            return new RefreshRequest(
                    Json.asText($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "refresh_token");
            Json.text($out, refreshToken);
            $out.append('}');
        }
    }

    /** Events after which a screen should be read again. */
    public static final class RefreshTrigger implements Wire {
        public static final RefreshTrigger FOCUS = new RefreshTrigger("focus");
        public static final RefreshTrigger SIGN_IN = new RefreshTrigger("sign_in");
        public static final RefreshTrigger PROFILE_CHANGE = new RefreshTrigger("profile_change");
        public static final RefreshTrigger LIST_CHANGE = new RefreshTrigger("list_change");
        public static final RefreshTrigger HISTORY_CHANGE = new RefreshTrigger("history_change");

        private final String value;

        public RefreshTrigger(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static RefreshTrigger read(Object $json) {
            return new RefreshTrigger(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof RefreshTrigger && (((RefreshTrigger) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** `/v1/reports`. */
    public static final class Report implements Wire {
        private final String build;
        private final String code;
        private final String detail;

        public Report(String build, String code, String detail) {
            this.build = build;
            this.code = code;
            this.detail = detail;
        }

        public String getBuild() {
            return build;
        }

        public String getCode() {
            return code;
        }

        public boolean hasDetail() {
            return detail != null;
        }

        public String getDetail() {
            return detail == null ? "" : detail;
        }

        public static Report read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "build", "Report");
            Object $raw1 = Json.need($fields, "code", "Report");
            Object $raw2 = Json.at($fields, "detail");
            return new Report(
                    Json.asText($raw0),
                    Json.asText($raw1),
                    $raw2 == null ? null : Json.asText($raw2));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "build");
            Json.text($out, build);
            $count = Json.key($out, $count, "code");
            Json.text($out, code);
            if (detail != null) {
                $count = Json.key($out, $count, "detail");
                Json.text($out, detail);
            }
            $out.append('}');
        }
    }

    /** Where play picks up: the episode of a series, and 0 for one not started yet. */
    public static final class Resume implements Wire {
        private final EpisodeRef episode;
        private final long positionS;

        public Resume(EpisodeRef episode, long positionS) {
            this.episode = episode;
            this.positionS = positionS;
        }

        public boolean hasEpisode() {
            return episode != null;
        }

        public EpisodeRef getEpisode() {
            return episode == null ? null : episode;
        }

        public long getPositionS() {
            return positionS;
        }

        public static Resume read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "episode");
            Object $raw1 = Json.need($fields, "position_s", "Resume");
            return new Resume(
                    $raw0 == null ? null : EpisodeRef.read($raw0),
                    Json.asLong($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (episode != null) {
                $count = Json.key($out, $count, "episode");
                episode.write($out);
            }
            $count = Json.key($out, $count, "position_s");
            $out.append(positionS);
            $out.append('}');
        }
    }

    /** A staged rollout. The bucket comes from the household, so a home moves together. */
    public static final class Rollout implements Wire {
        private final String name;
        private final long percent;

        public Rollout(String name, long percent) {
            this.name = name;
            this.percent = percent;
        }

        public String getName() {
            return name;
        }

        public long getPercent() {
            return percent;
        }

        public static Rollout read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "name", "Rollout");
            Object $raw1 = Json.need($fields, "percent", "Rollout");
            return new Rollout(
                    Json.asText($raw0),
                    Json.asLong($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "name");
            Json.text($out, name);
            $count = Json.key($out, $count, "percent");
            $out.append(percent);
            $out.append('}');
        }
    }

    /** What an address opens. */
    public static final class RouteTarget implements Wire {
        private final String code;
        private final EpisodeRef episode;
        private final String genre;
        private final String next;
        private final ScreenName screen;
        private final Target target;
        private final TitleKey title;

        public RouteTarget(String code, EpisodeRef episode, String genre, String next, ScreenName screen, Target target, TitleKey title) {
            this.code = code;
            this.episode = episode;
            this.genre = genre;
            this.next = next;
            this.screen = screen;
            this.target = target;
            this.title = title;
        }

        /** The code an approve address carried, as it stood in the address. */
        public boolean hasCode() {
            return code != null;
        }

        public String getCode() {
            return code == null ? "" : code;
        }

        public boolean hasEpisode() {
            return episode != null;
        }

        public EpisodeRef getEpisode() {
            return episode == null ? null : episode;
        }

        /** The genre, for the genre screen. */
        public boolean hasGenre() {
            return genre != null;
        }

        public String getGenre() {
            return genre == null ? "" : genre;
        }

        /** A safe address to return to after sign in. */
        public boolean hasNext() {
            return next != null;
        }

        public String getNext() {
            return next == null ? "" : next;
        }

        public boolean hasScreen() {
            return screen != null;
        }

        public ScreenName getScreen() {
            return screen == null ? null : screen;
        }

        public Target getTarget() {
            return target;
        }

        public boolean hasTitle() {
            return title != null;
        }

        public TitleKey getTitle() {
            return title == null ? null : title;
        }

        public static RouteTarget read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "code");
            Object $raw1 = Json.at($fields, "episode");
            Object $raw2 = Json.at($fields, "genre");
            Object $raw3 = Json.at($fields, "next");
            Object $raw4 = Json.at($fields, "screen");
            Object $raw5 = Json.need($fields, "target", "RouteTarget");
            Object $raw6 = Json.at($fields, "title");
            return new RouteTarget(
                    $raw0 == null ? null : Json.asText($raw0),
                    $raw1 == null ? null : EpisodeRef.read($raw1),
                    $raw2 == null ? null : Json.asText($raw2),
                    $raw3 == null ? null : Json.asText($raw3),
                    $raw4 == null ? null : ScreenName.read($raw4),
                    Target.read($raw5),
                    $raw6 == null ? null : TitleKey.read($raw6));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (code != null) {
                $count = Json.key($out, $count, "code");
                Json.text($out, code);
            }
            if (episode != null) {
                $count = Json.key($out, $count, "episode");
                episode.write($out);
            }
            if (genre != null) {
                $count = Json.key($out, $count, "genre");
                Json.text($out, genre);
            }
            if (next != null) {
                $count = Json.key($out, $count, "next");
                Json.text($out, next);
            }
            if (screen != null) {
                $count = Json.key($out, $count, "screen");
                screen.write($out);
            }
            $count = Json.key($out, $count, "target");
            target.write($out);
            if (title != null) {
                $count = Json.key($out, $count, "title");
                title.write($out);
            }
            $out.append('}');
        }
    }

    /** A built page. */
    public static final class Screen implements Wire {
        private final String cursor;
        private final ScreenName name;
        private final List<RefreshTrigger> refreshOn;
        private final List<Section> sections;
        private final long ttlS;

        public Screen(String cursor, ScreenName name, List<RefreshTrigger> refreshOn, List<Section> sections, long ttlS) {
            this.cursor = cursor;
            this.name = name;
            this.refreshOn = refreshOn;
            this.sections = sections;
            this.ttlS = ttlS;
        }

        /** Reads more sections through `/v1/more/{cursor}`. */
        public boolean hasCursor() {
            return cursor != null;
        }

        public String getCursor() {
            return cursor == null ? "" : cursor;
        }

        public ScreenName getName() {
            return name;
        }

        public List<RefreshTrigger> getRefreshOnList() {
            return refreshOn == null ? Collections.<RefreshTrigger>emptyList() : refreshOn;
        }

        public int getRefreshOnCount() {
            return refreshOn == null ? 0 : refreshOn.size();
        }

        public RefreshTrigger getRefreshOn(int $index) {
            return refreshOn.get($index);
        }

        public List<Section> getSectionsList() {
            return sections == null ? Collections.<Section>emptyList() : sections;
        }

        public int getSectionsCount() {
            return sections == null ? 0 : sections.size();
        }

        public Section getSections(int $index) {
            return sections.get($index);
        }

        /** Seconds this answer stays good. */
        public long getTtlS() {
            return ttlS;
        }

        public static Screen read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "cursor");
            Object $raw1 = Json.need($fields, "name", "Screen");
            Object $raw2 = Json.at($fields, "refresh_on");
            Object $raw3 = Json.need($fields, "sections", "Screen");
            Object $raw4 = Json.need($fields, "ttl_s", "Screen");
            return new Screen(
                    $raw0 == null ? null : Json.asText($raw0),
                    ScreenName.read($raw1),
                    $raw2 == null ? null : Json.list($raw2, $x0 -> RefreshTrigger.read($x0)),
                    Json.list($raw3, $x0 -> Section.read($x0)),
                    Json.asLong($raw4));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (cursor != null) {
                $count = Json.key($out, $count, "cursor");
                Json.text($out, cursor);
            }
            $count = Json.key($out, $count, "name");
            name.write($out);
            if (refreshOn != null) {
                $count = Json.key($out, $count, "refresh_on");
                $out.append('[');
                for (int $i0 = 0; $i0 < refreshOn.size(); $i0++) {
                    if ($i0 > 0) {
                        $out.append(',');
                    }
                    refreshOn.get($i0).write($out);
                }
                $out.append(']');
            }
            $count = Json.key($out, $count, "sections");
            $out.append('[');
            for (int $i0 = 0; $i0 < sections.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                sections.get($i0).write($out);
            }
            $out.append(']');
            $count = Json.key($out, $count, "ttl_s");
            $out.append(ttlS);
            $out.append('}');
        }
    }

    /** A page body handed to a screen. It stands in for the answer to an http effect whose body was going to the screen. */
    public static final class ScreenBody implements Wire {
        private final String body;
        private final String request;
        private final Boolean stale;

        public ScreenBody(String body, String request, Boolean stale) {
            this.body = body;
            this.request = request;
            this.stale = stale;
        }

        public String getBody() {
            return body;
        }

        /** Id of the http effect this answers. */
        public String getRequest() {
            return request;
        }

        /** True when this copy is past its ttl and was kept for when the network is gone. */
        public boolean hasStale() {
            return stale != null;
        }

        public boolean getStale() {
            return stale != null && stale;
        }

        public static ScreenBody read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "body", "ScreenBody");
            Object $raw1 = Json.need($fields, "request", "ScreenBody");
            Object $raw2 = Json.at($fields, "stale");
            return new ScreenBody(
                    Json.asText($raw0),
                    Json.asText($raw1),
                    $raw2 == null ? null : Boolean.valueOf(Json.asBool($raw2)));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "body");
            Json.text($out, body);
            $count = Json.key($out, $count, "request");
            Json.text($out, request);
            if (stale != null) {
                $count = Json.key($out, $count, "stale");
                $out.append(stale);
            }
            $out.append('}');
        }
    }

    /** The screens the server builds. */
    public static final class ScreenName implements Wire {
        public static final ScreenName HOME = new ScreenName("home");
        public static final ScreenName FILMS = new ScreenName("films");
        public static final ScreenName SERIES = new ScreenName("series");
        public static final ScreenName GENRE = new ScreenName("genre");
        public static final ScreenName NEW = new ScreenName("new");
        public static final ScreenName MY_LIST = new ScreenName("my-list");
        public static final ScreenName KIDS = new ScreenName("kids");
        public static final ScreenName SEARCH = new ScreenName("search");
        public static final ScreenName NOTIFICATIONS = new ScreenName("notifications");
        public static final ScreenName ACTIVITY = new ScreenName("activity");
        public static final ScreenName GAMES = new ScreenName("games");
        public static final ScreenName CLIPS = new ScreenName("clips");
        public static final ScreenName LANGUAGE = new ScreenName("language");
        public static final ScreenName MINE = new ScreenName("mine");
        public static final ScreenName HELP = new ScreenName("help");
        public static final ScreenName SETTINGS = new ScreenName("settings");

        private final String value;

        public ScreenName(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static ScreenName read(Object $json) {
            return new ScreenName(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof ScreenName && (((ScreenName) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** One season with its episodes. */
    public static final class Season implements Wire {
        private final List<Episode> episodes;
        private final String name;
        private final long number;

        public Season(List<Episode> episodes, String name, long number) {
            this.episodes = episodes;
            this.name = name;
            this.number = number;
        }

        public List<Episode> getEpisodesList() {
            return episodes == null ? Collections.<Episode>emptyList() : episodes;
        }

        public int getEpisodesCount() {
            return episodes == null ? 0 : episodes.size();
        }

        public Episode getEpisodes(int $index) {
            return episodes.get($index);
        }

        public boolean hasName() {
            return name != null;
        }

        public String getName() {
            return name == null ? "" : name;
        }

        public long getNumber() {
            return number;
        }

        public static Season read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "episodes", "Season");
            Object $raw1 = Json.at($fields, "name");
            Object $raw2 = Json.need($fields, "number", "Season");
            return new Season(
                    Json.list($raw0, $x0 -> Episode.read($x0)),
                    $raw1 == null ? null : Json.asText($raw1),
                    Json.asLong($raw2));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "episodes");
            $out.append('[');
            for (int $i0 = 0; $i0 < episodes.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                episodes.get($i0).write($out);
            }
            $out.append(']');
            if (name != null) {
                $count = Json.key($out, $count, "name");
                Json.text($out, name);
            }
            $count = Json.key($out, $count, "number");
            $out.append(number);
            $out.append('}');
        }
    }

    /** A season named on a title page. */
    public static final class SeasonSummary implements Wire {
        private final long episodeCount;
        private final long number;

        public SeasonSummary(long episodeCount, long number) {
            this.episodeCount = episodeCount;
            this.number = number;
        }

        public long getEpisodeCount() {
            return episodeCount;
        }

        public long getNumber() {
            return number;
        }

        public static SeasonSummary read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "episode_count", "SeasonSummary");
            Object $raw1 = Json.need($fields, "number", "SeasonSummary");
            return new SeasonSummary(
                    Json.asLong($raw0),
                    Json.asLong($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "episode_count");
            $out.append(episodeCount);
            $count = Json.key($out, $count, "number");
            $out.append(number);
            $out.append('}');
        }
    }

    /** Secret operations. */
    public static final class SecretOp implements Wire {
        public static final SecretOp GET = new SecretOp("get");
        public static final SecretOp PUT = new SecretOp("put");
        public static final SecretOp DELETE = new SecretOp("delete");

        private final String value;

        public SecretOp(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static SecretOp read(Object $json) {
            return new SecretOp(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof SecretOp && (((SecretOp) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** One row, hero, grid or notice of a screen. */
    public static final class Section implements Wire {
        private final String cursor;
        private final String id;
        private final SectionKind kind;
        private final Notice notice;
        private final List<Tile> tiles;
        private final String title;

        public Section(String cursor, String id, SectionKind kind, Notice notice, List<Tile> tiles, String title) {
            this.cursor = cursor;
            this.id = id;
            this.kind = kind;
            this.notice = notice;
            this.tiles = tiles;
            this.title = title;
        }

        /** Reads more of this section through `/v1/more/{cursor}`. */
        public boolean hasCursor() {
            return cursor != null;
        }

        public String getCursor() {
            return cursor == null ? "" : cursor;
        }

        public String getId() {
            return id;
        }

        public SectionKind getKind() {
            return kind;
        }

        public boolean hasNotice() {
            return notice != null;
        }

        public Notice getNotice() {
            return notice == null ? null : notice;
        }

        public List<Tile> getTilesList() {
            return tiles == null ? Collections.<Tile>emptyList() : tiles;
        }

        public int getTilesCount() {
            return tiles == null ? 0 : tiles.size();
        }

        public Tile getTiles(int $index) {
            return tiles.get($index);
        }

        /** Translated title. */
        public boolean hasTitle() {
            return title != null;
        }

        public String getTitle() {
            return title == null ? "" : title;
        }

        public static Section read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "cursor");
            Object $raw1 = Json.need($fields, "id", "Section");
            Object $raw2 = Json.need($fields, "kind", "Section");
            Object $raw3 = Json.at($fields, "notice");
            Object $raw4 = Json.at($fields, "tiles");
            Object $raw5 = Json.at($fields, "title");
            return new Section(
                    $raw0 == null ? null : Json.asText($raw0),
                    Json.asText($raw1),
                    SectionKind.read($raw2),
                    $raw3 == null ? null : Notice.read($raw3),
                    $raw4 == null ? null : Json.list($raw4, $x0 -> Tile.read($x0)),
                    $raw5 == null ? null : Json.asText($raw5));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (cursor != null) {
                $count = Json.key($out, $count, "cursor");
                Json.text($out, cursor);
            }
            $count = Json.key($out, $count, "id");
            Json.text($out, id);
            $count = Json.key($out, $count, "kind");
            kind.write($out);
            if (notice != null) {
                $count = Json.key($out, $count, "notice");
                notice.write($out);
            }
            if (tiles != null) {
                $count = Json.key($out, $count, "tiles");
                $out.append('[');
                for (int $i0 = 0; $i0 < tiles.size(); $i0++) {
                    if ($i0 > 0) {
                        $out.append(',');
                    }
                    tiles.get($i0).write($out);
                }
                $out.append(']');
            }
            if (title != null) {
                $count = Json.key($out, $count, "title");
                Json.text($out, title);
            }
            $out.append('}');
        }
    }

    /** Section kinds. A kind this build does not draw falls back to a plain row. */
    public static final class SectionKind implements Wire {
        public static final SectionKind HERO = new SectionKind("hero");
        public static final SectionKind ROW = new SectionKind("row");
        public static final SectionKind RANKED = new SectionKind("ranked");
        public static final SectionKind CONTINUE = new SectionKind("continue");
        public static final SectionKind GRID = new SectionKind("grid");
        public static final SectionKind NOTICE = new SectionKind("notice");

        private final String value;

        public SectionKind(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static SectionKind read(Object $json) {
            return new SectionKind(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof SectionKind && (((SectionKind) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** State of a part of the service. */
    public static final class ServiceState implements Wire {
        public static final ServiceState READY = new ServiceState("ready");
        public static final ServiceState DEGRADED = new ServiceState("degraded");
        public static final ServiceState DOWN = new ServiceState("down");

        private final String value;

        public ServiceState(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static ServiceState read(Object $json) {
            return new ServiceState(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof ServiceState && (((ServiceState) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** Where sign in stands. */
    public static final class SessionState implements Wire {
        public static final SessionState STARTING = new SessionState("starting");
        public static final SessionState SIGNED_OUT = new SessionState("signed_out");
        public static final SessionState SIGNED_IN = new SessionState("signed_in");

        private final String value;

        public SessionState(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static SessionState read(Object $json) {
            return new SessionState(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof SessionState && (((SessionState) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** Sign in state for screens. Tokens never appear here. */
    public static final class SessionView implements Wire {
        private final String email;
        private final EndReason ended;
        private final Long expiresAtMs;
        private final String profile;
        private final String scope;
        private final SessionState state;
        private final String userId;

        public SessionView(String email, EndReason ended, Long expiresAtMs, String profile, String scope, SessionState state, String userId) {
            this.email = email;
            this.ended = ended;
            this.expiresAtMs = expiresAtMs;
            this.profile = profile;
            this.scope = scope;
            this.state = state;
            this.userId = userId;
        }

        public boolean hasEmail() {
            return email != null;
        }

        public String getEmail() {
            return email == null ? "" : email;
        }

        public boolean hasEnded() {
            return ended != null;
        }

        public EndReason getEnded() {
            return ended == null ? null : ended;
        }

        public boolean hasExpiresAtMs() {
            return expiresAtMs != null;
        }

        public long getExpiresAtMs() {
            return expiresAtMs == null ? 0L : expiresAtMs;
        }

        public boolean hasProfile() {
            return profile != null;
        }

        public String getProfile() {
            return profile == null ? "" : profile;
        }

        /** Opaque scope of this account and profile, the same value http effects carry. */
        public boolean hasScope() {
            return scope != null;
        }

        public String getScope() {
            return scope == null ? "" : scope;
        }

        public SessionState getState() {
            return state;
        }

        public boolean hasUserId() {
            return userId != null;
        }

        public String getUserId() {
            return userId == null ? "" : userId;
        }

        public static SessionView read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "email");
            Object $raw1 = Json.at($fields, "ended");
            Object $raw2 = Json.at($fields, "expires_at_ms");
            Object $raw3 = Json.at($fields, "profile");
            Object $raw4 = Json.at($fields, "scope");
            Object $raw5 = Json.need($fields, "state", "SessionView");
            Object $raw6 = Json.at($fields, "user_id");
            return new SessionView(
                    $raw0 == null ? null : Json.asText($raw0),
                    $raw1 == null ? null : EndReason.read($raw1),
                    $raw2 == null ? null : Long.valueOf(Json.asLong($raw2)),
                    $raw3 == null ? null : Json.asText($raw3),
                    $raw4 == null ? null : Json.asText($raw4),
                    SessionState.read($raw5),
                    $raw6 == null ? null : Json.asText($raw6));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (email != null) {
                $count = Json.key($out, $count, "email");
                Json.text($out, email);
            }
            if (ended != null) {
                $count = Json.key($out, $count, "ended");
                ended.write($out);
            }
            if (expiresAtMs != null) {
                $count = Json.key($out, $count, "expires_at_ms");
                $out.append(expiresAtMs);
            }
            if (profile != null) {
                $count = Json.key($out, $count, "profile");
                Json.text($out, profile);
            }
            if (scope != null) {
                $count = Json.key($out, $count, "scope");
                Json.text($out, scope);
            }
            $count = Json.key($out, $count, "state");
            state.write($out);
            if (userId != null) {
                $count = Json.key($out, $count, "user_id");
                Json.text($out, userId);
            }
            $out.append('}');
        }
    }

    /** Settings a queued command may change. Age rating and blocked titles are absent on purpose: they need a step up, so a queued write can never touch them. */
    public static final class SettingKey implements Wire {
        public static final SettingKey AUTOPLAY_NEXT = new SettingKey("autoplay_next");
        public static final SettingKey AUTOPLAY_PREVIEWS = new SettingKey("autoplay_previews");
        public static final SettingKey AUDIO_LANGUAGE = new SettingKey("audio_language");
        public static final SettingKey SUBTITLE_LANGUAGE = new SettingKey("subtitle_language");
        public static final SettingKey SUBTITLE_STYLE = new SettingKey("subtitle_style");
        public static final SettingKey UI_LANGUAGE = new SettingKey("ui_language");
        public static final SettingKey HANDLE = new SettingKey("handle");
        public static final SettingKey GENDER = new SettingKey("gender");

        private final String value;

        public SettingKey(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static SettingKey read(Object $json) {
            return new SettingKey(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof SettingKey && (((SettingKey) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    public static final class SignOut implements Wire {
        private final boolean everywhere;

        public SignOut(boolean everywhere) {
            this.everywhere = everywhere;
        }

        /** True ends every session of this account. */
        public boolean getEverywhere() {
            return everywhere;
        }

        public static SignOut read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "everywhere", "SignOut");
            return new SignOut(
                    Json.asBool($raw0));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "everywhere");
            $out.append(everywhere);
            $out.append('}');
        }
    }

    public static final class SignUp implements Wire {
        private final String email;
        private final String firstName;
        private final String lastName;
        private final String password;

        public SignUp(String email, String firstName, String lastName, String password) {
            this.email = email;
            this.firstName = firstName;
            this.lastName = lastName;
            this.password = password;
        }

        public String getEmail() {
            return email;
        }

        public String getFirstName() {
            return firstName;
        }

        public String getLastName() {
            return lastName;
        }

        public String getPassword() {
            return password;
        }

        public static SignUp read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "email", "SignUp");
            Object $raw1 = Json.need($fields, "first_name", "SignUp");
            Object $raw2 = Json.need($fields, "last_name", "SignUp");
            Object $raw3 = Json.need($fields, "password", "SignUp");
            return new SignUp(
                    Json.asText($raw0),
                    Json.asText($raw1),
                    Json.asText($raw2),
                    Json.asText($raw3));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "email");
            Json.text($out, email);
            $count = Json.key($out, $count, "first_name");
            Json.text($out, firstName);
            $count = Json.key($out, $count, "last_name");
            Json.text($out, lastName);
            $count = Json.key($out, $count, "password");
            Json.text($out, password);
            $out.append('}');
        }
    }

    /** Opaque tokens. `expires_in` is seconds, so no device clock decides anything. */
    public static final class SignedIn implements Wire {
        private final String accessToken;
        private final String email;
        private final long expiresIn;
        private final String refreshToken;
        private final String userId;

        public SignedIn(String accessToken, String email, long expiresIn, String refreshToken, String userId) {
            this.accessToken = accessToken;
            this.email = email;
            this.expiresIn = expiresIn;
            this.refreshToken = refreshToken;
            this.userId = userId;
        }

        public String getAccessToken() {
            return accessToken;
        }

        public String getEmail() {
            return email;
        }

        public long getExpiresIn() {
            return expiresIn;
        }

        public String getRefreshToken() {
            return refreshToken;
        }

        public String getUserId() {
            return userId;
        }

        public static SignedIn read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "access_token", "SignedIn");
            Object $raw1 = Json.need($fields, "email", "SignedIn");
            Object $raw2 = Json.need($fields, "expires_in", "SignedIn");
            Object $raw3 = Json.need($fields, "refresh_token", "SignedIn");
            Object $raw4 = Json.need($fields, "user_id", "SignedIn");
            return new SignedIn(
                    Json.asText($raw0),
                    Json.asText($raw1),
                    Json.asLong($raw2),
                    Json.asText($raw3),
                    Json.asText($raw4));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "access_token");
            Json.text($out, accessToken);
            $count = Json.key($out, $count, "email");
            Json.text($out, email);
            $count = Json.key($out, $count, "expires_in");
            $out.append(expiresIn);
            $count = Json.key($out, $count, "refresh_token");
            Json.text($out, refreshToken);
            $count = Json.key($out, $count, "user_id");
            Json.text($out, userId);
            $out.append('}');
        }
    }

    /** Store operations. Only the offline layer uses store. */
    public static final class StoreOp implements Wire {
        public static final StoreOp GET = new StoreOp("get");
        public static final StoreOp PUT = new StoreOp("put");
        public static final StoreOp DELETE = new StoreOp("delete");
        public static final StoreOp LIST = new StoreOp("list");

        private final String value;

        public StoreOp(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static StoreOp read(Object $json) {
            return new StoreOp(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof StoreOp && (((StoreOp) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** `/v1/strings/{lang}`. Text holds the placeholders below. */
    public static final class Strings implements Wire {
        private final TextDirection dir;
        private final String lang;
        private final Map<String, String> texts;

        public Strings(TextDirection dir, String lang, Map<String, String> texts) {
            this.dir = dir;
            this.lang = lang;
            this.texts = texts;
        }

        /** Which way this language runs. Absent from older servers, which only spoke left to right. */
        public boolean hasDir() {
            return dir != null;
        }

        public TextDirection getDir() {
            return dir == null ? null : dir;
        }

        public String getLang() {
            return lang;
        }

        public Map<String, String> getTextsMap() {
            return texts == null ? Collections.<String, String>emptyMap() : texts;
        }

        public int getTextsCount() {
            return texts == null ? 0 : texts.size();
        }

        public static Strings read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "dir");
            Object $raw1 = Json.need($fields, "lang", "Strings");
            Object $raw2 = Json.need($fields, "texts", "Strings");
            return new Strings(
                    $raw0 == null ? null : TextDirection.read($raw0),
                    Json.asText($raw1),
                    Json.map($raw2, $x0 -> Json.asText($x0)));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (dir != null) {
                $count = Json.key($out, $count, "dir");
                dir.write($out);
            }
            $count = Json.key($out, $count, "lang");
            Json.text($out, lang);
            $count = Json.key($out, $count, "texts");
            $out.append('{');
            int $n0 = 0;
            for (Map.Entry<String, String> $e0 : texts.entrySet()) {
                $n0 = Json.key($out, $n0, $e0.getKey());
                Json.text($out, $e0.getValue());
            }
            $out.append('}');
            $out.append('}');
        }
    }

    /** `/v1/subtitles/{file}`. The core checks `last_end_ms` against the driver's duration. */
    public static final class SubtitleBody implements Wire {
        private final List<Cue> cues;
        private final long lastEndMs;

        public SubtitleBody(List<Cue> cues, long lastEndMs) {
            this.cues = cues;
            this.lastEndMs = lastEndMs;
        }

        public List<Cue> getCuesList() {
            return cues == null ? Collections.<Cue>emptyList() : cues;
        }

        public int getCuesCount() {
            return cues == null ? 0 : cues.size();
        }

        public Cue getCues(int $index) {
            return cues.get($index);
        }

        public long getLastEndMs() {
            return lastEndMs;
        }

        public static SubtitleBody read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "cues", "SubtitleBody");
            Object $raw1 = Json.need($fields, "last_end_ms", "SubtitleBody");
            return new SubtitleBody(
                    Json.list($raw0, $x0 -> Cue.read($x0)),
                    Json.asLong($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "cues");
            $out.append('[');
            for (int $i0 = 0; $i0 < cues.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                cues.get($i0).write($out);
            }
            $out.append(']');
            $count = Json.key($out, $count, "last_end_ms");
            $out.append(lastEndMs);
            $out.append('}');
        }
    }

    /** One subtitle file of a language, in the order to try. */
    public static final class SubtitleFile implements Wire {
        private final Boolean hearingImpaired;
        private final String id;

        public SubtitleFile(Boolean hearingImpaired, String id) {
            this.hearingImpaired = hearingImpaired;
            this.id = id;
        }

        public boolean hasHearingImpaired() {
            return hearingImpaired != null;
        }

        public boolean getHearingImpaired() {
            return hearingImpaired != null && hearingImpaired;
        }

        public String getId() {
            return id;
        }

        public static SubtitleFile read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "hearing_impaired");
            Object $raw1 = Json.need($fields, "id", "SubtitleFile");
            return new SubtitleFile(
                    $raw0 == null ? null : Boolean.valueOf(Json.asBool($raw0)),
                    Json.asText($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (hearingImpaired != null) {
                $count = Json.key($out, $count, "hearing_impaired");
                $out.append(hearingImpaired);
            }
            $count = Json.key($out, $count, "id");
            Json.text($out, id);
            $out.append('}');
        }
    }

    /** One language. */
    public static final class SubtitleLanguage implements Wire {
        private final List<SubtitleFile> files;
        private final String label;
        private final String lang;

        public SubtitleLanguage(List<SubtitleFile> files, String label, String lang) {
            this.files = files;
            this.label = label;
            this.lang = lang;
        }

        public List<SubtitleFile> getFilesList() {
            return files == null ? Collections.<SubtitleFile>emptyList() : files;
        }

        public int getFilesCount() {
            return files == null ? 0 : files.size();
        }

        public SubtitleFile getFiles(int $index) {
            return files.get($index);
        }

        public String getLabel() {
            return label;
        }

        public String getLang() {
            return lang;
        }

        public static SubtitleLanguage read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "files", "SubtitleLanguage");
            Object $raw1 = Json.need($fields, "label", "SubtitleLanguage");
            Object $raw2 = Json.need($fields, "lang", "SubtitleLanguage");
            return new SubtitleLanguage(
                    Json.list($raw0, $x0 -> SubtitleFile.read($x0)),
                    Json.asText($raw1),
                    Json.asText($raw2));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "files");
            $out.append('[');
            for (int $i0 = 0; $i0 < files.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                files.get($i0).write($out);
            }
            $out.append(']');
            $count = Json.key($out, $count, "label");
            Json.text($out, label);
            $count = Json.key($out, $count, "lang");
            Json.text($out, lang);
            $out.append('}');
        }
    }

    /** `/v1/subtitles`. */
    public static final class SubtitleList implements Wire {
        private final List<SubtitleLanguage> languages;

        public SubtitleList(List<SubtitleLanguage> languages) {
            this.languages = languages;
        }

        public List<SubtitleLanguage> getLanguagesList() {
            return languages == null ? Collections.<SubtitleLanguage>emptyList() : languages;
        }

        public int getLanguagesCount() {
            return languages == null ? 0 : languages.size();
        }

        public SubtitleLanguage getLanguages(int $index) {
            return languages.get($index);
        }

        public static SubtitleList read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "languages", "SubtitleList");
            return new SubtitleList(
                    Json.list($raw0, $x0 -> SubtitleLanguage.read($x0)));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "languages");
            $out.append('[');
            for (int $i0 = 0; $i0 < languages.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                languages.get($i0).write($out);
            }
            $out.append(']');
            $out.append('}');
        }
    }

    /** Search suggestions. */
    public static final class Suggestions implements Wire {
        private final List<String> items;

        public Suggestions(List<String> items) {
            this.items = items;
        }

        public List<String> getItemsList() {
            return items == null ? Collections.<String>emptyList() : items;
        }

        public int getItemsCount() {
            return items == null ? 0 : items.size();
        }

        public String getItems(int $index) {
            return items.get($index);
        }

        public static Suggestions read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "items", "Suggestions");
            return new Suggestions(
                    Json.list($raw0, $x0 -> Json.asText($x0)));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "items");
            $out.append('[');
            for (int $i0 = 0; $i0 < items.size(); $i0++) {
                if ($i0 > 0) {
                    $out.append(',');
                }
                Json.text($out, items.get($i0));
            }
            $out.append(']');
            $out.append('}');
        }
    }

    /** A semantic target. Each device maps it to one of its own screens. */
    public static final class Target implements Wire {
        public static final Target START = new Target("start");
        public static final Target SIGN_IN = new Target("sign_in");
        public static final Target PROFILES = new Target("profiles");
        public static final Target HOME = new Target("home");
        public static final Target KIDS_HOME = new Target("kids_home");
        public static final Target SCREEN = new Target("screen");
        public static final Target TITLE = new Target("title");
        public static final Target WATCH = new Target("watch");
        public static final Target NOT_FOUND = new Target("not_found");
        public static final Target WAIT = new Target("wait");
        public static final Target UPGRADE = new Target("upgrade");
        public static final Target ACTIVATE = new Target("activate");

        private final String value;

        public Target(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static Target read(Object $json) {
            return new Target(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof Target && (((Target) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** Which way a language's lines run. The apps lay out, mirror and move focus by it; the player's timeline and the brand's art never turn. */
    public static final class TextDirection implements Wire {
        public static final TextDirection LTR = new TextDirection("ltr");
        public static final TextDirection RTL = new TextDirection("rtl");

        private final String value;

        public TextDirection(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static TextDirection read(Object $json) {
            return new TextDirection(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof TextDirection && (((TextDirection) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** One title in a section. */
    public static final class Tile implements Wire {
        private final List<Badge> badges;
        private final EpisodeRef episode;
        private final List<Image> images;
        private final String name;
        private final Long percent;
        private final List<String> tags;
        private final TitleKey title;

        public Tile(List<Badge> badges, EpisodeRef episode, List<Image> images, String name, Long percent, List<String> tags, TitleKey title) {
            this.badges = badges;
            this.episode = episode;
            this.images = images;
            this.name = name;
            this.percent = percent;
            this.tags = tags;
            this.title = title;
        }

        public List<Badge> getBadgesList() {
            return badges == null ? Collections.<Badge>emptyList() : badges;
        }

        public int getBadgesCount() {
            return badges == null ? 0 : badges.size();
        }

        public Badge getBadges(int $index) {
            return badges.get($index);
        }

        public boolean hasEpisode() {
            return episode != null;
        }

        public EpisodeRef getEpisode() {
            return episode == null ? null : episode;
        }

        public List<Image> getImagesList() {
            return images == null ? Collections.<Image>emptyList() : images;
        }

        public int getImagesCount() {
            return images == null ? 0 : images.size();
        }

        public Image getImages(int $index) {
            return images.get($index);
        }

        public String getName() {
            return name;
        }

        /** Whole percent watched. */
        public boolean hasPercent() {
            return percent != null;
        }

        public long getPercent() {
            return percent == null ? 0L : percent;
        }

        public List<String> getTagsList() {
            return tags == null ? Collections.<String>emptyList() : tags;
        }

        public int getTagsCount() {
            return tags == null ? 0 : tags.size();
        }

        public String getTags(int $index) {
            return tags.get($index);
        }

        public TitleKey getTitle() {
            return title;
        }

        public static Tile read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "badges");
            Object $raw1 = Json.at($fields, "episode");
            Object $raw2 = Json.at($fields, "images");
            Object $raw3 = Json.need($fields, "name", "Tile");
            Object $raw4 = Json.at($fields, "percent");
            Object $raw5 = Json.at($fields, "tags");
            Object $raw6 = Json.need($fields, "title", "Tile");
            return new Tile(
                    $raw0 == null ? null : Json.list($raw0, $x0 -> Badge.read($x0)),
                    $raw1 == null ? null : EpisodeRef.read($raw1),
                    $raw2 == null ? null : Json.list($raw2, $x0 -> Image.read($x0)),
                    Json.asText($raw3),
                    $raw4 == null ? null : Long.valueOf(Json.asLong($raw4)),
                    $raw5 == null ? null : Json.list($raw5, $x0 -> Json.asText($x0)),
                    TitleKey.read($raw6));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (badges != null) {
                $count = Json.key($out, $count, "badges");
                $out.append('[');
                for (int $i0 = 0; $i0 < badges.size(); $i0++) {
                    if ($i0 > 0) {
                        $out.append(',');
                    }
                    badges.get($i0).write($out);
                }
                $out.append(']');
            }
            if (episode != null) {
                $count = Json.key($out, $count, "episode");
                episode.write($out);
            }
            if (images != null) {
                $count = Json.key($out, $count, "images");
                $out.append('[');
                for (int $i0 = 0; $i0 < images.size(); $i0++) {
                    if ($i0 > 0) {
                        $out.append(',');
                    }
                    images.get($i0).write($out);
                }
                $out.append(']');
            }
            $count = Json.key($out, $count, "name");
            Json.text($out, name);
            if (percent != null) {
                $count = Json.key($out, $count, "percent");
                $out.append(percent);
            }
            if (tags != null) {
                $count = Json.key($out, $count, "tags");
                $out.append('[');
                for (int $i0 = 0; $i0 < tags.size(); $i0++) {
                    if ($i0 > 0) {
                        $out.append(',');
                    }
                    Json.text($out, tags.get($i0));
                }
                $out.append(']');
            }
            $count = Json.key($out, $count, "title");
            title.write($out);
            $out.append('}');
        }
    }

    /** A span of a video the host compares to player time. Nothing ticks across the boundary. */
    public static final class TimeWindow implements Wire {
        private final long endMs;
        private final WindowKind kind;
        private final Long seekToMs;
        private final long startMs;
        private final String text;

        public TimeWindow(long endMs, WindowKind kind, Long seekToMs, long startMs, String text) {
            this.endMs = endMs;
            this.kind = kind;
            this.seekToMs = seekToMs;
            this.startMs = startMs;
            this.text = text;
        }

        public long getEndMs() {
            return endMs;
        }

        public WindowKind getKind() {
            return kind;
        }

        /** Where a skip button moves to. */
        public boolean hasSeekToMs() {
            return seekToMs != null;
        }

        public long getSeekToMs() {
            return seekToMs == null ? 0L : seekToMs;
        }

        public long getStartMs() {
            return startMs;
        }

        public boolean hasText() {
            return text != null;
        }

        public String getText() {
            return text == null ? "" : text;
        }

        public static TimeWindow read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "end_ms", "TimeWindow");
            Object $raw1 = Json.need($fields, "kind", "TimeWindow");
            Object $raw2 = Json.at($fields, "seek_to_ms");
            Object $raw3 = Json.need($fields, "start_ms", "TimeWindow");
            Object $raw4 = Json.at($fields, "text");
            return new TimeWindow(
                    Json.asLong($raw0),
                    WindowKind.read($raw1),
                    $raw2 == null ? null : Long.valueOf(Json.asLong($raw2)),
                    Json.asLong($raw3),
                    $raw4 == null ? null : Json.asText($raw4));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "end_ms");
            $out.append(endMs);
            $count = Json.key($out, $count, "kind");
            kind.write($out);
            if (seekToMs != null) {
                $count = Json.key($out, $count, "seek_to_ms");
                $out.append(seekToMs);
            }
            $count = Json.key($out, $count, "start_ms");
            $out.append(startMs);
            if (text != null) {
                $count = Json.key($out, $count, "text");
                Json.text($out, text);
            }
            $out.append('}');
        }
    }

    /** Timer operations. A clear uses the id of the timer it clears. */
    public static final class TimerOp implements Wire {
        public static final TimerOp SET = new TimerOp("set");
        public static final TimerOp CLEAR = new TimerOp("clear");

        private final String value;

        public TimerOp(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static TimerOp read(Object $json) {
            return new TimerOp(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof TimerOp && (((TimerOp) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** Facts of one title. */
    public static final class TitleDetail implements Wire {
        private final AgeMark age;
        private final CalendarDay availableFrom;
        private final List<String> genres;
        private final List<Image> images;
        private final Boolean inList;
        private final List<Tile> moreLikeThis;
        private final String name;
        private final String overview;
        private final Rating rating;
        private final Boolean reminderSet;
        private final Resume resume;
        private final Long runtimeMin;
        private final List<SeasonSummary> seasons;
        private final TitleKey title;
        private final Long year;

        public TitleDetail(AgeMark age, CalendarDay availableFrom, List<String> genres, List<Image> images, Boolean inList, List<Tile> moreLikeThis, String name, String overview, Rating rating, Boolean reminderSet, Resume resume, Long runtimeMin, List<SeasonSummary> seasons, TitleKey title, Long year) {
            this.age = age;
            this.availableFrom = availableFrom;
            this.genres = genres;
            this.images = images;
            this.inList = inList;
            this.moreLikeThis = moreLikeThis;
            this.name = name;
            this.overview = overview;
            this.rating = rating;
            this.reminderSet = reminderSet;
            this.resume = resume;
            this.runtimeMin = runtimeMin;
            this.seasons = seasons;
            this.title = title;
            this.year = year;
        }

        /** Absent when the title has no rating in either system. */
        public boolean hasAge() {
            return age != null;
        }

        public AgeMark getAge() {
            return age == null ? null : age;
        }

        /** When this title becomes available to watch. Absent when already available. */
        public boolean hasAvailableFrom() {
            return availableFrom != null;
        }

        public CalendarDay getAvailableFrom() {
            return availableFrom == null ? null : availableFrom;
        }

        public List<String> getGenresList() {
            return genres == null ? Collections.<String>emptyList() : genres;
        }

        public int getGenresCount() {
            return genres == null ? 0 : genres.size();
        }

        public String getGenres(int $index) {
            return genres.get($index);
        }

        public List<Image> getImagesList() {
            return images == null ? Collections.<Image>emptyList() : images;
        }

        public int getImagesCount() {
            return images == null ? 0 : images.size();
        }

        public Image getImages(int $index) {
            return images.get($index);
        }

        /** True when this profile has saved the title. */
        public boolean hasInList() {
            return inList != null;
        }

        public boolean getInList() {
            return inList != null && inList;
        }

        /** Hidden when empty. */
        public List<Tile> getMoreLikeThisList() {
            return moreLikeThis == null ? Collections.<Tile>emptyList() : moreLikeThis;
        }

        public int getMoreLikeThisCount() {
            return moreLikeThis == null ? 0 : moreLikeThis.size();
        }

        public Tile getMoreLikeThis(int $index) {
            return moreLikeThis.get($index);
        }

        public String getName() {
            return name;
        }

        public boolean hasOverview() {
            return overview != null;
        }

        public String getOverview() {
            return overview == null ? "" : overview;
        }

        /** What this profile said about the title. */
        public boolean hasRating() {
            return rating != null;
        }

        public Rating getRating() {
            return rating == null ? null : rating;
        }

        /** True when this profile set a reminder for when this title arrives. */
        public boolean hasReminderSet() {
            return reminderSet != null;
        }

        public boolean getReminderSet() {
            return reminderSet != null && reminderSet;
        }

        /** Where this profile left off. Absent when there is nothing to go back to. */
        public boolean hasResume() {
            return resume != null;
        }

        public Resume getResume() {
            return resume == null ? null : resume;
        }

        public boolean hasRuntimeMin() {
            return runtimeMin != null;
        }

        public long getRuntimeMin() {
            return runtimeMin == null ? 0L : runtimeMin;
        }

        public List<SeasonSummary> getSeasonsList() {
            return seasons == null ? Collections.<SeasonSummary>emptyList() : seasons;
        }

        public int getSeasonsCount() {
            return seasons == null ? 0 : seasons.size();
        }

        public SeasonSummary getSeasons(int $index) {
            return seasons.get($index);
        }

        public TitleKey getTitle() {
            return title;
        }

        public boolean hasYear() {
            return year != null;
        }

        public long getYear() {
            return year == null ? 0L : year;
        }

        public static TitleDetail read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "age");
            Object $raw1 = Json.at($fields, "available_from");
            Object $raw2 = Json.at($fields, "genres");
            Object $raw3 = Json.at($fields, "images");
            Object $raw4 = Json.at($fields, "in_list");
            Object $raw5 = Json.at($fields, "more_like_this");
            Object $raw6 = Json.need($fields, "name", "TitleDetail");
            Object $raw7 = Json.at($fields, "overview");
            Object $raw8 = Json.at($fields, "rating");
            Object $raw9 = Json.at($fields, "reminder_set");
            Object $raw10 = Json.at($fields, "resume");
            Object $raw11 = Json.at($fields, "runtime_min");
            Object $raw12 = Json.at($fields, "seasons");
            Object $raw13 = Json.need($fields, "title", "TitleDetail");
            Object $raw14 = Json.at($fields, "year");
            return new TitleDetail(
                    $raw0 == null ? null : AgeMark.read($raw0),
                    $raw1 == null ? null : CalendarDay.read($raw1),
                    $raw2 == null ? null : Json.list($raw2, $x0 -> Json.asText($x0)),
                    $raw3 == null ? null : Json.list($raw3, $x0 -> Image.read($x0)),
                    $raw4 == null ? null : Boolean.valueOf(Json.asBool($raw4)),
                    $raw5 == null ? null : Json.list($raw5, $x0 -> Tile.read($x0)),
                    Json.asText($raw6),
                    $raw7 == null ? null : Json.asText($raw7),
                    $raw8 == null ? null : Rating.read($raw8),
                    $raw9 == null ? null : Boolean.valueOf(Json.asBool($raw9)),
                    $raw10 == null ? null : Resume.read($raw10),
                    $raw11 == null ? null : Long.valueOf(Json.asLong($raw11)),
                    $raw12 == null ? null : Json.list($raw12, $x0 -> SeasonSummary.read($x0)),
                    TitleKey.read($raw13),
                    $raw14 == null ? null : Long.valueOf(Json.asLong($raw14)));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (age != null) {
                $count = Json.key($out, $count, "age");
                age.write($out);
            }
            if (availableFrom != null) {
                $count = Json.key($out, $count, "available_from");
                availableFrom.write($out);
            }
            if (genres != null) {
                $count = Json.key($out, $count, "genres");
                $out.append('[');
                for (int $i0 = 0; $i0 < genres.size(); $i0++) {
                    if ($i0 > 0) {
                        $out.append(',');
                    }
                    Json.text($out, genres.get($i0));
                }
                $out.append(']');
            }
            if (images != null) {
                $count = Json.key($out, $count, "images");
                $out.append('[');
                for (int $i0 = 0; $i0 < images.size(); $i0++) {
                    if ($i0 > 0) {
                        $out.append(',');
                    }
                    images.get($i0).write($out);
                }
                $out.append(']');
            }
            if (inList != null) {
                $count = Json.key($out, $count, "in_list");
                $out.append(inList);
            }
            if (moreLikeThis != null) {
                $count = Json.key($out, $count, "more_like_this");
                $out.append('[');
                for (int $i0 = 0; $i0 < moreLikeThis.size(); $i0++) {
                    if ($i0 > 0) {
                        $out.append(',');
                    }
                    moreLikeThis.get($i0).write($out);
                }
                $out.append(']');
            }
            $count = Json.key($out, $count, "name");
            Json.text($out, name);
            if (overview != null) {
                $count = Json.key($out, $count, "overview");
                Json.text($out, overview);
            }
            if (rating != null) {
                $count = Json.key($out, $count, "rating");
                rating.write($out);
            }
            if (reminderSet != null) {
                $count = Json.key($out, $count, "reminder_set");
                $out.append(reminderSet);
            }
            if (resume != null) {
                $count = Json.key($out, $count, "resume");
                resume.write($out);
            }
            if (runtimeMin != null) {
                $count = Json.key($out, $count, "runtime_min");
                $out.append(runtimeMin);
            }
            if (seasons != null) {
                $count = Json.key($out, $count, "seasons");
                $out.append('[');
                for (int $i0 = 0; $i0 < seasons.size(); $i0++) {
                    if ($i0 > 0) {
                        $out.append(',');
                    }
                    seasons.get($i0).write($out);
                }
                $out.append(']');
            }
            $count = Json.key($out, $count, "title");
            title.write($out);
            if (year != null) {
                $count = Json.key($out, $count, "year");
                $out.append(year);
            }
            $out.append('}');
        }
    }

    /** movie:<id> or tv:<id>. A bare id does not parse. */
    public static final class TitleKey implements Wire {
        private static final Pattern SHAPE = Pattern.compile("^(movie|tv):(?:[1-9][0-9]{0,8}|1[0-9]{9}|20[0-9]{8}|21[0-3][0-9]{7}|214[0-6][0-9]{6}|2147[0-3][0-9]{5}|21474[0-7][0-9]{4}|214748[0-2][0-9]{3}|2147483[0-5][0-9]{2}|21474836[0-3][0-9]|214748364[0-7])$");

        private final String value;

        public TitleKey(String value) {
            if (!SHAPE.matcher(value).matches()) {
                throw new IllegalArgumentException("not a TitleKey");
            }
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static TitleKey read(Object $json) {
            return new TitleKey(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof TitleKey && (((TitleKey) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** The two kinds of title. Closed on purpose: a new kind reaches old clients only through a dated transform, so a key always names a kind this build can open. */
    public static final class TitleKind implements Wire {
        public static final TitleKind MOVIE = new TitleKind("movie");
        public static final TitleKind TV = new TitleKind("tv");

        private final String value;

        public TitleKind(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static TitleKind read(Object $json) {
            return new TitleKind(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof TitleKind && (((TitleKind) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** One audio or text track, with the tag the manifest gave it. */
    public static final class Track implements Wire {
        private final String id;
        private final String label;
        private final String lang;
        private final Boolean selected;

        public Track(String id, String label, String lang, Boolean selected) {
            this.id = id;
            this.label = label;
            this.lang = lang;
            this.selected = selected;
        }

        public String getId() {
            return id;
        }

        public boolean hasLabel() {
            return label != null;
        }

        public String getLabel() {
            return label == null ? "" : label;
        }

        public boolean hasLang() {
            return lang != null;
        }

        public String getLang() {
            return lang == null ? "" : lang;
        }

        /** True for the track playing now. */
        public boolean hasSelected() {
            return selected != null;
        }

        public boolean getSelected() {
            return selected != null && selected;
        }

        public static Track read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "id", "Track");
            Object $raw1 = Json.at($fields, "label");
            Object $raw2 = Json.at($fields, "lang");
            Object $raw3 = Json.at($fields, "selected");
            return new Track(
                    Json.asText($raw0),
                    $raw1 == null ? null : Json.asText($raw1),
                    $raw2 == null ? null : Json.asText($raw2),
                    $raw3 == null ? null : Boolean.valueOf(Json.asBool($raw3)));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "id");
            Json.text($out, id);
            if (label != null) {
                $count = Json.key($out, $count, "label");
                Json.text($out, label);
            }
            if (lang != null) {
                $count = Json.key($out, $count, "lang");
                Json.text($out, lang);
            }
            if (selected != null) {
                $count = Json.key($out, $count, "selected");
                $out.append(selected);
            }
            $out.append('}');
        }
    }

    /** The wire dates a server answers. Never compared for equality. */
    public static final class VersionRange implements Wire {
        private final WireDate max;
        private final WireDate min;

        public VersionRange(WireDate max, WireDate min) {
            this.max = max;
            this.min = min;
        }

        public WireDate getMax() {
            return max;
        }

        public WireDate getMin() {
            return min;
        }

        public static VersionRange read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.need($fields, "max", "VersionRange");
            Object $raw1 = Json.need($fields, "min", "VersionRange");
            return new VersionRange(
                    WireDate.read($raw0),
                    WireDate.read($raw1));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            $count = Json.key($out, $count, "max");
            max.write($out);
            $count = Json.key($out, $count, "min");
            min.write($out);
            $out.append('}');
        }
    }

    /** Changed screen state. A part that is absent did not change. */
    public static final class View implements Wire {
        private final DeviceSignInView deviceSignIn;
        private final GateView gate;
        private final Hello hello;
        private final PlayView play;
        private final ProfileList profiles;
        private final RouteTarget route;
        private final SessionView session;

        public View(DeviceSignInView deviceSignIn, GateView gate, Hello hello, PlayView play, ProfileList profiles, RouteTarget route, SessionView session) {
            this.deviceSignIn = deviceSignIn;
            this.gate = gate;
            this.hello = hello;
            this.play = play;
            this.profiles = profiles;
            this.route = route;
            this.session = session;
        }

        public boolean hasDeviceSignIn() {
            return deviceSignIn != null;
        }

        public DeviceSignInView getDeviceSignIn() {
            return deviceSignIn == null ? null : deviceSignIn;
        }

        public boolean hasGate() {
            return gate != null;
        }

        public GateView getGate() {
            return gate == null ? null : gate;
        }

        /** The last hello read, so a screen takes limits and features from one place. */
        public boolean hasHello() {
            return hello != null;
        }

        public Hello getHello() {
            return hello == null ? null : hello;
        }

        public boolean hasPlay() {
            return play != null;
        }

        public PlayView getPlay() {
            return play == null ? null : play;
        }

        /** The account's profiles, for the picker. No PIN is ever held here. */
        public boolean hasProfiles() {
            return profiles != null;
        }

        public ProfileList getProfiles() {
            return profiles == null ? null : profiles;
        }

        public boolean hasRoute() {
            return route != null;
        }

        public RouteTarget getRoute() {
            return route == null ? null : route;
        }

        public boolean hasSession() {
            return session != null;
        }

        public SessionView getSession() {
            return session == null ? null : session;
        }

        public static View read(Object $json) {
            Map<String, Object> $fields = Json.object($json);
            Object $raw0 = Json.at($fields, "device_sign_in");
            Object $raw1 = Json.at($fields, "gate");
            Object $raw2 = Json.at($fields, "hello");
            Object $raw3 = Json.at($fields, "play");
            Object $raw4 = Json.at($fields, "profiles");
            Object $raw5 = Json.at($fields, "route");
            Object $raw6 = Json.at($fields, "session");
            return new View(
                    $raw0 == null ? null : DeviceSignInView.read($raw0),
                    $raw1 == null ? null : GateView.read($raw1),
                    $raw2 == null ? null : Hello.read($raw2),
                    $raw3 == null ? null : PlayView.read($raw3),
                    $raw4 == null ? null : ProfileList.read($raw4),
                    $raw5 == null ? null : RouteTarget.read($raw5),
                    $raw6 == null ? null : SessionView.read($raw6));
        }

        @Override
        public void write(StringBuilder $out) {
            $out.append('{');
            int $count = 0;
            if (deviceSignIn != null) {
                $count = Json.key($out, $count, "device_sign_in");
                deviceSignIn.write($out);
            }
            if (gate != null) {
                $count = Json.key($out, $count, "gate");
                gate.write($out);
            }
            if (hello != null) {
                $count = Json.key($out, $count, "hello");
                hello.write($out);
            }
            if (play != null) {
                $count = Json.key($out, $count, "play");
                play.write($out);
            }
            if (profiles != null) {
                $count = Json.key($out, $count, "profiles");
                profiles.write($out);
            }
            if (route != null) {
                $count = Json.key($out, $count, "route");
                route.write($out);
            }
            if (session != null) {
                $count = Json.key($out, $count, "session");
                session.write($out);
            }
            $out.append('}');
        }
    }

    /** Windows of time inside a video. */
    public static final class WindowKind implements Wire {
        public static final WindowKind RECAP = new WindowKind("recap");
        public static final WindowKind INTRO = new WindowKind("intro");
        public static final WindowKind CREDITS = new WindowKind("credits");
        public static final WindowKind AGE_CARD = new WindowKind("age_card");
        public static final WindowKind CUE = new WindowKind("cue");

        private final String value;

        public WindowKind(String value) {
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static WindowKind read(Object $json) {
            return new WindowKind(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof WindowKind && (((WindowKind) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }

    /** A wire date, YYYY-MM-DD. */
    public static final class WireDate implements Wire {
        private static final Pattern SHAPE = Pattern.compile("^[0-9]{4}-[0-9]{2}-[0-9]{2}$");

        private final String value;

        public WireDate(String value) {
            if (!SHAPE.matcher(value).matches()) {
                throw new IllegalArgumentException("not a WireDate");
            }
            this.value = value;
        }

        public String value() {
            return value;
        }

        public static WireDate read(Object $json) {
            return new WireDate(Json.asText($json));
        }

        @Override
        public void write(StringBuilder $out) {
            Json.text($out, value);
        }

        @Override
        public boolean equals(Object other) {
            return other instanceof WireDate && (((WireDate) other).value).equals(value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }

        @Override
        public String toString() {
            return value;
        }
    }
}
