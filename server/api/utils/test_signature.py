from Crypto.PublicKey import RSA
from . import signature
import base64
import pytest
import json
import hashlib


@pytest.fixture()
def sample_body():
    return json.dumps({"id": "aaaa", "key": "value", "foo": "bar"}).encode("utf-8")


@pytest.fixture()
def sample_body_sha256(sample_body):
    return hashlib.sha256(sample_body).digest()


def test_digest_body(sample_body, sample_body_sha256):
    digest = signature.digest_body(sample_body)
    assert digest == sample_body_sha256


def test_signature_string():
    sample_headers = [
        ("Date", "Mon, 23 Dec 2019 10:00:00\n GMT"),
        ("Digest", "SHA-256=abcdefg"),
        (
            "Cache-Control",
            "max-age=60",
        ),
        ("Cache-control", "must-revalidate"),
    ]
    method = "POST"
    path = "/api/register/"

    signature_string = signature.make_signature_string(sample_headers, method, path)

    assert signature_string == (
        "(request-target): post /api/register/\n"
        "date: Mon, 23 Dec 2019 10:00:00 GMT\n"
        "digest: SHA-256=abcdefg\n"
        "cache-control: max-age=60, must-revalidate"
    )
