use ::xi_rope::{Rope};
use ::syntect::highlighting::{HighlightState, Highlighter, HighlightIterator, Style};
use ::syntect::parsing::{ParseState, SyntaxDefinition, ScopeStack};

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
struct EditId(u64);
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct Location(u64, u64);

pub struct BufferHandle<'a> {
    view: &'a mut BufferView,
    starting_edit_id: EditId,
}

pub struct Buffer {
    /// The last id for the edit done on the buffer
    last_edit_id: EditId,
    /// A list of tuples where the edits go from oldest -> newest
    /// and where the last element in the list is the most recent edit.
    ///
    /// the tuple is of
    ///     EditId: a buffer-unique id of the edit.
    ///     LineNumber: the lowest line number effected by the edit.
    ///     Rope: The entire text at that point.
    edits: Vec<(EditId, Location, Rope)>,
}

pub struct BufferView {
    /// The underlying buffer type.
    buffer: Buffer,
    null_parse_state: ParseState,
    null_highlight_state: HighlightState,
    /// A list of (line number, parse state, highlight state) that is used to cache parsing for
    /// highlighting operations.  An edit on line "N" invalidates all caches
    /// at line > N
    highlight_cache: Vec<(Location, ParseState, HighlightState)>,
}

impl Buffer {
    pub fn empty() -> Buffer {
        Buffer {
            last_edit_id: EditId(0),
            edits: vec![(EditId(0), Location(0, 0), Rope::from(""))],
        }
    }

    pub fn new<S: Into<Rope>>(string: S) -> Buffer {
        Buffer { 
            last_edit_id: EditId(0),
            edits: vec![(EditId(0), Location(0, 0), string.into())],
        }
    }

    fn current_rope(&self) -> &Rope {
        let len = self.edits.len();
        &self.edits[len - 1].2
    }

    fn current_id(&self) -> EditId {
        self.last_edit_id
    }

    fn byte_at(&self, Location(line, chr): Location) -> u64 {
        self.current_rope().offset_of_line(line as usize) as u64 + chr
    }

    fn edit_rope<F>(&mut self, f: F) where F: FnOnce(&mut Rope) -> Location {
        self.last_edit_id.0 += 1;
        let my_edit_id = self.current_id();

        let mut rope = self.current_rope().clone();
        let line = f(&mut rope);
        self.edits.push((my_edit_id, line, rope));
    }

    pub fn insert<S: Into<Rope>>(&mut self, loc: Location, r: S) {
        let byte_loc = self.byte_at(loc);
        self.edit_rope(|rope| {
            let pos = byte_loc as usize;
            rope.edit(pos, pos, r.into());
            loc
        });
    }

    pub fn delete(&mut self, start: Location, end: Location) {
        let byte_start = self.byte_at(start);
        let byte_end = self.byte_at(end);
        assert!(byte_start < byte_end);

        self.edit_rope(|rope| {
            rope.edit_str(byte_start as usize, byte_end as usize, "");
            start
        });
    }
}

impl BufferView {
    pub fn new(buffer: Buffer, syntax_definition: &SyntaxDefinition, highlighter: &Highlighter) -> BufferView {
        BufferView {
            buffer: buffer,
            null_parse_state: ParseState::new(syntax_definition),
            null_highlight_state: HighlightState::new(highlighter, ScopeStack::new()),
            highlight_cache: vec![],

        }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn buffer_mut(&mut self) -> BufferHandle {
        let current_edit_id = self.buffer.last_edit_id;
        BufferHandle {
            view: self,
            starting_edit_id: current_edit_id,
        }
    }

    pub fn style_lines<'a>(&'a mut self, start: u64, end: u64, highlighter: &Highlighter) -> Vec<Vec<(Style, String)>> {
        let mut parse_state = self.null_parse_state.clone();
        let mut hi_state = self.null_highlight_state.clone();

        let mut styled_lines: Vec<Vec<(_, String)>> = vec![];

        for (i, line) in self.buffer.current_rope().lines().enumerate() {
            if i >= end as usize {
                break;
            }

            let text = &*line;
            let changes = parse_state.parse_line(text);
            let highlight_iter = HighlightIterator::new(&mut hi_state, &changes, text, highlighter);
            let styled = highlight_iter.map(|(a, b)| (a, b.into())).collect();
            
            if i >= start as usize {
                styled_lines.push(styled);
            }
        }

        return styled_lines;
    }

    fn invalidate_line(&mut self, invalid_loc: Location) {
        let mut truncate_pos = None;
        for (i, &(loc, _, _)) in self.highlight_cache.iter().enumerate() {
            if loc >= invalid_loc {
                truncate_pos = Some(i);
                break;
            }
        }

        if let Some(t_pos) = truncate_pos {
            self.highlight_cache.truncate(t_pos);
        }
    }

    fn invalidate_from_edit(&mut self, EditId(start): EditId) {
        let lowest = {
            let slice_of_edits = &self.buffer.edits[start as usize ..];
            slice_of_edits.iter().map(|&(_, loc, _)| loc).min()
        };

        if let Some(lowest) = lowest {
            self.invalidate_line(lowest);
        }
    }
}

impl <'a> BufferHandle<'a> {
    pub fn drop(self) {}

    pub fn edit(&mut self) -> &mut Buffer {
        &mut self.view.buffer
    }
}

impl <'a> Drop for BufferHandle<'a> {
    fn drop(&mut self) {
        self.view.invalidate_from_edit(self.starting_edit_id);
    }
}
