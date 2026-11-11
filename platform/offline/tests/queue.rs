//! The command queue. Each test names the behaviour or defect it covers.

use contract::command::{Command, CommandOp, OpAnswer, Outcome};
use contract::{EpisodeRef, TitleKey, TitleKind};
use offline::queue::{BATCH_MAX, DEBOUNCE_MS, Kept, Queue};

const SCOPE: &str = "u1/p1";
const BOOT: &str = "boot-1";

fn title(id: u32) -> TitleKey {
    TitleKey::new(TitleKind::Movie, id).unwrap()
}

fn list(id: u32) -> CommandOp {
    CommandOp {
        op_id: format!("op-{id}"),
        age_ms: Some(0),
        command: Command::List {
            title: title(id),
            in_list: true,
        },
    }
}

fn position(op: u32, id: u32, episode: Option<EpisodeRef>, seconds: u32) -> CommandOp {
    CommandOp {
        op_id: format!("op-{op}"),
        age_ms: Some(0),
        command: Command::Position {
            title: title(id),
            episode,
            position_s: seconds,
            duration_s: 6_000,
        },
    }
}

/// Plan and admit in one step, the way the layer does when the store has room.
fn add(queue: &mut Queue, op: CommandOp, mono_ms: u64) -> (Kept, Option<String>) {
    add_to(queue, op, SCOPE, mono_ms)
}

fn add_to(queue: &mut Queue, op: CommandOp, scope: &str, mono_ms: u64) -> (Kept, Option<String>) {
    let planned = queue
        .plan(op, Some(scope.to_string()), BOOT, mono_ms)
        .expect("an op writes out");
    let (kept, _, replaced) = queue.admit(planned);
    (kept, replaced)
}

fn applied(ids: &[&str]) -> Vec<OpAnswer> {
    ids.iter()
        .map(|id| OpAnswer {
            op_id: (*id).to_string(),
            outcome: Outcome::Applied,
        })
        .collect()
}

#[test]
fn c19_a_batch_holds_at_most_400_ops_oldest_first() {
    let mut queue = Queue::default();
    for id in 0..405 {
        add(&mut queue, list(id + 1), 0);
    }
    let batch = queue.batch(SCOPE, BOOT, 10).unwrap();
    assert_eq!(batch.ops.len(), BATCH_MAX);
    assert_eq!(BATCH_MAX, 400);
    assert_eq!(batch.ops[0].op_id, "op-1");
    let ids: Vec<&str> = batch.ops.iter().map(|op| op.op_id.as_str()).collect();
    assert_eq!(queue.acknowledge(&applied(&ids), 20).len(), 400);
    assert_eq!(queue.len(), 5);
    assert!(queue.flush_due(20), "the rest goes at once");
    assert_eq!(queue.batch(SCOPE, BOOT, 20).unwrap().ops[0].op_id, "op-401");
}

#[test]
fn c19_each_change_starts_the_4_s_wait_again() {
    let mut queue = Queue::default();
    add(&mut queue, list(1), 1_000);
    assert_eq!(queue.next_wake(), Some(1_000 + DEBOUNCE_MS));
    add(&mut queue, list(2), 3_000);
    assert_eq!(queue.next_wake(), Some(3_000 + 4_000));
    assert!(!queue.flush_due(6_999));
    assert!(queue.flush_due(7_000));
}

#[test]
fn h61_age_comes_from_monotonic_time_and_a_boot_change_makes_it_unknown() {
    let mut queue = Queue::default();
    add(&mut queue, list(1), 1_000);
    assert_eq!(
        queue.batch(SCOPE, BOOT, 9_000).unwrap().ops[0].age_ms,
        Some(8_000)
    );
    queue.failed(9_000);
    // the new boot's clock is past the old value, so only the boot id can tell
    assert_eq!(
        queue.batch(SCOPE, "boot-2", 50_000).unwrap().ops[0].age_ms,
        None
    );
}

