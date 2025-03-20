use crate::services::{
    auth::{check_password_user, register_user},
    id::UserID,
};

use super::common::{TestState, test_setup};

#[tokio::test]
async fn test_user_register() {
    let st = test_setup().await;

    let result = register_user(st.app.conn(), "testuser", "testnick", "testpass")
        .await
        .unwrap();

    let _ = result;
}

#[tokio::test]
async fn test_user_login_success() {
    let st = test_setup().await;

    let reg = register_user(st.app.conn(), "testuser", "testnick", "testpass")
        .await
        .unwrap();

    let login = check_password_user(st.app.conn(), "testuser", "testpass")
        .await
        .unwrap();

    assert!(login.is_some());
    assert_eq!(reg.user_id, login.unwrap().user_id);
}

#[tokio::test]
async fn test_user_login_pass_wrong() {
    let st = test_setup().await;

    register_user(st.app.conn(), "testuser", "testnick", "testpass")
        .await
        .unwrap();

    let login = check_password_user(st.app.conn(), "testuser", "testpas")
        .await
        .unwrap();

    assert!(login.is_none());
}

#[tokio::test]
async fn test_user_login_not_exists() {
    let st = test_setup().await;

    register_user(st.app.conn(), "testuser", "testnick", "testpass")
        .await
        .unwrap();

    let login = check_password_user(st.app.conn(), "testuse", "testpass")
        .await
        .unwrap();

    assert!(login.is_none());
}

pub async fn register_user_for_test(st: &TestState, username: &str) -> UserID {
    register_user(
        st.app.conn(),
        username,
        &format!("{username}_nick"),
        "testpass",
    )
    .await
    .unwrap()
    .user_id
}

pub async fn register_sample_user(st: &TestState) -> UserID {
    register_user_for_test(st, "testuser").await
}
