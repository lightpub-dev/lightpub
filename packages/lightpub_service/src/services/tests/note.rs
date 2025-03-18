use crate::services::{
    ServiceResult, UpsertOperation,
    id::{NoteID, UserID},
    note::{
        ContentType, NoteContentModel, PostCreateOptions, PostCreateOptionsBuilder,
        VisibilityModel, create_note, get_note_by_id_visibility_check, get_user_notes,
    },
    tests::{
        auth::register_sample_user,
        common::{BASE_URL, MY_DOMAIN, test_setup},
        user::user_follow_for_test,
    },
};

use super::{auth::register_user_for_test, common::TestState};

pub async fn create_note_for_test(
    st: &TestState,
    author_id: UserID,
    content: &str,
    content_type: ContentType,
    visibility: VisibilityModel,
    options: &PostCreateOptions,
) -> ServiceResult<NoteID> {
    let app = &st.app;
    let conn = app.conn();
    let rconn = app.rconn();
    let qconn = app.qconn();
    let fed = app.fed();
    let data = fed.to_request_data();
    create_note(
        conn,
        &rconn,
        qconn,
        app.ft(),
        author_id,
        content,
        content_type,
        visibility,
        options,
        MY_DOMAIN,
        &BASE_URL,
        &data,
    )
    .await
}

#[tokio::test]
async fn test_create_note_public() {
    let st = test_setup().await;

    let user_id = register_sample_user(&st).await;

    let note_id = create_note_for_test(
        &st,
        user_id,
        "content",
        ContentType::Plain,
        VisibilityModel::Public,
        &PostCreateOptionsBuilder::default()
            .sensitive(UpsertOperation::Set(true))
            .build()
            .unwrap(),
    )
    .await
    .unwrap();

    let get_note = get_note_by_id_visibility_check(
        &st.app.maybe_conn(),
        &st.app.rconn(),
        note_id,
        None,
        false,
    )
    .await
    .unwrap(); // anonymous viewer

    assert!(get_note.is_some());
    let get_note = get_note.unwrap();
    assert_eq!(
        get_note.basic.content.unwrap(),
        NoteContentModel::Plain("content".to_string())
    );
    assert_eq!(get_note.basic.sensitive, true);
}

#[tokio::test]
async fn test_create_note_unlisted() {
    let st = test_setup().await;

    let user_id = register_sample_user(&st).await;

    let note_id = create_note_for_test(
        &st,
        user_id,
        "content",
        ContentType::Plain,
        VisibilityModel::Unlisted,
        &PostCreateOptionsBuilder::default()
            .sensitive(UpsertOperation::Set(true))
            .build()
            .unwrap(),
    )
    .await
    .unwrap();

    let get_note = get_note_by_id_visibility_check(
        &st.app.maybe_conn(),
        &st.app.rconn(),
        note_id,
        None,
        false,
    )
    .await
    .unwrap(); // anonymous viewer

    assert!(get_note.is_some());
    let get_note = get_note.unwrap();
    assert_eq!(
        get_note.basic.content.unwrap(),
        NoteContentModel::Plain("content".to_string())
    );
    assert_eq!(get_note.basic.sensitive, true);
}

#[tokio::test]
async fn test_create_note_follower_anon() {
    let st = test_setup().await;

    let user_id = register_sample_user(&st).await;

    let note_id = create_note_for_test(
        &st,
        user_id,
        "content",
        ContentType::Plain,
        VisibilityModel::Follower,
        &PostCreateOptionsBuilder::default().build().unwrap(),
    )
    .await
    .unwrap();

    let get_note = get_note_by_id_visibility_check(
        &st.app.maybe_conn(),
        &st.app.rconn(),
        note_id,
        None,
        false,
    )
    .await
    .unwrap(); // anonymous viewer

    assert!(get_note.is_none());
}

