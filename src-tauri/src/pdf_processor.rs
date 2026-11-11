//! # Procesador de Documentos PDF Digitales (.pdf)
//!
//! Desarrolladora: Victoria Sanjurjo (`CN=CF220D35-0C1C-408C-BCB2-E85AA65BC95B`)
//!
//! Este módulo implementa:
//! 1. Extracción de texto digital seleccionable página por página mediante `lopdf`.
//! 2. Segmentación inteligente en títulos (`heading`) y párrafos (`paragraph`) manteniendo la numeración de páginas.
//! 3. Generación y exportación de documentos PDF traducidos de alta fidelidad con soporte de caracteres en español
//!    (acentos, ñ, ¿, ¡) mediante `printpdf` respetando la paginación y jerarquía tipográfica.

use crate::translator::translate_text_en_to_es;
use lopdf::Document as LoPdfDocument;
use printpdf::{BuiltinFont, Mm, PdfDocument};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

/// Representa un bloque o párrafo extraído de un documento PDF
/// para mostrar en la vista previa lado a lado y permitir su edición/traducción.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSegment {
    /// Identificador único del segmento (índice secuencial)
    pub id: usize,
    /// Tipo de bloque para estilizar la vista previa y el PDF ("heading" o "paragraph")
    pub kind: String,
    /// Número de página del documento PDF
    pub page: usize,
    /// Texto original en Inglés
    pub source_text: String,
    /// Texto traducido al Español
    pub translated_text: String,
}

/// Resultado completo del procesamiento de un documento PDF.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedDocument {
    pub file_name: String,
    pub file_path: String,
    pub format: String,
    pub page_count: usize,
    pub word_count: usize,
    pub char_count: usize,
    pub segments: Vec<DocumentSegment>,
}

/// Analiza un archivo PDF digital (con texto seleccionable), extrae sus bloques por página
/// y ejecuta la traducción Inglés -> Español.
pub fn parse_and_translate_pdf(path: &Path) -> Result<ProcessedDocument, String> {
    let doc = LoPdfDocument::load(path)
        .map_err(|e| format!("No se pudo abrir o decodificar el archivo PDF: {}", e))?;

    let pages = doc.get_pages();
    let page_count = pages.len().max(1);

    let mut segments = Vec::new();
    let mut total_words = 0usize;
    let mut total_chars = 0usize;
    let mut seg_id = 0usize;

    for (&page_num, &_page_id) in pages.iter() {
        let raw_page_text = doc.extract_text(&[page_num]).unwrap_or_default();

        let page_paragraphs = split_pdf_page_into_paragraphs(&raw_page_text);
        for (idx_in_page, para) in page_paragraphs.into_iter().enumerate() {
            let trimmed = para.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Omitir líneas de cabecera generadas por plantillas internas
            if trimmed.starts_with("Traductor de Documentos (EN -> ES)") {
                continue;
            }

            let wc = trimmed.split_whitespace().count();
            let cc = trimmed.chars().count();
            total_words += wc;
            total_chars += cc;

            // Heurística para distinguir encabezados/títulos de párrafos de cuerpo en PDF
            let is_heading = (idx_in_page == 0 && wc <= 14 && !trimmed.ends_with('.'))
                || (wc <= 10
                    && trimmed
                        .chars()
                        .filter(|c| c.is_alphabetic())
                        .all(|c| c.is_uppercase()));

            let kind = if is_heading {
                "heading".to_string()
            } else {
                "paragraph".to_string()
            };

            let translated = translate_text_en_to_es(trimmed);
            segments.push(DocumentSegment {
                id: seg_id,
                kind,
                page: page_num as usize,
                source_text: trimmed.to_string(),
                translated_text: translated,
            });
            seg_id += 1;
        }
    }

    if segments.is_empty() {
        return Err(
            "El archivo PDF no contiene texto digital seleccionable (parece ser una imagen escaneada). \
             La versión 1.0.0 soporta PDFs digitales con texto seleccionable."
                .to_string(),
        );
    }

    Ok(ProcessedDocument {
        file_name: path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("documento.pdf")
            .to_string(),
        file_path: path.to_string_lossy().to_string(),
        format: "PDF".to_string(),
        page_count,
        word_count: total_words,
        char_count: total_chars,
        segments,
    })
}

/// Agrupa las líneas extraídas de una página PDF en párrafos lógicos.
fn split_pdf_page_into_paragraphs(raw_text: &str) -> Vec<String> {
    let mut paragraphs = Vec::new();
    let mut current = String::new();

    for line in raw_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !current.trim().is_empty() {
                paragraphs.push(current.trim().to_string());
                current.clear();
            }
            continue;
        }

        if !current.is_empty() {
            let ends_sentence = current.ends_with('.')
                || current.ends_with(':')
                || current.ends_with('!')
                || current.ends_with('?');
            let current_words = current.split_whitespace().count();
            if (ends_sentence && current_words > 25) || (!ends_sentence && current_words <= 10) {
                paragraphs.push(current.trim().to_string());
                current.clear();
            } else {
                current.push(' ');
            }
        }
        current.push_str(trimmed);
    }

    if !current.trim().is_empty() {
        paragraphs.push(current.trim().to_string());
    }

    paragraphs
}

