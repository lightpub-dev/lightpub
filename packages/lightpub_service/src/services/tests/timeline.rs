use crate::services::{
    id::{NoteID, UserID},
    note::{ContentType, PostCreateOptionsBuilder, VisibilityModel, get_timeline_notes},
    tests::common::{TestState, test_setup},
};

use super::{auth::register_user_for_test, note::create_note_for_test, user::user_follow_for_test};

struct TimelineTestFixture {
    user1: UserID,
    follower_user: UserID,
    mentioned_user: UserID,
    other_user: UserID,
    note_public: NoteID,
    note_unlisted: NoteID,
    note_follower: NoteID,
    note_private: NoteID,
}

async fn setup_timeline_fixture(st: &TestState) -> TimelineTestFixture {
    let user1 = register_user_for_test(&st, "user1").await;

    let follower_user = register_user_for_test(&st, "follower").await;
    let mentioned_user = register_user_for_test(&st, "mentionme").await;
    let other_user = register_user_for_test(&st, "other").await;

    user_follow_for_test(st, follower_user, user1, true).await;

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

    TimelineTestFixture {
        user1,
        note_public: note1,
        note_unlisted: note2,
        note_follower: note3,
        note_private: note4,
        follower_user,
        mentioned_user,
        other_user,
    }
}

#[tokio::test]
async fn test_timeline_anon_viewer() {
    let st = test_setup().await;

    let result = get_timeline_notes(&st.app.maybe_conn(), &st.app.rconn(), None, false, 20, None)
        .await
        .unwrap();

    assert_eq!(result.len(), 0);
}

#[tokio::test]
async fn test_public_timeline_anon_viewer() {
    let st = test_setup().await;

    let fix = setup_timeline_fixture(&st).await;

    let timeline = get_timeline_notes(&st.app.maybe_conn(), &st.app.rconn(), None, true, 20, None)
        .await
        .unwrap();

    assert_eq!(timeline.len(), 1);
    assert_eq!(timeline[0].basic.id, fix.note_public);
}

#[tokio::test]
async fn test_public_timeline_follower_viewer() {
    let st = test_setup().await;

    let fix = setup_timeline_fixture(&st).await;

    let timeline = get_timeline_notes(
        &st.app.maybe_conn(),
        &st.app.rconn(),
        Some(fix.follower_user),
        true,
        20,
        None,
    )
    .await
    .unwrap();

    assert_eq!(timeline.len(), 3);
    assert_eq!(timeline[0].basic.id, fix.note_follower);
    assert_eq!(timeline[1].basic.id, fix.note_unlisted);
    assert_eq!(timeline[2].basic.id, fix.note_public);
}

#[tokio::test]
async fn test_timeline_follower_viewer() {
    let st = test_setup().await;

    let fix = setup_timeline_fixture(&st).await;

    let timeline = get_timeline_notes(
        &st.app.maybe_conn(),
        &st.app.rconn(),
        Some(fix.follower_user),
        false,
        20,
        None,
    )
    .await
    .unwrap();

    assert_eq!(timeline.len(), 3);
    assert_eq!(timeline[0].basic.id, fix.note_follower);
    assert_eq!(timeline[1].basic.id, fix.note_unlisted);
    assert_eq!(timeline[2].basic.id, fix.note_public);
}

#[tokio::test]
async fn test_public_timeline_mentioned_viewer() {
    let st = test_setup().await;

    let fix = setup_timeline_fixture(&st).await;

    let timeline = get_timeline_notes(
        &st.app.maybe_conn(),
        &st.app.rconn(),
        Some(fix.mentioned_user),
        true,
        20,
        None,
    )
    .await
    .unwrap();

    assert_eq!(timeline.len(), 2);
    assert_eq!(timeline[0].basic.id, fix.note_private);
    assert_eq!(timeline[1].basic.id, fix.note_public);
}

#[tokio::test]
async fn test_timeline_mentioned_viewer() {
    let st = test_setup().await;

    let fix = setup_timeline_fixture(&st).await;

    let timeline = get_timeline_notes(
        &st.app.maybe_conn(),
        &st.app.rconn(),
        Some(fix.mentioned_user),
        false,
        20,
        None,
    )
    .await
    .unwrap();

    assert_eq!(timeline.len(), 1);
    assert_eq!(timeline[0].basic.id, fix.note_private);
}

#[tokio::test]
async fn test_timeline_self_viewer() {
    let st = test_setup().await;

    let fix = setup_timeline_fixture(&st).await;

    let timeline = get_timeline_notes(
        &st.app.maybe_conn(),
        &st.app.rconn(),
        Some(fix.user1),
        false,
        20,
        None,
    )
    .await
    .unwrap();

    assert_eq!(timeline.len(), 4);
    assert_eq!(timeline[0].basic.id, fix.note_private);
    assert_eq!(timeline[1].basic.id, fix.note_follower);
    assert_eq!(timeline[2].basic.id, fix.note_unlisted);
    assert_eq!(timeline[3].basic.id, fix.note_public);
}

#[tokio::test]
async fn test_public_timeline_self_viewer() {
    let st = test_setup().await;

    let fix = setup_timeline_fixture(&st).await;

    let timeline = get_timeline_notes(
        &st.app.maybe_conn(),
        &st.app.rconn(),
        Some(fix.user1),
        true,
        20,
        None,
    )
    .await
    .unwrap();

    assert_eq!(timeline.len(), 4);
    assert_eq!(timeline[0].basic.id, fix.note_private);
    assert_eq!(timeline[1].basic.id, fix.note_follower);
    assert_eq!(timeline[2].basic.id, fix.note_unlisted);
    assert_eq!(timeline[3].basic.id, fix.note_public);
}

#[tokio::test]
async fn test_timeline_other_viewer() {
    let st = test_setup().await;

    let fix = setup_timeline_fixture(&st).await;

    let timeline = get_timeline_notes(
        &st.app.maybe_conn(),
        &st.app.rconn(),
        Some(fix.other_user),
        false,
        20,
        None,
    )
    .await
    .unwrap();

    assert_eq!(timeline.len(), 0);
}

#[tokio::test]
async fn test_public_timeline_other_viewer() {
    let st = test_setup().await;

    let fix = setup_timeline_fixture(&st).await;

    let timeline = get_timeline_notes(
        &st.app.maybe_conn(),
        &st.app.rconn(),
        Some(fix.other_user),
        true,
        20,
        None,
    )
    .await
    .unwrap();

    assert_eq!(timeline.len(), 1);
    assert_eq!(timeline[0].basic.id, fix.note_public);
}
