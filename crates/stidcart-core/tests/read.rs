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
        matches!(
            read::load(&wild_bank),
            Err(ReadError::BadPacket {
                problem: PacketProblem::Bank,
                ..
            })
        ),
        "bank 200 is past the end of a 64-bank image; the bank is file-controlled and reaches an \
         index, so it is checked before it is used. A container fault reports a file offset, which \
         is why it is a BadPacket rather than the entry-level OutOfBounds below — the two carry \
         offsets in different coordinate spaces"
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

/// AC-6 (none): inspecting reports the positions the file records, not the ones a rebuild would
/// choose.
///
/// This is the whole reason `inspect` exists beside `load`. `load` returns a [`CartBuilder`], which
/// holds no positions at all — a rebuild re-places from scratch. A `list` built on that would print
/// regenerated coordinates under a heading that claims to describe the file, which is wrong in a way
/// nothing downstream could catch. So the case below places an entry where this tool's placer never
/// would: a 4-byte program goes to bank 1 offset 0 on every fresh build, and this one is at bank 40
/// in ROMH. An implementation that regenerates reports bank 1.
#[test]
fn ac_6_inspect_reports_the_positions_the_file_records() {
    let mut planted = an_entry();
    planted.bank = 40;
    planted.offset = 0x2000;
    planted.size = 4;

    let cart = cart_with_slots(&[directory::encode(&planted), directory::end_marker()]);
    let info = read::inspect(&cart).expect("a well-formed image inspects");

    assert_eq!(
        info.entries,
        vec![planted],
        "the directory as the file records it"
    );

    // The negative control, asserted rather than described: the same entry through the rebuilding
    // path lands somewhere else entirely.
    let rebuilt = read::load(&cart)
        .expect("it loads too")
        .build_image()
        .unwrap();
    let slot: [u8; directory::ENTRY_LEN] = rebuilt.as_bytes()
        [reserved::DIRECTORY_OFFSET..reserved::DIRECTORY_OFFSET + directory::ENTRY_LEN]
        .try_into()
        .unwrap();
    let directory::Slot::Program(replaced) = directory::decode(&slot) else {
        panic!("the rebuilt directory should still hold a program");
    };
    assert_ne!(
        (replaced.bank, replaced.offset),
        (40, 0x2000),
        "if a rebuild happened to preserve the position, this test proves nothing"
    );
}

/// AC-6 (none): and it agrees with the reference images, where the two views do coincide.
#[test]
fn ac_6_inspect_agrees_with_the_reference_images() {
    let info = read::inspect(&fixture("two.crt")).expect("two.crt inspects");

    let names: Vec<&str> = info.entries.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(
        names,
        ["hello", "ramp"],
        "directory order, as loading reports it"
    );
    assert_eq!(info.ef_name, "two entries");
    assert_eq!(info.header_name, "ndefpack 1.0.1");
    assert!(
        info.entries
            .iter()
            .all(|e| e.kind == EntryType::Prg && !e.hidden),
        "the reference images hold plain visible programs only"
    );
}

/// Free space on a top-packed image is the capacity less what the entries occupy.
#[test]
fn free_space_on_a_top_packed_image_is_capacity_less_the_entries() {
    let empty = read::inspect(&fixture("single.crt")).expect("single.crt inspects");
    let occupied: usize = empty.entries.iter().map(|e| e.size as usize).sum();
    assert_eq!(empty.free_space(), 63 * 8192 * 2 - occupied);
}

