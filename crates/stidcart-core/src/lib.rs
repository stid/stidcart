//! Building EasyFlash cartridge images for the Commodore 64.
//!
//! The whole API is one owned value. Nothing here touches the filesystem: [`CartBuilder::build`]
//! returns bytes and the caller decides what to do with them. That is deliberate — the
//! predecessor's core was a global singleton, which is exactly why nothing could reuse it.
//!
//! ```
//! use stidcart_core::{CartBuilder, Program};
//!
//! let cart = CartBuilder::new()
//!     .with_ef_name("my cart")
//!     .with_program(Program::new("hello", vec![0x01, 0x08, 0x00]))
//!     .build()
//!     .unwrap();
//! assert_eq!(&cart[..16], b"C64 CARTRIDGE   ");
//! ```

pub mod crt;
pub mod directory;
pub mod firmware;
pub mod image;
pub mod petscii;
pub mod place;
pub mod read;
pub mod reserved;

pub use image::FlashImage;
pub use read::{inspect, load, CartInfo, ReadError};

use directory::{DirEntry, EntryType, ENTRY_LEN, MAX_ENTRIES};
use place::Placer;
use reserved::{DEFAULT_EF_NAME, DIRECTORY_OFFSET};

/// A program to place on the cartridge. `data` is the raw `.prg`, load address included.
#[derive(Clone, Debug)]
pub struct Program {
    pub name: String,
    pub data: Vec<u8>,
    /// Kept out of the menu the cartridge shows. The flag is part of the directory entry, so it
    /// survives a save and a reload — which is the whole reason [`read::load`] no longer refuses a
    /// cart that carries one (D-020).
    pub hidden: bool,
}

impl Program {
    pub fn new(name: &str, data: Vec<u8>) -> Self {
        Program {
            name: name.to_string(),
            data,
            hidden: false,
        }
    }

    pub fn with_hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BuildError {
    TooManyEntries {
        max: usize,
        got: usize,
    },
    OutOfSpace {
        name: String,
        size: usize,
        free: usize,
    },
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildError::TooManyEntries { max, got } => {
                write!(f, "{got} entries requested but the directory holds {max}")
            }
            BuildError::OutOfSpace { name, size, free } => write!(
                f,
                "'{name}' needs {size} bytes but only {free} are free on the cartridge"
            ),
        }
    }
}

impl std::error::Error for BuildError {}

/// Accumulates programs and produces a cartridge image.
#[derive(Clone, Debug)]
pub struct CartBuilder {
    ef_name: String,
    header_name: String,
    programs: Vec<Program>,
}

impl Default for CartBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl CartBuilder {
    pub fn new() -> Self {
        CartBuilder {
            ef_name: DEFAULT_EF_NAME.to_string(),
            header_name: format!("stidcart {}", env!("CARGO_PKG_VERSION")),
            programs: Vec::new(),
        }
    }

    /// The cartridge name the menu shows, stored inside the flash image.
    ///
    /// This is **not** the container header name — see [`CartBuilder::with_header_name`]. They are
    /// different fields at different offsets and conflating them is the single easiest way to
    /// produce an image that differs from a reference for a reason that has nothing to do with
    /// correctness.
    pub fn with_ef_name(mut self, name: &str) -> Self {
        self.ef_name = name.to_string();
        self
    }

    /// The 32-byte name in the `.crt` container header, which identifies the producing tool.
    /// Defaults to this crate's name and version.
    pub fn with_header_name(mut self, name: &str) -> Self {
        self.header_name = name.to_string();
        self
    }

    pub fn with_program(mut self, program: Program) -> Self {
        self.programs.push(program);
        self
    }

    pub fn programs(&self) -> &[Program] {
        &self.programs
    }

    /// Append a program. The directory keeps input order, so it lands last in the menu.
    pub fn push(&mut self, program: Program) {
        self.programs.push(program);
    }

    /// Remove the program at `index` and hand it back.
    ///
    /// This and the other index-addressed mutators panic on an out-of-range index, exactly as
    /// indexing a [`Vec`] does. A caller editing a cart knows how many entries it has — the
    /// command-line front end bounds-checks against [`CartBuilder::programs`] so it can name the
    /// index in its own error — and a second, unreachable check here would only be a second place
    /// for the message to be wrong.
    pub fn remove(&mut self, index: usize) -> Program {
        self.programs.remove(index)
    }

    pub fn set_name(&mut self, index: usize, name: &str) {
        self.programs[index].name = name.to_string();
    }

    pub fn set_hidden(&mut self, index: usize, hidden: bool) {
        self.programs[index].hidden = hidden;
    }

    /// Swap the program data at `index`, keeping the entry's name and its place in the order.
    pub fn replace_data(&mut self, index: usize, data: Vec<u8>) {
        self.programs[index].data = data;
    }

    /// The menu name — see [`CartBuilder::with_ef_name`] for why it is not the header name.
    pub fn set_ef_name(&mut self, name: &str) {
        self.ef_name = name.to_string();
    }

    pub fn set_header_name(&mut self, name: &str) {
        self.header_name = name.to_string();
    }

    /// The cartridge name the menu shows.
    pub fn ef_name(&self) -> &str {
        &self.ef_name
    }

    /// The name in the container header, identifying the tool that produced the image. A cart
    /// loaded from a file keeps the name it arrived with — see [`read::load`].
    pub fn header_name(&self) -> &str {
        &self.header_name
    }

    /// Lay out the flash image without wrapping it in a container.
    pub fn build_image(&self) -> Result<FlashImage, BuildError> {
        if self.programs.len() > MAX_ENTRIES {
            return Err(BuildError::TooManyEntries {
                max: MAX_ENTRIES,
                got: self.programs.len(),
            });
        }

        // Place largest first; the directory keeps input order regardless.
        let sizes: Vec<usize> = self.programs.iter().map(|p| p.data.len()).collect();
        let mut placer = Placer::new();
        let mut placements = vec![None; self.programs.len()];

        for i in place::placement_order(&sizes) {
            let free = placer.free_space();
            placements[i] =
                Some(
                    placer
                        .place_interleaved(sizes[i])
                        .ok_or_else(|| BuildError::OutOfSpace {
                            name: self.programs[i].name.clone(),
                            size: sizes[i],
                            free,
                        })?,
                );
        }

        let mut img = FlashImage::erased();
        reserved::write_eapi(&mut img);
        reserved::write_boot(&mut img);
        reserved::write_loader(&mut img);
        reserved::write_ef_name(&mut img, &self.ef_name);

        for (i, program) in self.programs.iter().enumerate() {
            let at = placements[i].expect("every program was placed above");
            img.write_program(at.bank as usize, at.offset as usize, &program.data);

            let entry = DirEntry {
                name: program.name.clone(),
                kind: EntryType::Prg,
                bank: at.bank,
                offset: at.offset,
                size: program.data.len() as u32,
                hidden: program.hidden,
            };
            img.write_at(DIRECTORY_OFFSET + i * ENTRY_LEN, &directory::encode(&entry));
        }

        img.write_at(
            DIRECTORY_OFFSET + self.programs.len() * ENTRY_LEN,
            &directory::end_marker(),
        );

        Ok(img)
    }

    /// The finished `.crt` bytes.
    pub fn build(&self) -> Result<Vec<u8>, BuildError> {
        Ok(crt::write(&self.build_image()?, &self.header_name))
    }
}
