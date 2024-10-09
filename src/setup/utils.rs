use k256::ecdsa::SigningKey;
use rand::rngs::OsRng;
use yaml_rust2::Yaml;

pub fn yaml_find_hash_node<'a>(node: &'a Yaml, path: &str) -> Option<&'a Yaml> {
    let (node_name, path) = match path.split_once(".") {
        Some((node_name, path)) => (node_name, Some(path)),
        None => (path, None),
    };

    match node {
        Yaml::Hash(map) => {
            if let Some((_, value)) = map
                .iter()
                .find(|(key, _)| matches!(key, Yaml::String(name) if name == node_name ))
            {
                let Some(path) = path else {
                    return Some(value);
                };

                yaml_find_hash_node(value, path)
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn generate_password() -> String {
    format!(
        "0x{}",
        hex::encode(SigningKey::random(&mut OsRng).to_bytes())
    )
}
