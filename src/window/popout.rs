//! Native space views. The main AppWindow remains the only model/document owner.
//! Pop-outs borrow its existing terminal entities: no second Core, model, or poller.
use super::*;

fn native_space_supported(space: &model::Space) -> bool {
    space.tabs.iter().all(|tab| tab.root.walk().iter().all(|n| {
        n.floating.is_empty()
            && !matches!(n.kind, Kind::Stack { .. })
            && n.content().is_none_or(|content| matches!(content, Content::Terminal { .. }))
    }))
}

fn local_command(name: &str) -> bool {
    matches!(name, "tab.new" | "tab.select" | "tab.next" | "tab.prev" | "tab.rename" | "pane.split" | "pane.zoom" | "pane.focus" | "pane.go" | "terminal.copy" | "terminal.paste" | "terminal.search" | "terminal.scroll" | "terminal.paste_image")
}

fn local_action(name: &str, args: Value) -> Result<(String, Value), String> {
    let current = crate::keys::current(name);
    let (base, shorthand) = current.split_once(':').map_or((current.as_str(), None), |(n, v)| (n, Some(v)));
    if !local_command(base) {
        return Err(format!("{name} is not available in a space window; return the space to main to use it"));
    }
    let command = crate::commands::find(base).ok_or("unknown local command")?;
    crate::commands::validate_args(command, &args)?;
    let args = if let Some(value) = shorthand {
        if command.args.is_empty() { return Err("command takes no shorthand argument".into()); }
        let mut expanded = crate::commands::shorthand(command, value);
        if let (Some(target), Some(extra)) = (expanded.as_object_mut(), args.as_object()) { target.extend(extra.clone()); }
        expanded
    } else { args };
    Ok((base.to_owned(), args))
}

fn check_local_target(model: &Model, space: &str, args: &Value) -> Result<(), &'static str> {
    let target_space = model.space(space).ok_or("space no longer exists")?;
    if !args.is_null() && !args.is_object() { return Err("args must be an object"); }
    if args.get("mod").is_some_and(|v| !v.is_null()) { return Err("Return this space to main before adding a mod pane"); }
    for key in ["id", "pane", "space"] {
        if let Some(value) = args.get(key).filter(|v| !v.is_null()) {
            let id = value.as_str().ok_or("target IDs must be strings")?;
            if model.space(id).or_else(|| model.space_of(id)).is_none_or(|s| s.id != space) {
                return Err("target is outside this space window");
            }
        }
    }
    if let Some(value) = args.get("terminal").filter(|v| !v.is_null()) {
        let address = value.as_str().ok_or("terminal must be a string")?;
        if !target_space.tabs.iter().any(|t| model::addresses(&t.root).iter().any(|a| a == address)) {
            return Err("terminal is outside this space window");
        }
    }
    Ok(())
}

pub(super) struct SpaceWindow {
    owner: Entity<AppWindow>,
    main: WindowHandle<AppWindow>,
    space: String,
    focus: FocusHandle,
    ready: bool,
    menu: Option<(Point<Pixels>, String)>,
    pub(super) error: Option<String>,
    _observe: Subscription,
}

impl SpaceWindow {
    fn new(owner: Entity<AppWindow>, main: WindowHandle<AppWindow>, space: String, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let observed = cx.observe(&owner, |_, _, cx| cx.notify());
        let focus = cx.focus_handle();
        focus.focus(window, cx);
        let closing_owner = owner.downgrade();
        let closing_space = space.clone();
        window.on_window_should_close(cx, move |_, cx| {
            if let Some(owner) = closing_owner.upgrade() {
                owner.update(cx, |owner, cx| {
                    owner.popouts.remove(&closing_space);
                    owner.popout_active.remove(&closing_space);
                    if owner.model.space(&closing_space).is_some() {
                        owner.model.active = Some(closing_space.clone());
                    }
                    cx.notify();
                });
                let _ = main.update(cx, |owner, window, cx| owner.sync(window, cx));
            }
            true
        });
        Self { owner, main, space, focus, ready: false, menu: None, error: None, _observe: observed }
    }

