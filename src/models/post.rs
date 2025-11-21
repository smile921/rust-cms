// src/models/post.rs
use rbatis::rbdc::datetime::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Post {
    pub id: Option<i64>,
    pub title: String,
    pub content: String,    // Markdown 原始内容
    pub slug: String,       // URL 友好的标识符
    pub created_at: String, // 简化处理，使用 String 存储 ISO 时间
}

// Rbatis 宏映射
rbatis::crud!(Post {});
