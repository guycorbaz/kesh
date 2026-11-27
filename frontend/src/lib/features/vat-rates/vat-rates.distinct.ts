/**
 * Revue 64-1 (L2-1, L2-4) — une ligne par valeur de taux, le représentant ne dépendant pas de
 * l'ordre reçu. Les listes déroulantes (`<select>`) ne proposent une valeur qu'une fois : leur
 * clé de `{#each}` est la valeur, et deux taux de même valeur (catégories ou dates de début
 * différentes) donneraient une clé en double.
 *
 * Représentant d'une valeur : d'abord un taux qui porte un libellé (non vide, distinct de sa
 * catégorie, que le serveur pose à défaut), puis la catégorie, puis l'`id`. L'ordre de sortie
 * est celui de la première apparition de chaque valeur.
 */
import type { VatRateResponse } from "./vat-rates.types";

type Distinguable = Pick<VatRateResponse, "rate"> &
  Partial<Pick<VatRateResponse, "id" | "category" | "label">>;

function hasOwnLabel(r: Distinguable): boolean {
  const label = (r.label ?? "").trim();
  return label !== "" && label !== r.category;
}

/** Vrai si `a` doit être préféré à `b` comme représentant d'une même valeur. */
function prefer(a: Distinguable, b: Distinguable): boolean {
  if (hasOwnLabel(a) !== hasOwnLabel(b)) return hasOwnLabel(a);
  const ca = a.category ?? "";
  const cb = b.category ?? "";
  if (ca !== cb) return ca < cb;
  return (a.id ?? 0) < (b.id ?? 0);
}

export function distinctVatRates<T extends Distinguable>(rates: T[]): T[] {
  const byValue = new Map<string, T>();
  for (const r of rates) {
    const kept = byValue.get(r.rate);
    if (kept === undefined || prefer(r, kept)) byValue.set(r.rate, r);
  }
  return [...byValue.values()];
}
