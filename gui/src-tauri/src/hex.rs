//! What the hex window is sent: the open cart's bytes in rows and the ranges each part owns.
//!
//! The values are `stidcart_core::hexmap`'s; this adds only the wire format and the words an owner
//! is named with. Both read the document's current step, so the window shows what saving would
//! write, as the listing does (D-035).

use serde::Serialize;
use stidcart_core::hexmap::{self, Owner, LINES, LINE_BYTES};
use stidcart_core::image::Half;

use crate::document::Document;

/// The most rows one request answers with. The window asks for what is in view.
pub const MAX_LINES: usize = 512;

/// Mirrored by `HexInfo` in `gui/src/lib/types.ts`, pinned by `fixtures/hex.json`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HexInfoDto {
    pub file: String,
    /// Rows in the whole view, and bytes in each.
    pub lines: usize,
    pub line_bytes: usize,
    /// Owned byte ranges, in order and never overlapping.
    pub regions: Vec<RegionDto>,
    /// The entry selected in the listing, if any.
    pub selected: Option<usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionDto {
    /// Position of the first byte: row × bytes per row + column.
    pub start: usize,
    pub len: usize,
    /// `loader`, `directory`, `eapi`, `name`, `boot`, `entry` or `meta`.
    pub kind: String,
    /// The owner in words.
    pub label: String,
    /// An entry's kind family, which picks its colour as in the listing (D-045).
    pub family: Option<String>,
    /// An entry's index, as the listing numbers it.
    pub entry: Option<usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HexLineDto {
    pub bank: u8,
    /// `L` or `H`.
    pub half: String,
    pub addr: u16,
    pub bytes: Vec<u8>,
    pub text: String,
}

/// Describe the view of the cart's current step.
pub fn info(doc: &Document, selected: Option<usize>) -> Result<HexInfoDto, String> {
    let dto = doc.dto();
    let regions = hexmap::regions(doc.bytes())
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|r| {
            let (kind, label, family, entry) = match r.owner {
                Owner::Loader => ("loader", "Menu loader".to_string(), None, None),
                Owner::Directory => ("directory", "Directory".to_string(), None, None),
                Owner::Eapi => ("eapi", "EAPI".to_string(), None, None),
                Owner::Name => ("name", "Cart name".to_string(), None, None),
                Owner::Boot => ("boot", "Boot code".to_string(), None, None),
                Owner::Meta => ("meta", "Metadata".to_string(), None, None),
                Owner::Entry(index) => {
                    let row = &dto.listing.rows[index];
                    (
                        "entry",
                        format!("{index} · {} ({})", row.name, row.kind),
                        Some(row.family.clone()),
                        Some(index),
                    )
                }
            };
            RegionDto {
                start: r.start,
                len: r.len,
                kind: kind.to_string(),
                label,
                family,
                entry,
            }
        })
        .collect();
    Ok(HexInfoDto {
        file: dto.file,
        lines: LINES,
        line_bytes: LINE_BYTES,
        regions,
        selected,
    })
}

/// Up to [`MAX_LINES`] rows of the cart's current step, from row `first`.
pub fn lines(doc: &Document, first: usize, count: usize) -> Result<Vec<HexLineDto>, String> {
    Ok(hexmap::lines(doc.bytes(), first, count.min(MAX_LINES))
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|l| HexLineDto {
            bank: l.bank,
            half: match l.half {
                Half::RomL => "L",
                Half::RomH => "H",
            }
            .to_string(),
            addr: l.addr,
            bytes: l.bytes.to_vec(),
            text: l.text,
        })
        .collect())
}
