pub mod state;

pub fn validate_username(username: &str) -> bool {
    // username must
    // - be at least 3 characters long, and at most 16 characters long
    // - contain only alphanumeric characters + '-' + '_'
    // - not start with '-' or '_'
    // - not contain consecutive '-' or '_'

    if username.len() < 3 || username.len() > 16 {
        return false;
    }

    for ch in username.chars() {
        if !ch.is_alphanumeric() && ch != '-' && ch != '_' {
            return false;
        }
    }

    if username.starts_with('-') || username.starts_with('_') {
        return false;
    }

    let mut prev = ' ';
    for ch in username.chars() {
        if ch == '-' || ch == '_' {
            if prev == '-' || prev == '_' {
                return false;
            }
        }
        prev = ch;
    }

    true
}

pub fn validate_password(password: &str) -> bool {
    // password must
    // - be at least 8 characters long and at most 48 characters long
    // - contain at least one uppercase letter
    // - contain at least one lowercase letter
    // - contain at least one digit
    // - contain at least one special character (!@#$%^&*=+-_)

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
        } else if "!@#$%^&*=+-_".contains(ch) {
            has_special = true;
        } else {
            // invalid character
            return false;
        }
    }

    has_upper && has_lower && has_digit && has_special
}
