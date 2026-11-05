// Compiles the page to one ES5 file with the TypeScript compiler. There is no
// bundler dependency: each module becomes a function in a small registry.

import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import ts from "typescript";

export function bundle(folder: string, entry: string): string {
  const modules = readdirSync(folder).filter((f) => f.endsWith(".ts")).sort();
  const parts: string[] = [];
  for (const file of modules) {
    const out = ts.transpileModule(readFileSync(join(folder, file), "utf8"), {
      compilerOptions: { target: ts.ScriptTarget.ES5, module: ts.ModuleKind.CommonJS, removeComments: true }
    });
    parts.push(`${JSON.stringify("./" + file)}:function(exports,require){${out.outputText}}`);
  }
  return `(function(){var d={${parts.join(",\n")}},c={};function r(n){if(c[n])return c[n].exports;var m=c[n]={exports:{}};d[n](m.exports,r);return m.exports}r(${JSON.stringify("./" + entry)})})();\n`;
}
