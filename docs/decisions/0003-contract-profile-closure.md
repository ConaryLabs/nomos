---
title: Contract revision 3 — canonical profile and workspace evidence closure
status: Proposed for owner disposition
number: 0003
date: 2026-08-21
issue: 4
supersedes_contract_revision: 2
establishes_contract_revision: 3
owner: Peter Permenter
implementing_reviewer: GPT-5.6
---

# Contract revision 3 — canonical profile and workspace evidence closure

## Proposed decision

Repair four underspecified parts of the Gate K contract that SW-B had to choose
while implementing contract revision 2. Revision 3 pins the existing accepted
implementation choices; it does not waive a failed proof or broaden Gate K.

This proposal is non-authoritative until the owner reviews the finished wording,
records a disposition below, and merges it. The revised contract becomes
effective on merge.

## Owner disposition

Pending owner review of the finished replacement wording.

## Amendments

### A1. Canonical JSON escape spelling

**Prior wording:** section 7 required JSON escaping for quotes, reverse solidus,
and control bytes, but did not select among JSON's equivalent spellings.

**Replacement wording:** `\"` and `\\` are the only spellings for quotation mark
and reverse solidus. Backspace, form feed, line feed, carriage return, and tab
use `\b`, `\f`, `\n`, `\r`, and `\t`. Every other code point below `U+0020`
uses `\u00xx` with lowercase hexadecimal digits. Solidus is emitted raw and
`\/` is refused. `U+007F` is emitted as raw UTF-8. All other non-ASCII string
characters are emitted as their UTF-8 bytes rather than `\u` escapes.

**Reason:** JSON permits multiple byte spellings for the same string. A hash
profile must permit exactly one.

**Effect on existing evidence:** none of the accepted bytes change. The encoder
and strict reader already implement this profile. Revision 3 adds direct tests
for every short escape and representative refused alternatives.

**Owner disposition:** pending.

**New contract revision:** 3.

### A2. Identifier and canonical field-name character set

**Prior wording:** section 7 required identifiers to be normalized to Unicode
NFC before validation and sorted object keys by normalized UTF-8 bytes, but did
not define the legal identifier or field-name alphabets.

Issue #4 summarized the implementation as identifier segments matching
`[a-z][a-z0-9_]*` and field names matching `[a-z0-9_]+`. That field-name
summary was inaccurate: the accepted implementation has always required a
lowercase ASCII letter first.

**Replacement wording:** every stable identifier segment and canonical object
field name is ASCII and matches `[a-z][a-z0-9_]*`. Composite stable IDs use only
their schema-declared separators between validated segments. The accepted
alphabet is invariant under NFC, so validation establishes normalization by
construction; non-ASCII identifiers and field names are refused. String values
remain arbitrary valid UTF-8 and are neither normalized nor ASCII-restricted.
Object fields sort by ascending UTF-8 bytes of their validated names.

**Reason:** normalization cannot be reproduced without a character-set and
Unicode-version contract. The ASCII grammar is deterministic, already
implemented, and fails closed. Requiring a leading letter also prevents the
contract repair from silently loosening the canonical key grammar to match an
incorrect issue summary.

**Effect on existing evidence:** no accepted identifier, field name, canonical
byte sequence, or hash changes. Existing non-ASCII refusal tests remain valid;
revision 3 adds direct field-name shape tests.

**Owner disposition:** pending.

**New contract revision:** 3.

### A3. Isolated workspace tooling

**Prior wording:** section 10 said Gate K used one Rust workspace and listed six
kernel crates. It required automated boundary checks without declaring where
the checker lived or whether tooling could be a workspace member.

**Replacement wording:** the workspace contains exactly six kernel crates plus
the isolated `xtask` tooling member. `xtask` builds no kernel artifact, depends
on no kernel crate, and is unreachable from every kernel crate. The boundary
checker fails closed if a declared kernel crate disappears or any undeclared
workspace member appears.

**Reason:** placing the checker inside `estate-cli` would put its implementation
dependencies inside the graph being checked and require exceptions to the
kernel boundary. An isolated member can inspect the graph without joining it.

**Effect on existing evidence:** the seven-member workspace and all five
planted boundary violations already match this wording. The six kernel-crate
dependency edges are unchanged.

**Owner disposition:** pending.

**New contract revision:** 3.

### A4. Evidence for unique canonical schema ownership

**Prior wording:** section 10 forbade canonical schema types from being defined
in more than one crate, while acceptance 15 said automated checks proved the
dependency graph and forbidden dependency rules. Cargo metadata cannot reveal
whether two Rust types express the same schema identity.

**Replacement wording:** the prohibition remains unchanged: canonical schema
types may not be defined in more than one crate. Cargo-metadata automation
proves workspace membership, permitted edges, cycles, forbidden dependencies,
and tooling isolation. It cannot infer whether two Rust types duplicate
canonical schema semantics. Gate K therefore also requires an explicit source-
review receipt enumerating each canonical schema identity, its owner crate, and
its authoritative Rust type set, then confirming no second crate defines that
schema. Local schema-ID uniqueness tests, compile-fail visibility tests at
forbidden boundaries, and compiler-crossing tests support that review; none is
claimed as semantic-uniqueness proof by itself.

**Reason:** an acceptance claim must name the evidence that can actually prove
it. Pretending Cargo metadata understands Rust type semantics would turn a
review obligation into a false automated receipt.

**Effect on existing evidence:** no dependency, schema owner, or forbidden
schema shape changes. The existing metadata checks, owner-crate tests,
compile-fail doctests, and compiler-crossing tests retain their scope. Revision
3 corrects the claim made about them and adds an explicit source-review receipt
obligation; it does not mark the whole Gate K workspace green.

**Owner disposition:** pending.

**New contract revision:** 3.

## Related non-normative record maintenance

`KERNEL.md` previously reported `status: Not started`. The replacement status
states that revision 3 is proposed pending owner disposition and implementation
has progressed through SW-C. This repairs stale repository state; it changes no
acceptance criterion and has no effect on existing evidence.

`THESIS.md` section 21 previously retained the authoring-source-language choice
as open after decision 0002 and SW-C had resolved it. The live open-question
ledger now omits that settled question and `docs/thesis-open-questions.md`
records why. This is thesis-ledger maintenance under decision 0002, not a new
mechanism or a Gate K contract amendment.

## Evidence limits preserved

Revision 3 does not satisfy the Linux aarch64 release target or the ten-runs-per-
target determinism matrix. Those remain unproved acceptance work. It also does
not complete any command surface, runtime, migration, replay, or formal
cold-agent gate. The final explicit schema-ownership source-review receipt also
remains unproved until the Gate K schema set is complete.
