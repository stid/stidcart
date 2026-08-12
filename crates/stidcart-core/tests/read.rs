//! AC-1, AC-3, AC-5..AC-10 and AC-12 — loading a cartridge image, and what it refuses.
//!
//! Every criterion in this work-item is `surfaces: none`, because the read path is a library
//! capability with no command-line surface (spec Out of scope). The cross-path audit skips `none`
//! rows outright, so these tests are the coverage rather than the audit being it.
//!
//! Several cases below build a deliberately broken image. They do it through the public image and
//! container writers rather than by hand-editing offsets, so a change to the format's layout moves
//! the test with it instead of leaving it asserting against a stale address.

use std::path::{Path, PathBuf};
use stidcart_core::directory::{self, DirEntry, EntryType};
use stidcart_core::read::{self, PacketProblem, ReadError, Unsupported};
use stidcart_core::{crt, reserved, CartBuilder};

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(fixtures_dir().join(name)).unwrap_or_else(|e| panic!("fixture {name}: {e}"))
}

/// A valid program entry, the starting point for the malformed-directory cases.
fn an_entry() -> DirEntry {
    DirEntry {
        name: "probe".into(),
        kind: EntryType::Prg,
        bank: 1,
        offset: 0,
        size: 2,
        hidden: false,
    }
}

/// Put raw slot bytes into the leading directory slots of an otherwise valid empty cart, and wrap
/// the result back into a container.
fn cart_with_slots(slots: &[[u8; directory::ENTRY_LEN]]) -> Vec<u8> {
    let mut img = CartBuilder::new()
        .build_image()
        .expect("an empty image builds");
    for (n, slot) in slots.iter().enumerate() {
        img.write_at(
            reserved::DIRECTORY_OFFSET + n * directory::ENTRY_LEN,
            slot.as_slice(),
        );
    }
    crt::write(&img, "probe")
}

/// AC-1 (none): a loaded image describes the programs that produced it.
#[test]
fn ac_1_a_loaded_image_describes_the_programs_that_produced_it() {
    let cart = read::load(&fixture("two.crt")).expect("two.crt should load");

    let names: Vec<&str> = cart.programs().iter().map(|p| p.name.as_str()).collect();
    assert_eq!(
        names,
        ["hello", "ramp"],
        "entries come back in directory order, which is the input order — not the largest-first \
         order the placer used to position them"
    );

    assert_eq!(
        cart.programs()[0].data,
        fixture("hello.prg"),
        "the recovered bytes include the two-byte load address the program begins with"
    );
    assert_eq!(cart.programs()[1].data, fixture("ramp.prg"));
}

/// AC-3 (none): an empty directory loads as a cartridge with no programs.
#[test]
fn ac_3_an_empty_directory_loads_as_a_cartridge_with_no_programs() {
    let empty = CartBuilder::new().build().expect("an empty cart builds");

    let cart = read::load(&empty).expect(
        "an end marker in slot 0 is an empty cartridge, not a missing directory. The reference \
         implementation refuses this on a break-ordering slip; D-007 fixes it rather than \
         reproducing it, and this tool produces exactly this image from an empty request",
    );
    assert!(cart.programs().is_empty());
}

/// AC-5 (none): a corrupt packet marker or a disagreeing length is refused.
#[test]
fn ac_5_a_corrupt_packet_marker_or_disagreeing_length_is_refused() {
    let good = fixture("single.crt");
    let first_packet = crt::HEADER_LEN;

    let mut bad_marker = good.clone();
    bad_marker[first_packet] = b'X';
    assert!(
        matches!(
            read::load(&bad_marker),
            Err(ReadError::BadPacket {
                problem: PacketProblem::Marker,
                ..
            })
        ),
        "the packet marker is validated. The reference implementation never checks it; D-007 \
         tightens this rather than inheriting the laxity"
    );

    // The total-length field at +4 and the data-length field at +14 describe the same packet, so
    // they must agree. Bumping one alone is the malformed case that decides whether the walk
    // advances by the stated total or by the derived length.
    let mut bad_len = good.clone();
    bad_len[first_packet + 7] = bad_len[first_packet + 7].wrapping_add(1);
    assert!(
        matches!(
            read::load(&bad_len),
            Err(ReadError::BadPacket {
                problem: PacketProblem::LengthDisagrees,
                ..
            })
        ),
        "total length must equal 16 + data length"
    );
}

/// AC-6 (none): a directory with no end marker anywhere is refused.
#[test]
fn ac_6_a_directory_with_no_end_marker_anywhere_is_refused() {
    let slot = directory::encode(&an_entry());
    let full = vec![slot; reserved::DIRECTORY_SLOTS];

    assert!(
        matches!(
            read::load(&cart_with_slots(&full)),
            Err(ReadError::DirectoryUnterminated)
        ),
        "every one of the {} slots holds a valid entry and none is the end marker. This rejection \
         is format-mandated rather than a quirk: accepting it would read as an empty cartridge and \
         destroy the directory on the next save",
        reserved::DIRECTORY_SLOTS
    );
}

