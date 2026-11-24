use gloo_timers::callback::Interval;
use yew::prelude::*;

use crate::components::address_decoder::AddressDecoder;
use crate::components::cache_grid::CacheGrid;
use crate::components::comparator_view::{parse_trace_line, ComparatorView};
use crate::components::eval_modal::EvalModal;
use crate::components::header::Header;
use crate::components::metrics_panel::MetricsPanel;
use crate::components::mode_banner::ModeBanner;
use crate::components::ram_view::RamView;
use crate::components::step_controller::{parse_address_str, StepController};
use crate::sim::engine::CacheEngine;
use crate::sim::presets::{challenges, check_challenge_passed, learn_lessons, preset_sequence};
use crate::sim::types::{
    format_hex, CacheConfig, MappingPolicy, MemInstruction, MemOp, PresetKind, ReplacementPolicy,
    SimMode, WritePolicy,
};

#[function_component(App)]
pub fn app() -> Html {
    let mode = use_state(|| SimMode::Lab);
    let learn_step = use_state(|| 0usize);
    let challenge_idx = use_state(|| 0usize);
    let completed_challenges = use_state(Vec::<usize>::new);
    let bonus_xp = use_state(|| 0u32);
    let show_eval = use_state(|| false);

    // Simulador principal
    let engine = use_state(|| CacheEngine::new(CacheConfig::default()));
    let selected_addr = use_state(|| 0u8);
    let custom_addr_input = use_state(|| "0x00".to_string());
    let addr_error = use_state(|| None::<String>);
    let write_value = use_state(|| "DAT_99".to_string());
    let queue = use_state(|| preset_sequence(PresetKind::Sequential));
    let queue_cursor = use_state(|| 0usize);
    let is_playing = use_state(|| false);
    let speed_ms = use_state(|| 420u32);

    // Comparador A/B (modo estricto L1 -> RAM por defecto para aislar L1 como en MARS)
    let engine_a = use_state(|| {
        CacheEngine::new(CacheConfig {
            num_blocks: 8,
            block_size: 4,
            mapping: MappingPolicy::DirectMapped,
            replacement: ReplacementPolicy::Lru,
            write_policy: WritePolicy::WriteBack,
            use_l2_l3: false,
        })
    });
    let engine_b = use_state(|| {
        CacheEngine::new(CacheConfig {
            num_blocks: 8,
            block_size: 4,
            mapping: MappingPolicy::SetAssociative(2),
            replacement: ReplacementPolicy::Lru,
            write_policy: WritePolicy::WriteBack,
            use_l2_l3: false,
        })
    });
    let cmp_trace = use_state(|| preset_sequence(PresetKind::Thrashing));
    let cmp_cursor = use_state(|| 0usize);
    let cmp_playing = use_state(|| false);
    let cmp_custom_text = use_state(|| "0x00, 0x20, 0x01, 0x21, 0x02, 0x22, 0x03, 0x23".to_string());
    let cmp_error = use_state(|| None::<String>);

    // Evaluar retos tras cada operacion en simulador principal
    let eval_challenges = {
        let completed_challenges = completed_challenges.clone();
        let bonus_xp = bonus_xp.clone();
        Callback::from(move |eng: CacheEngine| {
            let mut comp = (*completed_challenges).clone();
            let mut xp = *bonus_xp;
            let mut changed = false;
            for ch in challenges() {
                if !comp.contains(&ch.id) && check_challenge_passed(ch.id, &eng) {
                    comp.push(ch.id);
                    xp += ch.reward_xp;
                    changed = true;
                }
            }
            if changed {
                completed_challenges.set(comp);
                bonus_xp.set(xp);
            }
        })
    };

    // Intervalo Auto-Play para simulador principal
    {
        let is_playing = is_playing.clone();
        let queue = queue.clone();
        let queue_cursor = queue_cursor.clone();
        let engine = engine.clone();
        let selected_addr = selected_addr.clone();
        let custom_addr_input = custom_addr_input.clone();
        let eval_challenges = eval_challenges.clone();
        let speed = *speed_ms;
        let active = *is_playing;

        use_effect_with((active, speed, *queue_cursor, (*queue).clone()), move |_| {
            let interval_handle = if active {
                Some(Interval::new(speed, move || {
                    let cur = *queue_cursor;
                    let q = &*queue;
                    if cur >= q.len() {
                        is_playing.set(false);
                        return;
                    }
                    let instr = &q[cur];
                    let mut next_eng = (*engine).clone();
                    next_eng.execute(instr.op.clone(), instr.addr);
                    selected_addr.set(instr.addr);
                    custom_addr_input.set(format_hex(instr.addr));
                    eval_challenges.emit(next_eng.clone());
                    engine.set(next_eng);
                    if cur + 1 >= q.len() {
                        is_playing.set(false);
                    }
                    queue_cursor.set(cur + 1);
                }))
            } else {
                None
            };
            move || drop(interval_handle)
        });
    }

    // Intervalo Auto-Play para Comparador A/B
    {
        let cmp_playing = cmp_playing.clone();
        let cmp_trace = cmp_trace.clone();
        let cmp_cursor = cmp_cursor.clone();
        let engine_a = engine_a.clone();
        let engine_b = engine_b.clone();
        let active = *cmp_playing;

        use_effect_with((active, *cmp_cursor, (*cmp_trace).clone()), move |_| {
            let interval_handle = if active {
                Some(Interval::new(360, move || {
                    let cur = *cmp_cursor;
                    let tr = &*cmp_trace;
                    if cur >= tr.len() {
                        cmp_playing.set(false);
                        return;
                    }
                    let instr = &tr[cur];
                    let mut next_a = (*engine_a).clone();
                    let mut next_b = (*engine_b).clone();
                    next_a.execute(instr.op.clone(), instr.addr);
                    next_b.execute(instr.op.clone(), instr.addr);
                    engine_a.set(next_a);
                    engine_b.set(next_b);
                    if cur + 1 >= tr.len() {
                        cmp_playing.set(false);
                    }
                    cmp_cursor.set(cur + 1);
                }))
            } else {
                None
            };
            move || drop(interval_handle)
        });
    }

    let on_select_addr = {
        let selected_addr = selected_addr.clone();
        let custom_addr_input = custom_addr_input.clone();
        let addr_error = addr_error.clone();
        let engine = engine.clone();
        Callback::from(move |addr: u8| {
            selected_addr.set(addr);
            custom_addr_input.set(format_hex(addr));
            addr_error.set(None);
            let mut next_eng = (*engine).clone();
            next_eng.last_breakdown = next_eng.config.decode(addr);
            engine.set(next_eng);
        })
    };

    let on_custom_addr_change = {
        let custom_addr_input = custom_addr_input.clone();
        let selected_addr = selected_addr.clone();
        let addr_error = addr_error.clone();
        let engine = engine.clone();
        Callback::from(move |raw: String| {
            custom_addr_input.set(raw.clone());
            match parse_address_str(&raw) {
                Ok(addr) => {
                    selected_addr.set(addr);
                    addr_error.set(None);
                    let mut next_eng = (*engine).clone();
                    next_eng.last_breakdown = next_eng.config.decode(addr);
                    engine.set(next_eng);
                }
                Err(msg) => addr_error.set(Some(msg)),
            }
        })
    };

    let on_execute_single = {
        let engine = engine.clone();
        let selected_addr = selected_addr.clone();
        let addr_error = addr_error.clone();
        let is_playing = is_playing.clone();
        let eval_challenges = eval_challenges.clone();
        Callback::from(move |op: MemOp| {
            if addr_error.is_some() {
                return;
            }
            is_playing.set(false);
            let mut next_eng = (*engine).clone();
            next_eng.execute(op, *selected_addr);
            eval_challenges.emit(next_eng.clone());
            engine.set(next_eng);
        })
    };

    let on_enqueue_single = {
        let queue = queue.clone();
        let selected_addr = selected_addr.clone();
        let addr_error = addr_error.clone();
        Callback::from(move |op: MemOp| {
            if addr_error.is_some() {
                return;
            }
            let mut q = (*queue).clone();
            q.push(MemInstruction {
                op,
                addr: *selected_addr,
            });
            queue.set(q);
        })
    };

    let on_load_preset = {
        let queue = queue.clone();
        let queue_cursor = queue_cursor.clone();
        let is_playing = is_playing.clone();
        let engine = engine.clone();
        Callback::from(move |kind: PresetKind| {
            is_playing.set(false);
            let mut next_eng = (*engine).clone();
            next_eng.reset();
            next_eng.feedback = format!("Preset '{}' cargado en cola. Usa Step o Auto para ejecutar.", kind.label());
            engine.set(next_eng);
            queue.set(preset_sequence(kind));
            queue_cursor.set(0);
        })
    };

    let on_step_queue = {
        let queue = queue.clone();
        let queue_cursor = queue_cursor.clone();
        let engine = engine.clone();
        let selected_addr = selected_addr.clone();
        let custom_addr_input = custom_addr_input.clone();
        let eval_challenges = eval_challenges.clone();
        Callback::from(move |_| {
            let cur = *queue_cursor;
            let q = &*queue;
            if cur >= q.len() {
                return;
            }
            let instr = &q[cur];
            let mut next_eng = (*engine).clone();
            next_eng.execute(instr.op.clone(), instr.addr);
            selected_addr.set(instr.addr);
            custom_addr_input.set(format_hex(instr.addr));
            eval_challenges.emit(next_eng.clone());
            engine.set(next_eng);
            queue_cursor.set(cur + 1);
        })
    };

    let on_reset_all = {
        let engine = engine.clone();
        let queue_cursor = queue_cursor.clone();
        let is_playing = is_playing.clone();
        Callback::from(move |_| {
            is_playing.set(false);
            let mut next_eng = (*engine).clone();
            next_eng.reset();
            engine.set(next_eng);
            queue_cursor.set(0);
        })
    };

    let on_config_change = {
        let engine = engine.clone();
        let queue_cursor = queue_cursor.clone();
        let is_playing = is_playing.clone();
        Callback::from(move |cfg: CacheConfig| {
            is_playing.set(false);
            let mut next_eng = (*engine).clone();
            next_eng.reconfigure(cfg);
            engine.set(next_eng);
            queue_cursor.set(0);
        })
    };

    let on_page_fault = {
        let engine = engine.clone();
        let selected_addr = selected_addr.clone();
        Callback::from(move |_| {
            let mut next_eng = (*engine).clone();
            next_eng.simulate_page_fault(*selected_addr);
            engine.set(next_eng);
        })
    };

    let on_run_lesson_demo = {
        let engine = engine.clone();
        let learn_step = learn_step.clone();
        let selected_addr = selected_addr.clone();
        let custom_addr_input = custom_addr_input.clone();
        Callback::from(move |_| {
            let lessons = learn_lessons();
            let lesson = &lessons[(*learn_step).min(lessons.len() - 1)];
            let mut next_eng = (*engine).clone();
            next_eng.execute(lesson.demo_op.clone(), lesson.demo_addr);
            selected_addr.set(lesson.demo_addr);
            custom_addr_input.set(format_hex(lesson.demo_addr));
            engine.set(next_eng);
        })
    };

    let on_setup_challenge = {
        let engine = engine.clone();
        let queue = queue.clone();
        let queue_cursor = queue_cursor.clone();
        let is_playing = is_playing.clone();
        Callback::from(move |cid: usize| {
            is_playing.set(false);
            let mut next_eng = (*engine).clone();
            match cid {
                2 => {
                    next_eng.reconfigure(CacheConfig {
                        num_blocks: 4,
                        block_size: 2,
                        mapping: MappingPolicy::DirectMapped,
                        replacement: ReplacementPolicy::Lru,
                        write_policy: WritePolicy::WriteBack,
                        use_l2_l3: true,
                    });
                    queue.set(preset_sequence(PresetKind::Temporal));
                }
                3 => {
                    next_eng.reconfigure(CacheConfig {
                        num_blocks: 4,
                        block_size: 2,
                        mapping: MappingPolicy::SetAssociative(2),
                        replacement: ReplacementPolicy::Lru,
                        write_policy: WritePolicy::WriteBack,
                        use_l2_l3: true,
                    });
                    queue.set(preset_sequence(PresetKind::WriteHeavy));
                }
                4 => {
                    next_eng.reconfigure(CacheConfig {
                        num_blocks: 8,
                        block_size: 4,
                        mapping: MappingPolicy::SetAssociative(2),
                        replacement: ReplacementPolicy::Lru,
                        write_policy: WritePolicy::WriteBack,
                        use_l2_l3: true,
                    });
                    queue.set(preset_sequence(PresetKind::Thrashing));
                }
                _ => {
                    next_eng.reset();
                    queue.set(preset_sequence(PresetKind::Spatial));
                }
            }
            engine.set(next_eng);
            queue_cursor.set(0);
        })
    };

    let total_xp = engine.xp_earned + *bonus_xp;

    html! {
        <div class="app-shell">
            <Header
                active_mode={*mode}
                score_xp={total_xp}
                on_select_mode={{
                    let mode = mode.clone();
                    let is_playing = is_playing.clone();
                    let cmp_playing = cmp_playing.clone();
                    Callback::from(move |m| {
                        is_playing.set(false);
                        cmp_playing.set(false);
                        mode.set(m);
                    })
                }}
                on_open_eval={{
                    let show_eval = show_eval.clone();
                    Callback::from(move |_| show_eval.set(true))
                }}
            />

            <main class="main-wrap">
                <ModeBanner
                    mode={*mode}
                    learn_step={*learn_step}
                    challenge_idx={*challenge_idx}
                    completed_challenges={(*completed_challenges).clone()}
                    engine={(*engine).clone()}
                    on_run_lesson_demo={on_run_lesson_demo}
                    on_prev_lesson={{
                        let learn_step = learn_step.clone();
                        Callback::from(move |_| {
                            let cur = *learn_step;
                            if cur > 0 { learn_step.set(cur - 1); }
                        })
                    }}
                    on_next_lesson={{
                        let learn_step = learn_step.clone();
                        Callback::from(move |_| {
                            let total = learn_lessons().len();
                            learn_step.set((*learn_step + 1) % total);
                        })
                    }}
                    on_select_challenge={{
                        let challenge_idx = challenge_idx.clone();
                        Callback::from(move |idx| challenge_idx.set(idx))
                    }}
                    on_setup_challenge={on_setup_challenge}
                />

                { if *mode == SimMode::Compare {
                    html! {
                        <ComparatorView
                            engine_a={(*engine_a).clone()}
                            engine_b={(*engine_b).clone()}
                            trace={(*cmp_trace).clone()}
                            cursor={*cmp_cursor}
                            is_playing={*cmp_playing}
                            custom_trace_input={(*cmp_custom_text).clone()}
                            trace_error={(*cmp_error).clone()}
                            on_config_a={{
                                let engine_a = engine_a.clone();
                                let cmp_cursor = cmp_cursor.clone();
                                let cmp_playing = cmp_playing.clone();
                                let engine_b = engine_b.clone();
                                Callback::from(move |cfg| {
                                    cmp_playing.set(false);
                                    let mut a = (*engine_a).clone();
                                    a.reconfigure(cfg);
                                    let mut b = (*engine_b).clone();
                                    b.reset();
                                    engine_a.set(a);
                                    engine_b.set(b);
                                    cmp_cursor.set(0);
                                })
                            }}
                            on_config_b={{
                                let engine_b = engine_b.clone();
                                let engine_a = engine_a.clone();
                                let cmp_cursor = cmp_cursor.clone();
                                let cmp_playing = cmp_playing.clone();
                                Callback::from(move |cfg| {
                                    cmp_playing.set(false);
                                    let mut b = (*engine_b).clone();
                                    b.reconfigure(cfg);
                                    let mut a = (*engine_a).clone();
                                    a.reset();
                                    engine_a.set(a);
                                    engine_b.set(b);
                                    cmp_cursor.set(0);
                                })
                            }}
                            on_load_preset={{
                                let cmp_trace = cmp_trace.clone();
                                let cmp_cursor = cmp_cursor.clone();
                                let cmp_playing = cmp_playing.clone();
                                let cmp_error = cmp_error.clone();
                                let engine_a = engine_a.clone();
                                let engine_b = engine_b.clone();
                                Callback::from(move |kind: PresetKind| {
                                    cmp_playing.set(false);
                                    cmp_error.set(None);
                                    let mut a = (*engine_a).clone();
                                    let mut b = (*engine_b).clone();
                                    a.reset();
                                    b.reset();
                                    engine_a.set(a);
                                    engine_b.set(b);
                                    cmp_trace.set(preset_sequence(kind));
                                    cmp_cursor.set(0);
                                })
                            }}
                            on_custom_trace_input={{
                                let cmp_custom_text = cmp_custom_text.clone();
                                Callback::from(move |v| cmp_custom_text.set(v))
                            }}
                            on_apply_custom_trace={{
                                let cmp_custom_text = cmp_custom_text.clone();
                                let cmp_trace = cmp_trace.clone();
                                let cmp_cursor = cmp_cursor.clone();
                                let cmp_error = cmp_error.clone();
                                let cmp_playing = cmp_playing.clone();
                                let engine_a = engine_a.clone();
                                let engine_b = engine_b.clone();
                                Callback::from(move |_| {
                                    cmp_playing.set(false);
                                    match parse_trace_line(&cmp_custom_text) {
                                        Ok(parsed) => {
                                            cmp_error.set(None);
                                            let mut a = (*engine_a).clone();
                                            let mut b = (*engine_b).clone();
                                            a.reset();
                                            b.reset();
                                            engine_a.set(a);
                                            engine_b.set(b);
                                            cmp_trace.set(parsed);
                                            cmp_cursor.set(0);
                                        }
                                        Err(e) => cmp_error.set(Some(e)),
                                    }
                                })
                            }}
                            on_step={{
                                let cmp_trace = cmp_trace.clone();
                                let cmp_cursor = cmp_cursor.clone();
                                let engine_a = engine_a.clone();
                                let engine_b = engine_b.clone();
                                Callback::from(move |_| {
                                    let cur = *cmp_cursor;
                                    let tr = &*cmp_trace;
                                    if cur >= tr.len() { return; }
                                    let instr = &tr[cur];
                                    let mut a = (*engine_a).clone();
                                    let mut b = (*engine_b).clone();
                                    a.execute(instr.op.clone(), instr.addr);
                                    b.execute(instr.op.clone(), instr.addr);
                                    engine_a.set(a);
                                    engine_b.set(b);
                                    cmp_cursor.set(cur + 1);
                                })
                            }}
                            on_toggle_play={{
                                let cmp_playing = cmp_playing.clone();
                                Callback::from(move |_| cmp_playing.set(!*cmp_playing))
                            }}
                            on_run_all={{
                                let cmp_trace = cmp_trace.clone();
                                let cmp_cursor = cmp_cursor.clone();
                                let cmp_playing = cmp_playing.clone();
                                let engine_a = engine_a.clone();
                                let engine_b = engine_b.clone();
                                Callback::from(move |_| {
                                    cmp_playing.set(false);
                                    let cur = *cmp_cursor;
                                    let tr = &*cmp_trace;
                                    if cur >= tr.len() { return; }
                                    let mut a = (*engine_a).clone();
                                    let mut b = (*engine_b).clone();
                                    for instr in &tr[cur..] {
                                        a.execute(instr.op.clone(), instr.addr);
                                        b.execute(instr.op.clone(), instr.addr);
                                    }
                                    engine_a.set(a);
                                    engine_b.set(b);
                                    cmp_cursor.set(tr.len());
                                })
                            }}
                            on_reset={{
                                let cmp_cursor = cmp_cursor.clone();
                                let cmp_playing = cmp_playing.clone();
                                let engine_a = engine_a.clone();
                                let engine_b = engine_b.clone();
                                Callback::from(move |_| {
                                    cmp_playing.set(false);
                                    let mut a = (*engine_a).clone();
                                    let mut b = (*engine_b).clone();
                                    a.reset();
                                    b.reset();
                                    engine_a.set(a);
                                    engine_b.set(b);
                                    cmp_cursor.set(0);
                                })
                            }}
                        />
                    }
                } else {
                    html! {
                        <>
                            <MetricsPanel
                                stats={engine.stats.clone()}
                                feedback={engine.feedback.clone()}
                                logs={engine.logs.clone()}
                                on_clear_log={{
                                    let engine = engine.clone();
                                    Callback::from(move |_| {
                                        let mut next_eng = (*engine).clone();
                                        next_eng.logs.clear();
                                        engine.set(next_eng);
                                    })
                                }}
                            />

                            <div class="cockpit-grid">
                                <StepController
                                    selected_addr={*selected_addr}
                                    custom_addr_input={(*custom_addr_input).clone()}
                                    addr_error={(*addr_error).clone()}
                                    write_value={(*write_value).clone()}
                                    config={engine.config}
                                    queue={(*queue).clone()}
                                    queue_cursor={*queue_cursor}
                                    is_playing={*is_playing}
                                    speed_ms={*speed_ms}
                                    on_select_addr={on_select_addr.clone()}
                                    on_custom_addr_change={on_custom_addr_change}
                                    on_write_val_change={{
                                        let write_value = write_value.clone();
                                        Callback::from(move |v| write_value.set(v))
                                    }}
                                    on_execute_single={on_execute_single}
                                    on_enqueue_single={on_enqueue_single}
                                    on_load_preset={on_load_preset}
                                    on_step_queue={on_step_queue}
                                    on_toggle_play={{
                                        let is_playing = is_playing.clone();
                                        Callback::from(move |_| is_playing.set(!*is_playing))
                                    }}
                                    on_clear_queue={{
                                        let queue = queue.clone();
                                        let queue_cursor = queue_cursor.clone();
                                        let is_playing = is_playing.clone();
                                        Callback::from(move |_| {
                                            is_playing.set(false);
                                            queue.set(Vec::new());
                                            queue_cursor.set(0);
                                        })
                                    }}
                                    on_reset_all={on_reset_all}
                                    on_speed_change={{
                                        let speed_ms = speed_ms.clone();
                                        Callback::from(move |v| speed_ms.set(v))
                                    }}
                                    on_config_change={on_config_change}
                                />

                                <div class="stack-col">
                                    <AddressDecoder
                                        breakdown={engine.last_breakdown.clone()}
                                        config={engine.config}
                                    />
                                    <CacheGrid engine={(*engine).clone()} />
                                    <RamView
                                        ram={engine.ram.clone()}
                                        selected_addr={*selected_addr}
                                        last_hit_level={engine.last_hit_level}
                                        on_select_addr={on_select_addr}
                                        on_page_fault={on_page_fault}
                                    />
                                </div>
                            </div>
                        </>
                    }
                }}
            </main>

            <EvalModal
                show={*show_eval}
                engine={(*engine).clone()}
                score_xp={total_xp}
                completed_challenges={(*completed_challenges).clone()}
                on_close={{
                    let show_eval = show_eval.clone();
                    Callback::from(move |_| show_eval.set(false))
                }}
            />
        </div>
    }
}
