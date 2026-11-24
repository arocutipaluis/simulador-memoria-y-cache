use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

use crate::components::icons::{PauseIcon, PlayIcon, ResetIcon, StepIcon};
use crate::components::step_controller::parse_address_str;
use crate::components::EvictionNotice;
use crate::sim::engine::CacheEngine;
use crate::sim::types::{
    format_hex, BusFlow, CacheConfig, HitLevel, MappingPolicy, MemInstruction, MemOp, PresetKind,
    ReplacementPolicy, SlotFlash, WritePolicy,
};

#[derive(Properties, PartialEq)]
pub struct ComparatorViewProps {
    pub engine_a: CacheEngine,
    pub engine_b: CacheEngine,
    pub eviction_a: Option<EvictionNotice>,
    pub eviction_b: Option<EvictionNotice>,
    pub trace: Vec<MemInstruction>,
    pub cursor: usize,
    pub is_playing: bool,
    pub custom_trace_input: String,
    pub trace_error: Option<String>,
    pub on_config_a: Callback<CacheConfig>,
    pub on_config_b: Callback<CacheConfig>,
    pub on_load_preset: Callback<PresetKind>,
    pub on_custom_trace_input: Callback<String>,
    pub on_apply_custom_trace: Callback<()>,
    pub on_step: Callback<()>,
    pub on_toggle_play: Callback<()>,
    pub on_run_all: Callback<()>,
    pub on_reset: Callback<()>,
}

pub fn parse_trace_line(raw: &str) -> Result<Vec<MemInstruction>, String> {
    let mut out = Vec::new();
    for token in raw.split(|c| c == ',' || c == ';').map(str::trim).filter(|s| !s.is_empty()) {
        let upper = token.to_uppercase();
        if let Some(rest) = upper.strip_prefix("W:") {
            let mut parts = rest.splitn(2, '=');
            let addr_str = parts.next().unwrap_or("");
            let val_str = parts.next().unwrap_or("WR_VAL");
            let addr = parse_address_str(addr_str)?;
            let clean_val: String = val_str
                .chars()
                .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
                .take(7)
                .collect();
            out.push(MemInstruction {
                op: MemOp::Write(if clean_val.is_empty() { "WR_VAL".into() } else { clean_val }),
                addr,
            });
        } else {
            let addr_str = upper.strip_prefix("R:").unwrap_or(&upper);
            let addr = parse_address_str(addr_str)?;
            out.push(MemInstruction {
                op: MemOp::Read,
                addr,
            });
        }
    }
    if out.is_empty() {
        return Err("Ingresa al menos una direccion, ej: 0x00, 0x20, W:0x01=X9".to_string());
    }
    Ok(out)
}

#[derive(Properties, PartialEq)]
struct SystemColumnProps {
    pub title: &'static str,
    pub accent_class: &'static str,
    pub engine: CacheEngine,
    pub eviction: Option<EvictionNotice>,
    pub on_config: Callback<CacheConfig>,
}

