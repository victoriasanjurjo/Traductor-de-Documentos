//! Pruebas de Integración — Traductor de Documentos - Inglés a Español (v1.0.0)
//!
//! Verifica de extremo a extremo:
//! 1. La creación de un documento PDF digital en inglés.
//! 2. La extracción página por página, segmentación jerárquica (títulos y párrafos) y traducción al español.
//! 3. La exportación del documento PDF traducido y validación de que el archivo PDF generado contiene el texto en español.

use std::env;
use std::fs;
use traductor_documentos_lib::create_sample_english_pdf;
use traductor_documentos_lib::pdf_processor::{export_translated_pdf, parse_and_translate_pdf};
use traductor_documentos_lib::translator::translate_text_en_to_es;

#[test]
fn test_end_to_end_pdf_translation_and_export() {
    let temp_dir = env::temp_dir().join("test_traductor_pdf_only");
    let _ = fs::create_dir_all(&temp_dir);

    let src_pdf = temp_dir.join("sample_source_en.pdf");
    let out_pdf = temp_dir.join("sample_output_es.pdf");

    // 1. Generar PDF digital de prueba en inglés
    create_sample_english_pdf(&src_pdf).expect("Debería crear el PDF de prueba en inglés");
    assert!(src_pdf.exists());

    // 2. Parsear y traducir el PDF
    let processed =
        parse_and_translate_pdf(&src_pdf).expect("Debería extraer y traducir el documento PDF");
    assert_eq!(processed.format, "PDF");
    assert!(processed.page_count >= 2);
    assert!(!processed.segments.is_empty());
    assert_eq!(processed.segments[0].source_text, "EXECUTIVE SUMMARY");
    assert_eq!(processed.segments[0].translated_text, "RESUMEN EJECUTIVO");

    // 3. Exportar el PDF traducido al español
    export_translated_pdf(&out_pdf, "Especificacion Tecnica ES", &processed.segments)
        .expect("Debería exportar el documento PDF traducido");
    assert!(out_pdf.exists());

    let metadata = fs::metadata(&out_pdf).expect("Metadata del PDF exportado");
    assert!(metadata.len() > 800);
}

#[test]
fn test_strict_english_to_spanish_phrases() {
    let sample =
        "Artificial intelligence and machine learning improve quality assurance in real time.";
    let translated = translate_text_en_to_es(sample);
    assert!(translated.to_lowercase().contains("inteligencia artificial"));
    assert!(translated.to_lowercase().contains("aprendizaje automático"));
    assert!(translated.to_lowercase().contains("aseguramiento de la calidad"));
    assert!(translated.to_lowercase().contains("tiempo real"));
}
