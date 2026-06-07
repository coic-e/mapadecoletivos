import {
  DefaultTheme,
  createGlobalStyle,
  css,
  IStyledComponent,
} from "styled-components";

const GlobalStyles = createGlobalStyle`
@import url("https://rsms.me/inter/inter.css");
@font-face {
font-family: "Ayer Poster";
src: local(""),
  url("/AyerPoster-Medium.woff2") format("woff2"),
  url("/AyerPoster-Medium.woff") format("woff"),
  url("/AyerPoster-Medium.ttf") format("truetype");
font-weight: 500;
font-style: normal;
}

*{
  margin: 0;
  padding: 0;
  outline:0;
  box-sizing:border-box;
  font-family: 'Open Sans', sans-serif; 
}

html,
body,
#root{
  width: 100%;
  height: 100%;
  margin:0 auto;
}
a {
color: black;
}

input[type="range"] {
  display: block;
  -webkit-appearance: none;
  -moz-appearance: none;
  appearance: none;
  background: black;
  border-radius: 5px;
  width: 100%;
  height: 2px;
  outline: 0;
}

input[type="range"]::-webkit-slider-thumb {
  -webkit-appearance: none;
  background-color: #000;
  width: 25px;
  height: 25px;
  border-radius: 50%;
  cursor: pointer;
  transition: 0.3s ease-in-out;
}

input[type="range"]::-webkit-slider-thumb:active {
  transform: scale(1);
}

#gradient-container {
  width: 100vw;
  height: 100vh;
  position: fixed;
  top: 0;
  left: 0;
  /* z-index: -1; */
}
 `;

export default GlobalStyles;
