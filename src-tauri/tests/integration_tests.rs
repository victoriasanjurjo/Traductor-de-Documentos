//! Pruebas de Integración — Traducir textos inglés a español (v1.0.0)
//!
//! Verifica:
//! 1. La traducción correcta de textos en inglés a español.
//! 2. El cumplimiento estricto del límite máximo de 20.000 caracteres.
//! 3. La preservación de párrafos, saltos de línea y mayúsculas.

use traducir_textos_lib::{perform_text_translation, MAX_CHARS_LIMIT};

#[test]
fn test_text_translation_en_to_es() {
    let input = "EXECUTIVE SUMMARY\n\nArtificial intelligence and machine learning improve quality assurance in real time.";
    let res = perform_text_translation(input).expect("Debería traducir el texto correctamente");

    assert_eq!(res.max_chars_limit, 20_000);
    assert_eq!(res.paragraph_count, 2);
    assert!(res.translated_text.contains("RESUMEN EJECUTIVO"));
    assert!(res.translated_text.to_lowercase().contains("inteligencia artificial"));
    assert!(res.translated_text.to_lowercase().contains("aprendizaje automático"));
    assert!(res.translated_text.to_lowercase().contains("tiempo real"));
}

#[test]
fn test_exact_20000_chars_allowed() {
    let text_20k = "a".repeat(MAX_CHARS_LIMIT);
    let res = perform_text_translation(&text_20k);
    assert!(res.is_ok(), "Un texto de exactamente 20.000 caracteres debe ser aceptado");
}

#[test]
fn test_over_20000_chars_rejected() {
    let text_over = "a".repeat(MAX_CHARS_LIMIT + 1);
    let res = perform_text_translation(&text_over);
    assert!(res.is_err(), "Un texto de 20.001 caracteres debe ser rechazado por superar el límite de 20.000 caracteres");
}
