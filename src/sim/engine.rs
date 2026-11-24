use std::collections::HashSet;

use crate::sim::types::{
    format_hex, AddressBreakdown, BusFlow, CacheConfig, CacheLine, HitLevel, LogEntry, MemOp,
    RamCell, ReplacementPolicy, SecondaryBlock, SimStats, SlotFlash, WritePolicy, LAT_DISK,
    LAT_L1, LAT_L2, LAT_L3, LAT_RAM, LAT_REG, RAM_SIZE,
};

const L2_CAPACITY: usize = 12;
const L3_CAPACITY: usize = 16;

#[derive(Clone, PartialEq, Debug)]
pub struct CacheEngine {
    pub config: CacheConfig,
    pub ram: Vec<RamCell>,
    pub sets: Vec<Vec<Option<CacheLine>>>,
    pub l2: Vec<SecondaryBlock>,
    pub l3: Vec<SecondaryBlock>,
    pub clock: u64,
    pub rng_state: u64,
    pub seen_blocks: HashSet<usize>,
    pub stats: SimStats,
    pub logs: Vec<LogEntry>,
    pub feedback: String,
    pub last_breakdown: AddressBreakdown,
    pub reg_rax: String,
    pub reg_mar: u8,
    pub reg_mdr: String,
    pub reg_status: String,
    pub bus_flow: BusFlow,
    pub last_hit_level: Option<HitLevel>,
    pub xp_earned: u32,
}

impl CacheEngine {
    pub fn new(config: CacheConfig) -> Self {
        let mut engine = Self {
            config,
            ram: Vec::new(),
            sets: Vec::new(),
            l2: Vec::new(),
            l3: Vec::new(),
            clock: 0,
            rng_state: 0x9E37_79B9_7F4A_7C15,
            seen_blocks: HashSet::new(),
            stats: SimStats::default(),
            logs: Vec::new(),
            feedback: "Sistema listo. Selecciona una direccion o carga una secuencia para iniciar.".to_string(),
            last_breakdown: config.decode(0),
            reg_rax: "---".to_string(),
            reg_mar: 0,
            reg_mdr: "---".to_string(),
            reg_status: "IDLE".to_string(),
            bus_flow: BusFlow::Idle,
            last_hit_level: None,
            xp_earned: 0,
        };
        engine.reset();
        engine
    }

    pub fn reconfigure(&mut self, config: CacheConfig) {
        self.config = config;
        self.reset();
        self.feedback = format!(
            "Hardware actualizado: {} | {} bloques x {} palabras ({} B) | {} | {}.",
            config.mapping.label(),
            config.num_blocks,
            config.block_size,
            config.capacity_bytes(),
            config.replacement.label(),
            config.write_policy.label()
        );
    }

    pub fn reset(&mut self) {
        self.ram = (0..RAM_SIZE as u8)
            .map(|addr| RamCell {
                addr,
                value: format!("D_{:02X}", addr),
                dirty_in_cache: false,
                cached_in_l1: false,
                last_accessed: false,
            })
            .collect();

        let num_sets = self.config.num_sets();
        let ways = self.config.effective_ways();
        self.sets = vec![vec![None; ways]; num_sets];
        self.l2.clear();
        self.l3.clear();
        self.clock = 0;
        self.rng_state = 0x9E37_79B9_7F4A_7C15;
        self.seen_blocks.clear();
        self.stats = SimStats::default();
        self.logs.clear();
        self.last_breakdown = self.config.decode(0);
        self.reg_rax = "---".to_string();
        self.reg_mar = 0;
        self.reg_mdr = "---".to_string();
        self.reg_status = "IDLE".to_string();
        self.bus_flow = BusFlow::Idle;
        self.last_hit_level = None;
        self.feedback = "Sistema reiniciado: caches L1, L2 y L3 vacias; RAM restaurada.".to_string();
    }

    pub fn clear_flashes(&mut self) {
        for set in &mut self.sets {
            for slot in set.iter_mut().flatten() {
                slot.flash = None;
            }
        }
        for blk in &mut self.l2 {
            blk.last_hit = false;
        }
        for blk in &mut self.l3 {
            blk.last_hit = false;
        }
        self.bus_flow = BusFlow::Idle;
    }

