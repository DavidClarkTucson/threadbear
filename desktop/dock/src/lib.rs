//! BearDock for GPUI: dockable, rearrangeable panels, like the Swift original in
//! `dock/` (and it, not this, defines what's right; keep the two in step). The
//! window's panels can sit side by side, stack as tabs, or be torn out into
//! windows of their own, and the arrangement is plain data the app can save.
//!
//! The tree and the drag decisions are in the UI-free
//! [`threadbear_dock_layout`] crate, which this one re-exports. What's here is
//! the GPUI on top: [`DockView`] draws the tree and handles the pointer, and the
//! app says what each panel shows through [`DockContent`].
pub mod content;
pub mod model;
pub mod overlay;
pub mod platform;
pub mod theme;
pub mod view;
pub mod windows;

pub use content::{DockContent, DockOptions, SharedContent};
pub use model::DockModel;
pub use threadbear_dock_layout as layout;
pub use threadbear_dock_layout::*;
pub use view::DockView;
