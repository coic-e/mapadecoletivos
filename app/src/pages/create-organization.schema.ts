import { z } from "zod";

export const ABOUT_MAX_LENGTH = 300;

export const createOrganizationSchema = z.object({
  name: z.string().trim().min(3, "Informe pelo menos 3 caracteres"),
  about: z
    .string()
    .trim()
    .min(10, "Conte um pouco mais sobre o rolê")
    .max(ABOUT_MAX_LENGTH, `Máximo de ${ABOUT_MAX_LENGTH} caracteres`),
  email: z.email("E-mail inválido"),
  social: z.string().trim().min(1, "Informe um link ou @ do perfil"),
  uf: z.string().min(1, "Selecione o estado"),
  city: z.string().trim().min(2, "Informe a cidade"),
  type: z.string().min(1, "Selecione o tipo"),
  latitude: z.string().min(1, "Clique no mapa para marcar a localização"),
  longitude: z.string().min(1, "Clique no mapa para marcar a localização"),
});

export type CreateOrganizationValues = z.infer<typeof createOrganizationSchema>;
