/* A host for the core in one C file: the starting point for a device with a C toolchain.
 */
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "core.h"

typedef struct { char *at; size_t len, cap; } Text;
typedef struct { const char *at; size_t len; } Span; /* a slice of JSON text */
typedef struct { char *key, *value; } Pair;
typedef struct { Pair *pairs; size_t len; } Table; /* JSON literals, sorted by key */

typedef struct {
    CoreHandle *core;
    Table store, secrets;
    int has_store, has_secret, has_wake;
    char contract[32];
    unsigned long long wall_start, clock, mono_base, wake;
    char **lines;
    size_t line_count, next_script, next_answer;
} Host;

static void die(const char *form, ...) {
    va_list args;
    va_start(args, form);
    vfprintf(stderr, form, args);
    va_end(args);
    fputc('\n', stderr);
    exit(1);
}

static void *grown(void *block, size_t size) {
    void *moved = realloc(block, size ? size : 1);
    if (!moved) die("out of memory");
    return moved;
}

static void add(Text *text, const char *bytes, size_t len) {
    if (text->len + len + 1 > text->cap) {
        text->cap = (text->len + len + 1) * 2;
        text->at = grown(text->at, text->cap);
    }
    memcpy(text->at + text->len, bytes, len);
    text->len += len;
    text->at[text->len] = '\0';
}

static void add_word(Text *text, const char *word) { add(text, word, strlen(word)); }

static char *copy_of(Span span) {
    char *copy = grown(NULL, span.len + 1);
    memcpy(copy, span.at, span.len);
    copy[span.len] = '\0';
    return copy;
}

/* Reading JSON: enough to walk a message from the core, which holds no spaces. */

static const char *skip_string(const char *at) {
    for (at++; *at != '"'; at++) {
        if (*at == '\\') at++;
        if (!*at) die("a string has no end");
    }
    return at + 1;
}

static const char *skip_value(const char *at) {
    int depth = 0;
    if (*at != '{' && *at != '[' && *at != '"') {
        while (*at && *at != ',' && *at != '}' && *at != ']') at++;
        return at;
    }
    do {
        if (!*at) die("a value has no end");
        if (*at == '"') {
            at = skip_string(at);
            continue;
        }
        depth += (*at == '{' || *at == '[') - (*at == '}' || *at == ']');
        at++;
    } while (depth > 0);
    return at;
}

/* The value of a key in the object at `object`, as text. Empty when the key is absent. */
static Span member(const char *object, const char *key) {
    Span none = {NULL, 0};
    size_t want = strlen(key);
    const char *at = object + 1;
    while (*at == '"') {
        const char *colon = skip_string(at);
        const char *end = skip_value(colon + 1);
        if ((size_t)(colon - at - 2) == want && !memcmp(at + 1, key, want)) {
            Span found = {colon + 1, (size_t)(end - colon - 1)};
            return found;
        }
        at = *end == ',' ? end + 1 : end;
    }
    return none;
}

static int is(Span span, const char *word) {
    return span.len == strlen(word) && !memcmp(span.at, word, span.len);
}

/* Finds a key in a table, or the place it would go. */

static int find(const Table *table, const char *key, size_t *place) {
    for (*place = 0; *place < table->len; (*place)++) {
        int order = strcmp(table->pairs[*place].key, key);
        if (order == 0) return 1;
        if (order > 0) break;
    }
    return 0;
}

static void put(Table *table, char *key, char *value) {
    size_t place;
    if (find(table, key, &place)) {
        free(key);
        free(table->pairs[place].value);
        table->pairs[place].value = value;
        return;
    }
    table->pairs = grown(table->pairs, (table->len + 1) * sizeof(Pair));
    memmove(table->pairs + place + 1, table->pairs + place, (table->len - place) * sizeof(Pair));
    table->pairs[place].key = key;
    table->pairs[place].value = value;
    table->len++;
}

