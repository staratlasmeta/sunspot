use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

#[derive(Debug, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenListItem {
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub image_uri: Option<String>,
}

#[derive(Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct TokenList(pub HashMap<String, TokenListItem>);

impl Deref for TokenList {
    type Target = HashMap<String, TokenListItem>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TokenList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
