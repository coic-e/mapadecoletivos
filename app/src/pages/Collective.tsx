import React, { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { MapContainer, TileLayer, Marker } from "react-leaflet";
const { VITE_USERNAME, VITE_STYLE_ID, VITE_ACCESS_TOKEN } = import.meta.env;
import "leaflet/dist/leaflet.css";

import "../styles/pages/collective.css";
import Sidebar from "../components/Sidebar";
import mapIcon from "../utils/mapIcon";
import api from "../services/api";

interface ICollective {
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

interface CollectiveParams {
  id: string;
  [key: string]: string | undefined;
}


function Collective() {


  const params = useParams<CollectiveParams>();
  // const [collective, setCollective] = useState<ICollective>();
  const [activeImageIndex, setActiveImageIndex] = useState(0);

  const collective: ICollective = {
    id: 1,
    latitude: -13.702797,
    longitude: -50.6865109,
    name: "Teste",
    type: "Coletivo",
    about: "Teste",
    email: "teste@teste.com",
    social: "teste",
    city: "teste",
    uf: "teste",
    images: [
      {
        id: 1,
        url: "https://images.unsplash.com/photo-1611095782547-0b3b9b2c1d6b?ixid=MnwxMjA3fDB8MHxzZWFyY2h8Mnx8Y29sZXRpdmV8ZW58MHx8MHx8&ixlib=rb-1.2.1&w=1000&q=80",
      },
      {
        id: 2,
        url: "https://images.unsplash.com/photo-1611095782547-0b3b9b2c1d6b?ixid=MnwxMjA3fDB8MHxzZWFyY2h8Mnx8Y29sZXRpdmV8ZW58MHx8MHx8&ixlib=rb-1.2.1&w=1000&q=80",
      },
    ],
  };
  // useEffect(() => {
  //   api
  //     .get(`collectives/${params.id}`)
  //     .then((response) => {
  //       const collective = response.data;
  //       if (collective) {
  //         setCollective(collective);
  //       }
  //     })
  //     .catch((error) => {
  //       console.error("Error fetching collective:", error);
  //       // You can set some state to show an error message to the user
  //     });
  // }, [params.id]);

  if (!collective) {
    return <h4>Buscando informações do coletivo...</h4>;
  }

  return (
    <div id="page-collective">
      <Sidebar />

      {collective.id ? (
        <main>
          <div className="collective-details">
            <img
              src={collective.images[activeImageIndex].url}
              alt="Coletivo Teste"
            />

            <div className="images">
              {collective.images &&
                collective.images.map((image, index) => {
                  return (
                    <button
                      className={activeImageIndex === index ? "active" : ""}
                      type="button"
                      key={image.id}
                      onClick={() => {
                        setActiveImageIndex(index);
                      }}
                    >
                      <img src={image.url} alt="Coletivo Teste" />
                    </button>
                  );
                })}
            </div>

            <div className="collective-details-content">
              <h1>{collective.name}</h1>
              <p>{collective.about}</p>

              <div className="map-container">
                <MapContainer
                  center={[collective.latitude, collective.longitude]}
                  zoom={16}
                  style={{ width: "100%", height: 280 }}
                  dragging={false}
                  zoomControl={false}
                  scrollWheelZoom={false}
                  doubleClickZoom={false}
                >
                  <TileLayer
                    attribution='Imagery &copy; <a href="https://www.mapbox.com/">Mapbox</a>'
                    url={`https://api.mapbox.com/styles/v1/${VITE_USERNAME}/${VITE_STYLE_ID}/tiles/256/{z}/{x}/{y}@2x?access_token=${VITE_ACCESS_TOKEN}`}
                  />
                  <Marker
                    interactive={false}
                    icon={mapIcon}
                    position={[collective.latitude, collective.longitude]}
                  ></Marker>
                </MapContainer>

                <footer>
                  <a
                    target="_blank"
                    rel="noopener noreferrer"
                    href={`https://www.google.com/maps/dir/?api=1&destination=${collective.latitude},${collective.longitude}`}
                  >
                    Ver rotas no Google Maps
                  </a>
                </footer>
              </div>

              <hr />

              <h2>
                <a href={collective.social}>Social links</a>
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

export default Collective;
