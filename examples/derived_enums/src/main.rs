use erm::prelude::*;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

#[derive(Debug, Component)]
enum LightSwitch {
    On { field_a: i64 },
    Off { field_b: i64, field_c: u32 },
    Whatever,
}

#[tokio::main]
async fn main() {
    // Create an Sqlite backend using u64 as entity IDs
    let options = SqliteConnectOptions::new().in_memory(true);

    let pool = SqlitePoolOptions::new()
        .min_connections(1)
        .max_connections(1)
        .idle_timeout(None)
        .max_lifetime(None)
        .connect_with(options)
        .await
        .unwrap();

    let backend: SqliteBackend<i64> = SqliteBackend::new(pool);

    // This creates the component tables where data will be persisted.
    backend.register::<LightSwitch>().await.unwrap();

    backend.insert(&1, &LightSwitch::On { field_a: 10 }).await;
    backend.insert(&2, &LightSwitch::Whatever).await;

    let switch: LightSwitch = backend.get(&1).await.unwrap();

    println!("{switch:#?}");
}
