#!/usr/bin/env python3
"""Compile the generated client types and run the contract's fixtures through them.

Valid fixtures must read and write back the same in Swift and Kotlin and typecheck in TypeScript.
Tolerated fixtures must read. Refused fixtures must fail. Paths to compilers come from the
environment, because they differ per machine:

  TSC                  a TypeScript compiler
  SWIFTC               swiftc
  KOTLINC              kotlinc
  KOTLIN_PLUGIN        the kotlinx serialization compiler plugin jar
  KOTLIN_CLASSPATH     kotlinx serialization core and json jars, joined by ':'
  KOTLIN_STDLIB        the kotlin-stdlib jar
  JAVA                 java

A language with no compiler set is reported as not run, and the exit code is 1.
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

# A template type has no number range, so this one is refused only at run time.
TS_CANNOT_SEE = {"TitleKey.id_zero.json"}


def names(kind):
    return sorted(os.listdir(os.path.join(FIXTURES, kind)))


def type_of(name):
    return name.split(".")[0]


def run(command, **options):
    return subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, **options)


def check_typescript(work):
    compiler = os.environ.get("TSC")
    if not compiler:
        return None
    shutil.copy(os.path.join(TYPES, "contract.ts"), work)
    valid = names("valid")
    refused = [name for name in names("refused") if name not in TS_CANNOT_SEE]
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
    done = run([compiler, "--noEmit", "--strict", "--target", "es2022", "--module", "esnext", "--moduleResolution", "bundler", path], cwd=work)
    return done.returncode, "%d valid typed, %d refused at compile time\n%s" % (len(valid), len(refused), done.stdout[-3000:])


def check_swift(work):
    compiler = os.environ.get("SWIFTC")
    if not compiler:
        return None
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
    built = run([compiler, "Contract.swift", "main.swift", "-o", out], cwd=work)
    if built.returncode != 0:
        return built.returncode, built.stdout[-3000:]
    done = run([out, FIXTURES])
    return done.returncode, done.stdout[-3000:]


def check_kotlin(work):
    compiler = os.environ.get("KOTLINC")
    if not compiler:
        return None
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
    built = run([compiler, "-nowarn", "-Xplugin=" + plugin, "-cp", classpath, "Contract.kt", "Check.kt", "-d", out], cwd=work)
    if built.returncode != 0:
        return built.returncode, built.stdout[-3000:]
    done = run([os.environ["JAVA"], "-cp", os.pathsep.join([out, classpath, os.environ["KOTLIN_STDLIB"]]), "contract.CheckKt", FIXTURES])
    return done.returncode, done.stdout[-3000:]


def main():
    work = tempfile.mkdtemp(prefix="contract-types-")
    worst = 0
    for label, step in (("typescript", check_typescript), ("swift", check_swift), ("kotlin", check_kotlin)):
        folder = os.path.join(work, label)
        os.makedirs(folder)
        result = step(folder)
        if result is None:
            print("%s: not run, no compiler set" % label)
            worst = 1
            continue
        code, text = result
        print("%s: exit %d\n%s" % (label, code, text.strip()))
        worst = worst or code
    shutil.rmtree(work, ignore_errors=True)
    return worst


if __name__ == "__main__":
    sys.exit(main())
