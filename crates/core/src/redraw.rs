//! Paint invalidation is independent of per-frame input flags and native animation.
use crate::{ControlKind, Ui};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug)]
pub(crate) struct Revision(u64);
impl Default for Revision {
    fn default() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self(NEXT.fetch_add(1, Ordering::Relaxed))
    }
}

type PointerAppearance = (
    Option<usize>,
    Option<usize>,
    Option<usize>,
    Option<(usize, usize)>,
);

impl Ui {
    /// Opaque paint token. Compare for equality, not ordering. Tokens also distinguish
    /// different Ui instances, so replacing/reloading a UI invalidates backend caches.
    /// Relevant input and host setters advance it; `end_frame` does not.
    /// Native canvas animation is owned by the host and does not advance this token.
    pub fn paint_revision(&self) -> u64 {
        self.paint_revision.0
    }

    /// Force cached UI painting to refresh, for example after renderer resources reset.
    pub fn invalidate_paint(&mut self) {
        self.paint_revision = Revision::default();
    }

    // Pointer coordinates inside an unchanged target are not a visual change. Keep
    // canvas/game motion cheap, while still invalidating dragged text selections
    // and popup highlights. No text allocation or hashing on pointer movement.
    pub(crate) fn pointer_appearance(&self) -> PointerAppearance {
        let cursor = self
            .pointer_capture
            .and_then(|i| match &self.controls[i].kind {
                ControlKind::TextInput(input) => Some(input.cursor),
                _ => None,
            });
        (
            self.pointer_target(),
            cursor,
            self.popup.as_ref().and_then(|p| p.highlighted),
            self.pointer
                .and_then(|(x, y)| self.scrollbar_hit(x, y))
                .map(|b| (b.index, b.axis)),
        )
    }
}
