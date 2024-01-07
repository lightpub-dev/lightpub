export function storeLoginToken(token: string) {
    localStorage.setItem('token', token)
}

export function getLoginToken() {
    return localStorage.getItem('token')
}

export function storeUsername(username: string) {
    localStorage.setItem('username', username)
}

export function getUsername() {
    return localStorage.getItem('username')
}
