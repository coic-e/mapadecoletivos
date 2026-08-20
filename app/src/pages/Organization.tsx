import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { MapContainer, TileLayer, Marker } from "react-leaflet";

import { env } from "@/config/env";
import Sidebar from "@/components/Sidebar";
import { Button } from "@/components/ui/button";
import mapIcon from "@/utils/mapIcon";
import api from "@/services/api";
import { cn } from "@/lib/utils";
import { useSeo } from "@/hooks/useSeo";

interface IOrganization {
  id: number;
  latitude: number;
  longitude: number;
  name: string;
  type: string;
  about: string;
  email: string;
  social: string;
  city: string;
  uf: string;
  images: Array<{
    id: number;
    url: string;
  }>;
}

interface OrganizationParams extends Record<string, string | undefined> {
  id: string;
}

/**
 * O cadastro aceita o social em formato livre ("@perfil", "instagram.com/x",
 * URL completa). Sem normalizar, "@perfil" vira link relativo e leva para
 * /raves/@perfil. Handle solto assume Instagram, que é onde a cena vive.
 */
function toSocialUrl(social: string) {
  const value = social.trim();

  if (value === "") return null;
  if (/^https?:\/\//i.test(value)) return value;
  if (value.startsWith("@")) return `https://instagram.com/${value.slice(1)}`;

  return `https://${value}`;
}

function Organization() {
  const params = useParams<OrganizationParams>();
  const [organization, setOrganization] = useState<IOrganization>();
  const [status, setStatus] = useState<"loading" | "ready" | "error">("loading");
  const [activeImageIndex, setActiveImageIndex] = useState(0);

  useEffect(() => {
    // Impede que a resposta de um id antigo sobrescreva a do id atual quando
    // se navega rápido entre organizações.
    let cancelled = false;

    setStatus("loading");
    setActiveImageIndex(0);

    api
      .get(`organizations/${params.id}`)
      .then((response) => {
        if (cancelled) return;

        if (response.data) {
          setOrganization(response.data);
          setStatus("ready");
        } else {
          setStatus("error");
        }
      })
      .catch(() => {
        if (!cancelled) {
          setStatus("error");
        }
      });

    return () => {
      cancelled = true;
    };
  }, [params.id]);

  // Chamado antes do early return: hook não pode ficar atrás de condicional.
  useSeo({
    title: organization
      ? `${organization.name} — ${organization.type} em ${organization.city}/${organization.uf} | Mapa de Rave`
      : "Carregando… | Mapa de Rave",
    description: organization
      ? organization.about.slice(0, 155)
      : "Perfil de um coletivo de música eletrônica no Mapa de Rave.",
    path: `/raves/${params.id}`,
  });

  if (status === "error") {
    return (
      <div className="flex min-h-dvh flex-col items-center justify-center gap-4 bg-background px-6 text-center">
        <h1 className="font-display text-4xl tracking-wide text-foreground">Rolê não encontrado</h1>
        <p className="max-w-md font-sans text-base text-muted-foreground">
          Esse cadastro não existe ou saiu do ar. Volte para o mapa e escolha outro.
        </p>
        <Button asChild className="mt-2">
          <Link to="/raves">Voltar para o mapa</Link>
        </Button>
      </div>
    );
  }

  if (status === "loading" || !organization) {
    return (
      <div className="flex min-h-dvh items-center justify-center bg-background">
        <p className="font-sans text-base text-muted-foreground">Buscando informações do rolê...</p>
      </div>
    );
  }

  const images = organization.images ?? [];
  const socialUrl = toSocialUrl(organization.social);
  const activeImage = images[activeImageIndex];

  return (
    <div className="flex min-h-dvh flex-col bg-background md:flex-row">
      <Sidebar />

      <main className="flex flex-1 justify-center px-4 py-8 md:py-16 md:pr-8 md:pl-32">
        <article className="w-full max-w-3xl overflow-hidden rounded-xl border border-border bg-card">
          {activeImage && (
            <img
              src={activeImage.url}
              alt={organization.name}
              className="h-75 w-full object-cover"
            />
          )}

          {images.length > 1 && (
            <div className="grid grid-cols-[repeat(auto-fill,minmax(88px,1fr))] gap-4 px-6 pt-4 md:px-10">
              {images.map((image, index) => (
                <button
                  type="button"
                  key={image.id}
                  onClick={() => setActiveImageIndex(index)}
                  aria-label={`Foto ${index + 1} de ${images.length}`}
                  aria-pressed={activeImageIndex === index}
                  className={cn(
                    "h-22 cursor-pointer overflow-hidden rounded-lg border-0 bg-transparent p-0 opacity-60 transition-opacity",
                    activeImageIndex === index && "opacity-100"
                  )}
                >
                  <img src={image.url} alt="" className="h-22 w-full object-cover" />
                </button>
              ))}
            </div>
          )}

          <div className="p-6 md:p-12">
            <header className="flex flex-col gap-2">
              <p className="font-sans text-xs font-bold tracking-widest text-muted-foreground uppercase">
                {organization.type} · {organization.city}/{organization.uf}
              </p>
              <h1 className="font-display text-4xl leading-none tracking-wide text-foreground md:text-5xl">
                {organization.name}
              </h1>
            </header>

            <p className="mt-6 font-sans text-base leading-relaxed text-muted-foreground">
              {organization.about}
            </p>

            <div className="mt-10 overflow-hidden rounded-xl border border-border">
              <MapContainer
                center={[organization.latitude, organization.longitude]}
                zoom={16}
                dragging={false}
                zoomControl={false}
                scrollWheelZoom={false}
                doubleClickZoom={false}
                className="h-70 w-full bg-neutral-900 [&_.leaflet-tile]:bg-neutral-900 [&_.leaflet-tile-pane]:bg-neutral-900"
              >
                <TileLayer
                  attribution='Imagery &copy; <a href="https://www.mapbox.com/">Mapbox</a>'
                  url={`https://api.mapbox.com/styles/v1/${env.VITE_USERNAME}/${env.VITE_STYLE_ID}/tiles/256/{z}/{x}/{y}@2x?access_token=${env.VITE_ACCESS_TOKEN}`}
                />
                <Marker
                  interactive={false}
                  icon={mapIcon}
                  position={[organization.latitude, organization.longitude]}
                />
              </MapContainer>

              <footer className="border-t border-border py-4 text-center">
                <a
                  target="_blank"
                  rel="noopener noreferrer"
                  href={`https://www.google.com/maps/dir/?api=1&destination=${organization.latitude},${organization.longitude}`}
                  className="font-sans text-sm font-semibold text-foreground underline-offset-4 hover:underline"
                >
                  Ver rotas no Google Maps
                </a>
              </footer>
            </div>

            <hr className="my-10 h-px border-0 bg-border" />

            <div className="flex flex-col gap-4 sm:flex-row">
              {socialUrl && (
                <Button asChild variant="outline" className="flex-1">
                  <a href={socialUrl} target="_blank" rel="noopener noreferrer">
                    Ver redes sociais
                  </a>
                </Button>
              )}

              <Button asChild className="flex-1">
                <a href={`mailto:${organization.email}`}>Entrar em contato</a>
              </Button>
            </div>
          </div>
        </article>
      </main>
    </div>
  );
}

export default Organization;
