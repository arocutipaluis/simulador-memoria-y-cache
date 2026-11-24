pub mod address_decoder;
pub mod bus_datapath;
pub mod cache_grid;
pub mod comparator_view;
pub mod eval_modal;
pub mod header;
pub mod icons;
pub mod metrics_panel;
pub mod mode_banner;
pub mod ram_view;
pub mod step_controller;

use crate::sim::engine::CacheEngine;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EvictionNotice {
    pub set_idx: usize,
    pub way_idx: usize,
    pub old_block: usize,
    pub old_tag: usize,
    pub was_dirty: bool,
    pub new_block: usize,
    pub write_back_ram: bool,
}

pub fn detect_eviction(prev: &CacheEngine, next: &CacheEngine) -> Option<EvictionNotice> {
    if next.stats.evictions <= prev.stats.evictions {
        return None;
    }
    let write_back_ram = next.stats.write_backs > prev.stats.write_backs;
    for (set_idx, (prev_set, next_set)) in prev.sets.iter().zip(next.sets.iter()).enumerate() {
        for (way_idx, (prev_slot, next_slot)) in prev_set.iter().zip(next_set.iter()).enumerate() {
            if let (Some(old_line), Some(new_line)) = (prev_slot, next_slot) {
                if old_line.block_addr != new_line.block_addr {
                    return Some(EvictionNotice {
                        set_idx,
                        way_idx,
                        old_block: old_line.block_addr,
                        old_tag: old_line.tag,
                        was_dirty: old_line.dirty,
                        new_block: new_line.block_addr,
                        write_back_ram,
                    });
                }
            }
        }
    }
    None
}
