use enorm::prelude::*;
use futures::TryStreamExt as _;

#[derive(Component, Debug, PartialEq, Eq)]
pub struct Name(String);

#[derive(Component, Debug, PartialEq, Eq)]
pub struct Age(i64);

#[tokio::main]
async fn main() {
    // Create an Sqlite backend using u64 as entity IDs
    let backend: SqliteBackend<i64> = SqliteBackend::in_memory().await;

    // This creates the component tables where data will be persisted.
    backend.register::<Name>().await.unwrap();
    backend.register::<Age>().await.unwrap();

    // Create our entities: Jimothy and Andrea
    //
    // Since we're just using i64s as our "EntityId", our entities
    // are actually just numbers.
    let jimothy = 1;
    backend
        .insert(&jimothy, &(Name("Jimothy".to_string()), Age(10)))
        .await;

    let andrea = 2;
    backend
        .insert(&andrea, &(Name("Andrea".to_string()), Age(32)))
        .await;

    // Let's name an Archetype instead of just relying on a tuple.
    #[derive(Archetype, Debug, PartialEq, Eq)]
    #[allow(unused)]
    struct Person {
        name: Name,
        age: Age,
    }

    // List all the people we know
    let people = backend
        .list::<Person>()
        .components()
        .fetch()
        .try_collect::<Vec<_>>()
        .await
        .unwrap();

    assert_eq!(
        people,
        [
            Person {
                name: Name("Jimothy".to_string(),),
                age: Age(10,),
            },
            Person {
                name: Name("Andrea".to_string(),),
                age: Age(32,),
            },
        ]
    );

    backend.remove::<Person>(&jimothy).await;

    let remaining_names = backend
        .list::<Name>()
        .components()
        .fetch()
        .try_collect::<Vec<_>>()
        .await
        .unwrap();

    // Check that only Andrea is left.
    assert_eq!(remaining_names, &[Name("Andrea".to_string())]);

    // Fetch Andrea's age
    assert_eq!(backend.get::<Age>(&andrea).await.unwrap(), Age(32));
}
