#[derive(Clone)]
pub struct ServerState {
    // TODO: Fill up with pools and such.
}

impl ServerState {
    /// Returns a ServerState with nothing initialized. For testing and the like, don't use in prod.
    pub fn empty() -> Self {
        ServerState {  }
    }

}