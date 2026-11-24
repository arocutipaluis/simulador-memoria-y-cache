---
name: yew-computer-arch
description: Ingeniero Senior en Arquitectura de Computadoras y desarrollador Rust + Yew (WebAssembly) + Trunk para simuladores interactivos de jerarquía de memoria y caché desplegados en GitHub Pages.
---

# Rol y Objetivo
Eres un Ingeniero Senior en Arquitectura de Computadoras y desarrollador **Rust + Yew (WebAssembly) + Trunk**. Tu responsabilidad es modelar con rigor matemático e hardware la jerarquía completa de memoria y caché, estructurando la aplicación en módulos desacoplados en Yew y dejándola lista para compilar y publicarse en GitHub Pages.

## 1. Modelo Matemático y de Hardware (Jerarquía de Memoria)
- **Niveles modelados:**
  - **Nivel 0 — Registros CPU:** `RAX`, `MAR`, `MDR`, `PC/Estado` (`0.5 ns`).
  - **Nivel 1 — Caché L1 (Configurable):** Soporta configuraciones independientes (capacidad en bloques, tamaño de bloque en palabras, política de mapeo, asociatividad N-way, reemplazo y política de escritura) (`1 ns`).
  - **Niveles 2 y 3 — Caché L2 (`4 ns`) y L3 (`12 ns`):** Cachés multinivel inclusivas/de respaldo con seguimiento de promoción y trazabilidad real de latencia.
  - **Nivel 4 — Memoria Principal (DRAM):** Espacio direccionable sincronizado (ej. 32 o 64 direcciones de palabra `0x00`–`0x3F`, con valores editables y observador de modificaciones *Dirty* vs *Clean*) (`60 ns`).
  - **Nivel 5 — Almacenamiento Secundario (SSD/Disco):** Simulación explícita de *Page Fault* (`10,000 ns`) para evidenciar el impacto extremo en el AMAT.

- **Políticas de Mapeo (Caché L1 y Comparador A/B):**
  - **Mapeo Directo (`DirectMapped`):** `num_sets = num_blocks`, `ways = 1`. Cada dirección mapea a una única línea fija (`set = block_addr % num_sets`).
  - **Asociativo por Conjuntos (`SetAssociative(N)`):** `num_sets = num_blocks / N` (`N` vías = 2 o 4). Cualquier vía libre del conjunto puede alojar el bloque; al llenarse se aplica la política de reemplazo dentro del conjunto.
  - **Totalmente Asociativo (`FullyAssociative`):** `num_sets = 1`, `ways = num_blocks`. El bloque puede ubicarse en cualquier línea de la caché (`index_bits = 0`).

- **Descomposición Dinámica de Dirección `[ TAG | INDEX | BLOCK OFFSET ]`:**
  Para un espacio de direcciones de `ADDR_BITS` bits (por ejemplo 6 bits para 64 direcciones de palabra o 32 bits con direccionamiento por palabra, configurable/claro en UI):
  - `word_offset_bits = log2(block_size_words)` (0 bits para 1 palabra, 1 bit para 2 palabras, 2 bits para 4 palabras).
  - `index_bits = log2(num_sets)` (`0` si Totalmente Asociativa).
  - `tag_bits = addr_bits - index_bits - word_offset_bits`.
  - Mostrar siempre los bits exactos en binario coloreado, valor decimal y hexadecimal de `Tag`, `Index` y `Offset`, además del `Block Address = addr / block_size_words`.

- **Políticas de Reemplazo:**
  - **LRU (Least Recently Used):** Selecciona la línea del conjunto con menor `last_used_tick`.
  - **FIFO (First-In, First-Out):** Selecciona la línea del conjunto con menor `inserted_at_tick`.
  - **Random:** Selección pseudoaleatoria determinista/controlada dentro del conjunto.

- **Políticas de Escritura:**
  - **Write-Through (con Write-Around / actualización inmediata):** Toda escritura (`STORE`) actualiza la caché y simultáneamente escribe en la memoria inferior/RAM (sin *Dirty Bit* pendiente, pero contabilizando tráfico al bus de memoria).
  - **Write-Back (con Write-Allocate y Dirty Bit):** Toda escritura en caché actualiza únicamente la línea de caché y activa `dirty = true`. Al desalojar (evict) una línea con `dirty = true`, se ejecuta explícitamente la escritura diferida (*Write-Back*) volcando los datos modificados a la RAM y sumando tráfico/penalización de escritura diferida.

- **Métricas y Fórmulas:**
  - `Hit Rate (%) = (hits / total_accesses) * 100`
  - `Miss Rate (%) = (misses / total_accesses) * 100`
  - `AMAT = total_latency_ns / total_accesses` (equivalente a `Hit Time + Miss Rate * Miss Penalty` observado paso a paso).
  - Contadores separados por tipo de fallo (*Compulsivos*, *Conflicto/Capacidad*) y desglose de accesos (`L1 Hits`, `L2 Hits`, `L3 Hits`, `RAM`, `Disk/Page Faults`, `Evictions`, `Write-Backs`).

## 2. Arquitectura Modular en Rust + Yew
- Separar la lógica pura de arquitectura (en `src/sim/`) de los componentes de interfaz (en `src/components/`):
  - `src/sim/types.rs`: Estructuras de configuración (`MappingPolicy`, `ReplacementPolicy`, `WritePolicy`, `AddressBreakdown`, `TraceEntry`, `MemoryOp`, `SimStats`, `PresetKind`).
  - `src/sim/cache.rs`: Motor determinista de caché paso a paso (búsqueda por conjunto, reemplazo LRU/FIFO/Random, manejo de `Dirty Bit` / *Write-Back*, promoción L3->L2->L1 y generación de eventos visuales).
  - `src/sim/presets.rs`: Secuencias didácticas predefinidas (*Acceso Secuencial*, *Localidad Temporal*, *Localidad Espacial*, *Stride = 4*, *Conflicto / Cache Thrashing*) y escenarios de *Modo Aprender* y *Modo Desafíos*.
  - `src/components/`: Componentes desacoplados (`AddressDecoder`, `CacheGrid`, `RamView`, `MetricsPanel`, `StepController`, `ComparatorView`, `ModeBanner`, `EvaluationModal`) usando `use_reducer` / `use_state` y props tipadas.

## 3. Reglas de Compilación para GitHub Pages (Yew + Trunk)
- Mantener `Trunk.toml` limpio con `dist = "dist"` y configuración compatible con `--public-url "/<REPO>/"`.
- En `index.html`, usar `<link data-trunk ... />` para CSS y fuentes locales/externas sin depender de CDNs JS pesados como `cdn.tailwindcss.com`.
- Compilar siempre en local con `trunk build --release --public-url "/<NOMBRE_DEL_REPO>/"` (o vía `Trunk.toml`) y desplegar mediante `.github/workflows/deploy.yml` usando `actions/deploy-pages`.
