//! Walks WebAssembly code and names every feature it finds beyond the first version of the
//! format. Reads archives, objects and linked modules, so the scan sees compiled code.

use std::collections::BTreeSet;

/// One thing a module needs beyond the first version.
pub type Feature = String;

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, at: 0 }
    }

    const fn done(&self) -> bool {
        self.at >= self.bytes.len()
    }

    fn byte(&mut self) -> Result<u8, String> {
        let byte = *self.bytes.get(self.at).ok_or("ran out of bytes")?;
        self.at += 1;
        Ok(byte)
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8], String> {
        let end = self.at.checked_add(len).ok_or("length overflow")?;
        let part = self.bytes.get(self.at..end).ok_or("ran out of bytes")?;
        self.at = end;
        Ok(part)
    }

    fn unsigned(&mut self) -> Result<u64, String> {
        let mut value = 0u64;
        let mut shift = 0;
        loop {
            let byte = self.byte()?;
            value |= u64::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                return Ok(value);
            }
            shift += 7;
            if shift > 63 {
                return Err("number too long".to_string());
            }
        }
    }

    fn size(&mut self) -> Result<usize, String> {
        usize::try_from(self.unsigned()?).map_err(|_| "size too large".to_string())
    }

    /// Skip a signed number. A block type is one, so its first byte says what it is.
    fn skip_signed(&mut self) -> Result<(), String> {
        while self.byte()? & 0x80 != 0 {}
        Ok(())
    }
}

fn block_type(reader: &mut Reader<'_>, found: &mut BTreeSet<Feature>) -> Result<(), String> {
    let first = *reader.bytes.get(reader.at).ok_or("ran out of bytes")?;
    match first {
        0x40 | 0x7c..=0x7f => {
            reader.at += 1;
        }
        _ => {
            found.insert("multi-value".to_string());
            reader.skip_signed()?;
        }
    }
    Ok(())
}

fn instructions(body: &[u8], found: &mut BTreeSet<Feature>) -> Result<(), String> {
    let mut reader = Reader::new(body);
    while !reader.done() {
        let code = reader.byte()?;
        match code {
            0x00 | 0x01 | 0x05 | 0x0b | 0x0f | 0x1a | 0x1b | 0x45..=0xbf => {}
            0x02..=0x04 => block_type(&mut reader, found)?,
            0x0c | 0x0d | 0x10 | 0x20..=0x24 => {
                reader.unsigned()?;
            }
            0x0e => {
                let count = reader.size()?;
                for _ in 0..=count {
                    reader.unsigned()?;
                }
            }
            0x11 => {
                reader.unsigned()?;
                reader.unsigned()?;
            }
            0x28..=0x3e => {
                reader.unsigned()?;
                reader.unsigned()?;
            }
            0x3f | 0x40 => {
                reader.byte()?;
            }
            0x41 | 0x42 => reader.skip_signed()?,
            0x43 => {
                reader.take(4)?;
            }
            0x44 => {
                reader.take(8)?;
            }
            0xc0..=0xc4 => {
                found.insert("sign-ext".to_string());
            }
            0x06..=0x09 | 0x18 | 0x19 | 0x1f => return stop(found, "exception-handling"),
            0x12 | 0x13 => return stop(found, "tail-call"),
            0x1c | 0x25 | 0x26 | 0xd0..=0xd2 => return stop(found, "reference-types"),
            0xfc => {
                let sub = reader.unsigned()?;
                let name = if sub < 8 {
                    "nontrapping-fptoint"
                } else {
                    "bulk-memory"
                };
                return stop(found, name);
            }
            0xfd => return stop(found, "simd128"),
            0xfe => return stop(found, "atomics"),
            other => return stop(found, &format!("unknown opcode {other:#x}")),
        }
    }
    Ok(())
}

/// Once an instruction past the first version appears, its immediates are unknown to this walk,
/// so the walk of that function stops. The feature has been named, and that fails the gate.
fn stop(found: &mut BTreeSet<Feature>, name: &str) -> Result<(), String> {
    found.insert(name.to_string());
    Ok(())
}

fn types(section: &[u8], found: &mut BTreeSet<Feature>) -> Result<(), String> {
    let mut reader = Reader::new(section);
    for _ in 0..reader.size()? {
        reader.byte()?;
        let params = reader.size()?;
        reader.take(params)?;
        let results = reader.size()?;
        reader.take(results)?;
        if results > 1 {
            found.insert("multi-value".to_string());
        }
    }
    Ok(())
}

fn code(section: &[u8], found: &mut BTreeSet<Feature>) -> Result<(), String> {
    let mut reader = Reader::new(section);
    for _ in 0..reader.size()? {
        let size = reader.size()?;
        let mut body = Reader::new(reader.take(size)?);
        for _ in 0..body.size()? {
            body.unsigned()?;
            body.byte()?;
        }
        instructions(&body.bytes[body.at..], found)?;
    }
    Ok(())
}

fn declared(section: &[u8], found: &mut BTreeSet<Feature>) -> Result<(), String> {
    let mut reader = Reader::new(section);
    let count = reader.size()?;
    for _ in 0..count {
        let prefix = reader.byte()?;
        let len = reader.size()?;
        let name = String::from_utf8_lossy(reader.take(len)?).into_owned();
        if prefix != b'-' {
            found.insert(name);
        }
    }
    Ok(())
}

/// Scan one object or linked module.
///
/// # Errors
/// Bytes that do not read as WebAssembly.
pub fn scan_module(bytes: &[u8], found: &mut BTreeSet<Feature>) -> Result<(), String> {
    let mut reader = Reader::new(bytes);
    if reader.take(8)? != b"\0asm\x01\0\0\0" {
        return Err("not a module".to_string());
    }
    while !reader.done() {
        let id = reader.byte()?;
        let size = reader.size()?;
        let section = reader.take(size)?;
        match id {
            0 => {
                let mut custom = Reader::new(section);
                let len = custom.size()?;
                let name = custom.take(len)?;
                if name == b"target_features" {
                    declared(&section[custom.at..], found)?;
                }
            }
            1 => types(section, found)?,
            10 => code(section, found)?,
            _ => {}
        }
    }
    Ok(())
}

/// Scan an archive of objects, or a single module. Returns how many held code.
///
/// # Errors
/// Bytes that read as neither.
pub fn scan_file(bytes: &[u8], found: &mut BTreeSet<Feature>) -> Result<usize, String> {
    if !bytes.starts_with(b"!<arch>\n") {
        scan_module(bytes, found)?;
        return Ok(1);
    }
    let mut at = 8;
    let mut modules = 0;
    while at + 60 <= bytes.len() {
        let size: usize = std::str::from_utf8(&bytes[at + 48..at + 58])
            .map_err(|_| "bad archive header")?
            .trim()
            .parse()
            .map_err(|_| "bad archive size")?;
        let member = bytes
            .get(at + 60..at + 60 + size)
            .ok_or("archive ran out of bytes")?;
        if member.starts_with(b"\0asm") {
            scan_module(member, found)?;
            modules += 1;
        }
        at += 60 + size + (size & 1);
    }
    Ok(modules)
}
