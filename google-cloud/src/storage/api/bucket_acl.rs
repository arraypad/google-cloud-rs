use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BucketAclResource {
    /// Value: "storage#bucketAccessControl"
    pub kind: String,
    pub id: String,
    pub self_link: String,
    pub bucket: String,
    pub entity: String,
    pub role: String,
    pub email: Option<String>,
    pub entity_id: Option<String>,
    pub domain: Option<String>,
    pub project_team: Option<BucketAclProjectTeam>,
    pub etag: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BucketAclProjectTeam {
    pub project_number: String,
    pub team: String,
}