/// A half-confined entry is refused rather than read with the interleaved layout.
///
/// Types 0x02 and 0x03 confine an entry to one half of a bank, and nothing here consults
/// `DirEntry.kind` — so before this was refused, such an entry was read by stepping ROML -> ROMH ->
/// next bank and came back as part its own bytes and part whatever occupied the other half, then
/// rebuilt as a plain interleaved program. Silently corrupt in both directions. It is refused
/// rather than implemented because this tool cannot produce one, and the layout rule for them
/// belongs to the format spec rather than to a guess made here.
#[test]
fn a_half_confined_entry_is_refused_rather_than_read_with_the_wrong_layout() {
    for kind in [EntryType::PrgRomL, EntryType::PrgRomH] {
        let mut confined = an_entry();
        confined.kind = kind;
        confined.size = 0x3000;
        let cart = cart_with_slots(&[directory::encode(&confined), directory::end_marker()]);

        assert!(
            matches!(
                read::load(&cart),
                Err(ReadError::Unsupported(Unsupported::ConfinedEntry { .. }))
            ),
            "{kind:?} must be refused, not read with the interleaved progression"
        );
        assert!(
            matches!(
                read::inspect(&cart),
                Err(ReadError::Unsupported(Unsupported::ConfinedEntry { .. }))
            ),
            "{kind:?} is not describable either — listing it would advertise support that the \
             read path does not have"
        );
    }
}

/// A hidden entry is listed, and refused only by the path that would rebuild it.
#[test]
fn inspect_lists_a_hidden_entry_that_load_refuses() {
    let mut hidden = an_entry();
    hidden.name = "easyloader ocm".into();
    hidden.hidden = true;
    let mut visible = an_entry();
    visible.name = "visible".into();
    visible.bank = 2;

    // The hidden entry sits first, so a walk that aborts on it loses the visible one behind it.
    let cart = cart_with_slots(&[
        directory::encode(&hidden),
        directory::encode(&visible),
        directory::end_marker(),
    ]);

    let info = read::inspect(&cart).expect("a hidden entry is describable, so it is described");
    assert_eq!(
        info.entries
            .iter()
            .map(|e| e.name.as_str())
            .collect::<Vec<_>>(),
        ["easyloader ocm", "visible"],
        "the entries behind a hidden one must not be lost with it"
    );
    assert!(info.entries[0].hidden, "the flag is reported, not dropped");
    assert!(!info.entries[1].hidden);

    assert!(
        matches!(
            read::load(&cart),
            Err(ReadError::Unsupported(Unsupported::HiddenEntry { .. }))
        ),
        "rebuilding would silently drop the flag, so loading still refuses — the refusal belongs \
         to the path that rebuilds, not to the one that describes"
    );
}

/// Listing does not require the image to carry the firmware this tool embeds.
#[test]
fn inspect_does_not_require_the_firmware_this_tool_embeds() {
    let mut img = CartBuilder::new()
        .build_image()
        .expect("an empty image builds");
    img.write_at(reserved::EAPI_OFFSET, &[0x00; 32]);
    let tampered = crt::write(&img, "probe");

    assert!(
        read::inspect(&tampered).is_ok(),
        "the region check exists so a rebuild cannot silently change firmware; a listing rebuilds \
         nothing, and refusing it there means no cart built by another tool can be inspected"
    );
    assert!(
        matches!(
            read::load(&tampered),
            Err(ReadError::Unsupported(
                Unsupported::ReservedRegionMismatch { .. }
            ))
        ),
        "the rebuild path keeps the check"
    );
}

/// Free space is the run a further entry could actually use, given where the entries really sit.
#[test]
fn free_space_is_the_run_above_the_entries_not_the_unused_total() {
    // The AC-6 layout: one small entry parked high, with everything below it empty. Subtracting
    // sizes from the capacity would call almost the whole cartridge free, when a placer packing
    // from the top can only offer what lies above bank 40.
    let mut planted = an_entry();
    planted.bank = 40;
    planted.offset = 0x2000;
    planted.size = 4;
    let cart = cart_with_slots(&[directory::encode(&planted), directory::end_marker()]);

    let info = read::inspect(&cart).expect("it inspects");
    let above = (63 - 40) * 8192 * 2 + (8192 - 4);
    assert_eq!(
        info.free_space(),
        above,
        "the figure a user sizes a program against has to be the contiguous run above the \
         occupied banks, not the total of every unused byte"
    );

    // And the top-packed case still reports what it always did, so this is a fix rather than a
    // different arbitrary number.
    let single = read::inspect(&fixture("single.crt")).expect("single.crt inspects");
    let occupied: usize = single.entries.iter().map(|e| e.size as usize).sum();
    assert_eq!(single.free_space(), 63 * 8192 * 2 - occupied);
}

