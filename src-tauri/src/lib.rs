//! # Biblioteca Principal Tauri v2 — Traductor de Documentos - Inglés a Español (v1.0.0)
//!
//! Desarrolladora: Victoria Sanjurjo (`CN=CF220D35-0C1C-408C-BCB2-E85AA65BC95B`)
//!
//! Expone los comandos IPC invocados desde el frontend en HTML/CSS/JS para traducir documentos PDF:
//! - `select_pdf_dialog`: Abre el selector nativo de archivos filtrando documentos `.pdf`.
//! - `load_and_translate_pdf`: Procesa y traduce un documento `.pdf` desde su ruta en disco.
//! - `load_and_translate_pdf_bytes`: Procesa un archivo `.pdf` arrastrado (Drag & Drop) en memoria.
//! - `translate_single_segment`: Re-traduce un segmento individual de Inglés a Español.
//! - `export_translated_pdf_dialog`: Guarda el documento traducido en un nuevo archivo `.pdf` conservando su estructura.
//! - `generate_sample_pdf`: Crea un documento `.pdf` de ejemplo en inglés (2 páginas) y lo carga para prueba inmediata.

pub mod pdf_processor;
pub mod translator;

use pdf_processor::{
    export_translated_pdf, parse_and_translate_pdf, DocumentSegment, ProcessedDocument,
};
use std::fs;
use std::path::{Path, PathBuf};

/// Abre el diálogo nativo de Windows para elegir un documento PDF (`.pdf`).
#[tauri::command]
async fn select_pdf_dialog() -> Result<Option<String>, String> {
    let picked = rfd::FileDialog::new()
        .set_title("Seleccionar documento PDF en Inglés (*.pdf)")
        .add_filter("Documento PDF Digital (*.pdf)", &["pdf"])
        .pick_file();

    Ok(picked.map(|p| p.to_string_lossy().to_string()))
}

/// Carga un documento `.pdf` desde `file_path`, extrae su estructura por páginas y traduce sus bloques de Inglés a Español.
#[tauri::command]
async fn load_and_translate_pdf(file_path: String) -> Result<ProcessedDocument, String> {
    let path = Path::new(&file_path);
    if !path.exists() {
        return Err(format!(
            "El archivo PDF no existe en la ruta especificada: {}",
            file_path
        ));
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if ext != "pdf" {
        return Err(
            "Formato no admitido en la versión 1.0.0. Seleccione únicamente documentos PDF (.pdf)."
                .to_string(),
        );
    }

    parse_and_translate_pdf(path)
}

/// Permite arrastrar y soltar un archivo `.pdf` directamente en la interfaz web enviando sus bytes al backend.
#[tauri::command]
async fn load_and_translate_pdf_bytes(
    file_name: String,
    bytes: Vec<u8>,
) -> Result<ProcessedDocument, String> {
    let temp_dir = std::env::temp_dir().join("traductor_documentos_pdf_en_es");
    fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("No se pudo crear directorio temporal: {}", e))?;

    let temp_path = temp_dir.join(&file_name);
    fs::write(&temp_path, &bytes)
        .map_err(|e| format!("No se pudo almacenar el archivo PDF temporal: {}", e))?;

    load_and_translate_pdf(temp_path.to_string_lossy().to_string()).await
}

/// Traduce un segmento puntual en inglés a español utilizando el motor offline de Rust.
#[tauri::command]
fn translate_single_segment(text: String) -> String {
    translator::translate_text_en_to_es(&text)
}

/// Abre un diálogo nativo de guardado y exporta el documento PDF traducido al Español.
#[tauri::command]
async fn export_translated_pdf_dialog(
    source_path: String,
    segments: Vec<DocumentSegment>,
) -> Result<Option<String>, String> {
    let src = Path::new(&source_path);
    let stem = src
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("documento");

    let default_out_name = format!("{}_ES.pdf", stem);

    let save_dialog = rfd::FileDialog::new()
        .set_title("Exportar documento PDF traducido al Español")
        .set_file_name(&default_out_name)
        .add_filter("Documento PDF (*.pdf)", &["pdf"]);

    let Some(target_path) = save_dialog.save_file() else {
        return Ok(None);
    };

    let title = target_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Documento PDF Traducido");

    export_translated_pdf(&target_path, title, &segments)?;

    Ok(Some(target_path.to_string_lossy().to_string()))
}

/// Genera un documento `.pdf` de demostración en Inglés (con múltiples secciones y páginas)
/// y lo traduce inmediatamente para probar la vista previa lado a lado y la exportación PDF.
#[tauri::command]
async fn generate_sample_pdf() -> Result<ProcessedDocument, String> {
    let temp_dir = std::env::temp_dir().join("traductor_documentos_pdf_en_es");
    fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Error creando carpeta temporal: {}", e))?;

    let sample_pdf_path = temp_dir.join("Technical_Specification_Sample_EN.pdf");
    create_sample_english_pdf(&sample_pdf_path)?;
    parse_and_translate_pdf(&sample_pdf_path)
}

/// Construye un archivo `.pdf` digital seleccionable en inglés para pruebas inmediatas.
pub fn create_sample_english_pdf(path: &PathBuf) -> Result<(), String> {
    let sample_segments = vec![
        DocumentSegment {
            id: 0,
            kind: "heading".to_string(),
            page: 1,
            source_text: "EXECUTIVE SUMMARY".to_string(),
            translated_text: "EXECUTIVE SUMMARY".to_string(),
        },
        DocumentSegment {
            id: 1,
            kind: "paragraph".to_string(),
            page: 1,
            source_text: "This digital document provides an overview of the document translator from English to Spanish. The system works locally and preserves the structure of the source document.".to_string(),
            translated_text: "This digital document provides an overview of the document translator from English to Spanish. The system works locally and preserves the structure of the source document.".to_string(),
        },
        DocumentSegment {
            id: 2,
            kind: "heading".to_string(),
            page: 1,
            source_text: "SYSTEM ARCHITECTURE AND PERFORMANCE".to_string(),
            translated_text: "SYSTEM ARCHITECTURE AND PERFORMANCE".to_string(),
        },
        DocumentSegment {
            id: 3,
            kind: "paragraph".to_string(),
            page: 1,
            source_text: "Artificial intelligence and machine learning models improve the quality assurance in real time. Furthermore, the user interface displays the original format side by side.".to_string(),
            translated_text: "Artificial intelligence and machine learning models improve the quality assurance in real time. Furthermore, the user interface displays the original format side by side.".to_string(),
        },
        DocumentSegment {
            id: 4,
            kind: "heading".to_string(),
            page: 2,
            source_text: "SECURITY POLICY AND PRIVACY".to_string(),
            translated_text: "SECURITY POLICY AND PRIVACY".to_string(),
        },
        DocumentSegment {
            id: 5,
            kind: "paragraph".to_string(),
            page: 2,
            source_text: "All files are processed in the local computer without external servers. Therefore, confidential information and intellectual property remain completely safe according to the terms and conditions.".to_string(),
            translated_text: "All files are processed in the local computer without external servers. Therefore, confidential information and intellectual property remain completely safe according to the terms and conditions.".to_string(),
        },
    ];

    export_translated_pdf(path, "Technical Specification Sample EN", &sample_segments)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            select_pdf_dialog,
            load_and_translate_pdf,
            load_and_translate_pdf_bytes,
            translate_single_segment,
            export_translated_pdf_dialog,
            generate_sample_pdf
        ])
        .run(tauri::generate_context!())
        .expect("Error al ejecutar la aplicación Tauri Traductor de Documentos - Inglés a Español");
}
