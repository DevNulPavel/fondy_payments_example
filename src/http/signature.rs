use std::{
    sync::{
        Arc
    }
};
use sha1::{
    Digest
};
use tracing::{
    debug, 
    error, 
    instrument
};
use warp::{
    Filter,
    Reply,
    Rejection,
    reject::{
        Reject
    }
};
use serde::{
    Deserialize
};
use serde_json::{
    json
};
use tap::{
    prelude::{
        *
    }
};
use reqwest_inspect_json::{
    InspectJson
};
use crate::{
    error::{
        FondyError
    },
    application::{
        Application
    }
};
use super::{
    responses::{
        FondyDataOrErrorResponse,
        FondyInvalidResponse,
        FondyRedirectUrlResponse,
        FondyResponse
    }
};

#[instrument]
pub fn calculate_signature(password: &str, json_data: &serde_json::Value) -> Result<String, FondyError> {
    let data_map = json_data
        .as_object()
        .ok_or_else(||{
            FondyError::SignatureCalculateError("Json data must be dictionary".to_owned())
        })?;

    let mut key_value_vec: Vec<(&String, &serde_json::Value)> = data_map
        .iter()
        .collect();

    key_value_vec
        .sort_by(|v1, v2|{
            v1.0.cmp(v2.0)
        });

    let joined_string = key_value_vec
        .iter()
        .fold(password.to_owned(), |mut prev, val|{
            match val.1 {
                serde_json::Value::Bool(_) |
                serde_json::Value::String(_) |
                serde_json::Value::Number(_) => {
                    prev.push_str("|");
                    prev.push_str(val.1.to_string().trim_matches('\"'));
                },
                _ =>{
                }
            }
            prev
        });
    debug!("Joined result: {}", joined_string);

    // TODO: Может быть проще в цикле просто вызывать update для каждого значения?
    let mut sha = sha1::Sha1::new();
    sha.update(joined_string);
    let result = format!("{:x}", sha.finalize());
    debug!("Result SHA-1 hash: {}", result);

    Ok(result)
}