#[tokio::test]
async fn test_create_note_follower_self() {
    let st = test_setup().await;

    let user_id = register_sample_user(&st).await;

    let note_id = create_note_for_test(
        &st,
        user_id,
        "content",
        ContentType::Plain,
        VisibilityModel::Follower,
        &PostCreateOptionsBuilder::default().build().unwrap(),
    )
    .await
    .unwrap();

    let get_note = get_note_by_id_visibility_check(
        &st.app.maybe_conn(),
        &st.app.rconn(),
        note_id,
        Some(user_id),
        false,
    )
    .await
    .unwrap(); // anonymous viewer

    assert!(get_note.is_some());
}

#[tokio::test]
async fn test_create_note_follower_follower() {
    let st = test_setup().await;

    let user_id = register_sample_user(&st).await;
    let following_user = register_user_for_test(&st, "followyou").await;

    let note_id = create_note_for_test(
        &st,
        user_id,
        "content",
        ContentType::Plain,
        VisibilityModel::Follower,
        &PostCreateOptionsBuilder::default().build().unwrap(),
    )
    .await
    .unwrap();

    user_follow_for_test(&st, following_user, user_id, true).await;

    let get_note = get_note_by_id_visibility_check(
        &st.app.maybe_conn(),
        &st.app.rconn(),
        note_id,
        Some(following_user),
        false,
    )
    .await
    .unwrap(); // anonymous viewer

    assert!(get_note.is_some());
}

#[tokio::test]
async fn test_create_note_private_anon() {
    let st = test_setup().await;

    let user_id = register_sample_user(&st).await;

    let note_id = create_note_for_test(
        &st,
        user_id,
        "content",
        ContentType::Plain,
        VisibilityModel::Private,
        &PostCreateOptionsBuilder::default().build().unwrap(),
    )
    .await
    .unwrap();

    let get_note = get_note_by_id_visibility_check(
        &st.app.maybe_conn(),
        &st.app.rconn(),
        note_id,
        None,
        false,
    )
    .await
    .unwrap(); // anonymous viewer

    assert!(get_note.is_none());
}

#[tokio::test]
async fn test_create_note_private_self() {
    let st = test_setup().await;

    let user_id = register_sample_user(&st).await;

    let note_id = create_note_for_test(
        &st,
        user_id,
        "content",
        ContentType::Plain,
        VisibilityModel::Private,
        &PostCreateOptionsBuilder::default().build().unwrap(),
    )
    .await
    .unwrap();

    let get_note = get_note_by_id_visibility_check(
        &st.app.maybe_conn(),
        &st.app.rconn(),
        note_id,
        Some(user_id),
        false,
    )
    .await
    .unwrap(); // anonymous viewer

    assert!(get_note.is_some());
}

#[tokio::test]
async fn test_create_note_private_mentioned() {
    let st = test_setup().await;

    let user_id = register_sample_user(&st).await;
    let mentioned_user = register_user_for_test(&st, "mentionme").await;

    let note_id = create_note_for_test(
        &st,
        user_id,
        "content @mentionme",
        ContentType::Plain,
        VisibilityModel::Private,
        &PostCreateOptionsBuilder::default().build().unwrap(),
    )
    .await
    .unwrap();

    let get_note = get_note_by_id_visibility_check(
        &st.app.maybe_conn(),
        &st.app.rconn(),
        note_id,
        Some(mentioned_user),
        false,
    )
    .await
    .unwrap(); // anonymous viewer

    assert!(get_note.is_some());
}

