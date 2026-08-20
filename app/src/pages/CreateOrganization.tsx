import { useEffect, useRef, useState, type ChangeEvent } from "react";
import { useNavigate } from "react-router-dom";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { MapContainer, Marker, TileLayer, useMapEvents } from "react-leaflet";
import type { Map as LeafletMap } from "leaflet";
import { AlertCircle, Crosshair, Loader2, Plus, X } from "lucide-react";

import Sidebar from "@/components/Sidebar/Sidebar";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import api from "@/services/api";
import { env } from "@/config/env";
import mapIcon from "@/utils/mapIcon";
import { useSeo } from "@/hooks/useSeo";

import {
  ABOUT_MAX_LENGTH,
  createOrganizationSchema,
  type CreateOrganizationValues,
} from "./create-organization.schema";

const UF_OPTIONS = [
  "AC",
  "AL",
  "AP",
  "AM",
  "BA",
  "CE",
  "DF",
  "ES",
  "GO",
  "MA",
  "MT",
  "MS",
  "MG",
  "PA",
  "PB",
  "PR",
  "PE",
  "PI",
  "RJ",
  "RN",
  "RS",
  "RO",
  "RR",
  "SC",
  "SP",
  "SE",
  "TO",
];

const TYPE_OPTIONS = [
  { value: "Festa", label: "Festa" },
  { value: "Festival", label: "Festival" },
  { value: "Label", label: "Label" },
  { value: "Radio", label: "Rádio" },
  { value: "Podcast", label: "Podcast" },
  { value: "Coletivo", label: "Coletivo" },
  { value: "Nucleo", label: "Núcleo" },
  { value: "Club", label: "Club" },
  { value: "Bar", label: "Bar" },
  { value: "Produtora", label: "Produtora" },
  { value: "outro", label: "Outro" },
];

const IMAGE_MAX_BYTES = 5 * 1024 * 1024;
const DEFAULT_CENTER: [number, number] = [-30.0313778, -51.2256725];

type FormValues = CreateOrganizationValues;

interface ImagePreview {
  file: File;
  preview: string;
}

