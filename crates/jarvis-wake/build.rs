//! Путь к нативной библиотеке Vosk можно задать через `VOSK_LIB_DIR`.
//!
//! По умолчанию полагаемся на системные пути линковщика. На Windows это
//! каталог с `libvosk.lib`, на Linux — с `libvosk.so`.

fn main() {
    println!("cargo:rerun-if-env-changed=VOSK_LIB_DIR");

    if let Ok(dir) = std::env::var("VOSK_LIB_DIR") {
        if !dir.is_empty() {
            println!("cargo:rustc-link-search=native={dir}");
        }
    }
}
