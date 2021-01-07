use std::hash::Hash;

use crate::VClock;

/// Common Actor type. Actors are unique identifier for every `thing` mutating a VClock.
/// VClock based CRDT's will need to expose this Actor type to the user.
pub trait Actor: Ord + Clone + Hash {}
impl<A: Ord + Clone + Hash> Actor for A {}

/// State based CRDT's replicate by transmitting the entire CRDT state.
pub trait CvRDT {
    type Validation;

    /// Some CRDT's have stricter requirements on how they must be used.
    /// To avoid violating these requirements, CRDT's provide an interface
    /// to optionally validate merge compatibility before attempting to merge.
    ///
    /// An `Ok(())` response signals that the merge is safe to proceed.
    /// Otherwise a structured error is returned to help you determine what
    /// is wrong with the merge.
    fn validate_merge(&self, other: &Self) -> Result<(), Self::Validation>;

    /// Merge the given CRDT into the current CRDT.
    fn merge(&mut self, other: Self);
}

/// Operation based CRDT's replicate by transmitting each operation.
pub trait CmRDT {
    /// Op defines a mutation to the CRDT.
    /// As long as Op's from one actor are replayed in exactly the same order they
    /// were generated by that actor, the CRDT will converge. In other words, we must
    /// have a total ordering on each actors operations, while requiring only a partial
    /// order over all ops.
    /// E.g.
    ///
    /// Actor A produces ops A1, A2
    /// Actor B produces ops B1, B2
    ///
    /// the only valid orderings are:
    /// A1 < A2 < B1 < B2
    /// A1 < B1 < A2 < B2
    /// B1 < A1 < A2 < B2
    /// A1 < B1 < B2 < A2
    /// B1 < A1 < B2 < A2
    /// B1 < B2 < A1 < A2
    ///
    /// Applying ops in any of the valid orders will converge to the same CRDT state
    ///
    /// Op's must be idempotent, meaning any Op may be applied more than once.
    type Op;
    type Validation;

    /// Some CRDT's have stricter requirements on how they must be used.
    /// To avoid violating these requirements, CRDT's provide an interface
    /// to optionally validate op's before they are applied.
    ///
    /// An `Ok(())` response signals that this operation is safe to apply.
    /// Otherwise a structured error is returned to help you determine what
    /// is wrong with the operation
    fn validate_op(&self, op: &Self::Op) -> Result<(), Self::Validation>;

    /// Apply an Op to the CRDT
    fn apply(&mut self, op: Self::Op);
}

/// CRDT's are causal if they are built on top of vector clocks.
pub trait ResetRemove<A: Actor> {
    /// Remove data that is strictly smaller than this clock
    fn reset_remove(&mut self, clock: &VClock<A>);
}

/// Funky variant of the `CvRDT` trait.
///
/// This trait is for CvRDT's whose state space can't be easily encoded in rusts
/// typesystem so we rely on runtime error checking.
/// E.g. the unicity of timestamp assumption in LWWReg
pub trait FunkyCvRDT {
    /// Merge the given CRDT into the current CRDT.
    fn merge(&mut self, other: Self) -> crate::error::Result<()>;
}

/// Funky variant of the `CmRDT` trait.
///
/// This trait is for CvRDT's whose state space can't be easily encoded in rusts
/// typesystem so we rely on runtime error checking.
/// E.g. the unicity property of timestamp assumption in LWWReg
pub trait FunkyCmRDT {
    /// Same Op laws from non-funky CmRDT above
    type Op;

    /// Apply an Op to the CRDT
    fn apply(&mut self, op: Self::Op) -> crate::error::Result<()>;
}
