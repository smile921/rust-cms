// src/handlers/admin.rs
use crate::{db::AppState, models::post::Post};
use axum::{
    extract::{Multipart, Query, State}, // 引入 Query

    response::{Html, IntoResponse, Json},
};
use chrono::Local;
use serde_json::json;
use std::io::Write;
use std::sync::Arc;
use uuid::Uuid;

// [新增] 定义结构体接收 query param
#[derive(serde::Deserialize)]
pub struct AuthParams {
    token: String,
}

// 修改 editor_view，接收 Query 参数
pub async fn editor_view(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AuthParams>, // Axum 会自动解析 ?token=...
) -> impl IntoResponse {
    let mut ctx = tera::Context::new();
    // 将 token 注入到 HTML 模版中
    ctx.insert("admin_token", &params.token);

    let html = state.tera.render("editor.html", &ctx).unwrap();
    Html(html)
}
// // 渲染编辑器页面
// pub async fn editor_view(State(state): State<Arc<AppState>>) -> impl IntoResponse {
//     let ctx = tera::Context::new();
//     let html = state.tera.render("editor.html", &ctx).unwrap();
//     Html(html)
// }

// 处理文章发布
pub async fn create_post(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let title = payload.get("title").unwrap().as_str().unwrap();
    let content = payload.get("content").unwrap().as_str().unwrap();

    let new_post = Post {
        id: None,
        title: title.to_string(),
        content: content.to_string(),
        slug: Uuid::new_v4().to_string(), // 简单生成 Slug
        created_at: Local::now().to_rfc3339(),
    };

    let _ = Post::insert(&state.rb, &new_post).await;
    Json(json!({ "status": "ok", "slug": new_post.slug }))
}

// 处理图片上传
pub async fn upload_image(mut multipart: Multipart) -> impl IntoResponse {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        if name == "image" {
            let file_name = field.file_name().unwrap().to_string();
            let data = field.bytes().await.unwrap();

            // 生成唯一文件名
            let ext = std::path::Path::new(&file_name)
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("png");
            let new_name = format!("{}.{}", Uuid::new_v4(), ext);
            let filepath = format!("static/uploads/{}", new_name);

            // 确保目录存在
            std::fs::create_dir_all("static/uploads").unwrap();

            // 写入文件
            let mut file = std::fs::File::create(&filepath).unwrap();
            file.write_all(&data).unwrap();

            // 返回图片 URL (供编辑器使用)
            return Json(json!({
                "url": format!("/uploads/{}", new_name)
            }));
        }
    }
    Json(json!({ "error": "No file" }))
}
