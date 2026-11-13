//! `doctor` looks at the database, or says out loud that it could not.
//!
//! On 2026-11-13 `up` stopped with `migration 130 was previously applied but has been modified` and
//! `doctor`, run one second later against the same install, printed eight `ok` lines and exited 0.
//! It checked four file paths, root, a manifest and a pack count, and never opened the database.
//!
//! Every test here runs the real program against a home of its own, with a database that is
//! missing, unreachable, out of step, or in step. None of them touches an install of anyone's.

use std::path::{Path, PathBuf};
use std::process::Command;

use launcher::migrations;

fn scratch(name: &str) -> PathBuf {
    let at = std::env::temp_dir().join(format!("doctor-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(&at).expect("a folder");
    at
}

/// A home with a database in it that no program will ever really start.
fn home_with_a_database(name: &str) -> PathBuf {
    let home = scratch(name);
    let data = home.join("database/data");
    std::fs::create_dir_all(&data).expect("a data folder");
    std::fs::write(data.join("PG_VERSION"), "18\n").expect("a version file");
    std::fs::write(home.join("database/database.password"), "secret\n").expect("a password");
    home
}

fn runnable(path: &Path, script: &str) {
    std::fs::write(path, script).expect("a script");
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).expect("made runnable");
}

/// A `bin` folder that looks like a Postgres install and is not one: `pg_ctl` answers as told and
/// `psql` answers with the rows given here.
fn fake_bin(home: &Path, start_ok: bool, rows: &str) -> PathBuf {
    let bin = home.join("fake-bin");
    std::fs::create_dir_all(&bin).expect("a bin folder");
    std::fs::write(bin.join("postgres"), "").expect("the marker the finder looks for");
    let data = home.join("database/data");
    runnable(
        &bin.join("pg_ctl"),
        &format!(
            "#!/bin/sh\nfor word in \"$@\"; do\n  if [ \"$word\" = start ]; then\n    {}\n    exit \
             {}\n  fi\ndone\nexit 0\n",
            if start_ok {
                format!(
                    "printf '%s\\n' {} > {}",
                    std::process::id(),
                    data.join("postmaster.pid").display()
                )
            } else {
                String::from(":")
            },
            u8::from(!start_ok),
        ),
    );
    runnable(
        &bin.join("psql"),
        &format!(
            "#!/bin/sh\nfor word in \"$@\"; do\n  case \"$word\" in\n    \
             *to_regclass*) echo t; exit 0 ;;\n    *_sqlx_migrations*) printf '{rows}'; exit 0 \
             ;;\n  esac\ndone\nexit 0\n"
        ),
    );
    bin
}

/// The program's own `doctor`, over one home, with whatever database program it should find.
fn doctor(home: &Path, database_bin: Option<&Path>) -> (bool, String) {
    let mut run = Command::new(env!("CARGO_BIN_EXE_launcher"));
    run.arg("doctor").env("STACK_HOME", home);
    match database_bin {
        Some(bin) => run.env("DATABASE_BIN", bin),
        None => run.env_remove("DATABASE_BIN"),
    };
    let output = run.output().expect("the program ran");
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    (output.status.success(), said)
}

/// The line about the database, whatever else was printed.
fn database_line(said: &str) -> String {
    said.lines()
        .find(|line| line.contains("database:"))
        .unwrap_or_else(|| panic!("nothing was said about the database:\n{said}"))
        .to_owned()
}

/// One row per migration file of this copy, the way the runner would have recorded it.
///
/// They go into a shell `printf`, so the line breaks are written and not taken.
fn in_step() -> Vec<String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    migrations::steps(&migrations::folder(&root))
        .expect("this copy's migrations")
        .iter()
        .map(|step| format!("{} {}", step.version, step.checksum))
        .collect()
}

fn rows(lines: &[String]) -> String {
    lines
        .iter()
        .map(|line| format!("{line}\\n"))
        .collect::<Vec<String>>()
        .concat()
}

/// A fresh home has nothing to be wrong with, and saying so is not the same as saying nothing.
#[test]
fn a_home_with_no_database_yet_is_an_answer_and_not_a_silence() {
    let home = scratch("fresh");
    let (_, said) = doctor(&home, None);
    let line = database_line(&said);
    assert!(line.starts_with("ok"), "{line}");
    assert!(line.contains("none yet"), "{line}");
    assert!(!said.contains("COULD NOT CHECK"), "{said}");
    std::fs::remove_dir_all(&home).expect("cleaned");
}

/// The database is there and nothing on this machine can open it. The check that cannot be made
/// is a failure, which is the rule `check_types.py` states for a language it cannot compile.
#[test]
fn a_database_with_no_program_to_open_it_is_a_problem_and_not_a_pass() {
    let home = home_with_a_database("no-program");
    let (good, said) = doctor(&home, None);
    let line = database_line(&said);
    assert!(line.starts_with("PROBLEM"), "{line}");
    assert!(line.contains("COULD NOT CHECK"), "{line}");
    assert!(
        !good,
        "doctor passed an install it could not look at:\n{said}"
    );
    std::fs::remove_dir_all(&home).expect("cleaned");
}

/// The database is unreachable: it will not come up. Before today this printed `ok` eight times.
#[test]
fn a_database_that_will_not_come_up_is_said_so_and_the_exit_is_not_zero() {
    let home = home_with_a_database("will-not-start");
    let bin = fake_bin(&home, false, "");
    let (good, said) = doctor(&home, Some(&bin));
    let line = database_line(&said);
    assert!(line.contains("COULD NOT CHECK"), "{line}");
    assert!(line.contains("would not come up"), "{line}");
    assert!(!good, "{said}");
    std::fs::remove_dir_all(&home).expect("cleaned");
}

