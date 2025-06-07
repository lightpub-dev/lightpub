use actix_web::{body::BoxBody, http::header, HttpResponse, HttpResponseBuilder};
use chrono::{DateTime, Utc};
use handlebars::Handlebars;
use lightpub_service::{
    services::{
        id::{NoteID, NotificationID, UserID},
        note::{ContentType, VisibilityModel},
        MapToUnknown,
    },
    utils::sanitize::CleanString,
    ServiceResult,
};
use nestify::nest;
use serde::Serialize;
use url::Url;

#[derive(Debug, Clone)]
pub enum Template {
    Register(()),
    Login(()),
    NoteDetails(NoteDetails),
    Notification(()),
    PasswordChange(()),
    Profile(Profile),
    ProfileEdit(ProfileEdit),
    Search(()),
    Timeline(Timeline),
    UserList(UserList),
    TotpSetup(TotpSetup),
    TotpCompleted(()),
    TotpConfirm(()),

    PartsCreateNote(PartsCreateNote),
    PartsEditNote(PartsEditNote),
    PartsGlobal(()),
    PartsNavbar(()),
    PartsNote(PartsNote),
    PartsNotes(PartsNotes),
    PartsNotifyBase(PartsNotifyBase),
    PartsNotifyList(PartsNotifyList),
    PartsTrends(PartsTrends),
    PartsUserList(PartsUserList),

    PartsNotifyFollowRequested(NotifyFollowRequested),
    PartsNotifyFollowed(NotifyFollowed),
    PartsNotifyMentioned(NotifyMentioned),
    PartsNotifyRenoted(NotifyRenoted),
    PartsNotifyReplied(NotifyReplied),
}

