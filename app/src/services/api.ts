import axios from "axios";

import { env } from "@/config/env";

import { clearToken, getToken } from "./auth";

const api = axios.create({
  baseURL: env.VITE_API_URL,
});

// Anexa o token do moderador quando existe. As rotas públicas ignoram o
// cabeçalho, então não faz mal mandá-lo sempre.
api.interceptors.request.use((config) => {
  const token = getToken();

  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }

  return config;
});

// Token expirado ou revogado: descarta antes que a tela fique num limbo de
// "logado" sem conseguir carregar nada.
api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error?.response?.status === 401) {
      clearToken();
    }

    return Promise.reject(error);
  }
);

export default api;