    pub(super) fn run(&mut self, name: &str, args: Value, window: &mut Window, cx: &mut Context<Self>) -> Value {
        self.menu = None;
        if name == "window.close" || name == "space.dock" {
            self.owner.update(cx, |owner, _| {
                owner.popouts.remove(&self.space);
                owner.popout_active.remove(&self.space);
                owner.model.active = Some(self.space.clone());
            });
            window.remove_window();
            let _ = self.main.update(cx, |owner, window, cx| { owner.sync(window, cx); window.activate_window(); });
            return json!({"ok":true});
        }
        if name == "window.minimize" { window.minimize_window(); return json!({"ok":true}); }
        if name == "window.maximize" { window.zoom_window(); return json!({"ok":true}); }
        // Commands with a window-local terminal target use this OS window's focus.
        let result = self.owner.update(cx, |owner, cx| owner.run_in_space(&self.space, name, args, window, cx));
        self.error = result.get("error").and_then(Value::as_str).map(str::to_owned);
        let _ = self.main.update(cx, |owner, window, cx| {
            owner.sync(window, cx);
            owner.flush_documents(false, window, cx);
        });
        if name != "terminal.search" { self.refocus(window, cx); }
        cx.notify();
        result
    }

    fn refocus(&self, window: &mut Window, cx: &mut App) {
        let owner = self.owner.read(cx);
        if let Some(focus) = owner.model.space(&self.space).and_then(|s| s.active_tab())
            .and_then(|t| t.focused_pane()).and_then(|p| owner.pane_focus(&p, cx))
        { focus.focus(window, cx); }
    }
}

