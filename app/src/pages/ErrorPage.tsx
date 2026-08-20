/* SPDX-FileCopyrightText: 2014-present Kriasoft */
/* SPDX-License-Identifier: MIT */
import { Link, useRouteError } from "react-router-dom";

import { Button } from "@/components/ui/button";

export default function RootError(): React.JSX.Element {
  const err = useRouteError() as RouteError;

  return (
    <div className="flex min-h-dvh items-center justify-center bg-background px-6">
      <div className="flex max-w-md flex-col items-center gap-4 text-center">
        <p className="font-sans text-xs font-bold tracking-widest text-muted-foreground uppercase">
          Erro {err.status || 500}
        </p>

        <h1 className="font-display text-5xl tracking-wide text-foreground">Deu ruim</h1>

        <p className="font-sans text-base text-muted-foreground">
          {err.statusText ?? err.message ?? "Algo inesperado aconteceu."}
        </p>

        <Button asChild className="mt-2">
          <Link to="/">Voltar para o início</Link>
        </Button>
      </div>
    </div>
  );
}

type RouteError = Error & { status?: number; statusText?: string };
