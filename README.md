# Enorm is Not an ORM.

Enorm is a library for using [SQLx](https://github.com/launchbadge/sqlx)-compatible storage backends as a persistence layer for an [Entity-Component-System](https://en.wikipedia.org/wiki/Entity_component_system) (ECS) architecture.

While similar to [Object-Relational Mapping](https://en.wikipedia.org/wiki/Object%E2%80%93relational_mapping), Enorm enforces simplicity, and prevents hugely complex queries by imposing the following ECS-based limits and patterns:

### 1. Enorm can only store *Components*

A component is (in most cases) a plain struct type, containing [zero](#marker-traits) or more SQLx-de/serializable fields.

Since a component can only be made up of fields that can each be stored in a single column of the underlying database, they are very "flat". One-to-Many or Many-to-Many relationships can only be expressed through *Entity ID* fields, referencing other *Entities*.

```rust
#[derive(Component)]
struct Name(String);

#[derive(Component)]
struct Age(u8);
```

### 2. *Entities* are just the sum of their *Component Instances*.

Entities don't exist, except as *Component Instances* with the same associated *Entity ID*.

You can neither create nor destroy an *Entity* directly. Entities are "created" when a *Component* is instantiated with its *Entity ID*, and cease to exist when the last *Component Instance* associated with it is deleted.

```rust
// "Andrea" is created, using a tuple of Components:
enorm.insert(1234, &(Name("Andrea"), Age(32)));

// Andrea ceases to exist.
enorm.remove::<(Name, Age)>(1234);
```

### 3. *Archetypes* are a convenience for grouping *Components*.

Like *Entities*, *Archetypes* don't exist per se, but simply groups *Components*
into meta-types, that can be used in place of querying components individually (or through tuples):

```rust
// Instead of inserting components as collections of tuples,
// we can define such a collection as an Archetype:
#[derive(Archetype)]
struct Person {
    name: Name,
    age: Age,
}
 
enorm.insert(1234, &Person {
    name: Name("Andrea"),
    age: Age(32)
});

enorm.get::<Person>(1234); // <-- Person { name: Name("Andrea"), age: Age(32) }

// But Archetypes are just an abstraction, underneath it is still just Components:
enorm.get::<Name>(1234); // <-- Name("Andrea")
```
