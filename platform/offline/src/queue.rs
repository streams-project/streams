//! The durable command queue. A command is kept with its op id, the monotonic time it was made
//! and the boot it was made in, and it leaves only on the server's answer. Age comes from
//! monotonic time, which restarts at boot, so after a boot change the age is unknown and the
//! server orders by arrival.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use contract::command::{Command, CommandBatch, CommandOp, OpAnswer, Outcome};
use core_rules::net::Backoff;
use serde::{Deserialize, Serialize};

/// Wait after a change before sending. Each new change starts it again.
pub const DEBOUNCE_MS: u64 = 4_000;

/// Most ops in one batch.
pub const BATCH_MAX: usize = 400;

/// Prefix of queue keys in the store.
pub const KEY_PREFIX: &str = "q/";

/// One kept command.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Kept {
    pub seq: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    pub op: CommandOp,
    pub mono_ms: u64,
    pub boot: String,
    /// Bytes this takes in the store.
    #[serde(skip)]
    pub bytes: u64,
}

impl Kept {
    #[must_use]
    pub fn key(&self) -> String {
        format!("{KEY_PREFIX}{:016x}", self.seq)
    }

    /// `None` when this op was made in another boot, or the clock ran backwards.
    fn age_ms(&self, boot: &str, mono_ms: u64) -> Option<u64> {
        (self.boot == boot).then(|| mono_ms.checked_sub(self.mono_ms))?
    }
}

/// One op ready to be kept: the entry with the bytes the store needs, the text the store must
/// hold, and the unsent position it would replace. Nothing is in the queue until `admit` takes
/// it, so an op the store has no room for leaves the queue, the byte count and the older position
/// it would have replaced exactly as they were.
pub struct Planned {
    kept: Kept,
    text: String,
    /// Where the unsent position of the same title and episode sits, with its key and its bytes.
    replaces: Option<(usize, String, u64)>,
}

impl Planned {
    /// Bytes the store must find for this op. What a replaced position gives back is taken off,
    /// so a position replacing one of its own size needs no new room.
    #[must_use]
    pub fn need(&self) -> u64 {
        let given_back = self.replaces.as_ref().map_or(0, |(_, _, bytes)| *bytes);
        self.kept.bytes.saturating_sub(given_back)
    }
}

/// The queue. One batch is on its way at a time, so a session has one writer.
#[derive(Default)]
pub struct Queue {
    ops: Vec<Kept>,
    next_seq: u64,
    in_flight: Vec<String>,
    flush_at: Option<u64>,
    backoff: Backoff,
}

impl Queue {
    /// Put back what the store held, in the order it was made.
    pub fn load(&mut self, mut kept: Vec<Kept>) {
        kept.sort_by_key(|item| item.seq);
        self.next_seq = kept.last().map_or(0, |item| item.seq + 1);
        self.ops = kept;
    }

    /// Plan to keep one op. The entry is written out here, so what the store must hold is known
    /// before the queue takes anything; `None` when the op does not write out at all. A newer
    /// position replaces an unsent older one of the same title and episode, so a long offline run
    /// cannot fill the queue.
    #[must_use]
    pub fn plan(
        &self,
        op: CommandOp,
        scope: Option<String>,
        boot: &str,
        mono_ms: u64,
    ) -> Option<Planned> {
        let mut kept = Kept {
            seq: self.next_seq,
            scope,
            op,
            mono_ms,
            boot: boot.into(),
            bytes: 0,
        };
        let text = serde_json::to_string(&kept).ok()?;
        kept.bytes = u64::try_from(text.len()).unwrap_or(u64::MAX);
        let replaces = self
            .position_slot(&kept.op, kept.scope.as_deref())
            .map(|index| (index, self.ops[index].key(), self.ops[index].bytes));
        Some(Planned {
            kept,
            text,
            replaces,
        })
    }

    /// Take a planned op into the queue, with its store bytes already counted. Gives back the
    /// entry, the text the store must hold, and the key of the position it replaced, if any.
    pub fn admit(&mut self, planned: Planned) -> (Kept, String, Option<String>) {
        let Planned {
            kept,
            text,
            replaces,
        } = planned;
        let replaced = replaces.map(|(index, key, _)| {
            self.ops.remove(index);
            key
        });
        self.next_seq = kept.seq + 1;
        self.ops.push(kept.clone());
        self.flush_at = Some(kept.mono_ms + DEBOUNCE_MS);
        (kept, text, replaced)
    }

