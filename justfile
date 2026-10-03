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
    # 随包默认字体：独立放在 static/ 下（静态服务优先查 ./static），
    # 这样"默认字体"与"上传覆盖到工作目录的 LRC.otf / tLRC.otf"是两个文件
    mkdir -p ./dist/static
    cp ./static/LRC.otf ./static/tLRC.otf ./dist/static/

@build: build-frontend build-backend copy-backend

# 给已构建的 exe 做 Authenticode 签名。
# 需要正式代码签名证书（-PfxPath / OSU_LYRIC_PFX_PATH）；没有证书时会明确
# 说明原因并以非零码退出，不会生成假证书，也不会修改 Defender / SmartScreen。
@sign exe="./dist/osu-lyric.exe":
    powershell -NoProfile -ExecutionPolicy Bypass -File ./scripts/sign-windows.ps1 -ExePath "{{exe}}"