/// Exporta los segmentos traducidos a un documento PDF maquetado página por página,
/// respetando la estructura de páginas original, diferenciando encabezados (`HelveticaBold`)
/// y párrafos (`Helvetica`), con ajuste de línea automático y pies/encabezados de página numerados.
pub fn export_translated_pdf(
    output_path: &Path,
    doc_title: &str,
    segments: &[DocumentSegment],
) -> Result<(), String> {
    let (doc, page1, layer1) = PdfDocument::new(
        doc_title,
        Mm(210.0), // A4 ancho
        Mm(297.0), // A4 alto
        "Capa de Texto Traducido",
    );

    let font_regular = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| format!("Error cargando fuente PDF regular: {}", e))?;
    let font_bold = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| format!("Error cargando fuente PDF negrita: {}", e))?;

    let mut current_page_idx = page1;
    let mut current_layer_idx = layer1;
    let mut current_doc_page_num = 1usize;
    let mut cursor_y_mm = 272.0_f32;
    let left_margin_mm = 22.0_f32;
    let bottom_margin_mm = 24.0_f32;

    // Dibujar encabezado sutil en la primera página
    {
        let layer = doc.get_page(current_page_idx).get_layer(current_layer_idx);
        layer.use_text(
            format!(
                "Traductor de Documentos (EN -> ES) | Página {}",
                current_doc_page_num
            ),
            8.5,
            Mm(left_margin_mm),
            Mm(284.0),
            &font_regular,
        );
    }

    for seg in segments {
        if (seg.page > current_doc_page_num && cursor_y_mm < 260.0)
            || cursor_y_mm < bottom_margin_mm + 15.0
        {
            current_doc_page_num = seg.page.max(current_doc_page_num + 1);
            let (new_page, new_layer) = doc.add_page(
                Mm(210.0),
                Mm(297.0),
                format!("Página {}", current_doc_page_num),
            );
            current_page_idx = new_page;
            current_layer_idx = new_layer;
            cursor_y_mm = 272.0;

            let layer = doc.get_page(current_page_idx).get_layer(current_layer_idx);
            layer.use_text(
                format!(
                    "Traductor de Documentos (EN -> ES) | Página {}",
                    current_doc_page_num
                ),
                8.5,
                Mm(left_margin_mm),
                Mm(284.0),
                &font_regular,
            );
        }

        let is_heading = seg.kind == "heading";
        let font_size = if is_heading { 14.0_f32 } else { 10.5_f32 };
        let line_height_mm = if is_heading { 7.0_f32 } else { 5.2_f32 };
        let max_chars_per_line = if is_heading { 62usize } else { 84usize };
        let font_ref = if is_heading { &font_bold } else { &font_regular };

        if is_heading && cursor_y_mm < 268.0 {
            cursor_y_mm -= 3.5;
        }

        let wrapped_lines = wrap_text_lines(&seg.translated_text, max_chars_per_line);
        for line in wrapped_lines {
            if cursor_y_mm < bottom_margin_mm {
                current_doc_page_num += 1;
                let (new_page, new_layer) = doc.add_page(
                    Mm(210.0),
                    Mm(297.0),
                    format!("Página {}", current_doc_page_num),
                );
                current_page_idx = new_page;
                current_layer_idx = new_layer;
                cursor_y_mm = 272.0;

                let layer = doc.get_page(current_page_idx).get_layer(current_layer_idx);
                layer.use_text(
                    format!(
                        "Traductor de Documentos (EN -> ES) | Página {}",
                        current_doc_page_num
                    ),
                    8.5,
                    Mm(left_margin_mm),
                    Mm(284.0),
                    &font_regular,
                );
            }

            let layer = doc.get_page(current_page_idx).get_layer(current_layer_idx);
            layer.use_text(
                line,
                font_size,
                Mm(left_margin_mm),
                Mm(cursor_y_mm),
                font_ref,
            );
            cursor_y_mm -= line_height_mm;
        }

        // Espacio después de cada párrafo
        cursor_y_mm -= if is_heading { 4.5 } else { 3.2 };
    }

    let out_file = File::create(output_path)
        .map_err(|e| format!("No se pudo crear el archivo PDF de salida: {}", e))?;
    doc.save(&mut BufWriter::new(out_file))
        .map_err(|e| format!("Error guardando el documento PDF traducido: {}", e))?;

    Ok(())
}

fn wrap_text_lines(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for raw_line in text.split('\n') {
        let mut current = String::new();
        for word in raw_line.split_whitespace() {
            if current.is_empty() {
                current.push_str(word);
            } else if current.chars().count() + 1 + word.chars().count() <= max_chars {
                current.push(' ');
                current.push_str(word);
            } else {
                lines.push(current);
                current = word.to_string();
            }
        }
        if !current.is_empty() {
            lines.push(current);
        }
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

