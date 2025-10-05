#!/usr/bin/env just --justfile

default:
    @just --list

@build-dist:
    pnpm i
    pnpm build

[working-directory: './tosu-proxy']
@build-backend:
    cargo fmt
    cargo clippy --fix --allow-dirty
    cargo b -r --bin osu-lyric --features=new

@copy-backend:
    cp ./tosu-proxy/target/release/osu-lyric ./dist/tosu-proxy
    chmod +x ./dist/tosu-proxy

@build:
    just build-dist
    just build-backend
    just copy-backend

