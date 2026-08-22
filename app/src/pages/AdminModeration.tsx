import { useCallback, useEffect, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import {
  AlertCircle,
  Check,
  ChevronLeft,
  ChevronRight,
  Loader2,
  LogOut,
  PencilLine,
  X,
} from "lucide-react";

import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Textarea } from "@/components/ui/textarea";
import api from "@/services/api";
import { fetchCurrentAdmin, logout, type Admin } from "@/services/auth";
import { fetchEditRequests, reviewEditRequest, type EditRequest } from "@/services/editRequests";
import { useSeo } from "@/hooks/useSeo";
import { cn } from "@/lib/utils";

interface OrganizationSummary {
  id: number;
  slug: string;
  name: string;
  type: string;
  city: string;
  uf: string;
  about: string;
  email: string;
  genres: string[];
  instagram: string | null;
  website: string | null;
  is_active: boolean;
  status: string;
  created_at: string;
  rejection_reason: string | null;
  images: Array<{ id: number; url: string }>;
}

const FILTERS = [
  { value: "pending", label: "Na fila" },
  { value: "approved", label: "Aprovados" },
  { value: "rejected", label: "Rejeitados" },
] as const;

type Filter = (typeof FILTERS)[number]["value"];

/** Imagem aberta em tamanho cheio, com o cadastro de origem para navegar entre elas. */
interface Preview {
  organization: string;
  images: Array<{ id: number; url: string }>;
  index: number;
}

/** Rótulos dos campos que um pedido de correção pode tocar. */
const FIELD_LABELS: Record<string, string> = {
  name: "Nome",
  about: "Sobre",
  email: "E-mail",
  city: "Cidade",
  uf: "UF",
  address: "Endereço",
  instagram: "Instagram",
  soundcloud: "SoundCloud",
  bandcamp: "Bandcamp",
  youtube: "YouTube",
  spotify: "Spotify",
  website: "Site",
  genres: "Gêneros",
  frequency: "Periodicidade",
  is_active: "Ativo",
};

