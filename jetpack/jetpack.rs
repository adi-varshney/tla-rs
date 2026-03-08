// Jetpack plugin consensus protocol -- hand-translated from jetpack/jetpack.tla
//
// This is a tla-rs / Verus-style spec preserving the genuine 3-D log model,
// message-bag semantics, per-proposer commitIndex, and all Jetpack actions
// and safety properties from the original TLA+ spec.
//
// See jetpack/translation_audit.md for the translation mapping.

#![allow(unused)]

use vstd::prelude::*;

// In a real Verus build these would be module imports:
// use crate::jetpack::types::*;
// For now, types are defined in types.rs alongside this file.
// Include types inline for self-containment during development.
include!("types.rs");

verus! {

// The protocol spec body is in jetpack_body.rs so that proof files can
// include it inside their own verus!{} block without double-defining types.
include!("jetpack_body.rs");

} // verus!
