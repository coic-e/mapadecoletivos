import { useEffect, useRef, useState } from "react";
import type { UseFormReturn, FieldValues } from "react-hook-form";

/**
 * Guarda o que já foi digitado no localStorage e devolve na volta.
 *
 * O cadastro é longo: fechar a aba sem isso significa recomeçar do zero. As
 * fotos ficam de fora — File não sobrevive à serialização, e guardar imagem em
 * localStorage estouraria a cota.
 */
export function useFormDraft<T extends FieldValues>(
  key: string,
  form: UseFormReturn<T>,
  options?: { exclude?: (keyof T)[] }
) {
  const [restored, setRestored] = useState(false);
  const excludeRef = useRef(options?.exclude ?? []);

  // Restaura uma vez, na montagem.
  useEffect(() => {
    const saved = localStorage.getItem(key);

    if (!saved) return;

    try {
      const draft = JSON.parse(saved) as Partial<T>;

      form.reset({ ...form.getValues(), ...draft });
      setRestored(true);
    } catch {
      // Rascunho corrompido não pode impedir o formulário de abrir.
      localStorage.removeItem(key);
    }
    // Só a chave entra nas dependências: form é estável entre renders no
    // react-hook-form, e incluí-lo restauraria o rascunho a cada digitação.
  }, [key]);

  // Salva a cada mudança.
  useEffect(() => {
    const subscription = form.watch((values) => {
      const draft = { ...values } as Partial<T>;

      excludeRef.current.forEach((field) => {
        delete draft[field];
      });

      try {
        localStorage.setItem(key, JSON.stringify(draft));
      } catch {
        // Cota estourada não deve derrubar o formulário.
      }
    });

    return () => subscription.unsubscribe();
  }, [form, key]);

  const clearDraft = () => {
    localStorage.removeItem(key);
    setRestored(false);
  };

  return { restored, clearDraft };
}
