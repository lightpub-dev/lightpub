use crate::services::{
    follow::{FollowStats, follow_user, get_follow_stats, unfollow_user},
    id::UserID,
    note::{ContentType, PostCreateOptionsBuilder, VisibilityModel},
    tests::{
        auth::{register_sample_user, register_user_for_test},
        common::test_setup,
        note::create_note_for_test,
    },
    user::{
        UserProfileUpdate, block_user, get_user_profile_by_id, is_blocking_or_blocked,
        is_blocking_user, unblock_user, update_user_profile,
    },
};

use super::common::TestState;

#[tokio::test]
async fn test_user_profile_update() {
    let st = test_setup().await;

    let user_id = register_sample_user(&st).await;

    let data = st.app.fed().to_request_data();
    update_user_profile(
        st.app.conn(),
        &st.app.rconn(),
        st.app.qconn(),
        user_id,
        &UserProfileUpdate {
            nickname: "updatednick".to_string().into(),
            bio: "bio dayo".to_string().into(),
            auto_follow_accept: Some(false),
            hide_follows: Some(true),
            avatar_upload_id: Some(None),
        },
        st.app.base_url(),
        &data,
    )
    .await
    .unwrap();

    let profile = get_user_profile_by_id(&st.app.maybe_conn(), &st.app.rconn(), None, user_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(*profile.basic.id(), user_id);
    assert_eq!(profile.basic.username(), "testuser");
    assert_eq!(profile.basic.nickname(), "updatednick");
    assert_eq!(profile.basic.bio(), "bio dayo");
    assert_eq!(profile.auto_follow_accept, false);
    assert_eq!(profile.hide_follows, true);
    assert!(profile.basic.avatar().is_none());
}

#[tokio::test]
async fn test_user_profile_note_count() {
    let st = test_setup().await;

    let user_id = register_sample_user(&st).await;

    // let data = st.app.fed().to_request_data();

    create_note_for_test(
        &st,
        user_id,
        "aaa",
        ContentType::Plain,
        VisibilityModel::Public,
        &PostCreateOptionsBuilder::default().build().unwrap(),
    )
    .await
    .unwrap();
    create_note_for_test(
        &st,
        user_id,
        "aaa",
        ContentType::Plain,
        VisibilityModel::Unlisted,
        &PostCreateOptionsBuilder::default().build().unwrap(),
    )
    .await
    .unwrap();
    create_note_for_test(
        &st,
        user_id,
        "aaa",
        ContentType::Plain,
        VisibilityModel::Follower,
        &PostCreateOptionsBuilder::default().build().unwrap(),
    )
    .await
    .unwrap();
    create_note_for_test(
        &st,
        user_id,
        "aaa",
        ContentType::Plain,
        VisibilityModel::Private,
        &PostCreateOptionsBuilder::default().build().unwrap(),
    )
    .await
    .unwrap();

    let anon_profile = get_user_profile_by_id(&st.app.maybe_conn(), &st.app.rconn(), None, user_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(anon_profile.note_count, 2);

    let anon_profile = get_user_profile_by_id(
        &st.app.maybe_conn(),
        &st.app.rconn(),
        Some(user_id),
        user_id,
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(anon_profile.note_count, 4);
}

async fn get_follow_stats_for_test(st: &TestState, user_id: UserID) -> FollowStats {
    get_follow_stats(&st.app.maybe_conn(), user_id)
        .await
        .unwrap()
}

#[tokio::test]
async fn test_user_follow() {
    let st = test_setup().await;

    let user1 = register_user_for_test(&st, "user1").await;
    let user2 = register_user_for_test(&st, "user2").await;

    let u1_stats = get_follow_stats_for_test(&st, user1).await;
    let u2_stats = get_follow_stats_for_test(&st, user2).await;
    assert_eq!(u1_stats.following, 0);
    assert_eq!(u1_stats.followers, 0);
    assert_eq!(u2_stats.following, 0);
    assert_eq!(u2_stats.followers, 0);

    user_follow_for_test(&st, user1, user2, true).await;

    let u1_stats = get_follow_stats_for_test(&st, user1).await;
    let u2_stats = get_follow_stats_for_test(&st, user2).await;
    assert_eq!(u1_stats.following, 1);
    assert_eq!(u1_stats.followers, 0);
    assert_eq!(u2_stats.following, 0);
    assert_eq!(u2_stats.followers, 1);

    user_follow_for_test(&st, user1, user2, false).await;

    let u1_stats = get_follow_stats_for_test(&st, user1).await;
    let u2_stats = get_follow_stats_for_test(&st, user2).await;
    assert_eq!(u1_stats.following, 0);
    assert_eq!(u1_stats.followers, 0);
    assert_eq!(u2_stats.following, 0);
    assert_eq!(u2_stats.followers, 0);
}

#[tokio::test]
async fn test_user_block() {
    let st = test_setup().await;

    let user1 = register_user_for_test(&st, "user1").await;
    let user2 = register_user_for_test(&st, "user2").await;

    assert!(
        !is_blocking_or_blocked(&st.app.maybe_conn(), user1, user2)
            .await
            .unwrap()
    );

    block_user(&st.app.maybe_conn(), user1, user2)
        .await
        .unwrap();

    assert!(
        is_blocking_user(&st.app.maybe_conn(), user1, user2)
            .await
            .unwrap()
    );
    assert!(
        is_blocking_or_blocked(&st.app.maybe_conn(), user1, user2)
            .await
            .unwrap()
    );
    assert!(
        is_blocking_or_blocked(&st.app.maybe_conn(), user2, user1)
            .await
            .unwrap()
    );

    unblock_user(&st.app.maybe_conn(), user1, user2)
        .await
        .unwrap();

    assert!(
        !is_blocking_user(&st.app.maybe_conn(), user1, user2)
            .await
            .unwrap()
    );
}

pub async fn user_follow_for_test(st: &TestState, user1: UserID, user2: UserID, add: bool) {
    if add {
        follow_user(
            st.app.conn(),
            &st.app.rconn(),
            st.app.qconn(),
            st.app.wp(),
            user1,
            user2,
            st.app.base_url(),
        )
        .await
        .unwrap();
    } else {
        unfollow_user(
            st.app.conn(),
            &st.app.rconn(),
            st.app.qconn(),
            user1,
            user2,
            st.app.base_url(),
        )
        .await
        .unwrap();
    }
}
