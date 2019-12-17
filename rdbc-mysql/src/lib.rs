//! MySQL RDBC Driver
//!
//! This crate implements an RDBC Driver for the `mysql` crate.
//!
//! The RDBC (Rust DataBase Connectivity) API is loosely based on the ODBC and JDBC standards.
//!
//! ```rust,ignore
//! use rdbc_mysql::MySQLDriver;
//! let driver = MySQLDriver::new();
//! let conn = driver.connect("mysql://root:password@localhost:3307/mysql").unwrap();
//! let stmt = conn.create_statement("SELECT foo FROM bar").unwrap();
//! let rs = stmt.execute_query().unwrap();
//! let mut rs = rs.borrow_mut();
//! while rs.next() {
//!   println!("{}", rs.get_string(1));
//! }
//! ```

use std::cell::RefCell;
use std::rc::Rc;

use mysql as my;
use rdbc;
use mysql::QueryResult;
use mysql::serde::export::PhantomData;

pub struct MySQLDriver {

}

impl MySQLDriver {

    pub fn new() -> Self {
        MySQLDriver {}
    }

    pub fn connect(&self, url: &str) -> rdbc::Result<Rc<RefCell<MySQLConnection>>> {
        let opts = my::Opts::from_url(&url).expect("DATABASE_URL invalid");
        let conn = my::Conn::new(opts).unwrap();
        Ok(Rc::new(RefCell::new(MySQLConnection { conn, phantom: PhantomData })))
    }
}

pub struct MySQLConnection<'a> {
    conn: my::Conn,
    phantom: PhantomData<&'a usize>
}

impl<'a> MySQLConnection<'a> {

    pub fn execute_query(&'a mut self, sql: &str) -> rdbc::Result<Rc<RefCell<dyn rdbc::ResultSet<'a>>>> {
        let result: QueryResult<'a> = self.conn.query(sql).unwrap();
        let rs: dyn rdbc::ResultSet<'a> = MySQLResultSet { result, row: None };
        Ok(Rc::new(RefCell::new(rs)))
    }

}

pub struct MySQLResultSet<'a> {
    result: my::QueryResult<'a>,
    row: Option<my::Result<my::Row>>
}

impl<'a> rdbc::ResultSet<'_> for MySQLResultSet<'a> {

    fn next(&mut self) -> bool {
        self.row = self.result.next();
        self.row.is_some()
    }

    fn get_i32(&self, i: usize) -> Option<i32> {
        match &self.row {
            Some(Ok(row)) => row.get(i-1),
            _ => None
        }
    }

    fn get_string(&self, i: usize) -> Option<String> {
        match &self.row {
            Some(Ok(row)) => row.get(i-1),
            _ => None
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn query_direct() {

        let url = "mysql://root:secret@127.0.0.1:3307/mysql";

        let opts = my::Opts::from_url(&url).expect("DATABASE_URL invalid");
        let mut conn = my::Conn::new(opts).unwrap();

        let result = conn.query("SELECT 1").unwrap();

        for row in result {
            let row = row.unwrap();
            let value: Option<u32> = row.get(0);
            println!("{}", value.unwrap());
        }
    }

    #[test]
    fn query_via_rdbc() {

        let driver = MySQLDriver::new();

        let conn = driver
            .connect("mysql://root:secret@127.0.0.1:3307")
            .unwrap();

        let mut conn = conn.as_ref().borrow_mut();

        let rs = conn.execute_query("SELECT 1").unwrap();
        let mut rs = rs.as_ref().borrow_mut();

        while rs.next() {
            println!("{:?}", rs.get_i32(1))
        }
    }
}
