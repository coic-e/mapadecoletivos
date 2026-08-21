import api from "./api";

/**
 * Campos que podem ser corrigidos depois do cadastro. Espelha
 * OrganizationChanges do db-types: campo ausente significa "não mexer".
 */
export interface OrganizationChanges {
  name?: string;
  about?: string;
  email?: string;
  city?: string;
  uf?: string;
  address?: string;
  instagram?: string;
  soundcloud?: string;
  bandcamp?: string;
  youtube?: string;
  spotify?: string;
  website?: string;
  genres?: string[];
  frequency?: string;
  is_active?: boolean;
}

export interface EditRequest {
  id: number;
  organization_id: number;
  changes: OrganizationChanges;
  message: string | null;
  requester_email: string | null;
  status: "pending" | "applied" | "rejected";
  created_at: string;
}

/** Sugestão de correção. Não altera nada: entra na fila da moderação. */
export async function requestEdit(
  slug: string,
  changes: OrganizationChanges,
  message: string,
  requesterEmail: string
) {
  const { data } = await api.post<EditRequest>(`organizations/${slug}/edit-requests`, {
    changes,
    message: message.trim() || null,
    requester_email: requesterEmail.trim() || null,
  });

  return data;
}

export async function fetchEditRequests(status: string) {
  const { data } = await api.get<EditRequest[]>(`admin/edit-requests?status=${status}`);

  return data;
}

export async function reviewEditRequest(id: number, decision: "apply" | "reject") {
  await api.post(`admin/edit-requests/${id}/${decision}`);
}

/** Edição direta, só admin. */
export async function updateOrganization(id: number, changes: OrganizationChanges) {
  await api.patch(`admin/organizations/${id}`, changes);
}
