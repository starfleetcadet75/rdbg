pub(crate) mod unix;

/// An integer type, whose size equals a machine word
///
/// `ptrace` always returns a machine word. This type provides an abstraction
/// of the fact that on *nix systems, `c_long` is always a machine word,
/// so as to prevent the library from leaking C implementation-dependent types.
pub type Word = usize;
