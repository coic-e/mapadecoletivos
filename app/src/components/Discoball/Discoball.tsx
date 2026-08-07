import React from "react";
import { Link } from "react-router-dom";

import "../styles/components/discoball.css";

function Discoball() {
  return (
    <div className="conteiner disco">
      <div id="discoBallLight"></div>
      <div id="discoBall">
        <div id="discoBallMiddle"></div>
      </div>
      <Link to="https://timeconverter.ca">
        <img
          alt="Unix Time Converter"
          src="https://timeconverter.ca/images/tclogo-mini-white.png"
          width="49"
          height="20"
        />
      </Link>
    </div>
  );
}

export default Discoball;
