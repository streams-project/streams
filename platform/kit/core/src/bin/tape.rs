//! Writes the tape and transcript of one session, so a host in another language can run it.

use std::fs;
use std::path::Path;

use kit_core::host::{Mode, run};
use kit_core::sessions::{common, scripts};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [name, layer, folder] = args.as_slice() else {
        panic!("usage: tape <session name> <on|off> <folder>");
    };
    let mode = match layer.as_str() {
        "on" => Mode::LayerOn,
        "off" => Mode::LayerOff,
        other => panic!("the layer is on or off, not {other}"),
    };
    let scripts = scripts();
    let script = scripts
        .iter()
        .find(|script| script.name == *name)
        .unwrap_or_else(|| panic!("no session named {name}"));
    let report = run(script, &common(), mode).expect("the session runs");
    let folder = Path::new(folder);
    fs::create_dir_all(folder).expect("the folder can be made");
    fs::write(folder.join("session.tape"), report.tape).expect("the tape can be written");
    fs::write(folder.join("transcript.txt"), report.transcript).expect("it can be written");
}
