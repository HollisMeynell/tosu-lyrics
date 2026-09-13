#!/usr/bin/env just --justfile
#
# 构建入口。
#
# 注意：`pnpm build`（vite）会把 dist/ 清空，而 dist/ 里同时放着后端二进制、
# config.json5、lyric.db、字体等**运行产物**。因此这里先构建到临时目录，
# 再把前端产物并入 dist/，避免每次构建都丢掉运行产物。

default:
    @just --list

@install:
    pnpm i

[working-directory: './tosu-proxy']
@build-backend:
    cargo fmt
    cargo clippy --features=new
    cargo build -r --bin osu-lyric --features=new

@copy-backend:
    cp ./tosu-proxy/target/release/osu-lyric.exe ./dist/osu-lyric.exe

@build-frontend:
    pnpm build --outDir dist-new --emptyOutDir
    rm -rf ./dist/assets ./dist/index.html
    cp -r ./dist-new/assets ./dist-new/index.html ./dist-new/LRC.otf ./dist-new/osu.svg ./dist/
    rm -rf ./dist-new

@build: build-frontend build-backend copy-backend
