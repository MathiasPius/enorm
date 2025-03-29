use enorm::prelude::*;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

#[derive(Component, Debug)]
struct Counter(i64);

#[derive(Debug, Archetype)]
enum LightSwitch {
    On { counter: Counter },
    Off,
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
    backend.register::<Counter>().await.unwrap();

    backend.insert(&1, &Counter(10)).await;
    //backend.insert(&2, &LightSwitch::Off).await;

    let switch1: LightSwitch = backend.get(&1).await.unwrap();
    let switch2: LightSwitch = backend.get(&2).await.unwrap();

    println!("{switch1:#?}");
    println!("{switch2:#?}");
}
