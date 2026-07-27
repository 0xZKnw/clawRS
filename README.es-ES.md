<p align="center">
  <h1 align="center">ClawRS</h1>
  <p align="center">
    <strong>Tu IA privada, 100% local.</strong><br>
    Una aplicación de escritorio premium para ejecutar LLMs localmente con un sistema de herramientas agénticas, construida en Rust.
  </p>
  <p align="center">
    <img src="https://img.shields.io/badge/rust-2021-orange?style=flat-square&logo=rust" alt="Rust 2021">
    <img src="https://img.shields.io/badge/version-0.2.0-blue?style=flat-square" alt="Version 0.2.0">
    <img src="https://img.shields.io/badge/license-MIT-green?style=flat-square" alt="License MIT">
    <img src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey?style=flat-square" alt="Platform">
  </p>
</p>

---

## ¿Qué es ClawRS?

ClawRS es una aplicación de escritorio nativa que te permite ejecutar modelos de lenguaje extensos **completamente en tu máquina** — sin nube, sin claves de API, sin que tus datos salgan de tu dispositivo. Combina una interfaz moderna de glassmorphism con un potente sistema agéntico capaz de leer/escribir archivos, ejecutar comandos, buscar en la web y mucho más.

Piensa en ello como tu propio Claude o ChatGPT privado, ejecutándose offline con acceso total a tu ordenador.

### Características Principales

- **100% Local y Privado** — Toda la inferencia se ejecuta en tu hardware vía `llama.cpp`. Tus datos nunca abandonan tu máquina.
- **Sistema de Herramientas Agénticas** — Más de 30 herramientas integradas: operaciones de archivos, ejecución de shell, git, búsqueda web, búsqueda de código y más.
- **Soporte para Modelos GGUF** — Carga cualquier modelo `.gguf`. Descárgalos directamente desde HuggingFace dentro de la aplicación.
- **Aceleración por GPU** — CUDA (NVIDIA), Vulkan (NVIDIA/AMD/Intel), Metal (Apple Silicon).
- **UI Premium** — Diseño orgánico cálido con glassmorphism, temas claro/oscuro y animaciones fluidas.
- **Bilingüe** — Interfaz completa en francés e inglés.
- **Sistema de Permisos** — Permisos granulares para herramientas con lista de permitidos y modo de auto-aprobación.
- **Protocolo MCP** — Conéctate a servidores externos del Model Context Protocol para capacidades extendidas.
- **Historial de Conversaciones** — Historial de chat persistente guardado localmente.
- **Consciente de la VRAM** — Limita automáticamente el tamaño del contexto basándose en tu VRAM disponible.

---

## Capturas de Pantalla

![ClawRS Interface](assets/screen-readme.png)

---

## Instalación

### Descargar Versión Pre-construida

