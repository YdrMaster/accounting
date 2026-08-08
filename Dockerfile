# syntax=docker/dockerfile:1
# 多阶段构建：前端 dist → musl 静态二进制 → distroless 最小运行时
# 构建全部发生在 CI；本地 podman build 等价可用

# ─── stage 1: 前端构建 ───
FROM node:22-alpine AS frontend
WORKDIR /web
# 依赖层缓存：仅清单变化才重装依赖
COPY accounting-web/package.json accounting-web/package-lock.json accounting-web/.npmrc ./
RUN npm ci
COPY accounting-web/ ./
RUN npm run build

# ─── stage 2: Rust 静态编译（cargo-chef 缓存依赖层）───
FROM rust:alpine AS chef
# cargo-chef 从源码安装，避免依赖第三方镜像（国内镜像站无白名单）
RUN cargo install cargo-chef --locked
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
# 强制 musl 全静态（rust:alpine 默认产物仍动态链接 ld-musl，distroless/static 无该解释器）。
# 必须用 target 专属变量而非全局 RUSTFLAGS：全局会套到 host 侧 proc-macro 上导致其无法编译；
# 同时显式 --target 让 cargo 区分 host/target，proc-macro 不受影响。
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-C target-feature=+crt-static"
COPY --from=planner /app/recipe.json recipe.json
# 依赖层：仅 recipe.json 变化时重编译依赖
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl -p accounting-api -p accounting-auth \
 # musl 静态断言（static-PIE：无 INTERP 段、无 NEEDED 依赖；注意 musl 的 ldd 对
 # static-PIE 会打印 loader 路径且退出 0，不能用 ldd 判断）
 && ! readelf -l target/x86_64-unknown-linux-musl/release/accounting-api | grep -q INTERP \
 && ! readelf -d target/x86_64-unknown-linux-musl/release/accounting-api | grep -q NEEDED \
 && ! readelf -l target/x86_64-unknown-linux-musl/release/auth-admin | grep -q INTERP \
 && ! readelf -d target/x86_64-unknown-linux-musl/release/auth-admin | grep -q NEEDED

# ─── stage 3: 无发行版运行时（容器内 root；rootless podman 下映射为宿主用户）───
FROM gcr.io/distroless/static-debian12
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/accounting-api /app/accounting-api
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/auth-admin /app/auth-admin
COPY --from=frontend /web/dist /app/dist
# 让 podman exec <容器> auth-admin 可直接调用
ENV PATH="/app"
VOLUME ["/data"]
EXPOSE 3000
ENTRYPOINT ["/app/accounting-api", \
            "--db", "/data/my.db", \
            "--auth-db", "/data/auth.db", \
            "--static-dir", "/app/dist", \
            "--lang", "zh-cn", \
            "--port", "3000"]
