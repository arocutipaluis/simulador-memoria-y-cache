use yew::prelude::*;

use crate::sim::types::SimStats;

#[derive(Properties, PartialEq)]
pub struct MetricsPanelProps {
    pub stats: SimStats,
}

#[function_component(MetricsPanel)]
pub fn metrics_panel(props: &MetricsPanelProps) -> Html {
    let s = &props.stats;

    html! {
        <section class="metrics-strip" aria-label="Telemetria de rendimiento">
            <div class="kpi-tile">
                <span class="kpi-label">{"Accesos"}</span>
                <span class="kpi-val mono">{s.accesses}</span>
                <span class="kpi-sub mono">{format!("W:{}", s.writes_count)}</span>
            </div>

            <div class="kpi-tile hit">
                <span class="kpi-label">{"Hits"}</span>
                <span class="kpi-val mono text-emerald">{s.hits}</span>
                <span class="kpi-sub mono">{format!("L1:{} L2:{} L3:{}", s.l1_hits, s.l2_hits, s.l3_hits)}</span>
            </div>

            <div class="kpi-tile miss">
                <span class="kpi-label">{"Misses"}</span>
                <span class="kpi-val mono text-rose">{s.misses}</span>
                <span class="kpi-sub mono">{format!("C:{} X:{}", s.compulsory_misses, s.conflict_misses)}</span>
            </div>

            <div class="kpi-tile">
                <span class="kpi-label">{"L1 Hit Rate"}</span>
                <span class="kpi-val mono text-cyan">{format!("{:.1}%", s.l1_hit_rate())}</span>
                <span class="kpi-sub mono">{format!("Glob {:.1}%", s.hit_rate())}</span>
            </div>

            <div class="kpi-tile miss">
                <span class="kpi-label">{"Miss Rate"}</span>
                <span class="kpi-val mono text-rose">{format!("{:.1}%", s.miss_rate())}</span>
                <span class="kpi-sub mono">{format!("RAM:{} SSD:{}", s.ram_accesses, s.disk_faults)}</span>
            </div>

            <div class="kpi-tile">
                <span class="kpi-label">{"AMAT"}</span>
                <span class="kpi-val mono text-indigo">{format!("{:.1} ns", s.amat_ns())}</span>
                <span class="kpi-sub mono">{format!("{:.0} ns", s.total_latency_ns)}</span>
            </div>

            <div class="kpi-tile wb">
                <span class="kpi-label">{"Evict / WB"}</span>
                <span class="kpi-val mono text-amber">{format!("{}/{}", s.evictions, s.write_backs)}</span>
                <span class="kpi-sub mono">{"Desalojos"}</span>
            </div>
        </section>
    }
}
