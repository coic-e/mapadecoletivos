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

      <a
        href="https://www.niklasknaack.de/"
        target="_blank"
        rel="noopener noreferrer"
        className="absolute right-4 bottom-4 z-20 font-sans text-xs tracking-wide text-white/45 no-underline transition-colors hover:text-white/90"
      >
        cidade 3D por Niklas Knaack
      </a>
    </div>
  );
};

export default CityScape;
