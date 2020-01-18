
# Rust DataBase Connectivity (RDBC)

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Docs](https://docs.rs/rdbc/badge.svg)](https://docs.rs/rdbc)
[![Version](https://img.shields.io/crates/v/rdbc.svg)](https://crates.io/crates/rdbc)

Love them or hate them, the [ODBC](https://en.wikipedia.org/wiki/Open_Database_Connectivity) and [JDBC](https://en.wikipedia.org/wiki/Java_Database_Connectivity) standards have made it easy to use a wide range of desktop and server products with many different databases thanks to the availability of database drivers implementing these standards.

This project provides a Rust equivalent API as well as reference implementations (drivers) for Postgres, MySQL, and SQLite. There is also an RDBC-ODBC driver being developed, that will allow ODBC drivers to be called via the RDBC API, so that it is also possible to connect to databases that do not yet have Rust drivers available.

Note that the provided RDBC drivers are just wrappers around existing database driver crates and this project is not attempting to build new drivers from scratch but rather make it possible to leverage existing drivers through a common API.

# Why do we need this when we have Diesel?

This is filling a different need. I love the [Diesel](https://diesel.rs/) approach for building applications but if you are building a generic SQL tool, a business intelligence tool, or a distributed query engine, there is a need to connect to different databases and execute arbitrary SQL. This is where we need a standard API and available drivers.

# RDBC API PoC

Note that the design of the RDBC API is intentionally modeled directly after ODBC and JDBC (except that indices are 0-based rather than 1-based) and that is likely to change to make this more idiomatic for Rust.

There is currently no `async` support and that will be addressed soon.

There are also design flaws with the current design, such as the limitation of only being able to create one prepared statement per connection.

Work is in progress of the next iteration of this project and there will definitely be breaking changes.


```rust
/// Represents database driver that can be shared between threads, and can therefore implement
/// a connection pool
pub trait Driver: Sync + Send {
    /// Create a connection to the database. Note that connections are intended to be used
    /// in a single thread since most database connections are not thread-safe
    fn connect(&self, url: &str) -> Result<Box<dyn Connection>>;
}

/// Represents a connection to a database
pub trait Connection {
    /// Create a statement for execution
    fn create(&mut self, sql: &str) -> Result<Box<dyn Statement + '_>>;

    /// Create a prepared statement for execution
    fn prepare(&mut self, sql: &str) -> Result<Box<dyn Statement + '_>>;
}

/// Represents an executable statement
pub trait Statement {
    /// Execute a query that is expected to return a result set, such as a `SELECT` statement
    fn execute_query(&mut self, params: &[Value]) -> Result<Box<dyn ResultSet + '_>>;

    /// Execute a query that is expected to update some rows.
    fn execute_update(&mut self, params: &[Value]) -> Result<u64>;
}

/// Result set from executing a query against a statement
pub trait ResultSet {
    /// get meta data about this result set
    fn meta_data(&self) -> Result<Box<dyn ResultSetMetaData>>;

    /// Move the cursor to the next available row if one exists and return true if it does
    fn next(&mut self) -> bool;
}
    
pub trait Row {
    fn get_i8(&self, i: u64) -> Result<Option<i8>>;
    fn get_i16(&self, i: u64) -> Result<Option<i16>>;
    fn get_i32(&self, i: u64) -> Result<Option<i32>>;
    fn get_i64(&self, i: u64) -> Result<Option<i64>>;
    fn get_f32(&self, i: u64) -> Result<Option<f32>>;
    fn get_f64(&self, i: u64) -> Result<Option<f64>>;
    fn get_string(&self, i: u64) -> Result<Option<String>>;
    fn get_bytes(&self, i: u64) -> Result<Option<Vec<u8>>>;

    // NOTE that only a subset of data types are supported so far in this PoC
    // and accessors need to be added for other types such as date and time
}

/// Meta data for result set
pub trait ResultSetMetaData {
    fn num_columns(&self) -> u64;
    fn column_name(&self, i: usize) -> String;
    fn column_type(&self, i: usize) -> DataType;
}
```

# Examples

## Execute a Query

```rust
let driver: Arc<dyn rdbc::Driver> = Arc::new(PostgresDriver::new());
let mut conn = driver.connect("postgres://user:password@127.0.0.1:5433")?;
let mut stmt = conn.prepare("SELECT a FROM test")?;
let mut rs = stmt.execute_query(&vec![])?;
while rs.next() {
  println!("{:?}", rs.get_string(1)?);
}
```

# Current Status

This is just an experimental PoC and is not currently suitable for anything. However, I do intend to make it useful pretty quickly and I am tracking issues [here](https://github.com/andygrove/rdbc/issues).

The immediate priorities though are:

- [x] Announce project and get initial feedback
- [x] Support parameterized queries
- [x] Support prepared statements
- [x] Implement simple SQL console CLI
- [ ] Design for async
- [ ] Support connection pooling
- [ ] Implement comprehensive unit and integration tests
- [ ] Add support for more data types
- [ ] Implement RDBC-ODBC bridge
- [ ] Implement dynamic loading of drivers at runtime

# License

RDBC is licensed under [Apache Licence, Version 2.0](/LICENSE).

# Contributing

Please refer to the [contributors guide](CONTRIBUTING.md) before contributing to this project and for information on building and testing locally.
