import { describe, expect, it } from "vitest";

import { ABOUT_MAX_LENGTH, createOrganizationSchema } from "./create-organization.schema";

const validValues = {
  name: "Bunker 034",
  about: "Festa de techno no centro de Porto Alegre desde 2019.",
  email: "contato@bunker034.com",
  genres: ["Techno" as const, "Acid" as const],
  instagram: "@bunker034",
  isActive: true,
  uf: "RS",
  city: "Porto Alegre",
  type: "Festa",
  latitude: "-30.0313778",
  longitude: "-51.2256725",
  consent: true,
};

describe("createOrganizationSchema", () => {
  it("aceita um cadastro completo", () => {
    expect(createOrganizationSchema.safeParse(validValues).success).toBe(true);
  });

  it("recusa e-mail inválido", () => {
    const result = createOrganizationSchema.safeParse({
      ...validValues,
      email: "contato-arroba-nada",
    });

    expect(result.success).toBe(false);
  });

  it("exige que o ponto do mapa tenha sido marcado", () => {
    const result = createOrganizationSchema.safeParse({
      ...validValues,
      latitude: "",
      longitude: "",
    });

    expect(result.success).toBe(false);
  });

  it("exige pelo menos um gênero", () => {
    const result = createOrganizationSchema.safeParse({
      ...validValues,
      genres: [],
    });

    expect(result.success).toBe(false);
  });

  it("recusa gênero fora da lista", () => {
    const result = createOrganizationSchema.safeParse({
      ...validValues,
      genres: ["Sertanejo"],
    });

    expect(result.success).toBe(false);
  });

  it("exige pelo menos um link", () => {
    const semLink = { ...validValues, instagram: "" };

    expect(createOrganizationSchema.safeParse(semLink).success).toBe(false);

    const comOutroLink = { ...semLink, bandcamp: "bunker.bandcamp.com" };

    expect(createOrganizationSchema.safeParse(comOutroLink).success).toBe(true);
  });

  it("exige o aceite de autorização", () => {
    const result = createOrganizationSchema.safeParse({
      ...validValues,
      consent: false,
    });

    expect(result.success).toBe(false);
  });

  it("limita o texto de apresentação", () => {
    const result = createOrganizationSchema.safeParse({
      ...validValues,
      about: "a".repeat(ABOUT_MAX_LENGTH + 1),
    });

    expect(result.success).toBe(false);
  });

  it("remove espaços em volta dos campos de texto", () => {
    const result = createOrganizationSchema.safeParse({
      ...validValues,
      name: "  Bunker 034  ",
    });

    expect(result.success && result.data.name).toBe("Bunker 034");
  });
});
