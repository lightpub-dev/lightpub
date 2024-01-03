export function storeLoginToken(token: string) {
  localStorage.setItem("token", token);
}

export function getLoginToken() {
  return localStorage.getItem("token");
}