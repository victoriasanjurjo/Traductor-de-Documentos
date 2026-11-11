# Arquitectura y Decisiones Técnicas — Traductor de Documentos - Inglés a Español (v1.0.0)

## 1. Propósito del Sistema

**Traductor de Documentos - Inglés a Español (v1.0.0)** es una aplicación nativa de escritorio para Windows construida con **Rust + Tauri v2** en el backend y **HTML5, CSS3 y JavaScript ESNext** en el frontend, desarrollada por **Victoria Sanjurjo** (`CN=CF220D35-0C1C-408C-BCB2-E85AA65BC95B`), empaquetada y firmada digitalmente como un paquete instalador **MSIX**.

La versión `1.0.0` implementa de manera estricta los requerimientos definidos:
- **Nombre oficial:** `Traductor de Documentos - Inglés a Español`
- **Dirección de traducción unidireccional estricta:** Solamente de **Inglés (EN) → Español (ES)**.
- **Formato documental especializado:** Documentos PDF digitales con texto seleccionable (`.pdf`), con extracción página por página, vista previa sincronizada lado a lado con edición interactiva de bloques y exportación directa a un nuevo documento `.pdf`.
- **Motor 100% Offline y Autónomo:** Sin requerir claves de API, cuentas de pago ni programas externos preinstalados.

---

## 2. Decisiones de Diseño y Justificación Técnica

### 2.1. Backend en Rust + Tauri v2 (`src-tauri/`)
- **Por qué Tauri v2:** Proporciona un binario ligero, rápido y seguro utilizando WebView2 del sistema operativo Windows, reduciendo drásticamente el consumo de memoria RAM frente a alternativas como Electron y permitiendo acceso directo al sistema de archivos y procesamiento intensivo en Rust.
- **Módulo `translator.rs` (Motor Híbrido Offline Inglés → Español):**
  - Para garantizar que la aplicación funcione de forma inmediata tras la instalación MSIX incluso en entornos 100% desconectados (air-gapped), se implementó en Rust puro un motor lingüístico, fraseológico y morfológico determinista de alta velocidad.
  - Incluye tabla de colocaciones y expresiones multi-palabra ordenadas por longitud máxima, diccionario léxico técnico/corporativo/general, reglas de inversión sintáctica del inglés al español (`Adjetivo + Sustantivo` → `Sustantivo + Adjetivo` con concordancia de género y número), morfología de sufijos cognados grecolatinos (`-ation` → `-ación`, `-ity` → `-idad`, `-ment` → `-miento`, `-ally` → `-almente`) y contracciones gramaticales (`al`, `del`).
  - Adicionalmente, el frontend integra soporte opcional para cargar y almacenar en caché local el modelo neuronal **MarianMT (`Xenova/opus-mt-en-es`)** en formato **ONNX / WebAssembly** para refinamiento neuronal profundo.

### 2.2. Procesamiento de Documentos PDF Digitales (`pdf_processor.rs`)
- **Por qué `lopdf` + `printpdf`:**
  - `lopdf` permite recorrer las páginas del árbol interno de objetos PDF y extraer el texto digital seleccionable página por página sin depender de librerías C/C++ externas.
  - `printpdf` permite reconstruir y exportar el documento PDF traducido en formato estándar A4, diferenciando tipográficamente los títulos (`HelveticaBold`) y los párrafos de cuerpo (`Helvetica`), aplicando ajuste de línea automático y manteniendo la correspondencia de páginas.

### 2.3. Interfaz Gráfica Editorial Moderna (`src/`)
- **Estética Visual:** Se diseñó una interfaz **Editorial Clara (Warm Paper `#F8F6F1`, Deep Navy `#1E3A8A` y Teal `#0F766E`)** inspirada en entornos de lectura y edición documental profesional.
- **Estándares Web Modernos (Modern Web Guidance):**
  - Uso de `<dialog closedby="any">` nativo con `showModal()` y fallback de clic en el backdrop para navegadores que aún no implementen `closedBy`.
  - Uso del elemento semántico `<progress>` con estilizado personalizado (`appearance: none`, variables CSS, animación indeterminada mediante `container-type: inline-size` y `100cqi`, respetando `prefers-reduced-motion`).
  - Selección de texto estilizada con `::selection`, `accent-color` corporativo y barras de desplazamiento accesibles (`scrollbar-color` sin reducir el ancho con `scrollbar-width: thin`).

### 2.4. Empaquetado y Firma MSIX (`packaging/msix/`)
- Se incluye un manifiesto `AppxManifest.xml` conforme al esquema Windows 10/11 Desktop Bridge (`Windows.FullTrustApplication`) con `Identity Name="VictoriaSanjurjo.TraductorDeDocumentosInglesAEspanol"` y `Publisher="CN=CF220D35-0C1C-408C-BCB2-E85AA65BC95B"`.
- El script automatizado `packaging/msix/build_msix.ps1` invoca `makeappx.exe` del Windows SDK para empaquetar el layout, utiliza o genera el certificado autofirmado para la identidad de desarrolladora **Victoria Sanjurjo** (`CN=CF220D35-0C1C-408C-BCB2-E85AA65BC95B`, exportando `.pfx` y `.cer`) y firma el paquete `.msix` con `signtool.exe` usando `SHA256`.