/// AC-7 (none): cartridge and hidden entries are refused as unsupported.
#[test]
fn ac_7_cartridge_and_hidden_entries_are_refused_as_unsupported() {
    // Type 0x11 is a 16k cartridge. `EntryType` deliberately enumerates only the program types, so
    // this slot has to be built by hand.
    let mut cartridge = directory::encode(&an_entry());
    cartridge[16] = 0x11 | directory::RESERVED_FLAG_BITS;
    assert!(
        matches!(
            read::load(&cart_with_slots(&[cartridge, directory::end_marker()])),
            Err(ReadError::Unsupported(Unsupported::CartridgeEntry { .. }))
        ),
        "a cartridge entry is well-formed but not representable here, so it is refused as \
         unsupported rather than misparsed as a program"
    );

    let mut hidden = an_entry();
    hidden.hidden = true;
    assert!(
        matches!(
            read::load(&cart_with_slots(&[
                directory::encode(&hidden),
                directory::end_marker()
            ])),
            Err(ReadError::Unsupported(Unsupported::HiddenEntry { .. }))
        ),
        "the description has no way to express hiding, so loading it would silently drop the flag \
         and rebuild to different bytes"
    );

    // The distinguishability half of the criterion: a malformed image must not land in the same
    // bucket as a well-formed but unsupported one.
    let mut broken = fixture("single.crt");
    broken[crt::HEADER_LEN] = b'X';
    assert!(
        !matches!(read::load(&broken), Err(ReadError::Unsupported(_))),
        "a malformed image is refused, but not as 'unsupported' — a caller must be able to tell \
         'this file is broken' from 'this cartridge is fine, we cannot open it yet'"
    );
}

/// AC-8 (none): the container header name is recovered and never re-stamped.
#[test]
fn ac_8_the_container_header_name_is_recovered_and_never_re_stamped() {
    let cart = read::load(&fixture("single.crt")).expect("single.crt should load");
    assert_eq!(
        cart.header_name(),
        "ndefpack 1.0.1",
        "the header records the tool that produced the image"
    );

    let rebuilt = cart.build().expect("a loaded cart rebuilds");
    let name = &rebuilt[crt::NAME_OFFSET..crt::NAME_OFFSET + crt::NAME_LEN];
    assert!(
        name.starts_with(b"ndefpack 1.0.1"),
        "rebuilding carries the original name through rather than stamping this tool's own. \
         Otherwise every cart that merely passes through the tool mutates, and a version bump \
         rewrites images it never really changed"
    );
}

/// AC-9 (none): the header name is not character-swapped while entry and cartridge names are.
#[test]
fn ac_9_the_header_name_is_not_swapped_while_entry_and_cartridge_names_are() {
    let cart = read::load(&fixture("named.crt")).expect("named.crt should load");

    assert_eq!(
        cart.header_name(),
        "ndefpack 1.0.1",
        "the container header name is stored in the host's encoding. Running the machine's case \
         swap over it yields NDEFPACK 1.0.1 and breaks parity on every reference image — three \
         name fields, two encodings, and D-006 exists because two of them were conflated once"
    );

    assert_eq!(
        cart.ef_name(),
        "weird name",
        "the cartridge name is stored case-swapped and comes back as it was given"
    );
    assert_eq!(
        cart.programs()[0].name,
        "hello",
        "entry names are stored case-swapped too"
    );
}

/// AC-10 (none): reserved firmware differing from the embedded blobs is refused as unsupported.
#[test]
fn ac_10_reserved_firmware_differing_from_the_embedded_blobs_is_refused() {
    let mut img = CartBuilder::new()
        .build_image()
        .expect("an empty image builds");
    img.write_at(reserved::EAPI_OFFSET, &[0x00; 32]);
    let tampered = crt::write(&img, "probe");

    assert!(
        matches!(
            read::load(&tampered),
            Err(ReadError::Unsupported(
                Unsupported::ReservedRegionMismatch { .. }
            ))
        ),
        "rebuilding writes the firmware this tool embeds, so accepting an image whose reserved \
         regions hold something else would turn byte identity into a failure surfacing far from \
         its cause"
    );
}

/// AC-12 (none): a size or location reaching outside the image is refused.
#[test]
fn ac_12_a_size_or_location_reaching_outside_the_image_is_refused() {
    // A packet naming a bank past the top of flash.
    let mut wild_bank = fixture("single.crt");
    wild_bank[crt::HEADER_LEN + 10] = 0x00;
    wild_bank[crt::HEADER_LEN + 11] = 200;
    assert!(
        matches!(read::load(&wild_bank), Err(ReadError::OutOfBounds { .. })),
        "bank 200 is past the end of a 64-bank image; the bank is file-controlled and reaches an \
         index, so it is checked before it is used"
    );

    // A packet claiming more data than the file actually contains.
    let good = fixture("single.crt");
    let truncated = good[..good.len() - 64].to_vec();
    assert!(
        matches!(
            read::load(&truncated),
            Err(ReadError::Truncated { .. } | ReadError::OutOfBounds { .. })
        ),
        "a packet may not describe more data than the supplied image holds"
    );

    // An entry whose continuation runs off the top of flash. The interleave steps ROML -> ROMH ->
    // next bank, so from the last bank the next step wraps back into the reserved region;
    // recovering that as program data would be silently wrong rather than merely broken.
    let mut overrun = an_entry();
    overrun.bank = 63;
    overrun.offset = 0x1ff0;
    overrun.size = 0xe000;
    assert!(
        matches!(
            read::load(&cart_with_slots(&[
                directory::encode(&overrun),
                directory::end_marker()
            ])),
            Err(ReadError::OutOfBounds { .. })
        ),
        "an entry continuing past the last bank is refused, not wrapped around into bank 0"
    );
}
