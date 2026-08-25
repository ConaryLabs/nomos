//! The area collection: the route chain, the shared visual grammar, and every
//! refusal `experiments/executable-gaol/src/build-collection.mjs` performed.
//!
//! The plans here are written fresh, in this file's own vocabulary. Nothing is
//! read from `experiments/`: `RUNTIME.md` section 2 makes the study a
//! specification and a comparison target, and a test that read its committed
//! fixtures would quietly make it a source of truth. Each plan carries exactly
//! the fields the collection reads, which is the point — the collection is not
//! allowed to need anything else.
//!
//! `experiments/executable-gaol/src/area-collection.test.mjs` proved the same
//! statements against the four committed plans; the ones about the collection
//! document are here now, and `docs/review/area-collection.md` maps them
//! one for one.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};

use nomos_core::CanonicalValue;
use nomos_core::canonical::read::parse_canonical;
use nomos_core::hash::Sha256Digest;
use nomos_render_plan::collection::{self, PlanInput};
use nomos_render_plan::error::{PlanCode, PlanError, codes};
use nomos_render_plan::plan::rendering_plan_schema;

static COUNTER: AtomicU32 = AtomicU32::new(0);

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// One area's plan, in the fields the collection reads.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct AreaSpec {
    id: &'static str,
    label: &'static str,
    start: bool,
    gate: &'static str,
    to_area: Option<&'static str>,
    entry: Option<(i64, i64, i64)>,
    /// The architecture assembly, which is part of the shared visual grammar.
    /// Every area declares the same one unless a test is proving divergence.
    style: &'static str,
}

const STYLE: &str = "visual/beveled_masonry";

const fn area(
    id: &'static str,
    label: &'static str,
    gate: &'static str,
    to_area: Option<&'static str>,
    entry: Option<(i64, i64, i64)>,
) -> AreaSpec {
    AreaSpec {
        id,
        label,
        start: entry.is_none(),
        gate,
        to_area,
        entry,
        style: STYLE,
    }
}

/// The four-area corpus: one start, one exit, a chain through all four.
fn four_areas() -> Vec<AreaSpec> {
    vec![
        area(
            "lower-sump",
            "Lower Sump",
            "sump_gate",
            Some("kiln-yard"),
            None,
        ),
        area(
            "kiln-yard",
            "Kiln Yard",
            "kiln_gate",
            Some("relic-stair"),
            Some((7, 5, 0)),
        ),
        area(
            "relic-stair",
            "Relic Stair",
            "stair_gate",
            Some("upper-ward"),
            Some((1, 5, 0)),
        ),
        area(
            "upper-ward",
            "Upper Ward",
            "ward_gate",
            None,
            Some((2, 4, 0)),
        ),
    ]
}

fn cell(x: i64, y: i64, z: i64) -> CanonicalValue {
    CanonicalValue::object_declared([
        ("x", CanonicalValue::Int(x)),
        ("y", CanonicalValue::Int(y)),
        ("z", CanonicalValue::Int(z)),
    ])
}

fn schema(name: &str, version: u64) -> CanonicalValue {
    CanonicalValue::object_declared([
        ("name", CanonicalValue::text(name)),
        ("version", CanonicalValue::Uint(version)),
    ])
}

fn entity(id: &str, kind: &str, assembly: &str, material: &str) -> CanonicalValue {
    CanonicalValue::object_declared([
        ("id", CanonicalValue::text(id)),
        ("kind", CanonicalValue::text(kind)),
        ("material_family", CanonicalValue::text(material)),
        ("visual_assembly", CanonicalValue::text(assembly)),
    ])
}

