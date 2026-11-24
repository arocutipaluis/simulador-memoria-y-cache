use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

use crate::components::icons::{PauseIcon, PlayIcon, ResetIcon, StepIcon};
use crate::sim::types::{
    format_hex, CacheConfig, MappingPolicy, MemInstruction, MemOp, PresetKind, ReplacementPolicy,
    WritePolicy, RAM_SIZE,
};

#[derive(Properties, PartialEq)]
pub struct StepControllerProps {
    pub selected_addr: u8,
    pub custom_addr_input: String,
    pub addr_error: Option<String>,
    pub write_value: String,
    pub config: CacheConfig,
    pub queue: Vec<MemInstruction>,
    pub queue_cursor: usize,
    pub is_playing: bool,
    pub speed_ms: u32,
    pub on_select_addr: Callback<u8>,
    pub on_custom_addr_change: Callback<String>,
    pub on_write_val_change: Callback<String>,
    pub on_execute_single: Callback<MemOp>,
    pub on_enqueue_single: Callback<MemOp>,
    pub on_load_preset: Callback<PresetKind>,
    pub on_step_queue: Callback<()>,
    pub on_toggle_play: Callback<()>,
    pub on_clear_queue: Callback<()>,
    pub on_reset_all: Callback<()>,
    pub on_speed_change: Callback<u32>,
    pub on_config_change: Callback<CacheConfig>,
}

pub fn parse_address_str(raw: &str) -> Result<u8, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("Ingresa una direccion entre 0x00 y 0x3F (0..63).".to_string());
    }
    let val = if let Some(hex_part) = trimmed.strip_prefix("0x").or_else(|| trimmed.strip_prefix("0X")) {
        u16::from_str_radix(hex_part, 16)
            .map_err(|_| "Hexadecimal invalido. Ejemplo valido: 0x1A.".to_string())?
    } else {
        trimmed
            .parse::<u16>()
            .map_err(|_| "Numero invalido. Usa decimal (0..63) o hex (0x00..0x3F).".to_string())?
    };
    if val >= RAM_SIZE as u16 {
        return Err(format!("Fuera de rango. Usa 0x00 a 0x{:02X} (0..{}).", RAM_SIZE - 1, RAM_SIZE - 1));
    }
    Ok(val as u8)
}

