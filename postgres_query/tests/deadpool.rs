#![cfg(feature = "deadpool")]

use deadpool_postgres::{Client, Config, Pool};
use postgres_query::{query, Query};
use std::env;

fn connect() -> Pool {
    let pgconf: tokio_postgres::Config = env::var("POSTGRES_DB_CONFIG")
        .unwrap_or_else(|_| "user=postgres_query_test host=localhost".to_owned())
        .parse()
        .expect("Failed to open string");
    let mut cfg = Config::default();
    if let tokio_postgres::config::Host::Tcp(s) = &pgconf.get_hosts()[0] {
        cfg.host.replace(s.to_string());
    }
    if let Some(u) = pgconf.get_user() {
        cfg.user.replace(u.to_string());
    }
    if let Some(p) = pgconf.get_password() {
        cfg.password.replace(String::from_utf8_lossy(p).to_string());
    }
    if let Some(db) = pgconf.get_dbname() {
        cfg.dbname.replace(db.to_string());
    }
    cfg.create_pool(tokio_postgres::NoTls).unwrap()
}

#[tokio::test]
async fn simple_query() {
    let pool = connect();
    let client: Client = pool.get().await.unwrap();
    let query: Query = query!("SELECT 14");
    let res = query
        .fetch_one::<(i32,), _>(&client)
        .await
        .expect("Failure");
    assert_eq!(res, (14,));
}
