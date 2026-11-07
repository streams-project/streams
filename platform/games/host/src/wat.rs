//! Minimal WebAssembly text format (`.wat`) encoder for small game modules and tests.

use crate::GameError;

#[derive(Clone, PartialEq, Eq)]
struct Sig {
    params: Vec<u8>,
    results: Vec<u8>,
}

struct FuncDef {
    export: Option<String>,
    sig: Sig,
    body: Vec<u8>,
}

struct ImportDef {
    module: String,
    name: String,
    sig: Sig,
}

struct MemoryDef {
    export: Option<String>,
    min: u64,
    max: Option<u64>,
}

pub(crate) fn maybe_assemble(bytes: &[u8]) -> Result<Vec<u8>, GameError> {
    if bytes.starts_with(b"\0asm") {
        return Ok(bytes.to_vec());
    }
    let Ok(text) = std::str::from_utf8(bytes) else {
        return Ok(bytes.to_vec());
    };
    let trimmed = strip_comments(text);
    if !trimmed.trim_start().starts_with("(module") {
        return Ok(bytes.to_vec());
    }
    assemble_wat(&trimmed).ok_or(GameError::InvalidModule)
}

fn strip_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for line in text.lines() {
        let clean = match line.find(";;") {
            Some(idx) => &line[..idx],
            None => line,
        };
        out.push_str(clean);
        out.push('\n');
    }
    out
}

fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut chars = text.chars().peekable();
    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() {
            chars.next();
        } else if ch == '(' || ch == ')' {
            tokens.push(ch.to_string());
            chars.next();
        } else if ch == '"' {
            let mut s = String::new();
            s.push(chars.next().unwrap_or('"'));
            while let Some(c) = chars.next() {
                s.push(c);
                if c == '"' {
                    break;
                }
            }
            tokens.push(s);
        } else {
            let mut word = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_whitespace() || c == '(' || c == ')' {
                    break;
                }
                word.push(c);
                chars.next();
            }
            tokens.push(word);
        }
    }
    tokens
}

#[derive(Clone)]
enum Node {
    Atom(String),
    List(Vec<Node>),
}

fn parse_nodes(tokens: &[String], pos: &mut usize) -> Option<Vec<Node>> {
    let mut list = Vec::new();
    while *pos < tokens.len() {
        let tok = &tokens[*pos];
        *pos += 1;
        if tok == ")" {
            return Some(list);
        } else if tok == "(" {
            list.push(Node::List(parse_nodes(tokens, pos)?));
        } else {
            list.push(Node::Atom(tok.clone()));
        }
    }
    Some(list)
}

fn val_type(name: &str) -> Option<u8> {
    match name {
        "i32" => Some(0x7F),
        "i64" => Some(0x7E),
        "f32" => Some(0x7D),
        "f64" => Some(0x7C),
        _ => None,
    }
}

fn unquote(s: &str) -> Option<String> {
    s.strip_prefix('"')?.strip_suffix('"').map(str::to_owned)
}

fn encode_u64_leb(mut val: u64, out: &mut Vec<u8>) {
    loop {
        let mut byte = (val & 0x7F) as u8;
        val >>= 7;
        if val != 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if val == 0 {
            break;
        }
    }
}

fn encode_i32_leb(mut val: i32, out: &mut Vec<u8>) {
    loop {
        let byte = (val & 0x7F) as u8;
        val >>= 7;
        let done = (val == 0 && (byte & 0x40) == 0) || (val == -1 && (byte & 0x40) != 0);
        if done {
            out.push(byte);
            break;
        }
        out.push(byte | 0x80);
    }
}

fn encode_str(s: &str, out: &mut Vec<u8>) {
    encode_u64_leb(s.len() as u64, out);
    out.extend_from_slice(s.as_bytes());
}

fn encode_section(id: u8, payload: &[u8], out: &mut Vec<u8>) {
    out.push(id);
    encode_u64_leb(payload.len() as u64, out);
    out.extend_from_slice(payload);
}

