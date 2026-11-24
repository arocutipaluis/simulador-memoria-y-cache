use yew::prelude::*;

use crate::components::icons::{CheckIcon, PlayIcon};
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
                <section class="context-ribbon learn" aria-label="Guia interactiva">
                    <div class="ribbon-main">
                        <span class="badge badge-cyan">{format!("Guia {}/{}", idx + 1, lessons.len())}</span>
                        <strong class="ribbon-title">{lesson.title}</strong>
                        <span class="ribbon-desc" title={lesson.description}>{lesson.description}</span>
                    </div>
                    <div class="ribbon-actions">
                        <button
                            class="btn btn-xs"
                            type="button"
                            disabled={idx == 0}
                            onclick={
                                let cb = props.on_prev_lesson.clone();
                                Callback::from(move |_| cb.emit(()))
                            }
                        >
                            {"Ant."}
                        </button>
                        <button
                            class="btn btn-cyan btn-xs"
                            type="button"
                            onclick={
                                let cb = props.on_run_lesson_demo.clone();
                                Callback::from(move |_| cb.emit(()))
                            }
                        >
                            <PlayIcon />
                            <span>{lesson.action_label}</span>
                        </button>
                        <button
                            class="btn btn-xs"
                            type="button"
                            onclick={
                                let cb = props.on_next_lesson.clone();
                                Callback::from(move |_| cb.emit(()))
                            }
                        >
                            {if idx + 1 < lessons.len() { "Sig." } else { "Inicio" }}
                        </button>
                    </div>
                </section>
            }
        }
        SimMode::Challenge => {
            let idx = props.challenge_idx.min(ch_list.len() - 1);
            let ch = &ch_list[idx];
            let passed =
                props.completed_challenges.contains(&ch.id) || check_challenge_passed(ch.id, &props.engine);
            let next_idx = (idx + 1) % ch_list.len();
            html! {
                <section class="context-ribbon challenge" aria-label="Desafio activo">
                    <div class="ribbon-main">
                        <span class="badge badge-amber">{format!("Reto {}/{} · {} (+{} XP)", idx + 1, ch_list.len(), ch.difficulty, ch.reward_xp)}</span>
                        <strong class="ribbon-title">{ch.title}</strong>
                        <span class="ribbon-desc" title={format!("{} | Pista: {}", ch.description, ch.hint)}>
                            {format!("{} — Pista: {}", ch.description, ch.hint)}
                        </span>
                    </div>
                    <div class="ribbon-actions">
                        { if passed {
                            html! { <span class="badge badge-emerald"><CheckIcon /><span>{"Superado"}</span></span> }
                        } else {
                            html! { <span class="badge">{"En curso"}</span> }
                        }}
                        <button
                            class="btn btn-amber btn-xs"
                            type="button"
                            onclick={
                                let cb = props.on_setup_challenge.clone();
                                let cid = ch.id;
                                Callback::from(move |_| cb.emit(cid))
                            }
                        >
                            {"Cargar Reto"}
                        </button>
                        <button
                            class="btn btn-xs"
                            type="button"
                            onclick={
                                let cb = props.on_select_challenge.clone();
                                Callback::from(move |_| cb.emit(next_idx))
                            }
                        >
                            {"Siguiente"}
                        </button>
                    </div>
                </section>
            }
        }
        SimMode::Lab | SimMode::Compare => html! {},
    }
}
