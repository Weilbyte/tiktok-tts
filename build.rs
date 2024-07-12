fn build_css(input: String, output: String) {
    let result = std::process::Command::new("npx")
        .args(["--yes", "tailwindcss", "-i", &input, "-o", &output, "--minify"])
        .output()
        .expect("Unable to generate CSS");


    if !result.status.success() {
        let error = String::from_utf8_lossy(&result.stderr);
        println!("cargo:warning=Unable to generate CSS");
        println!("cargo:warning=Output: {error}");
    }
}

fn minify_js(input: String, output: String) {
    let result = std::process::Command::new("npx")
        .args(["--yes", "uglify-js", &input, "-o", &output])
        .output()
        .expect("Unable to minify JS");


    if !result.status.success() {
        let error = String::from_utf8_lossy(&result.stderr);
        println!("cargo:warning=Unable to minify JS");
        println!("cargo:warning=Output: {error}");
    }
}

fn main() {
    if std::env::var("BUILD_SCRIPT").map(|v| v == "1").unwrap_or(true) {
        let manifest_dir: String = std::env::var("CARGO_MANIFEST_DIR").unwrap();

        build_css(
            format!("{manifest_dir}/static/input.css").to_string(),
            format!("{manifest_dir}/static/style.css").to_string()
        );

        minify_js(
            format!("{manifest_dir}/static/script.js").to_string(),
            format!("{manifest_dir}/static/script.min.js").to_string()
        );
    }
}