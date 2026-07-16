use ccstatuskit::engine::{EngineOptions, ModuleRunner, render_rows};
use ccstatuskit::format::{ModuleRef, parse_format};
use ccstatuskit::segment::Segment;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

enum Stub {
    Emit(&'static str),
    Styled(&'static str, &'static str),
    Hide,
    Panic,
    SleepMs(u64),
}

struct StubRunner {
    map: HashMap<String, Stub>,
    calls: AtomicUsize,
}

impl StubRunner {
    fn new(entries: Vec<(&str, Stub)>) -> Arc<Self> {
        Arc::new(Self {
            map: entries
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
            calls: AtomicUsize::new(0),
        })
    }
}

impl ModuleRunner for StubRunner {
    fn run(&self, module: &ModuleRef) -> Option<Segment> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let name = match module {
            ModuleRef::Builtin(n) | ModuleRef::Custom(n) => n.clone(),
        };
        match self.map.get(&name) {
            Some(Stub::Emit(text)) => Some(Segment::plain(*text)),
            Some(Stub::Styled(text, spec)) => Some(Segment::styled(
                *text,
                ccstatuskit::style::parse_style(spec, &HashMap::new()).unwrap(),
            )),
            Some(Stub::Hide) | None => None,
            Some(Stub::Panic) => panic!("module blew up"),
            Some(Stub::SleepMs(ms)) => {
                std::thread::sleep(Duration::from_millis(*ms));
                Some(Segment::plain("late"))
            }
        }
    }
}

fn opts(budget_ms: u64, color: bool) -> EngineOptions {
    EngineOptions {
        budget: Duration::from_millis(budget_ms),
        color,
    }
}

#[test]
fn renders_text_and_segments_in_row_order() {
    let rows = parse_format("[$a] $b\n$c").unwrap();
    let runner = StubRunner::new(vec![
        ("a", Stub::Emit("A")),
        ("b", Stub::Emit("B")),
        ("c", Stub::Emit("C")),
    ]);
    let lines = render_rows(&rows, runner, &opts(1000, true));
    assert_eq!(lines, vec!["[A] B".to_string(), "C".to_string()]);
}

#[test]
fn panicking_module_omits_segment_but_row_survives() {
    let rows = parse_format("$boom $ok").unwrap();
    let runner = StubRunner::new(vec![("boom", Stub::Panic), ("ok", Stub::Emit("OK"))]);
    let lines = render_rows(&rows, runner, &opts(1000, true));
    assert_eq!(lines.len(), 1);
    assert!(lines[0].contains("OK"));
    assert!(!lines[0].contains("boom"));
}

#[test]
fn hidden_module_is_omitted_from_the_row() {
    let rows = parse_format("$gone $here").unwrap();
    let runner = StubRunner::new(vec![("gone", Stub::Hide), ("here", Stub::Emit("X"))]);
    let lines = render_rows(&rows, runner, &opts(1000, true));
    assert_eq!(lines, vec!["X".to_string()]);
}

#[test]
fn slow_module_is_abandoned_at_the_deadline() {
    let rows = parse_format("$slow $fast").unwrap();
    let runner = StubRunner::new(vec![
        ("slow", Stub::SleepMs(5000)),
        ("fast", Stub::Emit("F")),
    ]);
    let started = Instant::now();
    let lines = render_rows(&rows, runner, &opts(100, true));
    assert!(started.elapsed() < Duration::from_millis(2000));
    assert_eq!(lines, vec!["F".to_string()]);
}

#[test]
fn row_disappears_when_all_modules_hide_and_literals_are_whitespace() {
    let rows = parse_format("$a $b\n$keep").unwrap();
    let runner = StubRunner::new(vec![
        ("a", Stub::Hide),
        ("b", Stub::Hide),
        ("keep", Stub::Emit("K")),
    ]);
    let lines = render_rows(&rows, runner, &opts(1000, true));
    assert_eq!(lines, vec!["K".to_string()]);
}

#[test]
fn row_with_literal_ink_survives_hidden_modules() {
    let rows = parse_format("static $a").unwrap();
    let runner = StubRunner::new(vec![("a", Stub::Hide)]);
    let lines = render_rows(&rows, runner, &opts(1000, true));
    assert_eq!(lines, vec!["static".to_string()]);
}

#[test]
fn styled_segment_renders_ansi_when_color_is_on() {
    let rows = parse_format("$a").unwrap();
    let runner = StubRunner::new(vec![("a", Stub::Styled("hi", "bold"))]);
    let lines = render_rows(&rows, runner, &opts(1000, true));
    assert!(lines[0].contains('\u{1b}'));
    assert!(lines[0].contains("hi"));
}

#[test]
fn styled_segment_renders_plain_when_color_is_off() {
    let rows = parse_format("$a").unwrap();
    let runner = StubRunner::new(vec![("a", Stub::Styled("hi", "bold"))]);
    let lines = render_rows(&rows, runner, &opts(1000, false));
    assert_eq!(lines, vec!["hi".to_string()]);
}

#[test]
fn module_referenced_twice_runs_once() {
    let rows = parse_format("$a $a").unwrap();
    let runner = StubRunner::new(vec![("a", Stub::Emit("A"))]);
    let lines = render_rows(
        &rows,
        Arc::clone(&runner) as Arc<dyn ModuleRunner>,
        &opts(1000, true),
    );
    assert_eq!(lines, vec!["A A".to_string()]);
    assert_eq!(runner.calls.load(Ordering::SeqCst), 1);
}
