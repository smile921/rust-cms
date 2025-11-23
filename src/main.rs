// src/main.rs
// mod config;
mod book_gen;
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

    // -------------------------------------------------------
    // [新增]：启动时先构建一次，确保 book 目录存在
    // -------------------------------------------------------
    println!("Initializing mdBook content...");
    if let Err(e) = book_gen::rebuild_book(&rb).await {
        eprintln!("Warning: Initial book build failed: {}", e);
    }

    // mdBook 默认输出目录: data/book_workspace/book
    let book_output_dir = "data/book_workspace/book";
    // 确保目录存在，防止 ServeDir 崩溃
    fs::create_dir_all(book_output_dir).unwrap();
    // 3. 共享状态
    // 将 token 存入 state
    let state = Arc::new(AppState {
        rb,
        tera,
        admin_token,
    });

    // 4. 构建路由
    let app = Router::new()
        // 1. 系统静态资源 (编辑器用的 JS/CSS, 上传的图片)
        // 注意：mdBook 也会生成 css/js，不要跟系统本身的冲突
        // 这里的 /static 是给编辑器页面用的
        .nest_service("/static", ServeDir::new("static"))
        // 图片上传目录 (mdBook 里引用图片也用这个路径)
        .nest_service("/uploads", ServeDir::new("static/uploads"))
        // 2. 后台管理路由 (动态) - 必须放在静态托管之前匹配
        .nest(
            "/admin",
            Router::new()
                .route("/editor", get(handlers::admin::editor_view))
                .route("/api/posts", post(handlers::admin::create_post))
                .route("/api/upload", post(handlers::admin::upload_image))
                .layer(from_fn_with_state(
                    state.clone(),
                    middleware::auth_middleware,
                )),
        )
        // 兼容旧路由
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
        // 3. [核心变更] 博客前台：直接托管 mdBook 生成的 HTML
        // 只要不是上面的 API 路由，统统去 book 目录找文件
        // Fallback service 会处理 index.html
        .fallback_service(ServeDir::new(book_output_dir))
        .with_state(state);

    // 5. 启动服务
    // 使用拼接好的 addr 变量
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Listening on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}
