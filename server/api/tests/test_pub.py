import uuid
from datetime import datetime

import pytest
from pyld import jsonld

from ..models import User
from ..requester import Requester
from ..serializers import pub
from ..views import pub as pub_view

# Create your tests here.


@pytest.fixture()
def remote_user_object():
    return {
        "@context": [
            "https://www.w3.org/ns/activitystreams",
            "https://w3id.org/security/v1",
            {
                "manuallyApprovesFollowers": "as:manuallyApprovesFollowers",
                "sensitive": "as:sensitive",
                "Hashtag": "as:Hashtag",
                "quoteUrl": "as:quoteUrl",
                "toot": "http://joinmastodon.org/ns#",
                "Emoji": "toot:Emoji",
                "featured": "toot:featured",
                "discoverable": "toot:discoverable",
                "schema": "http://schema.org#",
                "PropertyValue": "schema:PropertyValue",
                "value": "schema:value",
                "misskey": "https://misskey-hub.net/ns#",
                "_misskey_content": "misskey:_misskey_content",
                "_misskey_quote": "misskey:_misskey_quote",
                "_misskey_reaction": "misskey:_misskey_reaction",
                "_misskey_votes": "misskey:_misskey_votes",
                "_misskey_summary": "misskey:_misskey_summary",
                "isCat": "misskey:isCat",
                "vcard": "http://www.w3.org/2006/vcard/ns#",
            },
        ],
        "type": "Person",
        "id": "https://misskey.tinax.local/users/9prqtbgp6qc10001",
        "inbox": "https://misskey.tinax.local/users/9prqtbgp6qc10001/inbox",
        "outbox": "https://misskey.tinax.local/users/9prqtbgp6qc10001/outbox",
        "followers": "https://misskey.tinax.local/users/9prqtbgp6qc10001/followers",
        "following": "https://misskey.tinax.local/users/9prqtbgp6qc10001/following",
        "featured": "https://misskey.tinax.local/users/9prqtbgp6qc10001/collections/featured",
        "sharedInbox": "https://misskey.tinax.local/inbox",
        "endpoints": {"sharedInbox": "https://misskey.tinax.local/inbox"},
        "url": "https://misskey.tinax.local/@misskey",
        "preferredUsername": "misskey",
        "name": "misskey's nickname",
        "summary": None,
        "_misskey_summary": None,
        "icon": None,
        "image": None,
        "tag": [],
        "manuallyApprovesFollowers": False,
        "discoverable": True,
        "publicKey": {
            "id": "https://misskey.tinax.local/users/9prqtbgp6qc10001#main-key",
            "type": "Key",
            "owner": "https://misskey.tinax.local/users/9prqtbgp6qc10001",
            "publicKeyPem": "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAwaA30rvBeqb3hq5KMfV+\nJhZWM7dRSht61tCxweVsWUwI+kLzU8hzwI7gDJb9XDqAx+hVkha7FgFohtcwtAP5\nleAVI4zxf5WAZzbhM5Fbwy6kX44eKtn7AwlSjkx2sdIlGJh42J/DAXg4cdEcdier\nu4zFjXaszEV+0ptFZUDjP8JYAcqg2U4kd1P00ztASatLVi/O85QKQqHwdj6pQf4c\nBoC9T77pSIXhciNkLuTlascPT8hy74QGpZGggpcgC7vH210ywd8vJNDALF9dkejL\ncetNj45JlF4btfUijp029LmQywttwnzWdatbCO7UIMc1LSXrywg11nTQKKIVt9b+\nIQIDAQAB\n-----END PUBLIC KEY-----\n",
        },
        "isCat": False,
    }


@pytest.fixture()
def follow_req():
    return [
        {
            "@id": "https://misskey.tinax.local/follows/9prxyzqi6qc1000n",
            "@type": ["https://www.w3.org/ns/activitystreams#Follow"],
            "https://www.w3.org/ns/activitystreams#actor": [
                {"@id": "https://misskey.tinax.local/users/9prqtbgp6qc10001"}
            ],
            "https://www.w3.org/ns/activitystreams#object": [
                {
                    "@id": "https://lightpub.tinax.local/api/users/9018aaeb-c698-4bcf-b5fd-c2feb0064c91"
                }
            ],
        }
    ]


@pytest.fixture()
def delete_req():
    return [
        {
            "@id": "https://misskey.tinax.local/a54921b2-c2f4-4561-aefc-e9bc72108f69",
            "@type": ["https://www.w3.org/ns/activitystreams#Delete"],
            "https://www.w3.org/ns/activitystreams#actor": [
                {"@id": "https://misskey.tinax.local/users/9q4qi2u1avoq0001"}
            ],
            "https://www.w3.org/ns/activitystreams#object": [
                {
                    "@id": "https://misskey.tinax.local/notes/9q6akv3nldg6001l",
                    "@type": ["https://www.w3.org/ns/activitystreams#Tombstone"],
                }
            ],
            "https://www.w3.org/ns/activitystreams#published": [
                {
                    "@type": "http://www.w3.org/2001/XMLSchema#dateTime",
                    "@value": "2024-02-26T09:22:31.673Z",
                }
            ],
        }
    ]


@pytest.fixture()
def sample_user():
    user = User.objects.create(
        id=uuid.UUID("9018aaeb-c698-4bcf-b5fd-c2feb0064c91"),
        username="tinaxd",
        host=None,
        bpassword="testtest",
        nickname="tinax",
    )
    yield user
    user.delete()


@pytest.mark.skip
@pytest.mark.django_db
def test_follow_activity_send_accept(
    mocker, sample_user, follow_req, remote_user_object
):
    req = follow_req
    actor = ("misskey.tinax.local", "misskey")
    user_spec = sample_user.id

    parsed = pub.FollowActivity.from_dict(req[0])

    spy = mocker.spy(Requester, "send_follow_accept")
    mocker.patch.object(
        Requester, "fetch_remote_user", return_value=jsonld.expand(remote_user_object)
    )

    pub_view.process_follow_activity(parsed)

    remote_user = User.objects.filter(host=actor[0], username=actor[1]).first()
    assert remote_user is not None, "Remote user not found"
    assert remote_user.nickname == "misskey's nickname"

    spy.assert_called_once()


def test_delete_activity(delete_req):
    parsed = pub.DeleteActivity.from_dict(delete_req[0])
    assert parsed.as_actor.id == "https://misskey.tinax.local/users/9q4qi2u1avoq0001"
    assert parsed.as_object.id == "https://misskey.tinax.local/notes/9q6akv3nldg6001l"
    assert pub.is_tombstone(parsed.as_object)
    assert parsed.as_published.as_datetime() == datetime.fromisoformat(
        "2024-02-26T09:22:31.673Z"
    )