fn compile_instrs(nodes: &[Node], body: &mut Vec<u8>) -> Option<()> {
    let mut i = 0;
    while i < nodes.len() {
        match &nodes[i] {
            Node::Atom(op) => match op.as_str() {
                "nop" => {
                    body.push(0x01);
                    i += 1;
                }
                "unreachable" => {
                    body.push(0x00);
                    i += 1;
                }
                "return" => {
                    body.push(0x0F);
                    i += 1;
                }
                "drop" => {
                    body.push(0x1A);
                    i += 1;
                }
                "i32.const" => {
                    let Node::Atom(arg) = nodes.get(i + 1)? else {
                        return None;
                    };
                    let n: i32 = arg.parse().ok()?;
                    body.push(0x41);
                    encode_i32_leb(n, body);
                    i += 2;
                }
                "br" => {
                    let Node::Atom(target) = nodes.get(i + 1)? else {
                        return None;
                    };
                    let depth: u64 = if target.starts_with('$') {
                        0
                    } else {
                        target.parse().ok()?
                    };
                    body.push(0x0C);
                    encode_u64_leb(depth, body);
                    i += 2;
                }
                "loop" => {
                    body.push(0x03);
                    body.push(0x40);
                    i += 1;
                    if let Some(Node::Atom(lbl)) = nodes.get(i)
                        && lbl.starts_with('$')
                    {
                        i += 1;
                    }
                }
                "end" => {
                    body.push(0x0B);
                    i += 1;
                }
                _ => return None,
            },
            Node::List(items) => {
                let Node::Atom(head) = items.first()? else {
                    return None;
                };
                match head.as_str() {
                    "loop" => {
                        body.push(0x03);
                        body.push(0x40);
                        let mut start = 1;
                        if let Some(Node::Atom(lbl)) = items.get(1)
                            && lbl.starts_with('$')
                        {
                            start = 2;
                        }
                        compile_instrs(&items[start..], body)?;
                        body.push(0x0B);
                    }
                    "i32.const" => {
                        let Node::Atom(arg) = items.get(1)? else {
                            return None;
                        };
                        let n: i32 = arg.parse().ok()?;
                        body.push(0x41);
                        encode_i32_leb(n, body);
                    }
                    "br" => {
                        let Node::Atom(target) = items.get(1)? else {
                            return None;
                        };
                        let depth: u64 = if target.starts_with('$') {
                            0
                        } else {
                            target.parse().ok()?
                        };
                        body.push(0x0C);
                        encode_u64_leb(depth, body);
                    }
                    "nop" => body.push(0x01),
                    "unreachable" => body.push(0x00),
                    _ => return None,
                }
                i += 1;
            }
        }
    }
    Some(())
}

fn parse_sig_and_export(items: &[Node]) -> Option<(Option<String>, Sig, usize)> {
    let mut export = None;
    let mut params = Vec::new();
    let mut results = Vec::new();
    let mut idx = 1;
    if let Some(Node::Atom(name)) = items.get(idx)
        && name.starts_with('$')
    {
        idx += 1;
    }
    while idx < items.len() {
        let Node::List(sub) = &items[idx] else {
            break;
        };
        let Some(Node::Atom(tag)) = sub.first() else {
            break;
        };
        match tag.as_str() {
            "export" => {
                let Node::Atom(raw) = sub.get(1)? else {
                    return None;
                };
                export = Some(unquote(raw)?);
                idx += 1;
            }
            "param" => {
                for p in &sub[1..] {
                    let Node::Atom(w) = p else {
                        return None;
                    };
                    if w.starts_with('$') {
                        continue;
                    }
                    params.push(val_type(w)?);
                }
                idx += 1;
            }
            "result" => {
                for r in &sub[1..] {
                    let Node::Atom(w) = r else {
                        return None;
                    };
                    results.push(val_type(w)?);
                }
                idx += 1;
            }
            _ => break,
        }
    }
    Some((export, Sig { params, results }, idx))
}

