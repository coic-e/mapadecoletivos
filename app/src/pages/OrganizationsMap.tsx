import { useState, useEffect, useCallback } from "react";
import { Link } from "react-router-dom";
import { FiPlus, FiArrowRight } from "react-icons/fi";
import { MapContainer, TileLayer, Marker, Popup, useMap } from "react-leaflet";
import Leaflet from "leaflet";

import { env } from "@/config/env";
import api from "@/services/api";
import mapMarkerImg from "@/images/map-marker.svg";
import { MARKER_RATIO } from "@/utils/mapIcon";
import { useSeo } from "@/hooks/useSeo";
import SeedDataNotice from "@/components/SeedDataNotice";

interface Organization {
  id: number;
  slug: string;
  latitude: number;
  longitude: number;
  name: string;
}

const MIN_ZOOM = 3;
const MAX_ZOOM = 18;
const MIN_MARKER_WIDTH = 18;
const MAX_MARKER_WIDTH = 44;

/**
 * Largura do marcador para o zoom atual, interpolada linearmente entre os
 * extremos do mapa.
 *
 * O cálculo anterior crescia 1,5× por nível de zoom, o que estourava o teto no
 * zoom 7: dos 15 níveis disponíveis, o tamanho só variava em dois, e nos
 * outros treze ficava preso no mínimo ou no máximo.
 */
const markerWidthFor = (zoom: number) => {
  const progress = (zoom - MIN_ZOOM) / (MAX_ZOOM - MIN_ZOOM);
  const clamped = Math.min(1, Math.max(0, progress));

  // Arredondado para múltiplos de 2px: o ícone só é recriado quando a mudança
  // é visível, e não a cada fração de zoom.
  return Math.round((MIN_MARKER_WIDTH + (MAX_MARKER_WIDTH - MIN_MARKER_WIDTH) * clamped) / 2) * 2;
};

const createScaledIcon = (zoom: number) => {
  const width = markerWidthFor(zoom);
  const height = width * MARKER_RATIO;

  return Leaflet.icon({
    iconUrl: mapMarkerImg,
    iconSize: [width, height],
    // A ponta do pin, embaixo e no meio: é ela que aponta a coordenada.
    iconAnchor: [width / 2, height],
    // Acima da ponta, para o balão não cobrir o próprio marcador.
    popupAnchor: [0, -height],
  });
};

function ZoomHandler({ onZoomChange }: { onZoomChange: (zoom: number) => void }) {
  const map = useMap();

  useEffect(() => {
    const handleZoom = () => {
      onZoomChange(map.getZoom());
    };

    map.on("zoomend", handleZoom);
    onZoomChange(map.getZoom());

    return () => {
      map.off("zoomend", handleZoom);
    };
  }, [map, onZoomChange]);

  return null;
}

