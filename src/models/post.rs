// src/models/post.rs
// use rbatis::rbdc::datetime::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Post {
    pub id: Option<i64>,
    pub title: String,
    pub content: String,    // Markdown 原始内容
    pub slug: String,       // URL 友好的标识符
    pub created_at: String, // 简化处理，使用 String 存储 ISO 时间
}

// ---------------------------------------------------------
// 【修改这里】：在 {} 后面添加 "posts" 字符串
// 告诉 Rbatis 这个结构体对应数据库中的 "posts" 表
// ---------------------------------------------------------
rbatis::crud!(Post {}, "posts");

// 2. [新增] 显式定义通过 Slug 查询的方法
// 这会自动为 Post 结构体生成一个静态方法 select_by_slug
rbatis::impl_select!(Post{select_by_slug(slug: &str) -> Option => "`where slug = #{slug} limit 1`"} ,"posts");
