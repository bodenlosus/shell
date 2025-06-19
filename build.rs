use std::env;
use std::fs;
use std::process::Command;
fn main() {
    println!("cargo:rerun-if-changed=data/blp/**/*.blp");
    let out_dir = env::var("OUT_DIR").unwrap();
    let files: Vec<String> = fs::read_dir("data/blp")
        .unwrap()
        .into_iter()
        .filter_map(|f| {
            let Ok(f) = f else {
                return None;
            };

 
            let path = f.path();

            let Some(extension) = path.extension().and_then(|e| e.to_str()) else {
                return None;
            };

            if extension == "blp" {
                return path.to_str().map(|s| s.to_string());
            }

            None
        })
        .collect();
    Command::new("blueprint-compiler")
        .args(&["batch-compile", format!("{out_dir}/data/ui").as_str(), "data/blp"])
        .args(files).status().unwrap();
    glib_build_tools::compile_resources(&["data/", &format!("{out_dir}/data/")], "data/shell.gresource.xml", "shell.gresource");
}
