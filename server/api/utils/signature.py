import base64
import datetime
import hashlib
import logging
import urllib.parse
from collections import OrderedDict
from uuid import UUID

from Crypto.Hash import SHA256
from Crypto.PublicKey import RSA
from Crypto.Signature import pkcs1_15
from django.http import HttpRequest
from requests import PreparedRequest

from api.models import User

logger = logging.getLogger(__name__)


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
            signatured_headers.append((key_lower, value))

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


class KeyRetriever:
    def get_public_key(self, key_id: str) -> tuple[bytes, UUID] | None:
        """
        Retrieve the public key for the given key ID.

        :param key_id: The key ID.
        :return: The public key bytes and owner's uuid,
                 or None if the key ID is not found.
        """
        raise NotImplementedError


class SignatureVerificationError(Exception):
    def __init__(self, msg) -> None:
        self.msg = msg
        super().__init__(msg)


_SPECIAL_HEADERS = [
    "(request-target)",
]
_SIGN_REQUIRED_HEADERS = [
    "(request-target)",
    "host",
    "date",
    "digest",
]


def verify_signature(req: HttpRequest, keyret: KeyRetriever) -> User:
    """
    Verifies the signature of an HTTP request.

    :param res: The HTTP request object.
    :param public_key_retriever: A callable that takes a key ID
                                 and returns the corresponding public key.
    :return: Returns the user who owns the public key if the signature is valid,
             otherwise raise SignatureVerificationError
    """
    # Parse the Signature header
    signature_header = req.headers.get("Signature")
    if not signature_header:
        return False

    # Extract keyId and signature from the Signature header
    signature_parts = {
        kv[0]: kv[1].strip('"')
        for kv in [part.split("=", maxsplit=1) for part in signature_header.split(",")]
        if len(kv) == 2
    }
    algorithm = signature_parts.get("algorithm", "")
    if algorithm.lower() != "rsa-sha256":
        return False
    key_id = signature_parts.get("keyId")
    try:
        received_signature = base64.b64decode(signature_parts["signature"])
    except KeyError:
        raise SignatureVerificationError("Signature header does not contain signature")

    # Fetch the public key using the retriever
    keyret_result = keyret.get_public_key(key_id)
    if keyret_result is None:
        raise SignatureVerificationError("Key not found")
    public_key_bytes, user_id = keyret_result
    try:
        user = User.objects.get(id=user_id)
    except User.DoesNotExist:
        raise SignatureVerificationError("User not found")

    # check signature contains required headers
    required_headers = [k.lower() for k in signature_parts.get("headers", "").split()]
    for header in _SIGN_REQUIRED_HEADERS:
        if header not in required_headers:
            raise SignatureVerificationError("Missing header in signature: " + header)
    not_found_headers = set(required_headers) - set(_SPECIAL_HEADERS)

    # Reconstruct the signing string
    method = req.method
    path = urllib.parse.urlparse(req.path).path
    headers = req.headers
    signatured_headers = []
    for sig_header in required_headers:
        if sig_header in _SPECIAL_HEADERS:
            continue
        if sig_header in headers:
            signatured_headers.append((sig_header, headers[sig_header]))
            not_found_headers.discard(sig_header)
    if not_found_headers:
        raise SignatureVerificationError(
            "Missing header in request: " + ", ".join(not_found_headers)
        )

    _, signature_string = make_signature_string(signatured_headers, method, path)

    # Verify the signature
    try:
        public_key = RSA.import_key(public_key_bytes)
        verifier = pkcs1_15.new(public_key)
        message_hash = SHA256.new(signature_string.encode("utf-8"))
        verifier.verify(message_hash, received_signature)
        return user
    except (ValueError, TypeError) as e:
        raise SignatureVerificationError("Verification failed")
