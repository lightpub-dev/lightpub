from django.test import TestCase
from ..serializers import pub
from ..serializers.pub import ActivityType
from ..views import pub as pub_view
from ..models import User
from ..requester import Requester
from pyld import jsonld
import uuid
import pytest

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
        "id": "http://misskey.tinax.local/users/9prqtbgp6qc10001",
        "inbox": "http://misskey.tinax.local/users/9prqtbgp6qc10001/inbox",
        "outbox": "http://misskey.tinax.local/users/9prqtbgp6qc10001/outbox",
        "followers": "http://misskey.tinax.local/users/9prqtbgp6qc10001/followers",
        "following": "http://misskey.tinax.local/users/9prqtbgp6qc10001/following",
        "featured": "http://misskey.tinax.local/users/9prqtbgp6qc10001/collections/featured",
        "sharedInbox": "http://misskey.tinax.local/inbox",
        "endpoints": {"sharedInbox": "http://misskey.tinax.local/inbox"},
        "url": "http://misskey.tinax.local/@misskey",
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
            "id": "http://misskey.tinax.local/users/9prqtbgp6qc10001#main-key",
            "type": "Key",
            "owner": "http://misskey.tinax.local/users/9prqtbgp6qc10001",
            "publicKeyPem": "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAwaA30rvBeqb3hq5KMfV+\nJhZWM7dRSht61tCxweVsWUwI+kLzU8hzwI7gDJb9XDqAx+hVkha7FgFohtcwtAP5\nleAVI4zxf5WAZzbhM5Fbwy6kX44eKtn7AwlSjkx2sdIlGJh42J/DAXg4cdEcdier\nu4zFjXaszEV+0ptFZUDjP8JYAcqg2U4kd1P00ztASatLVi/O85QKQqHwdj6pQf4c\nBoC9T77pSIXhciNkLuTlascPT8hy74QGpZGggpcgC7vH210ywd8vJNDALF9dkejL\ncetNj45JlF4btfUijp029LmQywttwnzWdatbCO7UIMc1LSXrywg11nTQKKIVt9b+\nIQIDAQAB\n-----END PUBLIC KEY-----\n",
        },
        "isCat": False,
    }


