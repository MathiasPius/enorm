//! # Entity Relational Mapping
//!
//! Enables persistence for your Entity-Component architectures using [sqlx]
//!
//! ## Quick Example
//!
//! ```rust
//! # #[tokio::main]
//! # async fn example1() {
//! use enorm::prelude::*;
//!
//! // Construct an in-memory SqliteBackend using Uuids for entity IDs.
//! let backend = SqliteBackend::<uuid::Uuid>::in_memory().await;
//!
//! // Define our components
//! #[derive(Component)]
//! struct Position {
//!     pub x: i64,
//!     pub y: i64,
//! }
//!
//! #[derive(Debug, Component)]
//! struct DisplayName {
//!     pub name: String,
//! }
//!
//! // Spawn a new entity (generates a new entity id),
//! // with the name "Position 1", and a position.
//! let pos1 = backend.spawn(&(
//!     DisplayName {
//!         name: "Position 1".to_string(),
//!     },
//!     Position {
//!         x: 100,
//!         y: 200
//!     }
//! )).await;
//!
//! // Spawn a second named position
//! let pos2 = backend.spawn(&(
//!     DisplayName {
//!         name: "Position 2".to_string(),
//!     },
//!     Position {
//!         x: -10000,
//!         y: -5
//!     }
//! )).await;
//!
//! # use futures::stream::StreamExt as _;
//! // Construct an iterator over all components with a DisplayName & Position
//! let query = backend.list::<(DisplayName, Position)>()
//!     .components()
//!     .fetch();
//!
//! // Streams must be pinned: https://rust-lang.github.io/async-book/04_pinning/01_chapter.html
//! let mut names = std::pin::pin!(query);
//! while let Some(Ok((display_name, position))) = names.next().await {
//!     println!("name: {} at {},{}", display_name.name, position.x, position.y);
//! }
//! // name: Position 1 at 100,200
//! // name: Position 2 at -10000,-5
//!
//! // Remove the DisplayName component from our pos2 entity.
//! backend.remove::<DisplayName>(&pos2).await;
//!
//! // Fetch the name of our first position.
//! let pos1_name = backend.get::<DisplayName>(&pos1).await.unwrap();
//! assert_eq!(pos1_name.name, "Position 1");
//!
//! // Update the name of our second position.
//! backend.update(&pos2, &DisplayName { name: "Second Position".to_string() }).await;
//!
//! # }
//! ```
//!
//! See [github.com/MathiasPius/enorm](https://github.com/MathiasPius/enorm/tree/main/examples) for more examples.

pub mod archetype;
pub mod backend;
pub mod component;
pub mod condition;
pub mod cte;
pub mod entity;
pub mod reflect;
pub mod row;
pub mod serialization;
pub mod tables;

#[cfg(feature = "bundled")]
pub use ::sqlx;

pub mod prelude {
    #[cfg(feature = "derive")]
    pub use enorm_derive::*;

    pub use crate::archetype::Archetype;
    pub use crate::backend::*;
    pub use crate::component::{ColumnDefinition, Component};
    pub use crate::condition;
    pub use crate::cte::*;
    pub use crate::reflect::Reflect;
    pub use crate::serialization::{Deserializeable, Serializable};
    pub use crate::tables::Removable;
}
