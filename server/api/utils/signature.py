import hashlib
from collections import OrderedDict
from Crypto.PublicKey import RSA
from Crypto.Signature import pkcs1_15
from Crypto.Hash import SHA256


def digest_body(body: bytes) -> bytes:
    return hashlib.sha256(body).digest()


def make_signature_string(
    headers: list[tuple[str, str]], method: str, path: str
) -> str:
    request_target = f"(request-target): {method.lower()} {path}"

    lines = [request_target]

    added_headers = OrderedDict()
    for key, value in headers:
        key_lower = key.lower()
        value_strip = value.strip().replace("\n\r", "").replace("\n", "")

        if key_lower in added_headers:
            added_headers[key_lower].append(value_strip)
        else:
            added_headers[key_lower] = [value_strip]

    for key, values in added_headers.items():
        lines.append(f"{key}: {', '.join(values)}")

    return "\n".join(lines)


def sign_message(private_key: bytes, message: bytes) -> bytes:
    key = RSA.import_key(private_key)
    signer = pkcs1_15.new(key)
    hash = SHA256.new(message)
    signed = signer.sign(hash)
    return signed
