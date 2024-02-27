import base64
import hashlib
import json

import pytest

from . import signature


@pytest.fixture()
def sample_body():
    return json.dumps({"id": "aaaa", "key": "value", "foo": "bar"}).encode("utf-8")


@pytest.fixture()
def sample_body_sha256(sample_body):
    return hashlib.sha256(sample_body).digest()


@pytest.fixture()
def sample_headers():
    return [
        ("Date", "Mon, 23 Dec 2019 10:00:00\n GMT"),
        ("Digest", "SHA-256=abcdefg"),
        (
            "Cache-Control",
            "max-age=60",
        ),
        ("Cache-control", "must-revalidate"),
    ]


@pytest.fixture()
def sample_private_key():
    return """-----BEGIN PRIVATE KEY-----
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
+XpK00Z7bJWNSER3b8oNtNxH0fO9mA==
-----END PRIVATE KEY-----
""".encode(
        "utf-8"
    )


@pytest.fixture()
def sample_txt_data():
    return "sample".encode("utf-8")


@pytest.fixture()
def sample_txt_data_sig_base64():
    return "VxOyi1IB8rgkYboNssphfGDnwNGeWUTsqIPkzjCH/5i+lRUTWoD/tGPOPUFpRYTtdhfUgFLL6yinplSf030i0NC0iITe6EiWe76NLhhQDYEEwh/5/OwqXpKLogip/yXjynnO0WIBqorxSTnpesppbKM1BP/yVNDTVW7OxTjK9NnJSLII0QjxnNdENH8JmJOHCU/4SOC8MhHYsx7dTQfSHKklRkonoQtIBXwg4ba//2OZ8PVXd7/tkURtliNLVwDt2Ufq4h4fHm0pCer7aLSeEkNIHyAzpmjtubAtu+StMoxLqrFHOnaZSyGLpA4Aatt7ihWFZCEAiBwcj4ateCt/oV3H0JGPOIFkkHjsDV0Hdi+M8QXUlBhcgx14DqXZtHGvcH2iPESW/JEK3HowUWX7wSFuZsNhKKjULk2m7ytxpXGSgn04+18nkmCWXche6wcjST5roVG1ya9amec5TMc7AEl+wYFC5nHns53b3FPiVzQclkD2pTeaSvY7gJr7MUTDRasQONvA6RbsoMcl15x7QII2hvvar37+fGeUKgCS/6+iHty7/jY+iRgtEbDORIrZvLDj8J1eeOkbXj9NMrCRFBj4PDLBWcX8aHmy4lv4yA0nMFVMbIwH6nzRc6sZ5BsVYCEn8/18NcINqCeaMedYuGNK+Wg1H0btsLO3o+BuAw0="


def test_digest_body(sample_body, sample_body_sha256):
    digest = signature.digest_body(sample_body)
    assert digest == sample_body_sha256


def test_signature_string(sample_headers):
    method = "POST"
    path = "/api/register/"

    _, signature_string = signature.make_signature_string(sample_headers, method, path)

    assert signature_string == (
        "(request-target): post /api/register/\n"
        "date: Mon, 23 Dec 2019 10:00:00 GMT\n"
        "digest: SHA-256=abcdefg\n"
        "cache-control: max-age=60, must-revalidate"
    )


def test_sign_message(sample_private_key, sample_txt_data, sample_txt_data_sig_base64):
    signed = signature.sign_message(sample_private_key, sample_txt_data)
    assert base64.b64encode(signed).decode("utf-8") == sample_txt_data_sig_base64
