//! Entry metadata: a title, year, author, note and folder per entry, stored in one hidden program
//! entry, `STIDCART.META` (D-054, D-055).
//!
//! This is a stidcart extension, not an EasyFlash or ndefpack fact. Its layout is the section
//! "stidcart extension: entry metadata" in `docs/format/easyflash-cart-format.md`; every offset
//! below comes from there. Encoding and decoding are pure. Which entry holds the bytes, and which
//! records attach to which entry, is decided by the directory walk in [`crate::read`] and the
//! build in [`crate::CartBuilder`].

use crate::directory::NAME_LEN;
use crate::petscii;

/// The entry's name as the builder holds it. `ascii_to_petscii` swaps case, so the 16 bytes on
/// flash read `STIDCART.META`.
pub const META_NAME: &str = "stidcart.meta";
/// The first eight payload bytes.
pub const MAGIC: &[u8; 8] = b"STIDMETA";
/// The layout version this build reads and writes.
pub const VERSION: u8 = 1;
/// Where the C64 side copies the payload: at most `$4000-$7FFF`.
pub const LOAD_ADDRESS: u16 = 0x4000;
/// The most payload a copy to [`LOAD_ADDRESS`] can hold.
pub const MAX_PAYLOAD: usize = 0x4000;

const HEADER_LEN: usize = 13;
const GROUP_LEN: usize = 16;
/// Length byte, index, name, group number, then four length bytes.
const RECORD_FIXED: usize = 1 + 1 + NAME_LEN + 1 + 4;

/// One of the five things an entry can say about itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Title,
    Year,
    Author,
    Note,
    Folder,
}

impl Field {
    pub const ALL: [Field; 5] = [
        Field::Title,
        Field::Year,
        Field::Author,
        Field::Note,
        Field::Folder,
    ];

    /// The name `edit`, the import file and the window use for it.
    pub fn name(self) -> &'static str {
        match self {
            Field::Title => "title",
            Field::Year => "year",
            Field::Author => "author",
            Field::Note => "note",
            Field::Folder => "folder",
        }
    }

    /// The most characters it holds.
    pub fn limit(self) -> usize {
        match self {
            Field::Title | Field::Note => 48,
            Field::Year => 4,
            Field::Author => 24,
            Field::Folder => 16,
        }
    }

    /// The field a name means. `group` is the folder's name before D-060, still accepted so
    /// `--group` and an import's `group` key keep working.
    pub fn from_name(name: &str) -> Option<Field> {
        match name {
            "group" => Some(Field::Folder),
            _ => Field::ALL.into_iter().find(|f| f.name() == name),
        }
    }
}

/// An entry's metadata. Every field is empty or valid: [`Meta::set`] is the only way in.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Meta {
    fields: [String; 5],
}

impl Meta {
    pub fn get(&self, field: Field) -> &str {
        &self.fields[field as usize]
    }

    /// Set `field` to `value`, or clear it with an empty one. Refused, leaving the field as it
    /// was, when the value is too long or holds a character with no PETSCII equivalent.
    pub fn set(&mut self, field: Field, value: &str) -> Result<(), MetaError> {
        check(field, value)?;
        self.fields[field as usize] = value.to_string();
        Ok(())
    }

    /// No field is set, so the entry has no record.
    pub fn is_empty(&self) -> bool {
        self.fields.iter().all(String::is_empty)
    }
}

/// Whether `value` may be stored in `field`. Text maps to PETSCII when it comes back unchanged,
/// which is exactly printable ASCII (D-055).
pub fn check(field: Field, value: &str) -> Result<(), MetaError> {
    if let Some(ch) = value.chars().find(|c| !(' '..='~').contains(c)) {
        return Err(MetaError::Unmappable { field, ch });
    }
    let len = value.len();
    if len > field.limit() {
        return Err(MetaError::TooLong {
            field,
            len,
            max: field.limit(),
        });
    }
    Ok(())
}

/// Why a metadata value cannot be set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetaError {
    TooLong {
        field: Field,
        len: usize,
        max: usize,
    },
    Unmappable {
        field: Field,
        ch: char,
    },
    /// The cart's metadata is in a layout this build can't rewrite, so it is kept as stored.
    Kept {
        reason: String,
    },
}

/// Why a change to the cart's folder list is refused (D-063).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FolderError {
    /// The name breaks the folder rules, or the cart's metadata is kept as stored.
    Invalid(MetaError),
    Empty,
    Exists(String),
    Unknown(String),
    Position {
        to: usize,
        len: usize,
    },
    /// The folder table holds at most 255 names.
    TooMany,
}

impl std::fmt::Display for FolderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FolderError::Invalid(e) => e.fmt(f),
            FolderError::Empty => write!(f, "a folder needs a name"),
            FolderError::Exists(name) => write!(f, "a folder named '{name}' already exists"),
            FolderError::Unknown(name) => write!(f, "the cart has no folder named '{name}'"),
            FolderError::Position { to, len } => write!(
                f,
                "no folder position {to}; the cart has {len} {}",
                if *len == 1 { "folder" } else { "folders" }
            ),
            FolderError::TooMany => write!(f, "a cart holds at most 255 folders"),
        }
    }
}

