use yew::prelude::*;

use crate::sim::types::{format_hex, HitLevel, RamCell};

#[derive(Properties, PartialEq)]
pub struct RamViewProps {
    pub ram: Vec<RamCell>,
    pub selected_addr: u8,
    pub last_hit_level: Option<HitLevel>,
    pub on_select_addr: Callback<u8>,
    pub on_page_fault: Callback<()>,
}

#[function_component(RamView)]
pub fn ram_view(props: &RamViewProps) -> Html {
    html! {
        <div class="stack-col">
            <section class="panel">
                <div class="panel-header">
                    <h2 class="panel-title">{"Nivel 4 • Memoria Principal (DRAM) — 64 Palabras (0x00 - 0x3F)"}</h2>
                    <div style="display:flex; gap:0.4rem; flex-wrap:wrap;">
                        <span class="badge badge-emerald">{"En Cache"}</span>
                        <span class="badge badge-amber">{"Dirty (WB Pendiente)"}</span>
                        <span class={classes!("badge", (props.last_hit_level == Some(HitLevel::Ram)).then_some("badge-rose"))}>
                            {"60.0 ns"}
                        </span>
                    </div>
                </div>

                <div class="ram-grid">
                    { for props.ram.iter().map(|cell| {
                        let addr = cell.addr;
                        let is_selected = addr == props.selected_addr;
                        let cb = props.on_select_addr.clone();
                        html! {
                            <button
                                type="button"
                                class={classes!(
                                    "ram-cell",
                                    is_selected.then_some("selected"),
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

            <section class="panel" style="display:flex; flex-wrap:wrap; align-items:center; justify-content:space-between; gap:0.75rem;">
                <div>
                    <h2 class="panel-title">{"Nivel 5 • Almacenamiento Secundario (SSD / Disco)"}</h2>
                    <p style="margin:0.25rem 0 0; font-size:0.75rem; color:var(--text-secondary);">
                        {"Latencia extrema de 10,000 ns por Fallo de Pagina (Page Fault). Observa el disparo del AMAT."}
                    </p>
                </div>
                <button
                    class="btn btn-rose btn-sm"
                    type="button"
                    onclick={
                        let cb = props.on_page_fault.clone();
                        Callback::from(move |_| cb.emit(()))
                    }
                >
                    {"Simular Page Fault (+10,000 ns)"}
                </button>
            </section>
        </div>
    }
}
