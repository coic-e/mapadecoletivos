import styled from "styled-components";

export const Container = styled.div`
  position: relative;
  display: flex;

  background: linear-gradient(
    180deg,
    var(--primary-color) 0%,
    var(--secondary-color) 100%
  );

  .leaflet-container {
    z-index: 5;
  }

  .map-popup .leaflet-popup-content-wrapper {
    background: rgba(255, 255, 255, 0.8);
    border-radius: 20px;
    box-shadow: none;
  }

  .map-popup .leaflet-popup-content {
    color: var(--text-color);
    font-size: 20px;
    margin: 8px 12px;
    font-weight: bold;

    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .map-popup .leaflet-popup-content a {
    width: 40px;
    height: 40px;
    background: var(--primary-color);
    box-shadow: 17.2868px 27.6589px 41.4884px rgba(23, 142, 166, 0.16);
    border-radius: 12px;

    display: flex;
    justify-content: center;
    align-items: center;
  }

  .map-popup .leaflet-popup-tip-container {
    display: none;
  }
`;

export const Aside = styled.aside`
  width: 495px;
  padding: 5rem;

  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: 20px;

  background: url("../../images/discoball.svg") no-repeat center top;
  background-size: 214px 224px;
`;

export const Header = styled.header`
  h2 {
    font-size: 2.25rem;
    font-weight: 600;
    line-height: 2.625rem;
    color: #fff;
  }

  p {
    font-size: 1.125rem;
    line-height: 1.625rem;
    color: #fff;
  }
`;

export const Button = styled.div`
  display: flex;
  align-items: center;
  justify-content: center;

  width: 48px;
  height: 48px;
  background: #ffd666;
  border-radius: 16px;

  position: absolute;
  bottom: 40px;
  right: 40px;
  z-index: 10;

  background: var(--hover-color);
`;
