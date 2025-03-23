use enorm::prelude::*;
use sqlx::{Database, Sqlite};

#[derive(Component, Debug)]
struct Counter(i64);

#[derive(Debug)]
enum LightSwitch {
    On { field_a: i64, counter: Counter },
    Off { field_b: i64, counter: Counter },
}

impl Deserializeable<Sqlite> for LightSwitch {
    fn cte() -> Box<dyn CommonTableExpression> {
        Box::new(Merge {
            tables: vec![
                Box::new(Extract {
                    table: "LightSwitch",
                    columns: &["field_a", "field_b"],
                }),
                Counter::cte(),
            ],
        })
    }

    fn deserialize(
        row: &mut enorm::row::OffsetRow<<Sqlite as Database>::Row>,
    ) -> Result<Self, sqlx::Error> {
        let tag = row.try_get::<String>()?;
        let field_a = row.try_get::<Option<i64>>()?;
        let field_b = row.try_get::<Option<i64>>()?;

        let counter = <Counter as Deserializeable<Sqlite>>::deserialize(row)?;

        Ok(match tag.as_str() {
            "On" => LightSwitch::On {
                field_a: field_a.unwrap(),
                counter,
            },
            "Off" => LightSwitch::Off {
                field_b: field_b.unwrap(),
                counter,
            },
            _ => panic!("Unknown variant"),
        })
    }
}

impl Serializable<Sqlite> for LightSwitch {
    fn serialize<'query>(
        &'query self,
        query: sqlx::query::Query<'query, Sqlite, <Sqlite as Database>::Arguments<'query>>,
    ) -> sqlx::query::Query<'query, Sqlite, <Sqlite as Database>::Arguments<'query>> {
        match self {
            LightSwitch::On { field_a, .. } => {
                let query = query.bind("On");
                let query = query.bind(Some(field_a));
                let query = query.bind::<Option<i64>>(None);
                query
            }
            LightSwitch::Off { field_b, .. } => {
                let query = query.bind("Off");
                let query = query.bind::<Option<i64>>(None);
                let query = query.bind(Some(field_b));
                query
            }
        }
    }

    fn insert<'query, EntityId>(
        &'query self,
        query: &mut enorm::entity::EntityPrefixedQuery<'query, Sqlite, EntityId>,
    ) where
        EntityId: sqlx::Encode<'query, Sqlite> + sqlx::Type<Sqlite> + Clone + 'query,
    {
        query.query(<Self as Component<::sqlx::Sqlite>>::INSERT, move |query| {
            <Self as Serializable<::sqlx::Sqlite>>::serialize(self, query)
        });

        match self {
            LightSwitch::On { counter, .. } => {
                <Counter as Serializable<Sqlite>>::insert(counter, query);
            }
            LightSwitch::Off { counter, .. } => {
                <Counter as Serializable<Sqlite>>::insert(counter, query);
            }
        }
    }

    fn update<'query, EntityId>(
        &'query self,
        query: &mut enorm::entity::EntityPrefixedQuery<'query, Sqlite, EntityId>,
    ) where
        EntityId: sqlx::Encode<'query, Sqlite> + sqlx::Type<Sqlite> + Clone + 'query,
    {
        query.query(<Self as Component<::sqlx::Sqlite>>::INSERT, move |query| {
            <Self as Serializable<::sqlx::Sqlite>>::serialize(self, query)
        })
    }
}

impl Component<Sqlite> for LightSwitch {
    const INSERT: &'static str = "";

    const UPDATE: &'static str = "";

    const DELETE: &'static str = "";

    fn table() -> &'static str {
        "LightSwitch"
    }

    fn columns() -> Vec<ColumnDefinition<Sqlite>> {
        todo!()
    }

    fn create_component_table<'pool, EntityId>(
        pool: &'pool sqlx::Pool<Sqlite>,
    ) -> impl std::future::Future<
        Output = Result<<Sqlite as sqlx::Database>::QueryResult, sqlx::Error>,
    > + Send
    where
        EntityId: sqlx::Type<Sqlite>,
    {
        use sqlx::TypeInfo;
        async move {
            let sql = format!("create table if not exists LightSwitch(entity {} primary key, field_a integer null, field_b integer null);", <EntityId as sqlx::Type::<Sqlite>>::type_info().name());

            sqlx::query(&sql).execute(pool).await
        }
    }
}

#[tokio::main]
async fn main() {
    // Create an Sqlite backend using u64 as entity IDs
    let backend: SqliteBackend<i64> = SqliteBackend::in_memory().await;

    // This creates the component tables where data will be persisted.
    backend.register::<Counter>().await.unwrap();
    backend.register::<LightSwitch>().await.unwrap();
}
