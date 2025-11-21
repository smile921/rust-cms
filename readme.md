总结与工作流回顾
Web 框架: 使用 Axum 搭建了路由 (/, /editor, /api/*)。
数据库: 使用 SQLite 存储文章元数据，Rbatis 实现了 CRUD（见 models.rs 和 handlers/blog.rs）。
Markdown: 后端使用 pulldown-cmark 进行渲染，前端使用 EasyMDE 进行编辑和上传。
图片上传: handlers/admin.rs 中的 upload_image 函数处理 Multipart 表单，将图片存入 static/uploads，并返回 URL 给编辑器插入 Markdown 中。
Docker: 提供了 Dockerfile，支持一键打包部署。
模块化: 代码按功能拆分为 db, models, handlers, utils，符合清晰架构要求。
如何运行：
在本地安装 Rust。
运行 cargo run。
访问 http://localhost:3000/editor 撰写文章并上传图片。
访问 http://localhost:3000/ 查看博客列表。
构建镜像：docker build -t rust-cms .

安全性：不在代码中硬编码敏感信息（如数据库密码，虽然 SQLite 通常没有密码，但这是一个好习惯）。
灵活性：在开发环境使用 sqlite://dev.db，在 Docker 生产环境中可以通过 docker run -e DATABASE_URL=... 动态注入不同的路径，而无需重新编译代码。
一致性：符合 The Twelve-Factor App 的配置原则。

这样修改后，你在本地运行时可以通过修改 .env 改变端口；在 Docker 部署时，也可以通过 docker run -e SERVER_PORT=8080 ... 轻松修改端口，无需重新构建镜像。

# macOS / Linux
export RUSTUP_DIST_SERVER=https://mirrors.tuna.tsinghua.edu.cn/rustup
export RUSTUP_UPDATE_ROOT=https://mirrors.tuna.tsinghua.edu.cn/rustup/rustup
rustup update


运行命令 (Docker Run)
当你运行容器时，必须把宿主机的目录挂载进去：
code
Bash
# 假设你在项目根目录运行
# -v $(pwd)/data:/app/data  --> 将宿主机的 ./data 映射到容器的 /app/data
# -v $(pwd)/uploads:/app/static/uploads --> 持久化上传的图片

docker run -d \
  -p 3000:3000 \
  -v $(pwd)/data:/app/data \
  -v $(pwd)/uploads:/app/static/uploads \
  --name my-rust-cms \
  rust-cms
总结
位置：推荐放在 data/cms.db。
自动创建：文件会自动创建，但文件夹不会。
代码增强：在 main.rs 中增加 fs::create_dir_all 逻辑来保证文件夹存在。
Docker：必须通过 -v 挂载 data 目录，否则重启容器数据即丢。


==========================================
Listening on http://0.0.0.0:3000
访问拦截：
如果直接访问 http://localhost:3000/editor -> 401 Unauthorized。
如果访问控制台打印的带 Token 的链接 -> 成功进入编辑器。
在编辑器中上传图片或发布文章 -> 成功（Token 已自动携带）。
这样你就拥有了一个类似 Jupyter Notebook 的简易而安全的认证机制，无需实现复杂的用户名密码数据库存储。
54.3s
