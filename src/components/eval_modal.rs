use yew::prelude::*;

use crate::sim::engine::CacheEngine;
use crate::sim::presets::challenges;
use crate::sim::types::{MappingPolicy, ReplacementPolicy, WritePolicy};

#[derive(Properties, PartialEq)]
pub struct EvalModalProps {
    pub show: bool,
    pub engine: CacheEngine,
    pub score_xp: u32,
    pub completed_challenges: Vec<usize>,
    pub on_close: Callback<()>,
}

#[function_component(EvalModal)]
pub fn eval_modal(props: &EvalModalProps) -> Html {
    if !props.show {
        return html! {};
    }

    let s = &props.engine.stats;
    let c = &props.engine.config;
    let total_ch = challenges().len();

    let mut mastered: Vec<String> = Vec::new();
    if s.hits > 0 {
        mastered.push("Aprovechamiento de aciertos en niveles rapidos (L1/L2/L3).".into());
    }
    if c.block_size > 1 && s.hits >= 2 {
        mastered.push("Explotacion de Localidad Espacial con bloques multipalabra.".into());
    }
    if !matches!(c.mapping, MappingPolicy::DirectMapped) && s.hits > 0 {
        mastered.push(format!("Uso de asociatividad ({}) para reducir conflictos.", c.mapping.label()));
    }
    if c.write_policy == WritePolicy::WriteBack && s.write_backs > 0 {
        mastered.push("Comprension de Dirty Bit y escritura diferida Write-Back al desalojar.".into());
    }
    if !props.completed_challenges.is_empty() {
        mastered.push(format!("Retos superados: {}/{}.", props.completed_challenges.len(), total_ch));
    }
    if mastered.is_empty() {
        mastered.push("Ejecuta algunas secuencias (Secuencial, Thrashing, WriteHeavy) para generar diagnostico.".into());
    }

    let mut recommendations: Vec<String> = Vec::new();
    if s.accesses > 0 && s.hit_rate() < 45.0 {
        recommendations.push("Tu Hit Rate es bajo (<45%). Prueba incrementar el tamano de bloque o activar Asociativo 2 vias para mitigar conflictos.".into());
    }
    if c.mapping == MappingPolicy::DirectMapped && s.conflict_misses >= 4 {
        recommendations.push("Detectados multiples fallos de conflicto en Mapeo Directo. Usa el Comparador A/B frente a Asociativo 2 vias.".into());
    }
    if c.replacement == ReplacementPolicy::Fifo && s.evictions >= 2 {
        recommendations.push("Con FIFO y desalojos frecuentes, prueba cambiar a LRU para retener bloques recientes.".into());
    }
    if c.block_size == 1 {
        recommendations.push("Un bloque de 1 palabra anula la localidad espacial; aumenta a 2 o 4 palabras.".into());
    }
    if recommendations.is_empty() {
        recommendations.push("Excelente balance de jerarquia. Experimenta comparando Write-Through vs Write-Back en el Comparador A/B.".into());
    }

    html! {
        <div class="modal-backdrop" role="dialog" aria-modal="true" aria-label="Reporte de Evaluacion">
            <div class="modal-card">
                <div class="panel-header">
                    <h2 class="panel-title">{"Reporte de Evaluacion y Telemetria"}</h2>
                    <button
                        class="btn btn-sm"
                        type="button"
                        onclick={
                            let cb = props.on_close.clone();
                            Callback::from(move |_| cb.emit(()))
                        }
                    >
                        {"Cerrar"}
                    </button>
                </div>

                <div style="display:grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap:0.6rem; margin-bottom:0.9rem;">
                    <div class="metric-card">
                        <div class="metric-label">{"Puntuacion Total"}</div>
                        <div class="metric-val" style="color:var(--accent-amber);">{format!("{} XP", props.score_xp)}</div>
                    </div>
                    <div class="metric-card hit">
                        <div class="metric-label">{"Hit Rate Global"}</div>
                        <div class="metric-val" style="color:var(--accent-cyan);">{format!("{:.1}%", s.hit_rate())}</div>
                    </div>
                    <div class="metric-card amat">
                        <div class="metric-label">{"AMAT Promedio"}</div>
                        <div class="metric-val" style="color:var(--accent-indigo);">{format!("{:.1} ns", s.amat_ns())}</div>
                    </div>
                </div>

                <div class="panel" style="margin-bottom:0.75rem; border-color:rgba(16,185,129,0.35);">
                    <h3 style="margin:0 0 0.45rem; font-size:0.82rem; color:var(--accent-emerald);">{"Conceptos Dominados"}</h3>
                    <ul style="margin:0; padding-left:1.1rem; font-size:0.78rem; color:var(--text-secondary); line-height:1.5;">
                        { for mastered.iter().map(|m| html! { <li>{m}</li> }) }
                    </ul>
                </div>

                <div class="panel" style="border-color:rgba(245,158,11,0.35);">
                    <h3 style="margin:0 0 0.45rem; font-size:0.82rem; color:var(--accent-amber);">{"Recomendaciones de Ingenieria"}</h3>
                    <ul style="margin:0; padding-left:1.1rem; font-size:0.78rem; color:var(--text-secondary); line-height:1.5;">
                        { for recommendations.iter().map(|r| html! { <li>{r}</li> }) }
                    </ul>
                </div>
            </div>
        </div>
    }
}
