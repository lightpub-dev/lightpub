from django.test import TestCase
from ..serializers import pub
from ..serializers.pub import ActivityType

# Create your tests here.


class PubTestCase(TestCase):
    def test_follow_activity(self):
        req = [
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

        ser = pub.FollowSerializer(data=req, many=True)
        self.assertTrue(ser.is_valid(), ser.errors)

        data = ser.validated_data[0]
        self.assertEqual(data["@type"], ActivityType.FOLLOW)
        self.assertEqual(
            data["actor"]["@id"], "http://misskey.tinax.local/users/9prqtbgp6qc10001"
        )
        self.assertEqual(
            data["object"]["@id"],
            "http://lightpub.tinax.local/api/users/9018aaeb-c698-4bcf-b5fd-c2feb0064c91/",
        )

    def test_unfollow_activity(self):
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
        self.assertTrue(ser.is_valid(), ser.errors)

        data = ser.validated_data[0]
        self.assertEqual(data["@type"], ActivityType.UNDO)

        self.assertEqual(data["object"]["@type"], ActivityType.FOLLOW)
        self.assertEqual(
            data["object"]["actor"]["@id"],
            "http://misskey.tinax.local/users/9prqtbgp6qc10001",
        )
        self.assertEqual(
            data["object"]["object"]["@id"],
            "http://lightpub.tinax.local/api/users/9018aaeb-c698-4bcf-b5fd-c2feb0064c91/",
        )
