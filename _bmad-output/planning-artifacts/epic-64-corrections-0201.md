# Epic 64 — Corrections de la v0.20.1

*Kickoff du 2026-11-27 (décision du Project Lead du 2026-11-27 : « quelques bugs simples » dans une release de correction).*
Branche `epic/64-corrections-0201`, partie de `main` `0e99cd2db` (la v0.20.0). Release visée : **v0.20.1**, dimanche 2026-11-29 ou
lundi 2026-11-30, **sans migration** (`kesh_version_min_required` inchangé).

## Objet

Trois défauts petits et indépendants, relevés pendant les fiches des releases précédentes : un formulaire de taux de TVA qui
n'envoie rien (#1178), une page de modification de facture qui s'ouvre sur une facture validée (#1758), et une route de réglage de
l'installation ouverte au rôle Consultation (#1712).

## Stories

| Story | Issues | Objet |
|---|---|---|
| 64-1 — Corrections de la v0.20.1 | #1178 (P2), #1758 (P3), #1712 (P3) | le taux de TVA saisi part en chaîne décimale ; `/invoices/{id}/edit` refuse une facture qui n'est pas un brouillon, et un rôle sans droit d'écriture ; `PUT /profile/mode` réservé à l'Administrateur |

Une seule story : trois correctifs indépendants, sans migration, sur **cinq modules** au kickoff (recompté par la validation P1 de la
fiche : écran des taux de TVA, écran des factures, cadre de l'application `(app)/+layout.svelte`, `kesh-api`, `kesh-i18n`) ;
la règle de splitting préventif (plus de cinq modules) ne s'appliquait pas alors. *Revue de code P1 (R1-1), 2026-11-27 : deux
correctifs d'une ligne ajoutent `InvoiceForm.svelte` (composant de facture) et `products/+page.svelte` (catalogue) : **sept
modules** ; ni découpage ni dérogation écrits, les défauts n'étant pas recyclés — à déclarer au Project Lead.* *Revue de code P2 (L2-1, L2-4), 2026-11-27 : le lot des LOW de code ajoute `features/supplier-invoices` (`SupplierInvoiceForm.svelte`) et une fonction partagée dans `features/vat-rates` (déjà compté par l'écran des taux) : **huit modules**, recomptés — écran des taux, écran des factures, cadre `(app)`, composant de facture, catalogue, facture fournisseur, `kesh-api`, `kesh-i18n` ; même motif, défauts neufs non recyclés, déclaration au Project Lead inchangée.* Pas de story de solde : l'epic ne laisse pas
de dette propre (toute dette découverte s'ouvre en issue, § *On trace toujours*).

## Dépendances

- Dépend de : aucune. Bloque : aucune. Suite : **#1763** (changement de taux de TVA atomique côté serveur, P2, release du
  vendredi 2026-12-04) reprend ce que #1178 ne traite plus (décision du Project Lead du 2026-11-27).
- Base : `main` a reçu l'Epic 55 (PR #1696) après le kickoff ; numéro et base de la release à trancher (fiche 64-1, Q3).
- Recoupe (relevé par la validation P1 de la fiche, 2026-11-27) : `epic/50-corrections` touche aussi les catalogues i18n des
  quatre locales, `i18n-keys.test.ts` (`sitesTotal`), `docs/api-external.md`, `docs/manual/fr/admin-manual.tex` et
  `user-manual.tex` ; la PR #1696 (Epic 55) touche `crates/kesh-api/src/lib.rs` (une route hors du bloc `admin_routes`),
  les catalogues i18n et `sprint-status.yaml`. Le second arrivé résout dans sa branche et recompte les compteurs
  (`sitesTotal`, registres de routes) depuis la source.

## Fichiers et modules touchés (déclarés au kickoff)

- `frontend/src/routes/(app)/settings/vat-rates/` (+ son test) ;
- `frontend/src/routes/(app)/invoices/[id]/edit/+page.svelte` (+ test) ;
- *Ajouts de la revue de code* : `frontend/src/lib/components/invoices/InvoiceForm.svelte`, `frontend/src/routes/(app)/products/+page.svelte`, `frontend/src/lib/features/supplier-invoices/SupplierInvoiceForm.svelte`, `frontend/src/lib/features/vat-rates/vat-rates.distinct.ts` (+ tests) ;
- `frontend/src/routes/(app)/+layout.svelte` et `frontend/src/lib/app/stores/mode.svelte.ts` (menu utilisateur : bascule du mode) ;
- `crates/kesh-api/src/lib.rs` (déplacement de la route `PUT /api/v1/profile/mode` dans le bloc `admin_routes`), tests
  `profile_e2e.rs`, `rbac_e2e.rs`, `admin_pat_denied_e2e.rs`, `audit_route_registry.rs` ; commentaire de `src/routes/profile.rs` ;
- catalogues i18n des quatre locales (`crates/kesh-i18n/locales/*/messages.ftl`) ;
- `docs/manual/fr/admin-manual.tex` et `user-manual.tex` (sources seules ; DE/IT/EN ne sont pas écrits), `docs/api-external.md`, `README.md` (feuille de route si elle cite l'epic).

## Décompte des issues

Celui du tag précédent (v0.20.0) et l'objectif : à écrire par l'orchestrateur à la PR de release (`gh issue list --state open
--limit 600 --json number --jq length`, reconstitué depuis `createdAt`/`closedAt` au tag passé). Les trois issues de la story se
ferment à la fusion de la PR (mot-clé dans la PR seulement).
