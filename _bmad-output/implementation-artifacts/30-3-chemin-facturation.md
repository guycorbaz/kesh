# Story 30-3 — Le chemin de la facturation

Status: review

Issue : [#585](https://github.com/guycorbaz/kesh/issues/585) (bug, P4). Branche `fix/30-3-chemin-facturation`,
depuis `origin/main` `fc3a4f7d`. Fiche courte (processus allégé : texte seulement, pas de boucle de validation).

## Constat (établi au code, 2026-10-16)

- `frontend/src/routes/(app)/+layout.svelte` : le groupe `nav-administration` (« Administration ») porte, côté
  administrateur, l'entrée `nav-invoicing` (« Facturation », `/settings/invoicing`) **à côté** de `nav-settings`
  (« Paramètres », `/settings`). « Facturation » est une entrée sœur de « Paramètres », pas son enfant.
- `/settings` (`settings/+page.svelte`) ne renvoie pas vers `/settings/invoicing`.
- Libellés des quatre locales (`nav-administration` / `nav-invoicing`) : fr « Administration → Facturation »,
  de « Administration → Fakturierung », it « Amministrazione → Fatturazione », en « Administration → Invoicing ».
- « Paramètres — Facturation » (`settings-invoicing-title`) est le **titre de l'écran**, non un chemin.

## Ce qui change

Le TEXTE seulement : « Paramètres → Facturation » (et ses graphies `Paramètres > Facturation`, `\emph{Paramètres} →
\emph{Facturation}`, `\emph{Paramètres → Facturation}`, `*Paramètres → Facturation*`, de/it/en équivalents) devient
« Administration → Facturation » (et traductions). Trois messages qui disaient seulement « dans les paramètres »
(`invoice-settings-required`, ×4 locales + son repli de `InvoiceForm.svelte`) nomment désormais le chemin.

## Inventaire des sites (inventaire clos, recompté avec `git grep` de la valeur, périmètre : l'arbre suivi hors
`_bmad-output`, PDF et CHANGELOG historique)

| Famille | Fichiers | Lignes modifiées |
|---|---|---|
| Catalogues i18n ×4 | `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` | 21 par locale (84), dont 2 variantes `>` et `invoice-settings-required` |
| Replis / messages Rust | `crates/kesh-api/src/errors.rs` (16 libellés + 1 test), `crates/kesh-i18n/src/loader.rs` (2 attendus + garde G3) | — |
| Doc-comments Rust | `company_invoice_settings.rs`, `kesh-db/src/errors.rs` (2), `kesh-qrbill/src/pdf.rs` | 4 |
| Tests Rust | `company_invoice_settings_postable_e2e.rs` (2), `invoice_echeancier_e2e.rs` (3), `invoices_validate_vat.rs` (1) | 6 |
| Replis Svelte / TS | `InvoiceForm.svelte` (2), `WriteOffDialog.svelte` (2), `VatPurchaseAssistant.svelte`, `failed-proposal-label.ts` (2), `invoices/[id]/+page.svelte` | 8 |
| Tests / specs front | `failed-proposal-label.test.ts`, `invoice-validate-page.test.ts`, `settings-invoicing-page.test.ts` (5), 4 specs E2E (commentaires) | 11 |
| Manuels FR | `admin-manual.tex` (3), `user-manual.tex` (17 lignes, 18 occurrences) ; PDF `admin-manual.pdf`, `user-manual.pdf` régénérés | 20 + 2 PDF |
| Docs | `docs/api-external.md` (3), `docs/migrations-idempotence-audit.md` (1) | 4 |

Manuels DE/EN/IT (`docs/manual/{de,en,it}/`) : un `README.md` seul, aucun chemin à corriger. `README.md` racine et
`website/` : aucune occurrence. Brochure : aucune occurrence (PDF non touché).

## Hors de cette story (signalé, non traité)

La même confusion existe pour d'autres entrées du groupe *Administration* (« Paramètres → Exercices », « → Taux de
TVA », « → Utilisateurs », « → Modèles d'e-mail », « → Clés API », « → Rappels débiteurs ») : une vingtaine de sites
au manuel et dans l'application. Chacun demande de vérifier la barre latérale avant correction ; issue à ouvrir par
l'orchestrateur.

## Choix (registre `epic-30-choix-autonomes-303.md`)

C-30-3-1 texte seulement, aucun lien ajouté sur `/settings` ; C-30-3-2 le titre d'écran « Paramètres — Facturation »
reste ; C-30-3-3 la garde G3 de `loader.rs` calcule le chemin depuis `nav-administration` et `nav-invoicing`.

## Gates

- `cargo fmt --all -- --check` : vert (après `cargo fmt --all`, qui a recoupé une assertion de `errors.rs`).
- `cargo clippy --workspace --all-targets -- -D warnings` : vert.
- `cargo nextest run` ciblé, `kesh-i18n` (tout) + `kesh-api` (`textes_coherents`, `company_invoice_settings_postable_e2e`,
  `invoice_echeancier_e2e`, tests de la lib) : **565 exécutés, 565 passés, 1 ignoré**. Premier essai : 1 échec
  (`a_rounding_gap_without_a_usable_account_is_a_dedicated_400`, assertion `contains("Paramètres")`, graphie que le
  premier grep n'avait pas), corrigé.
- Front : `npm run check` 0 erreur (29 avertissements préexistants) ; `npm run lint-i18n-ownership` PASS ;
  `npm run test:unit` **1752 tests, 151 fichiers, tous passés**. `npm run build` et l'E2E complet **non exécutés** :
  aucun comportement ne change, seuls des libellés et des commentaires d'E2E.
- `make fr` (sous verrou) : trois PDF régénérés ; contrôle aplati (`pdftotext … | tr`) : 18 occurrences
  « Administration → Facturation » au manuel utilisateur, 3 à l'administrateur, **0** « Paramètres → Facturation ».
  Le PDF de la brochure, inchangé en contenu, n'est pas versionné dans cette story.
- Gate Rust complet (`scripts/test-fast.sh`) et E2E complet : non exécutés (aucune logique touchée) ; la PR passe la CI.

## Change Log

- 2026-10-16 — développement : sites ci-dessus, garde G3 réécrite, fiche, registre, CHANGELOG `[0.14.0]` § Corrigé,
  ligne d'epic. Commits `refs #585`.
