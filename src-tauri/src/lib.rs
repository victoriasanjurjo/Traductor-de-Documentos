//! # Biblioteca Principal Tauri v2 — Traducir textos inglés a español (v1.0.0)
//!
//! Desarrolladora: Victoria Sanjurjo (`CN=CF220D35-0C1C-408C-BCB2-E85AA65BC95B`)
//!
//! Expone los comandos IPC invocados desde el frontend en HTML/CSS/JS:
//! - `translate_text`: Traduce un texto en inglés a español validando el límite estricto de 20.000 caracteres.
//! - `get_sample_english_text`: Devuelve un texto de ejemplo en inglés para prueba inmediata.

pub mod translator;

use serde::{Deserialize, Serialize};

/// Límite máximo de caracteres permitido en la versión 1.0.0 (20.000 caracteres).
pub const MAX_CHARS_LIMIT: usize = 20_000;

/// Respuesta estructurada de la traducción de texto con métricas detalladas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResponse {
    pub source_text: String,
    pub translated_text: String,
    pub source_char_count: usize,
    pub source_word_count: usize,
    pub target_char_count: usize,
    pub target_word_count: usize,
    pub paragraph_count: usize,
    pub max_chars_limit: usize,
}

/// Función pública de traducción con validación de límite (20.000 caracteres).
pub fn perform_text_translation(text: &str) -> Result<TranslationResponse, String> {
    let char_count = text.chars().count();
    if char_count > MAX_CHARS_LIMIT {
        return Err(format!(
            "El texto ingresado tiene {} caracteres y supera el límite máximo permitido de {} caracteres.",
            char_count, MAX_CHARS_LIMIT
        ));
    }

    let translated = translator::translate_text_en_to_es(text);
    let source_word_count = text.split_whitespace().count();
    let target_char_count = translated.chars().count();
    let target_word_count = translated.split_whitespace().count();
    let paragraph_count = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .count();

    Ok(TranslationResponse {
        source_text: text.to_string(),
        translated_text: translated,
        source_char_count: char_count,
        source_word_count,
        target_char_count,
        target_word_count,
        paragraph_count,
        max_chars_limit: MAX_CHARS_LIMIT,
    })
}

/// Comando Tauri que traduce texto de Inglés a Español respetando el máximo de 20.000 caracteres.
#[tauri::command]
fn translate_text(text: String) -> Result<TranslationResponse, String> {
    perform_text_translation(&text)
}

/// Comando Tauri que entrega un texto de ejemplo en inglés para probar la herramienta con un clic.
#[tauri::command]
fn get_sample_english_text() -> String {
    concat!(
        "EXECUTIVE SUMMARY\n\n",
        "This technical specification describes the software architecture and security policy for the new digital application. ",
        "The main goal of the project is to provide high quality results in real time without internet connection.\n\n",
        "SYSTEM REQUIREMENTS AND PERFORMANCE\n\n",
        "Artificial intelligence and machine learning models improve the translation performance. ",
        "Furthermore, the user interface is simple, fast, and reliable. ",
        "All information is processed locally in the computer in order to protect privacy and intellectual property.\n\n",
        "IN CONCLUSION\n\n",
        "Please note that the technical team completed the quality assurance tests successfully according to the terms and conditions."
    )
    .to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            translate_text,
            get_sample_english_text
        ])
        .run(tauri::generate_context!())
        .expect("Error al ejecutar la aplicación Tauri Traducir textos inglés a español");
}