impl Render for SpaceWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // GPUI draws synchronously inside open_window, while the model owner
        // is still borrowed. Do not read it until ownership has been registered.
        if !self.ready { return div().into_any_element(); }
        crate::terminal::next_frame();
        window.set_rem_size(px(settings::ui_font_size()));
        let space = self.owner.read(cx).model.space(&self.space).cloned();
        let Some(space) = space else {
            window.remove_window();
            return div().into_any_element();
        };
        self.owner.update(cx, |owner, cx| {
            if window.is_window_active() { owner.popout_active.insert(self.space.clone()); }
            else { owner.popout_active.remove(&self.space); }
            if window.is_window_active() {
                let addresses = space.active_tab().map(|t| model::addresses(t.shown())).unwrap_or_default();
                owner.model.mark_seen(&addresses);
                owner.share_seen(cx);
            }
        });
        let name = self.owner.read(cx).model.space_name(&space, &home());
        window.set_window_title(&format!("{name} — Herdr"));
        let mut tabs = div().id("space-tabs").flex().flex_row().h_full().flex_1().min_w_0().overflow_x_scroll();
        for tab in &space.tabs {
            let id = tab.id.clone();
            let active = space.active_tab().is_some_and(|t| t.id == tab.id);
            let status = self.owner.read(cx).model.rollup(&model::addresses(&tab.root));
            let label = Model::tab_name(&space, tab);
            tabs = tabs.child(div().id(SharedString::from(format!("popout-tab-{}", tab.id)))
                .flex_none().flex().items_center().px_3().h_full().gap_2().cursor_pointer()
                .bg(c(if active { "raised" } else { "surface" }))
                .text_color(c(if active { "foreground" } else { "muted" }))
                .on_click(cx.listener(move |this, _, window, cx| { this.run("tab.select", json!({"id":id}), window, cx); }))
                .child(label)
                .children(status.map(|status| div().text_color(c(status)).child("●"))));
        }
        let body = self.owner.update(cx, |owner, cx| {
            let Some(tab) = space.active_tab() else { return div().into_any_element() };
            let focused = tab.focused_pane().and_then(|p| tab.root.path_to(&p)).unwrap_or_default().into_iter().collect();
            owner.render_node(tab.shown(), &focused, None, cx)
        });
        let mut root = div().id("space-popout").size_full().relative().flex().flex_col()
            .track_focus(&self.focus).key_context("Herdr").bg(c("background")).text_color(c("foreground"))
            .font_family(settings::get().ui_font_family.clone())
            .capture_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                if this.menu.is_some() && event.keystroke.key == "escape" {
                    this.menu = None;
                    this.refocus(window, cx);
                    cx.stop_propagation();
                    cx.notify();
                }
            }))
            .on_action(cx.listener(|this, action: &ShellAction, window, cx| {
                if action.id() == "terminal.copy" {
                    if !this.owner.update(cx, |owner, cx| owner.copy_key(window, cx)) { cx.propagate(); }
                    return;
                }
                this.run(action.id(), Value::Null, window, cx);
            }))
            .on_action(cx.listener(|this, action: &crate::keys::CommandBinding, window, cx| {
                if let Ok((name, args)) = action.command() { this.run(&name, args, window, cx); }
            }))
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                this.owner.update(cx, |owner, cx| {
                    if owner.drag.is_some() && event.pressed_button == Some(MouseButton::Left) {
                        owner.drag_to(event.position); cx.notify();
                    }
                });
            }))
            .on_mouse_up(MouseButton::Left, cx.listener(|this, _, _, cx| {
                let _ = this.main.update(cx, |owner, window, cx| {
                    if owner.drag.take().is_some() { owner.save(); owner.publish(cx); owner.flush_documents(false, window, cx); }
                });
            }))
            .child(div().h(settings::z(38.)).flex_none().flex().flex_row().items_center().bg(c("surface"))
                .child(tabs)
                .child(div().id("popout-new-tab").px_3().cursor_pointer().child("+")
                    .on_click(cx.listener(|this, _, window, cx| { this.run("tab.new", json!({}), window, cx); })))
                .child(div().id("popout-return").px_3().cursor_pointer().child("Return to main window")
                    .on_click(cx.listener(|this, _, window, cx| { this.run("space.dock", Value::Null, window, cx); }))))
            .child(div().flex_1().min_h_0().child(body));
        if let Some(error) = &self.error {
            root = root.child(div().p_2().text_color(c("blocked")).child(error.clone()));
        }
        if let Some((at, pane)) = &self.menu {
            let mut menu = div().occlude().on_mouse_down_out(cx.listener(|this, _, window, cx| {
                this.menu = None; this.refocus(window, cx); cx.notify();
            })).absolute().left(at.x.min(window.viewport_size().width - px(210.)).max(px(0.)))
                .top(at.y.min(window.viewport_size().height - px(230.)).max(px(0.)))
                .w(px(210.)).p_1().bg(c("raised")).border_1().border_color(c("border")).flex().flex_col();
            for (label, command, args) in [
                ("Copy", "terminal.copy", json!({})), ("Paste", "terminal.paste", json!({})),
                ("Find", "terminal.search", json!({})),
                ("Split right", "pane.split", json!({"pane":pane,"direction":"right"})),
                ("Split down", "pane.split", json!({"pane":pane,"direction":"down"})),
                ("Zoom pane", "pane.zoom", json!({"pane":pane})),
                ("Dismiss", "", json!({})),
            ] {
                menu = menu.child(div().id(SharedString::from(format!("popout-menu-{label}"))).p_2().cursor_pointer().hover(|s| s.bg(c("hover"))).child(label)
                    .on_click(cx.listener(move |this, _, window, cx| {
                        if command.is_empty() { this.menu = None; this.refocus(window, cx); cx.notify(); }
                        else { this.run(command, args.clone(), window, cx); }
                    })));
            }
            root = root.child(menu);
        }
        root.into_any_element()
    }
}

impl AppWindow {
    pub(super) fn address_detached(&self, address: &str) -> bool {
        self.model.spaces.iter().filter(|s| self.popouts.contains_key(&s.id))
            .flat_map(|s| &s.tabs).any(|t| model::addresses(&t.root).iter().any(|a| a == address))
    }

    pub(super) fn close_space_windows(&mut self, cx: &mut Context<Self>) {
        for (_, handle) in self.popouts.drain() {
            let _ = handle.update(cx, |_, window, _| window.remove_window());
        }
        self.popout_active.clear();
    }

    pub(super) fn reconcile_space_windows(&mut self, cx: &mut Context<Self>) {
        let gone: Vec<_> = self.popouts.keys().filter(|id| self.model.space(id).is_none_or(|s| !native_space_supported(s))).cloned().collect();
        for id in gone {
            if let Some(handle) = self.popouts.remove(&id) {
                let _ = handle.update(cx, |_, window, _| window.remove_window());
            }
            self.popout_active.remove(&id);
            if self.model.space(&id).is_some() { self.model.active = Some(id); }
        }
    }

