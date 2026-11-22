use super::*;
use crate::render::KIND_RECT;
use crate::runtime::premul;
use crate::{Ime, Key, KeyEvent, Mods, SelectionStyle, TextEdit, TextStyle, text_input};

fn view(s: &TextEdit, _: &mut Cx<TextEdit>) -> El {
    text_input(s, TextStyle::new(14.0, 400.0))
        .single_line()
        .read_only(true)
        .key("field")
        .size(200.0, 30.0)
        .on_edit(|s: &mut TextEdit| s)
}

fn read_only() -> Runtime<TextEdit> {
    let mut edit = TextEdit::new("hello");
    edit.focused = true;
    edit.set_selection(0, 5);
    let mut rt = Runtime::new(edit, view, (200.0, 30.0), Env::new(true, 1.0));
    rt.frame(0.0);
    rt
}

fn press(rt: &mut Runtime<TextEdit>, key: Key) {
    rt.key(KeyEvent {
        key,
        mods: Mods::default(),
    });
    rt.frame(0.0);
}

#[test]
fn read_only_rejects_typing_deletion_paste_and_ime_but_keeps_selection() {
    let mut rt = read_only();
    press(&mut rt, Key::Text("x".into()));
    press(&mut rt, Key::Backspace);
    rt.paste("paste");
    rt.frame(0.0);
    rt.ime(Ime::Preedit("か".into(), None, Vec::new()));
    rt.ime(Ime::Commit("家".into()));
    rt.frame(0.0);
    assert_eq!(rt.state.text, "hello", "the value never changes");
    assert_eq!(rt.state.range(), (0, 5), "the initial selection stays");

    press(&mut rt, Key::Home);
    assert_eq!(rt.state.range(), (0, 0), "the caret still moves");
    press(&mut rt, Key::Right);
    assert_eq!(rt.state.range(), (1, 1));
    rt.key(KeyEvent {
        key: Key::Command('a'),
        mods: Mods::default(),
    });
    rt.frame(0.0);
    assert_eq!(rt.state.range(), (0, 5), "select all still works");
    rt.key(KeyEvent {
        key: Key::Command('c'),
        mods: Mods::default(),
    });
    rt.frame(0.0);
    assert_eq!(
        rt.copy_request.as_deref(),
        Some("hello"),
        "copy still works"
    );
}

#[test]
fn a_focused_read_only_field_does_not_take_ime_events_from_the_app() {
    #[derive(Default)]
    struct State {
        edit: TextEdit,
        log: Vec<String>,
    }
    fn app_view(s: &State, _: &mut Cx<State>) -> El {
        text_input(&s.edit, TextStyle::new(14.0, 400.0))
            .read_only(true)
            .key("field")
            .size(200.0, 30.0)
            .on_edit(|s: &mut State| &mut s.edit)
    }
    let mut edit = TextEdit::new("hello");
    edit.focused = true;
    let mut rt = Runtime::new(
        State {
            edit,
            ..Default::default()
        },
        app_view,
        (200.0, 30.0),
        Env::new(true, 1.0),
    );
    rt.hooks.on_ime = Some(|s, ev, _| {
        s.log.push(format!("{ev:?}"));
        true
    });
    rt.frame(0.0);
    rt.ime(Ime::Preedit("か".into(), None, Vec::new()));
    rt.ime(Ime::Commit("家".into()));
    rt.ime(Ime::Enabled);
    rt.ime(Ime::Disabled);
    rt.frame(0.0);
    assert_eq!(rt.state.edit.text, "hello");
    assert_eq!(
        rt.state.log,
        ["Enabled", "Disabled"],
        "composition belongs to the field, and it refuses it"
    );
}

fn count_fill(rt: &Runtime<TextEdit>, color: crate::Color) -> usize {
    let want = premul(color, 1.0, false);
    rt.instances
        .iter()
        .filter(|i| i.params[1] == KIND_RECT && (0..4).all(|k| (i.fill[k] - want[k]).abs() < 0.001))
        .count()
}

#[test]
fn a_field_keeps_its_selection_when_focus_or_the_window_leaves() {
    let mut edit = TextEdit::new("hello");
    edit.focused = true;
    edit.set_selection(1, 4);
    let view = |s: &TextEdit, _: &mut Cx<TextEdit>| {
        text_input(s, TextStyle::new(14.0, 400.0))
            .single_line()
            .selection_style(SelectionStyle::chromium())
            .key("field")
            .size(200.0, 30.0)
            .on_edit(|s: &mut TextEdit| s)
    };
    let mut rt = Runtime::new(edit, view, (200.0, 30.0), Env::new(true, 1.0));
    rt.frame(0.0);
    let style = SelectionStyle::chromium();
    let active = count_fill(&rt, style.pick(true).0);
    assert!(active > 0, "the focused field paints its active selection");

    rt.update(|s, _| s.focused = false);
    rt.frame(0.1);
    assert_eq!(
        count_fill(&rt, style.pick(true).0),
        active,
        "focus leaving keeps the selection"
    );
    assert_eq!(
        count_fill(&rt, style.pick(false).0),
        0,
        "the window is still active"
    );

    rt.window_focus(false);
    rt.frame(0.2);
    assert_eq!(
        count_fill(&rt, style.pick(true).0),
        0,
        "the window blur replaces the active pair"
    );
    assert!(
        count_fill(&rt, style.pick(false).0) > 0,
        "and paints the inactive pair"
    );
}
