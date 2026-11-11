/**
 * Traducir textos inglés a español (v1.0.0)
 * Desarrolladora: Victoria Sanjurjo (CN=CF220D35-0C1C-408C-BCB2-E85AA65BC95B)
 * Frontend Controller (HTML5 + CSS3 + JS ESNext) conectado a Rust + Tauri v2
 * Límite máximo estricto: 20.000 caracteres
 */

(function () {
  "use strict";

  const MAX_CHARS = 20000;

  // Referencia al puente IPC de Tauri v2
  const invoke =
    window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke
      ? window.__TAURI__.core.invoke
      : null;

  let onnxTranslatorPipeline = null;
  let debounceTimer = null;

  // Elementos del DOM
  const btnTranslateNow = document.getElementById("btnTranslateNow");
  const btnPasteText = document.getElementById("btnPasteText");
  const btnSampleText = document.getElementById("btnSampleText");
  const btnClearText = document.getElementById("btnClearText");
  const btnCopyTranslation = document.getElementById("btnCopyTranslation");
  const btnNeuralEngine = document.getElementById("btnNeuralEngine");
  const btnAbout = document.getElementById("btnAbout");
  const btnRunOnnxModel = document.getElementById("btnRunOnnxModel");

  const chkAutoTranslate = document.getElementById("chkAutoTranslate");
  const statusText = document.getElementById("statusText");
  const charProgress = document.getElementById("charProgress");
  const charCounterBadge = document.getElementById("charCounterBadge");
  const charRemainingHint = document.getElementById("charRemainingHint");
  const targetStatsHint = document.getElementById("targetStatsHint");
  const metricWords = document.getElementById("metricWords");
  const metricParagraphs = document.getElementById("metricParagraphs");

  const sourceTextarea = document.getElementById("sourceTextarea");
  const targetTextarea = document.getElementById("targetTextarea");

  const dialogEngine = document.getElementById("dialogEngine");
  const dialogAbout = document.getElementById("dialogAbout");
  const onnxStatusBadge = document.getElementById("onnxStatusBadge");
  const onnxProgressNote = document.getElementById("onnxProgressNote");
  const engineLabel = document.getElementById("engineLabel");

  // =========================================================================
  // 1. Inicialización de Diálogos Modales con Fallback Light-Dismiss (MWG)
  // =========================================================================
  function setupDialogs() {
    const dialogs = document.querySelectorAll("dialog");
    dialogs.forEach((dialog) => {
      if (!("closedBy" in HTMLDialogElement.prototype)) {
        dialog.addEventListener("click", (event) => {
          if (event.target !== dialog) return;
          const rect = dialog.getBoundingClientRect();
          const isDialogContent =
            rect.top <= event.clientY &&
            event.clientY <= rect.top + rect.height &&
            rect.left <= event.clientX &&
            event.clientX <= rect.left + rect.width;
          if (isDialogContent) return;
          dialog.close();
        });
      }
    });

    document.querySelectorAll("[data-close-dialog]").forEach((btn) => {
      btn.addEventListener("click", () => {
        const targetId = btn.getAttribute("data-close-dialog");
        const dlg = document.getElementById(targetId);
        if (dlg) dlg.close();
      });
    });

    btnNeuralEngine.addEventListener("click", () => {
      dialogEngine.showModal();
    });

    btnAbout.addEventListener("click", () => {
      dialogAbout.showModal();
    });
  }

  // =========================================================================
  // 2. Control Estricto del Límite de 20.000 Caracteres y Métricas
  // =========================================================================
  function enforceMaxCharsAndMetrics() {
    let rawText = sourceTextarea.value;
    // Contar code points reales (compatible con Unicode)
    const charsArray = Array.from(rawText);
    let wasTruncated = false;

    if (charsArray.length > MAX_CHARS) {
      rawText = charsArray.slice(0, MAX_CHARS).join("");
      sourceTextarea.value = rawText;
      wasTruncated = true;
    }

    const currentLen = wasTruncated ? MAX_CHARS : charsArray.length;
    const remaining = MAX_CHARS - currentLen;

    const formattedCurrent = currentLen.toLocaleString("es-ES");
    const formattedMax = MAX_CHARS.toLocaleString("es-ES");
    const formattedRemaining = remaining.toLocaleString("es-ES");

    charCounterBadge.textContent = `${formattedCurrent} / ${formattedMax} caracteres`;
    charRemainingHint.textContent = `${formattedRemaining} caracteres disponibles`;
    charProgress.value = currentLen;

    charProgress.classList.remove("limit-near", "limit-reached");
    charCounterBadge.classList.remove("limit-danger");

    if (currentLen >= MAX_CHARS) {
      charProgress.classList.add("limit-reached");
      charCounterBadge.classList.add("limit-danger");
    } else if (currentLen >= MAX_CHARS * 0.85) {
      charProgress.classList.add("limit-near");
    }

    const trimmed = rawText.trim();
    const wordCount = trimmed ? trimmed.split(/\s+/).length : 0;
    const paraCount = trimmed
      ? rawText.split(/\r?\n/).filter((l) => l.trim().length > 0).length
      : 0;

    metricWords.textContent = wordCount.toLocaleString("es-ES");
    metricParagraphs.textContent = paraCount.toLocaleString("es-ES");

    if (wasTruncated) {
      statusText.textContent =
        "Aviso: Se ha alcanzado el límite máximo de 20.000 caracteres. El texto excedente fue recortado.";
    }

    return rawText;
  }

  // =========================================================================
  // 3. Ejecución de la Traducción (Inglés -> Español)
  // =========================================================================
  async function executeTranslation() {
    const sourceText = enforceMaxCharsAndMetrics();

    if (!sourceText.trim()) {
      targetTextarea.value = "";
      targetStatsHint.textContent = "0 caracteres · 0 palabras";
      btnCopyTranslation.disabled = true;
      statusText.textContent =
        "Listo. Escriba o pegue un texto en inglés (máximo 20.000 caracteres) para traducirlo al español.";
      return;
    }

    try {
      if (onnxTranslatorPipeline) {
        statusText.textContent = "Traduciendo texto con red neuronal Opus-MT (Inglés → Español)...";
        const paragraphs = sourceText.split("\n");
        const translatedParagraphs = [];
        for (const p of paragraphs) {
          if (!p.trim()) {
            translatedParagraphs.push(p);
          } else {
            const out = await onnxTranslatorPipeline(p);
            if (Array.isArray(out) && out[0] && out[0].translation_text) {
              translatedParagraphs.push(out[0].translation_text);
            } else {
              translatedParagraphs.push(p);
            }
          }
        }
        const finalEs = translatedParagraphs.join("\n");
        updateTargetOutput(finalEs);
        statusText.textContent = "Traducción neuronal completada (Inglés → Español).";
        return;
      }

      if (invoke) {
        const response = await invoke("translate_text", { text: sourceText });
        updateTargetOutput(response.translated_text);
        statusText.textContent = `Texto traducido de Inglés a Español (${response.source_char_count.toLocaleString(
          "es-ES"
        )} / 20.000 caracteres).`;
      }
    } catch (err) {
      statusText.textContent = `Error en la traducción: ${err}`;
    }
  }

  function updateTargetOutput(translatedText) {
    targetTextarea.value = translatedText;
    const tChars = Array.from(translatedText).length;
    const tWords = translatedText.trim()
      ? translatedText.trim().split(/\s+/).length
      : 0;
    targetStatsHint.textContent = `${tChars.toLocaleString(
      "es-ES"
    )} caracteres · ${tWords.toLocaleString("es-ES")} palabras`;
    btnCopyTranslation.disabled = tChars === 0;
  }

  // =========================================================================
  // 4. Acciones de Barra de Herramientas (Pegar, Ejemplo, Limpiar, Copiar)
  // =========================================================================
  async function handlePasteFromClipboard() {
    try {
      const clipText = await navigator.clipboard.readText();
      if (!clipText) return;
      sourceTextarea.value = clipText;
      await executeTranslation();
    } catch (err) {
      statusText.textContent =
        "Use Ctrl+V directamente dentro del cuadro de texto en inglés para pegar.";
      sourceTextarea.focus();
    }
  }

  async function handleLoadSampleText() {
    try {
      if (invoke) {
        const sample = await invoke("get_sample_english_text");
        sourceTextarea.value = sample;
      } else {
        sourceTextarea.value =
          "EXECUTIVE SUMMARY\n\nArtificial intelligence and machine learning models improve the translation performance in real time.";
      }
      await executeTranslation();
    } catch (err) {
      statusText.textContent = `Error cargando texto de ejemplo: ${err}`;
    }
  }

  function handleClearText() {
    sourceTextarea.value = "";
    targetTextarea.value = "";
    enforceMaxCharsAndMetrics();
    targetStatsHint.textContent = "0 caracteres · 0 palabras";
    btnCopyTranslation.disabled = true;
    statusText.textContent = "Campos limpiados. Listo para traducir (máx. 20.000 caracteres).";
    sourceTextarea.focus();
  }

  async function handleCopyTranslation() {
    const text = targetTextarea.value;
    if (!text) return;
    try {
      await navigator.clipboard.writeText(text);
      statusText.textContent = "¡Traducción en español copiada al portapapeles!";
    } catch (err) {
      targetTextarea.select();
      document.execCommand("copy");
      statusText.textContent = "¡Traducción en español copiada al portapapeles!";
    }
  }

  // =========================================================================
  // 5. Motor Neuronal ONNX Opcional (MarianMT Xenova/opus-mt-en-es)
  // =========================================================================
  async function handleRunOnnxModel() {
    try {
      btnRunOnnxModel.disabled = true;
      onnxProgressNote.textContent =
        "Inicializando runtime ONNX WebAssembly y verificando caché de Xenova/opus-mt-en-es...";

      if (!onnxTranslatorPipeline) {
        const transformersModule = await import(
          "https://cdn.jsdelivr.net/npm/@huggingface/transformers@3.3.3"
        );
        const { pipeline, env } = transformersModule;
        env.allowLocalModels = true;
        env.useBrowserCache = true;

        onnxTranslatorPipeline = await pipeline(
          "translation_en_to_es",
          "Xenova/opus-mt-en-es",
          {
            progress_callback: (info) => {
              if (info.status === "progress" && typeof info.progress === "number") {
                onnxProgressNote.textContent = `Descargando en caché local (${info.file}): ${Math.round(
                  info.progress
                )}%`;
              } else if (info.status === "ready") {
                onnxProgressNote.textContent =
                  "Modelo Neuronal Opus-MT (en-es) cargado y almacenado en caché local.";
              }
            },
          }
        );

        onnxStatusBadge.textContent = "Activo en Caché Local";
        onnxStatusBadge.className = "badge-ready";
        engineLabel.textContent = "Motor Neuronal Opus-MT Activo";
      }

      dialogEngine.close();
      await executeTranslation();
    } catch (err) {
      onnxProgressNote.textContent =
        "Nota: Sin acceso a red para la descarga inicial de pesos ONNX; el Motor Lingüístico Offline Nativo en Rust sigue activo al 100%.";
    } finally {
      btnRunOnnxModel.disabled = false;
    }
  }

  // =========================================================================
  // 6. Eventos de Entrada y Sincronización de Scroll
  // =========================================================================
  sourceTextarea.addEventListener("input", () => {
    enforceMaxCharsAndMetrics();
    if (chkAutoTranslate.checked) {
      if (debounceTimer) clearTimeout(debounceTimer);
      debounceTimer = setTimeout(() => {
        executeTranslation();
      }, 120);
    }
  });

  btnTranslateNow.addEventListener("click", executeTranslation);
  btnPasteText.addEventListener("click", handlePasteFromClipboard);
  btnSampleText.addEventListener("click", handleLoadSampleText);
  btnClearText.addEventListener("click", handleClearText);
  btnCopyTranslation.addEventListener("click", handleCopyTranslation);
  btnRunOnnxModel.addEventListener("click", handleRunOnnxModel);

  setupDialogs();
  enforceMaxCharsAndMetrics();
})();