function AdminModeration() {
  const navigate = useNavigate();

  const [admin, setAdmin] = useState<Admin | null>(null);
  const [checkingSession, setCheckingSession] = useState(true);
  const [filter, setFilter] = useState<Filter>("pending");
  const [organizations, setOrganizations] = useState<OrganizationSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [busyId, setBusyId] = useState<number | null>(null);
  const [rejectingId, setRejectingId] = useState<number | null>(null);
  const [rejectReason, setRejectReason] = useState("");
  const [editRequests, setEditRequests] = useState<EditRequest[]>([]);
  const [busyRequestId, setBusyRequestId] = useState<number | null>(null);
  const [preview, setPreview] = useState<Preview | null>(null);

  useSeo({
    title: "Fila de moderação | Mapa de Rave",
    description: "Área restrita de moderação.",
    path: "/admin",
    noIndex: true,
  });

  // Token ausente ou expirado devolve para o login em vez de deixar a tela
  // vazia sem explicação.
  useEffect(() => {
    fetchCurrentAdmin()
      .then((current) => {
        setAdmin(current);
        setCheckingSession(false);
      })
      .catch(() => navigate("/admin/login", { replace: true }));
  }, [navigate]);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      const { data } = await api.get<OrganizationSummary[]>(`admin/organizations?status=${filter}`);

      setOrganizations(data);
    } catch {
      setError("Não consegui carregar a fila. Tente de novo.");
    } finally {
      setLoading(false);
    }
  }, [filter]);

  useEffect(() => {
    load();
  }, [load]);

  const loadEditRequests = useCallback(async () => {
    try {
      setEditRequests(await fetchEditRequests("pending"));
    } catch {
      setError("Não consegui carregar os pedidos de correção.");
    }
  }, []);

  useEffect(() => {
    loadEditRequests();
  }, [loadEditRequests]);

  const reviewRequest = async (id: number, decision: "apply" | "reject") => {
    setBusyRequestId(id);
    setError(null);

    try {
      await reviewEditRequest(id, decision);
      await Promise.all([loadEditRequests(), load()]);
    } catch {
      setError("A decisão não foi salva. Tente de novo.");
    } finally {
      setBusyRequestId(null);
    }
  };

  const review = async (id: number, decision: "approve" | "reject") => {
    setBusyId(id);
    setError(null);

    try {
      await api.post(
        `admin/organizations/${id}/${decision}`,
        decision === "reject" ? { reason: rejectReason } : undefined
      );

      setRejectingId(null);
      setRejectReason("");

      // Recarrega em vez de remover na mão: a lista depende do filtro atual.
      await load();
    } catch {
      setError("A decisão não foi salva. Tente de novo.");
    } finally {
      setBusyId(null);
    }
  };

  // Sem isto o painel aparece por um instante para quem não está logado, e o
  // redirecionamento em seguida parece falha da aplicação. Nenhum dado vazava
  // — as requisições respondem 401 —, mas a tela mentia sobre o estado.
  if (checkingSession) {
    return (
      <div className="flex min-h-dvh items-center justify-center bg-background">
        <p className="font-sans text-base text-muted-foreground">Verificando sessão…</p>
      </div>
    );
  }

  return (
    <div className="min-h-dvh bg-background">
      <header className="border-b border-border">
        <div className="mx-auto flex max-w-5xl flex-wrap items-center justify-between gap-4 px-4 py-6">
          <div>
            <h1 className="font-display text-3xl tracking-wide text-foreground">
              Fila de moderação
            </h1>
            {admin && (
              <p className="font-sans text-sm text-muted-foreground">
                {admin.name} · {admin.email}
              </p>
            )}
          </div>

          <Button
            variant="ghost"
            onClick={() => {
              logout();
              navigate("/admin/login");
            }}
          >
            <LogOut />
            Sair
          </Button>
        </div>
      </header>

      <main className="mx-auto max-w-5xl px-4 py-8">
        {editRequests.length > 0 && (
          <section className="mb-10">
            <h2 className="mb-4 flex items-center gap-2 font-display text-2xl tracking-wide text-foreground">
              <PencilLine className="size-5" />
              Pedidos de correção ({editRequests.length})
            </h2>

            <ul className="space-y-4">
              {editRequests.map((request) => (
                <li key={request.id}>
                  <Card>
                    <CardContent className="flex flex-col gap-3 p-4 sm:p-6">
                      <p className="font-sans text-xs tracking-widest text-muted-foreground uppercase">
                        Cadastro #{request.organization_id}
                        {request.requester_email
                          ? ` · sugerido por ${request.requester_email}`
                          : " · sem contato"}
                      </p>

                      {request.message && (
                        <p className="font-sans text-sm text-foreground italic">
                          “{request.message}”
                        </p>
                      )}

                      <dl className="flex flex-col gap-1">
                        {Object.entries(request.changes).map(([field, value]) => (
                          <div key={field} className="flex flex-wrap gap-2">
                            <dt className="font-sans text-sm font-bold text-foreground">
                              {FIELD_LABELS[field] ?? field}:
                            </dt>
                            <dd className="font-sans text-sm text-muted-foreground">
                              {Array.isArray(value) ? value.join(", ") : String(value)}
                            </dd>
                          </div>
                        ))}
                      </dl>

                      <div className="flex flex-wrap gap-2">
                        <Button
                          size="sm"
                          disabled={busyRequestId === request.id}
                          onClick={() => reviewRequest(request.id, "apply")}
                        >
                          {busyRequestId === request.id ? (
                            <Loader2 className="animate-spin" />
                          ) : (
                            <Check />
                          )}
                          Aplicar
                        </Button>

                        <Button
                          variant="outline"
                          size="sm"
                          disabled={busyRequestId === request.id}
                          onClick={() => reviewRequest(request.id, "reject")}
                        >
                          <X />
                          Descartar
                        </Button>
                      </div>
                    </CardContent>
                  </Card>
                </li>
              ))}
            </ul>
          </section>
        )}

        <div className="mb-6 flex flex-wrap gap-2">
          {FILTERS.map((option) => (
            <Button
              key={option.value}
              variant={filter === option.value ? "default" : "outline"}
              size="sm"
              onClick={() => setFilter(option.value)}
            >
              {option.label}
            </Button>
          ))}
        </div>

        {error && (
          <div
            role="alert"
            className="mb-6 flex items-start gap-3 rounded-md border border-destructive/40 bg-destructive/10 p-4"
          >
            <AlertCircle className="mt-0.5 size-5 shrink-0 text-destructive" />
            <p className="font-sans text-sm text-destructive">{error}</p>
          </div>
        )}

        {loading ? (
          <p className="font-sans text-base text-muted-foreground">Carregando…</p>
        ) : organizations.length === 0 ? (
          <p className="font-sans text-base text-muted-foreground">
            {filter === "pending"
              ? "Nada na fila. Tudo revisado."
              : "Nenhum cadastro neste estado."}
          </p>
        ) : (
          <ul className="space-y-4">
            {organizations.map((organization) => (
              <li key={organization.id}>
                <Card>
                  <CardContent className="flex flex-col gap-4 p-4 sm:flex-row sm:p-6">
                    {organization.images.length > 0 && (
                      <div className="flex w-full shrink-0 flex-wrap content-start gap-2 sm:w-48">
                        {organization.images.map((image, index) => (
                          <button
                            key={image.id}
                            type="button"
                            title={`Abrir foto ${index + 1} de ${organization.images.length}`}
                            onClick={() =>
                              setPreview({
                                organization: organization.name,
                                images: organization.images,
                                index,
                              })
                            }
                            className="h-24 flex-1 basis-[calc(50%-0.25rem)] cursor-pointer overflow-hidden rounded-md border-0 bg-muted p-0"
                          >
                            <img
                              src={image.url}
                              alt={`Foto ${index + 1} de ${organization.images.length} do cadastro ${organization.name}`}
                              loading="lazy"
                              className="size-full object-cover"
                            />
                          </button>
                        ))}
                      </div>
                    )}

                    <div className="flex min-w-0 flex-1 flex-col gap-2">
                      <div className="flex flex-wrap items-center gap-2">
                        <h2 className="font-display text-2xl tracking-wide text-foreground">
                          {organization.name}
                        </h2>
                        <span
                          className={cn(
                            "rounded-full px-2 py-0.5 font-sans text-xs font-bold uppercase",
                            organization.status === "pending" && "bg-muted text-muted-foreground",
                            organization.status === "approved" &&
                              "bg-primary text-primary-foreground",
                            organization.status === "rejected" &&
                              "bg-destructive/15 text-destructive"
                          )}
                        >
                          {organization.status}
                        </span>
                      </div>

                      <p className="font-sans text-xs tracking-widest text-muted-foreground uppercase">
                        {organization.type} · {organization.city}/{organization.uf}
                      </p>

                      <p className="font-sans text-sm text-muted-foreground">
                        {organization.about}
                      </p>

                      {organization.genres.length > 0 && (
                        <p className="font-sans text-xs text-muted-foreground">
                          {organization.genres.join(" · ")}
                        </p>
                      )}

                      <p className="font-sans text-xs text-muted-foreground">
                        {[
                          organization.email,
                          organization.instagram,
                          organization.website,
                          organization.is_active ? null : "encerrado",
                        ]
                          .filter(Boolean)
                          .join(" · ")}
                      </p>

                      {organization.rejection_reason && (
                        <p className="font-sans text-xs text-destructive">
                          Motivo da rejeição: {organization.rejection_reason}
                        </p>
                      )}

                      {rejectingId === organization.id ? (
                        <div className="mt-2 space-y-2">
                          <Textarea
                            value={rejectReason}
                            onChange={(event) => setRejectReason(event.target.value)}
                            placeholder="Motivo da rejeição (opcional, fica registrado)"
                            className="min-h-20"
                          />

                          <p className="font-sans text-xs text-muted-foreground">
                            Rejeitar apaga as fotos do bucket. O cadastro fica registrado, mas não
                            dá para aprová-lo depois.
                          </p>
                          <div className="flex gap-2">
                            <Button
                              variant="destructive"
                              size="sm"
                              disabled={busyId === organization.id}
                              onClick={() => review(organization.id, "reject")}
                            >
                              {busyId === organization.id && <Loader2 className="animate-spin" />}
                              Confirmar rejeição
                            </Button>
                            <Button
                              variant="ghost"
                              size="sm"
                              onClick={() => {
                                setRejectingId(null);
                                setRejectReason("");
                              }}
                            >
                              Cancelar
                            </Button>
                          </div>
                        </div>
                      ) : (
                        <div className="mt-2 flex flex-wrap gap-2">
                          {organization.status === "pending" && (
                            <Button
                              size="sm"
                              disabled={busyId === organization.id}
                              onClick={() => review(organization.id, "approve")}
                            >
                              {busyId === organization.id ? (
                                <Loader2 className="animate-spin" />
                              ) : (
                                <Check />
                              )}
                              Aprovar
                            </Button>
                          )}

                          {organization.status !== "rejected" && (
                            <Button
                              variant="outline"
                              size="sm"
                              onClick={() => setRejectingId(organization.id)}
                            >
                              <X />
                              Rejeitar
                            </Button>
                          )}

                          {organization.status === "approved" && (
                            <Button asChild variant="ghost" size="sm">
                              <Link to={`/raves/${organization.slug}`}>Ver no site</Link>
                            </Button>
                          )}
                        </div>
                      )}
                    </div>
                  </CardContent>
                </Card>
              </li>
            ))}
          </ul>
        )}
      </main>

      {/* Julgar um cadastro depende das fotos: a miniatura corta demais para
          decidir, então a modal mostra a imagem inteira sem sair da fila. */}
      <Dialog open={preview !== null} onOpenChange={(open) => !open && setPreview(null)}>
        <DialogContent className="max-w-4xl">
          {preview && (
            <>
              <DialogHeader>
                <DialogTitle>{preview.organization}</DialogTitle>
                <DialogDescription>
                  Foto {preview.index + 1} de {preview.images.length}
                </DialogDescription>
              </DialogHeader>

              <div className="flex items-center gap-2 p-4">
                {preview.images.length > 1 && (
                  <Button
                    variant="ghost"
                    size="icon"
                    aria-label="Foto anterior"
                    onClick={() =>
                      setPreview({
                        ...preview,
                        index: (preview.index - 1 + preview.images.length) % preview.images.length,
                      })
                    }
                  >
                    <ChevronLeft />
                  </Button>
                )}

                <img
                  src={preview.images[preview.index].url}
                  alt={`Foto ${preview.index + 1} de ${preview.images.length} do cadastro ${preview.organization}`}
                  className="max-h-[70dvh] min-w-0 flex-1 rounded-md object-contain"
                />

                {preview.images.length > 1 && (
                  <Button
                    variant="ghost"
                    size="icon"
                    aria-label="Próxima foto"
                    onClick={() =>
                      setPreview({
                        ...preview,
                        index: (preview.index + 1) % preview.images.length,
                      })
                    }
                  >
                    <ChevronRight />
                  </Button>
                )}
              </div>
            </>
          )}
        </DialogContent>
      </Dialog>
    </div>
  );
}

export default AdminModeration;
