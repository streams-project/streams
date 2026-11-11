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

/// Most ops held for this session only, with no store room for them. They count no store bytes,
/// so the byte cap does not bound them and this does. Well under `BATCH_MAX`, so a session-held
/// backlog always fits in one batch.
pub const SESSION_MAX: usize = 64;

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
    /// Kept for this session only: never written, so never deleted and gone on the next start.
    /// Skipped, and false by default, so an op read back from the store is a stored one.
    #[serde(skip)]
    pub session_only: bool,
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

/// One op ready to be kept: the entry with its reserved sequence number, the text the store must
/// hold, and the identity of the unsent position it would replace. Nothing is in the queue until
/// `admit` takes it, so an op the store has no room for leaves the queue, the byte count and the
/// older position it would have replaced exactly as they were, and `release` gives the reserved
/// sequence number back.
pub struct Planned {
    kept: Kept,
    text: String,
    /// The op id of the unsent position of the same title and episode. An op id, not a position
    /// in `Queue::ops`: anything may leave the queue between `plan` and `admit`, and a stale
    /// index would remove another op or run past the end, which panics in a core with no
    /// unwinding.
    replaces: Option<String>,
}

impl Planned {
    /// The sequence number reserved for this op. No other plan can be given it.
    #[must_use]
    pub const fn seq(&self) -> u64 {
        self.kept.seq
    }

    /// True when nothing else will send this command again, so refusing it loses what the person
    /// did. A `Position` is carried by the next heartbeat; the rest are sent once and a core with
    /// no layer only logs a dropped one (`platform/core/src/lib.rs`, `command-dropped`).
    #[must_use]
    pub const fn only_chance(&self) -> bool {
        !matches!(self.kept.op.command, Command::Position { .. })
    }
}

/// How long an admitted op is held.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hold {
    /// Written to the store, so it survives a restart. Counts against the byte cap.
    Stored,
    /// Held for this session only: nothing is written, nothing is counted, and it is gone on the
    /// next start. For an op the store has no room for and nothing else would send again.
    Session,
}

/// What `admit` took into the queue.
pub struct Admitted {
    pub kept: Kept,
    /// The text the store must hold. `None` for a `Hold::Session` op, which is never written.
    pub text: Option<String>,
    /// The key of the unsent position this op replaced, if it was still in the queue.
    pub replaced: Option<String>,
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
        // Never backwards: a sequence number already spent must not be handed out again.
        let after = kept.last().map_or(0, |item| item.seq + 1);
        self.next_seq = self.next_seq.max(after);
        self.ops = kept;
    }

    /// Plan to keep one op. The entry is written out here, so what the store must hold is known
    /// before the queue takes anything; `None` when the op does not write out at all. A newer
    /// position replaces an unsent older one of the same title and episode, so a long offline run
    /// cannot fill the queue.
    #[must_use]
    pub fn plan(
        &mut self,
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
            session_only: false,
        };
        let text = serde_json::to_string(&kept).ok()?;
        kept.bytes = u64::try_from(text.len()).unwrap_or(u64::MAX);
        let replaces = self
            .position_slot(&kept.op, kept.scope.as_deref())
            .map(|index| self.ops[index].op.op_id.clone());
        // The sequence number is spent here, not in `admit`. It is in the text the store will
        // hold and in the key that text is written under, so two plans in flight must not read
        // the same one: that is a second store key minting the first one's name.
        self.next_seq += 1;
        Some(Planned {
            kept,
            text,
            replaces,
        })
    }

    /// Bytes the store must find for this planned op, measured now. What the position it replaces
    /// gives back is taken off, so a position replacing one of its own size needs no new room —
    /// and only while that position is still in the queue, since an op that left has already
    /// given its bytes back.
    #[must_use]
    pub fn need(&self, planned: &Planned) -> u64 {
        let given_back = self
            .replaceable(planned)
            .map_or(0, |index| self.ops[index].bytes);
        planned.kept.bytes.saturating_sub(given_back)
    }

    /// Take a planned op into the queue, with its store bytes already counted. `hold` is asked
    /// for, never assumed: a `Hold::Session` op is not written, so it must not be counted or
    /// deleted either.
    pub fn admit(&mut self, planned: Planned, hold: Hold) -> Admitted {
        let Planned {
            mut kept,
            text,
            replaces,
        } = planned;
        // By op id, and read now: `plan` may have run before an `acknowledge`, a `clear`, a
        // `batch` or another `admit`, so the position this op sat at means nothing by the time we
        // get here.
        let replaced = self
            .replaceable_id(replaces.as_deref())
            .map(|index| self.ops.remove(index))
            .filter(|gone| !gone.session_only)
            .map(|gone| gone.key());
        let text = match hold {
            Hold::Stored => Some(text),
            Hold::Session => {
                kept.bytes = 0;
                kept.session_only = true;
                None
            }
        };
        self.ops.push(kept.clone());
        self.flush_at = Some(kept.mono_ms + DEBOUNCE_MS);
        Admitted {
            kept,
            text,
            replaced,
        }
    }

    /// Give back a planned op that was not admitted. Its sequence number returns when no other
    /// plan has taken one since, so a refusal leaves the queue exactly as it was.
    pub fn release(&mut self, planned: Planned) {
        if planned.kept.seq + 1 == self.next_seq {
            self.next_seq = planned.kept.seq;
        }
    }

    /// Ops held for this session only. Bounded, because they count no store bytes and the byte
    /// cap is what otherwise bounds the queue.
    #[must_use]
    pub fn session_held(&self) -> usize {
        self.ops.iter().filter(|kept| kept.session_only).count()
    }

    /// Where the op this plan would replace sits, if it is still there to be replaced. The rule
    /// `plan` applied holds here too: an op already on its way keeps its place and its store key,
    /// so a batch that fails still has it, and the server settles the pair by op id.
    fn replaceable(&self, planned: &Planned) -> Option<usize> {
        self.replaceable_id(planned.replaces.as_deref())
    }

    fn replaceable_id(&self, op_id: Option<&str>) -> Option<usize> {
        let op_id = op_id?;
        if self.in_flight.iter().any(|sent| sent == op_id) {
            return None;
        }
        self.ops.iter().position(|kept| kept.op.op_id == op_id)
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

    /// Clear what the server answered. Returns the store keys that left — a session-held op was
    /// never written, so it has none. An op left out of the answer stays for the next batch.
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
            .filter(|kept| settled(kept) && !kept.session_only)
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

    /// Forget everything. Returns the keys to delete, which a session-held op has none of.
    pub fn clear(&mut self) -> Vec<String> {
        self.in_flight.clear();
        self.flush_at = None;
        self.backoff.reset();
        let keys = self
            .ops
            .iter()
            .filter(|kept| !kept.session_only)
            .map(Kept::key)
            .collect();
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
