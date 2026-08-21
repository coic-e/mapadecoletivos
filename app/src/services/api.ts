import axios from "axios";

import { env, usesSeedData } from "@/config/env";

import { clearToken, getToken } from "./auth";
import { seedAdapter } from "./seed/adapter";

const api = axios.create({
  baseURL: env.VITE_API_URL,
  // Sem API configurada, quem atende é o adapter de demonstração. Trocar o
  // adapter, e não as chamadas, mantém o código das telas idêntico ao de
  // produção: o preview exercita o mesmo caminho, com outro atendente.
  ...(usesSeedData ? { adapter: seedAdapter } : {}),
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
