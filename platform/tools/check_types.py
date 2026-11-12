#!/usr/bin/env python3
"""Compile the generated client types and run the contract's fixtures through them.

Valid fixtures must read and write back the same in Swift and Kotlin and typecheck in TypeScript.
Tolerated fixtures must read. Refused fixtures must fail.

Each compiler is found on the PATH, so a machine that has one runs that language without being told
to. These name one elsewhere, because machines differ:

  TSC                  a TypeScript compiler
  SWIFTC               swiftc
  KOTLINC              kotlinc
  KOTLIN_PLUGIN        the kotlinx serialization compiler plugin jar
  KOTLIN_CLASSPATH     kotlinx serialization core and json jars, joined by ':'
  KOTLIN_STDLIB        the kotlin-stdlib jar
  JAVAC                javac
  JAVA                 java
  CXX                  a C++ compiler, which needs no library at all

A language whose compiler is nowhere is printed as not run, with what is missing, and counted in a
last line. That line is why this can be run everywhere: on a machine with no Swift and no Kotlin the
languages it could not prove are named out loud, never passed over in silence.

Java and C++ are the arms with no dependency to install: the generated Java and the generated
header each carry their own JSON reader and writer, so a JDK alone proves one and a C++ compiler
alone proves the other. They are also the two arms that cannot trust `which`; see `jdk` and `cxx`.

The exit code is 1 when a language that did run failed, and 1 when none of them ran: a guard that
proved nothing is a failure, not a pass. CI therefore installs a TypeScript compiler, so at least
one language is always proved; before 2026-11-11 CI had none of the three and this step printed
"0 of 3 languages compiled (none)" and passed, which is the silence this file exists to break.
"""

import json
import os
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
CONTRACT = os.path.join(os.path.dirname(HERE), "contract")
FIXTURES = os.path.join(CONTRACT, "fixtures", "2026-11-05")
TYPES = os.path.join(CONTRACT, "generated", "types")

# What TypeScript's type system cannot say, so the fixture is left out of the TypeScript run and
# named out loud in the output instead. A template literal type matches any number where it writes
# `${number}`: it holds the shape of a title key and of a date, and no range inside either. Every
# one of these is refused by Rust, by Kotlin and by Swift; TypeScript refuses it only at run time.
TS_CANNOT_SEE = {
    # `movie:${number}` has no lower bound, so id 0 fits the type.
    "TitleKey.id_zero.json",
    # ... and no upper bound, so an id above MAX_TITLE_ID fits it too.
    "TitleKey.id_over_max.json",
    "TitleKey.id_ten_digits.json",
    # `${number}-${number}-${number}` has no month range and no day range.
    "CalendarDay.month_13.json",
    "CalendarDay.day_32.json",
}


def compiler(variable, program):
    """The compiler to use, and what is missing when there is none."""
    named = os.environ.get(variable)
    if named:
        return named, None
    found = shutil.which(program)
    return found, None if found else f"no {program} on the PATH and {variable} is not set"


# Android Studio ships a JDK. On this project's machines it is the only real one, so it is the last
# place looked for a JDK tool, after the PATH and after the variable. It is read from JBR_BIN so a
# test can point it at an empty folder and simulate a machine with no JDK at all, which is the
# scenario test_check_types.py exists to hold down and which this fallback would otherwise defeat.
JBR = os.environ.get(
    "JBR_BIN", "/Applications/Android Studio.app/Contents/jbr/Contents/Home/bin"
)