/// Defect B as a person meets it: a migration applied to this database from a file that has
/// changed since. `up` stops on it, so `doctor` has to be able to name it.
#[test]
fn a_migration_applied_from_a_file_that_has_changed_is_named() {
    let home = home_with_a_database("drifted");
    // Version 0 applied, with a checksum that is not the file's.
    let bin = fake_bin(&home, true, "0 deadbeef\\n");
    let (good, said) = doctor(&home, Some(&bin));
    assert!(said.contains("0000_auth.sql"), "{said}");
    assert!(said.contains("has changed since"), "{said}");
    assert!(!good, "{said}");
    std::fs::remove_dir_all(&home).expect("cleaned");
}

/// And the same path with a database that agrees with this copy, so none of the tests above can
/// pass by a `doctor` that calls everything a problem.
#[test]
fn a_database_in_step_with_this_copy_is_asked_and_passes() {
    let home = home_with_a_database("in-step");
    let bin = fake_bin(&home, true, &rows(&in_step()));
    let (_, said) = doctor(&home, Some(&bin));
    let line = database_line(&said);
    assert!(line.starts_with("ok"), "{line}");
    assert!(line.contains("asked it"), "{line}");
    assert!(!said.contains("COULD NOT CHECK"), "{said}");
    assert!(!line.contains("waiting"), "every file is applied: {line}");
    std::fs::remove_dir_all(&home).expect("cleaned");
}

/// A copy of the program sitting beside a brand file of this test's own, which is what it reads
/// its own repository as.
fn copy_beside(name: &str, brand: &serde_json::Value) -> PathBuf {
    let root = scratch(name);
    std::fs::write(root.join("brand.json"), brand.to_string()).expect("a brand file");
    let program = root.join("program");
    std::fs::copy(env!("CARGO_BIN_EXE_launcher"), &program).expect("the program");
    program
}

fn brand_file() -> serde_json::Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../brand.json");
    serde_json::from_str(&std::fs::read_to_string(path).expect("the brand file")).expect("json")
}

/// Defect 0. `spacing.gutter` left the brand file on 2026-11-12 and the installed program, built
/// the day before, answered every command with `missing field gutter` and exit 1 -- `doctor`
/// included, so the command a person runs to find out what is wrong was the first thing the
/// change took away, and the stack had no way to start.
///
/// A program a key out of step with its brand file now says which key, says the two are different
/// ages, and goes on checking everything else it can still check.
#[test]
fn a_brand_file_of_another_age_does_not_take_doctor_down_with_it() {
    let mut older = brand_file();
    older["spacing"]
        .as_object_mut()
        .expect("the spacing group")
        .remove("unit");
    let program = copy_beside("older-brand", &older);
    let home = scratch("older-brand-home");
    let output = Command::new(&program)
        .arg("doctor")
        .env("STACK_HOME", &home)
        .output()
        .expect("the program ran");
    let said = String::from_utf8_lossy(&output.stdout).into_owned()
        + &String::from_utf8_lossy(&output.stderr);
    assert!(said.contains("different ages"), "{said}");
    assert!(said.contains("spacing.unit"), "{said}");
    assert!(
        said.lines().count() > 3,
        "doctor stopped at the brand file instead of carrying on:\n{said}"
    );
    assert!(
        !output.status.success(),
        "doctor passed an install whose brand file it cannot read:\n{said}"
    );
    std::fs::remove_dir_all(program.parent().expect("the folder")).expect("cleaned");
    std::fs::remove_dir_all(&home).expect("cleaned");
}

/// The same file, and the program can still say what its commands are. Before today a brand file
/// one key out of step meant no command at all, not even this one.
#[test]
fn a_brand_file_of_another_age_still_leaves_the_program_able_to_speak() {
    let mut newer = brand_file();
    newer["spacing"]["gutter"] = 16.into();
    let program = copy_beside("newer-brand", &newer);
    let home = scratch("newer-brand-home");
    let output = Command::new(&program)
        .env("STACK_HOME", &home)
        .output()
        .expect("the program ran");
    let said = String::from_utf8_lossy(&output.stdout).into_owned();
    assert!(output.status.success(), "help did not run: {said}");
    assert!(said.contains("doctor"), "{said}");
    assert!(said.contains("up ["), "{said}");
    // And the command that needs the look says why it will not run, naming the key.
    let refused = Command::new(&program)
        .arg("install")
        .env("STACK_HOME", &home)
        .output()
        .expect("the program ran");
    let why = String::from_utf8_lossy(&refused.stderr).into_owned();
    assert!(!refused.status.success(), "install ran anyway: {why}");
    assert!(why.contains("spacing.gutter"), "{why}");
    std::fs::remove_dir_all(program.parent().expect("the folder")).expect("cleaned");
    std::fs::remove_dir_all(&home).expect("cleaned");
}

/// A database behind this copy is not a fault: those files are what the next `up` is for.
#[test]
fn files_not_applied_yet_are_counted_and_are_not_a_problem() {
    let home = home_with_a_database("behind");
    let bin = fake_bin(&home, true, &rows(&in_step()[..1]));
    let (_, said) = doctor(&home, Some(&bin));
    let line = database_line(&said);
    assert!(line.starts_with("ok"), "{line}");
    assert!(line.contains("waiting for the next up"), "{line}");
    std::fs::remove_dir_all(&home).expect("cleaned");
}
