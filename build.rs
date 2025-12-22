use std::process::Command;

fn main() {
    run_tailwind();
}

fn run_tailwind() {
    println!(
        "{}",
        std::env::var("DATABASE_URL").unwrap_or("DATABASE_URL undefined".to_string())
    );
    let result = Command::new("npx")
        .args(["tailwindcss", "-i", "styles.css", "-o", "static/styles.css"])
        .output();
    match result {
        Ok(output) => {
            if !output.status.success() {
                panic!("{}", String::from_utf8(output.stderr).unwrap());
            }
        }
        Err(err) => {
            println!("[WARN] tailwind failed: {}", err.to_string());
        }
    }
}
