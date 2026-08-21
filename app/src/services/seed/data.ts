/**
 * Dados de demonstração.
 *
 * Usados só quando VITE_API_URL está vazia, para o front poder ser publicado
 * sozinho e as telas serem revisadas antes de a API existir num servidor.
 *
 * Tudo aqui é inventado: os coletivos, os endereços e os contatos. Os e-mails
 * usam o domínio reservado `example.com` justamente para nunca chegarem a
 * ninguém de verdade, e as capas são SVGs em `public/seed/`.
 */

export interface SeedImage {
  id: number;
  url: string;
}

export interface SeedOrganization {
  id: number;
  slug: string;
  name: string;
  latitude: number;
  longitude: number;
  type: string;
  city: string;
  uf: string;
  email: string;
  about: string;
  genres: string[];
  address: string | null;
  instagram: string | null;
  soundcloud: string | null;
  bandcamp: string | null;
  youtube: string | null;
  spotify: string | null;
  website: string | null;
  is_active: boolean;
  frequency: string | null;
  status: "pending" | "approved" | "rejected";
  created_at: string;
  rejection_reason: string | null;
  images: SeedImage[];
}

const cover = (name: string) => `/seed/${name}.svg`;

export const SEED_ORGANIZATIONS: SeedOrganization[] = [
  {
    id: 1,
    slug: "porao-do-cais",
    name: "Porão do Cais",
    latitude: -30.0277,
    longitude: -51.2287,
    type: "Club",
    city: "Porto Alegre",
    uf: "RS",
    email: "contato@porao-do-cais.example.com",
    about:
      "Club de techno num galpão portuário reformado, com pista escura, sistema de som próprio e residentes fixos desde 2019.",
    genres: ["Techno", "Hard Techno", "Acid"],
    address: "Av. do Cais, 210",
    instagram: "@poraodocais",
    soundcloud: "soundcloud.com/poraodocais",
    bandcamp: null,
    youtube: null,
    spotify: null,
    website: null,
    is_active: true,
    frequency: "Semanal",
    status: "approved",
    created_at: "2026-03-14T22:00:00",
    rejection_reason: null,
    images: [{ id: 1, url: cover("porao-do-cais") }],
  },
  {
    id: 2,
    slug: "corrente-coletivo",
    name: "Corrente Coletivo",
    latitude: -23.5505,
    longitude: -46.6333,
    type: "Coletivo",
    city: "São Paulo",
    uf: "SP",
    email: "contato@corrente.example.com",
    about:
      "Coletivo que organiza festas em espaços ocupados e cede a pista para artistas em primeira apresentação. Toca de house a bass music.",
    genres: ["House", "Deep House", "Bass Music"],
    address: null,
    instagram: "@correntecoletivo",
    soundcloud: "soundcloud.com/correntecoletivo",
    bandcamp: "correntecoletivo.bandcamp.com",
    youtube: null,
    spotify: null,
    website: "https://corrente.example.com",
    is_active: true,
    frequency: "Mensal",
    status: "approved",
    created_at: "2026-04-02T19:30:00",
    rejection_reason: null,
    images: [{ id: 2, url: cover("corrente-coletivo") }],
  },
  {
    id: 3,
    slug: "maresia-label",
    name: "Maresia Label",
    latitude: -8.0578,
    longitude: -34.8829,
    type: "Label",
    city: "Recife",
    uf: "PE",
    email: "demos@maresia.example.com",
    about:
      "Selo dedicado a produções do Nordeste, com lançamentos que cruzam eletrônica e percussão regional. Recebe demos o ano inteiro.",
    genres: ["Experimental", "Bass Music", "Funk"],
    address: null,
    instagram: "@maresialabel",
    soundcloud: "soundcloud.com/maresialabel",
    bandcamp: "maresialabel.bandcamp.com",
    youtube: null,
    spotify: "open.spotify.com/artist/maresia",
    website: null,
    is_active: true,
    frequency: null,
    status: "approved",
    created_at: "2026-01-20T10:15:00",
    rejection_reason: null,
    images: [{ id: 3, url: cover("maresia-label") }],
  },
  {
    id: 4,
    slug: "estacao-subterranea",
    name: "Estação Subterrânea",
    latitude: -19.9245,
    longitude: -43.9352,
    type: "Festa",
    city: "Belo Horizonte",
    uf: "MG",
    email: "contato@subterranea.example.com",
    about:
      "Festa itinerante que acontece em estacionamentos e galpões vazios, sempre com line-up divulgado no dia. Foco em minimal e electro.",
    genres: ["Minimal", "Electro", "Techno"],
    address: null,
    instagram: "@estacaosubterranea",
    soundcloud: null,
    bandcamp: null,
    youtube: "youtube.com/@estacaosubterranea",
    spotify: null,
    website: null,
    is_active: true,
    frequency: "Sazonal",
    status: "approved",
    created_at: "2026-02-08T23:45:00",
    rejection_reason: null,
    images: [{ id: 4, url: cover("estacao-subterranea") }],
  },
  {
    id: 5,
    slug: "radio-fluxo",
    name: "Rádio Fluxo",
    latitude: -12.9777,
    longitude: -38.5016,
    type: "Radio",
    city: "Salvador",
    uf: "BA",
    email: "contato@radiofluxo.example.com",
    about:
      "Rádio online com programação ao vivo de quinta a domingo, transmitindo sets de residentes e convidados de todo o país.",
    genres: ["Afro House", "Disco", "Ambient"],
    address: "Rua da Ladeira, 88",
    instagram: "@radiofluxo",
    soundcloud: "soundcloud.com/radiofluxo",
    bandcamp: null,
    youtube: "youtube.com/@radiofluxo",
    spotify: null,
    website: "https://radiofluxo.example.com",
    is_active: true,
    frequency: "Semanal",
    status: "approved",
    created_at: "2026-05-11T16:00:00",
    rejection_reason: null,
    images: [{ id: 5, url: cover("radio-fluxo") }],
  },
  {
    id: 6,
    slug: "nucleo-236",
    name: "Núcleo 236",
    latitude: -25.4284,
    longitude: -49.2733,
    type: "Nucleo",
    city: "Curitiba",
    uf: "PR",
    email: "contato@nucleo236.example.com",
    about:
      "Núcleo de produção e formação: oficinas de discotecagem, mentoria para produtoras iniciantes e uma festa de encerramento por temporada.",
    genres: ["Drum & Bass", "Jungle", "Breakbeat"],
    address: null,
    instagram: "@nucleo236",
    soundcloud: null,
    bandcamp: null,
    youtube: null,
    spotify: null,
    website: null,
    is_active: false,
    frequency: "Pontual",
    status: "approved",
    created_at: "2025-11-30T14:20:00",
    rejection_reason: null,
    images: [{ id: 6, url: cover("nucleo-236") }],
  },
  // Pendente: existe para o painel de moderação ter o que mostrar no preview.
  {
    id: 7,
    slug: "sala-vermelha",
    name: "Sala Vermelha",
    latitude: -22.9068,
    longitude: -43.1729,
    type: "Bar",
    city: "Rio de Janeiro",
    uf: "RJ",
    email: "contato@salavermelha.example.com",
    about:
      "Bar com pista nos fundos, discotecagem de vinil às sextas e uma programação que vai de disco a house clássico.",
    genres: ["Disco", "House"],
    address: "Rua do Lavradio, 55",
    instagram: "@salavermelha",
    soundcloud: null,
    bandcamp: null,
    youtube: null,
    spotify: null,
    website: null,
    is_active: true,
    frequency: "Semanal",
    status: "pending",
    created_at: "2026-08-19T21:10:00",
    rejection_reason: null,
    images: [{ id: 7, url: cover("sala-vermelha") }],
  },
];

/** Admin de demonstração: qualquer senha entra, porque nada aqui é real. */
export const SEED_ADMIN = {
  id: 1,
  name: "Moderação (demonstração)",
  email: "demo@example.com",
};
