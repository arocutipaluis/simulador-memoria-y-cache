use yew::prelude::*;

use crate::sim::types::{format_hex, LogEntry, SimStats};

#[derive(Properties, PartialEq)]
pub struct MetricsPanelProps {
    pub stats: SimStats,
    pub feedback: String,
    pub logs: Vec<LogEntry>,
    pub on_clear_log: Callback<()>,
}

#[function_component(MetricsPanel)]
pub fn metrics_panel(props: &MetricsPanelProps) -> Html {
    let s = &props.stats;

    html! {
        <div class="stack-col">
            <section class="metrics-grid" aria-label="Telemetria de rendimiento">
                <div class="metric-card">
                    <div class="metric-label">{"Total Accesos"}</div>
                    <div class="metric-val">{s.accesses}</div>
                    <div class="metric-sub">{format!("Escrituras: {}", s.writes_count)}</div>
                </div>

                <div class="metric-card hit">
                    <div class="metric-label" style="color:var(--accent-emerald);">{"Cache Hits"}</div>
                    <div class="metric-val" style="color:var(--accent-emerald);">{s.hits}</div>
                    <div class="metric-sub">{format!("L1:{} L2:{} L3:{}", s.l1_hits, s.l2_hits, s.l3_hits)}</div>
                </div>

                <div class="metric-card miss">
                    <div class="metric-label" style="color:var(--accent-rose);">{"Cache Misses"}</div>
                    <div class="metric-val" style="color:var(--accent-rose);">{s.misses}</div>
                    <div class="metric-sub">{format!("Comp:{} Conf:{}", s.compulsory_misses, s.conflict_misses)}</div>
                </div>

                <div class="metric-card hit">
                    <div class="metric-label">{"L1 Hit Rate"}</div>
                    <div class="metric-val" style="color:var(--accent-cyan);">{format!("{:.1}%", s.l1_hit_rate())}</div>
                    <div class="metric-sub">{format!("Global L1-L3: {:.1}%", s.hit_rate())}</div>
                </div>

                <div class="metric-card miss">
                    <div class="metric-label">{"Miss Rate"}</div>
                    <div class="metric-val" style="color:var(--accent-rose);">{format!("{:.1}%", s.miss_rate())}</div>
                    <div class="metric-sub">{"Accesos a RAM/SSD"}</div>
                </div>

                <div class="metric-card amat">
                    <div class="metric-label">{"AMAT Promedio"}</div>
                    <div class="metric-val" style="color:var(--accent-indigo);">{format!("{:.1} ns", s.amat_ns())}</div>
                    <div class="metric-sub">{format!("Acum: {:.0} ns", s.total_latency_ns)}</div>
                </div>

                <div class="metric-card wb">
                    <div class="metric-label">{"Evicts / Write-Back"}</div>
                    <div class="metric-val" style="color:var(--accent-amber);">{format!("{}/{}", s.evictions, s.write_backs)}</div>
                    <div class="metric-sub">{"Desalojos / Volcados"}</div>
                </div>
            </section>

            <section class="panel">
                <div class="panel-header">
                    <h2 class="panel-title">{"Diagnostico y Registro de Trazas"}</h2>
                    <button
                        class="btn btn-sm"
                        type="button"
                        onclick={
                            let cb = props.on_clear_log.clone();
                            Callback::from(move |_| cb.emit(()))
                        }
                    >
                        {"Limpiar Log"}
                    </button>
                </div>

                <div class="feedback-box" style="margin-bottom:0.65rem;">
                    {&props.feedback}
                </div>

                <div class="log-list">
                    { if props.logs.is_empty() {
                        html! { <div class="mono" style="font-size:0.72rem; color:var(--text-muted);">{"Sin accesos registrados todavia."}</div> }
                    } else {
                        html! { for props.logs.iter().map(|entry| {
                            let tag_kind = entry.miss_kind.as_deref().unwrap_or("HIT");
                            let wb_txt = if entry.write_back { " +WB" } else { "" };
                            html! {
                                <div class={classes!("log-item", if entry.is_hit { "hit" } else { "miss" })}>
                                    <span>{format!("#{:02} {} {} (Set {}, Tag 0x{:X})", entry.step, entry.op_name, format_hex(entry.addr), entry.set_idx, entry.tag)}</span>
                                    <span>{format!("{} [{}]{} • {:.1} ns", entry.level.label(), tag_kind, wb_txt, entry.latency_ns)}</span>
                                </div>
                            }
                        }) }
                    }}
                </div>
            </section>
        </div>
    }
}
