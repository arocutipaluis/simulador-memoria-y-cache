use yew::prelude::*;

use crate::sim::engine::CacheEngine;
use crate::sim::presets::{challenges, check_challenge_passed, learn_lessons};
use crate::sim::types::SimMode;

#[derive(Properties, PartialEq)]
pub struct ModeBannerProps {
    pub mode: SimMode,
    pub learn_step: usize,
    pub challenge_idx: usize,
    pub completed_challenges: Vec<usize>,
    pub engine: CacheEngine,
    pub on_run_lesson_demo: Callback<()>,
    pub on_prev_lesson: Callback<()>,
    pub on_next_lesson: Callback<()>,
    pub on_select_challenge: Callback<usize>,
    pub on_setup_challenge: Callback<usize>,
}

#[function_component(ModeBanner)]
pub fn mode_banner(props: &ModeBannerProps) -> Html {
    let lessons = learn_lessons();
    let ch_list = challenges();

    match props.mode {
        SimMode::Learn => {
            let idx = props.learn_step.min(lessons.len() - 1);
            let lesson = &lessons[idx];
            html! {
                <section class="mode-banner">
                    <div class="banner-text">
                        <span class="badge badge-cyan">{format!("GUIA INTERACTIVA {}/{}", idx + 1, lessons.len())}</span>
                        <h2>{lesson.title}</h2>
                        <p>{lesson.description}</p>
                    </div>
                    <div class="banner-actions">
                        <button
                            class="btn btn-sm"
                            type="button"
                            disabled={idx == 0}
                            onclick={
                                let cb = props.on_prev_lesson.clone();
                                Callback::from(move |_| cb.emit(()))
                            }
                        >
                            {"Anterior"}
                        </button>
                        <button
                            class="btn btn-cyan btn-sm"
                            type="button"
                            onclick={
                                let cb = props.on_run_lesson_demo.clone();
                                Callback::from(move |_| cb.emit(()))
                            }
                        >
                            {lesson.action_label}
                        </button>
                        <button
                            class="btn btn-sm"
                            type="button"
                            onclick={
                                let cb = props.on_next_lesson.clone();
                                Callback::from(move |_| cb.emit(()))
                            }
                        >
                            {if idx + 1 < lessons.len() { "Siguiente ->" } else { "Reiniciar Guia" }}
                        </button>
                    </div>
                </section>
            }
        }
        SimMode::Lab => html! {
            <section class="mode-banner">
                <div class="banner-text">
                    <span class="badge badge-emerald">{"LABORATORIO LIBRE"}</span>
                    <h2>{"Modo Experimentacion Abierta"}</h2>
                    <p>{"Configura mapeo (Directo, 2-Way, 4-Way, Full), reemplazo (LRU/FIFO/Random) y escritura (Write-Back vs Write-Through) para observar cada acceso paso a paso."}</p>
                </div>
            </section>
        },
        SimMode::Challenge => {
            let idx = props.challenge_idx.min(ch_list.len() - 1);
            let ch = &ch_list[idx];
            let passed = props.completed_challenges.contains(&ch.id) || check_challenge_passed(ch.id, &props.engine);
            let next_idx = (idx + 1) % ch_list.len();
            html! {
                <section class="mode-banner">
                    <div class="banner-text">
                        <div style="display:flex; gap:0.4rem; flex-wrap:wrap; align-items:center;">
                            <span class="badge badge-amber">{format!("RETO {}/{} • {} (+{} XP)", idx + 1, ch_list.len(), ch.difficulty, ch.reward_xp)}</span>
                            { if passed {
                                html! { <span class="badge badge-emerald">{"COMPLETADO"}</span> }
                            } else {
                                html! { <span class="badge">{"En progreso"}</span> }
                            }}
                        </div>
                        <h2>{ch.title}</h2>
                        <p>{ch.description}</p>
                        <p style="font-size:0.73rem; color:var(--accent-cyan); margin-top:0.25rem;">
                            {format!("Pista: {}", ch.hint)}
                        </p>
                    </div>
                    <div class="banner-actions">
                        <button
                            class="btn btn-amber btn-sm"
                            type="button"
                            onclick={
                                let cb = props.on_setup_challenge.clone();
                                let cid = ch.id;
                                Callback::from(move |_| cb.emit(cid))
                            }
                        >
                            {"Preparar Configuracion"}
                        </button>
                        <button
                            class="btn btn-sm"
                            type="button"
                            onclick={
                                let cb = props.on_select_challenge.clone();
                                Callback::from(move |_| cb.emit(next_idx))
                            }
                        >
                            {"Siguiente Reto ->"}
                        </button>
                    </div>
                </section>
            }
        }
        SimMode::Compare => html! {
            <section class="mode-banner">
                <div class="banner-text">
                    <span class="badge badge-indigo">{"FUNCION EXTRA • MODO COMPARATIVO A/B"}</span>
                    <h2>{"Comparador Simultaneo de Arquitecturas de Cache"}</h2>
                    <p>{"Ejecuta exactamente la misma secuencia de direcciones sobre dos caches distintas en paralelo para medir la brecha en Hit Rate, AMAT y fallos de conflicto."}</p>
                </div>
            </section>
        },
    }
}