#[test]
fn h61_a_clock_that_ran_backwards_in_one_boot_gives_no_age() {
    let mut queue = Queue::default();
    add(&mut queue, list(1), 9_000);
    assert_eq!(queue.batch(SCOPE, BOOT, 100).unwrap().ops[0].age_ms, None);
}

#[test]
fn h20_h47_h85_an_op_leaves_only_on_the_servers_answer() {
    let mut queue = Queue::default();
    add(&mut queue, list(1), 0);
    add(&mut queue, list(2), 0);
    queue.batch(SCOPE, BOOT, 10).unwrap();
    let keys = queue.acknowledge(&applied(&["op-2"]), 10);
    assert_eq!(keys.len(), 1);
    assert_eq!(queue.len(), 1);
    assert!(queue.has_ops(SCOPE));
}

#[test]
fn c2_superseded_and_rejected_ops_are_settled_too() {
    let mut queue = Queue::default();
    add(&mut queue, list(1), 0);
    add(&mut queue, list(2), 0);
    queue.batch(SCOPE, BOOT, 10).unwrap();
    let answers = [
        OpAnswer {
            op_id: "op-1".to_string(),
            outcome: Outcome::Superseded,
        },
        OpAnswer {
            op_id: "op-2".to_string(),
            outcome: Outcome::Rejected,
        },
    ];
    assert_eq!(queue.acknowledge(&answers, 10).len(), 2);
    assert!(queue.is_empty());
}

#[test]
fn an_answer_that_settles_nothing_backs_off_and_never_spins() {
    let mut queue = Queue::default();
    add(&mut queue, list(1), 0);
    queue.batch(SCOPE, BOOT, 10).unwrap();
    assert!(queue.acknowledge(&[], 10).is_empty());
    assert_eq!(queue.next_wake(), Some(2_010));
}

#[test]
fn c5_h48_a_failed_send_keeps_every_op_and_backs_off_then_a_success_resets_it() {
    let mut queue = Queue::default();
    add(&mut queue, list(1), 0);
    for (now, wake) in [(10, 2_010), (2_010, 6_010), (6_010, 14_010)] {
        queue.batch(SCOPE, BOOT, now).unwrap();
        queue.failed(now);
        assert_eq!(queue.next_wake(), Some(wake));
        assert_eq!(queue.len(), 1);
    }
    queue.batch(SCOPE, BOOT, 14_010).unwrap();
    queue.acknowledge(&applied(&["op-1"]), 14_010);
    add(&mut queue, list(2), 14_010);
    queue.batch(SCOPE, BOOT, 14_010).unwrap();
    queue.failed(14_010);
    assert_eq!(queue.next_wake(), Some(16_010));
}

#[test]
fn h53_h54_one_batch_at_a_time_so_a_session_has_one_writer() {
    let mut queue = Queue::default();
    add(&mut queue, list(1), 0);
    assert!(queue.batch(SCOPE, BOOT, 0).is_some());
    add(&mut queue, list(2), 0);
    assert!(queue.batch(SCOPE, BOOT, 0).is_none());
    assert_eq!(queue.next_wake(), None, "no wake while a batch is out");
}

#[test]
fn h53_a_batch_holds_only_the_ops_of_its_own_profile() {
    let mut queue = Queue::default();
    add_to(&mut queue, list(1), "u1/p1", 0);
    add_to(&mut queue, list(2), "u1/p2", 0);
    let batch = queue.batch("u1/p2", BOOT, 0).unwrap();
    assert_eq!(batch.ops.len(), 1);
    assert_eq!(batch.ops[0].op_id, "op-2");
}

#[test]
fn c2_a_newer_position_replaces_an_unsent_older_one_of_the_same_episode() {
    let first = Some(EpisodeRef {
        season: 1,
        episode: 1,
    });
    let second = Some(EpisodeRef {
        season: 1,
        episode: 2,
    });
    let mut queue = Queue::default();
    let (old, _) = add(&mut queue, position(1, 7, first.clone(), 30), 0);
    let (_, replaced) = add(&mut queue, position(2, 7, first.clone(), 60), 0);
    assert_eq!(replaced, Some(old.key()));
    assert_eq!(queue.len(), 1);
    assert_eq!(add(&mut queue, position(3, 7, second, 10), 0).1, None);
    assert_eq!(add(&mut queue, position(4, 8, first, 10), 0).1, None);
    assert_eq!(queue.len(), 3);
}

