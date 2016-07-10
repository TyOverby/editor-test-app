use ::xi_rope::{Rope};
use ::syntect::highlighting::HighlightState;
use ::syntect::parsing::ParseState;

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
struct EditId(u64);
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct Location(u64, u64);

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
    /// A list of edits that have "undone"
    redo_stack: Vec<(EditId, Location, Rope)>,
}

pub struct BufferView {
    /// The underlying buffer type.
    buffer: Buffer,
    /// The type of file that the buffer should be highlighted with.
    filetype: Option<String>,
    /// A list of (line number, parse state, highlight state) that is used to cache parsing for
    /// highlighting operations.  An edit on line "N" invalidates all caches
    /// at line > N
    parse_cache: Vec<(Location, ParseState, HighlightState)>,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            last_edit_id: EditId(0),
            edits: vec![(EditId(0), Location(0, 0), Rope::from(""))],
            redo_stack: vec![],
        }
    }

    fn current_rope(&self) -> &Rope {
        let len = self.edits.len();
        &self.edits[len - 1].2
    }

    fn byte_at(&self, Location(line, chr): Location) -> u64 {
        self.current_rope().offset_of_line(line as usize) as u64 + chr
    }

    fn edit_rope<F>(&mut self, f: F) where F: FnOnce(&mut Rope) -> Location {
        self.last_edit_id.0 += 1;
        let my_edit_id = self.last_edit_id;

        let mut rope = self.current_rope().clone();
        let line = f(&mut rope);
        self.edits.push((my_edit_id, line, rope));
        self.redo_stack.clear();
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

    pub fn undo(&mut self) -> bool {
        if self.edits.len() == 1 {
            return false;
        } else {
            self.redo_stack.push(self.edits.pop().unwrap());
            return true;
        }
    }

    pub fn redo(&mut self) -> bool {
        if self.edits.len() == 1 {
            return false;
        } else {
            self.redo_stack.push(self.edits.pop().unwrap());
            return true;
        }
    }
}

impl BufferView {

}