#[tokio::test]
async fn test_reply_to_public_note() {
    let st = test_setup().await;

    let user1 = register_user_for_test(&st, "user1").await;
    let user2 = register_user_for_test(&st, "user2").await;

    let note_id = create_note_for_test(
        &st,
        user1,
        "content",
        ContentType::Plain,
        VisibilityModel::Public,
        &PostCreateOptionsBuilder::default().build().unwrap(),
    )
    .await
    .unwrap();

    create_note_for_test(
        &st,
        user2,
        "reply",
        ContentType::Plain,
        VisibilityModel::Public,
        &PostCreateOptionsBuilder::default()
            .reply_to_id(UpsertOperation::Set(Some(note_id)))
            .build()
            .unwrap(),
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn test_reply_to_unlisted_note() {
    let st = test_setup().await;

    let user1 = register_user_for_test(&st, "user1").await;
    let user2 = register_user_for_test(&st, "user2").await;

    let note_id = create_note_for_test(
        &st,
        user1,
        "content",
        ContentType::Plain,
        VisibilityModel::Unlisted,
        &PostCreateOptionsBuilder::default().build().unwrap(),
    )
    .await
    .unwrap();

    create_note_for_test(
        &st,
        user2,
        "reply",
        ContentType::Plain,
        VisibilityModel::Public,
        &PostCreateOptionsBuilder::default()
            .reply_to_id(UpsertOperation::Set(Some(note_id)))
            .build()
            .unwrap(),
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn test_reply_to_follower_note() {
    let st = test_setup().await;

    let user1 = register_user_for_test(&st, "user1").await;
    let user2 = register_user_for_test(&st, "user2").await;

    let note_id = create_note_for_test(
        &st,
        user1,
        "content",
        ContentType::Plain,
        VisibilityModel::Follower,
        &PostCreateOptionsBuilder::default().build().unwrap(),
    )
    .await
    .unwrap();

    create_note_for_test(
        &st,
        user2,
        "reply",
        ContentType::Plain,
        VisibilityModel::Public,
        &PostCreateOptionsBuilder::default()
            .reply_to_id(UpsertOperation::Set(Some(note_id)))
            .build()
            .unwrap(),
    )
    .await
    .unwrap_err();
}

#[tokio::test]
async fn test_reply_to_private_note() {
    let st = test_setup().await;

    let user1 = register_user_for_test(&st, "user1").await;
    let user2 = register_user_for_test(&st, "user2").await;

    let note_id = create_note_for_test(
        &st,
        user1,
        "content",
        ContentType::Plain,
        VisibilityModel::Private,
        &PostCreateOptionsBuilder::default().build().unwrap(),
    )
    .await
    .unwrap();

    create_note_for_test(
        &st,
        user2,
        "reply",
        ContentType::Plain,
        VisibilityModel::Public,
        &PostCreateOptionsBuilder::default()
            .reply_to_id(UpsertOperation::Set(Some(note_id)))
            .build()
            .unwrap(),
    )
    .await
    .unwrap_err();
}

#[tokio::test]
async fn test_reply_to_mentioned_private_note() {
    let st = test_setup().await;

    let user1 = register_user_for_test(&st, "user1").await;
    let user2 = register_user_for_test(&st, "user2").await;

    let note_id = create_note_for_test(
        &st,
        user1,
        "content @user2",
        ContentType::Plain,
        VisibilityModel::Private,
        &PostCreateOptionsBuilder::default().build().unwrap(),
    )
    .await
    .unwrap();

    create_note_for_test(
        &st,
        user2,
        "reply",
        ContentType::Plain,
        VisibilityModel::Private,
        &PostCreateOptionsBuilder::default()
            .reply_to_id(UpsertOperation::Set(Some(note_id)))
            .build()
            .unwrap(),
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn test_reply_to_mentioned_private_note_publicly() {
    let st = test_setup().await;

    let user1 = register_user_for_test(&st, "user1").await;
    let user2 = register_user_for_test(&st, "user2").await;

    let note_id = create_note_for_test(
        &st,
        user1,
        "content @user2",
        ContentType::Plain,
        VisibilityModel::Private,
        &PostCreateOptionsBuilder::default().build().unwrap(),
    )
    .await
    .unwrap();

    create_note_for_test(
        &st,
        user2,
        "reply",
        ContentType::Plain,
        VisibilityModel::Public,
        &PostCreateOptionsBuilder::default()
            .reply_to_id(UpsertOperation::Set(Some(note_id)))
            .build()
            .unwrap(),
    )
    .await
    .unwrap_err();
}

struct UserNotesTestFixture {
    user1: UserID,
    note_public: NoteID,
    note_unlisted: NoteID,
    note_follower: NoteID,
    note_private: NoteID,
}

async fn setup_user_notes_fixture(st: &TestState) -> UserNotesTestFixture {
    let user1 = register_user_for_test(&st, "user1").await;

    let opts = PostCreateOptionsBuilder::default().build().unwrap();
    let note1 = create_note_for_test(
        st,
        user1,
        "content1",
        ContentType::Plain,
        VisibilityModel::Public,
        &opts,
    )
    .await
    .unwrap();
    let note2 = create_note_for_test(
        st,
        user1,
        "content2",
        ContentType::Plain,
        VisibilityModel::Unlisted,
        &opts,
    )
    .await
    .unwrap();
    let note3 = create_note_for_test(
        st,
        user1,
        "content3",
        ContentType::Plain,
        VisibilityModel::Follower,
        &opts,
    )
    .await
    .unwrap();
    let note4 = create_note_for_test(
        st,
        user1,
        "@mentionme content4",
        ContentType::Plain,
        VisibilityModel::Private,
        &opts,
    )
    .await
    .unwrap();

    UserNotesTestFixture {
        user1,
        note_public: note1,
        note_unlisted: note2,
        note_follower: note3,
        note_private: note4,
    }
}

#[tokio::test]
async fn test_user_notes_self_viewer() {
    let st = test_setup().await;

    let fix = setup_user_notes_fixture(&st).await;

    let notes = get_user_notes(
        &st.app.maybe_conn(),
        &st.app.rconn(),
        Some(fix.user1),
        fix.user1,
        20,
        None,
    )
    .await
    .unwrap();
    assert_eq!(notes.len(), 4);
    assert_eq!(notes[0].basic.id, fix.note_private);
    assert_eq!(notes[1].basic.id, fix.note_follower);
    assert_eq!(notes[2].basic.id, fix.note_unlisted);
    assert_eq!(notes[3].basic.id, fix.note_public);
}

#[tokio::test]
async fn test_user_notes_anon_viewer() {
    let st = test_setup().await;

    let fix = setup_user_notes_fixture(&st).await;

    let notes = get_user_notes(
        &st.app.maybe_conn(),
        &st.app.rconn(),
        None,
        fix.user1,
        20,
        None,
    )
    .await
    .unwrap();
    assert_eq!(notes.len(), 2);
    assert_eq!(notes[0].basic.id, fix.note_unlisted);
    assert_eq!(notes[1].basic.id, fix.note_public);
}

#[tokio::test]
async fn test_user_notes_follower_viewer() {
    let st = test_setup().await;

    let fix = setup_user_notes_fixture(&st).await;
    let following_user = register_user_for_test(&st, "followyou").await;

    user_follow_for_test(&st, following_user, fix.user1, true).await;

    let notes = get_user_notes(
        &st.app.maybe_conn(),
        &st.app.rconn(),
        Some(following_user),
        fix.user1,
        20,
        None,
    )
    .await
    .unwrap();
    assert_eq!(notes.len(), 3);
    assert_eq!(notes[0].basic.id, fix.note_follower);
    assert_eq!(notes[1].basic.id, fix.note_unlisted);
    assert_eq!(notes[2].basic.id, fix.note_public);
}

#[tokio::test]
async fn test_user_notes_mentioned_viewer() {
    let st = test_setup().await;

    let mentioned_user = register_user_for_test(&st, "mentionme").await;

    let fix = setup_user_notes_fixture(&st).await;

    let notes = get_user_notes(
        &st.app.maybe_conn(),
        &st.app.rconn(),
        Some(mentioned_user),
        fix.user1,
        20,
        None,
    )
    .await
    .unwrap();
    assert_eq!(notes.len(), 3);
    assert_eq!(notes[0].basic.id, fix.note_private);
    assert_eq!(notes[1].basic.id, fix.note_unlisted);
    assert_eq!(notes[2].basic.id, fix.note_public);
}