    pub(super) fn looking_addresses(&self) -> HashSet<String> {
        let mut looking: HashSet<_> = if self.window_active { self.visible_addresses().into_iter().collect() } else { HashSet::new() };
        for space in self.model.spaces.iter().filter(|s| self.popout_active.contains(&s.id) && self.popouts.contains_key(&s.id)) {
            if let Some(tab) = space.active_tab() { looking.extend(model::addresses(tab.shown())); }
        }
        looking
    }

    pub(super) fn popout_menu(&mut self, pane: &str, at: Point<Pixels>, cx: &mut Context<Self>) -> bool {
        let Some(handle) = self.model.space_of(pane).and_then(|s| self.popouts.get(&s.id)).copied() else { return false };
        if let Some(id) = self.model.tab_of(pane).map(|t| t.id.clone()) {
            self.model.tab_mut(&id).unwrap().focus(pane);
        }
        let pane = pane.to_owned();
        let _ = handle.update(cx, |view, window, cx| {
            view.menu = Some((at, pane));
            view.focus.focus(window, cx);
            cx.notify();
        });
        true
    }

    pub(super) fn open_space_window(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) -> Result<Value, String> {
        if let Some(handle) = self.popouts.get(id) {
            handle.update(cx, |_, window, _| window.activate_window()).map_err(|e| e.to_string())?;
            return Ok(json!({"id":id,"detached":true}));
        }
        let space = self.model.space(id).ok_or("no such space")?;
        // A pane mod owns arbitrary window-bound callbacks. Never silently move
        // one to another window or execute its cleanup while detaching terminals.
        if !native_space_supported(space) {
            return Err("This space contains mod/layout panes, stacks or floating blocks; native pop-outs currently support terminal panes with row/column splits. Its tabs have not been changed.".into());
        }
        let main = window.window_handle().downcast::<AppWindow>().ok_or("not the main window")?;
        let owner = cx.entity();
        let space = id.to_owned();
        let bounds = Bounds::centered(None, size(px(1000.), px(700.)), cx);
        let handle = cx.open_window(WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitlebarOptions { title: Some("Herdr space".into()), ..Default::default() }),
            window_decorations: Some(WindowDecorations::Server),
            ..Default::default()
        }, |window, cx| cx.new(|cx| SpaceWindow::new(owner, main, space, window, cx)))
            .map_err(|e| e.to_string())?;
        self.popouts.insert(id.to_owned(), handle);
        if self.model.active.as_deref() == Some(id) {
            self.model.active = self.model.spaces.iter().find(|s| !self.popouts.contains_key(&s.id)).map(|s| s.id.clone());
        }
        let focus = self.model.space(id).and_then(|s| s.active_tab()).and_then(|t| t.focused_pane()).and_then(|p| self.pane_focus(&p, cx));
        let _ = handle.update(cx, |view, window, cx| {
            view.ready = true;
            if let Some(focus) = focus { focus.focus(window, cx); }
            cx.notify();
        });
        self.sync(window, cx);
        Ok(json!({"id":id,"detached":true}))
    }

    pub(super) fn dock_space(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) -> Result<Value, String> {
        if self.model.space(id).is_none() { return Err("no such space".into()); }
        self.popout_active.remove(id);
        if let Some(handle) = self.popouts.remove(id) {
            // remove_window does not run the close-request hook; update ownership here.
            let _ = handle.update(cx, |_, window, _| window.remove_window());
        }
        self.model.active = Some(id.to_owned());
        self.sync(window, cx);
        window.activate_window();
        Ok(json!({"id":id,"detached":false}))
    }

    /// Execute model commands against a space without changing the main window's
    /// selection or running a second document writer. Only supported local actions
    /// are accepted; unsupported shortcuts never fall through to the main space.
    fn run_in_space(&mut self, space: &str, name: &str, args: Value, window: &mut Window, cx: &mut Context<Self>) -> Value {
        let (name, args) = match local_action(name, args) {
            Ok(action) => action,
            Err(error) => return json!({"error":error}),
        };
        if let Err(error) = check_local_target(&self.model, space, &args) { return json!({"error":error}); }
        let active = self.model.active.replace(space.to_owned());
        let block = self.focused_block.take();
        let focused = self.focused_pane.take();
        self.focused_pane = self.model.focused_pane();
        self.popout_command = true;
        let result = self.run_as(&name, args, false, window, cx);
        self.popout_command = false;
        self.model.active = active;
        self.focused_pane = focused;
        self.focused_block = block;
        self.save_view();
        cx.notify();
        result
    }

    pub(super) fn focus_owned_pane(&mut self, id: &str, cx: &mut Context<Self>) {
        let detached = self.model.space_of(id).is_some_and(|s| self.popouts.contains_key(&s.id));
        if detached {
            if let Some(tab_id) = self.model.tab_of(id).map(|t| t.id.clone()) {
                if let Some(tab) = self.model.tab_mut(&tab_id) { tab.focus(id); }
            }
        } else {
            self.model.focus_pane(id);
            self.focused_pane = Some(id.to_owned());
            self.synced_model_pane = self.model.focused_pane();
        }
        self.publish(cx);
        self.save();
        cx.notify();
    }
}

