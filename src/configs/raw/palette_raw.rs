use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize)]
/// The raw palette configuration.
pub struct PaletteRaw {
    /// The palette.
    pub palette: Option<HashMap<String, String>>,
}
