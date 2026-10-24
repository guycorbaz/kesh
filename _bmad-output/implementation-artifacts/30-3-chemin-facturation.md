# Story 30-3 — Le chemin de la facturation

Status: done

*(Annotation du 2026-10-24, Story 32-0 — action A2 de la rétrospective de l'Epic 31 : l'en-tête disait `review` ; la
revue est terminée et la story fusionnée par la PR #692, `4663f60b` — `sprint-status.yaml` porte `done`.)*

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

Rien. *Annotation du 2026-10-16 (revue P1, F4)* : la version de cette section écrite au premier développement
affirmait que « Paramètres → Exercices / Taux de TVA / Utilisateurs / Modèles d'e-mail / Clés API / Rappels débiteurs »
avaient le même défaut et demandait une issue. C'était faux, établi ensuite au code : `/settings` porte une section
pour chacun (`settings/+page.svelte` : utilisateurs `:507`, exercices `:585`, taux de TVA `:598`, modèles d'e-mail
`:611`, clés API `:489`, rappels débiteurs `:624`). Ces chemins sont vrais ; **aucune issue à ouvrir**. Seule
« Facturation » n'a pas de carte sur `/settings`.

La passe d'extension a seulement écrit en entier « Exercices » → « Exercices comptables » (fr, it) pour tenir le titre
de la section : `error-fiscal-year-missing` et `opening-balances-locked-no-fiscal-year` (fr, it), `notify.ts`,
`settings/opening-balances/+page.svelte`, `admin-manual.tex:1418`. La garde `les_chemins_cites_aux_catalogues_existent`
(G3-bis) vérifie mécaniquement, dans les 4 catalogues, que tout « Administration → X » ou « Paramètres → X » désigne un
lieu réel ; son angle mort (listes en dur, catalogues seuls, graphie « > » désormais refusée) est écrit dans sa
doc-comment.

Forme sans flèche (revue P1, F1) : `user-manual.tex:292-294` disait « Les Paramètres regroupent … la facturation ».
Corrigé le 2026-10-16 : la phrase renvoie à *Administration → Facturation*. Recherche des formes sans flèche
(`paramètres|réglages|Einstellungen|Impostazioni|Settings` à moins de 60 caractères de `facturation|Fakturierung|
Fatturazione|Invoicing`, et l'inverse « facturation … dans les paramètres ») sur les `.tex` FR, les 4 catalogues, les
replis, `api-external.md`, README et `website/` : le reste ne désigne pas un lieu (libellés d'audit « Réglages de
facturation », « les paramètres de facturation » comme objet).

## Choix (registre `epic-30-choix-autonomes-303.md`)

C-30-3-1 texte seulement, aucun lien ajouté sur `/settings` ; C-30-3-2 le titre d'écran « Paramètres — Facturation »
reste ; C-30-3-3 la garde G3 de `loader.rs` calcule le chemin depuis `nav-administration` et `nav-invoicing`.

## Gates

**Décomptes recomptés le 2026-10-16** depuis `git diff -U0 fc3a4f7d..fb2bee28` (lignes ajoutées), périmètre : les trois
commits de développement, avant la remédiation de revue. Catalogues : fr 23, de 21, it 23, en 21 = **88** (84 de la
passe 1 + 4 « Exercices comptables » fr/it de fb2bee28) ; `admin-manual.tex` **4** lignes (3 facturation + `:1418`) ;
`user-manual.tex` **17** lignes ; Svelte/TS hors tests **10** lignes dans 7 fichiers (`InvoiceForm.svelte`,
`WriteOffDialog.svelte`, `VatPurchaseAssistant.svelte`, `failed-proposal-label.ts`, `invoices/[id]/+page.svelte`,
`notify.ts`, `opening-balances/+page.svelte`) ; 30 fichiers versionnés hors planning et PDF. *(Les chiffres « 84 »,
« (3) » et « 8 » écrits au premier développement valaient à la date de a4db4a08 ; ils ne tenaient pas compte de
fb2bee28.)*

**Gate des commits 6a6482cd et a4db4a08 (périmètre : avant fb2bee28)** : `cargo fmt`, `cargo clippy --workspace`
verts ; nextest ciblé `kesh-i18n` + `kesh-api` (`textes_coherents`, `company_invoice_settings_postable_e2e`,
`invoice_echeancier_e2e`, lib) 565 passés, 1 ignoré (1 échec au premier essai, assertion `contains("Paramètres")`,
corrigé) ; `npm run check`, `lint-i18n-ownership`, `test:unit` 1752 tests / 151 fichiers verts.

**Gate COMPLET au dernier commit de code, fb2bee28** (base `kesh_303` remise à zéro) :
- `scripts/test-fast.sh` : 3825 exécutés, 3825 passés, 6 ignorés (G3-bis comprise).
- `npm run check` 0 erreur (29 avertissements préexistants) ; `lint-i18n-ownership` PASS ; `test:unit` **1752 tests,
  151 fichiers** ; `npm run build` OK.
- `make fr` sous verrou ; PDF aplatis : 0 « Paramètres → Facturation », « Administration → Facturation » 3 fois à
  l'administrateur et 18 à l'utilisateur.
- E2E complet (port 3053, base `kesh_e2e_303`, sous verrou) : **301 passés, 7 échoués, 17 ignorés**, 11,9 min ; les 7
  échecs sont exactement ceux de la KF-029 (#97) de `docs/testing.md` (`mode-expert:26`, `:41`,
  `onboarding-path-b:65`, `:92`, `onboarding:57`, `:77`, `:150`).

**Remédiation de revue P1 (commit suivant)** : touche `loader.rs` (graphie « > » refusée par G3-bis), un `.tex` et
son PDF, des `.md`. Gate ciblé : `cargo fmt --check` vert, `cargo clippy --workspace --all-targets -D warnings` vert,
nextest `kesh-i18n` + `textes_coherents` **56 exécutés, 56 passés**. Vitest non rejoué : aucun fichier front n'est
touché par la remédiation. PDF utilisateur régénéré sous verrou et contrôlé aplati (19 « Administration →
Facturation », phrase de `:292` corrigée, 0 « Paramètres → Facturation »). Le gate complet de fb2bee28 vaut pour tout
le code de production et de test de la story ; la remédiation n'ajoute que l'assertion « > » de G3-bis.

## Change Log

- 2026-10-16 — développement : sites ci-dessus, garde G3 réécrite, fiche, registre, CHANGELOG `[0.14.0]` § Corrigé,
  ligne d'epic. Commits `refs #585`.
- 2026-10-20 — report de la branche sur `main` (1aca02d2, merge de la 30-2b) : les cinq commits de la 30-3 rejoués sans
  perte (CHANGELOG, sprint-status, registres, catalogues i18n, manuels `.tex` réunis des deux côtés ; PDF des manuels
  régénérés, commit `8408f590`). Aucun site de la 30-2b ne porte « Paramètres → Facturation » (grep de la valeur sur le
  worktree, hors entrées historiques du CHANGELOG). Gates au dernier commit de code, sous verrou : `scripts/test-fast.sh`
  3831 exécutés, 3831 passés, 6 ignorés ; `npm run check` 0 erreur (29 avertissements) ; `lint-i18n-ownership` vert ;
  `npm run test:unit` 1794 passés (155 fichiers) ; `npm run build` vert ; E2E complet 304 passés, 17 ignorés, 7 échecs, les
  sept de la KF-029 (#97) de `docs/testing.md` (mode-expert 26 et 41, onboarding-path-b 65 et 92, onboarding 57, 77, 150).
