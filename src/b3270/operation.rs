use serde::{Deserialize, Serialize};

// Operations
#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Clone)]
#[serde(rename_all="kebab-case")]
pub struct Run {
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub r_tag: Option<String>,
    #[serde(rename="type", default, skip_serializing_if="Option::is_none")]
    pub type_: Option<String>,
    pub actions: Vec<Action>,
}

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Clone)]
#[serde(rename_all="kebab-case")]
pub struct Action {
    pub action: String,
    #[serde(default, skip_serializing_if="Vec::is_empty")]
    pub args: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Clone)]
#[serde(rename_all="kebab-case")]
pub struct Register {
    pub name: String,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub help_text: Option<String>,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub help_params: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Clone)]
/// Completes a passthru action unsuccessfully
#[serde(rename_all="kebab-case")]
pub struct Fail {
    pub p_tag: String,
    pub text: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Clone)]
#[serde(rename_all="kebab-case")]
pub struct Succeed {
    pub p_tag: String,
    #[serde(default, skip_serializing_if="Vec::is_empty")]
    pub text: Vec<String>,
}
