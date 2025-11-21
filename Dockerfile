# Stage 1: Build
FROM rust:1.75 as builder

WORKDIR /app
COPY . .

# 针对 Alpine 静态编译需要 musl 库，这里为了简单使用 debian slim
# 如果需要极小镜像，可以配置 musl target
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

WORKDIR /app

# 安装 SQLite 运行时依赖 (如果非静态编译)
RUN apt-get update && apt-get install -y sqlite3 ca-certificates && rm -rf /var/lib/apt/lists/*

# 复制二进制文件
COPY --from=builder /app/target/release/rust_cms /app/rust_cms

# 复制静态资源和模版
COPY --from=builder /app/templates /app/templates
COPY --from=builder /app/static /app/static


# 设置环境变量默认值
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=3000
ENV DATABASE_URL=sqlite://data/cms.db


# 3. 声明卷 (告诉 Docker 这两个目录需要持久化)
# 虽然这里声明了 VOLUME，但实际运行时最好显式挂载
VOLUME ["/app/data", "/app/static/uploads"]

# 暴露对应的端口
EXPOSE 3000

# 创建数据目录
RUN mkdir -p static/uploads

# 启动命令
CMD ["./rust_cms"]
