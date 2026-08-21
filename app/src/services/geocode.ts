import { env } from "@/config/env";

export interface GeocodeResult {
  id: string;
  /** Endereço formatado, como o Mapbox devolve. */
  label: string;
  latitude: number;
  longitude: number;
  /** Cidade e UF extraídas do resultado, para preencher o formulário. */
  city: string;
  uf: string;
  /** Logradouro: o primeiro trecho do endereço formatado, antes da cidade. */
  address: string;
}

interface MapboxContext {
  id: string;
  text?: string;
  short_code?: string;
}

interface MapboxFeature {
  id: string;
  place_name: string;
  center: [number, number];
  context?: MapboxContext[];
  place_type?: string[];
  text?: string;
}

function findContext(feature: MapboxFeature, prefix: string) {
  return feature.context?.find((item) => item.id.startsWith(prefix));
}

function toResult(feature: MapboxFeature): GeocodeResult {
  const [longitude, latitude] = feature.center;

  // Quando o próprio resultado é a cidade, ela não aparece no context.
  const isPlace = feature.place_type?.includes("place") === true;
  const place = findContext(feature, "place.");
  const region = findContext(feature, "region.");

  // short_code vem como "BR-RS"; a UF é o que vem depois do hífen.
  const uf = region?.short_code?.split("-")[1] ?? "";

  return {
    id: feature.id,
    label: feature.place_name,
    address: feature.place_name.split(",")[0]?.trim() ?? "",
    latitude,
    longitude,
    city: (isPlace ? feature.text : place?.text) ?? "",
    uf: uf.toUpperCase(),
  };
}

/**
 * Busca endereços no Mapbox, limitado ao Brasil.
 *
 * Usa o mesmo token dos tiles do mapa. Chamado direto com fetch porque não
 * passa pela API do projeto — o axios de services/api aponta para o backend.
 */
export async function searchAddress(query: string, signal?: AbortSignal): Promise<GeocodeResult[]> {
  const term = query.trim();

  if (term.length < 3) {
    return [];
  }

  const url =
    `https://api.mapbox.com/geocoding/v5/mapbox.places/${encodeURIComponent(term)}.json` +
    `?access_token=${env.VITE_MAPBOX_ACCESS_TOKEN}&country=br&language=pt&limit=5&types=address,poi,place,neighborhood`;

  const response = await fetch(url, { signal });

  if (!response.ok) {
    throw new Error(`Geocoding falhou: ${response.status}`);
  }

  const data: { features?: MapboxFeature[] } = await response.json();

  return (data.features ?? []).map(toResult);
}
