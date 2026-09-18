fn main() {
    tauri_plugin::Builder::new(&["update", "remove"])
        .ios_path("ios")
        .build();
}
