extern crate eta_buffer;
extern crate eta_highlight;

use eta_buffer::*;
use eta_highlight::*;

fn main() {
    let theme = load_default_theme("base16-ocean.dark").unwrap();
    let syntax = load_default_syntax("Rust").unwrap();

    let buffer = Buffer::new("fn main() {\n\t5 + 5\n}");
    let mut view = BufferView::new(buffer, syntax, theme);

    let out = view.style_lines(0, 4);
    for (i, line) in out.into_iter().enumerate() {
        println!("LINE {}", i);
        for (style, string) in line {
            println!("  {:?} for {}", style, string);
        }
    }
}
