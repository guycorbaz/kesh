import { describe, it, expect } from "vitest";
import { distinctVatRates } from "./vat-rates.distinct";

const r = (id: number, rate: string, category: string, label: string) => ({
  id,
  rate,
  category,
  label,
});

describe("distinctVatRates — le représentant d’une valeur ne dépend pas de l’ordre (L2-1)", () => {
  const sansLibelle = r(2, "8.10", "custom", "custom");
  const normal = r(1, "8.10", "normal", "product-vat-normal");

  it("préfère le taux qui a un libellé, quel que soit l’ordre", () => {
    expect(distinctVatRates([sansLibelle, normal])).toEqual([normal]);
    expect(distinctVatRates([normal, sansLibelle])).toEqual([normal]);
  });

  it("un libellé vide compte pour une absence de libellé", () => {
    const vide = r(3, "8.10", "a", "");
    expect(distinctVatRates([vide, normal])).toEqual([normal]);
  });

  it("à égalité de libellé : la catégorie, puis l’id", () => {
    const a = r(5, "2.60", "b", "x");
    const b = r(4, "2.60", "a", "y");
    expect(distinctVatRates([a, b])).toEqual([b]);
    expect(distinctVatRates([b, a])).toEqual([b]);
    const c = r(9, "2.60", "a", "z");
    expect(distinctVatRates([c, b])).toEqual([b]);
  });

  it("une ligne par valeur, ordre de première apparition", () => {
    const t = r(7, "2.60", "reduced", "product-vat-reduced");
    expect(
      distinctVatRates([sansLibelle, t, normal]).map((x) => x.rate),
    ).toEqual(["8.10", "2.60"]);
  });
});
