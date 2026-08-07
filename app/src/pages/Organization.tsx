import React, { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { MapContainer, TileLayer, Marker } from "react-leaflet";
import "leaflet/dist/leaflet.css";

import { env } from "../config/env";

import "../styles/pages/organization.css";
import Sidebar from "../components/Sidebar";
import mapIcon from "../utils/mapIcon";
import api from "../services/api";

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

function Organization() {
  const params = useParams<OrganizationParams>();
  const [organization, setOrganization] = useState<IOrganization>();
  const [activeImageIndex, setActiveImageIndex] = useState(0);

  useEffect(() => {
    api
      .get(`organizations/${params.id}`)
      .then((response) => {
        const organization = response.data;
        if (organization) {
          setOrganization(organization);
        }
      })
      .catch((error) => {
        console.error("Error fetching organization:", error);
        // You can set some state to show an error message to the user
      });
  }, [params.id]);

  if (!organization) {
    return <h4>Buscando informações da organização...</h4>;
  }

  return (
    <div id="page-organization">
      <Sidebar />

      {organization.id ? (
        <main>
          <div className="organization-details">
            <img src={organization.images[activeImageIndex].url} alt="Organização" />

            <div className="images">
              {organization.images &&
                organization.images.map((image, index) => {
                  return (
                    <button
                      className={activeImageIndex === index ? "active" : ""}
                      type="button"
                      key={image.id}
                      onClick={() => {
                        setActiveImageIndex(index);
                      }}
                    >
                      <img src={image.url} alt="Organização" />
                    </button>
                  );
                })}
            </div>

            <div className="organization-details-content">
              <h1>{organization.name}</h1>
              <p>{organization.about}</p>

              <div className="map-container">
                <MapContainer
                  center={[organization.latitude, organization.longitude]}
                  zoom={16}
                  style={{ width: "100%", height: 280 }}
                  dragging={false}
                  zoomControl={false}
                  scrollWheelZoom={false}
                  doubleClickZoom={false}
                >
                  <TileLayer
                    attribution='Imagery &copy; <a href="https://www.mapbox.com/">Mapbox</a>'
                    url={`https://api.mapbox.com/styles/v1/${env.VITE_USERNAME}/${env.VITE_STYLE_ID}/tiles/256/{z}/{x}/{y}@2x?access_token=${env.VITE_ACCESS_TOKEN}`}
                  />
                  <Marker
                    interactive={false}
                    icon={mapIcon}
                    position={[organization.latitude, organization.longitude]}
                  ></Marker>
                </MapContainer>

                <footer>
                  <a
                    target="_blank"
                    rel="noopener noreferrer"
                    href={`https://www.google.com/maps/dir/?api=1&destination=${organization.latitude},${organization.longitude}`}
                  >
                    Ver rotas no Google Maps
                  </a>
                </footer>
              </div>

              <hr />

              <h2>
                <a href={organization.social}>Social links</a>
              </h2>
              <button type="button" className="contact-button">
                Entrar em contato
              </button>
            </div>
          </div>
        </main>
      ) : (
        <p>Carregando...</p>
      )}
    </div>
  );
}

export default Organization;
