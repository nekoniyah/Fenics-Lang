const vscode = require("vscode");
const path = require("path");
const fs = require("fs");

function activate(context) {
  const keywords = [
    { label: "if", detail: "Control flow", doc: "Starts a conditional block." },
    {
      label: "else",
      detail: "Control flow",
      doc: "Fallback branch for conditionals.",
    },
    { label: "for", detail: "Loop", doc: "Iterate over collections or pairs." },
    { label: "while", detail: "Loop", doc: "Loop while a condition is true." },
    {
      label: "loop",
      detail: "Reactive loop",
      doc: "Loop that activates on condition change.",
    },
    {
      label: "return",
      detail: "Function",
      doc: "Return a value from a function.",
    },
    {
      label: "try",
      detail: "Error handling",
      doc: "Try block for error handling.",
    },
    {
      label: "catch",
      detail: "Error handling",
      doc: "Catch branch for handling errors.",
    },
    {
      label: "then",
      detail: "Ternary",
      doc: "Ternary true branch (if ... then ... otherwise ...).",
    },
    { label: "otherwise", detail: "Ternary", doc: "Ternary false branch." },
    {
      label: "in",
      detail: "Iteration",
      doc: "Membership or iteration clause.",
    },
    {
      label: "block",
      detail: "Effect control",
      doc: "Prevent further effects in expression.",
    },
    {
      label: "lib",
      detail: "Module export",
      doc: "Define a library module and export functions.",
    },
    {
      label: "import",
      detail: "Module import",
      doc: "Import a module by name or path.",
    },
    {
      label: "as",
      detail: "Alias",
      doc: "Assign an alias to an imported module.",
    },
  ];

  const builtins = [
    {
      label: "print",
      detail: "builtin",
      doc: "Prints values to console. Usage: print(value).",
    },
    {
      label: "input",
      detail: "builtin",
      doc: "Reads a line of input from the user.",
    },
    {
      label: "len",
      detail: "builtin",
      doc: "Gets length of arrays, strings, objects.",
    },
    {
      label: "type",
      detail: "builtin",
      doc: "Returns the type of a value as String.",
    },
    { label: "str", detail: "builtin", doc: "Converts a value to String." },
    { label: "int", detail: "builtin", doc: "Converts a value to Int." },
    {
      label: "sort",
      detail: "builtin",
      doc: "Returns a sorted copy of an array.",
    },
    {
      label: "reverse",
      detail: "builtin",
      doc: "Returns a reversed copy of an array.",
    },
    {
      label: "has",
      detail: "builtin",
      doc: "Checks if object/array contains a key or value.",
    },
    {
      label: "keys",
      detail: "builtin",
      doc: "Returns keys of an object as an array.",
    },
    {
      label: "first",
      detail: "builtin",
      doc: "Returns the first element of an array.",
    },
    {
      label: "last",
      detail: "builtin",
      doc: "Returns the last element of an array.",
    },
    {
      label: "split",
      detail: "builtin",
      doc: "Splits a string into an array by delimiter.",
    },
    { label: "length", detail: "builtin", doc: "Alias of len for strings." },
  ];

  const types = [
    { label: "Int", detail: "type", doc: "Integer number type." },
    { label: "Float", detail: "type", doc: "Floating-point number type." },
    { label: "String", detail: "type", doc: "String type." },
    { label: "Boolean", detail: "type", doc: "Boolean type." },
    { label: "Bool", detail: "type", doc: "Alias of Boolean." },
    { label: "Array", detail: "type", doc: "Array collection type." },
    { label: "Object", detail: "type", doc: "Object/dictionary type." },
    { label: "Regex", detail: "type", doc: "Regular expression literal type." },
    {
      label: "List",
      detail: "generic type",
      doc: "Typed list, e.g., List(Int).",
    },
    { label: "Pairs", detail: "generic type", doc: "Key-value typed pairs." },
  ];

  const modulesIndex = new Map(); // moduleName -> { exports: Set<string>, uri: vscode.Uri }

  function inferTypeFromValue(v) {
    if (!v) return undefined;
    const s = v.trim();
    if (!s) return undefined;
    if (s.startsWith('"')) return "String";
    if (/^\d+\.\d+$/.test(s)) return "Float";
    if (/^\d+$/.test(s)) return "Int";
    if (/^(true|false)\b/.test(s)) return "Boolean";
    if (s.startsWith("[")) return "Array";
    if (/^\/(?![/*])(?:\\.|[^/])+\/[a-z]*$/.test(s)) return "Regex";
    return undefined;
  }

  function indexVariablesFromText(text) {
    const vars = new Map(); // name -> { kind: 'const'|'var'|'global'|'parameter', declaredType?: string, inferredType?: string, fn?: string, line: number, valuePreview?: string, objectKeys?: string[] }
    const lines = text.split(/\r?\n/);
    function collectObjectKeys(startIndex) {
      const keys = [];
      for (let j = startIndex + 1; j < lines.length; j++) {
        const t = lines[j].trim();
        if (!t.startsWith("-")) break;
        const km =
          t.match(/^\-\s*([A-Za-z_][A-Za-z0-9_]*)\s*:/) ||
          t.match(/^\-\s*([A-Za-z_][A-Za-z0-9_]*)\s*$/);
        if (km) keys.push(km[1]);
      }
      return keys;
    }
    for (let i = 0; i < lines.length; i++) {
      const raw = lines[i];
      const line = raw.trim();
      if (!line || line.startsWith("//")) continue;
      if (line.startsWith("-")) continue; // object property entry, not a declaration

      // function parameters
      const fnMatch = line.match(
        /^fn\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(([^)]*)\)/
      );
      if (fnMatch) {
        const fnName = fnMatch[1];
        const params = fnMatch[2]
          .split(",")
          .map((s) => s.trim())
          .filter((s) => s.length);
        for (const p of params) {
          const pm = p.match(/^([a-zA-Z_][a-zA-Z0-9_]*)$/);
          if (pm) {
            vars.set(pm[1], { kind: "parameter", fn: fnName, line: i + 1 });
          }
        }
        continue;
      }

      // typed const: Type const name:
      const constTypedMatch = line.match(
        /^([A-Za-z][A-Za-z0-9_]*(?:\([^)]*\))?)\s+const\s+([A-Za-z_][A-Za-z0-9_]*)\s*:/
      );
      if (constTypedMatch) {
        const valuePart = raw.includes(":")
          ? raw.substring(raw.indexOf(":") + 1)
          : "";
        vars.set(constTypedMatch[2], {
          kind: "const",
          declaredType: constTypedMatch[1],
          inferredType: inferTypeFromValue(valuePart),
          line: i + 1,
          valuePreview: valuePart.trim().slice(0, 80),
        });
        continue;
      }
      // const name:
      const constMatch = line.match(/^const\s+([A-Za-z_][A-Za-z0-9_]*)\s*:/);
      if (constMatch) {
        const valuePart = raw.includes(":")
          ? raw.substring(raw.indexOf(":") + 1)
          : "";
        let inferred = inferTypeFromValue(valuePart);
        if (!inferred) {
          const next = (lines[i + 1] || "").trim();
          if (next.startsWith("-")) inferred = "Object";
        }
        vars.set(constMatch[1], {
          kind: "const",
          inferredType: inferred,
          line: i + 1,
          valuePreview: valuePart.trim().slice(0, 80),
          objectKeys: inferred === "Object" ? collectObjectKeys(i) : undefined,
        });
        continue;
      }
      // global name:
      const globalMatch = line.match(/^global\s+([A-Za-z_][A-Za-z0-9_]*)\s*:/);
      if (globalMatch) {
        const valuePart = raw.includes(":")
          ? raw.substring(raw.indexOf(":") + 1)
          : "";
        let inferred = inferTypeFromValue(valuePart);
        if (!inferred) {
          const next = (lines[i + 1] || "").trim();
          if (next.startsWith("-")) inferred = "Object";
        }
        vars.set(globalMatch[1], {
          kind: "global",
          inferredType: inferred,
          line: i + 1,
          valuePreview: valuePart.trim().slice(0, 80),
          objectKeys: inferred === "Object" ? collectObjectKeys(i) : undefined,
        });
        continue;
      }
      // typed var: Type name:
      const typedMatch = line.match(
        /^([A-Za-z][A-Za-z0-9_]*(?:\([^)]*\))?)\s+([A-Za-z_][A-Za-z0-9_]*)\s*:/
      );
      if (typedMatch && !/^fn\b/.test(line) && !/^lib\b/.test(line)) {
        const valuePart = raw.includes(":")
          ? raw.substring(raw.indexOf(":") + 1)
          : "";
        vars.set(typedMatch[2], {
          kind: "var",
          declaredType: typedMatch[1],
          inferredType: inferTypeFromValue(valuePart),
          line: i + 1,
          valuePreview: valuePart.trim().slice(0, 80),
        });
        continue;
      }
      // var: name:
      const varMatch = line.match(/^([A-Za-z_][A-Za-z0-9_]*)\s*:/);
      if (
        varMatch &&
        !/^fn\b/.test(line) &&
        !/^lib\b/.test(line) &&
        !/^(else|catch)\s*:/i.test(line)
      ) {
        const valuePart = raw.includes(":")
          ? raw.substring(raw.indexOf(":") + 1)
          : "";
        let inferred = inferTypeFromValue(valuePart);
        if (!inferred) {
          const next = (lines[i + 1] || "").trim();
          if (next.startsWith("-")) inferred = "Object";
        }
        vars.set(varMatch[1], {
          kind: "var",
          inferredType: inferred,
          line: i + 1,
          valuePreview: valuePart.trim().slice(0, 80),
          objectKeys: inferred === "Object" ? collectObjectKeys(i) : undefined,
        });
        continue;
      }
    }
    return vars;
  }
  function indexFunctionsFromText(text) {
    const fns = new Map(); // name -> { params: string[], returnType?: string, line: number }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
      const raw = lines[i];
      const m = raw
        .trim()
        .match(
          /^fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(([^)]*)\)\s*(?:->\s*([A-Za-z][A-Za-z0-9_]*(?:\([^)]*\))?))?\s*:/
        );
      if (m) {
        const name = m[1];
        const params = m[2]
          .split(",")
          .map((s) => s.trim())
          .filter((s) => s.length);
        const returnType = m[3];
        fns.set(name, { params, returnType, line: i + 1 });
      }
    }
    return fns;
  }

  function indexImportAliases(text) {
    const aliases = new Map(); // alias -> moduleName
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
      const raw = lines[i];
      const m = raw
        .trim()
        .match(
          /^import\s+([A-Za-z_][A-Za-z0-9_]*|"[^"]+")\s+as\s+([A-Za-z_][A-Za-z0-9_]*)\s*$/
        );
      if (m) {
        const target = m[1];
        const alias = m[2];
        const modName = target.startsWith('"') ? target.slice(1, -1) : target;
        aliases.set(alias, modName);
      }
    }
    return aliases;
  }

  async function buildModulesIndex() {
    modulesIndex.clear();
    const files = await vscode.workspace.findFiles("**/*.fenics");
    for (const uri of files) {
      try {
        const doc = await vscode.workspace.openTextDocument(uri);
        const lines = doc.getText().split(/\r?\n/);
        let currentModule = null;
        for (let i = 0; i < lines.length; i++) {
          const line = lines[i].trim();
          const libMatch = line.match(
            /^lib\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*:\s*$/
          );
          if (libMatch) {
            currentModule = libMatch[1];
            if (!modulesIndex.has(currentModule)) {
              modulesIndex.set(currentModule, { exports: new Set(), uri });
            }
            continue;
          }
          if (currentModule) {
            // capture lines like: - funcName (no colon)
            const exportMatch = line.match(
              /^\-\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*$/
            );
            if (exportMatch) {
              modulesIndex.get(currentModule).exports.add(exportMatch[1]);
              continue;
            }
            // end module block when a non-dashed line appears
            if (!/^\-\s*/.test(line) && line.length > 0) {
              currentModule = null;
            }
          }
        }
      } catch (e) {
        // ignore file read errors
      }
    }
  }

  // Build initial index and refresh on file changes
  buildModulesIndex();
  const watcher = vscode.workspace.createFileSystemWatcher("**/*.fenics");
  watcher.onDidCreate(buildModulesIndex);
  watcher.onDidChange(buildModulesIndex);
  watcher.onDidDelete(buildModulesIndex);
  context.subscriptions.push(watcher);

  context.subscriptions.push(
    vscode.commands.registerCommand("fenics.refreshIndex", buildModulesIndex)
  );

  // Diagnostics: validate imports, lib exports, and module method calls
  const diagnostics = vscode.languages.createDiagnosticCollection("fenics");
  context.subscriptions.push(diagnostics);

  function makeRange(lineNumber, start, end) {
    return new vscode.Range(
      new vscode.Position(lineNumber, start),
      new vscode.Position(lineNumber, end)
    );
  }

  function validateDocument(document) {
    if (document.languageId !== "fenics") return;
    const diags = [];
    const lines = document.getText().split(/\r?\n/);

    // Gather functions declared in this document for lib export validation
    const functions = new Set();
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      const m = line.trim().match(/^fn\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(/);
      if (m) functions.add(m[1]);
    }

    let currentModule = null;
    for (let i = 0; i < lines.length; i++) {
      const raw = lines[i];
      const line = raw.trim();
      if (!line) continue;

      // Validate import statements
      const importMatch = line.match(
        /^import\s+(.+?)(?:\s+as\s+[A-Za-z_][A-Za-z0-9_]*)?\s*$/
      );
      if (importMatch) {
        const target = importMatch[1].trim();
        if (target.startsWith('"') && target.endsWith('"')) {
          const rel = target.slice(1, -1);
          const base = path.dirname(document.uri.fsPath);
          const full = path.resolve(
            base,
            rel.endsWith(".fenics") ? rel : `${rel}.fenics`
          );
          if (!fs.existsSync(full)) {
            const start = raw.indexOf(target);
            const end = start + target.length;
            diags.push(
              new vscode.Diagnostic(
                makeRange(i, start, end),
                `Imported path not found: ${rel}`,
                vscode.DiagnosticSeverity.Error
              )
            );
          }
        } else {
          const modName = target;
          if (!modulesIndex.has(modName)) {
            const start = raw.indexOf(modName);
            const end = start + modName.length;
            diags.push(
              new vscode.Diagnostic(
                makeRange(i, start, end),
                `Module "${modName}" not found in workspace`,
                vscode.DiagnosticSeverity.Error
              )
            );
          }
        }
      }

      // Track lib module and validate exports are declared
      const libMatch = line.match(/^lib\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*:\s*$/);
      if (libMatch) {
        currentModule = libMatch[1];
        continue;
      }
      if (currentModule) {
        const exportMatch = line.match(/^\-\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*$/);
        if (exportMatch) {
          const fnName = exportMatch[1];
          if (!functions.has(fnName)) {
            const start = raw.indexOf(fnName);
            const end = start + fnName.length;
            diags.push(
              new vscode.Diagnostic(
                makeRange(i, start, end),
                `Export "${fnName}" is not declared as a function in this file`,
                vscode.DiagnosticSeverity.Warning
              )
            );
          }
        } else if (line.length > 0 && !line.startsWith("-")) {
          currentModule = null;
        }
      }

      // Validate module method calls: module.method(
      const methodCallRegex =
        /(\b[A-Za-z_][A-Za-z0-9_]*)\.(\b[A-Za-z_][A-Za-z0-9_]*)\s*\(/g;
      let mm;
      while ((mm = methodCallRegex.exec(raw)) !== null) {
        const modName = mm[1];
        const method = mm[2];
        const start = mm.index + modName.length + 1;
        const end = start + method.length;
        const mod = modulesIndex.get(modName);
        if (!mod) {
          diags.push(
            new vscode.Diagnostic(
              makeRange(i, mm.index, mm.index + modName.length),
              `Unknown module: ${modName}`,
              vscode.DiagnosticSeverity.Warning
            )
          );
        } else if (!mod.exports.has(method)) {
          diags.push(
            new vscode.Diagnostic(
              makeRange(i, start, end),
              `Module "${modName}" has no export "${method}"`,
              vscode.DiagnosticSeverity.Warning
            )
          );
        }
      }

      // Simple declaration sanity: likely missing colon
      const declLikely = line.match(
        /^(const|global|[A-Za-z][A-Za-z0-9_]*\s+(?:const\s+)?[A-Za-z_][A-Za-z0-9_]*)\s+[^:]/
      );
      if (declLikely) {
        const idx = raw.indexOf(declLikely[1]);
        diags.push(
          new vscode.Diagnostic(
            makeRange(i, idx, idx + declLikely[1].length),
            `Possible missing ':' in declaration`,
            vscode.DiagnosticSeverity.Hint
          )
        );
      }
    }

    diagnostics.set(document.uri, diags);
  }

  // Validate on open/change/save
  context.subscriptions.push(
    vscode.workspace.onDidOpenTextDocument(validateDocument)
  );
  context.subscriptions.push(
    vscode.workspace.onDidChangeTextDocument((e) =>
      validateDocument(e.document)
    )
  );
  context.subscriptions.push(
    vscode.workspace.onDidSaveTextDocument(validateDocument)
  );
  // Validate currently open docs
  for (const doc of vscode.workspace.textDocuments) validateDocument(doc);

  function toCompletionItem(entry, kind, doc) {
    const item = new vscode.CompletionItem(entry.label || entry, kind);
    item.detail = entry.detail || undefined;
    item.documentation = new vscode.MarkdownString(doc || entry.doc || "");
    return item;
  }

  context.subscriptions.push(
    vscode.languages.registerCompletionItemProvider(
      { language: "fenics" },
      {
        provideCompletionItems(document, position) {
          const range =
            document.getWordRangeAtPosition(position) ||
            new vscode.Range(position, position);
          const line = document.lineAt(position.line).text;
          const textBefore = line.substring(0, position.character);

          const items = [];

          // Context: after 'import '
          if (/\bimport\s+[a-zA-Z_0-9]*$/.test(textBefore)) {
            for (const [modName, mod] of modulesIndex.entries()) {
              const item = new vscode.CompletionItem(
                modName,
                vscode.CompletionItemKind.Module
              );
              item.detail = "module";
              item.documentation = new vscode.MarkdownString(
                `Module ${modName} with exports: ${Array.from(mod.exports).join(
                  ", "
                )}`
              );
              items.push(item);
            }
            return items;
          }

          // Context: identifier. -> module method completion
          const dotMatch = textBefore.match(/\b([a-zA-Z_][a-zA-Z0-9_]*)\.$/);
          if (dotMatch) {
            const ident = dotMatch[1];
            const mod = modulesIndex.get(ident);
            if (mod) {
              for (const exp of mod.exports) {
                const item = new vscode.CompletionItem(
                  exp,
                  vscode.CompletionItemKind.Method
                );
                item.detail = `method of ${ident}`;
                item.documentation = new vscode.MarkdownString(
                  `Exported function ${exp} from module ${ident}.`
                );
                items.push(item);
              }
              return items;
            }
          }

          // General: keywords, builtins, types
          items.push(
            ...keywords.map((k) =>
              toCompletionItem(k, vscode.CompletionItemKind.Keyword)
            )
          );
          items.push(
            ...builtins.map((b) =>
              toCompletionItem(b, vscode.CompletionItemKind.Function)
            )
          );
          items.push(
            ...types.map((t) =>
              toCompletionItem(t, vscode.CompletionItemKind.Class)
            )
          );
          return items;
        },
      },
      ".",
      " ",
      '"'
    )
  );

  context.subscriptions.push(
    vscode.languages.registerHoverProvider(
      { language: "fenics" },
      {
        provideHover(document, position) {
          const wordRange = document.getWordRangeAtPosition(position);
          if (!wordRange) return null;
          const word = document.getText(wordRange);

          // Ephemeral variable tooltip: #name
          const lineText = document.lineAt(position.line).text;
          const beforeWord = lineText.substring(0, wordRange.start.character);
          if (beforeWord.endsWith("#")) {
            return new vscode.Hover(
              new vscode.MarkdownString(`**#${word}** — ephemeral variable`)
            );
          }

          const k = keywords.find((k) => k.label === word);
          if (k)
            return new vscode.Hover(
              new vscode.MarkdownString(`**${word}** — ${k.doc}`)
            );

          const b = builtins.find((b) => b.label === word);
          if (b) {
            const md = new vscode.MarkdownString();
            md.appendMarkdown(`**${word}()** — ${b.doc}`);
            const signatures = {
              print: "print(value)",
              input: "input(prompt?) -> String",
              len: "len(value) -> Int",
              type: "type(value) -> String",
              str: "str(value) -> String",
              int: "int(value) -> Int",
              sort: "sort(array) -> Array",
              reverse: "reverse(array) -> Array",
              has: "has(collection, keyOrValue) -> Boolean",
              keys: "keys(object) -> Array",
              first: "first(array) -> Any",
              last: "last(array) -> Any",
              split: "split(text, delim) -> Array",
              length: "length(text) -> Int",
            };
            const sig = signatures[word];
            if (sig) md.appendCodeblock(sig, "fenics");
            return new vscode.Hover(md);
          }

          const t = types.find((t) => t.label === word);
          if (t)
            return new vscode.Hover(
              new vscode.MarkdownString(`**${word}** — ${t.doc}`)
            );

          // Hover for module names in import statements
          const line = lineText;
          if (/\bimport\b/.test(line) && modulesIndex.has(word)) {
            const mod = modulesIndex.get(word);
            return new vscode.Hover(
              new vscode.MarkdownString(
                `**module ${word}** — exports: ${Array.from(mod.exports).join(
                  ", "
                )}`
              )
            );
          }

          // Hover for methods: module.method
          const before = line.substring(0, wordRange.start.character);
          const moduleMatch = before.match(/\b([a-zA-Z_][a-zA-Z0-9_]*)\.$/);
          if (moduleMatch) {
            const modName = moduleMatch[1];
            const mod = modulesIndex.get(modName);
            if (mod && mod.exports.has(word)) {
              return new vscode.Hover(
                new vscode.MarkdownString(
                  `**${modName}.${word}()** — exported function from ${modName}`
                )
              );
            }
          }

          // Variable declarations and parameters
          const varIndex = indexVariablesFromText(document.getText());
          if (varIndex.has(word)) {
            const info = varIndex.get(word);
            let title = `**${word}**`;
            let kindLabel = "";
            switch (info.kind) {
              case "const":
                kindLabel = "constant variable";
                break;
              case "var":
                kindLabel = "mutable variable";
                break;
              case "global":
                kindLabel = "global variable";
                break;
              case "parameter":
                kindLabel = `function parameter (fn ${info.fn})`;
                break;
            }
            const declaredTypeLabel = info.declaredType
              ? `\nType: ${info.declaredType}`
              : "";
            const inferredTypeLabel = info.inferredType
              ? `\nImplicit type: ${info.inferredType}`
              : "";
            const valuePreviewLabel = info.valuePreview
              ? `\nValue: ${info.valuePreview}`
              : "";
            const objectKeysLabel =
              info.objectKeys && info.objectKeys.length
                ? `\nKeys: ${info.objectKeys.slice(0, 5).join(", ")}${
                    info.objectKeys.length > 5 ? " …" : ""
                  }`
                : "";
            const declaredAt = info.line
              ? `\nDeclared at line ${info.line}`
              : "";
            return new vscode.Hover(
              new vscode.MarkdownString(
                `${title} — ${kindLabel}${declaredTypeLabel}${inferredTypeLabel}${valuePreviewLabel}${objectKeysLabel}${declaredAt}`
              )
            );
          }

          // Function hover: show signature
          const fnIndex = indexFunctionsFromText(document.getText());
          if (fnIndex.has(word)) {
            const f = fnIndex.get(word);
            const sig = `fn ${word}(${f.params.join(", ")})${
              f.returnType ? " -> " + f.returnType : ""
            }`;
            const md = new vscode.MarkdownString();
            md.appendMarkdown(`**${word}** — function`);
            md.appendCodeblock(sig, "fenics");
            md.appendMarkdown(`\nDeclared at line ${f.line}`);
            return new vscode.Hover(md);
          }

          // Import alias hover: show module info
          const aliasMap = indexImportAliases(document.getText());
          if (aliasMap.has(word)) {
            const target = aliasMap.get(word);
            const mod = modulesIndex.get(target);
            if (mod) {
              return new vscode.Hover(
                new vscode.MarkdownString(
                  `**alias ${word} → ${target}** — exports: ${Array.from(
                    mod.exports
                  ).join(", ")}`
                )
              );
            }
          }

          return null;
        },
      }
    )
  );
}

function deactivate() {}

module.exports = { activate, deactivate };
