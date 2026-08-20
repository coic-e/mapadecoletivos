import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { AlertCircle, Loader2 } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { login } from "@/services/auth";
import { useSeo } from "@/hooks/useSeo";

const loginSchema = z.object({
  email: z.email("E-mail inválido"),
  password: z.string().min(1, "Informe a senha"),
});

type LoginValues = z.infer<typeof loginSchema>;

function AdminLogin() {
  const navigate = useNavigate();
  const [error, setError] = useState<string | null>(null);

  useSeo({
    title: "Moderação | Mapa de Rave",
    description: "Área restrita de moderação.",
    path: "/admin/login",
    noIndex: true,
  });

  const form = useForm<LoginValues>({
    resolver: zodResolver(loginSchema),
    defaultValues: { email: "", password: "" },
  });

  const onSubmit = async (values: LoginValues) => {
    setError(null);

    try {
      await login(values.email, values.password);
      navigate("/admin");
    } catch {
      // Mesma mensagem para e-mail inexistente e senha errada, igual à API.
      setError("E-mail ou senha inválidos.");
    }
  };

  return (
    <div className="flex min-h-dvh items-center justify-center bg-background px-4 py-12">
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle>Moderação</CardTitle>
          <CardDescription>Área restrita. Entre para revisar os cadastros na fila.</CardDescription>
        </CardHeader>

        <CardContent>
          <Form {...form}>
            <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-6" noValidate>
              <FormField
                control={form.control}
                name="email"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>E-mail</FormLabel>
                    <FormControl>
                      <Input
                        type="email"
                        autoComplete="email"
                        placeholder="voce@exemplo.com"
                        {...field}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="password"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Senha</FormLabel>
                    <FormControl>
                      <Input type="password" autoComplete="current-password" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              {error && (
                <div
                  role="alert"
                  className="flex items-start gap-3 rounded-md border border-destructive/40 bg-destructive/10 p-3"
                >
                  <AlertCircle className="mt-0.5 size-4 shrink-0 text-destructive" />
                  <p className="font-sans text-sm text-destructive">{error}</p>
                </div>
              )}

              <Button type="submit" className="w-full" disabled={form.formState.isSubmitting}>
                {form.formState.isSubmitting && <Loader2 className="animate-spin" />}
                Entrar
              </Button>
            </form>
          </Form>
        </CardContent>
      </Card>
    </div>
  );
}

export default AdminLogin;
