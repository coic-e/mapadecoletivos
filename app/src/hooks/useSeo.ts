import { useEffect } from "react";

import { env } from "@/config/env";

export const SITE_NAME = "Mapa de Rave";

interface SeoOptions {
  /** Título completo da aba. O nome do site não é acrescentado automaticamente. */
  title: string;
  description: string;
  /** Caminho da rota, começando com "/". Vira a URL canônica. */
  path: string;
  /** Páginas sem valor de busca (formulário, erro) saem do índice. */
  noIndex?: boolean;
}

function upsertMeta(selector: string, attribute: string, name: string) {
  let element = document.head.querySelector<HTMLMetaElement>(selector);

  if (!element) {
    element = document.createElement("meta");
    element.setAttribute(attribute, name);
    document.head.appendChild(element);
  }

  return element;
}

/**
 * Ajusta título, descrição e canonical por rota.
 *
 * As tags são atualizadas no lugar em vez de renderizadas pelo React: o
 * index.html já traz uma cópia estática de cada uma (é o que os robôs de
 * preview leem, já que eles não executam JS), e renderizar de novo criaria
 * elementos duplicados — com dois <title>, o browser usa o primeiro.
 */
export function useSeo({ title, description, path, noIndex }: SeoOptions) {
  useEffect(() => {
    const siteUrl = env.VITE_SITE_URL.replace(/\/$/, "");
    const url = `${siteUrl}${path}`;

    document.title = title;

    upsertMeta('meta[name="description"]', "name", "description").content = description;
    upsertMeta('meta[property="og:title"]', "property", "og:title").content = title;
    upsertMeta('meta[property="og:description"]', "property", "og:description").content =
      description;
    upsertMeta('meta[property="og:url"]', "property", "og:url").content = url;
    upsertMeta('meta[name="robots"]', "name", "robots").content = noIndex
      ? "noindex, nofollow"
      : "index, follow";

    let canonical = document.head.querySelector<HTMLLinkElement>('link[rel="canonical"]');

    if (!canonical) {
      canonical = document.createElement("link");
      canonical.rel = "canonical";
      document.head.appendChild(canonical);
    }

    canonical.href = url;
  }, [title, description, path, noIndex]);
}
