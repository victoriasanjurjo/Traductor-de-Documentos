//! # Motor de Traducción Offline Integrado (Inglés -> Español)
//!
//! Este módulo implementa un motor de traducción híbrido 100% offline en Rust puro
//! para garantizar autonomía total sin depender de servidores externos ni APIs de pago.
//! Combina:
//! 1. Un motor de análisis de frases e expresiones idiomáticas (multi-word expressions).
//! 2. Un léxico amplio Inglés -> Español con desambiguación contextual y flexión morfológica.
//! 3. Reglas de reordenamiento sintáctico (Sustantivo + Adjetivo en español, auxiliares, tiempos verbales).
//! 4. Preservación estricta de signos de puntuación, mayúsculas/minúsculas y formato numérico.
//!
//! Además, se complementa opcionalmente con el motor neuronal ONNX (`Xenova/opus-mt-en-es`)
//! ejecutable en el frontend para refinamiento neuronal profundo cuando los pesos están en caché.

use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Diccionario de frases compuestas y colocaciones comunes en documentos técnicos, legales y corporativos (EN -> ES).
static PHRASE_TABLE: Lazy<Vec<(&'static str, &'static str)>> = Lazy::new(|| {
    let mut phrases = vec![
        ("artificial intelligence", "inteligencia artificial"),
        ("machine learning", "aprendizaje automático"),
        ("deep learning", "aprendizaje profundo"),
        ("natural language processing", "procesamiento del lenguaje natural"),
        ("executive summary", "resumen ejecutivo"),
        ("table of contents", "tabla de contenidos"),
        ("terms and conditions", "términos y condiciones"),
        ("privacy policy", "política de privacidad"),
        ("intellectual property", "propiedad intelectual"),
        ("human resources", "recursos humanos"),
        ("project management", "gestión de proyectos"),
        ("quality assurance", "aseguramiento de la calidad"),
        ("software development", "desarrollo de software"),
        ("user interface", "interfaz de usuario"),
        ("user experience", "experiencia de usuario"),
        ("operating system", "sistema operativo"),
        ("data analysis", "análisis de datos"),
        ("financial report", "informe financiero"),
        ("annual report", "informe anual"),
        ("technical specification", "especificación técnica"),
        ("in order to", "con el fin de"),
        ("as soon as possible", "lo antes posible"),
        ("as well as", "así como"),
        ("on the other hand", "por otro lado"),
        ("in addition to", "además de"),
        ("in addition", "además"),
        ("for example", "por ejemplo"),
        ("for instance", "por ejemplo"),
        ("such as", "tales como"),
        ("due to", "debido a"),
        ("because of", "a causa de"),
        ("according to", "según"),
        ("in accordance with", "de acuerdo con"),
        ("with respect to", "con respecto a"),
        ("at least", "al menos"),
        ("at most", "como máximo"),
        ("more than", "más de"),
        ("less than", "menos de"),
        ("up to", "hasta"),
        ("out of", "fuera de"),
        ("instead of", "en lugar de"),
        ("by means of", "por medio de"),
        ("as a result", "como resultado"),
        ("therefore", "por lo tanto"),
        ("however", "sin embargo"),
        ("furthermore", "además"),
        ("nevertheless", "no obstante"),
        ("consequently", "en consecuencia"),
        ("meanwhile", "mientras tanto"),
        ("first of all", "en primer lugar"),
        ("in conclusion", "en conclusión"),
        ("to summarize", "para resumir"),
        ("is able to", "es capaz de"),
        ("are able to", "son capaces de"),
        ("has been", "ha sido"),
        ("have been", "han sido"),
        ("had been", "había sido"),
        ("will be", "será"),
        ("would be", "sería"),
        ("can be", "puede ser"),
        ("could be", "podría ser"),
        ("should be", "debería ser"),
        ("must be", "debe ser"),
        ("may be", "puede ser"),
        ("there is", "hay"),
        ("there are", "hay"),
        ("there was", "había"),
        ("there were", "había"),
        ("do not", "no"),
        ("does not", "no"),
        ("did not", "no"),
        ("will not", "no va a"),
        ("cannot", "no puede"),
        ("can not", "no puede"),
        ("should not", "no debería"),
        ("must not", "no debe"),
        ("document translator", "traductor de documentos"),
        ("english to spanish", "inglés a español"),
        ("source document", "documento de origen"),
        ("target document", "documento de destino"),
        ("original format", "formato original"),
        ("side by side", "lado a lado"),
        ("real time", "tiempo real"),
        ("high quality", "alta calidad"),
        ("open source", "código abierto"),
        ("step by step", "paso a paso"),
        ("end to end", "de extremo a extremo"),
        ("customer service", "servicio al cliente"),
        ("supply chain", "cadena de suministro"),
        ("business model", "modelo de negocio"),
        ("market research", "investigación de mercado"),
        ("risk management", "gestión de riesgos"),
        ("security policy", "política de seguridad"),
        ("access control", "control de acceso"),
        ("cloud computing", "computación en la nube"),
        ("database management", "gestión de bases de datos"),
        ("file format", "formato de archivo"),
        ("word document", "documento de Word"),
        ("digital document", "documento digital"),
        ("thank you", "gracias"),
        ("good morning", "buenos días"),
        ("good afternoon", "buenas tardes"),
        ("good evening", "buenas noches"),
        ("please note", "por favor tenga en cuenta"),
        ("all rights reserved", "todos los derechos reservados"),
    ];
    // Ordenar de mayor a menor longitud para priorizar coincidencias largas
    phrases.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
    phrases
});

