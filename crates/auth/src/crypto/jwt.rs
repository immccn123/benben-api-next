use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde_json::Value;

pub fn create_jwt(
    claims: &Value,
    private_key_pem: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let encoding_key = EncodingKey::from_ec_pem(private_key_pem.as_bytes())?;
    let header = Header::new(jsonwebtoken::Algorithm::ES256);
    encode(&header, claims, &encoding_key)
}

pub fn verify_jwt(token: &str, public_key_pem: &str) -> Result<Value, jsonwebtoken::errors::Error> {
    let decoding_key = DecodingKey::from_ec_pem(public_key_pem.as_bytes())?;
    let mut validation = Validation::new(jsonwebtoken::Algorithm::ES256);
    validation.validate_exp = true;
    validation.required_spec_claims.clear();
    let data = decode::<Value>(token, &decoding_key, &validation)?;
    Ok(data.claims)
}
