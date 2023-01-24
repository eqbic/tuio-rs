use std::time::Duration;

use crate::{cursor::Cursor, object::Object, blob::Blob};

use dyn_partial_eq::*;

#[dyn_partial_eq]
pub trait Listener {
    /// Adds an [Object] to the listener
    fn add_object(&mut self, object: &Object);

    /// Removes [Object]s
    ///
    /// # Arguments
    /// * `ids` - a slice of the IDs to remove
    fn remove_objects(&mut self, ids: &[i32]);
    
    /// Adds a [Cursor] to the listener
    fn add_cursor(&mut self, object: &Cursor);

    /// Removes [Cursor]s
    ///
    /// # Arguments
    /// * `ids` - a slice of the IDs to remove
    fn remove_cursors(&mut self, ids: &[i32]);
    
    /// Adds a [Blob] to the listener
    fn add_blob(&mut self, object: &Blob);

    /// Removes [Blob]s
    ///
    /// # Arguments
    /// * `ids` - a slice of the IDs to remove
    fn remove_blobs(&mut self, ids: &[i32]);
    fn refresh(&self, duration: Duration);
}