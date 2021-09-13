use chrono::{DateTime, Utc};
use hdk::prelude::*;

#[derive(Serialize, Deserialize, Clone, SerializedBytes, Debug)]
pub struct Link {
    pub source: String,
    pub target: String,
    pub predicate: Option<String>
}

#[derive(Serialize, Deserialize, Clone, SerializedBytes, Debug)]
pub struct ExpressionProof {
    pub signature: String,
    pub key: String,
}

#[derive(Serialize, Deserialize, Clone, SerializedBytes, Debug)]
pub struct LinkExpression {
    author: String,
    timestamp: DateTime<Utc>,    
    data: Link,
    proof: ExpressionProof,
}

#[derive(Serialize, Deserialize, Clone, SerializedBytes, Debug)]
pub struct Perspective {
    pub links: Vec<LinkExpression>
}

#[hdk_entry(id = "expression", visibility = "public")]
#[derive(Clone)]
pub struct PerspectiveExpression {
    pub author: String,
    pub timestamp: DateTime<Utc>,
    pub data: Perspective,
    pub proof: ExpressionProof
}