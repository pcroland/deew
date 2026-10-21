//! The frontend (`../dist`) is embedded into the binary, so it has to be built first.

fn main() {
    println!("cargo:rerun-if-changed=../dist");
    let index = std::path::Path::new("../dist/index.html");
    if !index.is_file() {
        println!(
            "cargo:warning=frontend not built: ../dist/index.html is missing. Run `npm run build` in gui/ first."
        );
        panic!(
            "frontend not built: gui/dist/index.html is missing. Run `npm run build` in gui/ and build deew-web again."
        );
    }
}
