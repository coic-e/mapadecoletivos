import type { AxiosAdapter, AxiosRequestConfig, AxiosResponse } from "axios";

import { SEED_ADMIN, SEED_ORGANIZATIONS, type SeedOrganization } from "./data";

/**
 * Responde as chamadas da API com dados estáticos.
 *
 * É um adapter do axios, e não um mock espalhado pelas páginas, de propósito:
 * as telas continuam chamando `api.get("organizations")` igual em produção, e
 * o código que roda no preview é o mesmo que roda de verdade. O que muda é só
 * quem atende do outro lado.
 *
 * O estado vive em memória: aprovar algo no preview vale enquanto a aba estiver
 * aberta e some no reload. Não existe persistência aqui, e nem deveria.
 */

let organizations: SeedOrganization[] = SEED_ORGANIZATIONS.map((org) => ({
  ...org,
}));

let editRequests: Array<{
  id: number;
  organization_id: number;
  changes: Record<string, unknown>;
  message: string | null;
  requester_email: string | null;
  status: string;
  created_at: string;
}> = [];

let nextId = 100;

const DEMO_TOKEN = "demonstracao";

function respond<T>(config: AxiosRequestConfig, status: number, data: T): AxiosResponse<T> {
  return {
    data,
    status,
    statusText: status === 201 ? "Created" : "OK",
    headers: {},
    config: config as AxiosResponse<T>["config"],
  };
}

function fail(status: number, message: string) {
  const error = new Error(message) as Error & {
    response: { status: number; data: unknown };
  };

  error.response = { status, data: { status: "Error", message } };

  return Promise.reject(error);
}

/** Espera um pouco para a interface exercitar seus estados de carregamento. */
const delay = () => new Promise((resolve) => setTimeout(resolve, 220));

export const seedAdapter: AxiosAdapter = async (config) => {
  await delay();

  const method = (config.method ?? "get").toLowerCase();
  const url = (config.url ?? "").replace(/^\/+/, "");
  const [path, query] = url.split("?");
  const params = new URLSearchParams(query ?? "");
  const segments = path.split("/").filter(Boolean);
  const body = typeof config.data === "string" ? JSON.parse(config.data) : config.data;

  const approved = () => organizations.filter((org) => org.status === "approved");

  const findPublic = (identifier: string) =>
    approved().find((org) => org.slug === identifier || String(org.id) === identifier);

  // --- público ---------------------------------------------------------

  if (method === "get" && path === "organizations") {
    return respond(config, 200, approved());
  }

  if (method === "get" && segments[0] === "organizations" && segments.length === 2) {
    const organization = findPublic(segments[1]);

    return organization ? respond(config, 200, organization) : fail(404, "Resource not found");
  }

  if (method === "post" && path === "organizations") {
    // Cadastro novo no preview não vira nada: só confirma que a tela funciona.
    return respond(config, 201, {
      ...organizations[0],
      id: nextId++,
      slug: "cadastro-de-demonstracao",
      name: "Cadastro de demonstração",
      status: "pending",
    });
  }

  if (method === "post" && segments[0] === "organizations" && segments[2] === "edit-requests") {
    const organization = findPublic(segments[1]);

    if (!organization) return fail(404, "Resource not found");

    const request = {
      id: nextId++,
      organization_id: organization.id,
      changes: body?.changes ?? {},
      message: body?.message ?? null,
      requester_email: body?.requester_email ?? null,
      status: "pending",
      created_at: new Date().toISOString(),
    };

    editRequests = [...editRequests, request];

    return respond(config, 201, request);
  }

  // --- autenticação ----------------------------------------------------

  if (method === "post" && path === "auth/login") {
    // Qualquer credencial entra: não há segredo a proteger em dado inventado,
    // e exigir senha só esconderia o painel de quem quer revisá-lo.
    return respond(config, 200, { token: DEMO_TOKEN, admin: SEED_ADMIN });
  }

  if (method === "get" && path === "auth/me") {
    const authorized = String(config.headers?.Authorization ?? "").includes(DEMO_TOKEN);

    return authorized ? respond(config, 200, SEED_ADMIN) : fail(401, "Sessão inválida ou expirada");
  }

  // --- moderação -------------------------------------------------------

  if (method === "get" && path === "admin/organizations") {
    const status = params.get("status") ?? "pending";

    return respond(
      config,
      200,
      status === "all" ? organizations : organizations.filter((org) => org.status === status)
    );
  }

  if (
    method === "post" &&
    segments[0] === "admin" &&
    segments[1] === "organizations" &&
    (segments[3] === "approve" || segments[3] === "reject")
  ) {
    const id = Number(segments[2]);
    const status = segments[3] === "approve" ? "approved" : "rejected";

    organizations = organizations.map((org) =>
      org.id === id ? { ...org, status: status as SeedOrganization["status"] } : org
    );

    const updated = organizations.find((org) => org.id === id);

    return updated ? respond(config, 200, updated) : fail(404, "Resource not found");
  }

  if (method === "patch" && segments[0] === "admin" && segments[1] === "organizations") {
    const id = Number(segments[2]);

    organizations = organizations.map((org) => (org.id === id ? { ...org, ...body } : org));

    const updated = organizations.find((org) => org.id === id);

    return updated ? respond(config, 200, updated) : fail(404, "Resource not found");
  }

  if (method === "get" && path === "admin/edit-requests") {
    const status = params.get("status") ?? "pending";

    return respond(
      config,
      200,
      status === "all" ? editRequests : editRequests.filter((request) => request.status === status)
    );
  }

  if (method === "post" && segments[0] === "admin" && segments[1] === "edit-requests") {
    const id = Number(segments[2]);
    const decision = segments[3];

    const request = editRequests.find((item) => item.id === id);

    if (!request) return fail(404, "Resource not found");

    if (decision === "apply") {
      organizations = organizations.map((org) =>
        org.id === request.organization_id ? { ...org, ...request.changes } : org
      );
    }

    editRequests = editRequests.map((item) =>
      item.id === id ? { ...item, status: decision === "apply" ? "applied" : "rejected" } : item
    );

    return respond(config, 200, { status: "ok" });
  }

  return fail(404, `Rota não coberta pelos dados de demonstração: ${method} ${path}`);
};
