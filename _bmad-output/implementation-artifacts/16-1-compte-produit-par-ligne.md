# Story 16.1 : Compte de produit par ligne de facture

## Status

ready-for-dev

## Story

**As a** indépendant / PME / fiduciaire qui facture des **natures de prestations différentes** (honoraires, prestations de services, marchandises, produits annexes),
**I want** choisir le **compte de produit** de chaque **ligne** de facture, avec repli sur le compte de produit par défaut de la société quand je ne précise rien,
**so that** l'écriture comptable générée à la validation **ventile le crédit produit sur les bons comptes** au lieu de tout créditer sur un compte unique — ce qui rend mon compte de résultat exploitable sans reclassement manuel a posteriori.

Issue : **#152**. Rattaché au CR **#265** (compte de revenu par ligne). Socle de l'Epic 16 « Facturation avancée ».

---

## Contexte

### Provenance du contexte d'epic (à lire avant de chercher ailleurs)

**Il n'existe pas d'`epic-16.md`.** `_bmad-output/planning-artifacts/epics.md` est explicitement déclaré **obsolescent** (bandeau en tête + tableau de correspondance de la renumérotation, décision CR-009 #61 fermée `not planned`) et **ne contient aucun Epic 16** — son « Epic 14 » correspond à l'Epic 15 actuel. La source de vérité de l'Epic 16 est donc :

- `sprint-status.yaml` (séquence des stories + motif de l'ordre) ;
- les issues GitHub **#152** (cette story), **#144** (16-2), **#151** (16-3, reliquat) ;
- le CR **#265**.

Ne pas chercher de spec d'epic ailleurs, et ne pas se fier à la numérotation d'`epics.md`.

### Ce qui existe aujourd'hui

- **`invoice_lines`** (`crates/kesh-db/migrations/20260416000001_invoices.sql:41`) : `id`, `invoice_id`, `position`, `description`, `quantity`, `unit_price`, `vat_rate`, `line_total`, `created_at`. **Aucune colonne de compte, et aucun `product_id`** — une ligne de facture n'est pas reliée au catalogue.
- **`generate_invoice_journal_lines`** (`crates/kesh-db/src/repositories/invoices.rs:1156`, doc à partir de `:1120`) produit exactement :
  - `[0]` **débit créance** = `total_ht + total_vat` (TTC) ;
  - `[1]` **crédit produit UNIQUE** = `total_ht` ;
  - `[2..N]` **crédit TVA due**, une ligne par taux dont le montant agrégé est `> 0`, triées par taux croissant (itération `BTreeMap` ASC).
- **`validate_invoice`** (`invoices.rs:1245`) lit `settings.default_revenue_account_id` (`:~1370`) et le passe au helper (`:1383-1387`). Absent → `DbError::ConfigurationRequired("default_revenue_account_id")`.
- **Résolution par rôle (14-3b)** : `crates/kesh-db/src/repositories/company_invoice_settings.rs:236` documente que `Receivable` / `DefaultRevenue` / `Payable` servent à **pré-remplir `company_invoice_settings` à la finalisation de l'onboarding**, via la colonne générée `singleton_role`. Ce n'est **pas** le chemin de résolution au posting.
- **`AccountAutocomplete.svelte`** (`frontend/src/lib/features/journal-entries/`) : composant de sélection de compte existant, qui filtre déjà `accounts.filter(a => a.active && a.postable)` — commentaire explicite « Story 14-3b : sélecteur de SAISIE d'écriture → seuls les comptes postables ».
- **Formulaire de facture** : `frontend/src/lib/components/invoices/InvoiceForm.svelte` (noter : `lib/components/`, **pas** `lib/features/`).
- **Avoirs** : `credit_note_lines` (`migrations/20260627000001_credit_notes.sql:63`) est une **copie snapshot** des lignes de facture (mêmes colonnes, **sans compte**). `generate_credit_note_journal_lines` (`credit_notes.rs:139`) est documenté comme l'**« inverse exact »** du helper facture et reçoit `lines: &[(Decimal, Decimal)]` = `(line_total, vat_rate)` + **un seul** `revenue_account_id`, lui-même relu depuis `settings.default_revenue_account_id` (`credit_notes.rs:280-282`).
- **Export CSV** : `serialize_invoice_lines_csv` (`crates/kesh-api/src/exports/csv_tables.rs:459`).
- **Re-validation au posting** : `validate_invoice` re-valide le **projet analytique** au moment de poster (`invoices.rs:~1290`, Story 19-4) parce qu'il peut avoir été archivé entre le brouillon et la validation. **C'est le patron à copier** pour le compte de ligne.

### Ce qui n'existe PAS (deltas à construire)

1. Colonne `revenue_account_id` sur `invoice_lines` (et sur `credit_note_lines`, cf. D5).
2. Champ `revenueAccountId` dans `CreateInvoiceLineRequest` (`crates/kesh-api/src/routes/invoices.rs:66`, aujourd'hui 4 champs) et dans la réponse.
3. Ventilation du crédit produit par compte dans le helper d'écriture.
4. Sélecteur de compte par ligne dans `InvoiceForm.svelte`.
5. Validation du compte choisi (type `Revenue`, `postable`, `active`, même société), à la saisie **et** au posting.

### Ce qui n'est PAS dans 16-1

- Le compte porté par la **fiche produit** du catalogue → **16-2 (#144)**, qui viendra pré-remplir ce champ. 16-1 ne touche pas `products`.
- Le **compte de charge** par ligne de facture **fournisseur** → hors périmètre (#265 second volet, à séquencer).
- Les coordonnées émetteur / n° client sur le PDF → **16-3 (#151)**.
- Aucun changement du **calcul de TVA** ni de la **présentation du PDF**.

---

## Décisions de conception

### D1 — Le repli est `company_invoice_settings.default_revenue_account_id`, PAS le rôle `DefaultRevenue`

Une ligne sans compte se poste sur `settings.default_revenue_account_id`, exactement comme aujourd'hui.

**Motif** : la colonne est la source de vérité au runtime et reste **configurable par l'utilisateur** dans les Réglages ; le rôle `DefaultRevenue` ne sert qu'à **pré-remplir cette colonne à l'onboarding** (14-3b, cf. docstring `company_invoice_settings.rs:236`). Résoudre par rôle au posting serait un **changement de comportement non demandé** : il écraserait silencieusement le choix d'un utilisateur qui a délibérément pointé un autre compte dans les Réglages. Le message d'erreur `ConfigurationRequired("default_revenue_account_id")` reste donc inchangé.

### D2 — `revenue_account_id` NULLABLE, liaison tardive (NULL n'est jamais matérialisé à la création)

La colonne est `BIGINT NULL`. `NULL` signifie « utiliser le défaut société **au moment de la validation** ». On ne copie pas le défaut dans la ligne à la création du brouillon.

**Motif** : conserve le comportement actuel à l'identique pour toute facture existante ou créée sans préciser de compte (un brouillon suit le défaut en vigueur au posting). Matérialiser le défaut à la création introduirait une divergence de comportement pour les brouillons de longue durée, non demandée par l'issue. Le compte effectivement utilisé est de toute façon tracé dans l'**écriture comptable** générée, qui est la pièce probante.

### D3 — Validation du compte : type `Revenue` + `postable` + `active` + même société, contrôlée DEUX fois

- **À la saisie** (création et modification de facture) : rejet `400` avec un code d'erreur dédié si le compte n'est pas un compte de la société, ou n'est pas de type `Revenue`, ou n'est pas `postable`, ou n'est pas `active`.
- **Au posting** (`validate_invoice`) : **re-validation**, exactement comme la re-validation du projet analytique de la Story 19-4 (`invoices.rs:~1290`) — un compte peut avoir été **archivé** ou passé **non-postable** entre le brouillon et la validation, et toute nouvelle entrée au grand livre doit viser un compte valide.

**Motif** : la garde de postabilité de 14-3b n'existe que sur la saisie d'écriture **manuelle** ; les flux automatiques (facturation) n'y sont pas soumis. Sans re-validation au posting, une facture validée pourrait créditer un compte archivé — et c'est précisément le trou que 19-4 a fermé pour les projets.

### D4 — Ventilation : `BTreeMap<i64, Decimal>` par compte, montants `> 0`, tri par `account_id`

Le helper agrège les `line_total` par compte effectif (compte de la ligne, ou défaut société si `NULL`), n'émet une ligne de crédit que si le montant agrégé est `> 0`, et itère en ordre croissant d'`account_id`.

**Motif** : reproduit exactement le patron déjà en place pour la TVA par taux (`BTreeMap` ASC), donc déterminisme des écritures et tests stables. Filtrer `> 0` évite une ligne d'écriture à zéro pour une ligne de facture de montant nul, cohérent avec le traitement des taux de TVA.

**Preuve d'équilibre** (à reprendre dans la docstring) : le débit créance vaut `total_ht + total_vat`. La ventilation ne touche **que** la répartition de `total_ht`, et `Σ_comptes (Σ_lignes line_total) = Σ_lignes line_total = total_ht` **exactement** — aucun arrondi n'intervient sur le HT (l'arrondi half-up par ligne ne concerne que la TVA, cf. `kesh_core::accounting::vat::line_vat_amount`). L'équilibre garanti par construction dans la docstring actuelle est donc **préservé**, et il faut le dire explicitement plutôt que de le laisser déduire.

### D5 — Les avoirs sont DANS le périmètre : `credit_note_lines` porte aussi le compte, la contre-passation ventile

`credit_note_lines` reçoit également `revenue_account_id BIGINT NULL`, copié depuis la ligne de facture lors de la création de l'avoir. `generate_credit_note_journal_lines` prend `lines: &[(Decimal, Decimal, Option<i64>)]` et **débite par compte**.

**Motif — c'est le point le plus grave de la story, et l'issue #152 ne le mentionne pas.** `generate_credit_note_journal_lines` est documenté comme l'« **inverse exact** » du helper facture et relit aujourd'hui `settings.default_revenue_account_id`. Livrer la ventilation côté facture **sans** toucher l'avoir produirait ceci : une facture créditée sur 3200 serait extournée sur 3000. Les deux écritures ne s'annulent plus → **résidu permanent au crédit de 3200 et au débit de 3000**, invisible au bilan (l'équation reste équilibrée) mais **faux au compte de résultat**. C'est une corruption comptable silencieuse ; elle doit être fermée dans la même story que la cause.

La colonne est portée par le snapshot (plutôt que relue depuis la facture d'origine) pour garder l'avoir **auto-descriptif** — c'est déjà le parti pris de `credit_note_lines`, qui duplique toutes les autres colonnes de ligne.

### D6 — Réutiliser `AccountAutocomplete`, en le déplaçant vers `lib/components/`

Ne PAS créer un second sélecteur de compte. `AccountAutocomplete.svelte` fait déjà le travail, filtre déjà `active && postable` (14-3b) et a ses tests.

Il vit dans `lib/features/journal-entries/` alors que `InvoiceForm.svelte` est dans `lib/components/invoices/`. **Le déplacer vers un emplacement partagé** (`lib/components/accounts/AccountAutocomplete.svelte`), en mettant à jour les imports de `JournalEntryForm.svelte` et `VatPurchaseAssistant.svelte` ainsi que le chemin du fichier de test.

**Motif** : `npm run lint-i18n-ownership` est un **gate CI** sur les violations i18n cross-feature. Importer un composant de `features/journal-entries` depuis une facture crée exactement le couplage que ce lint surveille. Le déplacement est mécanique et rend la réutilisation légitime. **Vérifier le lint après déplacement** — c'est le risque principal de cette tâche.

### D7 — Un compte persisté devenu non-postable ou archivé ne doit PAS disparaître silencieusement du sélecteur

Le sélecteur filtre `active && postable`. Si une ligne de brouillon porte un compte qui a depuis été archivé ou rendu non-postable, il ne figure plus dans la liste : le champ s'afficherait **vide**, et un simple enregistrement du formulaire le remplacerait par `NULL` sans que l'utilisateur voie quoi que ce soit.

**C'est exactement la dette #271** (`<select>` filtré `postable` → nullification silencieuse), constatée en 14-3b comme limitation L5. Ne pas la reproduire ici : la valeur persistée doit rester **affichée et signalée** (libellé + mention d'invalidité), et l'enregistrement doit **exiger un choix explicite** plutôt que de nullifier en silence.

**Motif** : sur un brouillon de facture, une nullification silencieuse ne casse rien visiblement — la facture se validera sur le compte par défaut. L'utilisateur croira avoir ventilé alors que non. Coût d'un oubli : erreur comptable silencieuse, la pire catégorie.

---

## Acceptance Criteria

### A. Base de données

- **AC1** — Migration ajoutant `revenue_account_id BIGINT NULL` à `invoice_lines`, FK vers `accounts(id)` en `ON DELETE RESTRICT` (un compte référencé par une ligne ne doit pas pouvoir disparaître), + index sur la colonne.
- **AC2** — Même ajout sur `credit_note_lines` (D5).
- **AC3** — Migration **non-breaking** (`ADD COLUMN` nullable + index) → **pas** de bump `kesh_version_min_required` (politique P1/P2). Le vérifier explicitement.
- **AC4** — Ligne ajoutée au tableau `docs/migrations-idempotence-audit.md` avec verdict et justification (garde-fou **P5** — son absence est un finding MEDIUM en code review).

### B. Backend — modèle et API

- **AC5** — `InvoiceLine` (entité), `LINE_COLUMNS` (`invoices.rs:39`), `insert_lines` (`:386`), `fetch_lines` (`:425`) et `invoice_snapshot_json` (`:51`) portent le nouveau champ. Le snapshot d'audit inclut le compte.
- **AC6** — `CreateInvoiceLineRequest` (`routes/invoices.rs:66`) accepte `revenueAccountId: Option<i64>`; la réponse de lecture le restitue. Omission = `NULL` (rétro-compatibilité totale des clients existants, dont les intégrations PAT).
- **AC7** — Validation à la saisie (création **et** modification) : compte de la société, type `Revenue`, `postable`, `active`. Sinon `400` avec un code d'erreur canonique dédié (pas de `format!` interpolé).
- **AC8** — Re-validation au posting dans `validate_invoice`, sur le modèle 19-4. Un compte devenu invalide entre brouillon et validation fait échouer la validation avec un message actionnable nommant la ligne concernée.

### C. Backend — moteur comptable

- **AC9** — `generate_invoice_journal_lines` ventile le crédit produit : une ligne de crédit **par compte effectif**, montants `> 0`, tri `account_id` ASC (D4). La ligne `[0]` débit créance et les lignes TVA par taux sont **inchangées**.
- **AC10** — Docstring du helper mise à jour, **preuve d'équilibre incluse** (D4).
- **AC11** — `generate_credit_note_journal_lines` débite par compte, en miroir exact (D5). Sa docstring « inverse exact » reste vraie.
- **AC12** — Une facture dont **toutes** les lignes sont sans compte produit **exactement la même écriture qu'avant** la story (non-régression, à prouver par un test qui compare les lignes d'écriture générées).
- **AC13** — Le **décompte TVA** et la réconciliation rapport ↔ grand livre restent justes : la ventilation ne touche pas les lignes de TVA, à vérifier par un test sur une facture multi-comptes **et** multi-taux.

### D. Frontend

- **AC14** — `AccountAutocomplete` déplacé vers un emplacement partagé, imports et test mis à jour, `npm run lint-i18n-ownership` **PASS** (D6).
- **AC15** — `InvoiceForm.svelte` propose un sélecteur de compte par ligne, optionnel, avec indication visible du repli (« défaut société ») quand rien n'est choisi.
- **AC16** — Comportement D7 : un compte persisté invalide reste affiché et signalé, et l'enregistrement exige un choix explicite au lieu de nullifier.
- **AC17** — Types TS et client API alignés (`revenueAccountId` optionnel).

### E. i18n, exports, doc

- **AC18** — Toutes les chaînes nouvelles dans les **4 locales** (FR/DE/IT/EN). Aucun libellé codé en dur.
- **AC19** — `serialize_invoice_lines_csv` (`csv_tables.rs:459`) et l'export des lignes d'avoir exposent la nouvelle colonne ; en-têtes mis à jour. Vérifier les **compteurs de colonnes/tables** des tests d'export et de sauvegarde complète (piège connu : l'ajout d'une colonne ou d'une table casse des assertions de compteur).
- **AC20** — CHANGELOG `[Non publié]` : entrée orientée utilisateur. README « Fonctionnalités » à compléter si le comportement devient visible.

### F. Tests & gate

- **AC21** — Tests unitaires du helper : mono-compte (non-régression AC12), multi-comptes, multi-comptes × multi-taux, ligne à montant nul filtrée, ordre déterministe.
- **AC22** — Tests d'intégration : validation d'une facture ventilée, avoir sur facture ventilée (**les deux écritures s'annulent compte par compte** — c'est le test qui protège D5), compte invalide à la saisie, compte devenu invalide au posting.
- **AC23** — Tests frontend : sélection par ligne, repli affiché, cas D7.
- **AC24** — Gate « Test Locally First » complet vert (4 checks backend + 4 frontend).

---

## Tasks / Subtasks

- [ ] **T1** — Migration `invoice_lines.revenue_account_id` + `credit_note_lines.revenue_account_id` + index + FK RESTRICT ; ligne dans `docs/migrations-idempotence-audit.md` (AC1-AC4).
- [ ] **T2** — Entités + `LINE_COLUMNS` + `insert_lines` / `fetch_lines` + snapshot d'audit, côté factures et avoirs (AC5).
- [ ] **T3** — API : `CreateInvoiceLineRequest` + réponse + validation à la saisie avec code d'erreur canonique (AC6, AC7).
- [ ] **T4** — `generate_invoice_journal_lines` : ventilation `BTreeMap` + docstring avec preuve d'équilibre (AC9, AC10).
- [ ] **T5** — Re-validation au posting sur le modèle 19-4 (AC8).
- [ ] **T6** — `generate_credit_note_journal_lines` en miroir + copie du compte à la création de l'avoir (AC11).
- [ ] **T7** — Déplacement d'`AccountAutocomplete` vers un emplacement partagé + mise à jour des imports/test + `lint-i18n-ownership` (AC14).
- [ ] **T8** — `InvoiceForm.svelte` : sélecteur par ligne, repli visible, cas D7 (AC15-AC17).
- [ ] **T9** — i18n 4 locales (AC18).
- [ ] **T10** — Exports CSV + compteurs (AC19).
- [ ] **T11** — Tests unitaires, intégration, frontend (AC21-AC23).
- [ ] **T12** — Doc-sync CHANGELOG / README (AC20) + gate complet (AC24).

**Ordre conseillé** : T1 → T2 → T4 (le helper d'abord, testable en isolation) → T3 → T5 → T6 → T7 → T8 → T9 → T10 → T11 → T12.

---

## Dev Notes

### Ancres ground-truth (vérifiées au 2026-07-26, commit `ef6cdf52`)

| Élément | Emplacement |
|---|---|
| Schéma `invoice_lines` | `crates/kesh-db/migrations/20260416000001_invoices.sql:41` |
| Schéma `credit_note_lines` | `crates/kesh-db/migrations/20260627000001_credit_notes.sql:63` |
| `LINE_COLUMNS` | `crates/kesh-db/src/repositories/invoices.rs:39` |
| `invoice_snapshot_json` | `invoices.rs:51` |
| `insert_lines` / `fetch_lines` | `invoices.rs:386` / `:425` |
| Doc + code du helper d'écriture | `invoices.rs:1120-1156` (doc), `:1156` (fn) |
| Lignes d'écriture construites | `invoices.rs:1186-1200` |
| `validate_invoice` | `invoices.rs:1245` ; lecture du défaut `~:1370` ; appel helper `:1383-1387` |
| Re-validation projet au posting (patron D3) | `invoices.rs:~1290` |
| Helper avoir « inverse exact » | `crates/kesh-db/src/repositories/credit_notes.rs:139` |
| Avoir : lecture du défaut à remplacer | `credit_notes.rs:280-282` ; passage `:327` |
| `CreateInvoiceLineRequest` | `crates/kesh-api/src/routes/invoices.rs:66` |
| Résolution par rôle = prefill onboarding | `crates/kesh-db/src/repositories/company_invoice_settings.rs:236` |
| Export CSV des lignes | `crates/kesh-api/src/exports/csv_tables.rs:459` |
| Sélecteur de compte à réutiliser | `frontend/src/lib/features/journal-entries/AccountAutocomplete.svelte:32-34` |
| Formulaire de facture | `frontend/src/lib/components/invoices/InvoiceForm.svelte` |

### Pièges, par ordre de coût

1. **L'avoir (D5)** — le plus coûteux si oublié : corruption comptable silencieuse, équation du bilan toujours équilibrée, donc **aucun signal**. Le test AC22 « les deux écritures s'annulent compte par compte » est le garde-fou.
2. **La nullification silencieuse (D7)** — l'utilisateur croit avoir ventilé. Même famille : faux sans bruit.
3. **`lint-i18n-ownership` après déplacement du composant (D6)** — gate CI, échec bruyant mais coûte un cycle.
4. **Compteurs d'export / sauvegarde (AC19)** — l'ajout d'une colonne casse des assertions de comptage ; piège déjà rencontré sur ce dépôt.
5. **Non-régression mono-compte (AC12)** — la ventilation ne doit rien changer quand personne ne l'utilise. C'est ce qui rend la migration sûre pour les bases existantes, dont l'instance de production.

### Propagation post-patch (§ CLAUDE.md)

Après chaque patch de remédiation, **grep le symptôme sur tout le dépôt** avant la passe suivante — pas seulement le site corrigé. Sur cette story, les symptômes à balayer sont `revenue_account_id`, `LINE_COLUMNS`, `generate_.*journal_lines`, et les compteurs d'export.

### References

- Issue **#152** (cette story), **#144** (16-2), **#265** (CR d'origine).
- Dette **#271** — le patron de nullification silencieuse à ne pas reproduire (D7).
- Stories antérieures : **14-3a/14-3b** (rôles, `postable`, résolution par rôle), **19-4** (re-validation au posting), **12-1** (avoirs et contre-passation).
- `CLAUDE.md` : politique de migration (P1-P5), pattern batch, Review Iteration Rule, propagation post-patch.

---

## Questions ouvertes pour Guy (à trancher avant `dev-story`, ou à confirmer en `validate`)

1. **D5 est-il bien dans le périmètre de 16-1 ?** Il élargit la story aux avoirs. L'alternative — livrer la ventilation côté facture seule et différer l'avoir — est **à écarter selon moi** : elle laisse une corruption comptable silencieuse en production entre les deux stories. Mais c'est un arbitrage de périmètre, donc ton appel.
2. **Comportement D7 précis** : afficher le compte invalide grisé avec un avertissement, ou bloquer l'enregistrement du formulaire tant qu'un compte valide n'est pas choisi ? Je propose l'avertissement visible + blocage à l'enregistrement de la ligne concernée seulement.
3. **Fiche produit (16-2)** : confirmer que le compte du produit ne servira qu'à **pré-remplir** la ligne côté frontend (donc aucune reprise rétroactive sur les factures existantes).

---

## Dev Agent Record

### Agent Model Used

_(à compléter par `dev-story`)_

### Debug Log References

### Completion Notes List

### File List
