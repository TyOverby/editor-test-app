extern crate editor_test_app;
extern crate syntect;

use editor_test_app::buffer::{Buffer, BufferView};

fn main() {
    let ss = syntect::parsing::SyntaxSet::load_defaults_newlines();
    let ts = syntect::highlighting::ThemeSet::load_defaults();

    let rust_syntax = ss.find_syntax_by_name("Rust").unwrap();
    let base_16 = &ts.themes["base16-ocean.dark"];
    let highlighter = syntect::highlighting::Highlighter::new(base_16);

    let buffer = Buffer::new("fn main() {\n\t5 + 5\n}");
    let mut view = BufferView::new(buffer, rust_syntax, &highlighter);

    let out = view.style_lines(0, 4, &highlighter);
    for (i, line) in out.into_iter().enumerate() {
        println!("LINE {}", i);
        for (style, string) in line {
            println!("  {:?} for {}", style, string);
        }
    }
}
