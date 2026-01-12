use fasttext::FastText;

/// Classify the language of the given text using fastText.
/// Returns the ISO 639-1 language code (e.g., "en", "fr", "de").
/// Falls back to "en" on any error.
pub async fn classify_text(text: &str) -> String {
    let text_owned = text.to_string();
    tokio::task::spawn_blocking(move || {
        let mut ft = FastText::new();
        // Use the pre-trained lid.176.bin model for language identification.
        // Users should download it from https://dl.fbaipublicfiles.com/fasttext/supervised-models/lid.176.bin
        // and place it at a known location, e.g., ~/.config/eezze/lid.176.bin
        let model_path = dirs_next::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".config")
            .join("eezze")
            .join("lid.176.bin");

        if !model_path.exists() {
            eprintln!(
                "fastText language model not found at {}. Falling back to 'en'.",
                model_path.display()
            );
            return "en".to_string();
        }

        if let Err(e) = ft.load_model(&model_path.to_string_lossy()) {
            eprintln!(
                "Failed to load fastText model from {}: {}. Falling back to 'en'.",
                model_path.display(),
                e
            );
            return "en".to_string();
        }

        // Predict language; fastText returns labels like "__label__en"
        match ft.predict(&text_owned, 1, 0.0) {
            Ok(predictions) => {
                if let Some(first) = predictions.first() {
                    // Strip the "__label__" prefix
                    first.label.replace("__label__", "")
                } else {
                    eprintln!("fastText returned no predictions. Falling back to 'en'.");
                    "en".to_string()
                }
            }
            Err(e) => {
                eprintln!(
                    "fastText prediction failed: {}. Falling back to 'en'.",
                    e
                );
                "en".to_string()
            }
        }
    })
    .await
    .unwrap_or_else(|_| {
        eprintln!("Task to classify language failed to spawn. Falling back to 'en'.");
        "en".to_string()
    })
}