import { z } from "zod";

export const ABOUT_MAX_LENGTH = 1200;

/**
 * Espelha MUSIC_GENRES e FREQUENCIES do db-types. São listas fechadas dos dois
 * lados: se divergirem, a API rejeita o cadastro com "gênero inválido".
 */
export const MUSIC_GENRES = [
  "Techno",
  "Hard Techno",
  "Minimal",
  "Acid",
  "House",
  "Deep House",
  "Tech House",
  "Afro House",
  "Disco",
  "Electro",
  "Trance",
  "Psytrance",
  "Drum & Bass",
  "Jungle",
  "Dubstep",
  "Bass Music",
  "Breakbeat",
  "Hardcore",
  "Funk",
  "Latin Club",
  "Guaracha",
  "Reggaeton",
  "Ambient",
  "Experimental",
  "Outro",
] as const;

export const FREQUENCIES = ["Semanal", "Quinzenal", "Mensal", "Sazonal", "Pontual"] as const;

const LINK_FIELDS = [
  "instagram",
  "soundcloud",
  "bandcamp",
  "youtube",
  "spotify",
  "website",
] as const;

const baseSchema = z.object({
  name: z.string().trim().min(3, "Informe pelo menos 3 caracteres"),
  about: z
    .string()
    .trim()
    .min(10, "Conte um pouco mais sobre o rolê")
    .max(ABOUT_MAX_LENGTH, `Máximo de ${ABOUT_MAX_LENGTH} caracteres`),
  email: z.email("E-mail inválido"),
  genres: z
    .array(z.enum(MUSIC_GENRES))
    .min(1, "Escolha pelo menos um gênero")
    .max(MUSIC_GENRES.length),
  address: z.string().trim().max(200, "Endereço longo demais").optional(),
  instagram: z.string().trim().max(200).optional(),
  soundcloud: z.string().trim().max(200).optional(),
  bandcamp: z.string().trim().max(200).optional(),
  youtube: z.string().trim().max(200).optional(),
  spotify: z.string().trim().max(200).optional(),
  website: z.string().trim().max(200).optional(),
  frequency: z.enum(FREQUENCIES).optional(),
  isActive: z.boolean(),
  uf: z.string().min(1, "Selecione o estado"),
  city: z.string().trim().min(2, "Informe a cidade"),
  type: z.string().min(1, "Selecione o tipo"),
  latitude: z.string().min(1, "Clique no mapa para marcar a localização"),
  longitude: z.string().min(1, "Clique no mapa para marcar a localização"),
  // Só existe no formulário: serve para a pessoa afirmar que pode cadastrar
  // aquele coletivo. Não vai para a API.
  consent: z.boolean().refine((accepted) => accepted, {
    message: "Confirme que você tem autorização para cadastrar este rolê",
  }),
});

/**
 * Sem nenhum link o cadastro não serve para o que o site existe: achar o rolê.
 * A mesma regra existe na API; aqui ela evita a viagem até o servidor.
 */
export const createOrganizationSchema = baseSchema.refine(
  (values) => LINK_FIELDS.some((field) => (values[field] ?? "") !== ""),
  {
    message: "Informe pelo menos um link — Instagram, SoundCloud, site…",
    path: ["instagram"],
  }
);

export type CreateOrganizationValues = z.infer<typeof createOrganizationSchema>;
