use yew::prelude::*;

use crate::sim::engine::CacheEngine;
use crate::sim::types::{format_hex, BusFlow, HitLevel, SlotFlash};

#[derive(Properties, PartialEq)]
pub struct CacheGridProps {
    pub engine: CacheEngine,
}

#[function_component(CacheGrid)]
pub fn cache_grid(props: &CacheGridProps) -> Html {
    let eng = &props.engine;
    let cfg = &eng.config;
    let active_set = eng.last_breakdown.index_val;
    let ways = cfg.effective_ways();

    let (bus_class, bus_label) = match eng.bus_flow {
        BusFlow::Idle => ("", "BUS INACTIVO"),
        BusFlow::DownToRam => ("active-down", "BUS: MISS -> CONSULTANDO RAM"),
        BusFlow::UpHit => ("active-up", "BUS: HIT -> RETORNANDO DATO A CPU"),
        BusFlow::WriteBackToRam => ("active-wb", "BUS: ESCRITURA / WRITE-BACK A RAM"),
        BusFlow::PageFaultDisk => ("active-wb", "BUS: CARGANDO PAGINA DESDE SSD"),
    };

    html! {
        <div class="stack-col">
            // NIVEL 0: REGISTROS CPU
            <section class="panel">
                <div class="panel-header">
                    <h2 class="panel-title">{"Nivel 0 • Registros CPU"}</h2>
                    <span class="badge badge-emerald">{"0.5 ns • 128 B"}</span>
                </div>
                <div class="reg-grid">
                    <div class="reg-cell">
                        <span class="reg-name">{"RAX"}</span>
                        <span class="reg-val">{&eng.reg_rax}</span>
                    </div>
                    <div class="reg-cell">
                        <span class="reg-name">{"MAR"}</span>
                        <span class="reg-val" style="color:var(--accent-cyan);">{format_hex(eng.reg_mar)}</span>
                    </div>
                    <div class="reg-cell">
                        <span class="reg-name">{"MDR"}</span>
                        <span class="reg-val" style="color:var(--accent-amber);">{&eng.reg_mdr}</span>
                    </div>
                    <div class="reg-cell">
                        <span class="reg-name">{"ESTADO"}</span>
                        <span class="reg-val" style="font-size:0.68rem;">{&eng.reg_status}</span>
                    </div>
                </div>
            </section>

            // BUS CPU <-> L1
            <div class="bus-bridge">
                <div class={classes!("bus-track", bus_class)}></div>
                <span class="badge">{bus_label}</span>
                <div class={classes!("bus-track", bus_class)}></div>
            </div>

            // NIVEL 1: CACHE L1 CONFIGURABLE (CONJUNTOS x VIAS)
            <section class="panel">
                <div class="panel-header">
                    <h2 class="panel-title">
                        {format!("Nivel 1 • Cache L1 ({})", cfg.mapping.label())}
                    </h2>
                    <span class="badge badge-cyan">
                        {format!("1.0 ns • {} pal ({} B)", cfg.capacity_words(), cfg.capacity_bytes())}
                    </span>
                </div>

                <div class="sets-list">
                    { for eng.sets.iter().enumerate().map(|(set_idx, set_slots)| {
                        let is_target_set = eng.stats.accesses > 0 && set_idx == active_set;
                        let cols_style = format!("grid-template-columns: repeat({}, minmax(0, 1fr));", ways.min(4));
                        html! {
                            <div class={classes!("set-row", is_target_set.then_some("set-active"))}>
                                <div class="set-label">
                                    <span>{format!("SET {}", set_idx)}</span>
                                    <span style="font-size:0.62rem; color:var(--text-muted);">{format!("{} via(s)", ways)}</span>
                                </div>
                                <div class="ways-grid" style={cols_style}>
                                    { for set_slots.iter().enumerate().map(|(way_idx, opt_line)| {
                                        match opt_line {
                                            None => html! {
                                                <div class="way-slot empty">
                                                    <span class="mono" style="font-size:0.68rem; color:var(--text-muted);">
                                                        {format!("Via {} • V=0 (Vacia)", way_idx)}
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
                                                html! {
                                                    <div class={classes!("way-slot", flash_cls)}>
                                                        <div class="way-meta">
                                                            <div style="display:flex; gap:0.25rem; align-items:center;">
                                                                <span class="flag-pill flag-v">{"V:1"}</span>
                                                                <span class={classes!("flag-pill", if line.dirty { "flag-d-on" } else { "flag-d-off" })}>
                                                                    {if line.dirty { "D:1" } else { "D:0" }}
                                                                </span>
                                                                <span style="color:var(--accent-indigo); font-weight:700;">
                                                                    {format!("Tag 0x{:X}", line.tag)}
                                                                </span>
                                                            </div>
                                                            <span style="color:var(--text-muted);">
                                                                {format!("Blq #{} • t{}", line.block_addr, line.last_used)}
                                                            </span>
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

            // NIVELES 2 Y 3
            { if eng.config.use_l2_l3 {
                html! {
                    <div class="l2-l3-grid">
                        <section class="panel">
                            <div class="panel-header">
                                <h2 class="panel-title">{"Nivel 2 • Cache L2"}</h2>
                                <span class={classes!("badge", (eng.last_hit_level == Some(HitLevel::L2)).then_some("badge-emerald"))}>
                                    {"4.0 ns • 12 Blq"}
                                </span>
                            </div>
                            <div class="mini-cache-list">
                                { if eng.l2.is_empty() {
                                    html! { <span class="mono" style="font-size:0.7rem; color:var(--text-muted);">{"Sin bloques cargados"}</span> }
                                } else {
                                    html! { for eng.l2.iter().map(|blk| {
                                        html! {
                                            <span class={classes!("mini-block", blk.last_hit.then_some("hit"))}>
                                                {format!("B#{} ({})", blk.block_addr, format_hex(blk.words.first().map(|w| w.0).unwrap_or(0)))}
                                            </span>
                                        }
                                    }) }
                                }}
                            </div>
                        </section>

                        <section class="panel">
                            <div class="panel-header">
                                <h2 class="panel-title">{"Nivel 3 • Cache L3"}</h2>
                                <span class={classes!("badge", (eng.last_hit_level == Some(HitLevel::L3)).then_some("badge-emerald"))}>
                                    {"12.0 ns • 16 Blq"}
                                </span>
                            </div>
                            <div class="mini-cache-list">
                                { if eng.l3.is_empty() {
                                    html! { <span class="mono" style="font-size:0.7rem; color:var(--text-muted);">{"Sin bloques cargados"}</span> }
                                } else {
                                    html! { for eng.l3.iter().map(|blk| {
                                        html! {
                                            <span class={classes!("mini-block", blk.last_hit.then_some("hit"))}>
                                                {format!("B#{} ({})", blk.block_addr, format_hex(blk.words.first().map(|w| w.0).unwrap_or(0)))}
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
                    <section class="panel">
                        <div class="panel-header" style="margin-bottom:0; padding-bottom:0; border-bottom:none;">
                            <h2 class="panel-title">{"Niveles 2 y 3 (L2 / L3) Desactivados"}</h2>
                            <span class="badge">{"Modo estricto L1 -> RAM (como MARS)"}</span>
                        </div>
                    </section>
                }
            }}
        </div>
    }
}
