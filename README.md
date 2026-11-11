# Traductor de Documentos - Inglés a Español (v1.0.0)

Aplicación nativa de escritorio para Windows desarrollada por **Victoria Sanjurjo** (`CN=CF220D35-0C1C-408C-BCB2-E85AA65BC95B`) con **Rust + Tauri v2** (backend) y **HTML5, CSS3 y JavaScript ESNext** (frontend), diseñada para traducir **Documentos PDF digitales (`.pdf`)** de manera **unidireccional estricta de Inglés (EN) a Español (ES)**, funcionando **100% offline**, con vista previa sincronizada lado a lado y exportación directa en formato **PDF (`.pdf`)**. Incluye empaquetado e instalador firmado en formato **MSIX**.

---

## 1. Características Principales (Versión 1.0.0)

- **Nombre oficial:** `Traductor de Documentos - Inglés a Español`
- **Dirección de traducción exclusiva:** Inglés (`EN`) → Español (`ES`), bloqueada por diseño según la especificación de la versión `1.0.0`.
- **Especializado en Documentos PDF Digitales (`.pdf`):**
  - Extracción página por página de PDFs con texto seleccionable (`lopdf`).
  - Detección jerárquica de encabezados/títulos y párrafos de cuerpo.
  - Reconstrucción y exportación del documento PDF traducido (`printpdf`) conservando la división de páginas y la jerarquía tipográfica (`HelveticaBold` y `Helvetica`).
- **Motor de Traducción 100% Offline y Autónomo:**
  - **Motor Lingüístico y Morfológico Nativo en Rust:** Integrado directamente en el ejecutable sin depender de conexión a internet ni de programas externos. Reconoce frases compuestas, terminología técnica/empresarial, aplica inversión sintáctica (`Adjetivo + Sustantivo` → `Sustantivo + Adjetivo` con concordancia de género y número), sufijos cognados (`-ation`, `-ity`, `-ment`, `-ally`) y contracciones del español.
  - **Soporte Neuronal Opcional ONNX (`Xenova/opus-mt-en-es`):** Permite descargar y almacenar en caché local el modelo neuronal MarianMT Opus-MT para refinamiento neuronal Transformer desde la ventana de configuración del motor.
- **Vista Previa Lado a Lado e Interactividad:**
  - Panel dividido con sincronización de desplazamiento vertical (`scroll sync`).
  - Resaltado cruzado de bloques (al pasar el cursor o enfocar un párrafo en español, se ilumina su equivalente en inglés).
  - Edición en vivo (`contentEditable`) de cualquier segmento traducido antes de exportar el PDF final.
- **Generador de Documento PDF de Ejemplo Integrado:** Botón de prueba inmediata para generar y cargar al instante un documento `.pdf` multipágina en inglés y comprobar todo el flujo de traducción y exportación.
- **Empaquetado MSIX Firmado Digitalmente:**
  - Desarrolladora: **Victoria Sanjurjo**
  - Publisher ID: `CN=CF220D35-0C1C-408C-BCB2-E85AA65BC95B`
  - Paquete instalador `.msix` generado con `MakeAppx.exe` y firmado con `SignTool.exe` (`SHA256`).

---

## 2. Arquitectura y Árbol de Directorios

```text
Traductor de Documentos/
├── assets/                                  # Recursos visuales e íconos principales (.ico, .png, .svg)
│   ├── icon.ico
│   ├── icon.png
│   └── icon.svg
├── config/                                  # Configuración declarativa de la aplicación
│   └── app_config.json
├── docs/                                    # Documentación técnica y decisiones de arquitectura
│   └── ARCHITECTURE.md
├── outputs/                                 # Salida de paquetes MSIX firmados y certificados públicos
│   ├── TraductorDeDocumentos_InglesAEspanol_1.0.0.0_x64.msix
│   └── VictoriaSanjurjo_MSIX.cer
├── packaging/                               # Manifiesto Appx y scripts de construcción y firma MSIX
│   └── msix/
│       ├── Assets/                          # Logos oficiales requeridos por el esquema MSIX
│       ├── AppxManifest.xml                 # Manifiesto Windows 10/11 (Publisher: CN=CF220D35-0C1C-408C-BCB2-E85AA65BC95B)
│       └── build_msix.ps1                   # Script automatizado de empaquetado MakeAppx + SignTool
├── src/                                     # Frontend Web (HTML5, CSS3 Editorial Moderno y JS ESNext)
│   ├── index.html
│   ├── main.js
│   └── styles.css
├── src-tauri/                               # Backend en Rust + Tauri v2
│   ├── capabilities/
│   │   └── default.json                     # Permisos IPC de Tauri v2
│   ├── icons/                               # Íconos de ventana y ejecutable Tauri
│   ├── src/
│   │   ├── lib.rs                           # Comandos IPC Tauri y generador de PDF de muestra
│   │   ├── main.rs                          # Punto de entrada binario de Windows
│   │   ├── pdf_processor.rs                 # Lector y exportador de documentos PDF digitales (.pdf)
│   │   └── translator.rs                    # Motor de traducción offline Inglés -> Español en Rust
│   ├── tests/
│   │   └── integration_tests.rs             # Suite de pruebas de integración E2E (PDF y Traductor)
│   ├── build.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── tests/                                   # Documentación de pruebas unitarias e integración
│   └── README_TESTS.md
├── .gitignore                               # Exclusiones de control de versiones Git
├── LICENSE.txt                              # Licencia MIT completa
├── README.md                                # Documentación general del proyecto
└── TraductorDeDocumentos_InglesAEspanol_1.0.0.0_x64.msix  # Copia directa del instalador MSIX firmado
```

---

## 3. Instrucciones Paso a Paso para Ejecutar, Probar y Compilar

### 3.1. Ejecutar las pruebas unitarias y de integración (Rust)

```powershell
cd src-tauri
cargo test
```

### 3.2. Ejecutar en modo desarrollo (Tauri v2)

```powershell
cd src-tauri
cargo tauri dev
```

### 3.3. Compilar el binario de producción (`release`) y generar el paquete MSIX firmado

1. Compilar el ejecutable optimizado en Rust:
   ```powershell
   cd src-tauri
   cargo build --release
   ```
2. Ejecutar el script de empaquetado y firma MSIX desde la raíz del proyecto:
   ```powershell
   powershell -ExecutionPolicy Bypass -File .\packaging\msix\build_msix.ps1
   ```

Esto generará el paquete firmado listo para instalar en:
- `outputs\TraductorDeDocumentos_InglesAEspanol_1.0.0.0_x64.msix`
- `TraductorDeDocumentos_InglesAEspanol_1.0.0.0_x64.msix` (en la raíz del proyecto)

### 3.4. Instalar el paquete MSIX en Windows

Para instalar el paquete MSIX con el certificado autofirmado de desarrollo (`CN=CF220D35-0C1C-408C-BCB2-E85AA65BC95B`):
1. Instale el certificado público `outputs\VictoriaSanjurjo_MSIX.cer` en el almacén **Entidades de certificación raíz de confianza** (Trusted Root Certification Authorities) o **Personas de confianza** (Trusted People) del equipo local.
2. Haga doble clic en `TraductorDeDocumentos_InglesAEspanol_1.0.0.0_x64.msix` (o ejecute `Add-AppxPackage -Path .\outputs\TraductorDeDocumentos_InglesAEspanol_1.0.0.0_x64.msix`).

---

## 4. Licencia

Distribuido bajo la licencia **MIT** — Copyright (c) 2026 Victoria Sanjurjo. Consulte el archivo `LICENSE.txt` para más información.
