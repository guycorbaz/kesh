# Epic 52 — Lignes de facture, suite : le réordonnancement

*Kickoff du 2026-11-21, par découpage de l'Epic 49 (décision du Project Lead du 2026-11-21 : un epic tient dans une
release). **Réduit le 2026-11-27** (décision du Project Lead du 2026-11-27) : l'Epic 52 devient l'epic de la **v0.21.0**
(milestone `v0.21.0`, vendredi 2026-12-04, coupure le jeudi 2026-12-03 au soir) avec la seule **49-2
(réordonnancement)** et sa story de solde ; la **49-3 (titres de section)** et la **49-4b (désignation longue au PDF,
#1363, #1400)** passent à l'**Epic 62** (`epic-62-lignes-de-facture-titres.md`), pour la v0.22.0.* Branche
`epic/52-lignes-de-facture-suite`, partie de `main` à `0e99cd2db` (la v0.20.0, qui porte les types de lignes `priced`,
`comment`, `heading` de l'Epic 49). **Migration : aucune** (la position d'une ligne est son rang dans le corps de la
requête, et la modification d'un brouillon remplace toutes ses lignes ; voir la fiche de la 49-2).

## Objet

Réordonner à la main les lignes d'une facture client **en brouillon** (#1242, section « Réordonnancement » : « on peut
créer toutes les lignes à la suite puis les réordonner »), à l'écran de saisie, par des boutons ↑ / ↓ accessibles au
clavier. Une facture validée ne se réordonne jamais : son PDF et son écriture font foi. Les avoirs n'ont pas de brouillon
(ils sont émis d'emblée) et reprennent l'ordre de leur facture : ils ne sont pas concernés.

## Stories

| Story | Issues | Détail |
|---|---|---|
| 49-2-reordonnancement-lignes | #1242 | ↑ / ↓ sur chaque ligne du formulaire de facture (création et brouillon), au clavier ; glisser-déposer écarté par la fiche ; `_bmad-output/implementation-artifacts/49-2-reordonnancement-lignes.md` |
| 52-solde-dette-epic | plafond proposé : **deux issues** (à confirmer par le Project Lead) | la seule dette de la 49-2 (LOW de revue renvoyés en issue) |

Deux stories : l'epic tient dans la borne de six. **Fonctionnalité nouvelle** de la release : la 49-2. La story de
correction de la release (CLAUDE.md § *Cadence*) n'est pas dans cet epic.

## Dépendances

- Dépend de : **Epic 49** (49-1a à 49-1d, 49-4a), entré dans `main` avec la v0.20.0 (`0e99cd2db`) — fait.
- Bloque : aucune. L'Epic 62 n'en dépend pas fonctionnellement ; il partage avec la 49-2 deux fichiers (ci-dessous) et
  reprendra `main` après la v0.21.0.

## Fichiers et modules touchés (déclarés au kickoff réduit)

| Story | Backend | Frontend | Docs, i18n, tests |
|---|---|---|---|
| 49-2 | aucun code de production (tests seuls : module de tests de `kesh-db/src/repositories/invoices.rs`, `kesh-api/tests/invoice_line_types_e2e.rs`) | `lib/components/invoices/InvoiceForm.svelte`, `lib/features/invoices/invoice-lines.ts` | `kesh-i18n` ×4, `docs/manual/fr/user-manual.tex`, `docs/api-external.md`, Vitest, E2E Playwright |
| 52-solde | selon les issues retenues | selon les issues retenues | — |

Fichiers communs : `InvoiceForm.svelte` et `user-manual.tex` avec l'Epic 62 (49-3) ; catalogues i18n et `sitesTotal`
avec tous les epics de la release.

## Décomptes au kickoff réduit

Issues ouvertes, bugs et P1 : à recompter depuis GitHub au lancement de la 49-2 (CLAUDE.md § *Priorités des défauts*) ;
le décompte du tag précédent et l'objectif s'écrivent ici à ce moment-là. #1242 (`enhancement`, P2) est au milestone
`v0.22.0` : elle ne se ferme pas avec la 49-2 (titres de section à l'Epic 62, sous-totaux #1330).