/// One `nomos.rendering_plan@2` document, in the fields the collection reads.
fn plan_document(spec: &AreaSpec) -> CanonicalValue {
    let mut route = vec![(
        "to_area",
        spec.to_area
            .map_or(CanonicalValue::Null, CanonicalValue::text),
    )];
    if let Some((x, y, z)) = spec.entry {
        route.push(("entry", cell(x, y, z)));
    }

    CanonicalValue::object_declared([
        (
            "actors",
            CanonicalValue::Array(vec![
                CanonicalValue::object_declared([
                    ("assembly", CanonicalValue::text("visual/player_silhouette")),
                    ("id", CanonicalValue::text("player")),
                ]),
                CanonicalValue::object_declared([
                    ("assembly", CanonicalValue::text("visual/gaoler_silhouette")),
                    ("id", CanonicalValue::text("gaoler")),
                ]),
            ]),
        ),
        (
            "architecture",
            CanonicalValue::object_declared([(
                "style",
                CanonicalValue::object_declared([
                    ("assembly", CanonicalValue::text(spec.style)),
                    ("material_family", CanonicalValue::text("stone_bounded")),
                    ("trim_family", CanonicalValue::text("broad_mortar")),
                ]),
            )]),
        ),
        (
            "area",
            CanonicalValue::object_declared([
                ("id", CanonicalValue::text(spec.id)),
                ("label", CanonicalValue::text(spec.label)),
                ("start", CanonicalValue::Bool(spec.start)),
            ]),
        ),
        (
            "effects",
            CanonicalValue::Array(vec![CanonicalValue::object_declared([
                ("assembly", CanonicalValue::text("visual/cyan_crescent")),
                ("id", CanonicalValue::text("ward_mark")),
            ])]),
        ),
        (
            "entities",
            CanonicalValue::Array(vec![
                entity(
                    spec.gate,
                    "door",
                    "visual/iron_barred_door",
                    "iron_oxidized",
                ),
                entity("brazier", "light", "visual/brazier", "iron_brazier"),
                entity("channel", "water", "visual/shallow_water", "water_cold"),
            ]),
        ),
        (
            "objective",
            CanonicalValue::object_declared([
                ("gate", CanonicalValue::text(spec.gate)),
                ("kind", CanonicalValue::text("exit_via")),
            ]),
        ),
        (
            "projection_schemas",
            CanonicalValue::Array(vec![
                schema("nomos.projection.simulation", 3),
                schema("nomos.projection.navigation", 1),
                schema("nomos.projection.persistence", 1),
                schema("nomos.projection.diagnostics", 1),
            ]),
        ),
        ("route", CanonicalValue::object_declared(route)),
        (
            "schema",
            CanonicalValue::text(rendering_plan_schema().to_string()),
        ),
    ])
}

/// A temporary corpus on disk, in the study's published layout.
struct Corpus {
    root: PathBuf,
}

