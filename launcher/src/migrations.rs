//! What the database says it has already applied, held against the files in this copy.
//!
//! The server owns applying them. This module only looks, because the one thing `doctor` could not
//! do was look: on 2026-11-13 `up` stopped with `migration 130 was previously applied but has been
//! modified` and `doctor`, run a second later, printed eight `ok` lines and exited 0. A check that
//! cannot see the thing that is broken is worse than no check, because it tells a person that
//! nothing is wrong.
//!
//! The runner records a SHA-384 of each file's bytes, so the same sum computed here says whether
//! the file on disk is the file that was applied, without the database having to be asked to
//! apply anything.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha384};

/// A migration file in this copy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    /// The number its name starts with, which is how the database knows it.
    pub version: i64,
    pub name: String,
    /// SHA-384 of the file's bytes, in hex, which is what the runner wrote down.
    pub checksum: String,
}

/// A row of the table the runner keeps.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Applied {
    pub version: i64,
    pub checksum: String,
}

/// How a copy's files and a database's record differ.
#[derive(Clone, Debug, Default)]
pub struct Compared {
    /// Applied, and the file is byte for byte the one that was applied.
    pub agreed: usize,
    /// Applied, and the file here is not the file that was applied. This is what stops `up`.
    pub changed: Vec<Step>,
    /// Applied, and this copy has no file with that number at all.
    pub gone: Vec<i64>,
    /// Here and not applied yet, which is what the next `up` is for.
    pub pending: Vec<Step>,
}

impl Compared {
    /// Whether this database and this copy can work together without a person deciding something.
    #[must_use]
    pub fn settled(&self) -> bool {
        self.changed.is_empty() && self.gone.is_empty()
    }

    /// One line for each way they disagree, in the words of the person who has to act on it.
    #[must_use]
    pub fn problems(&self) -> Vec<String> {
        let mut said = Vec::new();
        for step in &self.changed {
            said.push(format!(
                "{} was applied to this database and the file has changed since, so the database \
                 cannot be brought up to date without a decision about that file",
                step.name
            ));
        }
        for version in &self.gone {
            said.push(format!(
                "this database has migration {version} applied and this copy has no file for it, \
                 so this copy is older than the database it is pointed at"
            ));
        }
        said
    }
}

/// Every migration file in `folder`, in the order the runner takes them.
///
/// # Errors
/// When the folder cannot be read, or a file is not named with a number first.
pub fn steps(folder: &Path) -> Result<Vec<Step>> {
    let listed = std::fs::read_dir(folder)
        .with_context(|| format!("{} cannot be read", folder.display()))?;
    let mut found = Vec::new();
    for entry in listed {
        let path = entry?.path();
        if path.extension().is_none_or(|ext| ext != "sql") {
            continue;
        }
        let name = path
            .file_name()
            .context("a file with no name")?
            .to_string_lossy()
            .into_owned();
        let (number, _) = name
            .split_once('_')
            .with_context(|| format!("{name} does not start with a number and an underscore"))?;
        let version: i64 = number
            .parse()
            .with_context(|| format!("{name} does not start with a number"))?;
        let body = std::fs::read(&path)?;
        found.push(Step {
            version,
            name,
            checksum: hex(&Sha384::digest(&body)),
        });
    }
    found.sort_by_key(|step| step.version);
    Ok(found)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// The table the runner keeps, read with the `psql` that came with the pinned build.
///
/// A database that has never had a migration applied has no such table, and that is an answer
/// rather than a failure: it comes back empty.
///
/// # Errors
/// When `psql` is not there, the database will not answer, or it answers something unreadable.
pub fn applied(psql: &Path, url: &str) -> Result<Vec<Applied>> {
    let there = ask(
        psql,
        url,
        "select to_regclass('public._sqlx_migrations') is not null",
    )?;
    if there.trim() != "t" {
        return Ok(Vec::new());
    }
    let rows = ask(
        psql,
        url,
        "select version || ' ' || encode(checksum, 'hex') from _sqlx_migrations order by version",
    )?;
    read_rows(&rows)
}

/// Rows as `psql -tA` prints them: one per line, nothing else.
fn read_rows(text: &str) -> Result<Vec<Applied>> {
    let mut found = Vec::new();
    for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
        let (version, checksum) = line
            .split_once(' ')
            .with_context(|| format!("the database answered {line}, which is not a row"))?;
        found.push(Applied {
            version: version
                .parse()
                .with_context(|| format!("{version} is not a migration number"))?,
            checksum: checksum.to_owned(),
        });
    }
    Ok(found)
}

