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

## Relations

Relations can be specified and used later in queries.

It's done by adding a foreign key with `fk` attribute to a field. Related name in queries will then be the snake_case related struct name. For example: MyRoom=>my_room. The related name can be changed with the `related` attribute.

```rust
#[derive[Sqlo, Debug, PartialEq]]
struct House {
    id: i64,
    name: String,
    width: i64,
    height: i64
}

struct Room {
    id: i64,
    #[sqlo(fk = "House")]
    house_id: i64
}
// will use myhouse.room in queries

// or

struct Room {
    id: i64,
    #[sqlo(fk = "House", related = "therooms")]
    house_id: i64
    bed: bool
}
// will use myhouse.therooms in queries.

```

There is a type check so the `fk` field must have the same type as target struct's primary key.

Entities and Relations are kept in a `.sqlo` directory which is created at compile time. Depending the order of compilation,it might fails at first glance if a `Sqlo Entity` is targeted in a relation but not yet parsed . Just rebuild a second time and it will pass.

`.sqlo` may or not be added to VCS. Although it isn't its primary purpose, versionning `.sqlo` appears to add some more security in case of code change. The content is simple json files, which are very easy to read.

The `fk` literal can be identifier (`"MyRoom"`) or a path (`"mycrate::mydir::MyRoom"`).

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

### Select

Select queries are performed with the `select!` macro.

Let's use theese struct for this chapter.

```rust
#[derive[Sqlo, Debug, PartialEq]]
struct House {
    id: i64,
    name: String,
    width: i64,
    height: i64
}


struct Room {
    id: i64,
    #[sqlo(fk = "House", related = "therooms")]
    house_id: i64
    bed: bool
}

```

All SQL keywords and features are not implemented now. You can find a feature list of what is working right now below.

### Introduction

Basically it uses `sqlx::query_as` under the hood and just translate the query:

```rust
select![House where id == 1].fetch_one(&pool).await

//roughly is translated into

query_as![House, "SELECT DISTINCT id, name, width, height FROM house where id=?", 1].fetch_one(&pool).await
```

Some generals rules :

- It's rust syntax not sql: that's why we use `==` instead of `=`.
- `DISTINCT` is always added.

### The WHERE clause

It's an aggregate of binary expressions, here are some use cases, by SQL usage:

- field: `select![House where id == 1]`
- binary `operator: `select![House where width >= 1]`
- IS NULL: `select![House where width == None]`
- IS NOT NULL: `select![House where width != None]`
- BETWEEN: `select![House where  width > 1 && width <5]`
- use of parenthesis: `select![House where (width==1 || width==2) && height==4]`
- NOT: `select![House !(width>5)]`
- IN (range expression) :
  - `select![HOUSE where id..(1,3,4)` as tuple
  - `select![HOUSE where id..[1,2,3]]` as array
  - `select![HOUSE where id..(1..4)]` as exclusive range
  - `select![HOUSE where id..(1..=4)` as inclusive range
  - `let [a,b,c] = myarray; select![HOUSE where id..(a,b,c)]` for known size array. Plain array/vec not supported now.
- column from join: see [JOIN in where clause](####JOIN-in-where-clause)

### INNER JOIN:

#### Retrieving related rows

You can access related row/collections via a "virtual field", the specified with `fk` attribute.

- A row is accessed by indexing its primary key (`House[1]`,`House[myvar]`,`House[some.field]` or `House[someindex[1]]`).
- "virtual" related fielda is accessed by its related name: `House[1].therooms`.

```rust
let a = 1;
let romms: Vec<Room> = select![House[a].therooms where bed == true]
    .fetch_all(&pool).await.unwrap();
```

#### JOIN in where clause

```rust
select![House where therooms.bed == true]
select![House where width>3 && therooms.bed == true]
```

Rust items can be used as parameters:

```rust
// Variables
let width = 1;
select![House where height == width] // Right hand part of the expression will refere to the variable width not the field of house
select![House where width == width] // is possible

// Indexing
let array = [1 , 2, 3]
select![House where width == array[0]]

// struct field
struct A {b:i32}
let a = A{b:2}
select![House where width == a.b]
```