impl std::error::Error for FolderError {}

impl std::fmt::Display for MetaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetaError::TooLong { field, len, max } => write!(
                f,
                "the {} is {len} characters long, and a {} holds at most {max}",
                field.name(),
                field.name()
            ),
            MetaError::Unmappable { field, ch } => write!(
                f,
                "the {} holds '{ch}', which has no PETSCII character",
                field.name()
            ),
            MetaError::Kept { reason } => write!(
                f,
                "the cart's metadata can't be changed: {reason}, so it is kept as stored"
            ),
        }
    }
}

impl std::error::Error for MetaError {}

/// A metadata entry this build doesn't read, carried byte for byte (D-055).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeptMeta {
    /// The entry's whole data, load address included.
    pub data: Vec<u8>,
    /// Why it isn't read: "layout version 9 is unknown to this stidcart".
    pub reason: String,
}

/// One record to write: the entry at `index`, its name as its slot will store it.
pub(crate) struct Record<'a> {
    pub index: usize,
    pub name: [u8; NAME_LEN],
    pub meta: &'a Meta,
}

/// The entry's data for the cart's `folders`, in order (empty ones included), and `records`, which
/// run in ascending index: the load address, then the payload. A record's group number is its
/// folder's position in `folders` (D-063). The caller checks the payload against [`MAX_PAYLOAD`].
pub(crate) fn encode(folders: &[String], records: &[Record]) -> Vec<u8> {
    let groups: Vec<&str> = folders.iter().map(String::as_str).collect();

    let mut out = LOAD_ADDRESS.to_le_bytes().to_vec();
    out.extend_from_slice(MAGIC);
    out.push(VERSION);
    out.push(groups.len() as u8);
    out.push(records.len() as u8);
    out.extend_from_slice(&((HEADER_LEN + GROUP_LEN * groups.len()) as u16).to_le_bytes());
    for g in &groups {
        let mut name = petscii::ascii_to_petscii(g);
        name.resize(GROUP_LEN, 0);
        out.extend_from_slice(&name);
    }
    for r in records {
        let group = r.meta.get(Field::Folder);
        let number = groups
            .iter()
            .position(|g| *g == group)
            .map_or(0, |at| at + 1);
        let texts: Vec<Vec<u8>> = [Field::Title, Field::Year, Field::Author, Field::Note]
            .iter()
            .map(|f| petscii::ascii_to_petscii(r.meta.get(*f)))
            .collect();
        let len = RECORD_FIXED + texts.iter().map(Vec::len).sum::<usize>();
        out.push(len as u8);
        out.push(r.index as u8);
        out.extend_from_slice(&r.name);
        out.push(number as u8);
        for t in texts {
            out.push(t.len() as u8);
            out.extend_from_slice(&t);
        }
    }
    out
}

/// What a candidate entry's data turned out to hold.
#[derive(Debug)]
pub(crate) enum Decoded {
    /// No magic: an ordinary program that happens to share the name.
    NotMeta,
    /// Layout version 1: the cart's folders in stored order, then each record's index, stored name
    /// bytes and fields, in stored order.
    Records(Vec<String>, Records),
    /// The magic, in a layout this build can't read.
    Kept(KeptMeta),
}

/// Read a candidate entry's data, load address included.
pub(crate) fn decode(data: &[u8]) -> Decoded {
    let Some(payload) = data.get(2..) else {
        return Decoded::NotMeta;
    };
    if !payload.starts_with(MAGIC) {
        return Decoded::NotMeta;
    }
    let kept = |reason: String| {
        Decoded::Kept(KeptMeta {
            data: data.to_vec(),
            reason,
        })
    };
    match payload.get(MAGIC.len()) {
        Some(&VERSION) => {}
        Some(&v) => return kept(format!("layout version {v} is unknown to this stidcart")),
        None => return kept("the layout version is missing".into()),
    }
    match parse_v1(payload) {
        Ok((folders, records)) => Decoded::Records(folders, records),
        Err(why) => kept(format!("layout version 1 doesn't parse: {why}")),
    }
}

type Records = Vec<(usize, [u8; NAME_LEN], Meta)>;

