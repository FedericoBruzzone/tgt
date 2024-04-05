use {serde::Deserialize, std::collections::HashMap};

#[derive(Clone, Debug, Deserialize)]
pub struct PaletteRaw {
    pub palette: Option<HashMap<String, String>>,
}
