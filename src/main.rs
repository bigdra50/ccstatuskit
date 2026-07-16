use std::io::Read;

fn main() {
    // A statusline must never break Claude Code's UI: consume stdin,
    // render what we can, and always exit 0.
    let mut input = String::new();
    let _ = std::io::stdin().read_to_string(&mut input);
}