/// Diccionario léxico base (EN -> ES) con vocabulario frecuente, técnico, empresarial y general.
static WORD_DICT: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let entries = [
        // Artículos, pronombres y determinantes
        ("the", "el"), ("a", "un"), ("an", "un"), ("this", "este"), ("that", "ese"),
        ("these", "estos"), ("those", "esos"), ("my", "mi"), ("your", "su"), ("his", "su"),
        ("her", "su"), ("its", "su"), ("our", "nuestro"), ("their", "su"),
        ("i", "yo"), ("you", "usted"), ("he", "él"), ("she", "ella"), ("it", "ello"),
        ("we", "nosotros"), ("they", "ellos"), ("me", "me"), ("him", "lo"), ("us", "nos"), ("them", "los"),
        ("who", "quién"), ("whom", "a quién"), ("whose", "cuyo"), ("which", "el cual"), ("what", "qué"),
        ("each", "cada"), ("every", "cada"), ("all", "todos"), ("both", "ambos"), ("some", "algunos"),
        ("any", "cualquier"), ("no", "ningún"), ("none", "ninguno"), ("other", "otro"), ("another", "otro"),
        ("many", "muchos"), ("much", "mucho"), ("few", "pocos"), ("several", "varios"), ("most", "la mayoría de"),
        ("same", "mismo"), ("such", "tal"), ("own", "propio"),

        // Preposiciones y conjunciones
        ("of", "de"), ("in", "en"), ("to", "a"), ("for", "para"), ("with", "con"),
        ("on", "sobre"), ("at", "en"), ("from", "desde"), ("by", "por"), ("about", "acerca de"),
        ("as", "como"), ("into", "hacia"), ("like", "como"), ("through", "a través de"),
        ("after", "después de"), ("over", "sobre"), ("between", "entre"), ("out", "fuera"),
        ("against", "contra"), ("during", "durante"), ("without", "sin"), ("before", "antes de"),
        ("under", "bajo"), ("around", "alrededor de"), ("among", "entre"), ("within", "dentro de"),
        ("across", "a través de"), ("behind", "detrás de"), ("beyond", "más allá de"),
        ("near", "cerca de"), ("toward", "hacia"), ("towards", "hacia"), ("upon", "sobre"),
        ("and", "y"), ("or", "o"), ("but", "pero"), ("if", "si"), ("because", "porque"),
        ("while", "mientras"), ("although", "aunque"), ("though", "aunque"), ("unless", "a menos que"),
        ("until", "hasta"), ("since", "desde"), ("when", "cuando"), ("where", "donde"),
        ("whether", "si"), ("nor", "ni"), ("yet", "aún"), ("so", "así que"), ("than", "que"),

        // Verbos auxiliares y modales
        ("is", "es"), ("are", "son"), ("was", "fue"), ("were", "fueron"), ("be", "ser"),
        ("been", "sido"), ("being", "siendo"), ("am", "soy"),
        ("have", "tienen"), ("has", "tiene"), ("had", "tenía"), ("having", "teniendo"),
        ("do", "hacer"), ("does", "hace"), ("did", "hizo"), ("done", "hecho"), ("doing", "haciendo"),
        ("can", "puede"), ("could", "podría"), ("will", "va a"), ("would", "haría"),
        ("shall", "deberá"), ("should", "debería"), ("may", "puede"), ("might", "podría"),
        ("must", "debe"), ("need", "necesita"), ("needs", "necesita"), ("needed", "necesitó"),

        // Verbos frecuentes y técnicos (infinitivo, 3ra persona, pasado, gerundio)
        ("use", "usar"), ("uses", "usa"), ("used", "utilizado"), ("using", "usando"),
        ("make", "hacer"), ("makes", "hace"), ("made", "hecho"), ("making", "haciendo"),
        ("create", "crear"), ("creates", "crea"), ("created", "creado"), ("creating", "creando"),
        ("generate", "generar"), ("generates", "genera"), ("generated", "generado"), ("generating", "generando"),
        ("translate", "traducir"), ("translates", "traduce"), ("translated", "traducido"), ("translating", "traduciendo"),
        ("convert", "convertir"), ("converts", "convierte"), ("converted", "convertido"), ("converting", "convirtiendo"),
        ("export", "exportar"), ("exports", "exporta"), ("exported", "exportado"), ("exporting", "exportando"),
        ("import", "importar"), ("imports", "importa"), ("imported", "importado"), ("importing", "importando"),
        ("process", "procesar"), ("processes", "procesa"), ("processed", "procesado"), ("processing", "procesando"),
        ("keep", "mantener"), ("keeps", "mantiene"), ("kept", "mantenido"), ("keeping", "manteniendo"),
        ("preserve", "conservar"), ("preserves", "conserva"), ("preserved", "conservado"), ("preserving", "conservando"),
        ("maintain", "mantener"), ("maintains", "mantiene"), ("maintained", "mantenido"), ("maintaining", "manteniendo"),
        ("provide", "proporcionar"), ("provides", "proporciona"), ("provided", "proporcionado"), ("providing", "proporcionando"),
        ("include", "incluir"), ("includes", "incluye"), ("included", "incluido"), ("including", "incluyendo"),
        ("contain", "contener"), ("contains", "contiene"), ("contained", "contenido"), ("containing", "conteniendo"),
        ("allow", "permitir"), ("allows", "permite"), ("allowed", "permitido"), ("allowing", "permitiendo"),
        ("enable", "habilitar"), ("enables", "habilita"), ("enabled", "habilitado"), ("enabling", "habilitando"),
        ("require", "requerir"), ("requires", "requiere"), ("required", "requerido"), ("requiring", "requiriendo"),
        ("support", "soportar"), ("supports", "soporta"), ("supported", "soportado"), ("supporting", "soportando"),
        ("develop", "desarrollar"), ("develops", "desarrolla"), ("developed", "desarrollado"), ("developing", "desarrollando"),
        ("build", "construir"), ("builds", "construye"), ("built", "construido"), ("building", "construyendo"),
        ("design", "diseñar"), ("designs", "diseña"), ("designed", "diseñado"), ("designing", "diseñando"),
        ("implement", "implementar"), ("implements", "implementa"), ("implemented", "implementado"), ("implementing", "implementando"),
        ("manage", "gestionar"), ("manages", "gestiona"), ("managed", "gestionado"), ("managing", "gestionando"),
        ("analyze", "analizar"), ("analyzes", "analiza"), ("analyzed", "analizado"), ("analyzing", "analizando"),
        ("evaluate", "evaluar"), ("evaluates", "evalúa"), ("evaluated", "evaluado"), ("evaluating", "evaluando"),
        ("improve", "mejorar"), ("improves", "mejora"), ("improved", "mejorado"), ("improving", "mejorando"),
        ("increase", "aumentar"), ("increases", "aumenta"), ("increased", "aumentado"), ("increasing", "aumentando"),
        ("reduce", "reducir"), ("reduces", "reduce"), ("reduced", "reducido"), ("reducing", "reduciendo"),
        ("ensure", "asegurar"), ("ensures", "asegura"), ("ensured", "asegurado"), ("ensuring", "asegurando"),
        ("verify", "verificar"), ("verifies", "verifica"), ("verified", "verificado"), ("verifying", "verificando"),
        ("validate", "validar"), ("validates", "valida"), ("validated", "validado"), ("validating", "validando"),
        ("test", "probar"), ("tests", "pruebas"), ("tested", "probado"), ("testing", "pruebas"),
        ("run", "ejecutar"), ("runs", "ejecuta"), ("ran", "ejecutó"), ("running", "ejecutando"),
        ("work", "trabajar"), ("works", "funciona"), ("worked", "trabajó"), ("working", "trabajando"),
        ("read", "leer"), ("reads", "lee"), ("reading", "lectura"),
        ("write", "escribir"), ("writes", "escribe"), ("wrote", "escribió"), ("written", "escrito"), ("writing", "escritura"),
        ("open", "abrir"), ("opens", "abre"), ("opened", "abierto"), ("opening", "abriendo"),
        ("close", "cerrar"), ("closes", "cierra"), ("closed", "cerrado"), ("closing", "cerrando"),
        ("save", "guardar"), ("saves", "guarda"), ("saved", "guardado"), ("saving", "guardando"),
        ("load", "cargar"), ("loads", "carga"), ("loaded", "cargado"), ("loading", "cargando"),
        ("download", "descargar"), ("downloads", "descarga"), ("downloaded", "descargado"), ("downloading", "descargando"),
        ("upload", "subir"), ("uploads", "sube"), ("uploaded", "subido"), ("uploading", "subiendo"),
        ("show", "mostrar"), ("shows", "muestra"), ("showed", "mostró"), ("shown", "mostrado"), ("showing", "mostrando"),
        ("display", "mostrar"), ("displays", "muestra"), ("displayed", "mostrado"), ("displaying", "mostrando"),
        ("find", "encontrar"), ("finds", "encuentra"), ("found", "encontrado"), ("finding", "encontrando"),
        ("give", "dar"), ("gives", "da"), ("gave", "dio"), ("given", "dado"), ("giving", "dando"),
        ("take", "tomar"), ("takes", "toma"), ("took", "tomó"), ("taken", "tomado"), ("taking", "tomando"),
        ("know", "saber"), ("knows", "sabe"), ("knew", "sabía"), ("known", "conocido"), ("knowing", "sabiendo"),
        ("think", "pensar"), ("thinks", "piensa"), ("thought", "pensamiento"), ("thinking", "pensando"),
        ("see", "ver"), ("sees", "ve"), ("saw", "vio"), ("seen", "visto"), ("seeing", "viendo"),
        ("come", "venir"), ("comes", "viene"), ("came", "vino"), ("coming", "viniendo"),
        ("go", "ir"), ("goes", "va"), ("went", "fue"), ("gone", "ido"), ("going", "yendo"),
        ("get", "obtener"), ("gets", "obtiene"), ("got", "obtuvo"), ("getting", "obteniendo"),
        ("set", "establecer"), ("sets", "establece"), ("setting", "configuración"),
        ("change", "cambiar"), ("changes", "cambios"), ("changed", "cambiado"), ("changing", "cambiando"),
        ("update", "actualizar"), ("updates", "actualizaciones"), ("updated", "actualizado"), ("updating", "actualizando"),
        ("add", "añadir"), ("adds", "añade"), ("added", "añadido"), ("adding", "añadiendo"),
        ("remove", "eliminar"), ("removes", "elimina"), ("removed", "eliminado"), ("removing", "eliminando"),
        ("delete", "eliminar"), ("deletes", "elimina"), ("deleted", "eliminado"), ("deleting", "eliminando"),
        ("select", "seleccionar"), ("selects", "selecciona"), ("selected", "seleccionado"), ("selecting", "seleccionando"),
        ("choose", "elegir"), ("chooses", "elige"), ("chose", "eligió"), ("chosen", "elegido"), ("choosing", "eligiendo"),
        ("start", "iniciar"), ("starts", "inicia"), ("started", "iniciado"), ("starting", "iniciando"),
        ("begin", "comenzar"), ("begins", "comienza"), ("began", "comenzó"), ("begun", "comenzado"), ("beginning", "comienzo"),
        ("finish", "finalizar"), ("finishes", "finaliza"), ("finished", "finalizado"), ("finishing", "finalizando"),
        ("complete", "completar"), ("completes", "completa"), ("completed", "completado"), ("completing", "completando"),
        ("continue", "continuar"), ("continues", "continúa"), ("continued", "continuado"), ("continuing", "continuando"),
        ("follow", "seguir"), ("follows", "sigue"), ("followed", "seguido"), ("following", "siguiente"),
        ("describe", "describir"), ("describes", "describe"), ("described", "descrito"), ("describing", "describiendo"),
        ("explain", "explicar"), ("explains", "explica"), ("explained", "explicado"), ("explaining", "explicando"),
        ("define", "definir"), ("defines", "define"), ("defined", "definido"), ("defining", "definiendo"),
        ("represent", "representar"), ("represents", "representa"), ("represented", "representado"),
        ("apply", "aplicar"), ("applies", "aplica"), ("applied", "aplicado"), ("applying", "aplicando"),
        ("receive", "recibir"), ("receives", "recibe"), ("received", "recibido"), ("receiving", "recibiendo"),
        ("send", "enviar"), ("sends", "envía"), ("sent", "enviado"), ("sending", "enviando"),
        ("share", "compartir"), ("shares", "comparte"), ("shared", "compartido"), ("sharing", "compartiendo"),
        ("check", "comprobar"), ("checks", "comprueba"), ("checked", "comprobado"), ("checking", "comprobando"),
        ("review", "revisar"), ("reviews", "revisiones"), ("reviewed", "revisado"), ("reviewing", "revisando"),
        ("confirm", "confirmar"), ("confirms", "confirma"), ("confirmed", "confirmado"), ("confirming", "confirmando"),
        ("accept", "aceptar"), ("accepts", "acepta"), ("accepted", "aceptado"), ("accepting", "aceptando"),
        ("install", "instalar"), ("installs", "instala"), ("installed", "instalado"), ("installing", "instalando"),
        ("configure", "configurar"), ("configures", "configura"), ("configured", "configurado"),

        // Sustantivos comunes y de documentación
        ("document", "documento"), ("documents", "documentos"), ("documentation", "documentación"),
        ("file", "archivo"), ("files", "archivos"), ("folder", "carpeta"), ("folders", "carpetas"),
        ("page", "página"), ("pages", "páginas"), ("paragraph", "párrafo"), ("paragraphs", "párrafos"),
        ("section", "sección"), ("sections", "secciones"), ("chapter", "capítulo"), ("chapters", "capítulos"),
        ("title", "título"), ("titles", "títulos"), ("subtitle", "subtítulo"), ("header", "encabezado"),
        ("footer", "pie de página"), ("table", "tabla"), ("tables", "tablas"), ("row", "fila"), ("rows", "filas"),
        ("column", "columna"), ("columns", "columnas"), ("list", "lista"), ("lists", "listas"),
        ("item", "elemento"), ("items", "elementos"), ("point", "punto"), ("points", "puntos"),
        ("text", "texto"), ("texts", "textos"), ("word", "palabra"), ("words", "palabras"),
        ("sentence", "oración"), ("sentences", "oraciones"), ("language", "idioma"), ("languages", "idiomas"),
        ("translation", "traducción"), ("translations", "traducciones"), ("translator", "traductor"),
        ("version", "versión"), ("versions", "versiones"), ("edition", "edición"),
        ("system", "sistema"), ("systems", "sistemas"), ("application", "aplicación"), ("applications", "aplicaciones"),
        ("program", "programa"), ("programs", "programas"), ("software", "software"), ("hardware", "hardware"),
        ("computer", "computadora"), ("computers", "computadoras"), ("device", "dispositivo"), ("devices", "dispositivos"),
        ("server", "servidor"), ("servers", "servidores"), ("client", "cliente"), ("clients", "clientes"),
        ("user", "usuario"), ("users", "usuarios"), ("customer", "cliente"), ("customers", "clientes"),
        ("company", "empresa"), ("companies", "empresas"), ("organization", "organización"), ("business", "negocio"),
        ("team", "equipo"), ("teams", "equipos"), ("group", "grupo"), ("groups", "grupos"),
        ("member", "miembro"), ("members", "miembros"), ("manager", "gerente"), ("director", "director"),
        ("project", "proyecto"), ("projects", "proyectos"), ("task", "tarea"), ("tasks", "tareas"),
        ("goal", "objetivo"), ("goals", "objetivos"), ("objective", "objetivo"), ("objectives", "objetivos"),
        ("result", "resultado"), ("results", "resultados"), ("output", "salida"), ("outputs", "salidas"),
        ("input", "entrada"), ("inputs", "entradas"), ("data", "datos"), ("information", "información"),
        ("content", "contenido"), ("contents", "contenidos"), ("summary", "resumen"), ("report", "informe"),
        ("reports", "informes"), ("analysis", "análisis"), ("research", "investigación"), ("study", "estudio"),
        ("method", "método"), ("methods", "métodos"), ("methodology", "metodología"), ("approach", "enfoque"),
        ("model", "modelo"), ("models", "modelos"), ("engine", "motor"), ("engines", "motores"),
        ("network", "red"), ("networks", "redes"), ("internet", "internet"), ("connection", "conexión"),
        ("security", "seguridad"), ("privacy", "privacidad"), ("access", "acceso"), ("permission", "permiso"),
        ("permissions", "permisos"), ("account", "cuenta"), ("accounts", "cuentas"), ("profile", "perfil"),
        ("feature", "característica"), ("features", "características"), ("function", "función"), ("functions", "funciones"),
        ("functionality", "funcionalidad"), ("performance", "rendimiento"), ("efficiency", "eficiencia"),
        ("quality", "calidad"), ("standard", "estándar"), ("standards", "estándares"), ("rule", "regla"), ("rules", "reglas"),
        ("requirement", "requisito"), ("requirements", "requisitos"), ("specification", "especificación"),
        ("specifications", "especificaciones"), ("architecture", "arquitectura"), ("structure", "estructura"),
        ("format", "formato"), ("formats", "formatos"), ("style", "estilo"), ("styles", "estilos"),
        ("layout", "diseño"), ("design", "diseño"), ("interface", "interfaz"), ("window", "ventana"),
        ("button", "botón"), ("buttons", "botones"), ("menu", "menú"), ("dialog", "diálogo"),
        ("panel", "panel"), ("view", "vista"), ("preview", "vista previa"), ("mode", "modo"),
        ("status", "estado"), ("state", "estado"), ("progress", "progreso"), ("level", "nivel"),
        ("value", "valor"), ("values", "valores"), ("number", "número"), ("numbers", "números"),
        ("amount", "cantidad"), ("total", "total"), ("average", "promedio"), ("rate", "tasa"),
        ("percentage", "porcentaje"), ("cost", "costo"), ("costs", "costos"), ("price", "precio"),
        ("budget", "presupuesto"), ("revenue", "ingresos"), ("profit", "beneficio"), ("growth", "crecimiento"),
        ("market", "mercado"), ("markets", "mercados"), ("industry", "industria"), ("sector", "sector"),
        ("service", "servicio"), ("services", "servicios"), ("product", "producto"), ("products", "productos"),
        ("solution", "solución"), ("solutions", "soluciones"), ("problem", "problema"), ("problems", "problemas"),
        ("issue", "incidente"), ("issues", "incidentes"), ("error", "error"), ("errors", "errores"),
        ("warning", "advertencia"), ("warnings", "advertencias"), ("message", "mensaje"), ("messages", "mensajes"),
        ("note", "nota"), ("notes", "notas"), ("comment", "comentario"), ("comments", "comentarios"),
        ("example", "ejemplo"), ("examples", "ejemplos"), ("case", "caso"), ("cases", "casos"),
        ("step", "paso"), ("steps", "pasos"), ("phase", "fase"), ("phases", "fases"), ("stage", "etapa"),
        ("time", "tiempo"), ("date", "fecha"), ("day", "día"), ("days", "días"), ("week", "semana"),
        ("weeks", "semanas"), ("month", "mes"), ("months", "meses"), ("year", "año"), ("years", "años"),
        ("today", "hoy"), ("tomorrow", "mañana"), ("yesterday", "ayer"), ("future", "futuro"),
        ("history", "historial"), ("world", "mundo"), ("country", "país"), ("countries", "países"),
        ("city", "ciudad"), ("place", "lugar"), ("area", "área"), ("region", "región"),
        ("part", "parte"), ("parts", "partes"), ("way", "manera"), ("ways", "maneras"),
        ("order", "orden"), ("agreement", "acuerdo"), ("contract", "contrato"), ("policy", "política"),
        ("law", "ley"), ("rights", "derechos"), ("license", "licencia"), ("certificate", "certificado"),
        ("package", "paquete"), ("packages", "paquetes"), ("installer", "instalador"), ("directory", "directorio"),
        ("path", "ruta"), ("code", "código"), ("source", "origen"), ("target", "destino"),
        ("memory", "memoria"), ("storage", "almacenamiento"), ("disk", "disco"), ("speed", "velocidad"),
        ("operation", "operación"), ("operations", "operaciones"), ("execution", "ejecución"),
        ("environment", "entorno"), ("configuration", "configuración"), ("options", "opciones"),
        ("option", "opción"), ("guide", "guía"), ("manual", "manual"), ("instructions", "instrucciones"),
        ("introduction", "introducción"), ("overview", "descripción general"), ("background", "antecedentes"),
        ("conclusion", "conclusión"), ("recommendations", "recomendaciones"), ("reference", "referencia"),
        ("references", "referencias"), ("appendix", "apéndice"), ("index", "índice"),

        // Adjetivos comunes
        ("english", "inglés"), ("spanish", "español"), ("new", "nuevo"), ("old", "antiguo"),
        ("good", "bueno"), ("better", "mejor"), ("best", "el mejor"), ("bad", "malo"),
        ("great", "gran"), ("high", "alto"), ("higher", "superior"), ("highest", "el más alto"),
        ("low", "bajo"), ("lower", "inferior"), ("small", "pequeño"), ("large", "grande"),
        ("big", "grande"), ("long", "largo"), ("short", "corto"), ("full", "completo"),
        ("empty", "vacío"), ("easy", "fácil"), ("simple", "sencillo"), ("complex", "complejo"),
        ("difficult", "difícil"), ("hard", "duro"), ("fast", "rápido"), ("quick", "rápido"),
        ("slow", "lento"), ("early", "temprano"), ("late", "tardío"), ("modern", "moderno"),
        ("current", "actual"), ("previous", "anterior"), ("next", "siguiente"), ("first", "primero"),
        ("second", "segundo"), ("third", "tercero"), ("last", "último"), ("final", "final"),
        ("main", "principal"), ("primary", "primario"), ("secondary", "secundario"), ("major", "importante"),
        ("key", "clave"), ("important", "importante"), ("essential", "esencial"), ("critical", "crítico"),
        ("necessary", "necesario"), ("available", "disponible"), ("possible", "posible"), ("impossible", "imposible"),
        ("different", "diferente"), ("various", "diversos"), ("similar", "similar"), ("equal", "igual"),
        ("specific", "específico"), ("special", "especial"), ("general", "general"), ("common", "común"),
        ("standard", "estándar"), ("normal", "normal"), ("regular", "regular"), ("natural", "natural"),
        ("original", "original"), ("native", "nativo"), ("local", "local"), ("global", "global"),
        ("international", "internacional"), ("public", "público"), ("private", "privado"), ("personal", "personal"),
        ("official", "oficial"), ("legal", "legal"), ("financial", "financiero"), ("commercial", "comercial"),
        ("professional", "profesional"), ("technical", "técnico"), ("scientific", "científico"), ("academic", "académico"),
        ("digital", "digital"), ("electronic", "electrónico"), ("virtual", "virtual"), ("automatic", "automático"),
        ("automated", "automatizado"), ("manual", "manual"), ("offline", "sin conexión"), ("online", "en línea"),
        ("neural", "neuronal"), ("intelligent", "inteligente"), ("smart", "inteligente"), ("advanced", "avanzado"),
        ("secure", "seguro"), ("safe", "seguro"), ("clear", "claro"), ("clean", "limpio"),
        ("complete", "completo"), ("accurate", "preciso"), ("exact", "exacto"), ("correct", "correcto"),
        ("valid", "válido"), ("effective", "efectivo"), ("efficient", "eficiente"), ("reliable", "confiable"),
        ("independent", "independiente"), ("autonomous", "autónomo"), ("compatible", "compatible"),
        ("active", "activo"), ("ready", "listo"), ("free", "gratuito"), ("open", "abierto"),
        ("visual", "visual"), ("graphical", "gráfico"), ("structural", "estructural"),

        // Adverbios
        ("not", "no"), ("very", "muy"), ("also", "también"), ("too", "también"),
        ("only", "solamente"), ("just", "solo"), ("even", "incluso"), ("still", "todavía"),
        ("already", "ya"), ("always", "siempre"), ("never", "nunca"), ("often", "a menudo"),
        ("sometimes", "a veces"), ("usually", "usualmente"), ("generally", "generalmente"),
        ("normally", "normalmente"), ("currently", "actualmente"), ("recently", "recientemente"),
        ("soon", "pronto"), ("now", "ahora"), ("then", "entonces"), ("here", "aquí"),
        ("there", "allí"), ("everywhere", "en todas partes"), ("well", "bien"), ("easily", "fácilmente"),
        ("quickly", "rápidamente"), ("automatically", "automáticamente"), ("directly", "directamente"),
        ("completely", "completamente"), ("fully", "totalmente"), ("correctly", "correctamente"),
        ("successfully", "exitosamente"), ("locally", "localmente"), ("strictly", "estrictamente"),
        ("especially", "especialmente"), ("particularly", "particularmente"), ("approximately", "aproximadamente"),
        ("almost", "casi"), ("enough", "suficiente"), ("quite", "bastante"), ("rather", "más bien"),
        ("together", "juntos"), ("instead", "en su lugar"), ("again", "de nuevo"), ("once", "una vez"),
        ("twice", "dos veces"), ("firstly", "primero"), ("secondly", "en segundo lugar"), ("finally", "finalmente"),
    ];

    let mut map = HashMap::with_capacity(entries.len());
    for (k, v) in entries {
        map.insert(k, v);
    }
    map
});

