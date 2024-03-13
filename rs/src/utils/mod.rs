use uuid::{NoContext, Uuid};

pub mod key;
pub mod post;
pub mod user;

pub fn generate_uuid() -> uuid::fmt::Simple {
    let ts = uuid::Timestamp::now(NoContext);
    Uuid::new_v7(ts).simple()
}

pub fn generate_uuid_random() -> uuid::fmt::Simple {
    Uuid::new_v4().simple()
}

pub fn uuid_to_string(uuid: &Uuid) -> String {
    let mut buf = [0u8; 36];
    let s = uuid.simple().encode_lower(&mut buf);
    s.to_owned()
}

pub mod apub_hashtag {
    use activitystreams::Base;
    use activitystreams::BaseBox;
    use activitystreams::UnitString;
    use activitystreams::{properties, PropRefs};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Default, UnitString)]
    #[unit_string(Hashtag)]
    pub struct HashtagKind;

    properties! {
        Hashtag {
            name {
                types [String],
                functional,
                required
            }
        }
    }

    #[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
    #[serde(rename_all = "camelCase")]
    pub struct HashtagObject {
        pub kind: HashtagKind,

        #[prop_refs]
        pub hashtag_properties: HashtagProperties,
    }
}

pub mod apub_key {
    use activitystreams::{actor::Actor, ext::Extension};
    use derive_builder::Builder;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, Builder, Default)]
    #[serde(rename_all = "camelCase")]
    pub struct PublicKey {
        id: String,
        owner: String,
        public_key_pem: String,
        #[builder(default = "\"Key\".to_string()")]
        r#type: String,
    }

    impl PublicKey {
        pub fn into_ext(self) -> PublicKeyExtension {
            PublicKeyExtension { public_key: self }
        }
    }

    impl<T> Extension<T> for PublicKeyExtension where T: Actor {}

    #[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct PublicKeyExtension {
        pub public_key: PublicKey,
    }
}