def answers(tool, flag="-version"):
    """Whether the tool runs at all, asked by running it.

    The flag is a parameter because the JDK tools and clang disagree about it: `javac -version` is
    right and `clang++ -version` exits 1 with "unknown argument; did you mean '--version'?", so a
    probe that asked every tool the same way would call a working compiler broken.
    """
    try:
        done = subprocess.run([tool, flag], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    except OSError:
        return False
    return done.returncode == 0


def jdk(variable, program):
    """A JDK tool, believed only once it has answered `-version`.

    This does not use `compiler` above, and must not be simplified into it. macOS ships
    /usr/bin/javac and /usr/bin/java as stubs: they exist, they satisfy `shutil.which`, and when no
    JDK is installed they exit 1 saying "Unable to locate a Java Runtime". Measured on this machine
    on 2026-11-12: `which javac` answers /usr/bin/javac and `/usr/bin/javac -version` exits 1.

    So believing `which` would have this file name Java as a language it proved, then fail the whole
    run on a machine that simply has no JDK on the PATH — the exact silence the last line of this
    file exists to break, inverted into a false alarm. A found tool is therefore run before it is
    used, and a tool named by its variable is taken as given, because naming one is a deliberate act.
    """
    named = os.environ.get(variable)
    if named:
        return named, None
    for candidate in (shutil.which(program), os.path.join(JBR, program)):
        if candidate and answers(candidate):
            return candidate, None
    return None, (
        f"no {program} that runs, on the PATH or in Android Studio's JDK, and {variable} is not set"
    )


def cxx(variable, program):
    """A C++ compiler, believed only once it has answered `--version`.

    The same rule as `jdk` above, for the same shape of reason. /usr/bin/clang++ is on every mac
    and is not a compiler: it is a shim that asks xcrun for the active developer directory. With no
    Command Line Tools and no Xcode it satisfies `shutil.which`, then exits 1. Measured on this
    machine on 2026-11-12: `/usr/bin/clang++ --version` exits 0, and with DEVELOPER_DIR pointed at
    a path that does not exist it exits 1 with "xcrun: error: missing DEVELOPER_DIR path".

    A tool named by CXX is taken as given, because naming one is a deliberate act.
    """
    named = os.environ.get(variable)
    if named:
        return named, None
    found = shutil.which(program)
    if found and answers(found, "--version"):
        return found, None
    return None, f"no {program} that runs on the PATH and {variable} is not set"


def missing_vars(*variables):
    """The names of those that the environment does not set."""
    return [name for name in variables if not os.environ.get(name)]


def names(kind):
    return sorted(os.listdir(os.path.join(FIXTURES, kind)))


def type_of(name):
    return name.split(".")[0]


def run(command, **options):
    return subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, **options)


def check_typescript(work):
    tool, why = compiler("TSC", "tsc")
    if why:
        return None, why
    shutil.copy(os.path.join(TYPES, "contract.ts"), work)
    valid = names("valid")
    blind = [name for name in names("refused") if name in TS_CANNOT_SEE]
    refused = [name for name in names("refused") if name not in TS_CANNOT_SEE]
    unknown = sorted(TS_CANNOT_SEE - set(names("refused")))
    used = sorted({type_of(name) for name in valid + refused})
    lines = ["import type { %s } from \"./contract\";" % ", ".join(used), ""]
    for index, name in enumerate(valid):
        with open(os.path.join(FIXTURES, "valid", name)) as handle:
            lines.append("export const v%d: %s = %s;" % (index, type_of(name), handle.read().strip()))
    for index, name in enumerate(refused):
        with open(os.path.join(FIXTURES, "refused", name)) as handle:
            text = json.dumps(json.load(handle))
        lines.append("// @ts-expect-error %s" % name)
        lines.append("export const r%d: %s = %s;" % (index, type_of(name), text))
    path = os.path.join(work, "check.ts")
    with open(path, "w") as handle:
        handle.write("\n".join(lines) + "\n")
    done = run([tool, "--noEmit", "--strict", "--target", "es2022", "--module", "esnext", "--moduleResolution", "bundler", path], cwd=work)
    # A fixture this cannot ask about is said out loud, with the type that cannot say it. A stale
    # name in that list is a failure of its own: it would hide a fixture TypeScript does refuse.
    code = done.returncode
    text = "%d valid typed, %d refused at compile time; the type cannot refuse: %s" % (
        len(valid), len(refused), ", ".join(blind) or "nothing",
    )
    if unknown:
        code = code or 1
        text += "\nTS_CANNOT_SEE names %s, which is no refused fixture" % ", ".join(unknown)
    return (code, text + "\n" + done.stdout[-3000:]), None


