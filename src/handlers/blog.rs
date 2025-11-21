// src/handlers/blog.rs
use crate::{db::AppState, models::post::Post, utils::markdown::render_markdown};
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
};
use std::sync::Arc;
// use rbatis::rbdc::db::ExecResult;

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
    // [修改] 使用我们在 models/post.rs 中通过 impl_select! 定义的方法
    let post = Post::select_by_slug(&state.rb, &slug).await.unwrap();

    // impl_select! 指定了返回 -> Option，所以这里 post 是 Option<Post>
    if let Some(p) = post {
        let html_content = render_markdown(&p.content);

        let mut ctx = tera::Context::new();
        ctx.insert("post", &p);
        ctx.insert("content_html", &html_content);

        let html = state.tera.render("post.html", &ctx).unwrap();
        return Html(html);
    }

    Html("<h1>404 Not Found</h1>".to_string())
}
