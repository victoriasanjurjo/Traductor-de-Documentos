# Arquitectura y Decisiones Técnicas — Traducir textos inglés a español (v1.0.0)

## 1. Propósito del Sistema

**Traducir textos inglés a español (v1.0.0)** es una aplicación nativa de escritorio para Windows construida con **Rust + Tauri v2** en el backend y **HTML5, CSS3 y JavaScript ESNext** en el frontend, desarrollada por **Victoria Sanjurjo** (`CN=CF220D35-0C1C-408C-BCB2-E85AA65BC95B`), empaquetada y firmada digitalmente como un paquete instalador **MSIX**.

La versión `1.0.0` implementa de manera estricta los requerimientos definidos:
- **Nombre del programa:** `Traducir textos inglés a español`
- **Dirección de traducción unidireccional estricta:** Solamente de **Inglés (EN) → Español (ES)**.
- **Límite máximo de caracteres:** **20.000 caracteres** validados tanto en el frontend (`maxlength="20000"`, contador en vivo y barra `<progress max="20000">`) como en el backend de Rust (`MAX_CHARS_LIMIT = 20_000`).
- **Motor 100% Offline y Autónomo:** Sin requerir claves de API, cuentas de pago ni programas externos preinstalados.

---

## 2. Decisiones de Diseño y Justificación Técnica

### 2.1. Backend Ultraligero en Rust + Tauri v2 (`src-tauri/`)
- **Por qué Tauri v2:** Proporciona un binario nativo ligero, rápido y seguro utilizando WebView2 de Windows, con comunicación IPC instantánea mediante `#[tauri::command]`.
- **Módulo `translator.rs` (Motor Híbrido Offline Inglés → Español):**
  - Implementa en Rust puro un motor lingüístico, fraseológico y morfológico determinista de alta velocidad para textos de hasta 20.000 caracteres.
  - Incluye tabla de colocaciones y expresiones multi-palabra ordenadas por longitud máxima, diccionario léxico técnico/corporativo/general, reglas de inversión sintáctica del inglés al español (`Adjetivo + Sustantivo` → `Sustantivo + Adjetivo` con concordancia de género y número), morfología de sufijos cognados grecolatinos (`-ation` → `-ación`, `-ity` → `-idad`, `-ment` → `-miento`, `-ally` → `-almente`) y contracciones gramaticales (`al`, `del`), preservando párrafos, saltos de línea y capitalización.
  - Adicionalmente, el frontend integra soporte opcional para cargar y almacenar en caché local el modelo neuronal **MarianMT (`Xenova/opus-mt-en-es`)** en formato **ONNX / WebAssembly**.

### 2.2. Interfaz Gráfica Editorial Moderna (`src/`)
- **Estética Visual:** Se diseñó una interfaz **Editorial Clara (Warm Paper `#F8F6F1`, Deep Navy `#1E3A8A` y Teal `#0F766E`)** con dos paneles de edición lado a lado, contador de caracteres disponibles en tiempo real (`0 / 20.000 caracteres`) y acciones rápidas (Traducir, Pegar del Portapapeles, Cargar Texto de Ejemplo, Limpiar y Copiar Traducción).
- **Estándares Web Modernos (Modern Web Guidance):**
  - Uso de `<dialog closedby="any">` nativo con `showModal()` y fallback de clic en el backdrop.
  - Uso del elemento semántico `<progress max="20000">` con estilizado personalizado (`appearance: none`, variables CSS, cambio de color preventivo al acercarse o alcanzar los 20.000 caracteres).
  - Uso de `resize: vertical` en `<textarea>`, `::selection`, `accent-color` y `scrollbar-color` accesible.

### 2.3. Empaquetado y Firma MSIX (`packaging/msix/`)
- Manifiesto `AppxManifest.xml` con identidad `VictoriaSanjurjo.TraducirTextosInglesAEspanol`, `Publisher="CN=CF220D35-0C1C-408C-BCB2-E85AA65BC95B"` y `PublisherDisplayName="Victoria Sanjurjo"`.
- El script automatizado `packaging/msix/build_msix.ps1` invoca `makeappx.exe` del Windows SDK para empaquetar el layout, utiliza o genera el certificado autofirmado para `CN=CF220D35-0C1C-408C-BCB2-E85AA65BC95B` y firma el paquete `.msix` con `signtool.exe` usando `SHA256`.