impl Corpus {
    fn new(label: &str, specs: &[AreaSpec]) -> Self {
        let index = COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join("area-collection")
            .join(format!("{}-{label}-{index}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        for spec in specs {
            let directory = root.join(spec.id);
            fs::create_dir_all(&directory).unwrap();
            write_plan(&directory.join("rendering-plan.json"), &plan_document(spec));
        }
        Self { root }
    }

    fn inputs(&self) -> Vec<PlanInput> {
        collection::expand(&self.root).unwrap()
    }

    fn build(&self) -> Result<Vec<u8>, PlanError> {
        collection::build(&self.inputs()).map(|compiled| compiled.bytes)
    }

    fn plan(&self, id: &str) -> PathBuf {
        self.root.join(id).join("rendering-plan.json")
    }
}

impl Drop for Corpus {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// Plans are published as canonical bytes plus one `LF`, exactly as the plan
/// compiler writes them, so the digest the collection publishes is a digest of
/// the file a consumer will actually fetch.
fn write_plan(path: &Path, document: &CanonicalValue) {
    let mut bytes = document.to_canonical_bytes();
    bytes.push(b'\n');
    fs::write(path, bytes).unwrap();
}

fn refuses(code: PlanCode, specs: &[AreaSpec], label: &str) -> PlanError {
    let corpus = Corpus::new(label, specs);
    let error = corpus
        .build()
        .expect_err("the collection was expected to refuse this corpus");
    assert_eq!(
        error.code(),
        code,
        "expected {code}, got {error}",
        code = code.as_str()
    );
    error
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).unwrap()
}

// ---------------------------------------------------------------------------
// The document
// ---------------------------------------------------------------------------

#[test]
fn the_schema_identity_is_declared_here() {
    assert_eq!(
        collection::area_collection_schema().to_string(),
        "nomos.area_collection@1"
    );
}

#[test]
fn four_areas_compile_to_one_ordered_chain() {
    let corpus = Corpus::new("chain", &four_areas());
    let bytes = corpus.build().unwrap();
    let document = text(&bytes);

    // Areas in identity order, not in chain order: the chain is the `route`.
    let order: Vec<&str> = ["kiln-yard", "lower-sump", "relic-stair", "upper-ward"].to_vec();
    let mut cursor = 0;
    for id in &order {
        let needle = format!("\"id\":\"{id}\"");
        let at = document[cursor..]
            .find(&needle)
            .unwrap_or_else(|| panic!("`{id}` is not in identity order"));
        cursor += at + needle.len();
    }

    assert!(document.contains("\"schema\":\"nomos.area_collection@1\""));
    assert!(document.contains("\"start_area\":\"lower-sump\""));
    assert!(document.contains("\"rendering_plan_schema\":\"nomos.rendering_plan@2\""));
    // The chain: every hop reads its arrival cell from the destination's own
    // plan, and the last hop carries none.
    assert!(document.contains(
        "{\"entry\":{\"x\":7,\"y\":5,\"z\":0},\"from_area\":\"lower-sump\",\
         \"gate\":\"sump_gate\",\"to_area\":\"kiln-yard\"}"
    ));
    assert!(document.contains(
        "{\"entry\":null,\"from_area\":\"upper-ward\",\"gate\":\"ward_gate\",\"to_area\":null}"
    ));
    assert!(bytes.ends_with(b"\n"));
}

#[test]
fn the_document_is_canonical_and_names_the_plan_bytes() {
    let corpus = Corpus::new("canonical", &four_areas());
    let bytes = corpus.build().unwrap();

    // The strict reader accepts it, and re-encoding is byte-identical: the
    // collection is the kernel's canonical bytes, not an encoder of its own.
    let value = parse_canonical(&bytes[..bytes.len() - 1]).expect("the collection is canonical");
    assert_eq!(value.to_canonical_bytes(), bytes[..bytes.len() - 1]);

    for spec in four_areas() {
        let published = Sha256Digest::of_bytes(&fs::read(corpus.plan(spec.id)).unwrap()).to_hex();
        let expected = format!(
            "\"plan\":{{\"file\":\"{}.json\",\"sha256\":\"{published}\"}}",
            spec.id
        );
        assert!(
            text(&bytes).contains(&expected),
            "the collection does not name {}'s bytes",
            spec.id
        );
    }
}

#[test]
fn compiling_twice_is_byte_identical() {
    let corpus = Corpus::new("twice", &four_areas());
    assert_eq!(corpus.build().unwrap(), corpus.build().unwrap());
}

#[test]
fn the_grammar_digest_covers_the_grammar_and_nothing_else() {
    // Relabelling an area changes the document and not the grammar digest;
    // restyling one area changes both — by refusing, because the grammar is
    // shared. This is the pair `build-collection.mjs:40-47,90` implied.
    let corpus = Corpus::new("digest", &four_areas());
    let first = text(&corpus.build().unwrap());

    let mut relabelled = four_areas();
    relabelled[0].label = "Lower Sump, Flooded";
    let second = text(
        &Corpus::new("digest-relabelled", &relabelled)
            .build()
            .unwrap(),
    );

    assert_ne!(first, second);
    let digest = |document: &str| {
        let at = document.find("\"digest\":\"").unwrap() + "\"digest\":\"".len();
        document[at..at + 64].to_owned()
    };
    assert_eq!(digest(&first), digest(&second));
}

// ---------------------------------------------------------------------------
// Refusals
// ---------------------------------------------------------------------------

#[test]
fn a_collection_with_no_start_area_is_refused() {
    let mut specs = four_areas();
    specs[0].start = false;
    specs[0].entry = Some((3, 3, 0));
    let error = refuses(codes::COLLECTION_ROUTE_INVALID, &specs, "no-start");
    assert!(
        error.message().contains("exactly one start area"),
        "{error}"
    );
    assert!(error.message().contains("0 declared"), "{error}");
}

#[test]
fn a_collection_with_two_start_areas_is_refused() {
    // Two areas declaring no arrival cell, neither of them anything's
    // destination, so the arrival checks pass and the count is what refuses.
    let specs = [
        area(
            "lower-sump",
            "Lower Sump",
            "sump_gate",
            Some("upper-ward"),
            None,
        ),
        area(
            "kiln-yard",
            "Kiln Yard",
            "kiln_gate",
            Some("upper-ward"),
            None,
        ),
        area(
            "upper-ward",
            "Upper Ward",
            "ward_gate",
            None,
            Some((2, 4, 0)),
        ),
    ];
    let error = refuses(codes::COLLECTION_ROUTE_INVALID, &specs, "two-starts");
    assert!(
        error.message().contains("exactly one start area"),
        "{error}"
    );
    assert!(
        error
            .message()
            .contains("2 declared (kiln-yard, lower-sump)"),
        "{error}"
    );
}

#[test]
fn a_destination_that_is_not_a_declared_area_is_refused() {
    let mut specs = four_areas();
    specs[0].to_area = Some("the-undercroft");
    let error = refuses(codes::COLLECTION_ROUTE_INVALID, &specs, "unknown-target");
    assert!(
        error
            .message()
            .contains("area `lower-sump` targets unknown area `the-undercroft`"),
        "{error}"
    );
}

#[test]
fn a_destination_that_declares_no_arrival_cell_is_refused() {
    // Two areas, each leading to the other; the start area declares no arrival
    // cell, so it cannot receive one.
    let specs = [
        area(
            "lower-sump",
            "Lower Sump",
            "sump_gate",
            Some("kiln-yard"),
            None,
        ),
        area(
            "kiln-yard",
            "Kiln Yard",
            "kiln_gate",
            Some("lower-sump"),
            Some((7, 5, 0)),
        ),
    ];
    let error = refuses(codes::COLLECTION_ROUTE_INVALID, &specs, "no-entry");
    assert!(
        error
            .message()
            .contains("area `kiln-yard` leads to `lower-sump`, which declares no arrival cell"),
        "{error}"
    );
}

#[test]
fn a_route_that_cycles_is_refused() {
    let specs = [
        area(
            "lower-sump",
            "Lower Sump",
            "sump_gate",
            Some("kiln-yard"),
            None,
        ),
        area(
            "kiln-yard",
            "Kiln Yard",
            "kiln_gate",
            Some("relic-stair"),
            Some((7, 5, 0)),
        ),
        area(
            "relic-stair",
            "Relic Stair",
            "stair_gate",
            Some("kiln-yard"),
            Some((1, 5, 0)),
        ),
        area(
            "upper-ward",
            "Upper Ward",
            "ward_gate",
            None,
            Some((2, 4, 0)),
        ),
    ];
    let error = refuses(codes::COLLECTION_ROUTE_INVALID, &specs, "cycle");
    assert!(
        error.message().contains("route cycles at `kiln-yard`"),
        "{error}"
    );
}

#[test]
fn an_area_the_chain_never_visits_is_refused() {
    let specs = [
        area(
            "lower-sump",
            "Lower Sump",
            "sump_gate",
            Some("kiln-yard"),
            None,
        ),
        area("kiln-yard", "Kiln Yard", "kiln_gate", None, Some((7, 5, 0))),
        area(
            "relic-stair",
            "Relic Stair",
            "stair_gate",
            Some("kiln-yard"),
            Some((1, 5, 0)),
        ),
    ];
    let error = refuses(codes::COLLECTION_ROUTE_INVALID, &specs, "unvisited");
    assert!(
        error
            .message()
            .contains("visits 2 of 3 declared areas; unvisited (relic-stair)"),
        "{error}"
    );
}

#[test]
fn a_chain_that_does_not_terminate_at_one_exit_area_is_refused() {
    let specs = [
        area(
            "lower-sump",
            "Lower Sump",
            "sump_gate",
            Some("kiln-yard"),
            None,
        ),
        area("kiln-yard", "Kiln Yard", "kiln_gate", None, Some((7, 5, 0))),
        area(
            "relic-stair",
            "Relic Stair",
            "stair_gate",
            None,
            Some((1, 5, 0)),
        ),
    ];
    let error = refuses(codes::COLLECTION_ROUTE_INVALID, &specs, "two-exits");
    assert!(
        error
            .message()
            .contains("terminate at exactly one area declaring no destination"),
        "{error}"
    );
    assert!(
        error
            .message()
            .contains("2 declare none (kiln-yard, relic-stair)"),
        "{error}"
    );
}

#[test]
fn an_area_that_is_both_a_start_and_an_arrival_is_refused() {
    let mut specs = four_areas();
    specs[0].entry = Some((3, 3, 0));
    let error = refuses(codes::COLLECTION_ROUTE_INVALID, &specs, "start-and-entry");
    assert!(
        error
            .message()
            .contains("must declare an arrival cell if and only if it is not the start area"),
        "{error}"
    );
}

#[test]
fn an_area_that_diverges_from_the_shared_grammar_is_refused() {
    let mut specs = four_areas();
    specs[2].style = "visual/rough_masonry";
    let error = refuses(codes::COLLECTION_GRAMMAR_DIVERGED, &specs, "grammar");
    assert!(
        error
            .message()
            .contains("area `relic-stair` diverges from the shared visual grammar"),
        "{error}"
    );
}

#[test]
fn one_area_is_not_a_collection() {
    let specs = [area("lower-sump", "Lower Sump", "sump_gate", None, None)];
    let error = refuses(codes::COLLECTION_ROUTE_INVALID, &specs, "one-area");
    assert!(error.message().contains("at least two areas"), "{error}");
}

#[test]
fn a_directory_that_is_not_the_area_identity_is_refused() {
    let corpus = Corpus::new("directory", &four_areas());
    fs::rename(
        corpus.root.join("kiln-yard"),
        corpus.root.join("kiln-court"),
    )
    .unwrap();
    let error = collection::build(&corpus.inputs()).unwrap_err();
    assert_eq!(error.code(), codes::COLLECTION_ROUTE_INVALID);
    assert!(
        error
            .message()
            .contains("directory `kiln-court` does not match plan area identity `kiln-yard`"),
        "{error}"
    );
}

#[test]
fn a_repeated_area_identity_is_refused() {
    // Two plan files naming one area. The study's `byId` map (`:42`) resolved
    // this by keeping whichever plan it read last.
    let corpus = Corpus::new("repeated", &four_areas());
    let duplicate = corpus.root.join("copy.json");
    fs::copy(corpus.plan("kiln-yard"), &duplicate).unwrap();
    let mut inputs = corpus.inputs();
    inputs.push(PlanInput {
        path: duplicate,
        directory: None,
    });
    let error = collection::build(&inputs).unwrap_err();
    assert_eq!(error.code(), codes::COLLECTION_ROUTE_INVALID);
    assert!(
        error
            .message()
            .contains("area `kiln-yard` is declared twice"),
        "{error}"
    );
}

#[test]
fn the_plan_identity_and_version_are_bound() {
    let corpus = Corpus::new("identity", &four_areas());
    let mut document = plan_document(&four_areas()[1]);
    let CanonicalValue::Object(fields) = &mut document else {
        unreachable!()
    };
    fields.insert(
        nomos_core::FieldName::declared("schema"),
        CanonicalValue::text("nomos.rendering_plan@1"),
    );
    write_plan(&corpus.plan("kiln-yard"), &document);

    let error = collection::build(&corpus.inputs()).unwrap_err();
    assert_eq!(error.code(), codes::SCHEMA_MISMATCH);
    assert!(
        error
            .message()
            .contains("expected schema `nomos.rendering_plan@2`, found `nomos.rendering_plan@1`"),
        "{error}"
    );
    assert_eq!(error.path(), Some(corpus.plan("kiln-yard").as_path()));
}

#[test]
fn a_plan_that_is_not_canonical_bytes_is_refused() {
    let corpus = Corpus::new("not-canonical", &four_areas());
    fs::write(
        corpus.plan("kiln-yard"),
        b"{ \"schema\": \"nomos.rendering_plan@2\" }\n",
    )
    .unwrap();
    let error = collection::build(&corpus.inputs()).unwrap_err();
    assert_eq!(error.code(), codes::INPUT_NOT_CANONICAL);
}

// ---------------------------------------------------------------------------
// The command
// ---------------------------------------------------------------------------

#[test]
fn the_collection_mode_writes_the_document_and_reports_it() {
    let corpus = Corpus::new("command", &four_areas());
    let out = corpus.root.join("areas.json");
    let output = Command::new(env!("CARGO_BIN_EXE_nomos-render-plan"))
        .arg("collection")
        .arg("--plans")
        .arg(&corpus.root)
        .arg("--out")
        .arg(&out)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let reported = String::from_utf8(output.stdout).unwrap();
    assert!(
        reported.contains("\"command\":\"area-collection\""),
        "{reported}"
    );
    assert!(reported.contains("\"area_count\":4"), "{reported}");
    assert!(
        reported.contains("\"start_area\":\"lower-sump\""),
        "{reported}"
    );
    assert!(reported.contains("\"status\":\"completed\""), "{reported}");
    // The status document names the identity the file carries, and the file is
    // what the library would have produced.
    assert!(
        reported.contains("\"schema\":{\"name\":\"nomos.area_collection\",\"version\":1}"),
        "{reported}"
    );
    assert_eq!(fs::read(&out).unwrap(), corpus.build().unwrap());
}

#[test]
fn the_collection_mode_fails_closed_on_stdout() {
    let corpus = Corpus::new("command-refused", &four_areas());
    let out = corpus.root.join("areas.json");
    let output = Command::new(env!("CARGO_BIN_EXE_nomos-render-plan"))
        .arg("collection")
        .arg("--plan")
        .arg(&corpus.root)
        .arg("--out")
        .arg(&out)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let reported = String::from_utf8(output.stdout).unwrap();
    assert!(reported.contains("\"code\":\"RP0106\""), "{reported}");
    assert!(reported.contains("\"status\":\"rejected\""), "{reported}");
    assert!(!out.exists(), "a refused run wrote its output anyway");
}
