use std::time::Duration;

use crate::cursor::Cursor;

use dyn_partial_eq::*;

#[dyn_partial_eq]
pub trait Listener {
    fn add_cursor(&self, cursor: &Cursor);
    fn update_cursor(&self, cursor: &Cursor);
    fn remove_cursor(&self, cursor: &Cursor);
    fn refresh(&self, duration: Duration);
}