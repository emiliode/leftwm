use super::{Config, DisplayEvent, Manager, Mode};
use crate::display_action::DisplayAction;
use crate::display_servers::DisplayServer;
use crate::models::WindowHandle;
use crate::State;

impl<C: Config, SERVER: DisplayServer> Manager<C, SERVER> {
    /// Process a collection of events, and apply them changes to a manager.
    /// Returns true if changes need to be rendered.
    pub fn display_event_handler(&mut self, event: DisplayEvent) -> bool {
        let state = &mut self.state;
        match event {
            DisplayEvent::ScreenCreate(s) => self.screen_create_handler(s),
            DisplayEvent::WindowCreate(w, x, y) => self.window_created_handler(w, x, y),
            DisplayEvent::WindowChange(w) => self.window_changed_handler(w),
            DisplayEvent::WindowDestroy(handle) => self.window_destroyed_handler(&handle),
            DisplayEvent::SendCommand(command) => self.command_handler(&command),
            DisplayEvent::MouseCombo(mod_mask, button, handle, x, y) => self
                .state
                .mouse_combo_handler(mod_mask, button, handle, x, y),

            DisplayEvent::WindowTakeFocus(handle) => from_window_take_focus(state, handle),
            DisplayEvent::HandleWindowFocus(handle) => from_handle_window_focus(state, handle),
            DisplayEvent::MoveFocusTo(x, y) => from_move_focus_to(state, x, y),
            DisplayEvent::VerifyFocusedAt(handle) => from_verify_focus_at(state, handle),
            DisplayEvent::ChangeToNormalMode => from_change_to_normal_mode(state),
            DisplayEvent::Movement(handle, x, y) => from_movement(state, handle, x, y),
            DisplayEvent::MoveWindow(handle, x, y) => from_move_window(self, handle, x, y),
            DisplayEvent::ResizeWindow(handle, x, y) => from_resize_window(self, handle, x, y),
            DisplayEvent::ConfigureXlibWindow(handle) => from_configure_xlib_window(state, handle),
        }
    }
}

fn from_window_take_focus(state: &mut State, handle: WindowHandle) -> bool {
    state.focus_window(&handle);
    false
}

fn from_handle_window_focus(state: &mut State, handle: WindowHandle) -> bool {
    state.handle_window_focus(&handle);
    false
}

fn from_move_focus_to(state: &mut State, x: i32, y: i32) -> bool {
    state.focus_window_with_point(x, y);
    false
}

fn from_verify_focus_at(state: &mut State, handle: WindowHandle) -> bool {
    if state.focus_manager.behaviour.is_sloppy() {
        state.validate_focus_at(&handle);
    }
    false
}

fn from_change_to_normal_mode(state: &mut State) -> bool {
    match state.mode {
        Mode::MovingWindow(h) | Mode::ResizingWindow(h) => {
            // We want to update the windows tag once it is done moving. This means
            // when the window is re-tiled it is on the correct workspace. This also
            // prevents the focus switching between the floating window and the
            // workspace behind. We will also apply the margin_multiplier here so that
            // it is only called once the window has stopped moving.
            if let Some(window) = state.windows.iter_mut().find(|w| w.handle == h) {
                let loc = window.calculated_xyhw();
                let (x, y) = loc.center();
                let (margin_multiplier, tag, normal) =
                    match state.workspaces.iter().find(|ws| ws.contains_point(x, y)) {
                        Some(ws) => (ws.margin_multiplier(), ws.tag, ws.xyhw),
                        None => (1.0, Some(1), window.normal),
                    };
                let mut offset = window.get_floating_offsets().unwrap_or_default();
                // Re-adjust the floating offsets to the new workspace.
                let exact = window.normal + offset;
                offset = exact - normal;
                window.set_floating_offsets(Some(offset));
                window.tag = tag;
                window.apply_margin_multiplier(margin_multiplier);
                let act = DisplayAction::SetWindowTag(window.handle, tag);
                state.actions.push_back(act);
            }
            state.focus_window(&h);
        }
        _ => {}
    }
    state.mode = Mode::Normal;
    true
}

fn from_movement(state: &mut State, handle: WindowHandle, x: i32, y: i32) -> bool {
    if state.screens.iter().any(|s| s.root == handle) {
        state.focus_workspace_with_point(x, y);
    }
    false
}

fn from_move_window<C: Config, SERVER: DisplayServer>(
    manager: &mut Manager<C, SERVER>,
    handle: WindowHandle,
    x: i32,
    y: i32,
) -> bool {
    // Setup for when window first moves.
    if let Mode::ReadyToMove(h) = manager.state.mode {
        manager.state.mode = Mode::MovingWindow(h);
        prepare_window(&mut manager.state, h);
    }
    manager.window_move_handler(&handle, x, y)
}
fn from_resize_window<C: Config, SERVER: DisplayServer>(
    manager: &mut Manager<C, SERVER>,
    handle: WindowHandle,
    x: i32,
    y: i32,
) -> bool {
    // Setup for when window first resizes.
    if let Mode::ReadyToResize(h) = manager.state.mode {
        manager.state.mode = Mode::ResizingWindow(h);
        prepare_window(&mut manager.state, h);
    }
    manager.window_resize_handler(&handle, x, y)
}

fn from_configure_xlib_window(state: &mut State, handle: WindowHandle) -> bool {
    if let Some(window) = state.windows.iter().find(|w| w.handle == handle) {
        let act = DisplayAction::ConfigureXlibWindow(window.clone());
        state.actions.push_back(act);
    }
    false
}
// Save off the info about position of the window when we start to move/resize.
fn prepare_window(state: &mut State, handle: WindowHandle) {
    if let Some(w) = state.windows.iter_mut().find(|w| w.handle == handle) {
        if w.floating() {
            let offset = w.get_floating_offsets().unwrap_or_default();
            w.start_loc = Some(offset);
        } else {
            let container = w.container_size.unwrap_or_default();
            let normal = w.normal;
            let floating = normal - container;
            w.set_floating_offsets(Some(floating));
            w.start_loc = Some(floating);
            w.set_floating(true);
        }
    }
    state.move_to_top(&handle);
}
