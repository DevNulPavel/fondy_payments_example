use sha1::{
    Digest
};
use tracing::{
    debug, 
    instrument
};
use crate::{
    error::{
        FondyError
    },
};

#[instrument(err, skip(json_data, skip_keys, password))]
pub fn calculate_signature(password: &str, json_data: &serde_json::Value, skip_keys: &[&str]) -> Result<String, FondyError> {
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
        .filter(|val|{
            // Фильтруем на всякий пожарный ключ сигнатуры
            !skip_keys.contains(&val.0.as_str())
        })
        .fold(password.to_owned(), |mut prev, val|{
            match val.1 {
                serde_json::Value::Bool(_) |
                serde_json::Value::Number(_) => {
                    prev.push_str("|");
                    prev.push_str(val.1.to_string().trim_matches('\"'));
                },
                serde_json::Value::String(text) if !text.is_empty() => {
                    prev.push_str("|");
                    prev.push_str(val.1.to_string().trim_matches('\"'));
                }
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