#[function_component(SystemColumn)]
fn system_column(props: &SystemColumnProps) -> Html {
    let eng = &props.engine;
    let cfg = eng.config;
    let s = &eng.stats;
    let ways = cfg.effective_ways();
    let clock = eng.clock;

    let mapping_val = match cfg.mapping {
        MappingPolicy::DirectMapped => "direct",
        MappingPolicy::SetAssociative(2) => "set2",
        MappingPolicy::SetAssociative(4) => "set4",
        _ => "full",
    };
    let repl_val = match cfg.replacement {
        ReplacementPolicy::Lru => "lru",
        ReplacementPolicy::Fifo => "fifo",
        ReplacementPolicy::Random => "random",
    };
    let wp_val = match cfg.write_policy {
        WritePolicy::WriteThrough => "wt",
        WritePolicy::WriteBack => "wb",
    };

    let mini_pkt_cls = if clock == 0 {
        ""
    } else if eng.last_hit_level == Some(HitLevel::L1) {
        "mini-pkt-hit"
    } else if props.eviction.as_ref().map(|e| e.write_back_ram).unwrap_or(false)
        || eng.bus_flow == BusFlow::WriteBackToRam
    {
        "mini-pkt-wb"
    } else {
        "mini-pkt-miss"
    };

    let active_set = eng.last_breakdown.index_val;

    html! {
        <section class="panel compact-panel cmp-col-panel">
            <div class="panel-header compact-header">
                <h3 class="panel-title">{props.title}</h3>
                <span class={classes!("badge", props.accent_class)}>
                    {format!("{} · {}x{} ({}B)", cfg.mapping.label(), cfg.num_blocks, cfg.block_size, cfg.capacity_bytes())}
                </span>
            </div>

            // Configuracion inline 4 columnas
            <div class="cmp-selects-4">
                <div class="field-group">
                    <label class="field-label">{"Mapeo"}</label>
                    <select
                        class="select select-sm"
                        onchange={
                            let cb = props.on_config.clone();
                            Callback::from(move |e: Event| {
                                let sel: HtmlSelectElement = e.target_unchecked_into();
                                let mut next = cfg;
                                next.mapping = match sel.value().as_str() {
                                    "direct" => MappingPolicy::DirectMapped,
                                    "set2" => MappingPolicy::SetAssociative(2),
                                    "set4" => MappingPolicy::SetAssociative(4),
                                    _ => MappingPolicy::FullyAssociative,
                                };
                                cb.emit(next);
                            })
                        }
                    >
                        <option value="direct" selected={mapping_val == "direct"}>{"Directo"}</option>
                        <option value="set2" selected={mapping_val == "set2"}>{"2-Way"}</option>
                        <option value="set4" selected={mapping_val == "set4"}>{"4-Way"}</option>
                        <option value="full" selected={mapping_val == "full"}>{"Full"}</option>
                    </select>
                </div>

                <div class="field-group">
                    <label class="field-label">{"Geometria"}</label>
                    <select
                        class="select select-sm"
                        onchange={
                            let cb = props.on_config.clone();
                            Callback::from(move |e: Event| {
                                let sel: HtmlSelectElement = e.target_unchecked_into();
                                let mut next = cfg;
                                match sel.value().as_str() {
                                    "4x2" => { next.num_blocks = 4; next.block_size = 2; }
                                    "8x1" => { next.num_blocks = 8; next.block_size = 1; }
                                    "8x2" => { next.num_blocks = 8; next.block_size = 2; }
                                    _ => { next.num_blocks = 8; next.block_size = 4; }
                                }
                                cb.emit(next);
                            })
                        }
                    >
                        <option value="4x2" selected={cfg.num_blocks == 4 && cfg.block_size == 2}>{"4x2 (32B)"}</option>
                        <option value="8x1" selected={cfg.num_blocks == 8 && cfg.block_size == 1}>{"8x1 (32B)"}</option>
                        <option value="8x2" selected={cfg.num_blocks == 8 && cfg.block_size == 2}>{"8x2 (64B)"}</option>
                        <option value="8x4" selected={cfg.num_blocks == 8 && cfg.block_size == 4}>{"8x4 (128B)"}</option>
                    </select>
                </div>

                <div class="field-group">
                    <label class="field-label">{"Reemplazo"}</label>
                    <select
                        class="select select-sm"
                        disabled={matches!(cfg.mapping, MappingPolicy::DirectMapped)}
                        onchange={
                            let cb = props.on_config.clone();
                            Callback::from(move |e: Event| {
                                let sel: HtmlSelectElement = e.target_unchecked_into();
                                let mut next = cfg;
                                next.replacement = match sel.value().as_str() {
                                    "fifo" => ReplacementPolicy::Fifo,
                                    "random" => ReplacementPolicy::Random,
                                    _ => ReplacementPolicy::Lru,
                                };
                                cb.emit(next);
                            })
                        }
                    >
                        <option value="lru" selected={repl_val == "lru"}>{"LRU"}</option>
                        <option value="fifo" selected={repl_val == "fifo"}>{"FIFO"}</option>
                        <option value="random" selected={repl_val == "random"}>{"Random"}</option>
                    </select>
                </div>

                <div class="field-group">
                    <label class="field-label">{"Escritura"}</label>
                    <select
                        class="select select-sm"
                        onchange={
                            let cb = props.on_config.clone();
                            Callback::from(move |e: Event| {
                                let sel: HtmlSelectElement = e.target_unchecked_into();
                                let mut next = cfg;
                                next.write_policy = if sel.value() == "wt" {
                                    WritePolicy::WriteThrough
                                } else {
                                    WritePolicy::WriteBack
                                };
                                cb.emit(next);
                            })
                        }
                    >
                        <option value="wb" selected={wp_val == "wb"}>{"Write-Back"}</option>
                        <option value="wt" selected={wp_val == "wt"}>{"Write-Thru"}</option>
                    </select>
                </div>
            </div>

            // Mini Bus SVG animado por columna (CPU -> L1 [Set] -> DRAM)
            <div class="mini-svg-bus">
                <svg viewBox="0 0 380 34" preserveAspectRatio="xMidYMid meet" aria-hidden="true">
                    <path class="bus-base" d="M 62 17 L 138 17 M 242 17 L 318 17" />
                    <rect x="6" y="4" width="56" height="26" rx="5" class="mini-node" />
                    <text x="34" y="20" text-anchor="middle" class="mini-node-txt mono">
                        {if clock == 0 { "CPU".to_string() } else { format_hex(eng.reg_mar) }}
                    </text>

                    <rect x="138" y="4" width="104" height="26" rx="5" class={if clock == 0 { "mini-node" } else if eng.last_hit_level == Some(HitLevel::L1) { "mini-node hit" } else { "mini-node miss" }} />
                    <text x="190" y="20" text-anchor="middle" class="mini-node-txt mono">
                        {format!("L1 S{} {}", active_set, if clock == 0 { "" } else if eng.last_hit_level == Some(HitLevel::L1) { "HIT" } else { "MISS" })}
                    </text>

                    <rect x="318" y="4" width="56" height="26" rx="5" class={if clock > 0 && eng.last_hit_level != Some(HitLevel::L1) { "mini-node miss" } else { "mini-node" }} />
                    <text x="346" y="20" text-anchor="middle" class="mini-node-txt mono">{"DRAM"}</text>

                    { if clock > 0 {
                        html! {
                            <g key={format!("mpkt-{}", clock)} class={classes!("mini-pkt", mini_pkt_cls)}>
                                <circle cx="0" cy="0" r="5" />
                            </g>
                        }
                    } else {
                        html! {}
                    }}
                </svg>
            </div>

            // 4 KPIs compactos
            <div class="cmp-kpi-4">
                <div class="kpi-tile hit">
                    <span class="kpi-label">{"Hit Rate"}</span>
                    <span class="kpi-val mono text-emerald">{format!("{:.1}%", s.hit_rate())}</span>
                </div>
                <div class="kpi-tile miss">
                    <span class="kpi-label">{"Miss Rate"}</span>
                    <span class="kpi-val mono text-rose">{format!("{:.1}%", s.miss_rate())}</span>
                </div>
                <div class="kpi-tile">
                    <span class="kpi-label">{"AMAT"}</span>
                    <span class="kpi-val mono text-cyan">{format!("{:.1} ns", s.amat_ns())}</span>
                </div>
                <div class="kpi-tile wb">
                    <span class="kpi-label">{"Conf / WB"}</span>
                    <span class="kpi-val mono text-amber">{format!("{}/{}", s.conflict_misses, s.write_backs)}</span>
                </div>
            </div>

            // Sets L1 compactos
            <div class="sets-list cmp-sets-scroll">
                { for eng.sets.iter().enumerate().map(|(set_idx, set_slots)| {
                    let is_active = eng.stats.accesses > 0 && set_idx == active_set;
                    let cols_style = format!("grid-template-columns: repeat({}, minmax(0, 1fr));", ways.min(4));
                    html! {
                        <div class={classes!("set-row", is_active.then_some("set-active"))}>
                            <div class="set-label">
                                <span>{format!("S{}", set_idx)}</span>
                            </div>
                            <div class="ways-grid" style={cols_style}>
                                { for set_slots.iter().enumerate().map(|(w_idx, opt_line)| {
                                    let evicted_here = props.eviction.as_ref().filter(|e| e.set_idx == set_idx && e.way_idx == w_idx);
                                    match opt_line {
                                        None => html! {
                                            <div class={classes!("way-slot", "empty", (ways == 1).then_some("single-way"))}>
                                                <span class="mono slot-empty-txt">{format!("V{}: -", w_idx)}</span>
                                            </div>
                                        },
                                        Some(line) => {
                                            let flash_cls = match line.flash {
                                                Some(SlotFlash::Hit) => Some("flash-hit"),
                                                Some(SlotFlash::Miss) => Some("flash-miss"),
                                                Some(SlotFlash::WriteBack) => Some("flash-wb"),
                                                None => None,
                                            };
                                            let slot_key = format!("cmp-s{}-w{}-c{}", set_idx, w_idx, if line.flash.is_some() { clock } else { 0 });
                                            html! {
                                                <div key={slot_key} class={classes!("way-slot", "single-way", flash_cls)}>
                                                    <div class="way-meta">
                                                        <span class="blk-chip mono">{format!("B#{} T:{:X}", line.block_addr, line.tag)}</span>
                                                        <div class="way-right-meta">
                                                            { if let Some(ev) = evicted_here {
                                                                html! {
                                                                    <span class={classes!("evict-eject-pill", if ev.write_back_ram { "wb" } else { "ev" })}>
                                                                        {format!("OUT B#{}", ev.old_block)}
                                                                    </span>
                                                                }
                                                            } else { html! {} }}
                                                            { if line.dirty {
                                                                html! { <span class="flag-pill flag-d-on">{"D"}</span> }
                                                            } else { html! {} }}
                                                        </div>
                                                    </div>
                                                    <div class="mono muted-xs">
                                                        {format!("{}..{}", format_hex(line.words.first().map(|x| x.0).unwrap_or(0)), format_hex(line.words.last().map(|x| x.0).unwrap_or(0)))}
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
    }
}

#[function_component(ComparatorView)]
pub fn comparator_view(props: &ComparatorViewProps) -> Html {
    let sa = &props.engine_a.stats;
    let sb = &props.engine_b.stats;
    let has_pending = props.cursor < props.trace.len();

    let presets = [
        (PresetKind::Thrashing, "Thrashing"),
        (PresetKind::Sequential, "Secuencial"),
        (PresetKind::Stride4, "Stride = 4"),
        (PresetKind::Spatial, "Espacial"),
        (PresetKind::Temporal, "Temporal"),
        (PresetKind::WriteHeavy, "WB vs WT"),
    ];

    let hit_diff = sb.hit_rate() - sa.hit_rate();
    let amat_diff = sa.amat_ns() - sb.amat_ns();

    let winner_banner = if sa.accesses == 0 {
        "Presiona Step o Auto para comparar Sistema A vs Sistema B".to_string()
    } else if hit_diff.abs() < 0.05 && amat_diff.abs() < 0.05 {
        "Empate tecnico en Hit Rate y AMAT".to_string()
    } else if hit_diff > 0.0 || amat_diff > 0.0 {
        format!("Sistema B lidera: {:+.1}% Hit | -{:.1} ns AMAT", hit_diff, amat_diff)
    } else {
        format!("Sistema A lidera: {:+.1}% Hit | -{:.1} ns AMAT", -hit_diff, -amat_diff)
    };

    let max_amat = sa.amat_ns().max(sb.amat_ns()).max(1.0);
    let scale_hit_a = (sa.hit_rate() / 100.0).clamp(0.0, 1.0);
    let scale_hit_b = (sb.hit_rate() / 100.0).clamp(0.0, 1.0);
    let scale_amat_a = (sa.amat_ns() / max_amat).clamp(0.0, 1.0);
    let scale_amat_b = (sb.amat_ns() / max_amat).clamp(0.0, 1.0);

    html! {
        <div class="comparator-shell">
            // Top Controls + Shared Trace
            <section class="panel compact-panel cmp-top-panel">
                <div class="cmp-controls-bar">
                    <div class="cmp-presets-row">
                        { for presets.into_iter().map(|(p, label)| {
                            let cb = props.on_load_preset.clone();
                            html! {
                                <button class="btn btn-xs" type="button" onclick={Callback::from(move |_| cb.emit(p))}>
                                    {label}
                                </button>
                            }
                        }) }
                    </div>

                    <div class="cmp-transport-row">
                        <input
                            class="input input-sm cmp-trace-input"
                            type="text"
                            placeholder="0x00, 0x20, W:0x02=X9"
                            value={props.custom_trace_input.clone()}
                            oninput={
                                let cb = props.on_custom_trace_input.clone();
                                Callback::from(move |e: InputEvent| {
                                    let inp: HtmlInputElement = e.target_unchecked_into();
                                    cb.emit(inp.value());
                                })
                            }
                        />
                        <button
                            class="btn btn-xs"
                            type="button"
                            onclick={
                                let cb = props.on_apply_custom_trace.clone();
                                Callback::from(move |_| cb.emit(()))
                            }
                        >
                            {"Aplicar"}
                        </button>
                        <button
                            class="btn btn-xs btn-cyan"
                            type="button"
                            disabled={!has_pending || props.is_playing}
                            onclick={
                                let cb = props.on_step.clone();
                                Callback::from(move |_| cb.emit(()))
                            }
                        >
                            <StepIcon />
                            <span>{"Step"}</span>
                        </button>
                        <button
                            class={classes!("btn", "btn-xs", if props.is_playing { "btn-amber" } else { "btn-emerald" })}
                            type="button"
                            disabled={!has_pending && !props.is_playing}
                            onclick={
                                let cb = props.on_toggle_play.clone();
                                Callback::from(move |_| cb.emit(()))
                            }
                        >
                            {if props.is_playing { html! { <><PauseIcon /><span>{"Pausa"}</span></> } } else { html! { <><PlayIcon /><span>{"Play"}</span></> } }}
                        </button>
                        <button
                            class="btn btn-xs btn-indigo"
                            type="button"
                            disabled={!has_pending}
                            onclick={
                                let cb = props.on_run_all.clone();
                                Callback::from(move |_| cb.emit(()))
                            }
                        >
                            {"Todo"}
                        </button>
                        <button
                            class="btn btn-xs btn-rose"
                            type="button"
                            onclick={
                                let cb = props.on_reset.clone();
                                Callback::from(move |_| cb.emit(()))
                            }
                        >
                            <ResetIcon />
                        </button>
                        <span class="badge mono">{format!("{}/{}", props.cursor, props.trace.len())}</span>
                    </div>
                </div>

                { if let Some(err) = &props.trace_error {
                    html! { <div class="error-text">{err}</div> }
                } else { html! {} }}

                <div class="queue-strip cmp-queue-line">
                    { for props.trace.iter().enumerate().map(|(idx, instr)| {
                        let is_done = idx < props.cursor;
                        let is_curr = idx == props.cursor;
                        let tag = match &instr.op {
                            MemOp::Read => format!("R {}", format_hex(instr.addr)),
                            MemOp::Write(v) => format!("W {}={}", format_hex(instr.addr), v),
                        };
                        html! {
                            <span class={classes!("q-pill", is_done.then_some("done"), is_curr.then_some("current"))}>
                                {tag}
                            </span>
                        }
                    }) }
                </div>
            </section>

            // Live Scoreboard Strip (GPU scaleX transform)
            <section class="panel compact-panel cmp-bars-panel">
                <div class="cmp-bars-header">
                    <h2 class="panel-title">{"Comparativa en Vivo"}</h2>
                    <span class="badge badge-emerald">{winner_banner}</span>
                </div>

                <div class="delta-bars">
                    <div class="bar-pair">
                        <div class="mono bar-meta">
                            <span>{"Hit Rate (A vs B)"}</span>
                            <span>{format!("{:.1}% vs {:.1}%", sa.hit_rate(), sb.hit_rate())}</span>
                        </div>
                        <div class="bar-track">
                            <div class="bar-fill fill-amber" style={format!("transform: scaleX({:.4});", scale_hit_a)}></div>
                        </div>
                        <div class="bar-track">
                            <div class="bar-fill fill-emerald" style={format!("transform: scaleX({:.4});", scale_hit_b)}></div>
                        </div>
                    </div>

                    <div class="bar-pair">
                        <div class="mono bar-meta">
                            <span>{"AMAT (menor = mejor)"}</span>
                            <span>{format!("{:.1} ns vs {:.1} ns", sa.amat_ns(), sb.amat_ns())}</span>
                        </div>
                        <div class="bar-track">
                            <div class="bar-fill fill-rose" style={format!("transform: scaleX({:.4});", scale_amat_a)}></div>
                        </div>
                        <div class="bar-track">
                            <div class="bar-fill fill-cyan" style={format!("transform: scaleX({:.4});", scale_amat_b)}></div>
                        </div>
                    </div>
                </div>
            </section>

            // Split System A vs System B
            <div class="compare-grid">
                <SystemColumn
                    title="Sistema A"
                    accent_class="badge-amber"
                    engine={props.engine_a.clone()}
                    eviction={props.eviction_a.clone()}
                    on_config={props.on_config_a.clone()}
                />
                <SystemColumn
                    title="Sistema B"
                    accent_class="badge-emerald"
                    engine={props.engine_b.clone()}
                    eviction={props.eviction_b.clone()}
                    on_config={props.on_config_b.clone()}
                />
            </div>
        </div>
    }
}
