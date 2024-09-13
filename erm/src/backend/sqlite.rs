use std::{future::Future, marker::PhantomData};

use futures::Stream;
use sqlx::{Pool, Sqlite};

use crate::{condition::Condition, Archetype};

use super::Backend;

pub struct SqliteBackend<Entity> {
    pool: Pool<Sqlite>,
    _entity: PhantomData<Entity>,
}

impl<Entity> SqliteBackend<Entity> {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        SqliteBackend {
            pool,
            _entity: PhantomData,
        }
    }
}

impl<Entity> Backend<Sqlite, Entity> for SqliteBackend<Entity>
where
    Entity: for<'q> sqlx::Encode<'q, Sqlite>
        + for<'r> sqlx::Decode<'r, Sqlite>
        + sqlx::Type<Sqlite>
        + Unpin
        + Send
        + 'static,
    for<'entity> &'entity Entity: Send,
{
    fn init<T>(&self) -> impl Future<Output = Result<(), sqlx::Error>>
    where
        T: Archetype<Sqlite>,
    {
        <T as Archetype<Sqlite>>::create_component_tables::<Entity>(&self.pool)
    }

    fn list<T, Cond>(&self, condition: Cond) -> impl Stream<Item = Result<(Entity, T), sqlx::Error>>
    where
        T: Archetype<Sqlite> + Unpin + Send + 'static,
        Cond: Condition<Entity>,
    {
        <T as Archetype<Sqlite>>::list(&self.pool, condition)
    }

    fn get<T>(&self, entity: &Entity) -> impl Future<Output = Result<T, sqlx::Error>>
    where
        T: Archetype<Sqlite> + Unpin + Send + 'static,
    {
        <T as Archetype<Sqlite>>::get(&self.pool, entity)
    }

    fn insert<'a, 'b, 'c, T>(
        &'a self,
        entity: &'b Entity,
        components: &'c T,
    ) -> impl Future<Output = ()> + Send + 'c
    where
        'a: 'b,
        'b: 'c,
        T: Archetype<Sqlite> + Unpin + Send + 'static,
    {
        <T as Archetype<Sqlite>>::insert(&components, &self.pool, entity)
    }

    fn update<'a, T>(
        &'a self,
        entity: &'a Entity,
        components: &'a T,
    ) -> impl Future<Output = ()> + 'a
    where
        T: Archetype<Sqlite> + Unpin + Send + 'static,
    {
        <T as Archetype<Sqlite>>::update(&components, &self.pool, entity)
    }
}
