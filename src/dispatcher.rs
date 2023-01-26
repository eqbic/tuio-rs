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

        /// Adds an [Object] to the listener
        fn add_object(&mut self, object: &Object);

        /// Notifies a [Object] update
        ///
        /// # Arguments
        /// * `object` - the updated [Object]
        fn update_object(&mut self, object: &Object);

        /// Removes [Object]s
		///
		/// # Arguments
		/// * `ids` - a slice of the IDs to remove
        fn remove_objects(&mut self, ids: &[i32]);
        
        /// Adds a [Cursor] to the listener
        fn add_cursor(&mut self, cursor: &Cursor);
        
        /// Notifies a [Cursor] update
        ///
        /// # Arguments
        /// * `cursor` - the updated [Cursor]
        fn update_cursor(&mut self, cursor: &Cursor);

        /// Removes [Cursor]s
		///
		/// # Arguments
		/// * `ids` - a slice of the IDs to remove
        fn remove_cursors(&mut self, ids: &[i32]);
        
        /// Adds a [Blob] to the listener
        fn add_blob(&mut self, blob: &Blob);

        /// Notifies a [Blob] update
        ///
        /// # Arguments
        /// * `blob` - the updated [Blob]
        fn update_blob(&mut self, blob: &Blob);

        /// Removes [Blob]s
		///
		/// # Arguments
		/// * `ids` - a slice of the IDs to remove
        fn remove_blobs(&mut self, ids: &[i32]);

        /* 
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
        */
}

pub struct Dispatcher {
    pub listener_list: Vec<Box<dyn Listener>>
}

impl Dispatcher {
    pub fn new() -> Self {
        Dispatcher { listener_list: Vec::new() }
    }
}

impl Dispatch for Dispatcher {
    fn add_listener<L: Listener + 'static>(&mut self, listener: L) {
        self.listener_list.push(Box::new(listener))
    }

    fn remove_listener<L: Listener + 'static>(&mut self, listener: L) {
        let listener: Box<dyn Listener> = Box::new(listener);
        self.listener_list.retain(|x| x == &listener)
    }

    fn remove_all_listeners(&mut self) {
        self.listener_list.clear();
    }

    fn add_object(&mut self, object: &Object) {
        for listener in self.listener_list.iter_mut() {
            listener.add_object(object);
        }
    }

    fn update_object(&mut self, object: &Object) {
        for listener in self.listener_list.iter_mut() {
            listener.update_object(object);
        }
    }

    fn remove_objects(&mut self, ids: &[i32]) {
        for listener in self.listener_list.iter_mut() {
            listener.remove_objects(ids);
        }
    }
    
    fn add_cursor(&mut self, cursor: &Cursor) {
        for listener in self.listener_list.iter_mut() {
            listener.add_cursor(cursor);
        }
    }

    fn update_cursor(&mut self, cursor: &Cursor) {
        for listener in self.listener_list.iter_mut() {
            listener.update_cursor(cursor);
        }
    }

    fn remove_cursors(&mut self, ids: &[i32]) {
        for listener in self.listener_list.iter_mut() {
            listener.remove_cursors(ids);
        }
    }
    
    fn add_blob(&mut self, blob: &Blob) {
        for listener in self.listener_list.iter_mut() {
            listener.add_blob(blob);
        }
    }

    fn update_blob(&mut self, blob: &Blob) {
        for listener in self.listener_list.iter_mut() {
            listener.update_blob(blob);
        }
    }

    fn remove_blobs(&mut self, ids: &[i32]) {
        for listener in self.listener_list.iter_mut() {
            listener.remove_blobs(ids);
        }
    }
}
