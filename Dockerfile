# Stage 1: Build 环境
FROM rust:1.75 as builder

WORKDIR /app
COPY . .

# 编译 Release 版本
# 注意：这会编译 mdbook 库，第一次构建可能需要几分钟
RUN cargo build --release

# Stage 2: Runtime 环境
FROM debian:bookworm-slim

WORKDIR /app

# 安装必要的运行时依赖
# - ca-certificates: HTTPS 支持
# - sqlite3: 调试用 (可选)
# - openssl: 如果你的某些依赖动态链接了 SSL
RUN apt-get update && apt-get install -y \
    ca-certificates \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

# 从构建阶段复制二进制文件
COPY --from=builder /app/target/release/rust_cms /app/rust_cms

# 复制模版和静态资源 (编辑器界面需要)
COPY --from=builder /app/templates /app/templates
COPY --from=builder /app/static /app/static

# 创建必要的挂载点目录
RUN mkdir -p data/book_workspace && mkdir -p certs

# 设置环境变量默认值
ENV RUST_LOG=info
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=3000
ENV DATABASE_URL=sqlite://data/cms.db
ENV ENABLE_HTTPS=false

# 暴露端口
EXPOSE 3000

# 启动
CMD ["./rust_cms"]
