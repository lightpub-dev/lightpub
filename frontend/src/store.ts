export function setToken(token: string | null) {
  if (token === null) {
    localStorage.removeItem('token')
    return
  }
  localStorage.setItem('token', token)
}

export function getToken() {
  return localStorage.getItem('token')
}

export function axiosOptions() {
  const token = getToken()
  return {
    headers: {
      Authorization: `Bearer ${token}`
    }
  }
}
