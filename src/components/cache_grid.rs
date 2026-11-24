use yew::prelude::*;

use crate::components::EvictionNotice;
use crate::sim::engine::CacheEngine;
use crate::sim::types::{format_hex, HitLevel, SlotFlash};

#[derive(Properties, PartialEq)]
pub struct CacheGridProps {
    pub engine: CacheEngine,
    pub eviction: Option<EvictionNotice>,
}

#[function_component(CacheGrid)]
pub fn cache_grid(props: &CacheGridProps) -> Html {
    let eng = &props.engine;
    let cfg = &eng.config;
    let active_set = eng.last_breakdown.index_val;
    let ways = cfg.effective_ways();
    let clock = eng.clock;

    html! {
        <div class="cache-hierarchy-col">
            <section class="panel l1-panel" aria-label="Nivel 1 Cache L1">
                <div class="panel-header compact-header">
                    <h2 class="panel-title">
                        {format!("Nivel 1 · Cache L1 ({})", cfg.mapping.label())}
                    </h2>
                    <span class="badge badge-cyan">
                        {format!("1.0 ns · {} pal ({} B)", cfg.capacity_words(), cfg.capacity_bytes())}
                    </span>
                </div>

                <div class="sets-list">
                    { for eng.sets.iter().enumerate().map(|(set_idx, set_slots)| {
                        let is_target_set = eng.stats.accesses > 0 && set_idx == active_set;
                        let cols_style = format!("grid-template-columns: repeat({}, minmax(0, 1fr));", ways.min(4));
                        html! {
                            <div class={classes!("set-row", is_target_set.then_some("set-active"))}>
                                <div class="set-label">
                                    <span>{format!("S{}", set_idx)}</span>
                                    { if ways > 1 {
                                        html! { <span class="set-ways-hint">{format!("{}W", ways)}</span> }
                                    } else { html! {} }}
                                </div>
                                <div class="ways-grid" style={cols_style}>
                                    { for set_slots.iter().enumerate().map(|(way_idx, opt_line)| {
                                        let evicted_here = props.eviction.as_ref().filter(|e| e.set_idx == set_idx && e.way_idx == way_idx);
                                        match opt_line {
                                            None => html! {
                                                <div class={classes!("way-slot", "empty", (ways == 1).then_some("single-way"))}>
                                                    <span class="mono slot-empty-txt">
                                                        {format!("V{} · V=0", way_idx)}
                                                    </span>
                                                </div>
                                            },
                                            Some(line) => {
                                                let flash_cls = match line.flash {
                                                    Some(SlotFlash::Hit) => Some("flash-hit"),
                                                    Some(SlotFlash::Miss) => Some("flash-miss"),
                                                    Some(SlotFlash::WriteBack) => Some("flash-wb"),
                                                    None => None,
                                                };
                                                let slot_key = format!("slot-{}-{}-c{}", set_idx, way_idx, if line.flash.is_some() { clock } else { 0 });
                                                html! {
                                                    <div key={slot_key} class={classes!("way-slot", (ways == 1).then_some("single-way"), flash_cls)}>
                                                        <div class="way-meta">
                                                            <div class="way-flags">
                                                                <span class="flag-pill flag-v">{"V:1"}</span>
                                                                <span class={classes!("flag-pill", if line.dirty { "flag-d-on" } else { "flag-d-off" })}>
                                                                    {if line.dirty { "D:1" } else { "D:0" }}
                                                                </span>
                                                                <span class="tag-chip mono">
                                                                    {format!("T:0x{:X}", line.tag)}
                                                                </span>
                                                                <span class="blk-chip mono">
                                                                    {format!("B#{}", line.block_addr)}
                                                                </span>
                                                            </div>
                                                            <div class="way-right-meta">
                                                                { if let Some(ev) = evicted_here {
                                                                    html! {
                                                                        <span class={classes!("evict-eject-pill", if ev.write_back_ram { "wb" } else { "ev" })}>
                                                                            { if ev.write_back_ram {
                                                                                format!("WB B#{}", ev.old_block)
                                                                            } else {
                                                                                format!("OUT B#{}", ev.old_block)
                                                                            }}
                                                                        </span>
                                                                    }
                                                                } else {
                                                                    html! {}
                                                                }}
                                                                <span class="lru-tick mono">{format!("t{}", line.last_used)}</span>
                                                            </div>
                                                        </div>
                                                        <div class="words-row">
                                                            { for line.words.iter().map(|(wa, wv)| {
                                                                let is_target = *wa == eng.reg_mar && eng.stats.accesses > 0;
                                                                html! {
                                                                    <span class={classes!("word-chip", is_target.then_some("target"))}>
                                                                        {format!("{}:{}", format_hex(*wa), wv)}
                                                                    </span>
                                                                }
                                                            }) }
                                                        </div>
                                                    </div>
                                                }
                                            }
                                        }
                                    }) }
                                </div>
                            </div>
                        }
                    }) }
                </div>
            </section>

            // Nivel 2 y 3 en barra compacta horizontal
            { if eng.config.use_l2_l3 {
                html! {
                    <div class="l2-l3-grid">
                        <section class="panel compact-panel l23-panel">
                            <div class="panel-header compact-header">
                                <h2 class="panel-title">{"L2 (4 ns)"}</h2>
                                <span class={classes!("badge", (eng.last_hit_level == Some(HitLevel::L2)).then_some("badge-emerald"))}>
                                    {format!("{}/12 Blq", eng.l2.len())}
                                </span>
                            </div>
                            <div class="mini-cache-list">
                                { if eng.l2.is_empty() {
                                    html! { <span class="mono muted-xs">{"Vacia"}</span> }
                                } else {
                                    html! { for eng.l2.iter().map(|blk| {
                                        html! {
                                            <span class={classes!("mini-block", blk.last_hit.then_some("hit"))}>
                                                {format!("B#{}", blk.block_addr)}
                                            </span>
                                        }
                                    }) }
                                }}
                            </div>
                        </section>

                        <section class="panel compact-panel l23-panel">
                            <div class="panel-header compact-header">
                                <h2 class="panel-title">{"L3 (12 ns)"}</h2>
                                <span class={classes!("badge", (eng.last_hit_level == Some(HitLevel::L3)).then_some("badge-emerald"))}>
                                    {format!("{}/16 Blq", eng.l3.len())}
                                </span>
                            </div>
                            <div class="mini-cache-list">
                                { if eng.l3.is_empty() {
                                    html! { <span class="mono muted-xs">{"Vacia"}</span> }
                                } else {
                                    html! { for eng.l3.iter().map(|blk| {
                                        html! {
                                            <span class={classes!("mini-block", blk.last_hit.then_some("hit"))}>
                                                {format!("B#{}", blk.block_addr)}
                                            </span>
                                        }
                                    }) }
                                }}
                            </div>
                        </section>
                    </div>
                }
            } else {
                html! {
                    <section class="panel compact-panel">
                        <div class="panel-header compact-header" style="margin-bottom:0; padding-bottom:0; border-bottom:none;">
                            <h2 class="panel-title">{"Niveles L2 / L3 en Bypass"}</h2>
                            <span class="badge">{"Modo directo L1 -> DRAM"}</span>
                        </div>
                    </section>
                }
            }}
        </div>
    }
}