def check_java(work):
    """Compile the generated Java and run every fixture through the codec it carries.

    A valid fixture must read and write back the same document, a tolerated one must read, and a
    refused one must throw. The fixture list is read from the folder, never counted here: there are
    231 valid, 13 tolerated and 27 refused today and each of those numbers moves on its own.
    """
    javac, why = jdk("JAVAC", "javac")
    if why:
        return None, why
    java, why = jdk("JAVA", "java")
    if why:
        return None, why
    package = os.path.join(work, "contract")
    os.makedirs(package)
    shutil.copy(os.path.join(TYPES, "Contract.java"), package)
    lines = [
        "import contract.Contract;",
        "import java.nio.charset.StandardCharsets;",
        "import java.nio.file.Files;",
        "import java.nio.file.Path;",
        "import java.nio.file.Paths;",
        "import java.util.function.Function;",
        "",
        "public final class Check {",
        "    private static int failures = 0;",
        "    private static Path root;",
        "",
        "    private static String file(String kind, String name) throws Exception {",
        "        return new String(Files.readAllBytes(root.resolve(kind).resolve(name)), StandardCharsets.UTF_8);",
        "    }",
        "",
        "    private static void valid(String name, Function<Object, ? extends Contract.Wire> read) {",
        "        try {",
        "            String text = file(\"valid\", name);",
        "            String back = Contract.Json.render(read.apply(Contract.Json.parse(text)));",
        "            if (!Contract.Json.parse(text).equals(Contract.Json.parse(back))) {",
        "                failures++;",
        "                System.out.println(\"differs \" + name + \" \" + back);",
        "            }",
        "        } catch (Exception error) {",
        "            failures++;",
        "            System.out.println(\"valid failed \" + name + \" \" + error);",
        "        }",
        "    }",
        "",
        "    private static void tolerated(String name, Function<Object, ? extends Contract.Wire> read) {",
        "        try {",
        "            read.apply(Contract.Json.parse(file(\"tolerated\", name)));",
        "        } catch (Exception error) {",
        "            failures++;",
        "            System.out.println(\"tolerated failed \" + name + \" \" + error);",
        "        }",
        "    }",
        "",
        "    private static void refused(String name, Function<Object, ? extends Contract.Wire> read) {",
        "        boolean parsed;",
        "        try {",
        "            read.apply(Contract.Json.parse(file(\"refused\", name)));",
        "            parsed = true;",
        "        } catch (Exception error) {",
        "            parsed = false;",
        "        }",
        "        if (parsed) {",
        "            failures++;",
        "            System.out.println(\"parsed \" + name);",
        "        }",
        "    }",
        "",
        "    public static void main(String[] args) {",
        "        root = Paths.get(args[0]);",
    ]
    for kind in ("valid", "tolerated", "refused"):
        for name in names(kind):
            lines.append("        %s(\"%s\", Contract.%s::read);" % (kind, name, type_of(name)))
    lines += [
        "        System.out.println(\"failures \" + failures);",
        "        System.exit(failures == 0 ? 0 : 1);",
        "    }",
        "}",
    ]
    with open(os.path.join(work, "Check.java"), "w") as handle:
        handle.write("\n".join(lines) + "\n")
    out = os.path.join(work, "java-out")
    built = run([javac, "-Xlint:all", "-Werror", "-d", out, "contract/Contract.java", "Check.java"], cwd=work)
    if built.returncode != 0:
        return (built.returncode, built.stdout[-3000:]), None
    done = run([java, "-cp", out, "Check", FIXTURES])
    counted = "%d valid, %d tolerated, %d refused" % (
        len(names("valid")), len(names("tolerated")), len(names("refused")),
    )
    return (done.returncode, counted + "\n" + done.stdout[-3000:]), None


