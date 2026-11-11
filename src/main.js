/**
 * Traductor de Documentos - Inglés a Español (v1.0.0)
 * Desarrolladora: Victoria Sanjurjo (CN=CF220D35-0C1C-408C-BCB2-E85AA65BC95B)
 * Especializado en Documentos PDF Digitales (.pdf)
 * Frontend Controller (HTML5 + CSS3 + JS ESNext) conectado a Rust + Tauri v2
 */

(function () {
  "use strict";

  const invoke =
    window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke
      ? window.__TAURI__.core.invoke
      : null;

  let currentDocument = null;
  let isSyncingScroll = false;
  let onnxTranslatorPipeline = null;

  const btnOpenPdf = document.getElementById("btnOpenPdf");
  const btnSamplePdf = document.getElementById("btnSamplePdf");
  const btnExportPdf = document.getElementById("btnExportPdf");
  const btnNeuralEngine = document.getElementById("btnNeuralEngine");
  const btnAbout = document.getElementById("btnAbout");
  const btnRunOnnxModel = document.getElementById("btnRunOnnxModel");

  const chkSyncScroll = document.getElementById("chkSyncScroll");
  const statusText = document.getElementById("statusText");
  const progressWrapper = document.getElementById("progressWrapper");
  const translationProgress = document.getElementById("translationProgress");

  const docMetrics = document.getElementById("docMetrics");
  const metricFormat = document.getElementById("metricFormat");
  const metricPages = document.getElementById("metricPages");
  const metricSegments = document.getElementById("metricSegments");
  const metricWords = document.getElementById("metricWords");

  const emptyState = document.getElementById("emptyState");
  const hiddenFileInput = document.getElementById("hiddenFileInput");
  const splitPreview = document.getElementById("splitPreview");
  const sourcePaneTitle = document.getElementById("sourcePaneTitle");
  const targetPaneTitle = document.getElementById("targetPaneTitle");
  const sourceSheet = document.getElementById("sourceSheet");
  const targetSheet = document.getElementById("targetSheet");

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
  // 2. Utilidades de Estado y Barra de Progreso (<progress>)
  // =========================================================================
  function setStatus(message, progressMode = "hidden", progressValue = 0) {
    statusText.textContent = message;
    if (progressMode === "hidden") {
      progressWrapper.hidden = true;
    } else if (progressMode === "indeterminate") {
      progressWrapper.hidden = false;
      translationProgress.removeAttribute("value");
    } else if (progressMode === "determinate") {
      progressWrapper.hidden = false;
      translationProgress.value = progressValue;
    }
  }

  // =========================================================================
  // 3. Carga y Traducción de Documentos PDF (.pdf) vía Tauri Backend
  // =========================================================================
  async function handleSelectPdf() {
    if (!invoke) {
      hiddenFileInput.click();
      return;
    }

    try {
      const selectedPath = await invoke("select_pdf_dialog");
      if (!selectedPath) return;
      await processPdfByPath(selectedPath);
    } catch (err) {
      setStatus(`Error al seleccionar documento PDF: ${err}`, "hidden");
    }
  }

  async function processPdfByPath(filePath) {
    try {
      setStatus(`Analizando páginas del PDF y traduciendo de Inglés a Español...`, "indeterminate");
      const docData = await invoke("load_and_translate_pdf", { filePath });
      renderLoadedDocument(docData);

      if (onnxTranslatorPipeline) {
        await refineSegmentsWithOnnx();
      } else {
        setStatus(
          `Traducción de PDF completada (${docData.page_count} págs., ${docData.segments.length} bloques procesados de Inglés a Español). Lista para vista previa y exportación.`,
          "hidden"
        );
      }
    } catch (err) {
      setStatus(`Error procesando el documento PDF: ${err}`, "hidden");
    }
  }

  async function processDroppedPdf(file) {
    if (!file) return;
    const ext = file.name.split(".").pop().toLowerCase();
    if (ext !== "pdf") {
      setStatus(
        "Formato no soportado en v1.0.0. Por favor seleccione o arrastre únicamente un documento PDF (.pdf).",
        "hidden"
      );
      return;
    }

    try {
      setStatus(`Cargando "${file.name}" y traduciendo de Inglés a Español...`, "indeterminate");
      const arrayBuffer = await file.arrayBuffer();
      const uint8Array = Array.from(new Uint8Array(arrayBuffer));
      const docData = await invoke("load_and_translate_pdf_bytes", {
        fileName: file.name,
        bytes: uint8Array,
      });
      renderLoadedDocument(docData);
      if (onnxTranslatorPipeline) {
        await refineSegmentsWithOnnx();
      } else {
        setStatus(
          `Documento PDF "${docData.file_name}" traducido con éxito (${docData.segments.length} bloques).`,
          "hidden"
        );
      }
    } catch (err) {
      setStatus(`Error procesando el archivo PDF arrastrado: ${err}`, "hidden");
    }
  }

  async function handleLoadSamplePdf() {
    if (!invoke) return;
    try {
      setStatus(
        `Generando y traduciendo documento PDF de ejemplo en Inglés → Español...`,
        "indeterminate"
      );
      const docData = await invoke("generate_sample_pdf");
      renderLoadedDocument(docData);
      if (onnxTranslatorPipeline) {
        await refineSegmentsWithOnnx();
      } else {
        setStatus(
          `Documento PDF de ejemplo cargado y traducido (${docData.page_count} páginas, ${docData.segments.length} bloques). Puede editar cualquier párrafo o exportarlo a PDF directamente.`,
          "hidden"
        );
      }
    } catch (err) {
      setStatus(`Error cargando documento PDF de ejemplo: ${err}`, "hidden");
    }
  }

  // =========================================================================
  // 4. Renderizado de Vista Previa Lado a Lado con Preservación Jerárquica
  // =========================================================================
  function renderLoadedDocument(docData) {
    currentDocument = docData;

    metricFormat.textContent = "PDF";
    metricPages.textContent = docData.page_count;
    metricSegments.textContent = docData.segments.length;
    metricWords.textContent = docData.word_count;
    docMetrics.hidden = false;

    sourcePaneTitle.textContent = docData.file_name;
    const stem = docData.file_name.replace(/\.pdf$/i, "");
    targetPaneTitle.textContent = `${stem}_ES.pdf`;

    sourceSheet.innerHTML = "";
    targetSheet.innerHTML = "";

    let lastPage = 0;

    docData.segments.forEach((seg, idx) => {
      if (seg.page !== lastPage) {
        lastPage = seg.page;
        const srcDivider = document.createElement("div");
        srcDivider.className = "page-divider";
        srcDivider.textContent = `Página ${seg.page}`;
        sourceSheet.appendChild(srcDivider);

        const tgtDivider = document.createElement("div");
        tgtDivider.className = "page-divider";
        tgtDivider.textContent = `Página ${seg.page}`;
        targetSheet.appendChild(tgtDivider);
      }

      const srcBlock = document.createElement("article");
      srcBlock.className = `doc-block kind-${seg.kind}`;
      srcBlock.dataset.segmentIndex = String(idx);
      srcBlock.textContent = seg.source_text;

      const tgtBlock = document.createElement("article");
      tgtBlock.className = `doc-block kind-${seg.kind} editable-segment`;
      tgtBlock.dataset.segmentIndex = String(idx);
      tgtBlock.contentEditable = "true";
      tgtBlock.spellcheck = true;
      tgtBlock.setAttribute("role", "textbox");
      tgtBlock.setAttribute("aria-label", `Traducción del bloque ${idx + 1}`);
      tgtBlock.textContent = seg.translated_text;

      const highlightPair = (active) => {
        srcBlock.classList.toggle("highlighted", active);
        tgtBlock.classList.toggle("highlighted", active);
      };

      srcBlock.addEventListener("mouseenter", () => highlightPair(true));
      srcBlock.addEventListener("mouseleave", () => highlightPair(false));
      tgtBlock.addEventListener("mouseenter", () => highlightPair(true));
      tgtBlock.addEventListener("mouseleave", () => highlightPair(false));
      tgtBlock.addEventListener("focus", () => highlightPair(true));
      tgtBlock.addEventListener("blur", () => highlightPair(false));

      tgtBlock.addEventListener("input", () => {
        if (currentDocument && currentDocument.segments[idx]) {
          currentDocument.segments[idx].translated_text = tgtBlock.innerText;
        }
      });

      sourceSheet.appendChild(srcBlock);
      targetSheet.appendChild(tgtBlock);
    });

    emptyState.hidden = true;
    splitPreview.hidden = false;
    btnExportPdf.disabled = false;
  }

  // =========================================================================
  // 5. Exportación del Documento PDF Traducido (.pdf)
  // =========================================================================
  async function handleExportPdf() {
    if (!currentDocument || !invoke) return;

    try {
      setStatus(`Maquetando y exportando documento PDF traducido al Español...`, "indeterminate");
      const savedPath = await invoke("export_translated_pdf_dialog", {
        sourcePath: currentDocument.file_path,
        segments: currentDocument.segments,
      });

      if (savedPath) {
        setStatus(`Documento PDF exportado exitosamente en: ${savedPath}`, "hidden");
      } else {
        setStatus("Exportación cancelada por el usuario.", "hidden");
      }
    } catch (err) {
      setStatus(`Error al exportar el documento PDF: ${err}`, "hidden");
    }
  }

  // =========================================================================
  // 6. Motor Neuronal ONNX Opcional (MarianMT Xenova/opus-mt-en-es)
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

      if (currentDocument) {
        dialogEngine.close();
        await refineSegmentsWithOnnx();
      } else {
        onnxProgressNote.textContent =
          "¡Modelo Opus-MT listo! Cargue cualquier documento .pdf para traducirlo con la red neuronal.";
      }
    } catch (err) {
      onnxProgressNote.textContent =
        "Nota: Sin acceso a red para la descarga inicial de pesos ONNX; el Motor Lingüístico Offline Nativo en Rust sigue activo al 100%.";
    } finally {
      btnRunOnnxModel.disabled = false;
    }
  }

  async function refineSegmentsWithOnnx() {
    if (!onnxTranslatorPipeline || !currentDocument) return;
    const total = currentDocument.segments.length;
    for (let i = 0; i < total; i++) {
      const seg = currentDocument.segments[i];
      const pct = Math.round(((i + 1) / total) * 100);
      setStatus(
        `Traduciendo PDF con red neuronal Opus-MT (en-es): bloque ${i + 1} de ${total} (${pct}%)...`,
        "determinate",
        pct
      );
      const out = await onnxTranslatorPipeline(seg.source_text);
      if (Array.isArray(out) && out[0] && out[0].translation_text) {
        seg.translated_text = out[0].translation_text;
        const domNode = targetSheet.querySelector(`[data-segment-index="${i}"]`);
        if (domNode) {
          domNode.textContent = seg.translated_text;
        }
      }
    }
    setStatus(
      `Traducción neuronal Opus-MT del documento PDF completada (${total} bloques). Lista para exportar.`,
      "hidden"
    );
  }

  // =========================================================================
  // 7. Sincronización de Scroll Lado a Lado y Drag & Drop
  // =========================================================================
  function setupScrollSync() {
    sourceSheet.addEventListener("scroll", () => {
      if (!chkSyncScroll.checked || isSyncingScroll) return;
      isSyncingScroll = true;
      const ratio =
        sourceSheet.scrollTop /
        Math.max(1, sourceSheet.scrollHeight - sourceSheet.clientHeight);
      targetSheet.scrollTop =
        ratio * (targetSheet.scrollHeight - targetSheet.clientHeight);
      window.requestAnimationFrame(() => {
        isSyncingScroll = false;
      });
    });

    targetSheet.addEventListener("scroll", () => {
      if (!chkSyncScroll.checked || isSyncingScroll) return;
      isSyncingScroll = true;
      const ratio =
        targetSheet.scrollTop /
        Math.max(1, targetSheet.scrollHeight - targetSheet.clientHeight);
      sourceSheet.scrollTop =
        ratio * (sourceSheet.scrollHeight - sourceSheet.clientHeight);
      window.requestAnimationFrame(() => {
        isSyncingScroll = false;
      });
    });
  }

  function setupDragAndDrop() {
    emptyState.addEventListener("click", handleSelectPdf);
    emptyState.addEventListener("keydown", (e) => {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        handleSelectPdf();
      }
    });

    hiddenFileInput.addEventListener("change", (e) => {
      if (e.target.files && e.target.files[0]) {
        processDroppedPdf(e.target.files[0]);
      }
    });

    window.addEventListener("dragover", (e) => {
      e.preventDefault();
      emptyState.classList.add("drag-over");
    });

    window.addEventListener("dragleave", () => {
      emptyState.classList.remove("drag-over");
    });

    window.addEventListener("drop", (e) => {
      e.preventDefault();
      emptyState.classList.remove("drag-over");
      if (e.dataTransfer && e.dataTransfer.files && e.dataTransfer.files[0]) {
        processDroppedPdf(e.dataTransfer.files[0]);
      }
    });
  }

  btnOpenPdf.addEventListener("click", handleSelectPdf);
  btnSamplePdf.addEventListener("click", handleLoadSamplePdf);
  btnExportPdf.addEventListener("click", handleExportPdf);
  btnRunOnnxModel.addEventListener("click", handleRunOnnxModel);

  setupDialogs();
  setupScrollSync();
  setupDragAndDrop();
})();
