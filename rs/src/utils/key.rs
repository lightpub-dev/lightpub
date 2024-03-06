use base64::prelude::*;
use derive_builder::Builder;
use linked_hash_map::LinkedHashMap;
use reqwest::Method;
use rsa::pkcs1v15::Signature;
use rsa::signature::SignatureEncoding;
use rsa::signature::Signer;
use rsa::{pkcs1v15::SigningKey, sha2::Sha256, RsaPrivateKey};
use tracing::info;

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

#[derive(Debug)]
pub struct NotIncludedHeader(String);
#[derive(Debug, Clone, Builder)]
struct SignedHeader {
    message_to_sign: String,
    headers: String,
}

fn make_signed_message(
    req: &reqwest::Request,
    signed_headers: &[&str],
) -> Result<SignedHeader, NotIncludedHeader> {
    let headers = req.headers();
    let mut added_headers: LinkedHashMap<String, Vec<String>> = LinkedHashMap::new();
    for h in signed_headers {
        let h = h.to_lowercase();
        let value = if h == "(request-target)" {
            format!(
                "{} {}",
                req.method().as_str().to_lowercase(),
                req.url().path()
            )
        } else {
            headers
                .get(&h)
                .map(|v| v.to_str().unwrap().replace("\n\r", "").replace("\n", ""))
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

pub fn attach_signature(req: &mut reqwest::Request, key: SignKey) -> Result<(), NotIncludedHeader> {
    let mut signed_headers = vec!["(request-target)", "content-type", "host", "date"];

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

    if req.method() == Method::POST {
        signed_headers.push("digest");

        let body = req.body().expect("body not set");
        let digest = format!(
            "SHA-256={}",
            BASE64_STANDARD.encode(sha256::digest(body.as_bytes().unwrap()))
        );
        let headers = req.headers_mut();
        headers.insert("digest", digest.parse().unwrap());
    }

    let message = make_signed_message(req, &signed_headers)?;
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