fn parse_v1(p: &[u8]) -> Result<(Vec<String>, Records), String> {
    if p.len() < HEADER_LEN {
        return Err("the header is cut short".into());
    }
    let (g, r) = (p[9] as usize, p[10] as usize);
    let first = u16::from_le_bytes([p[11], p[12]]) as usize;
    let groups_end = HEADER_LEN + GROUP_LEN * g;
    if groups_end > p.len() || first < groups_end || first > p.len() {
        return Err("the group table doesn't fit".into());
    }
    let groups: Vec<String> = p[HEADER_LEN..groups_end]
        .chunks(GROUP_LEN)
        .map(petscii::petscii_to_ascii)
        .collect();

    let mut records = Vec::with_capacity(r);
    let mut at = first;
    for n in 0..r {
        let bad = |what: &str| format!("record {n} {what}");
        let len = *p.get(at).ok_or_else(|| bad("is missing"))? as usize;
        let rec = p
            .get(at..at + len)
            .filter(|rec| rec.len() >= RECORD_FIXED)
            .ok_or_else(|| bad("doesn't fit"))?;
        let index = rec[1] as usize;
        let name: [u8; NAME_LEN] = rec[2..2 + NAME_LEN].try_into().expect("sixteen bytes");
        let group = rec[2 + NAME_LEN] as usize;

        let mut meta = Meta::default();
        if group > 0 {
            let g = groups.get(group - 1).ok_or_else(|| bad("names no group"))?;
            meta.set(Field::Folder, g)
                .map_err(|e| bad(&e.to_string()))?;
        }
        let mut pos = 3 + NAME_LEN;
        for f in [Field::Title, Field::Year, Field::Author, Field::Note] {
            let n = *rec.get(pos).ok_or_else(|| bad("is cut short"))? as usize;
            let text = rec
                .get(pos + 1..pos + 1 + n)
                .ok_or_else(|| bad("is cut short"))?;
            meta.set(f, &petscii::petscii_to_ascii(text))
                .map_err(|e| bad(&e.to_string()))?;
            pos += 1 + n;
        }
        if pos != len {
            return Err(bad("has a wrong length"));
        }
        records.push((index, name, meta));
        at += len;
    }
    Ok((groups, records))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta(pairs: &[(Field, &str)]) -> Meta {
        let mut m = Meta::default();
        for (f, v) in pairs {
            m.set(*f, v).unwrap();
        }
        m
    }

    fn name(s: &[u8]) -> [u8; NAME_LEN] {
        let mut n = [0; NAME_LEN];
        n[..s.len()].copy_from_slice(s);
        n
    }

    #[test]
    fn the_layout_is_the_one_the_format_doc_gives() {
        let a = meta(&[(Field::Title, "Ab"), (Field::Folder, "Games")]);
        let b = meta(&[(Field::Year, "1984"), (Field::Folder, "Games")]);
        let data = encode(
            &["Games".to_string()],
            &[
                Record {
                    index: 0,
                    name: name(b"ONE"),
                    meta: &a,
                },
                Record {
                    index: 3,
                    name: name(b"TWO"),
                    meta: &b,
                },
            ],
        );
        let mut want = vec![0x00, 0x40];
        want.extend_from_slice(b"STIDMETA");
        want.extend_from_slice(&[1, 1, 2, 29, 0]);
        want.extend_from_slice(b"gAMES\0\0\0\0\0\0\0\0\0\0\0");
        want.extend_from_slice(&[25, 0]);
        want.extend_from_slice(&name(b"ONE"));
        want.extend_from_slice(&[1, 2, b'a', b'B', 0, 0, 0]);
        want.extend_from_slice(&[27, 3]);
        want.extend_from_slice(&name(b"TWO"));
        want.extend_from_slice(&[1, 0, 4, b'1', b'9', b'8', b'4', 0, 0]);
        assert_eq!(data, want);

        match decode(&data) {
            Decoded::Records(folders, r) => {
                assert_eq!(folders, ["Games"]);
                assert_eq!(r[0], (0, name(b"ONE"), a));
                assert_eq!(r[1], (3, name(b"TWO"), b));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn without_the_magic_it_is_not_metadata() {
        assert!(matches!(decode(&[0x01, 0x08, 1, 2, 3]), Decoded::NotMeta));
        assert!(matches!(decode(&[]), Decoded::NotMeta));
    }

    #[test]
    fn a_damaged_version_1_payload_is_kept() {
        let a = meta(&[(Field::Title, "Ab")]);
        let mut data = encode(
            &[],
            &[Record {
                index: 0,
                name: name(b"ONE"),
                meta: &a,
            }],
        );
        data.truncate(data.len() - 2);
        assert!(matches!(decode(&data), Decoded::Kept(k) if k.reason.contains("doesn't parse")));
    }

    #[test]
    fn values_must_fit_and_map_to_petscii() {
        let mut m = Meta::default();
        assert!(m.set(Field::Folder, &"g".repeat(16)).is_ok());
        assert_eq!(
            m.set(Field::Folder, &"g".repeat(17)),
            Err(MetaError::TooLong {
                field: Field::Folder,
                len: 17,
                max: 16
            })
        );
        assert_eq!(
            m.set(Field::Author, "Zoë"),
            Err(MetaError::Unmappable {
                field: Field::Author,
                ch: 'ë'
            })
        );
        assert!(m.set(Field::Note, "tab\there").is_err());
        assert_eq!(
            m.get(Field::Folder),
            "g".repeat(16),
            "a refusal leaves the field"
        );
        for c in ' '..='~' {
            let s = c.to_string();
            assert_eq!(petscii::petscii_to_ascii(&petscii::ascii_to_petscii(&s)), s);
        }
    }
}
