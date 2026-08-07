import React, { useEffect, useRef } from "react";
import { FiArrowRight } from "react-icons/fi";
import { Link } from "react-router-dom";
import gsap from "gsap";
import HorseAnimation from "../components/HorseAnimation";

import "../styles/pages/landing.css";

function Landing() {
  const titleRef = useRef<HTMLHeadingElement>(null);

  useEffect(() => {
    if (!titleRef.current) return;

    const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789@#$%^&*";
    const originalText = "MAPA DE RAVE";
    let currentText = originalText;

    const scramble = () => {
      const timeline = gsap.timeline();
      const duration = 2;
      const steps = 20;

      for (let i = 0; i <= steps; i++) {
        timeline.to(
          {},
          {
            duration: duration / steps,
            onStart: () => {
              if (!titleRef.current) return;
              
              const progress = i / steps;
              const scrambledText = originalText
                .split("")
                .map((char, index) => {
                  if (char === " ") return " ";
                  
                  // Gradually reveal characters from left to right
                  const charProgress = index / originalText.length;
                  if (progress > charProgress + 0.2) {
                    return char;
                  } else if (progress < charProgress - 0.2) {
                    return chars[Math.floor(Math.random() * chars.length)];
                  } else {
                    // In the transition zone, randomly show correct char
                    return Math.random() > 0.5 
                      ? char 
                      : chars[Math.floor(Math.random() * chars.length)];
                  }
                })
                .join("");
              
              currentText = scrambledText;
              titleRef.current.textContent = currentText;
            },
          }
        );
      }

      // Ensure final text is correct
      timeline.call(() => {
        if (titleRef.current) {
          titleRef.current.textContent = originalText;
        }
      });

      return timeline;
    };

    // Run scramble effect on mount
    const tl = scramble();

    return () => {
      tl.kill();
    };
  }, []);

  return (
    <div id="page-landing">
      <HorseAnimation />
      <div className="content-wrapper">
        <main>
          <h1 ref={titleRef}>MAPA DE RAVE</h1>
          <p>Descubra a Batida do Underground</p>
        </main>

        <Link to="/raves" className="enter-app">
          <FiArrowRight size={26} color="#ffffff" />
        </Link>
      </div>
    </div>
  );
}

export default Landing;
