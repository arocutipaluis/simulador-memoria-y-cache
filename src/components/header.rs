use yew::prelude::*;

use crate::components::icons::{BookIcon, ChartIcon, CompareIcon, CpuIcon, FlaskIcon, TrophyIcon};
use crate::sim::types::SimMode;

#[derive(Properties, PartialEq)]
pub struct HeaderProps {
    pub active_mode: SimMode,
    pub score_xp: u32,
    pub on_select_mode: Callback<SimMode>,
    pub on_open_eval: Callback<()>,
}

#[function_component(Header)]
pub fn header(props: &HeaderProps) -> Html {
    let modes = [
        (SimMode::Learn, "Aprender", html! { <BookIcon /> }),
        (SimMode::Lab, "Experimentar", html! { <FlaskIcon /> }),
        (SimMode::Challenge, "Desafios", html! { <TrophyIcon /> }),
        (SimMode::Compare, "Comparador A/B", html! { <CompareIcon /> }),
    ];

    html! {
        <header class="cockpit-header">
            <div class="header-inner">
                <div class="brand-group">
                    <div class="brand-mark">
                        <CpuIcon />
                    </div>
                    <div>
                        <h1 class="brand-title">
                            {"CacheLab Cockpit"}
                            <span class="version-pill">{"RUST/YEW WASM v3.0"}</span>
                        </h1>
                        <p class="brand-sub">{"Simulador de Jerarquia de Memoria, Mapeo y Politicas de Cache"}</p>
                    </div>
                </div>

                <nav class="mode-nav" aria-label="Modos del simulador">
                    { for modes.into_iter().map(|(mode, label, icon)| {
                        let is_active = props.active_mode == mode;
                        let cb = props.on_select_mode.clone();
                        let onclick = Callback::from(move |_| cb.emit(mode));
                        html! {
                            <button
                                class={classes!("mode-btn", is_active.then_some("active"))}
                                {onclick}
                                type="button"
                            >
                                {icon}
                                <span>{label}</span>
                            </button>
                        }
                    }) }
                </nav>

                <div class="header-actions">
                    <div class="xp-pill">
                        <div>
                            <div class="xp-label">{"XP"}</div>
                            <div class="xp-value mono">{format!("{} XP", props.score_xp)}</div>
                        </div>
                    </div>
                    <button
                        class="btn btn-sm"
                        type="button"
                        onclick={
                            let cb = props.on_open_eval.clone();
                            Callback::from(move |_| cb.emit(()))
                        }
                    >
                        <ChartIcon />
                        <span>{"Evaluacion"}</span>
                    </button>
                </div>
            </div>
        </header>
    }
}
