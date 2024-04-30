//! Data structure to efficiently store and deduplicate strings.

#[cfg(not(feature = "no-hash-maps"))]
mod detail {
    use super::{GetOrInternWithHint, Sym};
    use crate::hash;
    use string_interner::{backend::StringBackend, StringInterner, Symbol};

    pub type StringInternerImpl = StringInterner<StringBackend<Sym>, hash::RandomState>;

    impl GetOrInternWithHint for StringInternerImpl {
        #[inline]
        fn get_or_intern_with_hint<T>(&mut self, string: T, _hint: super::InternHint) -> Sym
        where
            T: AsRef<str>,
        {
            self.get_or_intern(string)
        }
    }

    impl Symbol for Sym {
        #[inline]
        fn try_from_usize(index: usize) -> Option<Self> {
            Some(Self(index))
        }

        #[inline]
        fn to_usize(self) -> usize {
            self.0
        }
    }
}

#[cfg(feature = "no-hash-maps")]
mod detail;

/// Internment hint to speed-up certain use cases.
#[derive(Debug, Copy, Clone)]
pub enum InternHint {
    /// No hint is given to the [`StringInterner`].
    None,
    /// Hint that the string to be interned likely already exists.
    LikelyExists,
    /// Hint that the string to be interned likely does not yet exist.
    LikelyNew,
}

/// Symbols returned by the [`StringInterner`] to resolve interned strings.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sym(usize);

/// Efficienty interns and deduplicates strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringInterner {
    inner: detail::StringInternerImpl,
}

impl Default for StringInterner {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl StringInterner {
    /// Creates a new empty [`StringInterner`].
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: detail::StringInternerImpl::new(),
        }
    }

    /// Returns the number of strings interned by the [`StringInterner`].
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if the [`StringInterner`] has no interned strings.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns the symbol for the given string if any.
    ///
    /// Can be used to query if a string has already been interned without interning.
    #[inline]
    pub fn get<T>(&self, string: T) -> Option<Sym>
    where
        T: AsRef<str>,
    {
        self.inner.get(string)
    }

    /// Interns the given string.
    ///
    /// Returns a symbol for resolution into the original string.
    ///
    /// # Panics
    ///
    /// If the interner already interns the maximum number of strings possible
    /// by the chosen symbol type.
    #[inline]
    pub fn get_or_intern<T>(&mut self, string: T) -> Sym
    where
        T: AsRef<str>,
    {
        self.inner.get_or_intern_with_hint(string, InternHint::None)
    }

    /// Interns the given string with usage hint.
    ///
    /// Returns a symbol for resolution into the original string.
    ///
    /// # Panics
    ///
    /// If the interner already interns the maximum number of strings possible
    /// by the chosen symbol type.
    #[inline]
    pub fn get_or_intern_with_hint<T>(&mut self, string: T, hint: InternHint) -> Sym
    where
        T: AsRef<str>,
    {
        self.inner.get_or_intern_with_hint(string, hint)
    }

    /// Returns the string for the given symbol if any.
    #[inline]
    pub fn resolve(&self, symbol: Sym) -> Option<&str> {
        self.inner.resolve(symbol)
    }
}

/// Extension trait for [`StringInterner`] backends.
trait GetOrInternWithHint {
    /// Interns the given string with usage hint.
    ///
    /// Returns a symbol for resolution into the original string.
    ///
    /// # Panics
    ///
    /// If the interner already interns the maximum number of strings possible
    /// by the chosen symbol type.
    fn get_or_intern_with_hint<T>(&mut self, string: T, hint: InternHint) -> Sym
    where
        T: AsRef<str>;
}
