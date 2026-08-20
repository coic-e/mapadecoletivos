import { useState, useEffect, useCallback } from "react";
import { Link } from "react-router-dom";
import { FiPlus, FiArrowRight } from "react-icons/fi";
import { MapContainer, TileLayer, Marker, Popup, useMap } from "react-leaflet";
import Leaflet from "leaflet";

import { env } from "@/config/env";
import api from "@/services/api";
import mapMarkerImg from "@/images/map-marker.svg";
import { useSeo } from "@/hooks/useSeo";

interface Organization {
  id: number;
  latitude: number;
  longitude: number;
  name: string;
}

const createScaledIcon = (zoom: number) => {
  // O marcador cresce junto com o zoom, a partir do nível 5.
  const baseZoom = 5;
  const baseSize = 30;
  const scale = Math.pow(1.5, zoom - baseZoom);
  const size = Math.max(20, Math.min(60, baseSize * scale));

  return Leaflet.icon({
    iconUrl: mapMarkerImg,
    iconSize: [size, size],
    iconAnchor: [size - 1, size - 1],
    popupAnchor: [165, 30],
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
    title: "Mapa dos coletivos de música eletrônica do Brasil | Mapa de Rave",
    description:
      "Navegue pelo mapa e descubra festas, coletivos, labels, clubs, rádios e produtoras de música eletrônica espalhados pelo Brasil.",
    path: "/raves",
  });
  const [mapIcon, setMapIcon] = useState(createScaledIcon(5.1));

  useEffect(() => {
    api.get("organizations").then((response) => {
      const organizations = response.data;

      if (organizations) {
        setOrganizations(organizations);
      }
    });
  }, []);

  const handleZoomChange = useCallback((newZoom: number) => {
    setMapIcon(createScaledIcon(newZoom));
  }, []);

  return (
    <div className="relative flex h-dvh w-screen flex-col bg-background md:flex-row">
      <aside className="flex w-full flex-col justify-center gap-6 bg-background p-6 md:min-w-80 md:max-w-90 md:justify-center md:p-10 lg:min-w-95 lg:max-w-110">
        <header className="flex flex-col items-center gap-4 text-center md:items-start md:text-left">
          <h2 className="font-display text-[clamp(24px,4vw,34px)] leading-tight tracking-wide text-foreground uppercase">
            Coletivos de música eletrônica no Brasil
          </h2>
          <p className="font-sans text-[clamp(13px,2.5vw,16px)] leading-snug text-muted-foreground">
            Você sabia que são mais de 260 atores que compõem nosso cenário?
          </p>
        </header>
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
          url={`https://api.mapbox.com/styles/v1/${env.VITE_USERNAME}/${env.VITE_STYLE_ID}/tiles/256/{z}/{x}/{y}@2x?access_token=${env.VITE_ACCESS_TOKEN}`}
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
                to={`/raves/${organization.id}`}
                aria-label={`Ver ${organization.name}`}
                className="flex size-10 min-w-10 items-center justify-center rounded-md bg-primary text-primary-foreground shadow-md transition-transform hover:scale-105"
              >
                <FiArrowRight size={20} />
              </Link>
            </Popup>
          </Marker>
        ))}
      </MapContainer>

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
