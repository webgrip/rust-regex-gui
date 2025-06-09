#[cfg(feature = "tauri")]
pub fn build() -> tauri::Builder<tauri::Wry> {
    tauri::Builder::default()
}

#[cfg(all(test, feature = "tauri"))]
mod tests {
    use super::*;

    #[test]
    fn build_returns_builder() {
        let _ = build();
    }
}