def check_cpp(work):
    """Compile the generated header and run every fixture through the codec it carries.

    The same three verdicts as Java: a valid fixture must read and write back the same document, a
    tolerated one must read, and a refused one must throw. The fixture list is read from the folder
    and never counted here.

    Two things this proves that no other arm can. The header is included twice in one translation
    unit, so the include guard is held down rather than assumed. And it is compiled with
    -Wall -Wextra -Wpedantic -Werror, which is the Xbox surface's own bar (note 220), so a doc
    comment that closed itself early or a parameter the emitter left unused fails here.

    There is no `Wire` interface to hand a reader through, the way check_java does: the generated
    types are unrelated aggregates and C++ has no common base to put them under. Each fixture is
    therefore a template instantiation named with its type, which is also how a C++ surface will
    read a message.
    """
    tool, why = cxx("CXX", "clang++")
    if why:
        return None, why
    shutil.copy(os.path.join(TYPES, "Contract.hpp"), work)
    lines = [
        "// Twice on purpose: the second include must add nothing, which is what the guard is for.",
        "#include \"Contract.hpp\"",
        "#include \"Contract.hpp\"",
        "",
        "#include <exception>",
        "#include <fstream>",
        "#include <iostream>",
        "#include <sstream>",
        "#include <stdexcept>",
        "#include <string>",
        "",
        "namespace {",
        "",
        "int failures = 0;",
        "std::string root;",
        "",
        "std::string file(const std::string& kind, const std::string& name) {",
        "    std::ifstream handle(root + \"/\" + kind + \"/\" + name);",
        "    if (!handle) {",
        "        throw std::runtime_error(\"cannot read \" + name);",
        "    }",
        "    std::ostringstream held;",
        "    held << handle.rdbuf();",
        "    return held.str();",
        "}",
        "",
        "template <class T>",
        "void valid(const char* name) {",
        "    try {",
        "        const contract::Json::Value before = contract::Json::parse(file(\"valid\", name));",
        "        const std::string back = contract::Json::render(T::read(before));",
        "        if (!contract::Json::same(before, contract::Json::parse(back))) {",
        "            failures++;",
        "            std::cout << \"differs \" << name << \" \" << back << \"\\n\";",
        "        }",
        "    } catch (const std::exception& error) {",
        "        failures++;",
        "        std::cout << \"valid failed \" << name << \" \" << error.what() << \"\\n\";",
        "    }",
        "}",
        "",
        "template <class T>",
        "void tolerated(const char* name) {",
        "    try {",
        "        T::read(contract::Json::parse(file(\"tolerated\", name)));",
        "    } catch (const std::exception& error) {",
        "        failures++;",
        "        std::cout << \"tolerated failed \" << name << \" \" << error.what() << \"\\n\";",
        "    }",
        "}",
        "",
        "template <class T>",
        "void refused(const char* name) {",
        "    bool parsed = false;",
        "    try {",
        "        T::read(contract::Json::parse(file(\"refused\", name)));",
        "        parsed = true;",
        "    } catch (const std::exception&) {",
        "        parsed = false;",
        "    }",
        "    if (parsed) {",
        "        failures++;",
        "        std::cout << \"parsed \" << name << \"\\n\";",
        "    }",
        "}",
        "",
        "}  // namespace",
        "",
        "int main(int count, char** given) {",
        "    if (count < 2) {",
        "        std::cout << \"no fixture folder\\n\";",
        "        return 1;",
        "    }",
        "    root = given[1];",
    ]
    for kind in ("valid", "tolerated", "refused"):
        for name in names(kind):
            lines.append("    %s<contract::%s>(\"%s\");" % (kind, type_of(name), name))
    lines += [
        "    std::cout << \"failures \" << failures << \"\\n\";",
        "    return failures == 0 ? 0 : 1;",
        "}",
    ]
    with open(os.path.join(work, "check.cpp"), "w") as handle:
        handle.write("\n".join(lines) + "\n")
    out = os.path.join(work, "check_cpp")
    built = run([tool, "-std=c++17", "-Wall", "-Wextra", "-Wpedantic", "-Werror", "check.cpp", "-o", out], cwd=work)
    if built.returncode != 0:
        return (built.returncode, built.stdout[-3000:]), None
    done = run([out, FIXTURES])
    counted = "%d valid, %d tolerated, %d refused" % (
        len(names("valid")), len(names("tolerated")), len(names("refused")),
    )
    return (done.returncode, counted + "\n" + done.stdout[-3000:]), None