#[cfg(test)]
mod tests {
    use super::{check_local_target, local_action, local_command, native_space_supported};
    use crate::layout::{Axis, Content, Kind, Node};
    use crate::model::{Model, Tab};
    use serde_json::{Value, json};

    fn model() -> Model {
        let mut model = Model::default();
        model.add_space("main", "Main", "/main", Tab::new("a", "A", "p1", "t1"));
        model.add_space("detached", "Detached", "/detached", Tab::new("b", "B", "p2", "t2"));
        model.add_tab("detached", Tab::new("c", "C", "p3", "remote:t3"));
        model
    }

    #[test]
    fn local_targets_never_escape_their_space() {
        let model = model();
        for args in [json!({}), json!({"id":"b"}), json!({"pane":"p3"}), json!({"space":"detached"}), json!({"terminal":"remote:t3"})] {
            assert!(check_local_target(&model, "detached", &args).is_ok());
        }
        for args in [json!({"id":"a"}), json!({"pane":"p1"}), json!({"space":"main"}), json!({"terminal":"t1"}), json!({"id":3}), json!({"pane":"unknown"}), json!({"mod":"files"}), json!([])] {
            assert!(check_local_target(&model, "detached", &args).is_err(), "{args}");
        }
        assert!(check_local_target(&model, "gone", &json!({})).is_err());
    }

    #[test]
    fn shorthand_is_expanded_before_window_target_validation() {
        let model = model();
        let (_, args) = local_action("pane.focus:p1", Value::Null).unwrap();
        assert!(check_local_target(&model, "detached", &args).is_err());
        let (_, args) = local_action("pane.focus:p2", Value::Null).unwrap();
        assert!(check_local_target(&model, "detached", &args).is_ok());
        assert!(local_action("tab.next:main", Value::Null).is_err());
    }

    #[test]
    fn terminal_space_eligibility_preserves_every_tab_and_address() {
        let mut model = model();
        model.split("p2", Axis::Row, "p4", "t4");
        let before = model.all_addresses();
        assert!(native_space_supported(model.space("detached").unwrap()));
        assert_eq!(model.space("detached").unwrap().tabs.len(), 2);
        assert_eq!(model.all_addresses(), before);
    }

    #[test]
    fn unsupported_hidden_tab_cannot_be_partially_detached() {
        let mut model = model();
        model.tab_mut("b").unwrap().root = Node::block("mod-pane", Content::Mod { name:"files".into(), props:Value::Null });
        assert_eq!(model.space("detached").unwrap().active.as_deref(), Some("c"));
        assert!(!native_space_supported(model.space("detached").unwrap()));
    }

    #[test]
    fn stacked_terminal_layout_is_not_moved_with_window_bound_headers() {
        let mut model = model();
        let tab = model.tab_mut("b").unwrap();
        tab.root = Node::new("stack", Kind::Stack { children:vec![tab.root.clone()], active:None, header:None });
        assert!(!native_space_supported(model.space("detached").unwrap()));
    }

    #[test]
    fn unknown_or_destructive_shortcuts_do_not_fall_through_to_main() {
        for name in ["tab.new", "tab.next", "pane.split", "terminal.search"] { assert!(local_command(name)); }
        for name in ["space.close", "tab.close", "app.quit", "layout.switch", "mod.callback", "terminal.kill"] { assert!(!local_command(name)); }
    }
}
