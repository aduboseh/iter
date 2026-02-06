#![cfg(feature = "schema-gen")]

//! Generates JSON Schemas for Iter MCP external contracts.
//!
//! Usage:
//! ```bash
//! cargo run --features schema-gen --example generate_schemas
//! ```

use std::{error::Error, fs, path::Path};

use schemars::schema::RootSchema;
use schemars::schema_for;
use serde_json::{json, Value};

use iter_mcp_server::audit::DecisionPacket;
use iter_mcp_server::substrate::stub::{
    AuditSearchFilter, AuditSearchResult, DecisionPreview, GovernanceProposal,
};

const SCHEMA_DIR: &str = "schemas/v1";
const DRAFT_URL: &str = "http://json-schema.org/draft-07/schema#";
const BASE_URL: &str = "https://iter.dev/schemas/v1/";

fn main() -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(SCHEMA_DIR)?;
    write_schema::<DecisionPacket>("decision_packet")?;
    write_schema::<DecisionPreview>("decision_preview")?;
    write_schema::<GovernanceProposal>("decision_check_request")?;
    write_audit_search_schema()?;
    Ok(())
}

fn write_schema<T: schemars::JsonSchema>(name: &str) -> Result<(), Box<dyn Error>> {
    let mut value = schema_value(schema_for!(T))?;
    value
        .as_object_mut()
        .expect("schema root is object")
        .insert(
            "$id".to_string(),
            Value::String(format!("{BASE_URL}{name}.schema.json")),
        );
    let path = Path::new(SCHEMA_DIR).join(format!("{name}.schema.json"));
    fs::write(path, serde_json::to_string_pretty(&value)?)?;
    Ok(())
}

fn write_audit_search_schema() -> Result<(), Box<dyn Error>> {
    let filter = schema_value(schema_for!(AuditSearchFilter))?;
    let result = schema_value(schema_for!(AuditSearchResult))?;
    let combined = json!({
        "$schema": DRAFT_URL,
        "$id": format!("{BASE_URL}audit_search.schema.json"),
        "title": "audit.search filter/result schemas",
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "filter": { "$ref": "#/definitions/AuditSearchFilter" },
            "result": { "$ref": "#/definitions/AuditSearchResult" }
        },
        "definitions": {
            "AuditSearchFilter": filter,
            "AuditSearchResult": result
        }
    });
    let path = Path::new(SCHEMA_DIR).join("audit_search.schema.json");
    fs::write(path, serde_json::to_string_pretty(&combined)?)?;
    Ok(())
}

fn schema_value(root: RootSchema) -> Result<Value, Box<dyn Error>> {
    let mut value = serde_json::to_value(root)?;
    enforce_no_additional(&mut value);
    if let Some(map) = value.as_object_mut() {
        map.insert("$schema".to_string(), Value::String(DRAFT_URL.to_string()));
    }
    Ok(value)
}

fn enforce_no_additional(node: &mut Value) {
    match node {
        Value::Object(map) => {
            if let Some(Value::String(kind)) = map.get("type") {
                if kind == "object" {
                    map.entry("additionalProperties")
                        .or_insert(Value::Bool(false));
                }
            }
            for value in map.values_mut() {
                enforce_no_additional(value);
            }
        }
        Value::Array(arr) => {
            for value in arr {
                enforce_no_additional(value);
            }
        }
        _ => {}
    }
}