function OrganizationsMap() {
  const [organizations, setOrganizations] = useState<Organization[]>([]);

  useSeo({
    title: "Mapa de Rave | Mapa de música eletrônica do Brasil",
    description:
      "Navegue pelo mapa e descubra festas, coletivos, labels, clubs, rádios e produtoras de música eletrônica espalhados pelo Brasil.",
    path: "/raves",
  });
  const [mapIcon, setMapIcon] = useState(createScaledIcon(5.1));

  const [loadError, setLoadError] = useState(false);

  useEffect(() => {
    api
      .get("organizations")
      .then((response) => {
        if (response.data) {
          setOrganizations(response.data);
        }
      })
      // Sem isto, API fora do ar virava mapa vazio sem explicação — e mapa
      // vazio parece "não há coletivos", não "não consegui carregar".
      .catch(() => setLoadError(true));
  }, []);

  const handleZoomChange = useCallback((newZoom: number) => {
    setMapIcon(createScaledIcon(newZoom));
  }, []);

  return (
    <div className="relative flex h-dvh w-screen flex-col bg-background">
      <SeedDataNotice />

      <div className="flex flex-1 flex-col overflow-hidden md:flex-row">
        <aside className="flex w-full flex-col justify-center gap-6 bg-background p-6 md:min-w-80 md:max-w-90 md:justify-center md:p-10 lg:min-w-95 lg:max-w-110">
          <header className="flex flex-col items-center gap-4 text-center md:items-start md:text-left">
            <h2 className="font-display text-[clamp(24px,4vw,34px)] leading-tight tracking-wide text-foreground uppercase">
              Coletivos de música eletrônica no Brasil
            </h2>
            <p className="font-sans text-[clamp(13px,2.5vw,16px)] leading-snug text-muted-foreground">
              Você sabia que são mais de 260 atores que compõem nosso cenário?
            </p>
          </header>

          {loadError && (
            <p
              role="alert"
              className="rounded-md border border-solid border-destructive/40 bg-destructive/10 p-3 font-sans text-sm text-destructive"
            >
              Não consegui carregar os coletivos. Verifique se a API está no ar e recarregue a
              página.
            </p>
          )}
        </aside>

        <MapContainer
          center={[-13.702797, -50.6865109]}
          zoom={5.1}
          minZoom={3}
          maxZoom={18}
          worldCopyJump={false}
          maxBounds={[
            [-90, -180],
            [90, 180],
          ]}
          maxBoundsViscosity={1.0}
          className="z-5 min-h-100 w-full flex-1 bg-neutral-900 [&_.leaflet-tile]:bg-neutral-900 [&_.leaflet-tile-pane]:bg-neutral-900"
        >
          <ZoomHandler onZoomChange={handleZoomChange} />
          <TileLayer
            attribution='Imagery &copy; <a href="https://www.mapbox.com/">Mapbox</a>'
            url={`https://api.mapbox.com/styles/v1/${env.VITE_MAPBOX_USERNAME}/${env.VITE_MAPBOX_STYLE_ID}/tiles/256/{z}/{x}/{y}@2x?access_token=${env.VITE_MAPBOX_ACCESS_TOKEN}`}
            noWrap={true}
          />

          {organizations.map((organization) => (
            <Marker
              icon={mapIcon}
              position={[organization.latitude, organization.longitude]}
              key={organization.id}
            >
              <Popup
                closeButton={false}
                minWidth={240}
                maxWidth={240}
                className="[&_.leaflet-popup-content-wrapper]:rounded-lg [&_.leaflet-popup-content-wrapper]:bg-card/95 [&_.leaflet-popup-content-wrapper]:shadow-xl [&_.leaflet-popup-content-wrapper]:backdrop-blur-sm [&_.leaflet-popup-content]:m-0 [&_.leaflet-popup-content]:flex [&_.leaflet-popup-content]:items-center [&_.leaflet-popup-content]:justify-between [&_.leaflet-popup-content]:gap-4 [&_.leaflet-popup-content]:px-4 [&_.leaflet-popup-content]:py-3 [&_.leaflet-popup-content]:font-sans [&_.leaflet-popup-content]:text-base [&_.leaflet-popup-content]:font-semibold [&_.leaflet-popup-content]:text-card-foreground [&_.leaflet-popup-tip-container]:hidden"
              >
                {organization.name}

                <Link
                  to={`/raves/${organization.slug}`}
                  aria-label={`Ver ${organization.name}`}
                  className="flex size-10 min-w-10 items-center justify-center rounded-md bg-primary text-primary-foreground shadow-md transition-transform hover:scale-105"
                >
                  <FiArrowRight size={20} />
                </Link>
              </Popup>
            </Marker>
          ))}
        </MapContainer>
      </div>

      <Link
        to="/raves/create"
        aria-label="Cadastrar rolê"
        className="absolute right-5 bottom-5 z-10 flex size-13 items-center justify-center rounded-full border-2 border-primary bg-primary text-primary-foreground shadow-lg transition-all hover:-translate-y-0.5 hover:bg-background hover:text-foreground md:right-10 md:bottom-10 md:size-16"
      >
        <FiPlus size={28} />
      </Link>
    </div>
  );
}

export default OrganizationsMap;