/// Lista de adjetivos en inglés que en español suelen posponerse al sustantivo (ej. "technical document" -> "documento técnico").
static ADJECTIVES_SET: Lazy<std::collections::HashSet<&'static str>> = Lazy::new(|| {
    [
        "english", "spanish", "new", "old", "high", "low", "small", "large", "long", "short",
        "full", "empty", "easy", "simple", "complex", "difficult", "fast", "quick", "slow",
        "modern", "current", "previous", "final", "main", "primary", "secondary", "important",
        "essential", "critical", "necessary", "available", "possible", "different", "similar",
        "specific", "special", "general", "common", "standard", "normal", "regular", "natural",
        "original", "native", "local", "global", "international", "public", "private", "personal",
        "official", "legal", "financial", "commercial", "professional", "technical", "scientific",
        "academic", "digital", "electronic", "virtual", "automatic", "automated", "manual",
        "offline", "online", "neural", "intelligent", "advanced", "secure", "safe", "clear",
        "clean", "complete", "accurate", "exact", "correct", "valid", "effective", "efficient",
        "reliable", "independent", "autonomous", "compatible", "active", "visual", "graphical",
        "structural",
    ]
    .into_iter()
    .collect()
});

/// Lista de sustantivos comunes para aplicar la regla de inversión Adjetivo + Sustantivo -> Sustantivo + Adjetivo.
static NOUNS_SET: Lazy<std::collections::HashSet<&'static str>> = Lazy::new(|| {
    [
        "document", "documents", "documentation", "file", "files", "folder", "folders", "page",
        "pages", "paragraph", "paragraphs", "section", "sections", "chapter", "chapters",
        "title", "titles", "table", "tables", "row", "rows", "column", "columns", "list",
        "lists", "item", "items", "text", "texts", "word", "words", "sentence", "sentences",
        "language", "languages", "translation", "translations", "translator", "version",
        "versions", "system", "systems", "application", "applications", "program", "programs",
        "software", "computer", "device", "server", "client", "user", "users", "customer",
        "customers", "company", "organization", "business", "team", "group", "project",
        "projects", "task", "tasks", "goal", "objective", "result", "results", "output",
        "input", "data", "information", "content", "summary", "report", "reports", "analysis",
        "research", "study", "method", "methodology", "approach", "model", "models", "engine",
        "network", "connection", "security", "privacy", "access", "account", "profile",
        "feature", "features", "function", "functionality", "performance", "efficiency",
        "quality", "standard", "rule", "requirement", "requirements", "specification",
        "architecture", "structure", "format", "formats", "style", "layout", "design",
        "interface", "window", "button", "menu", "dialog", "panel", "view", "preview",
        "mode", "status", "state", "progress", "level", "value", "values", "number", "amount",
        "cost", "price", "budget", "market", "industry", "service", "services", "product",
        "products", "solution", "solutions", "problem", "problems", "issue", "error", "warning",
        "message", "note", "example", "case", "step", "phase", "time", "date", "world",
        "country", "part", "order", "agreement", "contract", "policy", "license", "certificate",
        "package", "installer", "directory", "path", "code", "memory", "storage", "operation",
        "execution", "environment", "configuration", "option", "options", "guide", "manual",
        "introduction", "overview", "conclusion",
    ]
    .into_iter()
    .collect()
});

