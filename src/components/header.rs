use yew::prelude::*;

use crate::components::icons::{BookIcon, ChartIcon, CompareIcon, CpuIcon, FlaskIcon, TrophyIcon};
use crate::sim::types::{CacheConfig, SimMode};

#[derive(Properties, PartialEq)]
pub struct HeaderProps {
    pub active_mode: SimMode,
    pub config: CacheConfig,
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

    let cfg = props.config;

    html! {
        <header class="cockpit-header">
            <div class="header-inner">
                <div class="brand-group">
                    <div class="brand-mark">
                        <CpuIcon />
                    </div>
                    <div class="brand-copy">
                        <h1 class="brand-title">{"CacheLab Cockpit"}</h1>
                        <span class="brand-meta mono">
                            {format!(
                                "{} {}x{} ({}B) · {}",
                                cfg.mapping.label(),
                                cfg.num_blocks,
                                cfg.block_size,
                                cfg.capacity_bytes(),
                                cfg.write_policy.label()
                            )}
                        </span>
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
                                aria-pressed={is_active.to_string()}
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
                    <div class="xp-pill" title="Experiencia acumulada en hits y retos">
                        <span class="xp-label">{"XP"}</span>
                        <span class="xp-value mono">{props.score_xp}</span>
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
