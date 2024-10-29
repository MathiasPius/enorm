use erm::prelude::*;
use sqlx::{Database, Sqlite, TypeInfo};

#[derive(Debug)]
enum LightSwitch {
    On { field_a: i64 },
    Off { field_b: i64 },
}

impl Deserializeable<Sqlite> for LightSwitch {
    fn cte() -> Box<dyn CommonTableExpression> {
        Box::new(Extract {
            table: "LightSwitch",
            columns: &["tag", "field_a", "field_b"],
        })
    }

    fn deserialize(
        row: &mut erm::row::OffsetRow<<Sqlite as Database>::Row>,
    ) -> Result<Self, sqlx::Error> {
        let tag = row.try_get::<String>()?;
        let field_a = row.try_get::<Option<i64>>()?;
        let field_b = row.try_get::<Option<i64>>()?;

        match tag.as_str() {
            "On" => Ok(LightSwitch::On {
                field_a: field_a.unwrap(),
            }),
            "Off" => Ok(LightSwitch::Off {
                field_b: field_b.unwrap(),
            }),
            _ => Err(sqlx::Error::RowNotFound),
        }
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
        query: &mut erm::entity::EntityPrefixedQuery<'query, Sqlite, EntityId>,
    ) where
        EntityId: sqlx::Encode<'query, Sqlite> + sqlx::Type<Sqlite> + Clone + 'query,
    {
        query.query(<Self as Component<::sqlx::Sqlite>>::INSERT, move |query| {
            <Self as Serializable<::sqlx::Sqlite>>::serialize(self, query)
        });
    }

    fn update<'query, EntityId>(
        &'query self,
        query: &mut erm::entity::EntityPrefixedQuery<'query, Sqlite, EntityId>,
    ) where
        EntityId: sqlx::Encode<'query, Sqlite> + sqlx::Type<Sqlite> + Clone + 'query,
    {
        query.query(<Self as Component<::sqlx::Sqlite>>::INSERT, move |query| {
            <Self as Serializable<::sqlx::Sqlite>>::serialize(self, query)
        })
    }
}

impl Component<Sqlite> for LightSwitch {
    const INSERT: &'static str =
        "insert into LightSwitch(entity, tag, field_a, field_b) values(?, ?, ?, ?);";

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
        async move {
            let sql = format!("create table if not exists LightSwitch(entity {} primary key, tag text not null, field_a integer null, field_b integer null);", <EntityId as sqlx::Type::<Sqlite>>::type_info().name());
            let query = sqlx::query(&sql);
            query.execute(pool).await
        }
    }
}

impl Archetype<Sqlite> for LightSwitch {}

#[tokio::main]
async fn main() {
    // Create an Sqlite backend using u64 as entity IDs
    let backend: SqliteBackend<i64> = SqliteBackend::in_memory().await;

    // This creates the component tables where data will be persisted.
    backend.register::<LightSwitch>().await.unwrap();

    backend.insert(&1, &LightSwitch::On { field_a: 10 }).await;

    let switch: LightSwitch = backend.get(&1).await.unwrap();

    println!("{switch:#?}");
}