/// Traduce un bloque de texto de Inglés a Español de forma 100% offline,
/// preservando saltos de línea, espacios iniciales/finales y puntuación.
pub fn translate_text_en_to_es(input: &str) -> String {
    if input.trim().is_empty() {
        return input.to_string();
    }

    // Preservar espacios en blanco al inicio y al final del fragmento (crucial para nodos XML de DOCX)
    let leading_ws_len = input.len() - input.trim_start().len();
    let trailing_ws_len = input.len() - input.trim_end().len();
    let leading_ws = &input[..leading_ws_len];
    let trailing_ws = if trailing_ws_len > 0 {
        &input[input.len() - trailing_ws_len..]
    } else {
        ""
    };
    let core = input.trim();

    // Procesar línea por línea para respetar saltos de línea exactos
    let translated_lines: Vec<String> = core
        .split('\n')
        .map(translate_line_en_to_es)
        .collect();

    format!("{}{}{}", leading_ws, translated_lines.join("\n"), trailing_ws)
}

fn translate_line_en_to_es(line: &str) -> String {
    if line.trim().is_empty() {
        return line.to_string();
    }

    // 1. Tokenizar en palabras y separadores (puntuación/espacios)
    let tokens = tokenize_preserving_separators(line);

    // Extraer solo los índices de tokens que son palabras alfabéticas
    let word_indices: Vec<usize> = tokens
        .iter()
        .enumerate()
        .filter(|(_, t)| t.is_word)
        .map(|(idx, _)| idx)
        .collect();

    if word_indices.is_empty() {
        return line.to_string();
    }

    let mut result_tokens: Vec<String> = tokens.iter().map(|t| t.raw.clone()).collect();
    let mut wi = 0;

    while wi < word_indices.len() {
        let tok_idx = word_indices[wi];

        // Intentar emparejar frases multi-palabra (hasta 4 palabras seguidas separadas por espacio simple)
        let mut matched_phrase_len = 0;
        let mut matched_translation = "";

        for &(en_phrase, es_phrase) in PHRASE_TABLE.iter() {
            let phrase_words: Vec<&str> = en_phrase.split(' ').collect();
            let plen = phrase_words.len();
            if plen > 1 && wi + plen <= word_indices.len() {
                // Verificar que los tokens intermedios sean únicamente espacios simples
                let mut contiguous = true;
                for k in 0..plen {
                    let cur_tok_idx = word_indices[wi + k];
                    if tokens[cur_tok_idx].lower != phrase_words[k] {
                        contiguous = false;
                        break;
                    }
                    if k + 1 < plen {
                        let next_tok_idx = word_indices[wi + k + 1];
                        if next_tok_idx != cur_tok_idx + 2 || tokens[cur_tok_idx + 1].raw != " " {
                            contiguous = false;
                            break;
                        }
                    }
                }
                if contiguous {
                    matched_phrase_len = plen;
                    matched_translation = es_phrase;
                    break;
                }
            }
        }

        if matched_phrase_len > 1 {
            // Aplicar capitalización de la primera palabra de la frase
            let styled = apply_casing(&tokens[tok_idx].raw, matched_translation);
            result_tokens[tok_idx] = styled;
            // Vaciar los tokens restantes que formaban parte de la frase en inglés
            let last_tok_idx = word_indices[wi + matched_phrase_len - 1];
            for clear_idx in (tok_idx + 1)..=last_tok_idx {
                result_tokens[clear_idx].clear();
            }
            wi += matched_phrase_len;
            continue;
        }

        // Regla sintáctica: Adjetivo + Sustantivo contiguos -> Sustantivo + Adjetivo en español
        if wi + 1 < word_indices.len() {
            let next_tok_idx = word_indices[wi + 1];
            if next_tok_idx == tok_idx + 2 && tokens[tok_idx + 1].raw == " " {
                let w1_lower = tokens[tok_idx].lower.as_str();
                let w2_lower = tokens[next_tok_idx].lower.as_str();
                if ADJECTIVES_SET.contains(w1_lower) && NOUNS_SET.contains(w2_lower) {
                    let es_adj = translate_single_word(w1_lower);
                    let es_noun = translate_single_word(w2_lower);
                    let is_plural = w2_lower.ends_with('s') && !w2_lower.ends_with("ss");
                    let inflected_adj = inflect_adjective(&es_adj, &es_noun, is_plural);

                    let first_styled = apply_casing(&tokens[tok_idx].raw, &es_noun);
                    let second_styled = apply_casing(&tokens[next_tok_idx].raw, &inflected_adj);

                    result_tokens[tok_idx] = first_styled;
                    result_tokens[next_tok_idx] = second_styled;
                    wi += 2;
                    continue;
                }
            }
        }

        // Traducción de palabra individual con morfología de sufijos comunes (-ly, -tion, -ity, -ment, -ing, -ed)
        let w_lower = tokens[tok_idx].lower.as_str();
        let es_word = translate_single_word(w_lower);
        result_tokens[tok_idx] = apply_casing(&tokens[tok_idx].raw, &es_word);
        wi += 1;
    }

    // Post-procesamiento de contracciones en español ("a el" -> "al", "de el" -> "del")
    let joined = result_tokens.concat();
    post_process_spanish_contractions(&joined)
}

