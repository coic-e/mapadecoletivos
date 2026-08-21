import { useState } from "react";
import { AlertCircle, Check, Loader2, PencilLine } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogBody,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { requestEdit, type OrganizationChanges } from "@/services/editRequests";

/**
 * Campos que a sugestão pode tocar. Foto fica de fora: trocar imagem exige
 * upload e uma fila própria, que ainda não existe.
 */
const EDITABLE_FIELDS = [
  { name: "name", label: "Nome" },
  { name: "city", label: "Cidade" },
  { name: "uf", label: "UF" },
  { name: "address", label: "Endereço" },
  { name: "email", label: "E-mail" },
  { name: "instagram", label: "Instagram" },
  { name: "soundcloud", label: "SoundCloud" },
  { name: "bandcamp", label: "Bandcamp" },
  { name: "youtube", label: "YouTube" },
  { name: "spotify", label: "Spotify" },
  { name: "website", label: "Site" },
] as const;

type EditableField = (typeof EDITABLE_FIELDS)[number]["name"];

interface Props {
  slug: string;
  current: Partial<Record<EditableField, string | null>>;
}

/**
 * Sugestão de correção para quem está de fora.
 *
 * Não altera nada: o pedido entra na mesma fila da moderação e um admin
 * decide. Por isso o texto fala em "sugerir", não em "salvar".
 */
function EditRequestDialog({ slug, current }: Props) {
  const [open, setOpen] = useState(false);
  const [values, setValues] = useState<Record<string, string>>({});
  const [message, setMessage] = useState("");
  const [email, setEmail] = useState("");
  const [sending, setSending] = useState(false);
  const [sent, setSent] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Só o que foi realmente alterado vira pedido: mandar o formulário inteiro
  // encheria a fila de "mudanças" idênticas ao que já está lá.
  const changed = EDITABLE_FIELDS.filter(
    (field) =>
      values[field.name] !== undefined && values[field.name] !== (current[field.name] ?? "")
  );

  const reset = () => {
    setValues({});
    setMessage("");
    setEmail("");
    setError(null);
    setSent(false);
  };

  const submit = async () => {
    setSending(true);
    setError(null);

    const changes: OrganizationChanges = {};

    changed.forEach((field) => {
      changes[field.name] = values[field.name];
    });

    try {
      await requestEdit(slug, changes, message, email);
      setSent(true);
    } catch {
      setError("Não consegui enviar a sugestão. Tente de novo.");
    } finally {
      setSending(false);
    }
  };

  return (
    <Dialog
      open={open}
      onOpenChange={(next) => {
        setOpen(next);

        // Fechar depois de enviar limpa o formulário; fechar no meio preserva
        // o que já foi digitado, para reabrir e continuar.
        if (!next && sent) {
          reset();
        }
      }}
    >
      <DialogTrigger asChild>
        <Button variant="ghost" size="sm">
          <PencilLine />
          Sugerir correção
        </Button>
      </DialogTrigger>

      <DialogContent aria-describedby="edit-request-description">
        <DialogHeader>
          <DialogTitle>Sugerir correção</DialogTitle>
          <DialogDescription id="edit-request-description">
            Mude só o que está errado. A sugestão passa por moderação antes de entrar no ar.
          </DialogDescription>
        </DialogHeader>

        {sent ? (
          <DialogBody>
            <div className="flex items-start gap-3 rounded-md border border-solid border-border bg-muted p-4">
              <Check className="mt-0.5 size-5 shrink-0 text-foreground" />
              <p className="font-sans text-sm text-muted-foreground">
                Sugestão enviada. Um moderador vai revisar antes de entrar no ar. Obrigado por
                manter o mapa correto.
              </p>
            </div>
          </DialogBody>
        ) : (
          <DialogBody className="space-y-4">
            <div className="grid gap-4 sm:grid-cols-2">
              {EDITABLE_FIELDS.map((field) => (
                <div key={field.name} className="flex flex-col gap-2">
                  <Label htmlFor={`edit-${field.name}`}>{field.label}</Label>
                  <Input
                    id={`edit-${field.name}`}
                    value={values[field.name] ?? current[field.name] ?? ""}
                    onChange={(event) =>
                      setValues((previous) => ({
                        ...previous,
                        [field.name]: event.target.value,
                      }))
                    }
                  />
                </div>
              ))}
            </div>

            <div className="flex flex-col gap-2">
              <Label htmlFor="edit-message">O que mudou?</Label>
              <Textarea
                id="edit-message"
                value={message}
                onChange={(event) => setMessage(event.target.value)}
                placeholder="Ex.: o coletivo encerrou em 2025, ou o Instagram mudou de @"
                className="min-h-20"
              />
            </div>

            <div className="flex flex-col gap-2">
              <Label htmlFor="edit-email">Seu e-mail (opcional)</Label>
              <Input
                id="edit-email"
                type="email"
                value={email}
                onChange={(event) => setEmail(event.target.value)}
                placeholder="Para a moderação falar com você, se precisar"
              />
            </div>

            {error && (
              <div
                role="alert"
                className="flex items-start gap-3 rounded-md border border-solid border-destructive/40 bg-destructive/10 p-3"
              >
                <AlertCircle className="mt-0.5 size-4 shrink-0 text-destructive" />
                <p className="font-sans text-sm text-destructive">{error}</p>
              </div>
            )}
          </DialogBody>
        )}

        <DialogFooter>
          {sent ? (
            <DialogClose asChild>
              <Button>Fechar</Button>
            </DialogClose>
          ) : (
            <>
              <Button onClick={submit} disabled={sending || changed.length === 0}>
                {sending && <Loader2 className="animate-spin" />}
                Enviar sugestão
              </Button>

              <DialogClose asChild>
                <Button variant="ghost">Cancelar</Button>
              </DialogClose>

              <span className="font-sans text-xs text-muted-foreground">
                {changed.length === 0
                  ? "Nada alterado ainda"
                  : `${changed.length} campo(s) alterado(s)`}
              </span>
            </>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

export default EditRequestDialog;
