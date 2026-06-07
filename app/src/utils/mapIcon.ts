import Leaflet from "leaflet";
import mapMarkerImg from "../images/map-marker.svg";
const mapIcon = Leaflet.icon({
  iconUrl: mapMarkerImg,
  iconSize: [30, 30],
  iconAnchor: [29, 29],
  popupAnchor: [165, 30],
});

export default mapIcon;
