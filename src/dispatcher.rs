use crate::{listener::Listener, object::Object, cursor::Cursor, blob::Blob};

/// The Dispatch trait generates TUIO events which are broadcasted to all 
/// objects that implement the Listener interface.

pub trait Dispatch {
        /// Adds the provided Listener to the list of registered TUIO event listeners
		///
		/// # Arguments
		/// * `listener` - the Listener to add
        fn add_listener<L: Listener + 'static>(&mut self, listener: L);

        /// Removes the provided Listener from the list of registered TUIO event listeners
		///
		/// # Arguments
		/// * `listener` - the Listener to remove
        fn remove_listener<L: Listener + 'static>(&mut self, listener: L);
 
        /// Removes all Listener from the list of registered TUIO event listeners
        fn remove_all_listeners(&mut self);

        fn get_listeners(&mut self) -> &mut Vec<Box<dyn Listener>>;

        /// Adds an [Object] to the listener
        fn add_object(&mut self, object: &Object) {
            for listener in self.get_listeners() {
                listener.add_object(object);
            }
        }

        /// Removes [Object]s
		///
		/// # Arguments
		/// * `ids` - a slice of the IDs to remove
        fn remove_objects(&mut self, ids: &[i32]) {
            for listener in self.get_listeners() {
                listener.remove_objects(ids);
            }
        }
        
        /// Adds a [Cursor] to the listener
        fn add_cursor(&mut self, cursor: &Cursor) {
            for listener in self.get_listeners() {
                listener.add_cursor(cursor);
            }
        }

        /// Removes [Cursor]s
		///
		/// # Arguments
		/// * `ids` - a slice of the IDs to remove
        fn remove_cursors(&mut self, ids: &[i32]) {
            for listener in self.get_listeners() {
                listener.remove_cursors(ids);
            }
        }
        
        /// Adds a [Blob] to the listener
        fn add_blob(&mut self, blob: &Blob) {
            for listener in self.get_listeners() {
                listener.add_blob(blob);
            }
        }

        /// Removes [Blob]s
		///
		/// # Arguments
		/// * `ids` - a slice of the IDs to remove
        fn remove_blobs(&mut self, ids: &[i32]) {
            for listener in self.get_listeners() {
                listener.remove_blobs(ids);
            }
        }
         
        /// Returns a slice of all currently active Objects
        fn get_objects(&self) -> Vec<&Object>;
 
        /// Returns the number of all currently active Objects
        fn get_object_count(&self) -> usize;
         
        /// Returns a slice of all currently active Cursors
        fn get_cursors(&self) -> Vec<&Cursor>;
 
        /// Returns the number of all currently active Cursors
        fn get_cursor_count(&self) -> usize;
        
        /// Returns a slice of all currently active Blobs
        fn get_blobs(&self) -> Vec<&Blob>;
 
        /// Returns the number of all currently active Blobs
        fn get_blob_count(&self) -> usize;
         
        ///  Returns an Option of the Object corresponding to the provided Session ID
        ///
		/// # Arguments
		/// * `session_id` - the id of the object
        fn get_object(&self, session_id: i32) -> Option<&Object>;
 
        ///  Returns an Option of the Cursor corresponding to the provided Session ID
        ///
		/// # Arguments
		/// * `session_id` - the id of the cursor
        fn get_cursor(&self, session_id: i32) -> Option<&Cursor>;

        ///  Returns an Option of the Blob corresponding to the provided Session ID
        ///
		/// # Arguments
		/// * `session_id` - the id of the blob
        fn get_blob(&self, session_id: i32) -> Option<&Blob>;
}