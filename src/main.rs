// src/main.rs
// mod config;
mod db;
mod handlers;
mod middleware;
mod models;
mod utils;

use crate::db::{init_db, AppState};
use axum::{
    middleware::from_fn_with_state, // <--- 【在这里添加】
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tera::Tera;
use tower_http::services::ServeDir;

use dotenv::dotenv; // 引入 dotenv
use std::env; // 引入 env
use uuid::Uuid; // [新增]

use std::fs;
use std::path::Path; // 【修复】添加 Path 引用 // 【修复】添加 fs 引用

#[tokio::main]
async fn main() {
    // 1. 加载 .env 文件环境
    // dotenv().ok() 会尝试加载 .env 文件，如果文件不存在也不会报错（方便生产环境直接通过环境变量注入）
    dotenv().ok();

    // 2. 获取数据库连接地址
    // 如果环境变量中没有 DATABASE_URL，这里会 panic 报错，强制要求配置
    // let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

    // 或者：你可以提供一个默认值作为 fallback（可选）
    let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://cms.db".to_string());

    // 服务器 Host，默认为 0.0.0.0
    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

    // 服务器 Port，默认为 3000
    let port = env::var("SERVER_PORT").unwrap_or_else(|_| "3000".to_string());

    // 拼接完整地址
    let addr = format!("{}:{}", host, port);
    // 【修改这里】只为了打印好看，不影响实际监听
    // 如果监听的是 0.0.0.0，打印时显示 localhost，方便点击
    let display_host = if host == "0.0.0.0" {
        "127.0.0.1"
    } else {
        &host
    };
    // -----------------------------------------------------------
    // [新增] 生成并打印 Admin Token
    // -----------------------------------------------------------
    let admin_token = Uuid::new_v4().to_string();
    println!("\n============================================================");
    println!("SECURITY ALERT: Admin access requires the following token:");
    println!("Token: {}", admin_token);
    println!(
        "Editor URL: http://{}:{}/editor?token={}",
        display_host, port, admin_token
    );
    println!("============================================================\n");

    println!("Connecting to database: {}", db_url);

    // -------------------------------------------------------
    // 【新增】: 预处理数据库路径，确保父文件夹存在
    // -------------------------------------------------------
    if db_url.starts_with("sqlite://") {
        // 去掉前缀，拿到文件路径 (例如 "data/cms.db")
        let path_str = &db_url["sqlite://".len()..];
        let path = Path::new(path_str);

        // 如果路径中有父目录 (例如 "data")，则创建它
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                println!("Creating database directory: {:?}", parent);
                fs::create_dir_all(parent).expect("Failed to create database directory");
            }
        }
    }
    // -------------------------------------------------------
    // 3. 初始化数据库
    // 注意：init_db 函数签名是 &str，所以我们要传 &db_url
    let rb = init_db(&db_url).await;

    // 2. 初始化模版引擎
    let tera = Tera::new("templates/**/*").expect("Template parsing error");

    // 3. 共享状态
    // 将 token 存入 state
    let state = Arc::new(AppState {
        rb,
        tera,
        admin_token,
    });

    // 4. 构建路由
    let app = Router::new()
        // 公开路由 (博客浏览)
        .route("/", get(handlers::blog::list_posts))
        .route("/post/:slug", get(handlers::blog::get_post))
        .nest_service("/static", ServeDir::new("static"))
        .nest_service("/uploads", ServeDir::new("static/uploads"))
        // -------------------------------------------------------
        // 受保护路由 (需要 Token)
        // -------------------------------------------------------
        .nest(
            "/admin",
            Router::new()
                // 注意：为了方便 URL 结构，我将 /editor 移到了 /admin/editor
                // 或者你可以保持 /editor，只要用 .route 包裹并 layer 即可
                .route("/editor", get(handlers::admin::editor_view))
                .route("/api/posts", post(handlers::admin::create_post))
                .route("/api/upload", post(handlers::admin::upload_image))
                // 应用中间件
                .layer(from_fn_with_state(
                    state.clone(),
                    middleware::auth_middleware,
                )),
        )
        // 兼容原来的路径 (如果不想改 path，用这种方式组合)
        .route(
            "/editor",
            get(handlers::admin::editor_view).layer(from_fn_with_state(
                state.clone(),
                middleware::auth_middleware,
            )),
        )
        .route(
            "/api/posts",
            post(handlers::admin::create_post).layer(from_fn_with_state(
                state.clone(),
                middleware::auth_middleware,
            )),
        )
        .route(
            "/api/upload",
            post(handlers::admin::upload_image).layer(from_fn_with_state(
                state.clone(),
                middleware::auth_middleware,
            )),
        )
        .with_state(state);

    // 5. 启动服务
    // 使用拼接好的 addr 变量
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Listening on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}
