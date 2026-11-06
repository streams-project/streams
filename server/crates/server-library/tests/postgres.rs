//! Needs a real database: set `TEST_DATABASE_URL` and `TEST_PROFILE_ID` and run with `--ignored`.
//! The database needs an `auth.users` table, the chain in `migrations/`, and that profile row.

use server_accounts::ProfileId;
use server_library::{LibraryStore, LibraryTx, PgLibrary};

#[tokio::test]
#[ignore = "needs a real Postgres in TEST_DATABASE_URL"]
async fn h53_one_writer_per_profile_keeps_change_numbers_in_order() {
    let url = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL");
    let id = std::env::var("TEST_PROFILE_ID").expect("TEST_PROFILE_ID");
    let pool = sqlx::PgPool::connect(&url).await.expect("connect");
    let store = PgLibrary::new(pool);
    let profile = ProfileId::parse(&id).expect("profile");
    let mut tx: Box<dyn LibraryTx> = store.begin(profile).await.expect("begin");
    let first = tx.next_seq().await.expect("seq");
    let second = tx.next_seq().await.expect("seq");
    assert_eq!(second, first + 1);
    tx.commit().await.expect("commit");
}
