use yew::prelude::*;

use crate::sim::types::{format_bin, format_hex, AddressBreakdown, BitField, CacheConfig};

#[derive(Properties, PartialEq)]
pub struct AddressDecoderProps {
    pub breakdown: AddressBreakdown,
    pub config: CacheConfig,
}

#[function_component(AddressDecoder)]
pub fn address_decoder(props: &AddressDecoderProps) -> Html {
    let b = &props.breakdown;
    let tag_cells: Vec<u8> = b.bits.iter().filter(|c| c.field == BitField::Tag).map(|c| c.bit).collect();
    let idx_cells: Vec<u8> = b.bits.iter().filter(|c| c.field == BitField::Index).map(|c| c.bit).collect();
    let off_cells: Vec<u8> = b.bits.iter().filter(|c| c.field == BitField::Offset).map(|c| c.bit).collect();

    html! {
        <section class="panel" aria-label="Decodificador de Direccion">
            <div class="panel-header">
                <h2 class="panel-title">{"Decodificador de Direccion [ TAG | INDEX | OFFSET ]"}</h2>
                <span class="badge badge-cyan">
                    {format!("{} | Bloque #{}", format_hex(b.addr), b.block_addr)}
                </span>
            </div>

            <div class="bit-strip">
                <div class="bit-segment tag">
                    <div class="seg-head">
                        <span>{"TAG"}</span>
                        <span class="mono">{format!("{} bits", b.tag_bits)}</span>
                    </div>
                    <div class="bit-cells">
                        { if tag_cells.is_empty() {
                            html! { <div class="bit-box">{"-"}</div> }
                        } else {
                            html! { for tag_cells.iter().map(|bit| html! { <div class="bit-box">{bit}</div> }) }
                        }}
                    </div>
                    <div class="seg-foot">
                        {format!("0x{:X} (bin {})", b.tag_val, format_bin(b.tag_val, b.tag_bits))}
                    </div>
                </div>

                <div class="bit-segment index">
                    <div class="seg-head">
                        <span>{"INDEX (Set)"}</span>
                        <span class="mono">{format!("{} bits", b.index_bits)}</span>
                    </div>
                    <div class="bit-cells">
                        { if idx_cells.is_empty() {
                            html! { <div class="bit-box">{"0"}</div> }
                        } else {
                            html! { for idx_cells.iter().map(|bit| html! { <div class="bit-box">{bit}</div> }) }
                        }}
                    </div>
                    <div class="seg-foot">
                        { if b.index_bits == 0 {
                            "Set 0 (Full Assoc.)".to_string()
                        } else {
                            format!("Set {} (bin {})", b.index_val, format_bin(b.index_val, b.index_bits))
                        }}
                    </div>
                </div>

                <div class="bit-segment offset">
                    <div class="seg-head">
                        <span>{"OFFSET"}</span>
                        <span class="mono">{format!("{} bits", b.offset_bits)}</span>
                    </div>
                    <div class="bit-cells">
                        { if off_cells.is_empty() {
                            html! { <div class="bit-box">{"-"}</div> }
                        } else {
                            html! { for off_cells.iter().map(|bit| html! { <div class="bit-box">{bit}</div> }) }
                        }}
                    </div>
                    <div class="seg-foot">
                        { if b.offset_bits == 0 {
                            "1 palabra/bloque".to_string()
                        } else {
                            format!("Palabra {} (bin {})", b.offset_val, format_bin(b.offset_val, b.offset_bits))
                        }}
                    </div>
                </div>
            </div>

            <div style="margin-top:0.5rem; font-size:0.71rem; color:var(--text-secondary);" class="mono">
                {format!(
                    "Direccion 6b = Tag({}b) + Index(log2({})={}b) + Offset(log2({})={}b) -> Consulta en Set #{}",
                    b.tag_bits,
                    props.config.num_sets(),
                    b.index_bits,
                    props.config.block_size,
                    b.offset_bits,
                    b.index_val
                )}
            </div>
        </section>
    }
}