@pytest.fixture()
def follow_req():
    return [
        {
            "@id": "http://misskey.tinax.local/follows/9prxyzqi6qc1000n",
            "@type": ["https://www.w3.org/ns/activitystreams#Follow"],
            "https://www.w3.org/ns/activitystreams#actor": [
                {"@id": "http://misskey.tinax.local/users/9prqtbgp6qc10001"}
            ],
            "https://www.w3.org/ns/activitystreams#object": [
                {
                    "@id": "http://lightpub.tinax.local/api/users/9018aaeb-c698-4bcf-b5fd-c2feb0064c91/"
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


def test_follow_activity(follow_req):
    req = follow_req

    ser = pub.FollowSerializer(data=req, many=True)
    assert ser.is_valid()

    data = ser.validated_data[0]
    assert data["@type"] == ActivityType.FOLLOW
    assert data["actor"]["@id"] == "http://misskey.tinax.local/users/9prqtbgp6qc10001"
    assert (
        data["object"]["@id"]
        == "http://lightpub.tinax.local/api/users/9018aaeb-c698-4bcf-b5fd-c2feb0064c91/"
    )


@pytest.mark.django_db
def test_follow_activity_send_accept(
    mocker, sample_user, follow_req, remote_user_object
):
    req = follow_req
    actor = ("misskey.tinax.local", "misskey")
    user_spec = sample_user.id

    parsed = pub.FollowSerializer(data=req[0])
    assert parsed.is_valid()

    spy = mocker.spy(Requester, "send_follow_accept")
    mocker.patch.object(
        Requester, "fetch_remote_user", return_value=jsonld.expand(remote_user_object)
    )

    pub_view.process_follow_activity(parsed.validated_data)

    remote_user = User.objects.filter(host=actor[0], username=actor[1]).first()
    assert remote_user is not None, "Remote user not found"
    assert remote_user.nickname == "misskey's nickname"

    spy.assert_called_once()


def test_unfollow_activity():
    req = [
        {
            "@id": "http://misskey.tinax.local/follows/9prqtbgp6qc10001/9prroved6qc1000a/undo",
            "@type": ["https://www.w3.org/ns/activitystreams#Undo"],
            "https://www.w3.org/ns/activitystreams#actor": [
                {"@id": "http://misskey.tinax.local/users/9prqtbgp6qc10001"}
            ],
            "https://www.w3.org/ns/activitystreams#object": [
                {
                    "@id": "http://misskey.tinax.local/follows/9prqtbgp6qc10001/9prroved6qc1000a",
                    "@type": ["https://www.w3.org/ns/activitystreams#Follow"],
                    "https://www.w3.org/ns/activitystreams#actor": [
                        {"@id": "http://misskey.tinax.local/users/9prqtbgp6qc10001"}
                    ],
                    "https://www.w3.org/ns/activitystreams#object": [
                        {
                            "@id": "http://lightpub.tinax.local/api/users/9018aaeb-c698-4bcf-b5fd-c2feb0064c91/"
                        }
                    ],
                }
            ],
            "https://www.w3.org/ns/activitystreams#published": [
                {
                    "@type": "http://www.w3.org/2001/XMLSchema#dateTime",
                    "@value": "2024-02-16T08:18:03.363Z",
                }
            ],
        }
    ]

    ser = pub.UndoSerializer(data=req, many=True)
    assert ser.is_valid()

    data = ser.validated_data[0]
    assert data["@type"] == ActivityType.UNDO

    assert data["object"]["@type"] == ActivityType.FOLLOW
    assert (
        data["object"]["actor"]["@id"]
        == "http://misskey.tinax.local/users/9prqtbgp6qc10001"
    )
    assert (
        data["object"]["object"]["@id"]
        == "http://lightpub.tinax.local/api/users/9018aaeb-c698-4bcf-b5fd-c2feb0064c91/"
    )


def test_unfollow_activity_ref(mocker):
    req = [
        {
            "@id": "http://misskey.tinax.local/follows/9prqtbgp6qc10001/9prroved6qc1000a/undo",
            "@type": ["https://www.w3.org/ns/activitystreams#Undo"],
            "https://www.w3.org/ns/activitystreams#actor": [
                {"@id": "http://misskey.tinax.local/users/9prqtbgp6qc10001"}
            ],
            "https://www.w3.org/ns/activitystreams#object": [
                {
                    "@id": "http://misskey.tinax.local/follows/9prqtbgp6qc10001/9prroved6qc1000a"
                }
            ],
            "https://www.w3.org/ns/activitystreams#published": [
                {
                    "@type": "http://www.w3.org/2001/XMLSchema#dateTime",
                    "@value": "2024-02-16T08:18:03.363Z",
                }
            ],
        }
    ]

    mocker.patch.object(
        Requester,
        "fetch_remote_id",
        return_value=jsonld.expand(
            {
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
                "id": "http://misskey.tinax.local/follows/9prqtbgp6qc10001/9prroved6qc1000a",
                "type": "Follow",
                "actor": "http://misskey.tinax.local/users/9prqtbgp6qc10001",
                "object": "http://lightpub.tinax.local/api/users/9018aaeb-c698-4bcf-b5fd-c2feb0064c91/",
            }
        ),
    )

    ser = pub.UndoSerializer(data=req, many=True)
    assert ser.is_valid(), ser.errors

    data = ser.validated_data[0]
    assert data["@type"] == ActivityType.UNDO

    assert data["object"]["@type"] == ActivityType.FOLLOW
    assert (
        data["object"]["actor"]["@id"]
        == "http://misskey.tinax.local/users/9prqtbgp6qc10001"
    )
    assert (
        data["object"]["object"]["@id"]
        == "http://lightpub.tinax.local/api/users/9018aaeb-c698-4bcf-b5fd-c2feb0064c91/"
    )
