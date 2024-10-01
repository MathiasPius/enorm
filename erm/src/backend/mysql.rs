use std::{future::Future, marker::PhantomData};

use sqlx::mysql::MySqlQueryResult;
use sqlx::{MySql, Pool};

use crate::prelude::{Component, Deserializeable, Serializable};
use crate::row::Rowed;
use crate::tables::Removable;
use crate::{archetype::Archetype, condition::All};

use super::{Backend, List};

pub struct MySqlBackend<Entity> {
    pool: Pool<MySql>,
    _entity: PhantomData<Entity>,
}

impl<Entity> MySqlBackend<Entity> {
    pub fn new(pool: Pool<MySql>) -> Self {
        MySqlBackend {
            pool,
            _entity: PhantomData,
        }
    }
}

impl<Entity> Backend<MySql, Entity> for MySqlBackend<Entity>
where
    Entity: for<'q> sqlx::Encode<'q, MySql>
        + for<'r> sqlx::Decode<'r, MySql>
        + sqlx::Type<MySql>
        + Unpin
        + Send
        + 'static,
    for<'entity> &'entity Entity: Send,
{
    fn register<T>(&self) -> impl Future<Output = Result<MySqlQueryResult, sqlx::Error>>
    where
        T: Component<MySql>,
    {
        <T as Component<MySql>>::create_component_table::<Entity>(&self.pool)
    }

    fn list<T>(&self) -> List<MySql, Entity, T, (), All> {
        List {
            pool: self.pool.clone(),
            _data: PhantomData,
            condition: All,
        }
    }

    fn get<T>(&self, entity: &Entity) -> impl Future<Output = Result<T, sqlx::Error>>
    where
        T: Deserializeable<MySql> + Unpin + Send + 'static,
    {
        async move {
            let sql = crate::cte::serialize(<T as Deserializeable<MySql>>::cte().as_ref()).unwrap();

            let result: Rowed<Entity, T> = sqlx::query_as(&sql)
                .bind(entity)
                .fetch_one(&self.pool)
                .await?;

            Ok(result.inner)
        }
    }

    fn insert<'a, 'b, 'c, T>(
        &'a self,
        entity: &'b Entity,
        components: &'c T,
    ) -> impl Future<Output = ()> + Send + 'c
    where
        'a: 'b,
        'b: 'c,
        T: Archetype<MySql> + Serializable<MySql> + Unpin + Send + 'static,
    {
        <T as Archetype<MySql>>::insert(&components, &self.pool, entity)
    }

    fn update<'a, T>(
        &'a self,
        entity: &'a Entity,
        components: &'a T,
    ) -> impl Future<Output = ()> + 'a
    where
        T: Archetype<MySql> + Serializable<MySql> + Unpin + Send + 'static,
    {
        <T as Archetype<MySql>>::update(&components, &self.pool, entity)
    }

    fn remove<'a, T>(&'a self, entity: &'a Entity) -> impl Future<Output = ()> + 'a
    where
        T: Archetype<MySql> + Removable<MySql> + Unpin + Send + 'static,
    {
        <T as Archetype<MySql>>::remove(&self.pool, entity)
    }
}
