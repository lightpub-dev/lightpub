pub mod state;

pub fn validate_username(username: &str) -> bool {
    todo!()
}

pub fn validate_password(password: &str) -> bool {
    // password must
    // - be at least 8 characters long and at most 48 characters long
    // - contain at least one uppercase letter
    // - contain at least one lowercase letter
    // - contain at least one digit
    // - contain at least one special character (!?@#$%^&*=+-_)

    if password.len() < 8 || password.len() > 48 {
        return false;
    }

    let mut has_upper = false;
    let mut has_lower = false;
    let mut has_digit = false;
    let mut has_special = false;
    for ch in password.chars() {
        if ch.is_uppercase() {
            has_upper = true;
        } else if ch.is_lowercase() {
            has_lower = true;
        } else if ch.is_digit(10) {
            has_digit = true;
        } else if "!?@#$%^&*=+-_".contains(ch) {
            has_special = true;
        } else {
            // invalid character
            return false;
        }
    }

    has_upper && has_lower && has_digit && has_special
}

#[cfg(test)]
mod tests {
    use super::validate_password;
    use super::validate_username;

    #[test]
    fn check_valid_username() {
        assert!(validate_username("abc"));
        assert!(!validate_username("ab"));
        assert!(validate_username("12c"));
        assert!(validate_username("long_user_name_1"));
        assert!(!validate_username("long_user_name_12"));
        assert!(!validate_username("-damedayo"));
        assert!(!validate_username("_damedayo"));
        assert!(!validate_username("no_python__style"));
        assert!(!validate_username("no_python--style"));
        assert!(validate_username("username"));
        assert!(validate_username("username1234"));
        assert!(!validate_username("with漢字"));
    }

    #[test]
    fn check_valid_password() {
        assert!(validate_password("Password1!"));
        assert!(validate_password("1234AbcD!?"));
        assert!(!validate_password("ToSrt!"));
        assert!(!validate_password("alphabetOnly!"));
        assert!(!validate_password("noSymbols1234"));
        assert!(!validate_password("12345609801!?"));
        assert!(!validate_password("1234AbcD!?with漢字"))
    }
}