def check_swift(work):
    tool, why = compiler("SWIFTC", "swiftc")
    if why:
        return None, why
    shutil.copy(os.path.join(TYPES, "Contract.swift"), work)
    lines = [
        "import Foundation",
        "var failures = 0",
        "let root = URL(fileURLWithPath: CommandLine.arguments[1])",
        "func file(_ kind: String, _ name: String) throws -> Data {",
        "    try Data(contentsOf: root.appendingPathComponent(kind).appendingPathComponent(name))",
        "}",
        "func valid<T: Codable>(_ type: T.Type, _ name: String) {",
        "    do {",
        "        let data = try file(\"valid\", name)",
        "        let back = try JSONEncoder().encode(try JSONDecoder().decode(T.self, from: data))",
        "        let a = try JSONSerialization.jsonObject(with: data, options: .fragmentsAllowed) as! NSObject",
        "        let b = try JSONSerialization.jsonObject(with: back, options: .fragmentsAllowed) as! NSObject",
        "        if !a.isEqual(b) { failures += 1; print(\"differs\", name, a, b) }",
        "    } catch { failures += 1; print(\"valid failed\", name, error) }",
        "}",
        "func tolerated<T: Codable>(_ type: T.Type, _ name: String) {",
        "    do { _ = try JSONDecoder().decode(T.self, from: try file(\"tolerated\", name)) }",
        "    catch { failures += 1; print(\"tolerated failed\", name, error) }",
        "}",
        "func refused<T: Codable>(_ type: T.Type, _ name: String) {",
        "    if (try? JSONDecoder().decode(T.self, from: try file(\"refused\", name))) != nil {",
        "        failures += 1; print(\"parsed\", name)",
        "    }",
        "}",
    ]
    for kind in ("valid", "tolerated", "refused"):
        for name in names(kind):
            lines.append("%s(%s.self, \"%s\")" % (kind, type_of(name), name))
    lines.append("print(\"failures\", failures)")
    lines.append("exit(failures == 0 ? 0 : 1)")
    with open(os.path.join(work, "main.swift"), "w") as handle:
        handle.write("\n".join(lines) + "\n")
    out = os.path.join(work, "check_swift")
    built = run([tool, "Contract.swift", "main.swift", "-o", out], cwd=work)
    if built.returncode != 0:
        return (built.returncode, built.stdout[-3000:]), None
    done = run([out, FIXTURES])
    return (done.returncode, done.stdout[-3000:]), None


