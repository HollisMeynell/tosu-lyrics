

bin_name := if os() == "windows" { "osu-lyric.exe" } else { "osu-lyric" }
ext      := if os() == "windows" { ".exe" } else { "" }

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
    cp ./tosu-proxy/target/release/{{bin_name}} ./dist/osu-lyric{{ext}}

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

# 组装 embed/ 目录：前端产物 + 字体 → cargo build 时由 rust-embed 嵌入二进制。
assemble-embed:
    #!/usr/bin/env bash
    set -euo pipefail
    rm -rf ./embed
    mkdir -p ./embed
    pnpm exec vite build --outDir tosu-proxy/target/dist-package --emptyOutDir
    cp ./tosu-proxy/target/dist-package/index.html ./embed/
    cp -r ./tosu-proxy/target/dist-package/assets ./embed/
    [ -f ./tosu-proxy/target/dist-package/osu.svg ] && cp ./tosu-proxy/target/dist-package/osu.svg ./embed/ || true
    cp ./static/LRC.otf ./static/tLRC.otf ./embed/
    [ -f ./tosu-proxy/lib/ffprobe.exe ] && cp ./tosu-proxy/lib/ffprobe.exe ./embed/ffprobe || true
    echo "embed/ 文件数: $(find ./embed -type f | wc -l)"

# 单文件发行：前端 → embed/ → cargo release build → 输出到目标目录
@package final_dir="./release": assemble-embed build-backend
    mkdir -p {{final_dir}}
    cp ./tosu-proxy/target/release/{{bin_name}} {{final_dir}}/tosu-lyrics{{ext}}
    echo "输出: $(realpath {{final_dir}}/tosu-lyrics{{ext}}) ($(du -h {{final_dir}}/tosu-lyrics{{ext}} | cut -f1))"
