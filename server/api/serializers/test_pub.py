import pytest
from pyld import jsonld

from . import pub


@pytest.fixture()
def sample_follow_req():
    return jsonld.expand(
        {
            "@context": "https://www.w3.org/ns/activitystreams/",
            "id": "http://misskey.tinax.local/follows/9prswext6qc1000g",
            "type": "Follow",
            "actor": "http://misskey.tinax.local/users/9prqtbgp6qc10001",
            "object": "http://lightpub.tinax.local/api/users/9018aaeb-c698-4bcf-b5fd-c2feb0064c91/",
        }
    )


def test_follow_activity(sample_follow_req):
    obj = pub.Object.from_dict(sample_follow_req[0])

    assert pub.is_follow(obj)


def test_is_public():
    obj = pub.Object.from_dict({"@id": "https://www.w3.org/ns/activitystreams#Public"})

    assert pub.is_public(obj)