static void remove_key(Table *table, const char *key) {
    size_t place;
    if (!find(table, key, &place)) return;
    free(table->pairs[place].key);
    free(table->pairs[place].value);
    table->len--;
    memmove(table->pairs + place, table->pairs + place + 1, (table->len - place) * sizeof(Pair));
}

/* Effects. Each answer is one event added to `events`, which starts with its opening bracket. */

static void event_start(Text *events, Span id) {
    if (events->len > 1) add_word(events, ",");
    add_word(events, "{\"id\":");
    add(events, id.at, id.len);
}

/* Get, put, delete and list on a table. The store and the secret store differ only in names. */
static void perform_table(Table *table, Span id, Span op, Span key, Span value, const char *done,
                          Text *events) {
    Span empty = {"\"\"", 2};
    char *name = copy_of(key.at ? key : empty);
    const char *found = NULL;
    size_t place;
    event_start(events, id);
    if (is(op, "\"list\"")) {
        const char *comma = "";
        add_word(events, ",\"keys\":[");
        for (place = 0; place < table->len; place++) {
            if (strncmp(table->pairs[place].key, name, strlen(name) - 1)) continue;
            add_word(events, comma);
            add_word(events, table->pairs[place].key);
            comma = ",";
        }
        add_word(events, "]");
    } else if (is(op, "\"get\"")) {
        if (find(table, name, &place)) found = table->pairs[place].value;
    } else if (is(op, "\"put\"")) {
        put(table, copy_of(key.at ? key : empty), copy_of(value.at ? value : empty));
    } else if (is(op, "\"delete\"")) {
        remove_key(table, name);
    }
    add_word(events, ",\"type\":\"");
    add_word(events, done);
    add_word(events, "\"");
    if (found) {
        add_word(events, ",\"value\":");
        add_word(events, found);
    }
    add_word(events, "}");
    free(name);
}

/* The next answer line: http, tab, id, tab, method, tab, path, tab, event. */
static void perform_http(Host *host, const char *effect, Text *events) {
    const char *fields[3] = {"id", "method", "path"};
    const char *at;
    int part;
    while (host->next_answer < host->line_count &&
           strncmp(host->lines[host->next_answer], "http\t", 5)) host->next_answer++;
    if (host->next_answer == host->line_count) die("no answer left on the tape");
    at = host->lines[host->next_answer++] + 5;
    for (part = 0; part < 3; part++) {
        Span want = member(effect, fields[part]);
        const char *end = strchr(at, '\t');
        if (!end || (size_t)(end - at) != want.len || memcmp(at, want.at, want.len))
            die("the tape does not fit the core's %s", fields[part]);
        at = end + 1;
    }
    if (events->len > 1) add_word(events, ",");
    add_word(events, at);
}

static void perform(Host *host, const char *effect, Text *events) {
    Span id = member(effect, "id"), kind = member(effect, "kind");
    if (is(kind, "\"http\"")) {
        perform_http(host, effect, events);
    } else if (is(kind, "\"store\"") && host->has_store) {
        perform_table(&host->store, id, member(effect, "op"), member(effect, "key"),
                      member(effect, "value"), "store_done", events);
    } else if (is(kind, "\"secret\"") && host->has_secret) {
        perform_table(&host->secrets, id, member(effect, "op"), member(effect, "name"),
                      member(effect, "value"), "secret_done", events);
    } else if (is(kind, "\"player\"") || is(kind, "\"view\"") || is(kind, "\"open\"") ||
               is(kind, "\"log\"")) {
        fprintf(stderr, "%.*s\n", (int)kind.len, kind.at); /* a real host acts on these */
    } else {
        event_start(events, id);
        add_word(events, ",\"type\":\"unsupported\"}");
    }
}

