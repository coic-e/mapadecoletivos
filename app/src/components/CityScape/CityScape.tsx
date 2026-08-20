import { useEffect, useRef } from "react";

import { createCityScape } from "./cityscape-engine";

/**
 * Cidade procedural infinita em WebGL2, usada como plano de fundo.
 * Baseado no pen "Endless city" de Niklas Knaack.
 */
const CityScape = () => {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const container = containerRef.current;

    if (!container) return;

    const cityScape = createCityScape(container);

    return () => {
      cityScape.destroy();
    };
  }, []);

  return (
    <div className="absolute inset-0 overflow-hidden bg-black">
      <div
        ref={containerRef}
        aria-hidden="true"
        className="size-full [&>canvas]:block [&>canvas]:size-full [&>canvas]:touch-none"
      />
    </div>
  );
};

export default CityScape;
