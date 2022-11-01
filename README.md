# sqlo

Syntactic sugar for sqlx.

## What is it ?

Sqlo is another attempt to make a nice/pleasant API in Rust using relational database.

Sqlo is built on top of sqlx and uses sqlx macros so you keep all the power of sqlx at compile time with less boiler plate.

Right now, only sqlite is supported. PR welcomed :-)

## Install

```toml
#Cargo.toml
sqlo = {version="0.1.0", features=["sqlite"]}
```

## How it works ?

Just derive `Sqlo` macro:

```rust
#[derive(Sqlo, PartialEq, Debug)]
struct MyTable {
    id: i64,
    text: String,
    maybe: Option<i64>,
}
...
//
let pool = get_my_db_pool().await?;

// create row
let a = MyTable::create(&pool, "hello", None).await?;

// retrieve row by primary_key
let mut b = MyTable::get(&pool, a.id).await?
assert_eq!(a,b);

// update a full row with instance
b.text = "bye".to_string();
b.save(&pool).await?;

// update selected fields only
let b = update_MyTable![b, text="I'm Back", maybe=Some(12)](&pool).await?;

// or the same by primary_key
let pk = b.id;
let c = update_MyTable![pk = pk, text="I'm reBack", maybe=None](&pool).await?;

// remove by instance
c.remove(&pool).await?
//or delete with pk
MyTable.delete(&pool, pk).await?
```

## Attributes

#### Struct attributes

Every attributes (struct or field) are expected under the `sqlo` attribute.

##### tablename

Sqlo expects the tablename be the struct name converted to snake_case.
You can change it with tablename attribute :

```rust
#[derive(Sqlo)]
#[sqlo(tablename="another_name")]
struct MyTable {}
```

### Fields Attribute

#### primary_key

By default, the `id` field is used as Primary Key. Change it with the `primary_key` flag:

```rust
#[derive(Sqlo)]
struct MyTable {
    #[sqlo(primary_key)]
    name: String
}
```

#### column

By default, the field name is used as column name. Change it with the `column` flag:

```rust
#[derive(Sqlo)]
struct MyTable {
    #[sqlo(column="what_a_column_name")]
    name: String
}
```

#### create_fn and create_arg

By default, `Sqlo` relies on Database Backend `auto-increment` feature for primary key when adding a new row with the `create` method. So, by default there is no argument to provide for primary_key.

```rust
#[derive(Sqlo)]
struct MyTable {
    id: i64,
    name: String
}
//...
let instance = MyTable::create(&pool, "some string");
```

This can be changed in two ways:

- `create_arg`: allows primary_key argument in create

```rust
#[derive(Sqlo)]
struct MyTable {
    #[sqlo(create_arg)]
    id: i64,
    name: String
}
//...
let instance = MyTable::create(&pool, 234234, "some string");
assert_eq!(instance.id, 234234);
```

- `create_fn`: provides callable as string which is called as primary_key value.

```rust
#[derive(Sqlo)]
struct MyTable {
    #[sqlo(create_fn="uuid::Uuid::new_v4")]
    id: Uuid,
    name: String
}
//...
let instance = MyTable::create(&pool, "some string");
assert_eq!(instance.id, Uuid("someuuidv4"))
```

#### type_override

Under the hood `Sqlo` uses sqlx's `query_as!` for `get`, `create` and `update`.
This attribute gives you access to [sqlx type override](https://docs.rs/sqlx/latest/sqlx/macro.query_as.html#column-type-override-infer-from-struct-field) so the query uses `select field as "field:_", ...` instead of `select field, ...`?

## Methods

### Introduction

- Every method returning an instance of the derived struct uses `sqlx::query_as!` under the hood.
- The first parameter is always the database connection.

### Parameters type

- `i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64`, `bool` are NOT passed by reference.
- `String` expects `&str`.
- `BString` expects `&BStr`.
- `Vec<u8>` expects `&[u8]`.
- `Option<T>` expects `Option<T>`
- Everything else is passed by reference.

```rust
#[derive(Sqlo)]
struct MyTable {
    #[sqlo(create_arg)]
    id: i64,
    name: String,
    some_type: Option<String>
}
///...
let instance = MyTable::create(23, "a aname", Some("some_type".to_string()));
```

### get

Get a row by his primary_key.

Return: `sqlx::Result<T>`

```rust
let row = MyTable::get(&pool,   23).await?
assert_eq!(row.id, 23);
```

### create

Insert a new row.

Handling of primary_key, see [create_fn and create_arg](####create_fn-and-create_arg).

Return: `sqlx::Result<T>`

```rust
#[derive(Sqlo)]
struct MyTable {
    id: i64,
    name: String,
    alive: bool,
    members: Option<i64>
}
//...
let mytable = MyTable::create(&pool, "bla", true, None).await?;
assert_eq!(mytable.name, "bla".to_string());
```

### save

Update a full row or insert it if exists. It's an UPSERT based on primary_key.

Return: `sqlx::Result<DB::QueryResult>`

```rust
#[derive(Sqlo, Debug, PartialEq)]
struct MyTable {
    #[sqlo(create_arg)]
    id: i64,
    name: String,
    alive: bool,
    members: Option<i64>
}
//...
let mut mytable = MyTable{id:1, name:"bla".to_string(), alive:true, membres:None};
mytable.save(&pool).await?;
// equivalent to MyTable::create(&pool, 1, "bla", true, None).await?
mytable.members = Some(345);
mytable.save(&pool);
let mytable2 = MyTable::get(&pool, 1).await?;
assert_eq!(mytable, mytable2);
```

### delete

Delete a row by it's primary key.

Return: `sqlx::Result<DB::QueryResult>`>>

```rust
MyTable::delete(&pool, 43).await?
```

### remove

Delete a row via its instance. `remove` takes ownership of the instance which is not usable after.

Return: `sqlx::Result<DB::QueryResult>`

```rust
myrow.remove(&pool).await?;
myrow.some_field = 1; // compile_error
```

## Macros

### update_Table!

Rust handles variable number of argument with macro (like vec!, ...), but it can't put as method.
So `Sqlo` generates an update macro which is named as follow : `update_MyStruct`.

`sqlx::query_as!` witch `fetch_one` is used under the hood.

Return: `Fn(&DBPool) -> Future<sqlx::Result<T>>`

**The output of the macros has to be called with a database pool. It won't work with a simple database connection.**

It supports the followings formats:

```rust
update_MyStruct![instance; field1=value1, field2=value2](&pool).await?
// this format takes ownership of instance

// or

update_Mystruct![pk = value, field1=value1, field2=value2](&pool).await?
```

```rust
#[derive[Sqlo, Debug, PartialEq]]
struct House {
    id: i64,
    name: String,
    width: i64,
    height: i64
}
//...
let house = House::get(&pool, 2);
let house = update_House![house; name= "bla", width=34](&pool).await?;
let other_update = update_House!(pk=2, height=345)(&pool).await?;

```

## Relations

- use fk
- related pour le lom
- same type needed
- compile might fail (order of compilation), relaunch
- check: struct exist, same type (in pk)
- use ident or some::path::ident
- .sqlo dir: may or not be versionned
- no one letter table (because if joins)
  -sqlo_select: user field or related.field(for fk)

divers: ajouter option parse_only
