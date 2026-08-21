---
title: Gate K dependency policy
status: Owner-authorized; effective when merged
number: 0005
date: 2026-08-21
issue: 23
contract_revision: 4
owner: Peter Permenter
implementing_reviewer: GPT-5.6
---

# Gate K dependency policy

## Decision authority

The GPT Pro architecture checkpoint filed issue #23 after finding that the
workspace's zero-third-party-dependency state was drifting from an implementation
choice into an unrecorded permanent rule. On 2026-08-21, Peter Permenter approved
the temporary Gate K policy below. It becomes effective when this record merges.

## Decision

The Gate K workspace admits no third-party dependency. This constraint lasts
through Gate K and affirms the existing wording of SW-D issue #14.

This is a temporary proof constraint, not a constitutional zero-dependency
policy for the repository, thesis, or later gates. After Gate K, a third-party
dependency may be admitted only by a separate owner-authorized decision that
records at least:

- its exact locked version, license, and provenance;
- why it is preferable to a local implementation;
- whether it can affect authoritative determinism;
- confirmation that it performs no build-time or runtime network access;
- the vendoring or cache plan for formal offline proof; and
- maintenance ownership, especially for security-sensitive code.

The dependency-boundary checker changes only through that same explicit
decision. No dependency is added by this record.

## Reason

Gate K is small, renderer-free, and required to build and test offline. Keeping
its dependency graph local preserves a cheap clean-machine proof, a small audit
surface, and an inspectable lockfile while the semantic architecture is still
being falsified.

Those are the invariants. A permanently pleasing dependency count is not. Later
renderer, signing, networking, asset, compression, image, or platform work must
not reimplement mature libraries merely to preserve a rule that Gate K never
needed to impose on them.

## Effect on existing evidence

Existing evidence is unchanged. `Cargo.lock` still contains seven local
workspace entries, historical receipts retain their recorded toolchains, and
the in-crate SHA-256 vectors and frozen canonical hash remain valid.

The current SHA-256 and canonical JSON reader are the Gate K integrity and
serialization implementations. Their presence is not evidence that production
package signing, an adversarial cryptographic threat model, or every future
serialization need has been solved.

This decision does not amend `KERNEL.md`, change contract revision 4, weaken the
offline cold-agent or clean-rebuild requirements, or authorize any dependency
currently forbidden by the Gate K contract.

## Owner disposition

Peter accepted checkpoint findings #21, #22, and #24 at their recorded slice
boundaries and selected the temporary Gate K constraint for #23. Once this
record and the checkpoint handoff merge, #23 and #25 may close and SW-D may
begin with #21 as its first isolated prerequisite commit.
