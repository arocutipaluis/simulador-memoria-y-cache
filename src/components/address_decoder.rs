use yew::prelude::*;

use crate::sim::types::{format_bin, format_hex, AddressBreakdown, BitField, CacheConfig};

#[derive(Properties, PartialEq)]
pub struct AddressDecoderProps {
    pub breakdown: AddressBreakdown,
    pub config: CacheConfig,
    pub clock: u64,
}

#[function_component(AddressDecoder)]
pub fn address_decoder(props: &AddressDecoderProps) -> Html {
    let b = &props.breakdown;
    let tag_cells: Vec<u8> = b
        .bits
        .iter()
        .filter(|c| c.field == BitField::Tag)
        .map(|c| c.bit)
        .collect();
    let idx_cells: Vec<u8> = b
        .bits
        .iter()
        .filter(|c| c.field == BitField::Index)
        .map(|c| c.bit)
        .collect();
    let off_cells: Vec<u8> = b
        .bits
        .iter()
        .filter(|c| c.field == BitField::Offset)
        .map(|c| c.bit)
        .collect();

    html! {
        <section class="panel compact-panel" aria-label="Decodificador de Direccion">
            <div class="panel-header compact-header">
                <h2 class="panel-title">{"Decodificador [ TAG | INDEX | OFFSET ]"}</h2>
                <span class="mono formula-inline">
                    {format!(
                        "{} (Blq #{}) = {}b Tag + {}b Set({}) + {}b Off",
                        format_hex(b.addr),
                        b.block_addr,
                        b.tag_bits,
                        b.index_bits,
                        props.config.num_sets(),
                        b.offset_bits
                    )}
                </span>
            </div>

            <div class="bit-strip" key={format!("dec-{}", props.clock)}>
                <div class="bit-segment tag">
                    <div class="seg-head">
                        <span>{"TAG"}</span>
                        <span class="mono">{format!("0x{:X} ({})", b.tag_val, format_bin(b.tag_val, b.tag_bits))}</span>
                    </div>
                    <div class="bit-cells">
                        { if tag_cells.is_empty() {
                            html! { <div class="bit-box">{"-"}</div> }
                        } else {
                            html! { for tag_cells.iter().map(|bit| html! { <div class="bit-box">{bit}</div> }) }
                        }}
                    </div>
                </div>

                <div class="bit-segment index">
                    <div class="seg-head">
                        <span>{"INDEX"}</span>
                        <span class="mono">
                            {if b.index_bits == 0 {
                                "Set 0 (Full)".to_string()
                            } else {
                                format!("Set {} ({})", b.index_val, format_bin(b.index_val, b.index_bits))
                            }}
                        </span>
                    </div>
                    <div class="bit-cells">
                        { if idx_cells.is_empty() {
                            html! { <div class="bit-box">{"0"}</div> }
                        } else {
                            html! { for idx_cells.iter().map(|bit| html! { <div class="bit-box">{bit}</div> }) }
                        }}
                    </div>
                </div>

                <div class="bit-segment offset">
                    <div class="seg-head">
                        <span>{"OFFSET"}</span>
                        <span class="mono">
                            {if b.offset_bits == 0 {
                                "Pal 0".to_string()
                            } else {
                                format!("Pal {} ({})", b.offset_val, format_bin(b.offset_val, b.offset_bits))
                            }}
                        </span>
                    </div>
                    <div class="bit-cells">
                        { if off_cells.is_empty() {
                            html! { <div class="bit-box">{"-"}</div> }
                        } else {
                            html! { for off_cells.iter().map(|bit| html! { <div class="bit-box">{bit}</div> }) }
                        }}
                    </div>
                </div>
            </div>
        </section>
    }
}