/* Step the core, perform what it asks, and step again until it asks for nothing more. */
static void step(Host *host, const char *first_events) {
    Text in = {0}, events = {0};
    char now[96];
    add_word(&events, first_events);
    for (;;) {
        uint8_t *out = NULL;
        size_t out_len = 0;
        char *message;
        const char *at;
        Span wake, effects;
        int status;
        in.len = 0;
        add_word(&in, "{\"contract\":\"");
        add_word(&in, host->contract);
        add_word(&in, "\",\"events\":");
        add_word(&in, events.at);
        snprintf(now, sizeof now, ",\"mono_ms\":%llu,\"now_ms\":%llu,\"type\":\"core_in\"}",
                 host->clock - host->mono_base, host->wall_start + host->clock);
        add_word(&in, now);
        printf("in %s\n", in.at);
        status = core_step(host->core, (const uint8_t *)in.at, in.len, &out, &out_len);
        if (status != CORE_OK) die("core_step gave %d", status);
        message = copy_of((Span){(const char *)out, out_len});
        core_free(out, out_len);
        printf("out %s\n", message);
        wake = member(message, "next_wake_ms");
        host->has_wake = wake.at != NULL;
        if (wake.at) host->wake = strtoull(wake.at, NULL, 10);
        effects = member(message, "effects");
        events.len = 0;
        add_word(&events, "[");
        for (at = effects.at + 1; *at != ']'; at += *at == ',') {
            perform(host, at, &events);
            at = skip_value(at);
        }
        free(message);
        if (events.len == 1) break;
        add_word(&events, "]");
    }
    if (host->has_wake && host->wake + host->mono_base <= host->clock) die("busy loop");
    free(in.at);
    free(events.at);
}

static void advance_to(Host *host, unsigned long long at_ms) {
    while (host->has_wake && host->wake + host->mono_base <= at_ms) {
        host->clock = host->wake + host->mono_base;
        step(host, "[]");
    }
    host->clock = at_ms;
}

static void open_core(Host *host, const char *line) {
    const char *config = strchr(line, ' ') + 1;
    if (line[0] == '1') host->mono_base = host->clock;
    if (host->core) core_close(host->core);
    host->core = core_open((const uint8_t *)config, strlen(config));
    if (!host->core) die("core_open refused %s", config);
    host->has_wake = 0;
    printf("open %s\n", config);
}

static void read_tape(Host *host, const char *path) {
    FILE *file = fopen(path, "rb");
    char *bytes, *line;
    long size;
    if (!file) die("cannot read %s", path);
    fseek(file, 0, SEEK_END);
    size = ftell(file);
    rewind(file);
    bytes = grown(NULL, (size_t)size + 1);
    if (fread(bytes, 1, (size_t)size, file) != (size_t)size) die("cannot read %s", path);
    bytes[size] = '\0';
    fclose(file);
    for (line = strtok(bytes, "\n"); line; line = strtok(NULL, "\n")) {
        host->lines = grown(host->lines, (host->line_count + 1) * sizeof(char *));
        host->lines[host->line_count++] = line;
    }
}

int main(int argc, char **argv) {
    static Host host;
    if (argc != 2) die("usage: terminal session.tape");
    if (core_abi() != CORE_ABI) die("this host was built for ABI %d", CORE_ABI);
    read_tape(&host, argv[1]);
    for (; host.next_script < host.line_count; host.next_script++) {
        const char *line = host.lines[host.next_script];
        if (!strncmp(line, "contract ", 9)) {
            snprintf(host.contract, sizeof host.contract, "%s", line + 9);
        } else if (!strncmp(line, "wall ", 5)) {
            host.wall_start = strtoull(line + 5, NULL, 10);
        } else if (!strncmp(line, "device ", 7)) {
            host.has_store = line[7] == '1';
            host.has_secret = line[9] == '1';
        } else if (!strncmp(line, "open ", 5)) {
            open_core(&host, line + 5);
        } else if (!strncmp(line, "at ", 3)) {
            advance_to(&host, strtoull(line + 3, NULL, 10));
        } else if (!strcmp(line, "forget")) {
            while (host.secrets.len) remove_key(&host.secrets, host.secrets.pairs[0].key);
        } else if (!strncmp(line, "step ", 5)) {
            step(&host, line + 5);
        } else if (strncmp(line, "http\t", 5)) {
            die("unknown line: %s", line);
        }
    }
    if (host.core) core_close(host.core);
    return 0;
}