function CreateOrganization() {
  const navigate = useNavigate();

  useSeo({
    title: "Cadastrar seu rolê no mapa | Mapa de Rave",
    description:
      "Coloque seu coletivo, festa, label ou club no Mapa de Rave. Leva menos de dois minutos.",
    path: "/raves/create",
    noIndex: true,
  });

  const [map, setMap] = useState<LeafletMap | null>(null);
  const [images, setImages] = useState<ImagePreview[]>([]);
  const [imageError, setImageError] = useState<string | null>(null);
  const [submitError, setSubmitError] = useState<string | null>(null);
  const [locating, setLocating] = useState(false);

  // As object URLs precisam ser revogadas na saída da página, senão cada foto
  // adicionada fica retida em memória até o reload.
  const imagesRef = useRef<ImagePreview[]>([]);
  imagesRef.current = images;

  useEffect(() => {
    return () => {
      imagesRef.current.forEach((image) => URL.revokeObjectURL(image.preview));
    };
  }, []);

  const form = useForm<FormValues>({
    resolver: zodResolver(createOrganizationSchema),
    mode: "onBlur",
    defaultValues: {
      name: "",
      about: "",
      email: "",
      social: "",
      uf: "",
      city: "",
      type: "",
      latitude: "",
      longitude: "",
    },
  });

  const aboutLength = form.watch("about").length;
  const latitude = form.watch("latitude");
  const longitude = form.watch("longitude");
  const hasPosition = latitude !== "" && longitude !== "";

  const setPosition = (lat: number, lng: number) => {
    form.setValue("latitude", lat.toString(), { shouldValidate: true });
    form.setValue("longitude", lng.toString(), { shouldValidate: true });
  };

  const handleUseMyLocation = () => {
    if (!navigator.geolocation) {
      setSubmitError("Seu navegador não suporta geolocalização.");
      return;
    }

    setLocating(true);

    navigator.geolocation.getCurrentPosition(
      ({ coords }) => {
        setPosition(coords.latitude, coords.longitude);
        map?.flyTo([coords.latitude, coords.longitude], 15);
        setLocating(false);
      },
      () => {
        setSubmitError("Não consegui pegar sua localização. Marque no mapa manualmente.");
        setLocating(false);
      }
    );
  };

  const handleSelectedImages = (event: ChangeEvent<HTMLInputElement>) => {
    if (!event.target.files) return;

    const selected = Array.from(event.target.files);
    const tooLarge = selected.filter((file) => file.size > IMAGE_MAX_BYTES);
    const accepted = selected.filter((file) => file.size <= IMAGE_MAX_BYTES);

    setImageError(
      tooLarge.length > 0 ? `${tooLarge.length} arquivo(s) acima de 5 MB foram ignorados` : null
    );

    setImages((current) => [
      ...current,
      ...accepted.map((file) => ({
        file,
        preview: URL.createObjectURL(file),
      })),
    ]);

    // Permite escolher o mesmo arquivo de novo depois de removê-lo.
    event.target.value = "";
  };

  const handleRemoveImage = (index: number) => {
    setImages((current) => {
      URL.revokeObjectURL(current[index].preview);
      return current.filter((_, position) => position !== index);
    });
  };

  const onSubmit = async (values: FormValues) => {
    setSubmitError(null);

    const data = new FormData();

    Object.entries(values).forEach(([key, value]) => {
      data.append(key, value);
    });

    images.forEach((image) => {
      data.append("images", image.file);
    });

    try {
      await api.post("organizations", data);
      navigate("/raves");
    } catch {
      setSubmitError("Não rolou salvar o cadastro. Confira sua conexão e tente de novo.");
    }
  };

  const { isSubmitting } = form.formState;

  return (
    <div
      id="page-create-organization"
      className="flex min-h-screen flex-col bg-background md:flex-row"
    >
      <Sidebar />

      <main className="flex flex-1 justify-center px-4 py-8 md:py-12 md:pr-8 md:pl-32">
        <Form {...form}>
          <form
            onSubmit={form.handleSubmit(onSubmit)}
            className="w-full max-w-3xl space-y-6"
            noValidate
          >
            <header className="space-y-2">
              <h1 className="font-display text-4xl tracking-wide text-foreground">
                Cadastrar rolê
              </h1>
              <p className="font-sans text-base font-normal text-muted-foreground">
                Coloque seu coletivo, festa ou label no mapa. Leva menos de dois minutos.
              </p>
            </header>

            <Card>
              <CardHeader>
                <CardTitle>Identidade</CardTitle>
                <CardDescription>Como as pessoas encontram e reconhecem vocês.</CardDescription>
              </CardHeader>

              <CardContent className="space-y-6">
                <FormField
                  control={form.control}
                  name="name"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>Nome</FormLabel>
                      <FormControl>
                        <Input placeholder="Ex.: Bunker 034" {...field} />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />

                <FormField
                  control={form.control}
                  name="type"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>Tipo</FormLabel>
                      <Select onValueChange={field.onChange} value={field.value || undefined}>
                        <FormControl>
                          <SelectTrigger>
                            <SelectValue placeholder="Selecione o tipo" />
                          </SelectTrigger>
                        </FormControl>
                        <SelectContent>
                          {TYPE_OPTIONS.map((option) => (
                            <SelectItem key={option.value} value={option.value}>
                              {option.label}
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                      <FormMessage />
                    </FormItem>
                  )}
                />

                <FormField
                  control={form.control}
                  name="about"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>Sobre</FormLabel>
                      <FormControl>
                        <Textarea
                          maxLength={ABOUT_MAX_LENGTH}
                          placeholder="O que rola, para quem, desde quando."
                          {...field}
                        />
                      </FormControl>
                      <div className="flex items-center justify-between gap-4">
                        <FormMessage />
                        <span className="ml-auto font-sans text-xs font-normal text-muted-foreground tabular-nums">
                          {aboutLength}/{ABOUT_MAX_LENGTH}
                        </span>
                      </div>
                    </FormItem>
                  )}
                />
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Localização</CardTitle>
                <CardDescription>
                  Clique no mapa para marcar o ponto — ou use sua localização atual.
                </CardDescription>
              </CardHeader>

              <CardContent className="space-y-6">
                <div className="space-y-3">
                  <div className="overflow-hidden rounded-lg border border-solid border-border">
                    <MapContainer
                      ref={setMap}
                      center={DEFAULT_CENTER}
                      zoom={13}
                      scrollWheelZoom={false}
                      doubleClickZoom={false}
                      className="h-70 w-full cursor-crosshair"
                    >
                      <TileLayer
                        attribution='Imagery &copy; <a href="https://www.mapbox.com/">Mapbox</a>'
                        url={`https://api.mapbox.com/styles/v1/${env.VITE_USERNAME}/${env.VITE_STYLE_ID}/tiles/256/{z}/{x}/{y}@2x?access_token=${env.VITE_ACCESS_TOKEN}`}
                      />
                      <MapClickHandler onClick={setPosition} />
                      {hasPosition && (
                        <Marker
                          interactive={false}
                          icon={mapIcon}
                          position={[Number(latitude), Number(longitude)]}
                        />
                      )}
                    </MapContainer>
                  </div>

                  <div className="flex flex-wrap items-center justify-between gap-3">
                    <p className="font-sans text-xs font-normal text-muted-foreground tabular-nums">
                      {hasPosition
                        ? `Marcado em ${Number(latitude).toFixed(5)}, ${Number(longitude).toFixed(5)}`
                        : "Nenhum ponto marcado ainda"}
                    </p>

                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      onClick={handleUseMyLocation}
                      disabled={locating}
                    >
                      {locating ? <Loader2 className="animate-spin" /> : <Crosshair />}
                      Usar minha localização
                    </Button>
                  </div>

                  {form.formState.errors.latitude && (
                    <p className="font-sans text-xs font-semibold text-destructive">
                      {form.formState.errors.latitude.message}
                    </p>
                  )}
                </div>

                <div className="grid gap-6 sm:grid-cols-[120px_1fr]">
                  <FormField
                    control={form.control}
                    name="uf"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>UF</FormLabel>
                        <Select onValueChange={field.onChange} value={field.value || undefined}>
                          <FormControl>
                            <SelectTrigger>
                              <SelectValue placeholder="UF" />
                            </SelectTrigger>
                          </FormControl>
                          <SelectContent>
                            {UF_OPTIONS.map((uf) => (
                              <SelectItem key={uf} value={uf}>
                                {uf}
                              </SelectItem>
                            ))}
                          </SelectContent>
                        </Select>
                        <FormMessage />
                      </FormItem>
                    )}
                  />

                  <FormField
                    control={form.control}
                    name="city"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>Cidade</FormLabel>
                        <FormControl>
                          <Input placeholder="Ex.: Porto Alegre" {...field} />
                        </FormControl>
                        <FormMessage />
                      </FormItem>
                    )}
                  />
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Contato e fotos</CardTitle>
                <CardDescription>Por onde falar com vocês e como é o rolê.</CardDescription>
              </CardHeader>

              <CardContent className="space-y-6">
                <div className="grid gap-6 sm:grid-cols-2">
                  <FormField
                    control={form.control}
                    name="email"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>E-mail</FormLabel>
                        <FormControl>
                          <Input
                            type="email"
                            inputMode="email"
                            autoComplete="email"
                            placeholder="contato@exemplo.com"
                            {...field}
                          />
                        </FormControl>
                        <FormMessage />
                      </FormItem>
                    )}
                  />

                  <FormField
                    control={form.control}
                    name="social"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>Social</FormLabel>
                        <FormControl>
                          <Input placeholder="@seuperfil" {...field} />
                        </FormControl>
                        <FormMessage />
                      </FormItem>
                    )}
                  />
                </div>

                <FormItem>
                  <FormLabel htmlFor="images">Fotos</FormLabel>

                  <div className="grid grid-cols-[repeat(auto-fill,minmax(96px,1fr))] gap-4">
                    {images.map((image, index) => (
                      <div
                        key={image.preview}
                        className="group relative overflow-hidden rounded-md border border-solid border-border"
                      >
                        <img
                          src={image.preview}
                          alt={image.file.name}
                          className="h-24 w-full object-cover"
                        />
                        <button
                          type="button"
                          onClick={() => handleRemoveImage(index)}
                          aria-label={`Remover ${image.file.name}`}
                          className="absolute top-1 right-1 flex size-7 cursor-pointer items-center justify-center rounded-full border-0 bg-background/90 text-foreground transition-colors hover:bg-destructive hover:text-destructive-foreground"
                        >
                          <X className="size-4" />
                        </button>
                      </div>
                    ))}

                    <label
                      htmlFor="images"
                      className="flex h-24 cursor-pointer items-center justify-center rounded-md border border-dashed border-input bg-muted text-muted-foreground transition-colors hover:border-ring hover:text-foreground"
                    >
                      <Plus className="size-6" />
                      <span className="sr-only">Adicionar fotos</span>
                    </label>
                  </div>

                  <input
                    id="images"
                    type="file"
                    multiple
                    accept="image/*"
                    onChange={handleSelectedImages}
                    className="hidden"
                  />

                  <FormDescription>JPG ou PNG, até 5 MB por arquivo. Opcional.</FormDescription>

                  {imageError && (
                    <p className="font-sans text-xs font-semibold text-destructive">{imageError}</p>
                  )}
                </FormItem>
              </CardContent>
            </Card>

            {submitError && (
              <div
                role="alert"
                className="flex items-start gap-3 rounded-md border border-solid border-destructive/40 bg-destructive/10 p-4"
              >
                <AlertCircle className="mt-0.5 size-5 shrink-0 text-destructive" />
                <p className="font-sans text-sm font-normal text-destructive">{submitError}</p>
              </div>
            )}

            <div className="flex flex-col-reverse gap-3 sm:flex-row sm:justify-end">
              <Button
                type="button"
                variant="ghost"
                onClick={() => navigate(-1)}
                disabled={isSubmitting}
              >
                Cancelar
              </Button>

              <Button type="submit" size="lg" disabled={isSubmitting}>
                {isSubmitting && <Loader2 className="animate-spin" />}
                {isSubmitting ? "Salvando..." : "Confirmar cadastro"}
              </Button>
            </div>
          </form>
        </Form>
      </main>
    </div>
  );
}

function MapClickHandler({ onClick }: { onClick: (lat: number, lng: number) => void }) {
  useMapEvents({
    click: (event) => {
      onClick(event.latlng.lat, event.latlng.lng);
    },
  });

  return null;
}

export default CreateOrganization;
