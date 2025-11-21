// src/db.rs
use rbatis::Rbatis;
use rbdc_sqlite::driver::SqliteDriver;
use std::sync::Arc;

pub struct AppState {
    pub rb: Rbatis,
    pub tera: tera::Tera,
    pub admin_token: String, // [新增] 存储管理密钥
}

pub async fn init_db(url: &str) -> Rbatis {
    let rb = Rbatis::new();
    rb.init(SqliteDriver {}, url).unwrap();

    // 初始化表结构 (简单的迁移逻辑)
    let sql = "
    CREATE TABLE IF NOT EXISTS posts (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        title TEXT NOT NULL,
        content TEXT NOT NULL,
        slug TEXT UNIQUE NOT NULL,
        created_at TEXT NOT NULL
    );
    ";
    rb.exec(sql, vec![]).await.expect("Failed to init db");
    rb
}