#[function_component(StepController)]
pub fn step_controller(props: &StepControllerProps) -> Html {
    let cfg = props.config;

    let presets = [
        PresetKind::Sequential,
        PresetKind::Temporal,
        PresetKind::Spatial,
        PresetKind::Stride4,
        PresetKind::Thrashing,
        PresetKind::WriteHeavy,
    ];

    let on_select_change = {
        let cb = props.on_select_addr.clone();
        Callback::from(move |e: Event| {
            let sel: HtmlSelectElement = e.target_unchecked_into();
            if let Ok(v) = sel.value().parse::<u8>() {
                cb.emit(v);
            }
        })
    };

    let on_custom_input = {
        let cb = props.on_custom_addr_change.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            cb.emit(input.value());
        })
    };

    let on_write_input = {
        let cb = props.on_write_val_change.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            cb.emit(input.value());
        })
    };

    let on_mapping_change = {
        let cb = props.on_config_change.clone();
        Callback::from(move |e: Event| {
            let sel: HtmlSelectElement = e.target_unchecked_into();
            let mut next = cfg;
            next.mapping = match sel.value().as_str() {
                "direct" => MappingPolicy::DirectMapped,
                "set2" => MappingPolicy::SetAssociative(2),
                "set4" => MappingPolicy::SetAssociative(4),
                "full" => MappingPolicy::FullyAssociative,
                _ => MappingPolicy::DirectMapped,
            };
            cb.emit(next);
        })
    };

    let on_blocks_change = {
        let cb = props.on_config_change.clone();
        Callback::from(move |e: Event| {
            let sel: HtmlSelectElement = e.target_unchecked_into();
            let mut next = cfg;
            if let Ok(v) = sel.value().parse::<usize>() {
                next.num_blocks = v;
            }
            cb.emit(next);
        })
    };

    let on_block_size_change = {
        let cb = props.on_config_change.clone();
        Callback::from(move |e: Event| {
            let sel: HtmlSelectElement = e.target_unchecked_into();
            let mut next = cfg;
            if let Ok(v) = sel.value().parse::<usize>() {
                next.block_size = v;
            }
            cb.emit(next);
        })
    };

    let on_replacement_change = {
        let cb = props.on_config_change.clone();
        Callback::from(move |e: Event| {
            let sel: HtmlSelectElement = e.target_unchecked_into();
            let mut next = cfg;
            next.replacement = match sel.value().as_str() {
                "lru" => ReplacementPolicy::Lru,
                "fifo" => ReplacementPolicy::Fifo,
                "random" => ReplacementPolicy::Random,
                _ => ReplacementPolicy::Lru,
            };
            cb.emit(next);
        })
    };

    let on_write_policy_change = {
        let cb = props.on_config_change.clone();
        Callback::from(move |e: Event| {
            let sel: HtmlSelectElement = e.target_unchecked_into();
            let mut next = cfg;
            next.write_policy = match sel.value().as_str() {
                "wt" => WritePolicy::WriteThrough,
                "wb" => WritePolicy::WriteBack,
                _ => WritePolicy::WriteBack,
            };
            cb.emit(next);
        })
    };

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

    let has_pending_queue = props.queue_cursor < props.queue.len();

    html! {
        <div class="stack-col">
            // Unidad de Control CPU
            <section class="panel">
                <div class="panel-header">
                    <h2 class="panel-title">{"Unidad de Control CPU"}</h2>
                    <span class="badge badge-cyan">{"Bus 6-bit (0x00-0x3F)"}</span>
                </div>

                <div class="stack-col" style="gap:0.6rem;">
                    <div class="addr-select-grid">
                        <div class="field-group">
                            <label class="field-label" for="addr-select">{"Selector Direccion"}</label>
                            <select id="addr-select" class="select" onchange={on_select_change}>
                                { for (0..RAM_SIZE as u8).map(|a| {
                                    let blk = (a as usize) / cfg.block_size;
                                    html! {
                                        <option value={a.to_string()} selected={props.selected_addr == a}>
                                            {format!("{} (Dec {:02}) - Blq #{}", format_hex(a), a, blk)}
                                        </option>
                                    }
                                }) }
                            </select>
                        </div>

                        <div class="field-group">
                            <label class="field-label" for="addr-custom">{"Manual (Hex/Dec)"}</label>
                            <input
                                id="addr-custom"
                                class="input"
                                type="text"
                                placeholder="0x1A o 26"
                                value={props.custom_addr_input.clone()}
                                oninput={on_custom_input}
                            />
                        </div>
                    </div>

                    { if let Some(err) = &props.addr_error {
                        html! { <div style="font-size:0.72rem; color:var(--accent-rose);">{err}</div> }
                    } else {
                        html! {}
                    }}

                    <div class="field-group">
                        <label class="field-label" for="write-val">{"Dato para STORE (max 7 chars)"}</label>
                        <input
                            id="write-val"
                            class="input"
                            type="text"
                            maxlength="7"
                            value={props.write_value.clone()}
                            oninput={on_write_input}
                        />
                    </div>

                    <div style="display:grid; grid-template-columns: 1fr 1fr; gap:0.5rem;">
                        <button
                            class="btn btn-cyan"
                            type="button"
                            onclick={
                                let cb = props.on_execute_single.clone();
                                Callback::from(move |_| cb.emit(MemOp::Read))
                            }
                        >
                            {"LEER (LOAD)"}
                        </button>
                        <button
                            class="btn btn-amber"
                            type="button"
                            onclick={
                                let cb = props.on_execute_single.clone();
                                let wv = props.write_value.clone();
                                Callback::from(move |_| {
                                    let clean: String = wv.trim().chars().filter(|c| c.is_ascii_alphanumeric() || *c == '_').take(7).collect();
                                    let val = if clean.is_empty() { "DAT_99".to_string() } else { clean };
                                    cb.emit(MemOp::Write(val));
                                })
                            }
                        >
                            {"ESCRIBIR (STORE)"}
                        </button>
                    </div>

                    <div style="display:grid; grid-template-columns: 1fr 1fr; gap:0.5rem;">
                        <button
                            class="btn btn-sm"
                            type="button"
                            onclick={
                                let cb = props.on_enqueue_single.clone();
                                Callback::from(move |_| cb.emit(MemOp::Read))
                            }
                        >
                            {"+ Encolar READ"}
                        </button>
                        <button
                            class="btn btn-sm"
                            type="button"
                            onclick={
                                let cb = props.on_enqueue_single.clone();
                                let wv = props.write_value.clone();
                                Callback::from(move |_| {
                                    let clean: String = wv.trim().chars().filter(|c| c.is_ascii_alphanumeric() || *c == '_').take(7).collect();
                                    let val = if clean.is_empty() { "DAT_99".to_string() } else { clean };
                                    cb.emit(MemOp::Write(val));
                                })
                            }
                        >
                            {"+ Encolar WRITE"}
                        </button>
                    </div>
                </div>
            </section>

            // Controlador Paso a Paso y Cola de Presets
            <section class="panel">
                <div class="panel-header">
                    <h2 class="panel-title">{"Secuencias y Control Paso a Paso"}</h2>
                    <span class="badge">
                        {format!("{}/{}", props.queue_cursor, props.queue.len())}
                    </span>
                </div>

                <div class="stack-col" style="gap:0.55rem;">
                    <div style="display:grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap:0.4rem;">
                        { for presets.into_iter().map(|p| {
                            let cb = props.on_load_preset.clone();
                            html! {
                                <button
                                    class="btn btn-sm"
                                    type="button"
                                    onclick={Callback::from(move |_| cb.emit(p))}
                                >
                                    {p.label()}
                                </button>
                            }
                        }) }
                    </div>

                    <div class="queue-strip" aria-label="Cola de instrucciones">
                        { if props.queue.is_empty() {
                            html! { <span style="font-size:0.72rem; color:var(--text-muted);">{"Cola vacia. Carga un preset o encola accesos."}</span> }
                        } else {
                            html! { for props.queue.iter().enumerate().map(|(idx, instr)| {
                                let is_done = idx < props.queue_cursor;
                                let is_curr = idx == props.queue_cursor;
                                let is_write = matches!(instr.op, MemOp::Write(_));
                                let tag = match &instr.op {
                                    MemOp::Read => format!("R {}", format_hex(instr.addr)),
                                    MemOp::Write(v) => format!("W {}={}", format_hex(instr.addr), v),
                                };
                                html! {
                                    <span class={classes!(
                                        "q-pill",
                                        is_done.then_some("done"),
                                        is_curr.then_some("current"),
                                        is_write.then_some("write")
                                    )}>
                                        {tag}
                                    </span>
                                }
                            }) }
                        }}
                    </div>

                    <div class="ctrl-4-grid">
                        <button
                            class="btn btn-sm btn-cyan"
                            type="button"
                            disabled={!has_pending_queue || props.is_playing}
                            onclick={
                                let cb = props.on_step_queue.clone();
                                Callback::from(move |_| cb.emit(()))
                            }
                        >
                            <StepIcon />
                            <span>{"Step"}</span>
                        </button>
                        <button
                            class={classes!("btn", "btn-sm", if props.is_playing { "btn-amber" } else { "btn-emerald" })}
                            type="button"
                            disabled={!has_pending_queue && !props.is_playing}
                            onclick={
                                let cb = props.on_toggle_play.clone();
                                Callback::from(move |_| cb.emit(()))
                            }
                        >
                            { if props.is_playing {
                                html! { <><PauseIcon /><span>{"Pausa"}</span></> }
                            } else {
                                html! { <><PlayIcon /><span>{"Auto"}</span></> }
                            }}
                        </button>
                        <button
                            class="btn btn-sm"
                            type="button"
                            onclick={
                                let cb = props.on_clear_queue.clone();
                                Callback::from(move |_| cb.emit(()))
                            }
                        >
                            {"Vaciar"}
                        </button>
                        <button
                            class="btn btn-sm btn-rose"
                            type="button"
                            onclick={
                                let cb = props.on_reset_all.clone();
                                Callback::from(move |_| cb.emit(()))
                            }
                        >
                            <ResetIcon />
                            <span>{"Reset"}</span>
                        </button>
                    </div>

                    <div class="field-group">
                        <div style="display:flex; justify-content:space-between; font-size:0.72rem; color:var(--text-secondary);">
                            <span>{"Velocidad Auto-Play"}</span>
                            <span class="mono">{format!("{} ms/paso", props.speed_ms)}</span>
                        </div>
                        <input
                            type="range"
                            min="120"
                            max="1200"
                            step="60"
                            value={props.speed_ms.to_string()}
                            oninput={
                                let cb = props.on_speed_change.clone();
                                Callback::from(move |e: InputEvent| {
                                    let input: HtmlInputElement = e.target_unchecked_into();
                                    if let Ok(v) = input.value().parse::<u32>() {
                                        cb.emit(v);
                                    }
                                })
                            }
                        />
                    </div>
                </div>
            </section>

            // Configuracion de Arquitectura L1
            <section class="panel">
                <div class="panel-header">
                    <h2 class="panel-title">{"Arquitectura Cache L1"}</h2>
                    <span class="badge badge-indigo">
                        {format!("{} Conj x {} Vias", cfg.num_sets(), cfg.effective_ways())}
                    </span>
                </div>

                <div class="stack-col" style="gap:0.55rem;">
                    <div class="field-group">
                        <label class="field-label" for="cfg-mapping">{"Politica de Mapeo"}</label>
                        <select id="cfg-mapping" class="select" onchange={on_mapping_change} value={mapping_val}>
                            <option value="direct" selected={mapping_val == "direct"}>{"Mapeo Directo (1-way)"}</option>
                            <option value="set2" selected={mapping_val == "set2"}>{"Asociativo por Conjuntos (2-way)"}</option>
                            <option value="set4" selected={mapping_val == "set4"}>{"Asociativo por Conjuntos (4-way)"}</option>
                            <option value="full" selected={mapping_val == "full"}>{"Totalmente Asociativo"}</option>
                        </select>
                    </div>

                    <div style="display:grid; grid-template-columns: 1fr 1fr; gap:0.5rem;">
                        <div class="field-group">
                            <label class="field-label" for="cfg-blocks">{"Bloques L1"}</label>
                            <select id="cfg-blocks" class="select" onchange={on_blocks_change}>
                                <option value="2" selected={cfg.num_blocks == 2}>{"2 Bloques"}</option>
                                <option value="4" selected={cfg.num_blocks == 4}>{"4 Bloques"}</option>
                                <option value="8" selected={cfg.num_blocks == 8}>{"8 Bloques"}</option>
                            </select>
                        </div>

                        <div class="field-group">
                            <label class="field-label" for="cfg-bsize">{"Tamano Bloque"}</label>
                            <select id="cfg-bsize" class="select" onchange={on_block_size_change}>
                                <option value="1" selected={cfg.block_size == 1}>{"1 Palabra (4 B)"}</option>
                                <option value="2" selected={cfg.block_size == 2}>{"2 Palabras (8 B)"}</option>
                                <option value="4" selected={cfg.block_size == 4}>{"4 Palabras (16 B)"}</option>
                            </select>
                        </div>
                    </div>

                    <div style="display:grid; grid-template-columns: 1fr 1fr; gap:0.5rem;">
                        <div class="field-group">
                            <label class="field-label" for="cfg-repl">{"Reemplazo"}</label>
                            <select
                                id="cfg-repl"
                                class="select"
                                onchange={on_replacement_change}
                                disabled={matches!(cfg.mapping, MappingPolicy::DirectMapped)}
                            >
                                <option value="lru" selected={repl_val == "lru"}>{"LRU (Menos Reciente)"}</option>
                                <option value="fifo" selected={repl_val == "fifo"}>{"FIFO (Mas Antiguo)"}</option>
                                <option value="random" selected={repl_val == "random"}>{"Random (Aleatorio)"}</option>
                            </select>
                        </div>

                        <div class="field-group">
                            <label class="field-label" for="cfg-wp">{"Escritura"}</label>
                            <select id="cfg-wp" class="select" onchange={on_write_policy_change}>
                                <option value="wb" selected={wp_val == "wb"}>{"Write-Back (Dirty)"}</option>
                                <option value="wt" selected={wp_val == "wt"}>{"Write-Through"}</option>
                            </select>
                        </div>
                    </div>

                    <label style="display:flex; align-items:center; gap:0.5rem; font-size:0.75rem; color:var(--text-secondary); cursor:pointer;">
                        <input
                            type="checkbox"
                            checked={cfg.use_l2_l3}
                            onchange={
                                let cb = props.on_config_change.clone();
                                Callback::from(move |e: Event| {
                                    let inp: HtmlInputElement = e.target_unchecked_into();
                                    let mut next = cfg;
                                    next.use_l2_l3 = inp.checked();
                                    cb.emit(next);
                                })
                            }
                        />
                        <span>{"Habilitar caches L2 y L3 (desmarcar para modo estricto L1 -> RAM como MARS)"}</span>
                    </label>
                </div>
            </section>
        </div>
    }
}
