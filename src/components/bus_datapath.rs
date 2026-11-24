use yew::prelude::*;

use crate::components::EvictionNotice;
use crate::sim::engine::CacheEngine;
use crate::sim::types::{format_hex, BusFlow, HitLevel};

#[derive(Properties, PartialEq)]
pub struct BusDatapathProps {
    pub engine: CacheEngine,
    pub eviction: Option<EvictionNotice>,
    pub speed_ms: u32,
    pub is_playing: bool,
}

#[function_component(BusDatapath)]
pub fn bus_datapath(props: &BusDatapathProps) -> Html {
    let eng = &props.engine;
    let b = &eng.last_breakdown;
    let clock = eng.clock;
    let use_l2_l3 = eng.config.use_l2_l3;

    let anim_ms = if props.is_playing {
        ((props.speed_ms * 88) / 100).clamp(240, 720)
    } else {
        560
    };

    let is_page_fault = eng.bus_flow == BusFlow::PageFaultDisk || eng.last_hit_level == Some(HitLevel::Disk);
    let is_wb_bus = eng.bus_flow == BusFlow::WriteBackToRam
        || props.eviction.as_ref().map(|e| e.write_back_ram).unwrap_or(false);

    let (pkt_class, status_badge_cls, bus_summary) = if clock == 0 {
        ("pkt-idle", "badge", "BUS EN REPOSO")
    } else if is_page_fault {
        ("pkt-page-fault", "badge badge-rose", "PAGE FAULT -> SSD (+10,000 ns)")
    } else {
        match eng.last_hit_level {
            Some(HitLevel::L1) => {
                if eng.bus_flow == BusFlow::WriteBackToRam {
                    ("pkt-write-through", "badge badge-amber", "HIT L1 + WRITE-THROUGH -> RAM")
                } else {
                    ("pkt-l1-hit", "badge badge-emerald", "HIT L1 -> DATO A CPU")
                }
            }
            Some(HitLevel::L2) => ("pkt-l2-hit", "badge badge-emerald", "MISS L1 -> HIT L2 -> CPU"),
            Some(HitLevel::L3) => ("pkt-l3-hit", "badge badge-emerald", "MISS L1/L2 -> HIT L3 -> CPU"),
            Some(HitLevel::Ram) => {
                if is_wb_bus {
                    ("pkt-ram-miss", "badge badge-amber", "MISS -> RAM FETCH + WB FLUSH")
                } else {
                    ("pkt-ram-miss", "badge badge-rose", "MISS -> FETCH DRAM -> L1 -> CPU")
                }
            }
            _ => ("pkt-l1-hit", "badge", "BUS ACTIVO"),
        }
    };

    let l1_active = clock > 0 && !is_page_fault;
    let l1_hit = eng.last_hit_level == Some(HitLevel::L1);
    let l23_hit = matches!(eng.last_hit_level, Some(HitLevel::L2 | HitLevel::L3));
    let dram_hit = matches!(eng.last_hit_level, Some(HitLevel::Ram | HitLevel::Disk))
        || eng.bus_flow == BusFlow::WriteBackToRam;

    let cached_count = eng.ram.iter().filter(|c| c.cached_in_l1).count();
    let dirty_count = eng.ram.iter().filter(|c| c.dirty_in_cache).count();

    let node_l1_cls = if clock == 0 {
        "svg-node"
    } else if l1_hit {
        "svg-node node-hit"
    } else if props.eviction.is_some() {
        "svg-node node-wb"
    } else {
        "svg-node node-miss"
    };

    let node_l23_cls = if !use_l2_l3 {
        "svg-node node-bypass"
    } else if l23_hit {
        "svg-node node-hit"
    } else if clock > 0 && !l1_hit {
        "svg-node node-probe"
    } else {
        "svg-node"
    };

    let node_dram_cls = if is_page_fault {
        "svg-node node-miss"
    } else if is_wb_bus {
        "svg-node node-wb"
    } else if dram_hit && clock > 0 {
        "svg-node node-miss"
    } else {
        "svg-node"
    };

    let req_beam_cls = if clock == 0 {
        "bus-trace"
    } else if is_page_fault || eng.last_hit_level == Some(HitLevel::Ram) {
        "bus-trace trace-miss"
    } else if eng.bus_flow == BusFlow::WriteBackToRam {
        "bus-trace trace-wb"
    } else {
        "bus-trace trace-hit"
    };

    let evict_notice = props.eviction.as_ref();

    html! {
        <section class="panel datapath-panel" aria-label="Flujo de Buses SVG y Registros CPU">
            <div class="panel-header compact-header">
                <div class="reg-inline" aria-label="Registros CPU">
                    <span class="reg-chip">
                        <span class="reg-k">{"RAX"}</span>
                        <span class="reg-v mono">{&eng.reg_rax}</span>
                    </span>
                    <span class="reg-chip">
                        <span class="reg-k">{"MAR"}</span>
                        <span class="reg-v mono text-cyan">{format_hex(eng.reg_mar)}</span>
                    </span>
                    <span class="reg-chip">
                        <span class="reg-k">{"MDR"}</span>
                        <span class="reg-v mono text-amber">{&eng.reg_mdr}</span>
                    </span>
                    <span class="reg-chip">
                        <span class="reg-k">{"CICLO"}</span>
                        <span class="reg-v mono">{format!("#{}", clock)}</span>
                    </span>
                </div>

                <div class="datapath-badges">
                    { if let Some(ev) = evict_notice {
                        html! {
                            <span
                                key={format!("ev-badge-{}", clock)}
                                class={classes!("badge", "badge-pop", if ev.write_back_ram { "badge-amber" } else { "badge-rose" })}
                            >
                                { if ev.write_back_ram {
                                    format!("DESALOJO B#{} -> WB RAM", ev.old_block)
                                } else {
                                    format!("DESALOJO B#{} (S{} V{})", ev.old_block, ev.set_idx, ev.way_idx)
                                }}
                            </span>
                        }
                    } else {
                        html! {}
                    }}
                    <span class={classes!(status_badge_cls)}>{bus_summary}</span>
                </div>
            </div>

            <div class="svg-bus-stage">
                <svg
                    class="datapath-svg"
                    viewBox="0 0 760 118"
                    preserveAspectRatio="xMidYMid meet"
                    role="img"
                    aria-label="Diagrama animado de flujo por buses entre CPU, Decodificador, Cache y RAM"
                >
                    <defs>
                        <marker id="arrow-cyan" viewBox="0 0 8 8" refX="6" refY="4" markerWidth="5" markerHeight="5" orient="auto-start-reverse">
                            <path d="M 1 1 L 7 4 L 1 7 z" fill="#22d3ee" />
                        </marker>
                        <marker id="arrow-emerald" viewBox="0 0 8 8" refX="6" refY="4" markerWidth="5" markerHeight="5" orient="auto-start-reverse">
                            <path d="M 1 1 L 7 4 L 1 7 z" fill="#10b981" />
                        </marker>
                        <marker id="arrow-amber" viewBox="0 0 8 8" refX="6" refY="4" markerWidth="5" markerHeight="5" orient="auto-start-reverse">
                            <path d="M 1 1 L 7 4 L 1 7 z" fill="#f59e0b" />
                        </marker>
                        <marker id="arrow-rose" viewBox="0 0 8 8" refX="6" refY="4" markerWidth="5" markerHeight="5" orient="auto-start-reverse">
                            <path d="M 1 1 L 7 4 L 1 7 z" fill="#f43f5e" />
                        </marker>
                    </defs>

                    // Pistas base PCB (Req superior y=42, Retorno y=66, Evict/WB loop y=102)
                    <path class="bus-base" d="M 122 42 L 154 42 M 274 42 L 306 42 M 430 42 L 462 42 M 582 42 L 616 42" />
                    <path class="bus-base" d="M 616 66 L 582 66 M 462 66 L 430 66 M 306 66 L 274 66 M 154 66 L 122 66" />
                    <path class="bus-base bus-wb-base" d="M 368 86 L 368 102 L 684 102 L 684 86" />

                    // Trazos activos segun resultado del ciclo actual
                    { if clock > 0 {
                        html! {
                            <g key={format!("trace-{}", clock)}>
                                // Tramo CPU -> Decoder -> L1
                                <path
                                    class={classes!(req_beam_cls, "trace-flow")}
                                    d="M 122 42 L 154 42 M 274 42 L 306 42"
                                    marker-end="url(#arrow-cyan)"
                                />

                                // Tramo L1 -> L2/L3 (si no acerto en L1)
                                { if !l1_hit {
                                    html! {
                                        <path
                                            class={classes!(req_beam_cls, "trace-flow")}
                                            d="M 430 42 L 462 42"
                                        />
                                    }
                                } else { html! {} }}

                                // Tramo L2/L3 -> DRAM (si miss hasta RAM o Write-Through)
                                { if dram_hit {
                                    html! {
                                        <path
                                            class={classes!(req_beam_cls, "trace-flow")}
                                            d="M 582 42 L 616 42"
                                        />
                                    }
                                } else { html! {} }}

                                // Linea de retorno a CPU
                                <path
                                    class="bus-trace trace-return"
                                    d={if dram_hit {
                                        "M 616 66 L 122 66"
                                    } else if l23_hit {
                                        "M 462 66 L 122 66"
                                    } else {
                                        "M 306 66 L 122 66"
                                    }}
                                    marker-end="url(#arrow-emerald)"
                                />

                                // Linea Write-Back / Desalojo Dirty hacia DRAM
                                { if is_wb_bus {
                                    html! {
                                        <path
                                            class="bus-trace trace-wb trace-flow"
                                            d="M 368 86 L 368 102 L 684 102 L 684 86"
                                            marker-end="url(#arrow-amber)"
                                        />
                                    }
                                } else { html! {} }}
                            </g>
                        }
                    } else {
                        html! {}
                    }}

                    // Etiquetas de buses
                    <text x="138" y="36" text-anchor="middle" class="bus-lbl">{"ADDR"}</text>
                    <text x="290" y="36" text-anchor="middle" class="bus-lbl">{"IDX"}</text>
                    <text x="446" y="36" text-anchor="middle" class="bus-lbl">{"PROBE"}</text>
                    <text x="599" y="36" text-anchor="middle" class="bus-lbl">{"FETCH"}</text>
                    <text x="526" y="113" text-anchor="middle" class={if is_wb_bus { "bus-lbl lbl-amber" } else { "bus-lbl" }}>
                        {if is_wb_bus { "WRITE-BACK BUS (DIRTY FLUSH -> DRAM)" } else { "WRITE-BACK / EVICT BUS" }}
                    </text>

                    // NODO 0: CPU CORE
                    <g class={if clock > 0 { "svg-node node-cpu" } else { "svg-node" }}>
                        <rect x="8" y="14" width="114" height="72" rx="8" class="node-box" />
                        <text x="65" y="32" text-anchor="middle" class="node-k">{"CPU CORE"}</text>
                        <text x="65" y="51" text-anchor="middle" class="node-v mono">{format_hex(eng.reg_mar)}</text>
                        <text x="65" y="70" text-anchor="middle" class="node-s mono">
                            {if clock == 0 { "0.5 ns · IDLE".to_string() } else { eng.reg_rax.clone() }}
                        </text>
                    </g>

                    // NODO 1: DECODER
                    <g class={if clock > 0 { "svg-node node-dec" } else { "svg-node" }}>
                        <rect x="154" y="14" width="120" height="72" rx="8" class="node-box" />
                        <text x="214" y="32" text-anchor="middle" class="node-k">{"DECODER 6b"}</text>
                        <text x="214" y="51" text-anchor="middle" class="node-v mono">
                            {format!("T:0x{:X} S:{}", b.tag_val, b.index_val)}
                        </text>
                        <text x="214" y="70" text-anchor="middle" class="node-s mono">
                            {format!("Blq #{} · Off {}", b.block_addr, b.offset_val)}
                        </text>
                    </g>

                    // NODO 2: CACHE L1
                    <g class={node_l1_cls}>
                        <rect x="306" y="14" width="124" height="72" rx="8" class="node-box" />
                        <text x="368" y="32" text-anchor="middle" class="node-k">{"CACHE L1"}</text>
                        <text x="368" y="51" text-anchor="middle" class="node-v mono">
                            {format!("Set #{}", b.index_val)}
                        </text>
                        <text x="368" y="70" text-anchor="middle" class="node-s mono">
                            {if !l1_active {
                                "1.0 ns".to_string()
                            } else if l1_hit {
                                "HIT L1".to_string()
                            } else {
                                "MISS L1".to_string()
                            }}
                        </text>
                    </g>

                    // NODO 3: CACHE L2 / L3
                    <g class={node_l23_cls}>
                        <rect x="462" y="14" width="120" height="72" rx="8" class="node-box" />
                        <text x="522" y="32" text-anchor="middle" class="node-k">{"CACHE L2/L3"}</text>
                        <text x="522" y="51" text-anchor="middle" class="node-v mono">
                            {if use_l2_l3 {
                                format!("L2:{} L3:{}", eng.l2.len(), eng.l3.len())
                            } else {
                                "BYPASS".to_string()
                            }}
                        </text>
                        <text x="522" y="70" text-anchor="middle" class="node-s mono">
                            {if !use_l2_l3 { "L1 -> RAM" } else { "4 / 12 ns" }}
                        </text>
                    </g>

                    // NODO 4: DRAM / SSD
                    <g class={node_dram_cls}>
                        <rect x="616" y="14" width="136" height="72" rx="8" class="node-box" />
                        <text x="684" y="32" text-anchor="middle" class="node-k">{"DRAM / SSD"}</text>
                        <text x="684" y="51" text-anchor="middle" class="node-v mono">
                            {format!("C:{} D:{}", cached_count, dirty_count)}
                        </text>
                        <text x="684" y="70" text-anchor="middle" class="node-s mono">
                            {if is_page_fault { "SSD 10,000ns" } else { "DRAM 60 ns" }}
                        </text>
                    </g>

                    // PAQUETES ANIMADOS POR CADA CICLO DE RELOJ (key = clock reinicia la trayectoria GPU)
                    { if clock > 0 {
                        html! {
                            <g key={format!("pkt-layer-{}", clock)} style={format!("--pkt-dur: {}ms;", anim_ms)}>
                                // Paquete principal CPU -> jerarquia -> retorno CPU
                                <g class={classes!("bus-packet", pkt_class)}>
                                    <rect x="-25" y="-10" width="50" height="20" rx="5" class="pkt-pill" />
                                    <text x="0" y="3.5" text-anchor="middle" class="pkt-txt mono">
                                        {if is_page_fault { "SSD!".to_string() } else { format_hex(eng.reg_mar) }}
                                    </text>
                                </g>

                                // Paquete simultaneo de desalojo / Write-Back cuando se reemplaza un bloque
                                { if let Some(ev) = evict_notice {
                                    if ev.write_back_ram {
                                        html! {
                                            <g class="bus-packet pkt-wb-flush">
                                                <rect x="-26" y="-9" width="52" height="18" rx="4" class="pkt-pill-wb" />
                                                <text x="0" y="3" text-anchor="middle" class="pkt-txt mono">
                                                    {format!("WB#{}", ev.old_block)}
                                                </text>
                                            </g>
                                        }
                                    } else if use_l2_l3 {
                                        html! {
                                            <g class="bus-packet pkt-demote-l2">
                                                <rect x="-25" y="-9" width="50" height="18" rx="4" class="pkt-pill-evict" />
                                                <text x="0" y="3" text-anchor="middle" class="pkt-txt mono">
                                                    {format!("EV#{}", ev.old_block)}
                                                </text>
                                            </g>
                                        }
                                    } else {
                                        html! {}
                                    }
                                } else if eng.bus_flow == BusFlow::WriteBackToRam && l1_hit {
                                    // Write-Through flush en hit de escritura
                                    html! {
                                        <g class="bus-packet pkt-wb-flush">
                                            <rect x="-25" y="-9" width="50" height="18" rx="4" class="pkt-pill-wb" />
                                            <text x="0" y="3" text-anchor="middle" class="pkt-txt mono">
                                                {"WT->M"}
                                            </text>
                                        </g>
                                    }
                                } else {
                                    html! {}
                                }}
                            </g>
                        }
                    } else {
                        html! {}
                    }}
                </svg>
            </div>
        </section>
    }
}