    fn next_rand(&mut self, bound: usize) -> usize {
        let mut x = self.rng_state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng_state = x;
        (x as usize) % bound.max(1)
    }

    fn build_words_from_ram(&self, block_addr: usize) -> Vec<(u8, String)> {
        let block_start = block_addr * self.config.block_size;
        (0..self.config.block_size)
            .filter_map(|offset| {
                let a = block_start + offset;
                if a < RAM_SIZE {
                    Some((a as u8, self.ram[a].value.clone()))
                } else {
                    None
                }
            })
            .collect()
    }

    fn sync_ram_flags(&mut self, active_addr: u8) {
        for cell in &mut self.ram {
            cell.cached_in_l1 = false;
            cell.dirty_in_cache = false;
            cell.last_accessed = cell.addr == active_addr;
        }
        for set in &self.sets {
            for line in set.iter().flatten() {
                for (a, _) in &line.words {
                    if let Some(cell) = self.ram.get_mut(*a as usize) {
                        cell.cached_in_l1 = true;
                        if line.dirty {
                            cell.dirty_in_cache = true;
                        }
                    }
                }
            }
        }
    }

    fn update_secondary_word(&mut self, block_addr: usize, addr: u8, val: &str) {
        for blk in &mut self.l2 {
            if blk.block_addr == block_addr {
                if let Some((_, w)) = blk.words.iter_mut().find(|(a, _)| *a == addr) {
                    *w = val.to_string();
                }
            }
        }
        for blk in &mut self.l3 {
            if blk.block_addr == block_addr {
                if let Some((_, w)) = blk.words.iter_mut().find(|(a, _)| *a == addr) {
                    *w = val.to_string();
                }
            }
        }
    }

    fn insert_secondary(cache: &mut Vec<SecondaryBlock>, cap: usize, new_blk: SecondaryBlock, policy: ReplacementPolicy) {
        if let Some(pos) = cache.iter().position(|b| b.block_addr == new_blk.block_addr) {
            cache[pos] = new_blk;
            return;
        }
        if cache.len() < cap {
            cache.push(new_blk);
            return;
        }
        let victim_idx = match policy {
            ReplacementPolicy::Fifo => cache
                .iter()
                .enumerate()
                .min_by_key(|(_, b)| b.inserted_at)
                .map(|(i, _)| i)
                .unwrap_or(0),
            ReplacementPolicy::Lru | ReplacementPolicy::Random => cache
                .iter()
                .enumerate()
                .min_by_key(|(_, b)| b.last_used)
                .map(|(i, _)| i)
                .unwrap_or(0),
        };
        cache[victim_idx] = new_blk;
    }

    fn select_victim_way(&mut self, set_idx: usize) -> usize {
        let ways = self.sets[set_idx].len();
        if ways <= 1 {
            return 0;
        }
        match self.config.replacement {
            ReplacementPolicy::Fifo => self.sets[set_idx]
                .iter()
                .enumerate()
                .filter_map(|(i, slot)| slot.as_ref().map(|l| (i, l.inserted_at)))
                .min_by_key(|(_, ts)| *ts)
                .map(|(i, _)| i)
                .unwrap_or(0),
            ReplacementPolicy::Lru => self.sets[set_idx]
                .iter()
                .enumerate()
                .filter_map(|(i, slot)| slot.as_ref().map(|l| (i, l.last_used)))
                .min_by_key(|(_, ts)| *ts)
                .map(|(i, _)| i)
                .unwrap_or(0),
            ReplacementPolicy::Random => self.next_rand(ways),
        }
    }