    fn position_slot(&self, op: &CommandOp, scope: Option<&str>) -> Option<usize> {
        let Command::Position { title, episode, .. } = &op.command else {
            return None;
        };
        self.ops.iter().position(|kept| {
            let same = matches!(
                &kept.op.command,
                Command::Position { title: t, episode: e, .. } if t == title && e == episode
            );
            same && kept.scope.as_deref() == scope && !self.in_flight.contains(&kept.op.op_id)
        })
    }

    #[must_use]
    pub const fn sending(&self) -> bool {
        !self.in_flight.is_empty()
    }

    /// True when ops of this scope wait to go.
    #[must_use]
    pub fn has_ops(&self, scope: &str) -> bool {
        self.ops
            .iter()
            .any(|kept| kept.scope.as_deref() == Some(scope))
    }

    #[must_use]
    pub const fn flush_due(&self, mono_ms: u64) -> bool {
        match self.flush_at {
            Some(at) => mono_ms >= at,
            None => false,
        }
    }

    /// Send as soon as anyone looks. Cuts any wait.
    pub const fn flush_now(&mut self, mono_ms: u64) {
        self.flush_at = Some(mono_ms);
    }

    pub const fn stop_waiting(&mut self) {
        self.flush_at = None;
    }

    #[must_use]
    pub const fn next_wake(&self) -> Option<u64> {
        if self.sending() { None } else { self.flush_at }
    }

    /// The next batch, oldest first, at most 400 ops of this scope.
    pub fn batch(&mut self, scope: &str, boot: &str, mono_ms: u64) -> Option<CommandBatch> {
        if self.sending() {
            return None;
        }
        let ops: Vec<CommandOp> = self
            .ops
            .iter()
            .filter(|kept| kept.scope.as_deref() == Some(scope))
            .take(BATCH_MAX)
            .map(|kept| CommandOp {
                age_ms: kept.age_ms(boot, mono_ms),
                ..kept.op.clone()
            })
            .collect();
        if ops.is_empty() {
            return None;
        }
        self.flush_at = None;
        self.in_flight = ops.iter().map(|op| op.op_id.clone()).collect();
        Some(CommandBatch { ops })
    }

    /// Clear what the server answered. Returns the keys that left. An op left out of the answer
    /// stays for the next batch.
    pub fn acknowledge(&mut self, answers: &[OpAnswer], mono_ms: u64) -> Vec<String> {
        let settled = |kept: &Kept| {
            answers.iter().any(|answer| {
                answer.op_id == kept.op.op_id
                    && matches!(
                        answer.outcome,
                        Outcome::Applied | Outcome::Superseded | Outcome::Rejected
                    )
            })
        };
        let keys: Vec<String> = self
            .ops
            .iter()
            .filter(|kept| settled(kept))
            .map(Kept::key)
            .collect();
        self.ops.retain(|kept| !settled(kept));
        self.in_flight.clear();
        if keys.is_empty() {
            self.failed(mono_ms);
        } else {
            self.backoff.reset();
            if !self.ops.is_empty() {
                self.flush_at = Some(mono_ms);
            }
        }
        keys
    }

    /// The batch failed and every op stays. Try again after a wait.
    pub fn failed(&mut self, mono_ms: u64) {
        self.in_flight.clear();
        self.flush_at = Some(mono_ms + self.backoff.next_wait());
    }

    /// The server asked for a wait, or refused the batch. A wait of `None` holds the ops until
    /// the next change or nudge.
    pub fn wait(&mut self, wait_ms: Option<u64>, mono_ms: u64) {
        self.in_flight.clear();
        self.flush_at = wait_ms.map(|wait| mono_ms + wait);
    }

    /// Forget everything. Returns the keys to delete.
    pub fn clear(&mut self) -> Vec<String> {
        self.in_flight.clear();
        self.flush_at = None;
        self.backoff.reset();
        let keys = self.ops.iter().map(Kept::key).collect();
        self.ops.clear();
        keys
    }

    /// Bytes the kept ops take in the store.
    #[must_use]
    pub fn bytes(&self) -> u64 {
        self.ops.iter().map(|kept| kept.bytes).sum()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.ops.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }
}