def check_kotlin(work):
    tool, why = compiler("KOTLINC", "kotlinc")
    if why:
        return None, why
    # Kotlin cannot be run from the compiler alone: it needs the serialization plugin and the jars
    # to run against. Saying which one is absent is the difference between a skip and a mystery.
    absent = missing_vars("KOTLIN_PLUGIN", "KOTLIN_CLASSPATH", "KOTLIN_STDLIB")
    if absent:
        return None, "kotlinc is here but " + ", ".join(absent) + " is not set"
    # Through `jdk`, not `compiler`: Kotlin runs on the same JVM and would otherwise be handed the
    # macOS stub that `which java` finds. Latent until someone installs kotlinc, and then baffling.
    java, why = jdk("JAVA", "java")
    if why:
        return None, why
    plugin = os.environ["KOTLIN_PLUGIN"]
    classpath = os.environ["KOTLIN_CLASSPATH"]
    shutil.copy(os.path.join(TYPES, "Contract.kt"), work)
    lines = [
        "package contract",
        "",
        "import java.io.File",
        "import kotlinx.serialization.KSerializer",
        "",
        "var failures = 0",
        "",
        "fun <T> valid(root: File, name: String, serializer: KSerializer<T>) {",
        "    try {",
        "        val text = File(File(root, \"valid\"), name).readText()",
        "        val back = contractJson.encodeToString(serializer, contractJson.decodeFromString(serializer, text))",
        "        if (contractJson.parseToJsonElement(text) != contractJson.parseToJsonElement(back)) {",
        "            failures += 1",
        "            println(\"differs $name $back\")",
        "        }",
        "    } catch (error: Exception) {",
        "        failures += 1",
        "        println(\"valid failed $name $error\")",
        "    }",
        "}",
        "",
        "fun <T> tolerated(root: File, name: String, serializer: KSerializer<T>) {",
        "    try {",
        "        contractJson.decodeFromString(serializer, File(File(root, \"tolerated\"), name).readText())",
        "    } catch (error: Exception) {",
        "        failures += 1",
        "        println(\"tolerated failed $name $error\")",
        "    }",
        "}",
        "",
        "fun <T> refused(root: File, name: String, serializer: KSerializer<T>) {",
        "    val parsed = try {",
        "        contractJson.decodeFromString(serializer, File(File(root, \"refused\"), name).readText())",
        "        true",
        "    } catch (error: Exception) {",
        "        false",
        "    }",
        "    if (parsed) {",
        "        failures += 1",
        "        println(\"parsed $name\")",
        "    }",
        "}",
        "",
        "fun main(args: Array<String>) {",
        "    val root = File(args[0])",
    ]
    for kind in ("valid", "tolerated", "refused"):
        for name in names(kind):
            lines.append("    %s(root, \"%s\", %s.serializer())" % (kind, name, type_of(name)))
    lines += ["    println(\"failures $failures\")", "    kotlin.system.exitProcess(if (failures == 0) 0 else 1)", "}"]
    with open(os.path.join(work, "Check.kt"), "w") as handle:
        handle.write("\n".join(lines) + "\n")
    out = os.path.join(work, "kotlin-out")
    built = run([tool, "-nowarn", "-Xplugin=" + plugin, "-cp", classpath, "Contract.kt", "Check.kt", "-d", out], cwd=work)
    if built.returncode != 0:
        return (built.returncode, built.stdout[-3000:]), None
    done = run([java, "-cp", os.pathsep.join([out, classpath, os.environ["KOTLIN_STDLIB"]]), "contract.CheckKt", FIXTURES])
    return (done.returncode, done.stdout[-3000:]), None


def main():
    work = tempfile.mkdtemp(prefix="contract-types-")
    worst = 0
    ran = []
    skipped = []
    steps = (
        ("typescript", check_typescript),
        ("java", check_java),
        ("swift", check_swift),
        ("kotlin", check_kotlin),
        ("cpp", check_cpp),
    )
    for label, step in steps:
        folder = os.path.join(work, label)
        os.makedirs(folder)
        result, why = step(folder)
        if result is None:
            print("%s: NOT RUN, %s" % (label, why))
            skipped.append(label)
            continue
        code, text = result
        print("%s: exit %d\n%s" % (label, code, text.strip()))
        ran.append(label)
        worst = worst or code
    shutil.rmtree(work, ignore_errors=True)
    # The last line is the whole point of running this where a compiler may be absent: it says how
    # much of the contract was proved, so a run that proved none of it cannot read as a pass.
    # The total is read from the table above and not written down here: an arm added without the
    # count moving would read as a full run that proved one language less.
    print("types: %d of %d languages compiled (%s)%s" % (
        len(ran),
        len(steps),
        ", ".join(ran) or "none",
        "; NOT RUN: " + ", ".join(skipped) if skipped else "",
    ))
    if not ran:
        # Nothing was proved, so there is nothing to pass. A machine with no compiler at all says
        # so with an exit code, and CI installs one (.github/workflows/check.yml).
        print("types: no language compiled, so nothing about the generated types was proved")
        return worst or 1
    return worst


if __name__ == "__main__":
    sys.exit(main())
