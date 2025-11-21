// src/handlers/blog.rs
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
};
use std::sync::Arc;
use crate::{db::AppState, models::post::Post, utils::markdown::render_markdown};
use rbatis::rbdc::db::ExecResult;

// 首页：文章列表
pub async fn list_posts(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // 使用 Rbatis 查询所有文章
    let posts = Post::select_all(&state.rb).await.unwrap_or_default();

    let mut ctx = tera::Context::new();
    ctx.insert("posts", &posts);
    let html = state.tera.render("index.html", &ctx).unwrap();
    Html(html)
}

// 详情页：渲染 Markdown
pub async fn get_post(
    Path(slug): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let post = Post::select_by_column(&state.rb, "slug", slug).await.unwrap();

    if let Some(p) = post.first() {
        let html_content = render_markdown(&p.content);

        let mut ctx = tera::Context::new();
        ctx.insert("post", p);
        ctx.insert("content_html", &html_content);

        let html = state.tera.render("post.html", &ctx).unwrap();
        return Html(html);
    }

    Html("<h1>404 Not Found</h1>".to_string())
}
