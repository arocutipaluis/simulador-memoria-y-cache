use yew::prelude::*;

use crate::sim::types::{format_hex, HitLevel, LogEntry, RamCell};

#[derive(Properties, PartialEq)]
pub struct RamViewProps {
    pub ram: Vec<RamCell>,
    pub selected_addr: u8,
    pub last_hit_level: Option<HitLevel>,
    pub clock: u64,
    pub feedback: String,
    pub logs: Vec<LogEntry>,
    pub on_select_addr: Callback<u8>,
    pub on_page_fault: Callback<()>,
    pub on_clear_log: Callback<()>,
}

#[function_component(RamView)]
pub fn ram_view(props: &RamViewProps) -> Html {
    let clean_feedback = props
        .feedback
        .replace(" en L2 en L2", " en L2")
        .replace(" en L3 en L3", " en L3")
        .replace(" en RAM en RAM", " en RAM");

    html! {
        <div class="right-workbench-col">
            <section class="panel compact-panel ram-panel" aria-label="Nivel 4 Memoria Principal DRAM">
                <div class="panel-header compact-header">
                    <h2 class="panel-title">{"Nivel 4 · DRAM (0x00-0x3F)"}</h2>
                    <div class="legend-inline">
                        <span class="badge badge-emerald">{"L1"}</span>
                        <span class="badge badge-amber">{"Dirty"}</span>
                        <span class={classes!("badge", (props.last_hit_level == Some(HitLevel::Ram)).then_some("badge-rose"))}>
                            {"60 ns"}
                        </span>
                    </div>
                </div>

                <div class="ram-grid">
                    { for props.ram.iter().map(|cell| {
                        let addr = cell.addr;
                        let is_selected = addr == props.selected_addr;
                        let is_active = cell.last_accessed && props.clock > 0;
                        let cb = props.on_select_addr.clone();
                        html! {
                            <button
                                type="button"
                                class={classes!(
                                    "ram-cell",
                                    is_selected.then_some("selected"),
                                    is_active.then_some("just-accessed"),
                                    cell.cached_in_l1.then_some("in-cache"),
                                    cell.dirty_in_cache.then_some("dirty-ram")
                                )}
                                onclick={Callback::from(move |_| cb.emit(addr))}
                                title={format!("Seleccionar {}", format_hex(addr))}
                            >
                                <div class="ram-addr">
                                    <span>{format_hex(addr)}</span>
                                    <span>{if cell.dirty_in_cache { "D" } else if cell.cached_in_l1 { "C" } else { "" }}</span>
                                </div>
                                <div class="ram-val">{&cell.value}</div>
                            </button>
                        }
                    }) }
                </div>
            </section>

            <section class="panel compact-panel ssd-bar" aria-label="Nivel 5 SSD Page Fault">
                <div class="ssd-copy">
                    <h2 class="panel-title">{"Nivel 5 · SSD"}</h2>
                    <span class="mono muted-xs">{"Page Fault +10,000 ns"}</span>
                </div>
                <button
                    class="btn btn-rose btn-xs"
                    type="button"
                    onclick={
                        let cb = props.on_page_fault.clone();
                        Callback::from(move |_| cb.emit(()))
                    }
                >
                    {"Simular Page Fault"}
                </button>
            </section>

            <section class="panel compact-panel log-panel" aria-label="Registro de Eventos">
                <div class="panel-header compact-header">
                    <h2 class="panel-title">{"Diagnostico & Trazas"}</h2>
                    <button
                        class="btn btn-xs"
                        type="button"
                        onclick={
                            let cb = props.on_clear_log.clone();
                            Callback::from(move |_| cb.emit(()))
                        }
                    >
                        {"Limpiar"}
                    </button>
                </div>

                <div class="feedback-box">{clean_feedback}</div>

                <div class="log-list">
                    { if props.logs.is_empty() {
                        html! { <div class="mono muted-xs">{"Sin accesos registrados todavia."}</div> }
                    } else {
                        html! { for props.logs.iter().map(|entry| {
                            let tag_kind = entry.miss_kind.as_deref().unwrap_or("HIT");
                            let wb_txt = if entry.write_back { " +WB" } else { "" };
                            html! {
                                <div class={classes!("log-item", if entry.is_hit { "hit" } else { "miss" })}>
                                    <span class="log-left">{format!("#{:02} {} {} (S{}, T:{:X})", entry.step, entry.op_name, format_hex(entry.addr), entry.set_idx, entry.tag)}</span>
                                    <span class="log-right">{format!("{} [{}]{} · {:.0}ns", entry.level.label(), tag_kind, wb_txt, entry.latency_ns)}</span>
                                </div>
                            }
                        }) }
                    }}
                </div>
            </section>
        </div>
    }
}
