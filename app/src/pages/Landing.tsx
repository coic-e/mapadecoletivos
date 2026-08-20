import { useEffect, useRef } from "react";
import { FiArrowRight } from "react-icons/fi";
import { Link } from "react-router-dom";

import CityScape from "@/components/CityScape";
import { useSeo } from "@/hooks/useSeo";

const TITLE = "MAPA DE RAVE";
const SCRAMBLE_CHARS = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789@#$%^&*";
const SCRAMBLE_DURATION = 2000;

function Landing() {
  const titleRef = useRef<HTMLHeadingElement>(null);

  useSeo({
    title: "Mapa de Rave — coletivos, festas e labels de música eletrônica no Brasil",
    description:
      "O mapa da música eletrônica brasileira: encontre coletivos, festas, labels, clubs e produtoras perto de você, ou coloque o seu rolê no mapa.",
    path: "/",
  });

  useEffect(() => {
    const title = titleRef.current;

    if (!title) return;

    let frame: number;
    let start: number | null = null;

    const scramble = (timestamp: number) => {
      if (start === null) {
        start = timestamp;
      }

      const progress = Math.min((timestamp - start) / SCRAMBLE_DURATION, 1);

      title.textContent = TITLE.split("")
        .map((char, index) => {
          if (char === " ") return " ";

          // Revela os caracteres da esquerda para a direita.
          const charProgress = index / TITLE.length;

          if (progress > charProgress + 0.2) return char;

          if (progress < charProgress - 0.2) {
            return SCRAMBLE_CHARS[Math.floor(Math.random() * SCRAMBLE_CHARS.length)];
          }

          return Math.random() > 0.5
            ? char
            : SCRAMBLE_CHARS[Math.floor(Math.random() * SCRAMBLE_CHARS.length)];
        })
        .join("");

      if (progress < 1) {
        frame = requestAnimationFrame(scramble);
      } else {
        title.textContent = TITLE;
      }
    };

    frame = requestAnimationFrame(scramble);

    return () => {
      cancelAnimationFrame(frame);
    };
  }, []);

  return (
    <div className="relative flex h-dvh w-screen items-center justify-center overflow-hidden bg-black text-center">
      <CityScape />

      {/* pointer-events-none deixa o mouse chegar no canvas e mover a cidade. */}
      <div className="pointer-events-none relative z-10 flex max-w-full flex-col items-center gap-10 p-8">
        <main className="flex flex-col items-center gap-6">
          <h1
            ref={titleRef}
            className="m-0 font-display text-[clamp(32px,12vw,80px)] leading-none tracking-wider break-words text-white drop-shadow-[0_4px_24px_rgba(0,0,0,0.85)]"
          >
            {TITLE}
          </h1>

          <p className="m-0 max-w-[90%] font-sans text-[clamp(14px,4vw,20px)] leading-snug font-semibold text-white/80 drop-shadow-[0_2px_12px_rgba(0,0,0,0.85)]">
            Descubra a Batida do Underground
          </p>
        </main>

        <Link
          to="/raves"
          className="group pointer-events-auto inline-flex items-center gap-3 rounded-full border border-white/35 bg-white/8 px-7 py-3.5 font-display text-lg tracking-[0.18em] whitespace-nowrap text-white uppercase no-underline backdrop-blur-md transition-colors hover:border-white/70 hover:bg-white/16 sm:px-6 sm:py-3"
        >
          <span>Entrar no mapa</span>
          <FiArrowRight
            size={20}
            className="transition-transform duration-300 group-hover:translate-x-1"
          />
        </Link>
      </div>
    </div>
  );
}

export default Landing;
