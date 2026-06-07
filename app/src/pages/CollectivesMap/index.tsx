import React, { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { FiPlus, FiArrowRight } from "react-icons/fi";
import { MapContainer, TileLayer, Marker, Popup } from "react-leaflet";
const { VITE_USERNAME, VITE_STYLE_ID, VITE_ACCESS_TOKEN } = import.meta.env;
import "leaflet/dist/leaflet.css";
import mapIcon from "../../utils/mapIcon";
import api from "../../services/api";

interface Collective {
  id: number;
  latitude: number;
  longitude: number;
  name: string;
}

function CollectivesMap() {
  // const [collectives, setCollectives] = useState<Collective[]>([]);
  const collectives: Collective[] = [
    {
      id: 1,
      latitude: -13.702797,
      longitude: -50.6865109,
      name: "Teste",
    },
    {
      id: 2,
      latitude: -13.702797,
      longitude: -50.6865109,
      name: "Teste",
    },
  ];

  // useEffect(() => {
  //   api.get("collectives").then((response) => {
  //     const collectives = response.data;

  //     if (collectives) {
  //       setCollectives(collectives);
  //     }
  //   });
  // }, []);

  return (
    <div id="page-map">
      <aside>
        <header>
          <h2>Coletivos de música eletrônica no Brasil</h2>
          <p>
            Você sabia que são mais de 260 atores que compõem nosso cenário?
          </p>
        </header>
        {/* <footer>
          <strong>Porto Alegre</strong>
          <br />
          <span>Rio Grande do Sul</span>
        </footer> */}
      </aside>

      <MapContainer
        center={[-13.702797, -50.6865109]}
        zoom={5.1}
        style={{
          width: "100%",
          height: "100%",
        }}
      >
        <TileLayer
          attribution='Imagery &copy; <a href="https://www.mapbox.com/">Mapbox</a>'
          url={`https://api.mapbox.com/styles/v1/${VITE_USERNAME}/${VITE_STYLE_ID}/tiles/256/{z}/{x}/{y}@2x?access_token=${VITE_ACCESS_TOKEN}`}
        />
        {collectives.map((collective) => {
          return (
            <Marker
              icon={mapIcon}
              position={[collective.latitude, collective.longitude]}
              key={collective.id}
            >
              <Popup
                closeButton={false}
                minWidth={240}
                maxWidth={240}
                className="map-popup"
              >
                {collective.name}
                <Link to={`/collectives/${collective.id}`}>
                  <FiArrowRight size={20} color="#FFF" />
                </Link>
              </Popup>
            </Marker>
          );
        })}
      </MapContainer>

      <Link to="/raves/create" className="create-collective">
        <FiPlus size={32} color="#FFF" />
      </Link>
    </div>
  );
}

export default CollectivesMap;
