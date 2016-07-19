extern crate syntect;

pub use syntect::highlighting::Theme;
pub use syntect::highlighting::Style;
pub use syntect::parsing::SyntaxDefinition;

use syntect::highlighting::{HighlightState, HighlightIterator, Highlighter};
use syntect::parsing::{ParseState, ScopeStack};

#[derive(Clone)]
pub struct HiState {
    theme: Theme,
    syntax: SyntaxDefinition,
    highlight_state: HighlightState,
    parse_state: ParseState,
}

impl PartialEq for HiState {
    fn eq(&self, other: &HiState) -> bool {
        self.theme.name == other.theme.name &&
        self.syntax == other.syntax &&
        self.highlight_state == other.highlight_state &&
        self.parse_state == other.parse_state
    }
}

impl HiState {
    pub fn new(theme: Theme, syntax: SyntaxDefinition) -> HiState {
        let parse_state = ParseState::new(&syntax);
        let hi_state = {
            let highlighter = Highlighter::new(&theme);
            let hi_state = HighlightState::new(&highlighter, ScopeStack::new());
            hi_state
        };

        HiState {
            theme: theme,
            syntax: syntax,
            highlight_state: hi_state,
            parse_state: parse_state,
        }
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    pub fn advanced_line(&mut self, string: &str) {
        let operations = self.parse_state.parse_line(string);
        for (_, op) in operations {
            self.highlight_state.path.apply(&op);
        }
    }

    pub fn highlight_and_advance_line<'a>(&mut self, string: &'a str) -> Vec<(Style, &'a str)> {
        let &mut HiState {ref mut theme, ref mut parse_state, ref mut highlight_state, ..} = self;

        let operations = parse_state.parse_line(string);

        let highlighter = Highlighter::new(theme);
        let highlight_iter = HighlightIterator::new(highlight_state, &operations[..], string, &highlighter);
        highlight_iter.collect()
    }
}

pub fn load_prebuilt_theme(name: &str) -> Option<Theme> {
    let ts = syntect::highlighting::ThemeSet::load_defaults();
    ts.themes.get(name).cloned()
}

pub fn load_prebuilt_syntax(name: &str) -> Option<SyntaxDefinition> {
    let ss = syntect::parsing::SyntaxSet::load_defaults_newlines();
    ss.find_syntax_by_name(name).cloned()
}
