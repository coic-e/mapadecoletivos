import Leaflet from "leaflet";

import mapMarkerImg from "../images/map-marker.svg";

/** Proporção do desenho: a ponta do pin fica embaixo, no meio. */
export const MARKER_RATIO = 40 / 32;

/**
 * Marcador de tamanho fixo, para os mapas que mostram um ponto só.
 *
 * A âncora é a ponta do pin — meio da largura, base da altura. Ancorar pelo
 * canto, como estava, faz o marcador apontar para um lugar que não é o
 * cadastrado, e o desvio muda junto com o tamanho.
 */
const SIZE = 32;

const mapIcon = Leaflet.icon({
  iconUrl: mapMarkerImg,
  iconSize: [SIZE, SIZE * MARKER_RATIO],
  iconAnchor: [SIZE / 2, SIZE * MARKER_RATIO],
  popupAnchor: [0, -SIZE * MARKER_RATIO],
});

export default mapIcon;