    pub fn execute(&mut self, op: MemOp, raw_addr: u8) {
        let addr = raw_addr & ((RAM_SIZE - 1) as u8);
        self.clear_flashes();

        self.clock += 1;
        self.stats.accesses += 1;
        if matches!(op, MemOp::Write(_)) {
            self.stats.writes_count += 1;
        }

        let decoded = self.config.decode(addr);
        self.last_breakdown = decoded.clone();
        self.reg_mar = addr;

        let set_idx = decoded.index_val;
        let tag = decoded.tag_val;
        let block_addr = decoded.block_addr;

        // 1. Buscar en L1 dentro del Set asignado
        let found_l1_way = self.sets[set_idx]
            .iter()
            .position(|slot| matches!(slot, Some(line) if line.valid && line.tag == tag));

        if let Some(way_idx) = found_l1_way {
            self.stats.hits += 1;
            self.stats.l1_hits += 1;
            self.xp_earned += 25;
            self.last_hit_level = Some(HitLevel::L1);

            let mut latency = LAT_REG + LAT_L1;
            let mut did_wb_msg = "";
            let mut val_result = String::new();

            if let Some(line) = self.sets[set_idx][way_idx].as_mut() {
                line.last_used = self.clock;
                match &op {
                    MemOp::Read => {
                        line.flash = Some(SlotFlash::Hit);
                        val_result = line
                            .words
                            .iter()
                            .find(|(a, _)| *a == addr)
                            .map(|(_, v)| v.clone())
                            .unwrap_or_else(|| self.ram[addr as usize].value.clone());
                        self.bus_flow = BusFlow::UpHit;
                    }
                    MemOp::Write(new_val) => {
                        if let Some((_, w)) = line.words.iter_mut().find(|(a, _)| *a == addr) {
                            *w = new_val.clone();
                        }
                        val_result = new_val.clone();
                        match self.config.write_policy {
                            WritePolicy::WriteThrough => {
                                line.dirty = false;
                                line.flash = Some(SlotFlash::Hit);
                                self.ram[addr as usize].value = new_val.clone();
                                self.update_secondary_word(block_addr, addr, new_val);
                                latency += LAT_RAM;
                                self.bus_flow = BusFlow::WriteBackToRam;
                                did_wb_msg = " (Write-Through a RAM +60 ns)";
                            }
                            WritePolicy::WriteBack => {
                                line.dirty = true;
                                line.flash = Some(SlotFlash::WriteBack);
                                self.bus_flow = BusFlow::UpHit;
                                did_wb_msg = " (Dirty Bit = 1, sin ir a RAM)";
                            }
                        }
                    }
                }
            }

            self.stats.total_latency_ns += latency;
            self.reg_rax = val_result.clone();
            self.reg_mdr = val_result;
            self.reg_status = format!("HIT L1 (Set {})", set_idx);
            self.sync_ram_flags(addr);

            self.feedback = format!(
                "CACHE HIT en L1 [Set {}, Via {}]: Direccion {} (Tag 0x{:X}, Bloque #{}){} en {:.1} ns.",
                set_idx, way_idx, format_hex(addr), tag, block_addr, did_wb_msg, latency
            );
            self.logs.insert(
                0,
                LogEntry {
                    step: self.clock,
                    op_name: op.to_string(),
                    addr,
                    set_idx,
                    tag,
                    level: HitLevel::L1,
                    is_hit: true,
                    miss_kind: None,
                    write_back: false,
                    latency_ns: latency,
                },
            );
            return;
        }

        // Fallo en L1 -> clasificar Compulsivo vs Conflicto/Capacidad
        let is_compulsory = !self.seen_blocks.contains(&block_addr);
        self.seen_blocks.insert(block_addr);
        let miss_kind_str = if is_compulsory {
            "Compulsivo"
        } else {
            "Conflicto/Capacidad"
        };

        // 2. Buscar en L2 y L3 (si estan habilitadas) antes de ir a RAM
        let mut latency = LAT_REG + LAT_L1;
        let mut found_level = HitLevel::Ram;
        let mut words: Option<Vec<(u8, String)>> = None;

        if self.config.use_l2_l3 {
            if let Some(pos) = self.l2.iter().position(|b| b.block_addr == block_addr) {
                latency += LAT_L2;
                found_level = HitLevel::L2;
                self.l2[pos].last_used = self.clock;
                self.l2[pos].last_hit = true;
                words = Some(self.l2[pos].words.clone());
            } else {
                latency += LAT_L2;
                if let Some(pos) = self.l3.iter().position(|b| b.block_addr == block_addr) {
                    latency += LAT_L3;
                    found_level = HitLevel::L3;
                    self.l3[pos].last_used = self.clock;
                    self.l3[pos].last_hit = true;
                    words = Some(self.l3[pos].words.clone());
                } else {
                    latency += LAT_L3 + LAT_RAM;
                }
            }
        } else {
            latency += LAT_RAM;
        }

        let mut loaded_words = words.unwrap_or_else(|| self.build_words_from_ram(block_addr));
        let mut line_dirty = false;

        if let MemOp::Write(new_val) = &op {
            if let Some((_, w)) = loaded_words.iter_mut().find(|(a, _)| *a == addr) {
                *w = new_val.clone();
            }
            match self.config.write_policy {
                WritePolicy::WriteThrough => {
                    self.ram[addr as usize].value = new_val.clone();
                    self.update_secondary_word(block_addr, addr, new_val);
                    if found_level != HitLevel::Ram {
                        latency += LAT_RAM;
                    }
                }
                WritePolicy::WriteBack => {
                    line_dirty = true;
                }
            }
        }

        // Insertar en L1 (Set set_idx), desalojando si no hay via libre
        let empty_way = self.sets[set_idx].iter().position(|s| s.is_none());
        let target_way = match empty_way {
            Some(w) => w,
            None => self.select_victim_way(set_idx),
        };

        let mut evicted_info: Option<(usize, bool)> = None;
        let mut did_wb = false;

        if let Some(victim) = self.sets[set_idx][target_way].take() {
            self.stats.evictions += 1;
            let victim_dirty = victim.dirty;
            let victim_blk = victim.block_addr;
            if victim_dirty && self.config.write_policy == WritePolicy::WriteBack {
                for (va, vw) in &victim.words {
                    if let Some(cell) = self.ram.get_mut(*va as usize) {
                        cell.value = vw.clone();
                    }
                    self.update_secondary_word(victim_blk, *va, vw);
                }
                self.stats.write_backs += 1;
                latency += LAT_RAM;
                did_wb = true;
            }
            if self.config.use_l2_l3 {
                Self::insert_secondary(
                    &mut self.l2,
                    L2_CAPACITY,
                    SecondaryBlock {
                        block_addr: victim_blk,
                        words: victim.words,
                        inserted_at: self.clock,
                        last_used: self.clock,
                        last_hit: false,
                    },
                    self.config.replacement,
                );
            }
            evicted_info = Some((victim_blk, victim_dirty));
        }

        let new_line = CacheLine {
            valid: true,
            dirty: line_dirty,
            tag,
            block_addr,
            words: loaded_words.clone(),
            inserted_at: self.clock,
            last_used: self.clock,
            flash: Some(if did_wb {
                SlotFlash::WriteBack
            } else {
                SlotFlash::Miss
            }),
        };
        self.sets[set_idx][target_way] = Some(new_line);

        // Actualizar L2/L3 si estan habilitadas y vino de RAM o L3
        if self.config.use_l2_l3 {
            if found_level == HitLevel::Ram {
                let sec = SecondaryBlock {
                    block_addr,
                    words: loaded_words.clone(),
                    inserted_at: self.clock,
                    last_used: self.clock,
                    last_hit: false,
                };
                Self::insert_secondary(&mut self.l3, L3_CAPACITY, sec.clone(), self.config.replacement);
                Self::insert_secondary(&mut self.l2, L2_CAPACITY, sec, self.config.replacement);
            } else if found_level == HitLevel::L3 {
                let sec = SecondaryBlock {
                    block_addr,
                    words: loaded_words.clone(),
                    inserted_at: self.clock,
                    last_used: self.clock,
                    last_hit: false,
                };
                Self::insert_secondary(&mut self.l2, L2_CAPACITY, sec, self.config.replacement);
            }
        }

        let is_cache_hit = matches!(found_level, HitLevel::L2 | HitLevel::L3);
        if is_cache_hit {
            self.stats.hits += 1;
            if found_level == HitLevel::L2 {
                self.stats.l2_hits += 1;
            } else {
                self.stats.l3_hits += 1;
            }
            self.xp_earned += 15;
            self.bus_flow = if did_wb {
                BusFlow::WriteBackToRam
            } else {
                BusFlow::UpHit
            };
        } else {
            self.stats.misses += 1;
            self.stats.ram_accesses += 1;
            if is_compulsory {
                self.stats.compulsory_misses += 1;
            } else {
                self.stats.conflict_misses += 1;
            }
            self.xp_earned += 5;
            self.bus_flow = if did_wb {
                BusFlow::WriteBackToRam
            } else {
                BusFlow::DownToRam
            };
        }

        self.stats.total_latency_ns += latency;
        self.last_hit_level = Some(found_level);

        let val_out = loaded_words
            .iter()
            .find(|(a, _)| *a == addr)
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| self.ram[addr as usize].value.clone());

