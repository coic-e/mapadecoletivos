import { FiArrowLeft } from "react-icons/fi";
import { useNavigate } from "react-router-dom";

import iconDiscoBall from "@/images/discoball-pequeno.svg";

function Sidebar() {
  const navigate = useNavigate();

  return (
    <aside className="relative flex w-full flex-row items-center justify-between border-b border-border bg-background p-4 md:fixed md:top-0 md:left-0 md:h-dvh md:w-24 md:flex-col md:justify-end md:border-r md:border-b-0 md:px-6 md:pt-0 md:pb-8 lg:w-28">
      <img
        src={iconDiscoBall}
        alt="Mapa de Rave"
        className="h-14 w-12 md:absolute md:top-0 md:left-1/2 md:h-20 md:w-17.5 md:-translate-x-1/2 lg:h-27 lg:w-23.5"
      />

      <footer className="flex items-center">
        <button
          type="button"
          aria-label="Voltar"
          onClick={() => navigate(-1)}
          className="flex size-10 cursor-pointer items-center justify-center rounded-full border-2 border-primary bg-primary text-primary-foreground transition-colors hover:bg-background hover:text-foreground md:size-11 lg:size-12"
        >
          <FiArrowLeft size={20} />
        </button>
      </footer>
    </aside>
  );
}

export default Sidebar;
