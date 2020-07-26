use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectAclResource {
    /// Value: "storage#objectAccessControl"
    pub kind: Option<String>,
    pub entity: String,
    pub role: Option<String>,
    pub email: Option<String>,
    pub entity_id: Option<String>,
    pub domain: Option<String>,
    pub project_team: Option<ObjectAclProjectTeam>,
    pub etag: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectAclProjectTeam {
    pub project_number: String,
    pub team: String,
}
