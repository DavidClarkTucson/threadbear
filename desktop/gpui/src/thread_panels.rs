// The panels round the main one (the Swift app's ThreadPanels.swift and the
// Threads panel's toggles in AppModel+Chrome.swift): the Threads panel folded
// away with ⌘/ and back as it was.
use crate::app::Threadbear;
use crate::shell::THREADS;
use gpui_kit::*;
use threadbear_dock::{DockDrop, DockEdge};

impl Threadbear {
    /// ⌘/ folded the Threads panel away (`threadsCollapsed`). It stays in the
    /// layout with nothing to show (`panel_available`), so the dock gives its
    /// room to the rest and it comes back where it was, as wide as it was.
    pub(crate) fn threads_collapsed(&self) -> bool {
        self.settings.saved.threads_collapsed == Some(true)
    }

    /// The Threads panel is on screen: in the layout and not folded away
    /// (`threadsShowing`), which is what the View menu's titles go by.
    pub(crate) fn threads_showing(&self, cx: &App) -> bool {
        self.dock.read(cx).layout().contains(THREADS) && !self.threads_collapsed()
    }

    pub(crate) fn set_threads_collapsed(&mut self, collapsed: bool, cx: &mut Context<Self>) {
        if self.threads_collapsed() == collapsed {
            return;
        }
        self.settings.saved.threads_collapsed = collapsed.then_some(true);
        self.save_settings_soon(cx);
        // The dock asks what's available as it draws, so a redraw is all it
        // takes; the menus' titles follow (`menus_stale`).
        self.dock.update(cx, |_, cx| cx.notify());
        cx.notify();
    }

    /// ⌘/ (`toggleThreadsCollapsed`): folds the Threads panel away, or
    /// brings it back as it was. The activity bar stays. If Hide Threads had
    /// taken the panel out of the layout, this puts it back instead.
    pub(crate) fn toggle_threads_collapsed(&mut self, cx: &mut Context<Self>) {
        if !self.dock.read(cx).layout().contains(THREADS) {
            self.set_threads_collapsed(false, cx);
            self.dock.update(cx, |m, cx| m.open(THREADS, DockDrop::window(DockEdge::Left), cx));
            return;
        }
        let collapsed = !self.threads_collapsed();
        self.set_threads_collapsed(collapsed, cx);
    }

    /// A view's icon in the activity bar, or Show Inbox and the rest in the
    /// Thread menu, bring the Threads panel back if ⌘/ folded it away, as
    /// the Swift app's do.
    pub(crate) fn unfold_threads(&mut self, cx: &mut Context<Self>) {
        self.set_threads_collapsed(false, cx);
    }
}