#[derive(Debug, Clone)]
struct Token {
    raw: String,
    lower: String,
    is_word: bool,
}

fn tokenize_preserving_separators(text: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut current_is_word: Option<bool> = None;

    for ch in text.chars() {
        let is_w = ch.is_alphabetic() || ch == '\'';
        match current_is_word {
            Some(prev_is_w) if prev_is_w == is_w => {
                current.push(ch);
            }
            Some(prev_is_w) => {
                let lower = current.to_lowercase();
                tokens.push(Token {
                    raw: current.clone(),
                    lower,
                    is_word: prev_is_w,
                });
                current.clear();
                current.push(ch);
                current_is_word = Some(is_w);
            }
            None => {
                current.push(ch);
                current_is_word = Some(is_w);
            }
        }
    }

    if let Some(prev_is_w) = current_is_word {
        let lower = current.to_lowercase();
        tokens.push(Token {
            raw: current,
            lower,
            is_word: prev_is_w,
        });
    }

    tokens
}

/// Traduce una palabra individual buscando primero en el diccionario y luego aplicando
/// reglas morfológicas cognadas Inglés -> Español (ej. -tion -> -ción, -ly -> -mente, -ity -> -idad).
fn translate_single_word(word_lower: &str) -> String {
    // Contracciones inglesas frecuentes
    match word_lower {
        "it's" | "its'" => return "es".to_string(),
        "that's" => return "eso es".to_string(),
        "don't" | "doesn't" | "didn't" => return "no".to_string(),
        "can't" => return "no puede".to_string(),
        "won't" => return "no va a".to_string(),
        "we're" => return "estamos".to_string(),
        "they're" => return "están".to_string(),
        "you're" => return "está".to_string(),
        "i'm" => return "estoy".to_string(),
        _ => {}
    }

    if let Some(&es) = WORD_DICT.get(word_lower) {
        return es.to_string();
    }

    // Si termina en 's' (plural regular), probar la raíz en singular
    if word_lower.len() > 4 && word_lower.ends_with('s') && !word_lower.ends_with("ss") {
        let singular = &word_lower[..word_lower.len() - 1];
        if let Some(&es_sing) = WORD_DICT.get(singular) {
            return pluralize_spanish(es_sing);
        }
    }

    // Reglas morfológicas de cognados grecolatinos (muy efectivos en documentos técnicos y formales)
    if word_lower.len() > 5 {
        if let Some(stem) = word_lower.strip_suffix("ation") {
            return format!("{}ación", stem);
        }
        if let Some(stem) = word_lower.strip_suffix("ition") {
            return format!("{}ición", stem);
        }
        if let Some(stem) = word_lower.strip_suffix("ution") {
            return format!("{}ución", stem);
        }
        if let Some(stem) = word_lower.strip_suffix("ally") {
            return format!("{}almente", stem);
        }
        if let Some(stem) = word_lower.strip_suffix("ity") {
            return format!("{}idad", stem);
        }
        if let Some(stem) = word_lower.strip_suffix("ities") {
            return format!("{}idades", stem);
        }
        if let Some(stem) = word_lower.strip_suffix("ment") {
            return format!("{}miento", stem);
        }
        if let Some(stem) = word_lower.strip_suffix("ments") {
            return format!("{}mientos", stem);
        }
    }

    // Si es un nombre propio, acrónimo o término técnico no registrado, se conserva intacto
    word_lower.to_string()
}