Descarga la última versión desde la [página de Releases](https://github.com/0xZKnw/clawRS/releases).

### Construir desde el Código Fuente

#### Prerrequisitos

- **Rust** (1.75+ recomendado) — [rustup.rs](https://rustup.rs)
- **CMake** — Requerido para construir `llama.cpp`
- **Compilador C++** — MSVC en Windows, GCC/Clang en Linux/macOS

#### Windows

**Requisitos:**
- Visual Studio Build Tools con carga de trabajo de C++
- CUDA Toolkit (para CUDA) y/o Vulkan SDK (para Vulkan)

**Comandos de construcción:**

```powershell
# CPU + Vulkan (detección automática)
cargo build --release

# Solo CUDA (GPU NVIDIA)
cargo build --release --features cuda

# Vulkan + CUDA (GPU NVIDIA + fallback)
cargo build --release --features "cuda,vulkan"
```

O usa los scripts incluidos:
```powershell
# Construcción CUDA
.\build_cuda.bat

# Construcción estándar (CPU + Vulkan)
.\build.bat
```

#### macOS

**Requisitos:**
- Xcode Command Line Tools
- (Metal es automático - no requiere instalación adicional)

```bash
# Instalar prerrequisitos
xcode-select --install

# Construir (Metal es automático en macOS)
cargo build --release

# O usa el script de construcción
chmod +x build_macos.sh
./build_macos.sh
```

**Nota:** La aceleración por GPU Metal está habilitada por defecto en Macs con Apple Silicon. No requiere características especiales.

#### Linux

```bash
# Instalar dependencias (Ubuntu/Debian)
sudo apt install cmake build-essential libssl-dev pkg-config

# Construir
cargo build --release

# Para soporte de Vulkan (opcional)
sudo apt install libvulkan-dev
```

---

## Inicio Rápido

1. **Construye el proyecto:**
   ```bash
   cargo build --release
   ```

2. **Ejecuta la aplicación:**
   ```bash
   cargo run --release
   ```

3. **Carga un modelo:**
   - Coloca cualquier archivo de modelo `.gguf` en el directorio de modelos, o
   - Usa el descargador de HuggingFace integrado (barra lateral > "Download from HuggingFace")
   - Selecciona y carga un modelo desde el menú desplegable del encabezado o la barra lateral

4. **¡Empieza a chatear!**
   La IA puede leer tus archivos, ejecutar comandos, buscar en la web y más, todo localmente.

---

## Limitaciones Importantes

ClawRS se ejecuta totalmente **offline** usando modelos locales. Esto tiene implicaciones importantes que debes comprender:

### Tamaño del Modelo y Requisitos de Hardware

- **VRAM/ RAM**: La mayoría de los modelos locales requieren entre 4-16GB de VRAM (GPU) o RAM (CPU). Los modelos más grandes necesitan más recursos.
- **Modelos Recomendados**: Los modelos de 4-8GB funcionan bien en la mayoría del hardware comercial. Los modelos de más de 12B parámetros requieren GPUs de gama alta.
- **Descarga de Modelos**: Obtén archivos `.gguf` desde HuggingFace (recomendados: Ministral 3 8b, Falcon H1R 7b...)

### Límites de la Ventana de Contexto

- **Contexto Limitado**: Los modelos locales suelen soportar entre 4K-32K tokens de contexto (frente a los 100K+ de los modelos en la nube).
- **Uso de Memoria**: Cada 1K tokens de contexto utiliza ~1-2MB de VRAM/RAM.
- **Consciente de la VRAM**: ClawRS ajusta automáticamente el tamaño del contexto basándose en tu VRAM disponible.

### Diferencias de Capacidad

- **Modelos más pequeños = Menos conocimiento**: Los modelos locales tienen menos conocimiento general que GPT-4/Claude.
- **Sin acceso a Internet**: No puede navegar por la web en vivo (aunque las herramientas de búsqueda web pueden ayudar).
- **Razonamiento limitado**: El razonamiento complejo de varios pasos puede ser menos fiable que en los modelos en la nube.
- **Sin seguridad ajustada**: Ocasionalmente puede generar respuestas inesperadas.

### Expectativas de Rendimiento

- **Velocidad**: Depende de tu hardware (se prefiere GPU). Lo habitual es entre 10-50 tokens/seg.
- **Calidad vs Nube**: Un modelo local de 7B $\approx$ nivel GPT-3.5/4 (básico). Un local de 70B $\approx$ nivel GPT-4o (pero más lento).
- **Idoneidad de tareas**: Ideal para ayuda de programación, operaciones de archivos y tareas locales. No es ideal para investigación profunda.

### Consejos para la mejor experiencia

1. Usa **modelos cuantizados** (Q4_K_M, Q5_K_S, Q8_0) para obtener la mejor relación velocidad/calidad.
2. Asegúrate de tener **VRAM suficiente** antes de cargar modelos grandes.
3. Mantén las conversaciones **enfocadas** para evitar alcanzar los límites de contexto.
4. Usa **ChatGPT, Gemini o Claude** para razonamientos complejos y ClawRS para tareas de ejecución.

---

## Arquitectura

```
src/
├── main.rs              # Punto de entrada, configuración de ventana
├── app.rs               # Estado de la aplicación (AppState)
├── agent/               # Sistema de IA agéntica
│   ├── mod.rs           # Configuración del agente, registro de herramientas
│   ├── permissions.rs   # Niveles de permiso y flujo de aprobación
│   ├── prompts.rs       # Prompts de sistema dinámicos
│   ├── planning.rs      # Planificación de tareas (sistema TODO)
│   ├── runner.rs        # Extracción y formateo de llamadas a herramientas
│   ├── loop_runner.rs   # Bucle del agente (pensar → actuar → observar)
│   └── tools/           # Más de 30 herramientas
│       ├── filesystem.rs    # Lectura/escritura/edición/búsqueda de archivos
│       ├── shell.rs         # Ejecución de Bash/PowerShell
│       ├── git.rs           # Operaciones de Git
│       ├── web.rs           # Búsqueda/descarga web
│       ├── exa.rs           # Búsqueda web y de código de Exa
│       ├── dev.rs           # Diff, buscar-reemplazar, patch
│       ├── system.rs        # Lista de procesos, info del sistema, tree
│       └── mcp_client.rs    # Cliente del protocolo MCP
├── inference/           # Motor LLM (bindings de llama.cpp)
│   ├── engine.rs        # Carga del modelo, generación
│   └── streaming.rs     # Streaming token por token
├── storage/             # Capa de persistencia
│   ├── settings.rs      # Preferencias del usuario (JSON)
│   ├── conversations.rs # Historial de chat
│   ├── models.rs        # Escaneo de modelos GGUF
│   └── huggingface.rs   # Descarga de modelos
├── system/              # Detección de hardware
│   ├── gpu.rs           # Detección de GPU/VRAM
│   └── resources.rs     # Monitoreo de RAM/CPU
└── ui/                  # Componentes de UI de Dioxus
    ├── mod.rs           # Diseño, selector de modelo en encabezado
    ├── chat/            # Vista de chat, mensajes, entrada
    ├── sidebar/         # Barra lateral, selector de modelos, conversaciones
    ├── settings/        # Pestañas de ajustes (inferencia, hardware, herramientas, apariencia)
    └── components/      # Diálogos de permisos, spinners de carga
```

---

## Herramientas

ClawRS incluye **más de 30 herramientas integradas** que la IA puede utilizar:

| Categoría | Herramientas | Permiso |
|----------|-------|------------|
| **Lectura de Archivos** | `file_read`, `file_list`, `grep`, `glob`, `file_info`, `file_search` | Solo lectura |
| **Escritura de Archivos** | `file_write`, `file_edit`, `file_create`, `file_delete`, `file_move`, `file_copy`, `directory_create` | Escritura |
| **Shell** | `bash`, `bash_background`, `command` | Ejecutar |
| **Git** | `git_status`, `git_diff`, `git_log`, `git_commit`, `git_branch`, `git_stash` | Leer / Ejecutar |
| **Web** | `web_search`, `code_search`, `company_research`, `web_fetch`, `web_download` | Red |
| **Dev** | `diff`, `find_replace`, `patch`, `wc` | Leer / Escribir |
| **Sistema** | `process_list`, `environment`, `system_info`, `which`, `tree` | Solo lectura |

### Modos de Permisos

- **Aprobación manual** (por defecto) — Cada llamada a una herramienta muestra un diálogo de permiso.
- **Lista de permitidos** — Pre-aprueba herramientas específicas o grupos de herramientas en Ajustes > Herramientas.
- **Auto-aprobar todo** — Omite todos los diálogos de permiso (usar con precaución).

---

## Ajustes

Accesibles a través del icono de engranaje en la barra lateral:

| Pestaña | Opciones |
|-----|---------|
| **Inferencia** | Temperatura, Top-p, Top-k, Max tokens, Tamaño de contexto, Prompt de sistema |
| **Hardware** | Capas de GPU, Monitoreo de VRAM, Directorio de modelos, Carga automática de modelo |
| **Herramientas** | Modo auto-aprobar, Lista de permitidos (por grupo y por herramienta) |
| **Apariencia** | Tema Claro/Oscuro, Tamaño de fuente, Idioma (FR/EN) |

---

## Stack Tecnológico

- **Lenguaje:** Rust (edición 2021)
- **Framework de UI:** [Dioxus](https://dioxuslabs.com) (nativo de escritorio vía WebView)
- **Backend de LLM:** [llama.cpp](https://github.com/ggerganov/llama.cpp) vía el crate `llama-cpp-2`
- **Runtime Async:** Tokio
- **Estilos:** CSS personalizado con glassmorphism, variables CSS para temas

---

## Contribuir

¡Las contribuciones son bienvenidas! No dudes en abrir issues o pull requests.

1. Haz un fork del repositorio
2. Crea una rama para tu funcionalidad (`git checkout -b feature/amazing-feature`)
3. Haz commit de tus cambios (`git commit -m 'Add amazing feature'`)
4. Haz push de la rama (`git push origin feature/amazing-feature`)
5. Abre un Pull Request

---

## Licencia

Este proyecto está licenciado bajo la Licencia MIT — consulta el archivo [LICENSE](LICENSE) para más detalles.

---

<p align="center">
  <sub>Construido con Rust y amor. Tu IA, tus reglas, tu máquina.</sub>
</p>