/// Clear the reserved flag bits on an encoded slot, which is what makes it an old-format entry.
fn as_old_format(mut slot: [u8; directory::ENTRY_LEN]) -> [u8; directory::ENTRY_LEN] {
    slot[16] &= !directory::RESERVED_FLAG_BITS;
    slot
}

/// AC-14 (none): an old-format (EasyFS1) directory is refused by both read paths.
///
/// Bits 5-6 of the flags byte are the only thing distinguishing EasyFS1 from EasyFS2 — the format
/// carries no version field (D-017). Read as EasyFS2, an EasyFS1 entry's bank comes back corrupt,
/// because that field is one byte there and two here. So this is not a D-011 exception: there is no
/// accurate description to give, which is a different situation from a hidden entry the walk
/// describes perfectly well and only `load` has cause to refuse.
#[test]
fn an_old_format_directory_is_refused_rather_than_read_as_easyfs2() {
    let cart = cart_with_slots(&[
        as_old_format(directory::encode(&an_entry())),
        directory::end_marker(),
    ]);

    assert!(
        matches!(
            read::inspect(&cart),
            Err(ReadError::Unsupported(
                Unsupported::OldFormatDirectory { .. }
            ))
        ),
        "inspecting an EasyFS1 directory must refuse it as old-format, not report entries whose \
         bank number was decoded under the wrong layout"
    );
    assert!(
        matches!(
            read::load(&cart),
            Err(ReadError::Unsupported(
                Unsupported::OldFormatDirectory { .. }
            ))
        ),
        "loading one must refuse for the same reason"
    );
}

/// AC-15 (none): slots that disagree about the format are malformed, not old.
///
/// The first non-end slot decides for the whole directory; a later slot contradicting it is a
/// corrupt image rather than a second opinion. Reporting it as old-format would name the wrong
/// problem, and would do so on the images least likely to deserve the benefit of the doubt.
#[test]
fn slots_disagreeing_about_the_format_are_refused_as_malformed() {
    let cart = cart_with_slots(&[
        directory::encode(&an_entry()),
        as_old_format(directory::encode(&an_entry())),
        directory::end_marker(),
    ]);

    let err = read::inspect(&cart).expect_err("a directory that contradicts itself cannot be read");
    assert!(
        !matches!(
            err,
            ReadError::Unsupported(Unsupported::OldFormatDirectory { .. })
        ),
        "an EasyFS2 first entry settles the format, so a later slot with the bits clear is \
         corruption — calling it old-format would report the wrong fault: {err}"
    );
}

/// AC-16 (none): the check does not disturb a well-formed EasyFS2 directory.
///
/// The regression half of AC-14. `encode` sets the reserved bits and always has, so every fixture
/// and everything this tool writes must be entirely unaffected by the new refusal.
#[test]
fn well_formed_easyfs2_directories_are_untouched_by_the_old_format_check() {
    for name in ["single.crt", "named.crt", "two.crt", "spill.crt"] {
        let bytes = fixture(name);
        read::inspect(&bytes).unwrap_or_else(|e| panic!("{name} must still inspect: {e}"));
        let rebuilt = read::load(&bytes)
            .unwrap_or_else(|e| panic!("{name} must still load: {e}"))
            .build()
            .unwrap_or_else(|e| panic!("{name} must still rebuild: {e}"));
        assert_eq!(
            rebuilt, bytes,
            "{name} must still round-trip byte-identically"
        );
    }
}