fn assemble_wat(text: &str) -> Option<Vec<u8>> {
    let tokens = tokenize(text);
    let mut pos = 0;
    let top = parse_nodes(&tokens, &mut pos)?;
    let Some(Node::List(module_items)) = top.first() else {
        return None;
    };
    let Some(Node::Atom(head)) = module_items.first() else {
        return None;
    };
    if head != "module" {
        return None;
    }

    let mut types: Vec<Sig> = Vec::new();
    let mut imports: Vec<ImportDef> = Vec::new();
    let mut funcs: Vec<FuncDef> = Vec::new();
    let mut memory: Option<MemoryDef> = None;

    let intern_sig = |sig: &Sig, types: &mut Vec<Sig>| -> u32 {
        if let Some(idx) = types.iter().position(|t| t == sig) {
            idx as u32
        } else {
            types.push(sig.clone());
            (types.len() - 1) as u32
        }
    };

    for item in &module_items[1..] {
        let Node::List(parts) = item else {
            return None;
        };
        let Node::Atom(kind) = parts.first()? else {
            return None;
        };
        match kind.as_str() {
            "import" => {
                let Node::Atom(m_raw) = parts.get(1)? else {
                    return None;
                };
                let Node::Atom(n_raw) = parts.get(2)? else {
                    return None;
                };
                let Node::List(desc) = parts.get(3)? else {
                    return None;
                };
                let Node::Atom(desc_kind) = desc.first()? else {
                    return None;
                };
                if desc_kind != "func" {
                    return None;
                }
                let (_, sig, _) = parse_sig_and_export(desc)?;
                intern_sig(&sig, &mut types);
                imports.push(ImportDef {
                    module: unquote(m_raw)?,
                    name: unquote(n_raw)?,
                    sig,
                });
            }
            "memory" => {
                let mut idx = 1;
                let mut export = None;
                if let Some(Node::Atom(name)) = parts.get(idx)
                    && name.starts_with('$')
                {
                    idx += 1;
                }
                if let Some(Node::List(sub)) = parts.get(idx) {
                    let Node::Atom(tag) = sub.first()? else {
                        return None;
                    };
                    if tag == "export" {
                        let Node::Atom(raw) = sub.get(1)? else {
                            return None;
                        };
                        export = Some(unquote(raw)?);
                        idx += 1;
                    }
                }
                let Node::Atom(min_str) = parts.get(idx)? else {
                    return None;
                };
                let min: u64 = min_str.parse().ok()?;
                let max = if let Some(Node::Atom(max_str)) = parts.get(idx + 1) {
                    Some(max_str.parse().ok()?)
                } else {
                    None
                };
                memory = Some(MemoryDef { export, min, max });
            }
            "func" => {
                let (export, sig, body_start) = parse_sig_and_export(parts)?;
                intern_sig(&sig, &mut types);
                let mut body = Vec::new();
                compile_instrs(&parts[body_start..], &mut body)?;
                body.push(0x0B);
                funcs.push(FuncDef { export, sig, body });
            }
            _ => return None,
        }
    }

    let mut out = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];

    if !types.is_empty() {
        let mut sec = Vec::new();
        encode_u64_leb(types.len() as u64, &mut sec);
        for sig in &types {
            sec.push(0x60);
            encode_u64_leb(sig.params.len() as u64, &mut sec);
            sec.extend_from_slice(&sig.params);
            encode_u64_leb(sig.results.len() as u64, &mut sec);
            sec.extend_from_slice(&sig.results);
        }
        encode_section(1, &sec, &mut out);
    }

    if !imports.is_empty() {
        let mut sec = Vec::new();
        encode_u64_leb(imports.len() as u64, &mut sec);
        for imp in &imports {
            encode_str(&imp.module, &mut sec);
            encode_str(&imp.name, &mut sec);
            sec.push(0x00);
            let tidx = types.iter().position(|t| t == &imp.sig)? as u64;
            encode_u64_leb(tidx, &mut sec);
        }
        encode_section(2, &sec, &mut out);
    }

    if !funcs.is_empty() {
        let mut sec = Vec::new();
        encode_u64_leb(funcs.len() as u64, &mut sec);
        for f in &funcs {
            let tidx = types.iter().position(|t| t == &f.sig)? as u64;
            encode_u64_leb(tidx, &mut sec);
        }
        encode_section(3, &sec, &mut out);
    }

    if let Some(mem) = &memory {
        let mut sec = Vec::new();
        encode_u64_leb(1, &mut sec);
        match mem.max {
            Some(max) => {
                sec.push(0x01);
                encode_u64_leb(mem.min, &mut sec);
                encode_u64_leb(max, &mut sec);
            }
            None => {
                sec.push(0x00);
                encode_u64_leb(mem.min, &mut sec);
            }
        }
        encode_section(5, &sec, &mut out);
    }

    let mut exports: Vec<(String, u8, u64)> = Vec::new();
    if let Some(mem) = &memory
        && let Some(name) = &mem.export
    {
        exports.push((name.clone(), 0x02, 0));
    }
    let imported_funcs = imports.len() as u64;
    for (idx, f) in funcs.iter().enumerate() {
        if let Some(name) = &f.export {
            exports.push((name.clone(), 0x00, imported_funcs + (idx as u64)));
        }
    }
    if !exports.is_empty() {
        let mut sec = Vec::new();
        encode_u64_leb(exports.len() as u64, &mut sec);
        for (name, kind, idx) in &exports {
            encode_str(name, &mut sec);
            sec.push(*kind);
            encode_u64_leb(*idx, &mut sec);
        }
        encode_section(7, &sec, &mut out);
    }

    if !funcs.is_empty() {
        let mut sec = Vec::new();
        encode_u64_leb(funcs.len() as u64, &mut sec);
        for f in &funcs {
            let mut body = Vec::new();
            encode_u64_leb(0, &mut body);
            body.extend_from_slice(&f.body);
            encode_u64_leb(body.len() as u64, &mut sec);
            sec.extend_from_slice(&body);
        }
        encode_section(10, &sec, &mut out);
    }

    Some(out)
}
