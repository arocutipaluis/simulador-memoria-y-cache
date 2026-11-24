use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

use crate::components::icons::{PauseIcon, PlayIcon, ResetIcon, StepIcon};
use crate::components::step_controller::parse_address_str;
use crate::sim::engine::CacheEngine;
use crate::sim::types::{
    format_hex, CacheConfig, MappingPolicy, MemInstruction, MemOp, PresetKind, ReplacementPolicy,
    SlotFlash, WritePolicy,
};

#[derive(Properties, PartialEq)]
pub struct ComparatorViewProps {
    pub engine_a: CacheEngine,
    pub engine_b: CacheEngine,
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
    pub on_config: Callback<CacheConfig>,
}

#[function_component(SystemColumn)]
fn system_column(props: &SystemColumnProps) -> Html {
    let cfg = props.engine.config;
    let s = &props.engine.stats;
    let ways = cfg.effective_ways();

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

    html! {
        <section class="panel">
            <div class="panel-header">
                <h3 class="panel-title">{props.title}</h3>
                <span class={classes!("badge", props.accent_class)}>
                    {format!("{} | {}x{} ({} B)", cfg.mapping.label(), cfg.num_blocks, cfg.block_size, cfg.capacity_bytes())}
                </span>
            </div>

            <div style="display:grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap:0.45rem; margin-bottom:0.7rem;">
                <div class="field-group">
                    <label class="field-label">{"Mapeo"}</label>
                    <select
                        class="select"
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
                        <option value="direct" selected={mapping_val == "direct"}>{"Mapeo Directo"}</option>
                        <option value="set2" selected={mapping_val == "set2"}>{"Asociativo 2 vias"}</option>
                        <option value="set4" selected={mapping_val == "set4"}>{"Asociativo 4 vias"}</option>
                        <option value="full" selected={mapping_val == "full"}>{"Totalmente Asociativo"}</option>
                    </select>
                </div>

                <div class="field-group">
                    <label class="field-label">{"Bloques x Palabras"}</label>
                    <select
                        class="select"
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
                        <option value="4x2" selected={cfg.num_blocks == 4 && cfg.block_size == 2}>{"4 Blq x 2 Pal (32 B)"}</option>
                        <option value="8x1" selected={cfg.num_blocks == 8 && cfg.block_size == 1}>{"8 Blq x 1 Pal (32 B)"}</option>
                        <option value="8x2" selected={cfg.num_blocks == 8 && cfg.block_size == 2}>{"8 Blq x 2 Pal (64 B)"}</option>
                        <option value="8x4" selected={cfg.num_blocks == 8 && cfg.block_size == 4}>{"8 Blq x 4 Pal (128 B)"}</option>
                    </select>
                </div>

                <div class="field-group">
                    <label class="field-label">{"Reemplazo"}</label>
                    <select
                        class="select"
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
                        class="select"
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
                        <option value="wt" selected={wp_val == "wt"}>{"Write-Through"}</option>
                    </select>
                </div>
            </div>

            // Metricas compactas por sistema
            <div style="display:grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap:0.4rem; margin-bottom:0.7rem;">
                <div class="metric-card hit" style="padding:0.45rem;">
                    <div class="metric-label">{"Hit Rate"}</div>
                    <div class="metric-val" style="font-size:1rem; color:var(--accent-emerald);">{format!("{:.1}%", s.hit_rate())}</div>
                </div>
                <div class="metric-card miss" style="padding:0.45rem;">
                    <div class="metric-label">{"Miss Rate"}</div>
                    <div class="metric-val" style="font-size:1rem; color:var(--accent-rose);">{format!("{:.1}%", s.miss_rate())}</div>
                </div>
                <div class="metric-card amat" style="padding:0.45rem;">
                    <div class="metric-label">{"AMAT"}</div>
                    <div class="metric-val" style="font-size:1rem; color:var(--accent-cyan);">{format!("{:.1} ns", s.amat_ns())}</div>
                </div>
                <div class="metric-card wb" style="padding:0.45rem;">
                    <div class="metric-label">{"Conf / WB"}</div>
                    <div class="metric-val" style="font-size:1rem; color:var(--accent-amber);">{format!("{}/{}", s.conflict_misses, s.write_backs)}</div>
                </div>
            </div>

            // Estado visual de L1 en este sistema
            <div class="sets-list">
                { for props.engine.sets.iter().enumerate().map(|(set_idx, set_slots)| {
                    let cols_style = format!("grid-template-columns: repeat({}, minmax(0, 1fr));", ways.min(4));
                    html! {
                        <div class="set-row">
                            <div class="set-label">
                                <span>{format!("S{}", set_idx)}</span>
                            </div>
                            <div class="ways-grid" style={cols_style}>
                                { for set_slots.iter().enumerate().map(|(w_idx, opt_line)| {
                                    match opt_line {
                                        None => html! {
                                            <div class="way-slot empty" style="min-height:38px;">
                                                <span class="mono" style="font-size:0.65rem; color:var(--text-muted);">{format!("V{}: -", w_idx)}</span>
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
                                                <div class={classes!("way-slot", flash_cls)} style="padding:0.32rem 0.45rem;">
                                                    <div class="way-meta">
                                                        <span style="color:var(--accent-indigo); font-weight:700;">{format!("B#{} (T:0x{:X})", line.block_addr, line.tag)}</span>
                                                        { if line.dirty {
                                                            html! { <span class="flag-pill flag-d-on">{"D"}</span> }
                                                        } else {
                                                            html! {}
                                                        }}
                                                    </div>
                                                    <div class="mono" style="font-size:0.64rem; color:var(--text-secondary);">
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
        PresetKind::Thrashing,
        PresetKind::Sequential,
        PresetKind::Stride4,
        PresetKind::Spatial,
        PresetKind::Temporal,
        PresetKind::WriteHeavy,
    ];

    let hit_diff = sb.hit_rate() - sa.hit_rate();
    let amat_diff = sa.amat_ns() - sb.amat_ns();

    let winner_banner = if sa.accesses == 0 {
        "Ejecuta la secuencia compartida para comparar Sistema A vs Sistema B en tiempo real.".to_string()
    } else if hit_diff.abs() < 0.05 && amat_diff.abs() < 0.05 {
        "Empate tecnico en Hit Rate y AMAT con esta secuencia.".to_string()
    } else if hit_diff > 0.0 || amat_diff > 0.0 {
        format!(
            "Sistema B lidera: {:+.1} pts en Hit Rate | {:.1} ns menos de AMAT.",
            hit_diff, amat_diff
        )
    } else {
        format!(
            "Sistema A lidera: {:+.1} pts en Hit Rate | {:.1} ns menos de AMAT.",
            -hit_diff, -amat_diff
        )
    };

    let max_amat = sa.amat_ns().max(sb.amat_ns()).max(1.0);

    html! {
        <div class="stack-col">
            // Barra de control compartida de la traza A/B
            <section class="panel">
                <div class="panel-header">
                    <h2 class="panel-title">{"Traza Compartida (Ejecucion Sincronizada A/B)"}</h2>
                    <span class="badge badge-cyan">{format!("Paso {}/{}", props.cursor, props.trace.len())}</span>
                </div>

                <div style="display:flex; flex-wrap:wrap; gap:0.4rem; margin-bottom:0.6rem;">
                    { for presets.into_iter().map(|p| {
                        let cb = props.on_load_preset.clone();
                        html! {
                            <button class="btn btn-sm" type="button" onclick={Callback::from(move |_| cb.emit(p))}>
                                {p.label()}
                            </button>
                        }
                    }) }
                </div>

                <div style="display:flex; flex-wrap:wrap; gap:0.5rem; margin-bottom:0.6rem;">
                    <input
                        class="input"
                        style="flex:1 1 280px;"
                        type="text"
                        placeholder="Ej: 0x00, 0x20, 0x01, 0x21, W:0x02=X9"
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
                        class="btn btn-sm"
                        type="button"
                        onclick={
                            let cb = props.on_apply_custom_trace.clone();
                            Callback::from(move |_| cb.emit(()))
                        }
                    >
                        {"Cargar Traza Manual"}
                    </button>
                    <button
                        class="btn btn-sm btn-cyan"
                        type="button"
                        disabled={!has_pending || props.is_playing}
                        onclick={
                            let cb = props.on_step.clone();
                            Callback::from(move |_| cb.emit(()))
                        }
                    >
                        <StepIcon />
                        <span>{"Step A/B"}</span>
                    </button>
                    <button
                        class={classes!("btn", "btn-sm", if props.is_playing { "btn-amber" } else { "btn-emerald" })}
                        type="button"
                        disabled={!has_pending && !props.is_playing}
                        onclick={
                            let cb = props.on_toggle_play.clone();
                            Callback::from(move |_| cb.emit(()))
                        }
                    >
                        {if props.is_playing { html! { <><PauseIcon /><span>{"Pausa"}</span></> } } else { html! { <><PlayIcon /><span>{"Auto"}</span></> } }}
                    </button>
                    <button
                        class="btn btn-sm btn-indigo"
                        type="button"
                        disabled={!has_pending}
                        onclick={
                            let cb = props.on_run_all.clone();
                            Callback::from(move |_| cb.emit(()))
                        }
                    >
                        {"Ejecutar Todo"}
                    </button>
                    <button
                        class="btn btn-sm btn-rose"
                        type="button"
                        onclick={
                            let cb = props.on_reset.clone();
                            Callback::from(move |_| cb.emit(()))
                        }
                    >
                        <ResetIcon />
                        <span>{"Reset"}</span>
                    </button>
                </div>

                { if let Some(err) = &props.trace_error {
                    html! { <div style="font-size:0.74rem; color:var(--accent-rose); margin-bottom:0.45rem;">{err}</div> }
                } else {
                    html! {}
                }}

                <div class="queue-strip">
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

            // Comparativa visual de barras Hit Rate y AMAT
            <section class="panel">
                <div class="panel-header">
                    <h2 class="panel-title">{"Resultado Comparativo en Vivo"}</h2>
                    <span class="badge badge-emerald">{winner_banner}</span>
                </div>

                <div class="delta-bars">
                    <div>
                        <div class="mono" style="font-size:0.74rem; display:flex; justify-content:space-between;">
                            <span>{"Hit Rate Sistema A vs B"}</span>
                            <span>{format!("{:.1}% vs {:.1}%", sa.hit_rate(), sb.hit_rate())}</span>
                        </div>
                        <div class="bar-track">
                            <div class="bar-fill" style={format!("width:{:.1}%; background:var(--accent-amber);", sa.hit_rate())}></div>
                        </div>
                        <div class="bar-track">
                            <div class="bar-fill" style={format!("width:{:.1}%; background:var(--accent-emerald);", sb.hit_rate())}></div>
                        </div>
                    </div>

                    <div>
                        <div class="mono" style="font-size:0.74rem; display:flex; justify-content:space-between;">
                            <span>{"AMAT Sistema A vs B (menor es mejor)"}</span>
                            <span>{format!("{:.1} ns vs {:.1} ns", sa.amat_ns(), sb.amat_ns())}</span>
                        </div>
                        <div class="bar-track">
                            <div class="bar-fill" style={format!("width:{:.1}%; background:var(--accent-rose);", (sa.amat_ns() / max_amat) * 100.0)}></div>
                        </div>
                        <div class="bar-track">
                            <div class="bar-fill" style={format!("width:{:.1}%; background:var(--accent-cyan);", (sb.amat_ns() / max_amat) * 100.0)}></div>
                        </div>
                    </div>
                </div>
            </section>

            // Dos sistemas lado a lado
            <div class="compare-grid">
                <SystemColumn
                    title="Sistema A"
                    accent_class="badge-amber"
                    engine={props.engine_a.clone()}
                    on_config={props.on_config_a.clone()}
                />
                <SystemColumn
                    title="Sistema B"
                    accent_class="badge-emerald"
                    engine={props.engine_b.clone()}
                    on_config={props.on_config_b.clone()}
                />
            </div>
        </div>
    }
}
