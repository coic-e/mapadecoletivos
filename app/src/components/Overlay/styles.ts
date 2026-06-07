import styled, { keyframes } from "styled-components";

export const fade = keyframes`
  from { opacity: 1; }
  to { opacity: 0; }
`;

export const FadeIn = styled.div`
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
  background: #f103e5;
  animation: ${fade} 4s normal forwards ease-in-out;
`;

export const Container = styled.div`
  align-items: center;
  font-family: "Inter";
  font-size: 16px;
  & h1 {
    padding: 0;
    margin: 0 0 0.05em 0;
    font-family: "Ayer Poster", serif;
    font-weight: 400;
    font-size: min(18vw, 20em);
    line-height: 0.85em;
  }

  & p {
    font-family: "Ayer Poster", serif;
    margin: 0;
    font-weight: 400;
    font-size: 2em;
    line-height: 1.5em;
  }
`;

export const TopLeft = styled.div`
  position: absolute;
  top: 5vw;
  left: 5vw;
`;

export const BottomLeft = styled.div`
  position: absolute;
  bottom: 5vw;
  left: 5vw;
  width: 30ch;
  max-width: 40%;
`;

export const BottomRight = styled.div`
  display: flex;
  flex-direction: column;
  position: absolute;
  bottom: 5vw;
  right: 5vw;
  width: 35ch;
  max-width: 40%;
  line-height: 1em;
  letter-spacing: -0.01em;
  text-align: right;
`;

export const Hamburger = styled.div`
  position: absolute;
  display: flex;
  flex-direction: column;
  top: 5vw;
  right: 5vw;
  & > div {
    position: relative;
    width: 24px;
    height: 2px;
    background: #252525;
    margin-bottom: 6px;
  }
`;

export const LeftMiddle = styled.div`
  position: absolute;
  bottom: 50%;
  right: 5vw;
  font-family: "Inter";
  font-weight: 400;
  line-height: 1em;
  letter-spacing: -0.01em;
  font-size: 12px;
  transform: rotate(90deg) translate3d(50%, 0, 0);
  transform-origin: 100% 50%;
`;

export const TopCenter = styled.div`
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;

  text-align: center;
  position: absolute;
  top: 5vw;
  left: 50%;
  transform: translateX(-50%);
  font-family: "Inter";
  font-weight: 400;
  line-height: 1em;
  letter-spacing: -0.01em;
  font-size: 12px;
`;

export const BottomCenter = styled.div`
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  position: absolute;
  bottom: 5vw;
  left: 50%;
  transform: translateX(-50%);
  font-family: "Inter";
  font-weight: 400;
  line-height: 1em;
  letter-spacing: -0.01em;
  font-size: 12px;
`;

export const Button = styled.div`
  width: 80px;
  height: 80px;
  margin-top: 40px;
  background: #fff;
  border-radius: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background-color 0.2s, transform 0.2s;
  box-shadow: 0px 4px 8px rgba(0, 0, 0, 0.2);

  &:hover {
    background: #c7a7a7;
    transform: scale(1.05);
  }
`;
