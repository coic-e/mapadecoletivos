import React, { useState, useEffect, useCallback } from "react";
import { Link } from "react-router-dom";
import { FiPlus, FiArrowRight } from "react-icons/fi";
import { MapContainer, TileLayer, Marker, Popup, useMap } from "react-leaflet";
import Leaflet from "leaflet";
import "leaflet/dist/leaflet.css";

import { env } from "../config/env";
import api from "../services/api";

import "../styles/pages/organizations-map.css";
import mapMarkerImg from "../images/map-marker.svg";

interface Organization {
  id: number;
  latitude: number;
  longitude: number;
  name: string;
}

// Helper function to create icon based on zoom level
const createScaledIcon = (zoom: number) => {
  // Base size at zoom level 5, scale from there
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

// Component to handle zoom changes
function ZoomHandler({ onZoomChange }: { onZoomChange: (zoom: number) => void }) {
  const map = useMap();

  useEffect(() => {
    const handleZoom = () => {
      onZoomChange(map.getZoom());
    };

    map.on("zoomend", handleZoom);
    // Set initial zoom
    onZoomChange(map.getZoom());

    return () => {
      map.off("zoomend", handleZoom);
    };
  }, [map, onZoomChange]);

  return null;
}

function OrganizationsMap() {
  const [organizations, setOrganizations] = useState<Organization[]>([]);
  const [zoom, setZoom] = useState(5.1);
  const [mapIcon, setMapIcon] = useState(createScaledIcon(5.1));

  useEffect(() => {
    api.get("organizations").then((response) => {
      const organizations = response.data;

      if (organizations) {
        setOrganizations(organizations);
      }
    });
  }, []);

  // Update icon when zoom changes
  const handleZoomChange = useCallback((newZoom: number) => {
    setZoom(newZoom);
    setMapIcon(createScaledIcon(newZoom));
  }, []);

  return (
    <div id="page-map">
      <aside>
        <header>
          <h2>Coletivos de música eletrônica no Brasil</h2>
          <p>Você sabia que são mais de 260 atores que compõem nosso cenário?</p>
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
        style={{
          width: "100%",
          height: "100%",
        }}
      >
        <ZoomHandler onZoomChange={handleZoomChange} />
        <TileLayer
          attribution='Imagery &copy; <a href="https://www.mapbox.com/">Mapbox</a>'
          url={`https://api.mapbox.com/styles/v1/${env.VITE_USERNAME}/${env.VITE_STYLE_ID}/tiles/256/{z}/{x}/{y}@2x?access_token=${env.VITE_ACCESS_TOKEN}`}
          noWrap={true}
        />
        {organizations.map((organization) => {
          return (
            <Marker
              icon={mapIcon}
              position={[organization.latitude, organization.longitude]}
              key={organization.id}
            >
              <Popup closeButton={false} minWidth={240} maxWidth={240} className="map-popup">
                {organization.name}
                <Link to={`/raves/${organization.id}`}>
                  <FiArrowRight size={20} color="#FFF" />
                </Link>
              </Popup>
            </Marker>
          );
        })}
      </MapContainer>

      <Link to="/raves/create" className="create-organization">
        <FiPlus size={32} color="#FFF" />
      </Link>
    </div>
  );
}

export default OrganizationsMap;
