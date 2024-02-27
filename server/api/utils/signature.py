import base64
import datetime
import hashlib
import urllib.parse
from collections import OrderedDict

from Crypto.Hash import SHA256
from Crypto.PublicKey import RSA
from Crypto.Signature import pkcs1_15
from requests import PreparedRequest


def digest_body(body: bytes) -> bytes:
    return hashlib.sha256(body).digest()


def make_signature_string(
    headers: list[tuple[str, str]], method: str, path: str
) -> tuple[str, str]:
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

    headers_line = " ".join(["(request-target)"] + [key for key, _ in headers])

    return headers_line, "\n".join(lines)


def sign_message(private_key: bytes, message: bytes) -> bytes:
    key = RSA.import_key(private_key)
    signer = pkcs1_15.new(key)
    hash = SHA256.new(message)
    signed = signer.sign(hash)
    return signed


def attach_signature(req: PreparedRequest, key_id: str, priv_key: bytes) -> None:
    included_headers = set(["host", "date", "digest", "content-type"])

    url_parsed = urllib.parse.urlparse(req.url)

    if "Host" not in req.headers:
        req.headers["Host"] = url_parsed.hostname

    if "Date" not in req.headers:
        req.headers["Date"] = datetime.datetime.utcnow().strftime(
            "%a, %d %b %Y %H:%M:%S GMT"
        )

    # add digest header if this is a POST request
    if req.method == "POST":
        # to ensure Digest is appended at the end
        # (order is important in HTTP signature)
        req.headers.pop("Digest", None)

        body = req.body
        if body is None:
            body = b""
        digest = digest_body(body)
        req.headers["Digest"] = f"SHA-256={base64.b64encode(digest).decode('utf-8')}"

    headers = req.headers.items()

    signatured_headers = []
    for key, value in headers:
        key_lower = key.lower()
        if key_lower in included_headers:
            signatured_headers.append((key, value))

    path = url_parsed.path
    headers_line, signature_string = make_signature_string(
        signatured_headers, req.method, path
    )

    signed_signature = base64.b64encode(
        sign_message(priv_key, signature_string.encode("utf-8"))
    ).decode("utf-8")

    signature = (
        f'keyId="{key_id}",algorithm="rsa-sha256",'
        f'headers="{headers_line}",signature="{signed_signature}"'
    )

    req.headers["Signature"] = signature
    # for Misskey
    req.headers["Authorization"] = f"Signature {signature}"