#[test]
fn c2_a_position_on_its_way_is_not_replaced() {
    let mut queue = Queue::default();
    add(&mut queue, position(1, 7, None, 30), 0);
    queue.batch(SCOPE, BOOT, 0).unwrap();
    assert_eq!(add(&mut queue, position(2, 7, None, 60), 0).1, None);
    assert_eq!(queue.len(), 2);
}

#[test]
fn a_kept_op_comes_back_from_the_store_in_order_and_new_ops_follow_it() {
    let mut first = Queue::default();
    let (one, _) = add(&mut first, list(1), 0);
    let (two, _) = add(&mut first, list(2), 0);
    let text = serde_json::to_string(&two).unwrap();
    let back: Kept = serde_json::from_str(&text).unwrap();
    assert_eq!(back.key(), two.key());
    assert!(text.contains("\"op_id\":\"op-2\""));

    let mut second = Queue::default();
    second.load(vec![two, one]);
    let (three, _) = add(&mut second, list(3), 0);
    assert_eq!(three.seq, 2);
    let batch = second.batch(SCOPE, BOOT, 0).unwrap();
    let ids: Vec<&str> = batch.ops.iter().map(|op| op.op_id.as_str()).collect();
    assert_eq!(ids, ["op-1", "op-2", "op-3"]);
}

#[test]
fn p13_clearing_gives_every_key_and_leaves_nothing_due() {
    let mut queue = Queue::default();
    let (one, _) = add(&mut queue, list(1), 0);
    let (two, _) = add(&mut queue, list(2), 0);
    assert_eq!(queue.clear(), vec![one.key(), two.key()]);
    assert!(queue.is_empty());
    assert_eq!(queue.next_wake(), None);
}

#[test]
fn a_kept_op_counts_its_store_bytes_before_it_is_admitted() {
    let mut queue = Queue::default();
    let planned = queue
        .plan(list(1), Some(SCOPE.to_string()), BOOT, 0)
        .unwrap();
    // the bytes are known while the queue is still empty, so a full store can refuse the op
    assert!(planned.need() > 0);
    assert_eq!(queue.bytes(), 0);
    let (kept, text, _) = queue.admit(planned);
    assert_eq!(kept.bytes, text.len() as u64);
    assert_eq!(queue.bytes(), kept.bytes);

    // A position that replaces an unsent one asks only for what it takes beyond it.
    let (first, _) = add(&mut queue, position(2, 7, None, 30), 0);
    let planned = queue
        .plan(position(3, 7, None, 60), Some(SCOPE.to_string()), BOOT, 0)
        .unwrap();
    assert!(
        planned.need() < first.bytes,
        "the older one gives its bytes back"
    );
    let (second, _, replaced) = queue.admit(planned);
    assert_eq!(replaced, Some(first.key()));
    assert_eq!(queue.bytes(), kept.bytes + second.bytes);
}

#[test]
fn an_op_that_is_planned_and_not_admitted_leaves_the_queue_alone() {
    let mut queue = Queue::default();
    let (first, _) = add(&mut queue, position(1, 7, None, 30), 0);
    let planned = queue
        .plan(
            position(2, 7, None, 60),
            Some(SCOPE.to_string()),
            BOOT,
            5_000,
        )
        .unwrap();
    drop(planned);
    assert_eq!(queue.len(), 1, "the older position is still there");
    assert_eq!(queue.bytes(), first.bytes);
    assert_eq!(
        queue.next_wake(),
        Some(DEBOUNCE_MS),
        "no new wait was started"
    );
    let batch = queue.batch(SCOPE, BOOT, 10).unwrap();
    assert_eq!(batch.ops.len(), 1);
    assert_eq!(batch.ops[0].op_id, "op-1");
}
