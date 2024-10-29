use erm::prelude::*;

#[derive(Debug, Component)]
enum LightSwitch {
    On { field_a: i64 },
    Off { field_b: i64 },
}

#[tokio::main]
async fn main() {
    // Create an Sqlite backend using u64 as entity IDs
    let backend: SqliteBackend<i64> = SqliteBackend::in_memory().await;

    /*
    // This creates the component tables where data will be persisted.
    backend.register::<LightSwitch>().await.unwrap();

    backend.insert(&1, &LightSwitch::On { field_a: 10 }).await;

    let switch: LightSwitch = backend.get(&1).await.unwrap();

    println!("{switch:#?}");
     */
}