        self.reg_rax = val_out.clone();
        self.reg_mdr = val_out;
        self.reg_status = if is_cache_hit {
            format!("HIT {} -> L1", found_level.label())
        } else {
            format!("MISS ({})", miss_kind_str)
        };
        self.sync_ram_flags(addr);

        let evict_note = match evicted_info {
            Some((vb, true)) if did_wb => format!(
                " Desalojo Bloque #{} con Write-Back a RAM (+60 ns).",
                vb
            ),
            Some((vb, _)) => format!(
                " Desalojo Bloque #{} ({}).",
                vb,
                self.config.replacement.label()
            ),
            None => String::new(),
        };

        self.feedback = format!(
            "{} en {} -> instalado en Set {}, Via {} (Bloque #{}).{} Latencia: {:.1} ns.",
            if is_cache_hit {
                format!("Miss L1 -> HIT en {}", found_level.label())
            } else {
                format!("CACHE MISS ({})", miss_kind_str)
            },
            found_level.label(),
            set_idx,
            target_way,
            block_addr,
            evict_note,
            latency
        );

        self.logs.insert(
            0,
            LogEntry {
                step: self.clock,
                op_name: op.to_string(),
                addr,
                set_idx,
                tag,
                level: found_level,
                is_hit: is_cache_hit,
                miss_kind: if is_cache_hit {
                    None
                } else {
                    Some(miss_kind_str.to_string())
                },
                write_back: did_wb,
                latency_ns: latency,
            },
        );
    }

    pub fn simulate_page_fault(&mut self, raw_addr: u8) {
        let addr = raw_addr & ((RAM_SIZE - 1) as u8);
        self.clear_flashes();
        self.clock += 1;
        self.stats.accesses += 1;
        self.stats.misses += 1;
        self.stats.disk_faults += 1;
        self.stats.compulsory_misses += 1;

        let latency = LAT_REG + LAT_L1 + LAT_L2 + LAT_L3 + LAT_RAM + LAT_DISK;
        self.stats.total_latency_ns += latency;
        self.last_breakdown = self.config.decode(addr);
        self.reg_mar = addr;
        self.reg_status = "PAGE FAULT (SSD)".to_string();
        self.bus_flow = BusFlow::PageFaultDisk;
        self.last_hit_level = Some(HitLevel::Disk);
        self.sync_ram_flags(addr);

        self.feedback = format!(
            "FALLO DE PAGINA (Page Fault) en {}: la pagina no estaba en RAM y se cargo desde SSD (+{:.1} ns).",
            format_hex(addr),
            latency
        );

        self.logs.insert(
            0,
            LogEntry {
                step: self.clock,
                op_name: "PAGE_FAULT".to_string(),
                addr,
                set_idx: self.last_breakdown.index_val,
                tag: self.last_breakdown.tag_val,
                level: HitLevel::Disk,
                is_hit: false,
                miss_kind: Some("Page Fault".to_string()),
                write_back: false,
                latency_ns: latency,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::presets::preset_sequence;
    use crate::sim::types::{MappingPolicy, PresetKind};

    #[test]
    fn decode_bits_direct_mapped_8x4() {
        let cfg = CacheConfig {
            num_blocks: 8,
            block_size: 4,
            mapping: MappingPolicy::DirectMapped,
            replacement: ReplacementPolicy::Lru,
            write_policy: WritePolicy::WriteBack,
            use_l2_l3: false,
        };
        let d = cfg.decode(0x25); // 0b100101 -> tag=1, index=1, offset=1
        assert_eq!(d.offset_bits, 2);
        assert_eq!(d.index_bits, 3);
        assert_eq!(d.tag_bits, 1);
        assert_eq!(d.tag_val, 1);
        assert_eq!(d.index_val, 1);
        assert_eq!(d.offset_val, 1);
    }

    #[test]
    fn sequential_vs_stride4_hit_rates() {
        let cfg = CacheConfig {
            num_blocks: 8,
            block_size: 4,
            mapping: MappingPolicy::DirectMapped,
            replacement: ReplacementPolicy::Lru,
            write_policy: WritePolicy::WriteBack,
            use_l2_l3: false,
        };
        let mut eng_seq = CacheEngine::new(cfg);
        for instr in preset_sequence(PresetKind::Sequential) {
            eng_seq.execute(instr.op, instr.addr);
        }
        assert_eq!(eng_seq.stats.accesses, 16);
        assert_eq!(eng_seq.stats.hits, 12);
        assert!((eng_seq.stats.hit_rate() - 75.0).abs() < 0.01);

        let mut eng_str = CacheEngine::new(cfg);
        for instr in preset_sequence(PresetKind::Stride4) {
            eng_str.execute(instr.op, instr.addr);
        }
        assert_eq!(eng_str.stats.accesses, 8);
        assert_eq!(eng_str.stats.hits, 0);
        assert!((eng_str.stats.hit_rate() - 0.0).abs() < 0.01);
    }

    #[test]
    fn thrashing_direct_vs_2way_associative() {
        let trace = preset_sequence(PresetKind::Thrashing);

        // Solo L1 (como MARS)
        let mut direct = CacheEngine::new(CacheConfig {
            num_blocks: 8,
            block_size: 4,
            mapping: MappingPolicy::DirectMapped,
            replacement: ReplacementPolicy::Lru,
            write_policy: WritePolicy::WriteBack,
            use_l2_l3: false,
        });
        for instr in &trace {
            direct.execute(instr.op.clone(), instr.addr);
        }
        assert_eq!(direct.stats.hits, 0);
        assert!((direct.stats.hit_rate() - 0.0).abs() < 0.01);

        let mut two_way = CacheEngine::new(CacheConfig {
            num_blocks: 8,
            block_size: 4,
            mapping: MappingPolicy::SetAssociative(2),
            replacement: ReplacementPolicy::Lru,
            write_policy: WritePolicy::WriteBack,
            use_l2_l3: false,
        });
        for instr in &trace {
            two_way.execute(instr.op.clone(), instr.addr);
        }
        assert_eq!(two_way.stats.hits, 12);
        assert!((two_way.stats.hit_rate() - 75.0).abs() < 0.01);

        // Con jerarquia L2/L3 activa, L1 Hit Rate sigue siendo 0% en DirectMapped vs 75% en 2-Way
        let mut direct_ml = CacheEngine::new(CacheConfig {
            num_blocks: 8,
            block_size: 4,
            mapping: MappingPolicy::DirectMapped,
            replacement: ReplacementPolicy::Lru,
            write_policy: WritePolicy::WriteBack,
            use_l2_l3: true,
        });
        for instr in &trace {
            direct_ml.execute(instr.op.clone(), instr.addr);
        }
        assert_eq!(direct_ml.stats.l1_hits, 0);
        assert!((direct_ml.stats.l1_hit_rate() - 0.0).abs() < 0.01);
    }

    #[test]
    fn write_back_flushes_dirty_line_on_eviction() {
        let mut eng = CacheEngine::new(CacheConfig {
            num_blocks: 4,
            block_size: 2,
            mapping: MappingPolicy::DirectMapped,
            replacement: ReplacementPolicy::Lru,
            write_policy: WritePolicy::WriteBack,
            use_l2_l3: false,
        });
        eng.execute(MemOp::Write("NEW_00".into()), 0x00);
        assert_eq!(eng.ram[0].value, "D_00");
        assert!(eng.ram[0].dirty_in_cache);
        assert_eq!(eng.stats.write_backs, 0);

        // 0x08 mapea al mismo Set 0 en 4x2 Direct Mapped -> desaloja 0x00..0x01 con Write-Back
        eng.execute(MemOp::Read, 0x08);
        assert_eq!(eng.ram[0].value, "NEW_00");
        assert!(!eng.ram[0].dirty_in_cache);
        assert_eq!(eng.stats.write_backs, 1);
    }
}
