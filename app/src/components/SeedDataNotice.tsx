import { FlaskConical } from "lucide-react";

import { usesSeedData } from "@/config/env";

/**
 * Aviso de que o que está na tela é inventado.
 *
 * Sem isto, um preview publicado passa por site real: alguém veria "coletivos"
 * que não existem e tentaria ir num endereço que não existe. O aviso some
 * sozinho assim que VITE_API_URL for configurada.
 */
function SeedDataNotice() {
  if (!usesSeedData) return null;

  return (
    <div
      role="status"
      className="flex items-center justify-center gap-2 bg-foreground px-4 py-2 text-center font-sans text-xs font-semibold text-background"
    >
      <FlaskConical className="size-4 shrink-0" />
      Dados de demonstração: os coletivos abaixo são inventados e nada é salvo.
    </div>
  );
}

export default SeedDataNotice;