impl Template {
    pub fn name(&self) -> &'static str {
        use Template::*;
        match self {
            Register(_) => "register",
            Login(_) => "login",
            NoteDetails(_) => "note_details",
            Notification(_) => "notification",
            PasswordChange(_) => "password_change",
            Profile(_) => "profile",
            ProfileEdit(_) => "profile_edit",
            Search(_) => "search",
            Timeline(_) => "timeline",
            UserList(_) => "user_list",
            TotpSetup(_) => "totp_setup",
            TotpCompleted(_) => "totp_completed",
            TotpConfirm(_) => "totp_confirm",

            PartsCreateNote(_) => "parts/create_note",
            PartsEditNote(_) => "parts/edit_note",
            PartsGlobal(_) => "parts/global",
            PartsNavbar(_) => "parts/navbar",
            PartsNote(_) => "parts/note",
            PartsNotes(_) => "parts/notes",
            PartsNotifyBase(_) => "parts/notify_base",
            PartsNotifyList(_) => "parts/notify_list",
            PartsTrends(_) => "parts/trends",
            PartsUserList(_) => "parts/user_list",

            PartsNotifyFollowRequested(_) => "parts/notify/follow_requested",
            PartsNotifyFollowed(_) => "parts/notify/followed",
            PartsNotifyMentioned(_) => "parts/notify/mentioned",
            PartsNotifyRenoted(_) => "parts/notify/renoted",
            PartsNotifyReplied(_) => "parts/notify/replied",
        }
    }

    pub fn data(&self) -> serde_json::Value {
        match self {
            Template::Register(s) => Self::process(s),
            Template::Login(s) => Self::process(s),
            Template::NoteDetails(s) => Self::process(s),
            Template::Notification(s) => Self::process(s),
            Template::PasswordChange(s) => Self::process(s),
            Template::Profile(s) => Self::process(s),
            Template::ProfileEdit(s) => Self::process(s),
            Template::Search(s) => Self::process(s),
            Template::Timeline(s) => Self::process(s),
            Template::UserList(s) => Self::process(s),
            Template::TotpSetup(s) => Self::process(s),
            Template::TotpCompleted(s) => Self::process(s),
            Template::TotpConfirm(s) => Self::process(s),

            Template::PartsCreateNote(s) => Self::process(s),
            Template::PartsEditNote(s) => Self::process(s),
            Template::PartsGlobal(s) => Self::process(s),
            Template::PartsNavbar(s) => Self::process(s),
            Template::PartsNote(s) => Self::process(s),
            Template::PartsNotes(s) => Self::process(s),
            Template::PartsNotifyBase(s) => Self::process(s),
            Template::PartsNotifyList(s) => Self::process(s),
            Template::PartsTrends(s) => Self::process(s),
            Template::PartsUserList(s) => Self::process(s),

            Template::PartsNotifyFollowRequested(s) => Self::process(s),
            Template::PartsNotifyFollowed(s) => Self::process(s),
            Template::PartsNotifyMentioned(s) => Self::process(s),
            Template::PartsNotifyRenoted(s) => Self::process(s),
            Template::PartsNotifyReplied(s) => Self::process(s),
        }
    }

    fn process(s: impl Serialize) -> serde_json::Value {
        serde_json::to_value(s).unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Authed {
    pub authed: bool,
}

impl From<bool> for Authed {
    fn from(authed: bool) -> Self {
        Self { authed }
    }
}

nest! {
    #[derive(Debug, Clone, Serialize, )]*
    #[serde(rename_all = "camelCase")]*
    pub struct NoteDetails {
        pub og: pub struct NoteOg {
            pub url: Url,
            pub base_url: Url,
        },
        pub note: PartsNoteNote,
        #[serde(flatten)]
        pub authed: Authed,
    }
}

nest! {
    #[derive(Debug, Clone, Serialize)]*
    #[serde(rename_all = "camelCase")]*
    pub struct PartsNote {
        pub renote_info: Option<pub struct RenoteInfo {
            pub user: pub struct RenoteInfoUser {
                pub nickname: String,
                pub specifier: String,
            }
        }>,
        pub note: pub struct PartsNoteNote {
            pub id: NoteID,
            pub author: pub struct NoteAuthorData {
                pub id: UserID,
                pub nickname: String,
                pub specifier: String,
            },
            pub content: Option<pub struct NoteContentData {
                pub content: CleanString,
            }>,
            pub visibility: VisibilityModel,
            pub uploads: Option<Vec<String>>,
            pub created_at: DateTime<Utc>,
            pub sensitive: bool,
            pub reply_to_id: Option<NoteID>,
            pub renote_of_id: Option<NoteID>,
            pub is_my_note: bool,

            pub reply_count: u64,
            pub renote_count: u64,
            pub like_count: u64,

            pub renotable: bool,
            pub renoted: Option<bool>,
            pub liked: bool,
            pub bookmarked: bool,

            pub view_url: Option<Url>,
        },
        #[serde(flatten)]
        pub authed: Authed,
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PartsNotes {
    pub data: Vec<PartsNote>,
    #[serde(flatten)]
    pub authed: Authed,
    pub next_url: Option<String>,
}

nest! {
    #[derive(Debug, Clone, Serialize, )]*
    #[serde(rename_all = "camelCase")]*
    pub struct PartsEditNote {
        pub note: pub struct EditNoteData {
            pub id: NoteID,
            pub content: pub struct EditNoteContentData {
                pub content: String,
                pub content_type: ContentType,
            }
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PartsCreateNote {
    pub title: String,
    pub reply_to_id: Option<NoteID>,
    #[serde(flatten)]
    pub authed: Authed,
}

nest! {
    #[derive(Debug, Clone, Serialize, )]*
    #[serde(rename_all = "camelCase")]*
    pub struct PartsTrends {
        pub data: Vec<pub struct PartsTrendData {
            pub url: String,
            pub hashtag: String,
            pub count: u64,
        }>,
    }
}

nest! {
    #[derive(Debug, Clone, Serialize, )]*
    #[serde(rename_all = "camelCase")]*
    pub struct PartsUserList {
        pub data: Vec<pub struct PartsUserListData {
            pub user: pub struct PartsUserListDataUser {
                pub id: UserID,
                pub nickname: String,
                pub specifier: String,
            },
            pub created_at: Option<DateTime<Utc>>,
        }>,
        pub next_url: Option<String>,
    }
}

nest! {
    #[derive(Debug, Clone, Serialize)]*
    #[serde(rename_all = "camelCase")]*
    pub struct Profile {
        #[serde(flatten)]
        pub authed: Authed,
        pub og: pub struct ProfileOg {
            pub url: Url,
            pub title: String,
            pub description: String,
            pub site_name: String,
            pub image: Option<Url>,
        },
        pub user: pub struct ProfileUser {
            pub basic: pub struct ProfileUserBasic {
                pub id: UserID,
                pub nickname:String,
                pub specifier: String,
                pub bio: CleanString,
            },
            pub note_count: u64,
            pub follow_count: u64,
            pub follower_count: u64,

            pub is_me: Option<bool>,
            pub is_following: Option<bool>,
            pub is_followed: Option<bool>,
            pub is_blocked: Option<bool>,
            pub is_following_requested: Option<bool>,
            pub is_followed_requested: Option<bool>,

            pub can_follow: Option<bool>,
            pub can_unfollow: Option<bool>,
            pub can_accept_follow: Option<bool>,
            pub can_refuse_follow: Option<bool>,

            pub view_url: Option<Url>,
        }
    }
}

nest! {
    #[derive(Debug, Clone, Serialize)]*
    #[serde(rename_all = "camelCase")]*
    pub struct ProfileEdit {
        pub user: pub struct ProfileEditUser {
            pub basic: pub struct ProfileEditBasic {
                pub id: UserID,
                pub nickname: String,
                pub bio: String,
            },
            pub auto_follow_accept: bool,
            pub hide_follows: bool,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Timeline {
    #[serde(flatten)]
    pub authed: Authed,
    pub timeline_url: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserList {
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PartsNotifyBase {
    pub notify: NotifyNotifyBase,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotifyNotifyBase {
    pub id: NotificationID,
    pub icon_url: Option<Url>,
    pub created_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
    pub template_name: String,
}

nest! {
    #[derive(Debug, Clone, Serialize)]*
    #[serde(rename_all = "camelCase")]*
    pub struct NotifyFollowRequested {
        pub notify: pub struct NotifyFollowRequestedBody{
            pub data: pub struct NotifyFollowRequestedBodyData {
                pub follower_url: String,
                pub follower_nickname:String,
            },
            #[serde(flatten)]
            pub base: NotifyNotifyBase,
        }
    }
}

nest! {
    #[derive(Debug, Clone, Serialize)]*
    #[serde(rename_all = "camelCase")]*
    pub struct NotifyFollowed {
        pub notify: pub struct NotifyFollowedBody{
            pub data: pub struct NotifyFollowedBodyData {
                pub follower_url: String,
                pub follower_nickname:String,
            },
            #[serde(flatten)]
            pub base: NotifyNotifyBase,
        }
    }
}

nest! {
    #[derive(Debug, Clone, Serialize)]*
    #[serde(rename_all = "camelCase")]*
    pub struct NotifyMentioned {
        pub notify: pub struct NotifyMentionedBody{
            pub data: pub struct NotifyMentionedBodyData {
                pub author_url: String,
                pub author_nickname:String,
                pub note_url: String,
            },
            #[serde(flatten)]
            pub base: NotifyNotifyBase,
        }
    }
}

nest! {
    #[derive(Debug, Clone, Serialize)]*
    #[serde(rename_all = "camelCase")]*
    pub struct NotifyRenoted {
        pub notify: pub struct NotifyRenotedBody{
            pub data: pub struct NotifyRenotedBodyData {
                pub author_url: String,
                pub author_nickname:String,
                pub renoted_note_url: String,
            },
            #[serde(flatten)]
            pub base: NotifyNotifyBase,
        }
    }
}

nest! {
    #[derive(Debug, Clone, Serialize)]*
    #[serde(rename_all = "camelCase")]*
    pub struct NotifyReplied {
        pub notify: pub struct NotifyRepliedBody{
            pub data: pub struct NotifyRepliedBodyData {
                pub author_url: String,
                pub author_nickname: String,
                pub replied_note_url: String,
                pub reply_note_url: String,
            },
            #[serde(flatten)]
            pub base: NotifyNotifyBase,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PartsNotifyList {
    pub data: Vec<PartsNotifyListEntry>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum PartsNotifyListEntry {
    FollowRequested(NotifyFollowRequestedBody),
    Followed(NotifyFollowedBody),
    Mentioned(NotifyMentionedBody),
    Renoted(NotifyRenotedBody),
    Replied(NotifyRepliedBody),
}

nest! {
    #[derive(Debug, Clone, Serialize)]*
    #[serde(rename_all = "camelCase")]*
    pub struct TotpSetup {
        pub qr_base64: Option<String>,
        pub success: bool,
    }
}

pub fn render_template(
    template: &Handlebars<'_>,
    template_target: &Template,
) -> ServiceResult<HttpResponse<BoxBody>> {
    render_template_builder(template, template_target, |_| {})
}

pub fn render_template_builder<F: Fn(&mut HttpResponseBuilder)>(
    template: &Handlebars<'_>,
    template_target: &Template,
    builder_mod: F,
) -> ServiceResult<HttpResponse<BoxBody>> {
    let name = template_target.name();
    let data = template_target.data();
    let rendered = template.render(name, &data).map_err_unknown()?;

    let mut builder = HttpResponse::Ok();
    builder.insert_header((
        header::CONTENT_TYPE,
        actix_web::http::header::ContentType::html().to_string(),
    ));
    builder_mod(&mut builder);
    Ok(builder.body(rendered))
}
