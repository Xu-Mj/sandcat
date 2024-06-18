pub type Reason = String;
pub enum Error {
    /// convert server message to local error
    Convert(Reason),
    /// database query not found
    NotFound(Reason),
    /// database error
    Database(Reason),
    /// request server error
    Network(Reason),
    /// js related error
    JavaScript(Reason),
}