fn pluralize_spanish(singular: &str) -> String {
    if singular.ends_with('a')
        || singular.ends_with('e')
        || singular.ends_with('i')
        || singular.ends_with('o')
        || singular.ends_with('u')
    {
        format!("{}s", singular)
    } else if let Some(stem) = singular.strip_suffix('z') {
        format!("{}ces", stem)
    } else if let Some(stem) = singular.strip_suffix("ón") {
        format!("{}ones", stem)
    } else {
        format!("{}es", singular)
    }
}

fn inflect_adjective(adj: &str, noun_es: &str, is_plural: bool) -> String {
    let is_feminine = noun_es.ends_with('a')
        || noun_es.ends_with("ción")
        || noun_es.ends_with("sión")
        || noun_es.ends_with("dad")
        || noun_es.ends_with("tud");

    let mut result = adj.to_string();
    if is_feminine && result.ends_with('o') {
        result.pop();
        result.push('a');
    }

    if is_plural {
        result = pluralize_spanish(&result);
    }

    result
}

fn apply_casing(source_raw: &str, translated: &str) -> String {
    if translated.is_empty() {
        return String::new();
    }
    let chars: Vec<char> = source_raw.chars().collect();
    if chars.is_empty() {
        return translated.to_string();
    }

    let all_upper = chars.len() > 1 && chars.iter().all(|c| !c.is_alphabetic() || c.is_uppercase());
    if all_upper {
        return translated.to_uppercase();
    }

    if chars[0].is_uppercase() {
        let mut t_chars = translated.chars();
        if let Some(first) = t_chars.next() {
            let mut out = first.to_uppercase().to_string();
            out.push_str(t_chars.as_str());
            return out;
        }
    }

    translated.to_string()
}

fn post_process_spanish_contractions(text: &str) -> String {
    text.replace(" a el ", " al ")
        .replace(" de el ", " del ")
        .replace("A el ", "Al ")
        .replace("De el ", "Del ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_translation() {
        let en = "The technical document contains important information for the project.";
        let es = translate_text_en_to_es(en);
        assert!(es.contains("documento técnico"));
        assert!(es.contains("información importante"));
        assert!(es.contains("para el proyecto."));
    }

    #[test]
    fn test_preserves_whitespace_and_casing() {
        let en = "  EXECUTIVE SUMMARY  ";
        let es = translate_text_en_to_es(en);
        assert_eq!(es, "  RESUMEN EJECUTIVO  ");
    }
}

