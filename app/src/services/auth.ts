import api from "./api";

const TOKEN_KEY = "mapaderave.admin.token";

export interface Admin {
  id: number;
  name: string;
  email: string;
}

export function getToken(): string | null {
  return localStorage.getItem(TOKEN_KEY);
}

export function setToken(token: string) {
  localStorage.setItem(TOKEN_KEY, token);
}

export function clearToken() {
  localStorage.removeItem(TOKEN_KEY);
}

export async function login(email: string, password: string): Promise<Admin> {
  const { data } = await api.post<{ token: string; admin: Admin }>("auth/login", {
    email,
    password,
  });

  setToken(data.token);

  return data.admin;
}

/** O painel usa para saber se o token guardado ainda vale. */
export async function fetchCurrentAdmin(): Promise<Admin> {
  const { data } = await api.get<Admin>("auth/me");

  return data;
}

export function logout() {
  clearToken();
}
