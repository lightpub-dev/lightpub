use crate::model::{
    http::{HeaderMapWrapper, Method},
    UserSpecifier,
};
use async_trait::async_trait;
use base64::prelude::*;
use derive_builder::Builder;
use derive_more::Constructor;
use derive_more::From;
use linked_hash_map::LinkedHashMap;
use rsa::pkcs1v15::Signature;
use rsa::signature::SignatureEncoding;
use rsa::signature::Signer;
use rsa::signature::Verifier;
use rsa::RsaPublicKey;
use rsa::{
    pkcs1v15::SigningKey, pkcs1v15::VerifyingKey, sha2::Digest, sha2::Sha256, RsaPrivateKey,
};
use thiserror::Error;
use tracing::debug;
use tracing::info;
use uuid::fmt::Simple;

pub fn generate() -> RsaPrivateKey {
    let mut rng = rand::thread_rng();
    let bits = 2048;
    let priv_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate private key");

    priv_key
}

#[derive(Debug, Builder)]
pub struct SignKey {
    private_key: RsaPrivateKey,
    id: String,
}

#[derive(Debug, Constructor)]
pub struct KeyFetcherResult {
    pub key: RsaPublicKey,
    pub user_id: Simple,
}

#[async_trait]
pub trait KeyFetcher {
    async fn fetch_pubkey(&mut self, id: &str) -> Result<KeyFetcherResult, anyhow::Error>;
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct NotIncludedHeader(String);
#[derive(Debug, Clone, Builder)]
struct SignedHeader {
    message_to_sign: String,
    headers: String,
}

fn make_signed_message(
    headers: HeaderMapWrapper,
    method: Method,
    url_path: &str,
    signed_headers: &[String],
) -> Result<SignedHeader, NotIncludedHeader> {
    let mut added_headers: LinkedHashMap<String, Vec<String>> = LinkedHashMap::new();
    for h in signed_headers {
        let h = h.to_lowercase();
        let value = if h == "(request-target)" {
            format!("{} {}", method.as_str().to_lowercase(), url_path)
        } else {
            headers
                .get(&h)
                .map(|v| v.replace("\n\r", "").replace("\n", ""))
                .ok_or(NotIncludedHeader(h.to_string()))?
        };
        if let Some(array) = added_headers.get_mut(&h) {
            array.push(value);
        } else {
            added_headers.insert(h, vec![value]);
        }
    }

    let lines: Vec<String> = added_headers
        .iter()
        .map(|(k, v)| format!("{}: {}", k, v.join(", ")))
        .collect();

    let message_to_sign = lines.join("\n");
    info!("message_to_sign: {}", message_to_sign);

    Ok(SignedHeaderBuilder::default()
        .message_to_sign(message_to_sign)
        .headers(signed_headers.join(" "))
        .build()
        .unwrap())
}

fn sign_message(message: &str, key: RsaPrivateKey) -> Signature {
    SigningKey::<Sha256>::new(key).sign(message.as_bytes())
}

#[derive(Debug, Error)]
pub enum VerifyError {
    #[error("signature not found")]
    SignatureNotFound,
    #[error("signature invalid")]
    SignatureInvalid,
    #[error("signature not match")]
    SignatureNotMatch,
    #[error("key not found")]
    KeyNotFound,
    #[error("required header not found")]
    InsufficientHeader,
    #[error("server error: {0}")]
    Other(#[from] anyhow::Error),
}

fn extract_headers(signature_header: &str) -> Vec<String> {
    // Iterate over comma-separated parts of the signature string
    signature_header
        .split(',')
        .find_map(|part| {
            // Look for the part starting with 'headers='
            if part.starts_with("headers=") {
                // Extract the value within the quotes
                let headers_str = part
                    .split('=')
                    .nth(1)? // Get the second element after '='
                    .trim_matches('"');
                // Split the headers string by spaces and return
                Some(headers_str.split(' ').map(String::from).collect())
            } else {
                None // Not the headers part
            }
        })
        .unwrap_or_default() // Return an empty vector if headers are not found
}

fn extract_signature(signature_header: &str) -> Option<String> {
    signature_header.split(',').find_map(|part| {
        if part.starts_with("signature=") {
            Some(part.split_once('=')?.1.trim_matches('"').to_string())
        } else {
            None
        }
    })
}

fn extract_key_id(signature_header: &str) -> Option<String> {
    signature_header.split(',').find_map(|part| {
        if part.starts_with("keyId=") {
            Some(part.split('=').nth(1)?.trim_matches('"').to_string())
        } else {
            None
        }
    })
}

pub async fn verify_signature<F>(
    headers: HeaderMapWrapper<'_>,
    method: Method,
    url_path: &str,
    key: &mut F,
) -> Result<UserSpecifier, VerifyError>
where
    F: KeyFetcher + ?Sized,
{
    let signature = headers
        .get("signature")
        .ok_or(VerifyError::SignatureNotFound)?
        .to_string();
    debug!("signature header: {}", &signature);

    let signed_headers = extract_headers(&signature);
    let must_check_headers = {
        let mut v = vec!["(request-target)", "date", "host"];
        if method == Method::POST {
            v.push("digest");
        }
        v
    };
    let all_headers_present = must_check_headers
        .iter()
        .all(|required_header| signed_headers.contains(&required_header.to_string()));
    if !all_headers_present {
        return Err(VerifyError::InsufficientHeader);
    }

    let message = make_signed_message(headers, method, url_path, &signed_headers)
        .map_err(|_| VerifyError::InsufficientHeader)?;

    let signed_signature = extract_signature(&signature).ok_or(VerifyError::SignatureInvalid)?;
    let signed_signature_bytes = BASE64_STANDARD
        .decode(signed_signature.as_bytes())
        .map_err(|e| VerifyError::Other(e.into()))?;

    let key_id = extract_key_id(&signature).ok_or(VerifyError::KeyNotFound)?;
    let pubkey = key.fetch_pubkey(&key_id).await?;
    let verify_key = VerifyingKey::<Sha256>::new(pubkey.key);

    verify_key
        .verify(
            message.message_to_sign.as_bytes(),
            &Signature::try_from(signed_signature_bytes.as_slice()).unwrap(),
        )
        .map_err(|_| VerifyError::SignatureNotMatch)?;

    Ok(UserSpecifier::from(pubkey.user_id))
}

pub fn attach_signature(req: &mut reqwest::Request, key: SignKey) -> Result<(), NotIncludedHeader> {
    let mut signed_headers: Vec<String> = vec![
        "(request-target)".into(),
        "content-type".into(),
        "host".into(),
        "date".into(),
    ];

    let hostname = req.url().host().expect("hostname not set").to_owned();
    let date = httpdate::fmt_http_date(std::time::SystemTime::now());

    {
        let headers = req.headers_mut();
        if !headers.contains_key("host") {
            headers.insert("host", hostname.to_string().parse().unwrap());
        }
        if !headers.contains_key("date") {
            headers.insert("date", date.parse().unwrap());
        }
    }

    if req.method() == reqwest::Method::POST {
        signed_headers.push("digest".into());

        let body = req.body().expect("body not set");
        let mut hasher = Sha256::new();
        hasher.update(body.as_bytes().unwrap());
        let digest = hasher.finalize();
        let digest = format!("SHA-256={}", BASE64_STANDARD.encode(digest));
        let headers = req.headers_mut();
        headers.insert("digest", digest.parse().unwrap());
    }

    let message = make_signed_message(
        HeaderMapWrapper::from_reqwest(req.headers()),
        Method::from_reqwest(req.method()),
        req.url().path(),
        &signed_headers,
    )?;
    let signature = sign_message(&message.message_to_sign, key.private_key);
    let signature = BASE64_STANDARD.encode(signature.to_bytes());

    let signature_header = format!(
        r#"keyId="{}",algorithm="rsa-sha256",headers="{}",signature="{}""#,
        key.id, message.headers, signature
    );

    {
        let headers = req.headers_mut();
        headers.insert("signature", signature_header.parse().unwrap());
        headers.insert(
            "authorization",
            format!("Signature {}", signature_header).parse().unwrap(),
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_extract_headers_from_sig() {
        let sig = r#"keyId="https://misskey.tinax.local/users/9qov6wdgdsq50001#main-key",algorithm="rsa-sha256",headers="(request-target) date host digest",signature="cvrDo17JtNuPsQItSF0nd6/EIiTkMoZcwG4XvzaraVtzY8yGUktvzakL5h9LZOte/cwoRLVn87mUczziWu36PEEl7nCW23jOieOEAl+nv4tkYvv160ne/zcUjbRJYP6t9seb6FlfXuzRfgvxdEDmdnox0hLn+yZTiOsVWfCoSvRTroUbESxIHGQZOL7ubwN/0SnAxeP7TJpHqxs8lynY3/plpRDWekUwFu0E8ncKgSq8UqO5qPZoGSgiFJCb7zp7BlcWro7xbu6JYzHBVAn9WQC0zTal08WrvQ38U9GcTiGhy8EOl7/qAO85WAOashfPwW9GArhaXL2E/TvuLuOlcQ==""#;
        let headers = super::extract_headers(sig);
        assert_eq!(headers, vec!["(request-target)", "date", "host", "digest"]);
    }

    #[test]
    fn test_extract_sig_from_sig() {
        let sig = r#"keyId="https://misskey.tinax.local/users/9qov6wdgdsq50001#main-key",algorithm="rsa-sha256",headers="(request-target) date host digest",signature="cvrDo17JtNuPsQItSF0nd6/EIiTkMoZcwG4XvzaraVtzY8yGUktvzakL5h9LZOte/cwoRLVn87mUczziWu36PEEl7nCW23jOieOEAl+nv4tkYvv160ne/zcUjbRJYP6t9seb6FlfXuzRfgvxdEDmdnox0hLn+yZTiOsVWfCoSvRTroUbESxIHGQZOL7ubwN/0SnAxeP7TJpHqxs8lynY3/plpRDWekUwFu0E8ncKgSq8UqO5qPZoGSgiFJCb7zp7BlcWro7xbu6JYzHBVAn9WQC0zTal08WrvQ38U9GcTiGhy8EOl7/qAO85WAOashfPwW9GArhaXL2E/TvuLuOlcQ==""#;
        let signature = super::extract_signature(sig);
        assert_eq!(signature, Some("cvrDo17JtNuPsQItSF0nd6/EIiTkMoZcwG4XvzaraVtzY8yGUktvzakL5h9LZOte/cwoRLVn87mUczziWu36PEEl7nCW23jOieOEAl+nv4tkYvv160ne/zcUjbRJYP6t9seb6FlfXuzRfgvxdEDmdnox0hLn+yZTiOsVWfCoSvRTroUbESxIHGQZOL7ubwN/0SnAxeP7TJpHqxs8lynY3/plpRDWekUwFu0E8ncKgSq8UqO5qPZoGSgiFJCb7zp7BlcWro7xbu6JYzHBVAn9WQC0zTal08WrvQ38U9GcTiGhy8EOl7/qAO85WAOashfPwW9GArhaXL2E/TvuLuOlcQ==".to_string()));
    }

    #[test]
    fn test_extract_key_id_from_sig() {
        let sig = r#"keyId="https://misskey.tinax.local/users/9qov6wdgdsq50001#main-key",algorithm="rsa-sha256",headers="(request-target) date host digest",signature="cvrDo17JtNuPsQItSF0nd6/EIiTkMoZcwG4XvzaraVtzY8yGUktvzakL5h9LZOte/cwoRLVn87mUczziWu36PEEl7nCW23jOieOEAl+nv4tkYvv160ne/zcUjbRJYP6t9seb6FlfXuzRfgvxdEDmdnox0hLn+yZTiOsVWfCoSvRTroUbESxIHGQZOL7ubwN/0SnAxeP7TJpHqxs8lynY3/plpRDWekUwFu0E8ncKgSq8UqO5qPZoGSgiFJCb7zp7BlcWro7xbu6JYzHBVAn9WQC0zTal08WrvQ38U9GcTiGhy8EOl7/qAO85WAOashfPwW9GArhaXL2E/TvuLuOlcQ==""#;
        let signature = super::extract_key_id(sig);
        assert_eq!(
            signature,
            Some("https://misskey.tinax.local/users/9qov6wdgdsq50001#main-key".to_string())
        );
    }
}