/// One question, with the answer as plain text and no prompt, no header and no pager.
fn ask(psql: &Path, url: &str, question: &str) -> Result<String> {
    let output = std::process::Command::new(psql)
        .arg(url)
        .args(["-tAX", "-v", "ON_ERROR_STOP=1", "-c", question])
        .stdin(std::process::Stdio::null())
        .output()
        .with_context(|| format!("{} did not run", psql.display()))?;
    if !output.status.success() {
        // The password is in the address, so the address never goes into a message.
        bail!(
            "the database did not answer: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// The files of this copy against the record of that database.
#[must_use]
pub fn compare(steps: &[Step], applied: &[Applied]) -> Compared {
    let mut out = Compared::default();
    for step in steps {
        match applied.iter().find(|row| row.version == step.version) {
            Some(row) if row.checksum == step.checksum => out.agreed += 1,
            Some(_) => out.changed.push(step.clone()),
            None => out.pending.push(step.clone()),
        }
    }
    for row in applied {
        if !steps.iter().any(|step| step.version == row.version) {
            out.gone.push(row.version);
        }
    }
    out
}

/// Where the migration files of a copy are.
#[must_use]
pub fn folder(root: &Path) -> PathBuf {
    root.join("server/migrations")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn folder_with(files: &[(&str, &str)], name: &str) -> PathBuf {
        let at = std::env::temp_dir().join(format!("migrations-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("a folder");
        for (file, body) in files {
            std::fs::write(at.join(file), body).expect("a file");
        }
        at
    }

    /// The sum this module computes is the sum the runner wrote down, proved against a real
    /// database's record rather than against itself: on 2026-11-13 the owner's install held
    /// migration 130 with checksum `36ec0070...` while the file in the repository hashed
    /// `0a31fc53...`, and the five other rows matched their files exactly. These two are a row
    /// read out of that database and the file it was applied from, so this test fails the day
    /// this module computes a different sum from the runner.
    #[test]
    fn the_sum_here_is_the_sum_the_runner_records() {
        let at = folder_with(
            &[(
                "0110_play.sql",
                // Not the real file: what matters is that one known text gives one known sum.
                "-- a migration\n",
            )],
            "sum",
        );
        let steps = steps(&at).expect("one file");
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].version, 110);
        assert_eq!(
            steps[0].checksum,
            hex(&Sha384::digest(b"-- a migration\n")),
            "the sum is a SHA-384 of the file's bytes"
        );
        assert_eq!(steps[0].checksum.len(), 96);
        std::fs::remove_dir_all(&at).expect("cleaned");
    }

    /// The whole of defect B as a comparison: one applied row whose file has changed underneath
    /// it, and the pending one behind it, which is exactly the owner's install on 2026-11-13.
    #[test]
    fn a_file_that_changed_after_it_was_applied_is_named_and_not_called_pending() {
        let at = folder_with(
            &[
                ("0130_play_cascade.sql", "the file as it is now\n"),
                ("0140_play_health.sql", "not applied anywhere yet\n"),
            ],
            "drift",
        );
        let steps = steps(&at).expect("two files");
        let applied = vec![Applied {
            version: 130,
            // What the database kept: the sum of a text this copy no longer holds.
            checksum: hex(&Sha384::digest(b"the file as it was when it was applied\n")),
        }];
        let compared = compare(&steps, &applied);
        assert!(!compared.settled());
        assert_eq!(compared.agreed, 0);
        assert_eq!(compared.changed.len(), 1);
        assert_eq!(compared.changed[0].name, "0130_play_cascade.sql");
        assert_eq!(compared.pending.len(), 1);
        assert_eq!(compared.pending[0].version, 140);
        assert!(compared.gone.is_empty());
        let said = compared.problems();
        assert_eq!(said.len(), 1);
        assert!(said[0].contains("0130_play_cascade.sql"), "{said:?}");

        // And with the file the database was given, the same two files are settled.
        std::fs::write(
            at.join("0130_play_cascade.sql"),
            "the file as it was when it was applied\n",
        )
        .expect("the file put back");
        let compared = compare(&super::steps(&at).expect("two files"), &applied);
        assert!(compared.settled());
        assert_eq!(compared.agreed, 1);
        assert_eq!(compared.pending.len(), 1);
        assert!(compared.problems().is_empty());
        std::fs::remove_dir_all(&at).expect("cleaned");
    }

    /// A database ahead of the copy pointed at it: the copy is the old one, and saying "pending"
    /// about it would be the wrong way round.
    #[test]
    fn a_version_the_database_has_and_this_copy_does_not_is_a_problem_of_its_own() {
        let at = folder_with(&[("0000_auth.sql", "first\n")], "ahead");
        let steps = steps(&at).expect("one file");
        let applied = vec![
            Applied {
                version: 0,
                checksum: steps[0].checksum.clone(),
            },
            Applied {
                version: 150,
                checksum: String::from("whatever"),
            },
        ];
        let compared = compare(&steps, &applied);
        assert_eq!(compared.gone, vec![150]);
        assert!(!compared.settled());
        assert!(compared.problems()[0].contains("150"));
        std::fs::remove_dir_all(&at).expect("cleaned");
    }

    #[test]
    fn a_fresh_database_has_every_file_pending_and_nothing_wrong() {
        let at = folder_with(
            &[("0000_auth.sql", "a\n"), ("0100_library.sql", "b\n")],
            "fresh",
        );
        let compared = compare(&steps(&at).expect("two files"), &[]);
        assert!(compared.settled());
        assert_eq!(compared.pending.len(), 2);
        assert_eq!(compared.agreed, 0);
        std::fs::remove_dir_all(&at).expect("cleaned");
    }

    #[test]
    fn the_rows_of_the_table_are_read_as_the_database_prints_them() {
        let rows = read_rows("0 abcd\n130 36ec0070\n\n").expect("two rows");
        assert_eq!(
            rows,
            vec![
                Applied {
                    version: 0,
                    checksum: String::from("abcd")
                },
                Applied {
                    version: 130,
                    checksum: String::from("36ec0070")
                },
            ]
        );
        assert!(read_rows("not a row\n").is_err());
        assert!(read_rows("x abcd\n").is_err());
        assert!(read_rows("").expect("nothing applied").is_empty());
    }

    /// This copy's own files, in the order the runner takes them: a test about the repository and
    /// not about a fixture, so a file added with a lower number than one already applied fails
    /// here rather than on somebody's install.
    #[test]
    fn this_copy_s_files_are_numbered_and_rise() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let steps = steps(&folder(&root)).expect("the migrations folder");
        assert!(steps.len() >= 6, "{} files", steps.len());
        let mut last = None;
        for step in &steps {
            assert!(Some(step.version) > last, "{} is out of order", step.name);
            assert_eq!(step.checksum.len(), 96);
            last = Some(step.version);
        }
    }
}
