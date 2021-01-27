/// Used to note where a part of code is located in the source.
#[derive(Clone, Copy, Debug)]
pub struct Span {
    pub lo: usize,
    pub hi: usize,
}

impl Span {
    pub fn new(lo: usize, hi: usize) -> Self {
        Self { lo, hi }
    }

    pub fn str_from_source<'a>(&'a self, source: &'a str) -> &'a str {
        &source[self.lo..self.hi]
    }
}

/// Wraps an inner value with a span.
#[derive(Clone, Debug)]
pub struct Spanned<T> {
    pub inner: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(inner: T, lo: usize, hi: usize) -> Self {
        Self {
            inner,
            span: Span::new(lo, hi),
        }
    }

    /// Takes self and return inner
    pub fn unwrap(self) -> T {
        self.inner
    }
}

impl<T> std::ops::Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> std::ops::DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
