# Guía de Ejecución de Pruebas Unitarias y de Integración

Las pruebas automatizadas del motor de traducción offline, del procesador OpenXML (`.docx`) y del procesador PDF digital (`.pdf`) se encuentran en:

- `src-tauri/src/translator.rs` (Pruebas unitarias del motor lingüístico y morfológico Inglés → Español).
- `src-tauri/tests/integration_tests.rs` (Pruebas de integración de extremo a extremo para generación, lectura, segmentación y exportación de `.docx` y `.pdf`).

## Comando para ejecutar toda la suite de pruebas

```powershell
cd src-tauri
cargo test
```

