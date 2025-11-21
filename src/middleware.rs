// src/middleware.rs
use crate::db::AppState;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 1. 尝试从 URL Query 获取 token (?token=xxx)
    let query_token = req.uri().query().and_then(|q| {
        q.split('&')
            .find(|p| p.starts_with("token="))
            .map(|p| p.split_at(6).1)
    });

    // 2. 尝试从 Header 获取 token (Authorization: xxx)
    let header_token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    // 3. 获取我们生成的正确 Token
    let valid_token = &state.admin_token;

    // 4. 校验
    if let Some(token) = query_token {
        if token == valid_token {
            return Ok(next.run(req).await);
        }
    }

    if let Some(token) = header_token {
        if token == valid_token {
            return Ok(next.run(req).await);
        }
    }

    // 校验失败，拒绝访问
    Err(StatusCode::UNAUTHORIZED)
}
