use std::path::{Path, PathBuf};
use std::{env, fs};

fn collect(base: &Path, dir: &Path, out: &mut Vec<(String, PathBuf)>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for entry in rd.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(base, &path, out);
        } else if let Ok(rel) = path.strip_prefix(base) {
            out.push((rel.to_string_lossy().replace('\\', "/"), path.clone()));
        }
    }
}

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let base = manifest.parent().unwrap().join("embed");
    println!("cargo:rerun-if-changed={}", base.display());
    let mut items: Vec<(String, PathBuf)> = Vec::new();
    if !base.is_dir() {
        println!("cargo:warning=embed/ 不存在：单文件发行将不含前端/字体资源。请先运行 scripts/package-release.ps1");
    }
    if base.is_dir() {
        collect(&base, &base, &mut items);
    }
    items.sort();
    let mut src = String::from(
        "/// 由 build.rs 生成：内嵌程序资源清单（来源：仓库根目录 embed/，由打包流程填充）\npub static EMBEDDED: &[(&str, &[u8])] = &[\n",
    );
    for (rel, abs) in &items {
        src.push_str(&format!(
            "    ({:?}, include_bytes!({:?})),\n",
            rel,
            abs.to_string_lossy()
        ));
    }
    src.push_str("];\n");
    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("embedded_assets.rs");
    fs::write(out, src).unwrap();
}