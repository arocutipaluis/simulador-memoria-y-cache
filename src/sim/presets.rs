use crate::sim::engine::CacheEngine;
use crate::sim::types::{MappingPolicy, MemInstruction, MemOp, PresetKind, ReplacementPolicy, WritePolicy};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LearnLesson {
    pub title: &'static str,
    pub description: &'static str,
    pub action_label: &'static str,
    pub demo_addr: u8,
    pub demo_op: MemOp,
}

pub fn learn_lessons() -> Vec<LearnLesson> {
    vec![
        LearnLesson {
            title: "1. Piramide de la Jerarquia de Memoria",
            description: "La CPU opera en sub-nanosegundos mientras la RAM tarda ~60 ns. La jerarquia (Registros -> L1 -> L2 -> L3 -> RAM -> SSD) reduce cuellos de botella acercando los datos frecuentes.",
            action_label: "Leer 0x00 (Fallo inicial)",
            demo_addr: 0x00,
            demo_op: MemOp::Read,
        },
        LearnLesson {
            title: "2. Cache Miss vs Cache Hit (Localidad Temporal)",
            description: "Tras el fallo compulsivo en 0x00, el bloque ya reside en L1. Re-acceder a 0x00 produce un Cache Hit en 1.5 ns totales, explotando la localidad temporal.",
            action_label: "Re-leer 0x00 (Hit L1)",
            demo_addr: 0x00,
            demo_op: MemOp::Read,
        },
        LearnLesson {
            title: "3. Localidad Espacial y Decodificador de Bits",
            description: "Cada linea carga un bloque de varias palabras contiguas. Con bloque de 2 o 4 palabras, leer 0x01 inmediatamente despues de 0x00 acierta en L1 sin ir a RAM.",
            action_label: "Leer vecina 0x01 (Hit Espacial)",
            demo_addr: 0x01,
            demo_op: MemOp::Read,
        },
        LearnLesson {
            title: "4. Mapeo, Colisiones y Write-Back",
            description: "En Mapeo Directo, dos bloques distantes un multiplo del tamano de cache (ej. 0x00 y 0x10 con 8 bloques x 2 palabras) compiten por el mismo Set. Prueba cambiar a Asociativo 2 vias.",
            action_label: "Leer 0x10 (Verificar Set)",
            demo_addr: 0x10,
            demo_op: MemOp::Read,
        },
    ]
}

pub struct ChallengeDef {
    pub id: usize,
    pub title: &'static str,
    pub difficulty: &'static str,
    pub reward_xp: u32,
    pub description: &'static str,
    pub hint: &'static str,
}

pub fn challenges() -> Vec<ChallengeDef> {
    vec![
        ChallengeDef {
            id: 1,
            title: "Desafio 1: Calentamiento de Cache",
            difficulty: "Basico",
            reward_xp: 150,
            description: "Ejecuta al menos 6 accesos a memoria y alcanza un Hit Rate >= 50%.",
            hint: "Usa los botones de Localidad Temporal o Espacial para reutilizar bloques cargados.",
        },
        ChallengeDef {
            id: 2,
            title: "Desafio 2: Optimizacion de AMAT",
            difficulty: "Intermedio",
            reward_xp: 250,
            description: "Con L1 de 4 bloques y bloque de 2 palabras, realiza >= 8 accesos manteniendo AMAT < 20.0 ns.",
            hint: "Configura 4 bloques y tamano 2 palabras; trabaja sobre un rango compacto de 4 bloques.",
        },
        ChallengeDef {
            id: 3,
            title: "Desafio 3: Duelo LRU + Write-Back",
            difficulty: "Intermedio",
            reward_xp: 300,
            description: "Activa reemplazo LRU y escritura Write-Back, ejecuta >= 8 operaciones (>= 2 escrituras STORE) y logra Hit Rate >= 60%.",
            hint: "Realiza escrituras repetidas sobre direcciones recientes para ver el Dirty Bit en accion sin ir a RAM en cada acierto.",
        },
        ChallengeDef {
            id: 4,
            title: "Desafio 4: Mitigar Cache Thrashing",
            difficulty: "Avanzado",
            reward_xp: 350,
            description: "Usa Mapeo Asociativo por Conjuntos (2 o 4 vias) con 8 bloques x 4 palabras, ejecuta el preset Thrashing (>= 12 accesos) y supera 65% de Hit Rate.",
            hint: "En Mapeo Directo el preset Thrashing da 0%. Cambia a Asociativo 2 vias y ejecuta el preset Conflicto / Thrashing.",
        },
    ]
}

pub fn check_challenge_passed(id: usize, engine: &CacheEngine) -> bool {
    let s = &engine.stats;
    let c = &engine.config;
    match id {
        1 => s.accesses >= 6 && s.hit_rate() >= 50.0,
        2 => c.num_blocks == 4 && c.block_size == 2 && s.accesses >= 8 && s.amat_ns() < 20.0,
        3 => {
            c.replacement == ReplacementPolicy::Lru
                && c.write_policy == WritePolicy::WriteBack
                && s.accesses >= 8
                && s.writes_count >= 2
                && s.hit_rate() >= 60.0
        }
        4 => {
            matches!(c.mapping, MappingPolicy::SetAssociative(_))
                && c.num_blocks == 8
                && c.block_size == 4
                && s.accesses >= 12
                && s.l1_hit_rate() >= 65.0
        }
        _ => false,
    }
}

pub fn preset_sequence(kind: PresetKind) -> Vec<MemInstruction> {
    match kind {
        PresetKind::Sequential => (0..16u8)
            .map(|addr| MemInstruction {
                op: MemOp::Read,
                addr,
            })
            .collect(),
        PresetKind::Temporal => [0x04, 0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 0x04]
            .into_iter()
            .map(|addr| MemInstruction {
                op: MemOp::Read,
                addr,
            })
            .collect(),
        PresetKind::Spatial => [0x00, 0x01, 0x02, 0x03, 0x10, 0x11, 0x12, 0x13]
            .into_iter()
            .map(|addr| MemInstruction {
                op: MemOp::Read,
                addr,
            })
            .collect(),
        PresetKind::Stride4 => [0x00, 0x04, 0x08, 0x0C, 0x10, 0x14, 0x18, 0x1C]
            .into_iter()
            .map(|addr| MemInstruction {
                op: MemOp::Read,
                addr,
            })
            .collect(),
        // Con 8 bloques x 4 palabras (32 palabras = 0x20), 0x00..0x07 y 0x20..0x27 colisionan en Set 0 y Set 1 en Directo
        PresetKind::Thrashing => [
            0x00, 0x20, 0x01, 0x21, 0x02, 0x22, 0x03, 0x23,
            0x04, 0x24, 0x05, 0x25, 0x06, 0x26, 0x07, 0x27,
        ]
        .into_iter()
        .map(|addr| MemInstruction {
            op: MemOp::Read,
            addr,
        })
        .collect(),
        PresetKind::WriteHeavy => vec![
            MemInstruction { op: MemOp::Write("W_A1".into()), addr: 0x00 },
            MemInstruction { op: MemOp::Write("W_A2".into()), addr: 0x00 },
            MemInstruction { op: MemOp::Write("W_A3".into()), addr: 0x01 },
            MemInstruction { op: MemOp::Read, addr: 0x00 },
            MemInstruction { op: MemOp::Write("W_B1".into()), addr: 0x20 },
            MemInstruction { op: MemOp::Read, addr: 0x00 },
            MemInstruction { op: MemOp::Write("W_B2".into()), addr: 0x20 },
            MemInstruction { op: MemOp::Read, addr: 0x21 },
        ],
    }
}
