use std::fmt;

pub const ADDR_BITS: usize = 6;
pub const RAM_SIZE: usize = 64; // 0x00 .. 0x3F

pub const LAT_REG: f64 = 0.5;
pub const LAT_L1: f64 = 1.0;
pub const LAT_L2: f64 = 4.0;
pub const LAT_L3: f64 = 12.0;
pub const LAT_RAM: f64 = 60.0;
pub const LAT_DISK: f64 = 10_000.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MappingPolicy {
    DirectMapped,
    SetAssociative(usize),
    FullyAssociative,
}

impl MappingPolicy {
    pub fn label(self) -> &'static str {
        match self {
            MappingPolicy::DirectMapped => "Mapeo Directo (1-way)",
            MappingPolicy::SetAssociative(2) => "Asociativo 2 vias",
            MappingPolicy::SetAssociative(4) => "Asociativo 4 vias",
            MappingPolicy::SetAssociative(_) => "Asociativo N vias",
            MappingPolicy::FullyAssociative => "Totalmente Asociativo",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ReplacementPolicy {
    Lru,
    Fifo,
    Random,
}

impl ReplacementPolicy {
    pub fn label(self) -> &'static str {
        match self {
            ReplacementPolicy::Lru => "LRU",
            ReplacementPolicy::Fifo => "FIFO",
            ReplacementPolicy::Random => "Random",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WritePolicy {
    WriteThrough,
    WriteBack,
}

impl WritePolicy {
    pub fn label(self) -> &'static str {
        match self {
            WritePolicy::WriteThrough => "Write-Through",
            WritePolicy::WriteBack => "Write-Back (Dirty Bit)",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CacheConfig {
    pub num_blocks: usize,
    pub block_size: usize,
    pub mapping: MappingPolicy,
    pub replacement: ReplacementPolicy,
    pub write_policy: WritePolicy,
    pub use_l2_l3: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            num_blocks: 8,
            block_size: 2,
            mapping: MappingPolicy::DirectMapped,
            replacement: ReplacementPolicy::Lru,
            write_policy: WritePolicy::WriteBack,
            use_l2_l3: true,
        }
    }
}

impl CacheConfig {
    pub fn effective_ways(&self) -> usize {
        match self.mapping {
            MappingPolicy::DirectMapped => 1,
            MappingPolicy::SetAssociative(w) => w.min(self.num_blocks).max(1),
            MappingPolicy::FullyAssociative => self.num_blocks.max(1),
        }
    }

    pub fn num_sets(&self) -> usize {
        (self.num_blocks / self.effective_ways()).max(1)
    }

    pub fn capacity_words(&self) -> usize {
        self.num_blocks * self.block_size
    }

    pub fn capacity_bytes(&self) -> usize {
        self.capacity_words() * 4
    }

    pub fn offset_bits(&self) -> usize {
        self.block_size.trailing_zeros() as usize
    }

    pub fn index_bits(&self) -> usize {
        self.num_sets().trailing_zeros() as usize
    }

    pub fn tag_bits(&self) -> usize {
        ADDR_BITS.saturating_sub(self.offset_bits() + self.index_bits())
    }

    pub fn decode(&self, addr: u8) -> AddressBreakdown {
        let addr_usize = (addr as usize) & (RAM_SIZE - 1);
        let offset_bits = self.offset_bits();
        let index_bits = self.index_bits();
        let tag_bits = self.tag_bits();

        let offset_mask = (1usize << offset_bits) - 1;
        let index_mask = (1usize << index_bits) - 1;

        let offset_val = addr_usize & offset_mask;
        let block_addr = addr_usize >> offset_bits;
        let index_val = block_addr & index_mask;
        let tag_val = block_addr >> index_bits;

        let mut bits = Vec::with_capacity(ADDR_BITS);
        for bit_pos in (0..ADDR_BITS).rev() {
            let bit = ((addr_usize >> bit_pos) & 1) as u8;
            let field = if bit_pos >= offset_bits + index_bits {
                BitField::Tag
            } else if bit_pos >= offset_bits {
                BitField::Index
            } else {
                BitField::Offset
            };
            bits.push(BitCell { bit, field });
        }

        AddressBreakdown {
            addr: addr_usize as u8,
            block_addr,
            offset_bits,
            index_bits,
            tag_bits,
            offset_val,
            index_val,
            tag_val,
            bits,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BitField {
    Tag,
    Index,
    Offset,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BitCell {
    pub bit: u8,
    pub field: BitField,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AddressBreakdown {
    pub addr: u8,
    pub block_addr: usize,
    pub offset_bits: usize,
    pub index_bits: usize,
    pub tag_bits: usize,
    pub offset_val: usize,
    pub index_val: usize,
    pub tag_val: usize,
    pub bits: Vec<BitCell>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MemOp {
    Read,
    Write(String),
}

impl fmt::Display for MemOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemOp::Read => write!(f, "READ"),
            MemOp::Write(v) => write!(f, "WRITE({})", v),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MemInstruction {
    pub op: MemOp,
    pub addr: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HitLevel {
    L1,
    L2,
    L3,
    Ram,
    Disk,
}

impl HitLevel {
    pub fn label(self) -> &'static str {
        match self {
            HitLevel::L1 => "L1",
            HitLevel::L2 => "L2",
            HitLevel::L3 => "L3",
            HitLevel::Ram => "RAM",
            HitLevel::Disk => "SSD",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SlotFlash {
    Hit,
    Miss,
    WriteBack,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CacheLine {
    pub valid: bool,
    pub dirty: bool,
    pub tag: usize,
    pub block_addr: usize,
    pub words: Vec<(u8, String)>,
    pub inserted_at: u64,
    pub last_used: u64,
    pub flash: Option<SlotFlash>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SecondaryBlock {
    pub block_addr: usize,
    pub words: Vec<(u8, String)>,
    pub inserted_at: u64,
    pub last_used: u64,
    pub last_hit: bool,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RamCell {
    pub addr: u8,
    pub value: String,
    pub dirty_in_cache: bool,
    pub cached_in_l1: bool,
    pub last_accessed: bool,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct SimStats {
    pub accesses: u64,
    pub hits: u64,
    pub misses: u64,
    pub l1_hits: u64,
    pub l2_hits: u64,
    pub l3_hits: u64,
    pub ram_accesses: u64,
    pub disk_faults: u64,
    pub compulsory_misses: u64,
    pub conflict_misses: u64,
    pub evictions: u64,
    pub write_backs: u64,
    pub writes_count: u64,
    pub total_latency_ns: f64,
}

impl SimStats {
    pub fn l1_hit_rate(&self) -> f64 {
        if self.accesses == 0 {
            0.0
        } else {
            (self.l1_hits as f64 / self.accesses as f64) * 100.0
        }
    }

    pub fn hit_rate(&self) -> f64 {
        if self.accesses == 0 {
            0.0
        } else {
            (self.hits as f64 / self.accesses as f64) * 100.0
        }
    }

    pub fn miss_rate(&self) -> f64 {
        if self.accesses == 0 {
            0.0
        } else {
            (self.misses as f64 / self.accesses as f64) * 100.0
        }
    }

    pub fn amat_ns(&self) -> f64 {
        if self.accesses == 0 {
            0.0
        } else {
            self.total_latency_ns / self.accesses as f64
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct LogEntry {
    pub step: u64,
    pub op_name: String,
    pub addr: u8,
    pub set_idx: usize,
    pub tag: usize,
    pub level: HitLevel,
    pub is_hit: bool,
    pub miss_kind: Option<String>,
    pub write_back: bool,
    pub latency_ns: f64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BusFlow {
    Idle,
    DownToRam,
    UpHit,
    WriteBackToRam,
    PageFaultDisk,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SimMode {
    Learn,
    Lab,
    Challenge,
    Compare,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PresetKind {
    Sequential,
    Temporal,
    Spatial,
    Stride4,
    Thrashing,
    WriteHeavy,
}

impl PresetKind {
    pub fn label(self) -> &'static str {
        match self {
            PresetKind::Sequential => "Secuencial (Stride 1)",
            PresetKind::Temporal => "Localidad Temporal",
            PresetKind::Spatial => "Localidad Espacial",
            PresetKind::Stride4 => "Saltos Stride = 4",
            PresetKind::Thrashing => "Conflicto / Thrashing",
            PresetKind::WriteHeavy => "Escrituras (WB vs WT)",
        }
    }
}

pub fn format_hex(addr: u8) -> String {
    format!("0x{:02X}", addr)
}

pub fn format_bin(val: usize, bits: usize) -> String {
    if bits == 0 {
        "-".to_string()
    } else {
        format!("{:0width$b}", val, width = bits)
    }
}
