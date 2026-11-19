# Epic 49 — Types de lignes de la facture client (#1242)

*Kickoff du 2026-11-18, tenu en autonomie par l'orchestrateur (Opus 5.5), worktree `kesh-k0200`, branche
`chore/kickoff-0.20.0`. Code lu à `main` = tag **v0.19.0** (`773df80da`). Release **0.20.0** : publication le **vendredi
2026-11-27**, coupure le **jeudi 2026-11-26 au soir** (plan : `release-0.20.0.md`). Décisions de Guy du 2026-11-18 ;
demande et réponses de Guy du 2026-11-17 au corps de #1242. Révisé le même jour sur les réponses de Guy aux points
ouverts du kickoff : sous-totaux repoussés, ordre de fusion avec la 48-13, règlements de l'orchestrateur ; puis sur le
complément de Guy du même jour (factures réelles déjà émises, § *Contraintes de production*) ; puis sur le découpage
préventif de la 49-1 en 49-1a à 49-1d (`kesh-gate-logs/e491/decoupage-49-1.md`) ; puis le 2026-11-19 : 49-4 ajoutée par Guy, #1400
rattachée à la 49-4 et règle « la 49-1c ne sort pas sans la 49-4 » (décisions de Guy du 2026-11-19).*

## Objet

La fonctionnalité de la 0.20.0 (kesh:D14 : « une ou deux fonctionnalités »). Guy émettra ses **vraies factures clients**
avant la fin novembre (#1242, P2, échéance souhaitée fin novembre 2026) ; une facture doit pouvoir porter autre chose
qu'une suite de lignes chiffrées :

- **ligne produit** : comme aujourd'hui, un article du catalogue dont les valeurs sont **recopiées** dans la ligne,
  modifiables, **sans lien** gardé vers le catalogue ;
- **ligne sans produit** : un texte, une quantité, un prix unitaire, comme aujourd'hui (taux de TVA et compte de produit
  pris des réglages par défaut, modifiables) ;
- **ligne de commentaire** : du texte seul, sans montant, **hors du total et des écritures**, plaçable n'importe où ;
- **titre de section** : accepté par Guy (« bonne idée »), sans montant ;
- **sous-total** : accepté sur le principe, mais **repoussé à une release ultérieure** (décision de Guy du 2026-11-18) :
  Beat, contributeur, tient une comptabilité **assujettie à la TVA** ; un sous-total doit être juste en HT et TTC et par
  taux, ou attendre. Issue #1330 (sous-totaux de section, milestone « Plus tard ») ;
- **réordonnancement** des lignes à la main **avant la validation** (brouillon).

Les **avoirs** reprennent les lignes de commentaire (et les titres) de la facture d'origine. Le **PDF** rend commentaires
et titres (pleine largeur, sans colonnes de quantité ni de prix).

C'est une **épique de fonctionnalité** : elle n'est pas tenue à la borne de six stories. Elle en compte **six** depuis
le découpage préventif de la 49-1 (49-1a, 49-1b, 49-1c, 49-1d, 49-2, 49-3).

## Ce qui se lit au code (`773df80da`)

- `invoice_lines` (`20260416000001_invoices.sql`) : `position INT NOT NULL`, `description VARCHAR(1000) NOT NULL`,
  `quantity`, `unit_price`, `vat_rate`, `line_total` en `NOT NULL` ; contraintes `chk_invoice_lines_quantity_positive`
  (`quantity > 0`), `chk_invoice_lines_description_not_empty`, `chk_invoice_lines_line_total_non_negative`
  (`20260416000002`), `uq_invoice_lines_position (invoice_id, position)` ; `revenue_account_id` ajouté en
  `20260727000001`. **Aucune colonne de type.**
- `credit_note_lines` (`20260627000001_credit_notes.sql`) a **la même structure et les mêmes contraintes** ; un avoir en
  est un instantané (`credit_notes.rs`, étape « (10) INSERT credit_note_lines (snapshot) », ~l. 771). Pour qu'un avoir
  reprenne commentaires et titres, **la migration touche aussi `credit_note_lines`**.
- Brouillon : `invoices::update` (`invoices.rs:1113`) **supprime et réinsère** les lignes (`DELETE FROM invoice_lines`,
  l. 1291 ; `insert_lines`, l. 471) : la position suit l'ordre du corps de la requête. Le réordonnancement (49-2) ne
  demande donc probablement **aucun changement du serveur**, ni de manipulation de `uq_invoice_lines_position`.
- Validation : `validate_invoice` (`invoices.rs:2096`) tire l'écriture de vente des lignes (compte de produit, TVA) ; la
  TVA se calcule par taux (`kesh-core/src/accounting/vat.rs`) et le rapport TVA lit `invoice_lines` par taux
  (`kesh-report/src/vat_report.rs:121`). **Une ligne sans montant ne doit produire ni ligne d'écriture, ni ventilation de
  TVA, ni « taux 0 % » fantôme** : chaque lecteur de `invoice_lines` filtre par type ou est montré neutre.
- Autres lecteurs de `invoice_lines` à inventorier par la fiche (liste close des sites, `CLAUDE.md` § *Inventorier les
  sites NON RÉSOLUS*) : `kesh-db` (`invoice_settlements_write.rs`, `journal_entries.rs`, `letterings/open_items.rs`,
  `supplier_invoices.rs`, `test_fixtures.rs`, `backup.rs`, `post_restore.rs`), `kesh-api` (`routes/invoices.rs`,
  `credit_notes.rs`, `invoice_pdf_service.rs`, `invoice_email.rs`, `exports/csv_tables.rs`, `exports/global.rs`),
  `kesh-seed/src/pieges.rs`.
- PDF : `kesh-qrbill/src/types.rs:191` (`InvoiceLinePdf`), `kesh-qrbill/src/pdf.rs`, alimentés par
  `kesh-api/src/routes/invoice_pdf_service.rs`.
- Écran : `frontend/src/lib/components/invoices/InvoiceForm.svelte`, `ProductPicker.svelte` ;
  `frontend/src/lib/features/invoices/invoices.types.ts` (`InvoiceLineResponse`, `CreateInvoiceLineRequest`),
  `invoice-helpers.ts`, `unit-price.ts` ; pages `routes/(app)/invoices/new`, `[id]` ; avoirs :
  `features/credit-notes/`, `routes/(app)/credit-notes/`.

## Contraintes de production — des factures réelles existent déjà

**Guy a déjà créé de vraies factures client dans Kesh** (production, NAS) et les a remises sur papier à un locataire
(complément de Guy du 2026-11-18). Ces contraintes priment sur tout le reste de l'épique :

1. **La migration conserve à l'identique chaque facture et chaque avoir existants.** `invoice_lines` et
   `credit_note_lines` **ne gardent aucun lien** vers le catalogue (pas de colonne d'article ; la ligne recopie les
   valeurs) : « produit » et « sans produit » sont **le même enregistrement** et ne diffèrent qu'à l'écran. La fiche de la
   49-1a en tire **un type stocké unique pour les lignes chiffrées, `priced`**, que reçoit toute ligne existante, rendue
   exactement comme aujourd'hui — aucune origine n'est devinée. Totaux, TVA, écritures et numéros **inchangés**.
2. **Triage P7** : si la migration écrit des données (rétro-remplissage du type), elle se trie au registre
   `POST_RESTORE_BACKFILLS` ou à `EXEMPT_MIGRATIONS` (`crates/kesh-db/src/post_restore.rs`), avec sa justification.
   **Un import d'une sauvegarde 0.19.0 doit donner les mêmes factures** que la montée en place.
3. **Une facture déjà émise garde son PDF.** Le PDF archivé (story 25-6-b, `kesh-api/src/routes/issued_invoice_pdf.rs`,
   gelé au premier téléchargement) reste la référence. Si le rendu des lignes change, une facture émise avant la 0.20.0
   doit se **réimprimer à l'identique**, ou son PDF archivé être servi tel quel. Deux chemins à couvrir : une facture
   validée dont le PDF n'a pas encore été gelé (il le sera avec le code 0.20.0), et le regel explicite
   (`refreeze_invoice_pdf`). **La fiche le prouve par un test** : rendu avant / après d'une facture validée de la 0.19.0.
4. **Recette de montée obligatoire, avec des factures réelles simulées** (gate de la release, plan § *Gates*) : base
   peuplée par la 0.19.0 avec des factures client validées, payées et non payées, un avoir, une facture réglée en groupe ;
   après montée, comparaison des totaux, des écritures et du PDF de chacune. Jamais sur la base du NAS.

## La migration — la seule de la 0.20.0

Portée par la **49-1a**, elle pose **d'emblée trois types stockés** — `priced` (ligne chiffrée : produit ou sans
produit), `comment`, `heading` (titre) — et leurs contraintes, pour que la 49-3 n'ait pas à migrer et puisse sortir à la
coupure sans emporter de migration (décisions de Guy du 2026-11-18). **Pas de type sous-total** : la fiche de la 49-1a
(D5) n'a pas établi qu'il ne préjugeait pas de son sens TVA (HT, TTC, ventilation par taux) ; il viendra avec les
sous-totaux (#1330), par un simple changement de `CHECK`, non breaking.

Exigences (le choix non breaking est **arrêté par l'orchestrateur le 2026-11-18**, sur délégation de Guy ; le reste se
précise à la fiche) :
- une colonne de type sur `invoice_lines` **et** `credit_note_lines` ; toute ligne existante reçoit un type qui la rend
  comme aujourd'hui (§ *Contraintes de production*, point 1) ; s'il faut écrire des données, triage P7 au registre
  `POST_RESTORE_BACKFILLS` ou à `EXEMPT_MIGRATIONS`, avec justification (point 2) ;
- des **contraintes par type** remplaçant `chk_*_quantity_positive`, `chk_*_description_not_empty`, et les contrôles de
  montant : une ligne chiffrée garde exactement les règles d'aujourd'hui ; un commentaire et un titre portent un texte et
  des valeurs neutres (quantité, prix, montant à 0) ;
- **non breaking** (`CLAUDE.md` § *Migration breaking policy*) — **décidé** : les colonnes `quantity`, `unit_price`,
  `vat_rate`, `line_total` restent **`NOT NULL`** ; les lignes non chiffrées y portent des **valeurs neutres** ; les
  `CHECK` sont **assouplis par type** ; **`credit_note_lines` comprise**. Pas de `DROP COLUMN`, pas de `MODIFY COLUMN`.
  Motif : rendre ces colonnes nullables ferait qu'une ligne écrite par la 0.20.0 ne se lirait plus par un binaire
  0.19.0 — cas breaking (P1, P3), qui exigerait le bump de `kesh_version_min_required` et des crates ;
  `kesh_version_min_required` reste donc à 0.17.6 ;
- **audit d'idempotence** (P5) : une ligne au tableau de `docs/migrations-idempotence-audit.md`, totaux recomptés ;
- **couplage positionnel** (P6) : `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` et chaque site inspecté ;
- **schéma de test** : `crates/kesh-db/test-schema/0001_schema_squash.sql` mis à jour ;
- **`.keshbackup` et export de souveraineté** : la nouvelle colonne voyage (`backup.rs`, `exports/global.rs`,
  `csv_tables.rs`) ; une sauvegarde 0.19.0 s'importe en 0.20.0 (lignes classées par la valeur par défaut) ;
- nom par horodatage du jour de son écriture, **postérieur** à `20261118000001_grouped_settlements` ; ce sera la
  **96ᵉ** migration (95 au tag v0.19.0) ;
- à la release : **sauvegarde complète obligatoire avant la mise à jour du NAS** (base, `.keshbackup`, dossier des
  documents), écrite en tête des notes et au manuel d'administration ; montée répétée sur une base peuplée par la 0.19.0
  (`scripts/recette-montee.sh`).

## Stories

| Clé | Objet | Migration | Dépend de |
|---|---|---|---|
| 49-1-types-de-lignes | **découpée** le 2026-11-18 (`split`) en 49-1a à 49-1d | — | — |
| 49-1a-types-de-lignes-migration-api | La migration, `LineType`, les dépôts, la validation, la reprise des avoirs, les lecteurs filtrés (TVA, validation, avoir), exports, MCP ; **API gardée** : seul `priced` est accepté, `comment` et `heading` refusés `400` ; paragraphe « Pour les intégrations » du manuel d'administration (×4) | **oui** | — |
| 49-1b-types-de-lignes-pdf | Rendu PDF d'un commentaire (pleine largeur, sans quantité ni prix), facture **et** avoir ; test avant / après d'une facture et d'un avoir de la 0.19.0 ; PDF figé servi tel quel ; regel | non | 49-1a |
| 49-1c-types-de-lignes-ecran | Sélecteur de type, ligne de commentaire, affichage dans l'avoir, i18n ×4, E2E, manuel utilisateur (FR, seul existant) ; **ouvre `comment`** dans l'API ; traite `verify_vat_rates_against_db` (ne collecter que les taux des lignes `priced`) | non | 49-1a, 49-1b |
| 49-1d-types-de-lignes-recette-montee | Gate de la release, sans code de production : peuplement de `scripts/recette-montee.sh` (factures validées payées et impayées, avoir, règlement groupé, PDF figés et non figés), **`credit_notes` et `credit_note_lines` ajoutées à l'empreinte** (`TABLES=`), `--diff-attendu` des deux tables de lignes, `docs/testing.md`, section « Passer à la 0.20.0 » du manuel d'administration (×4) | non | 49-1a |
| 49-2-reordonnancement-lignes | Réordonner les lignes d'un brouillon : ↑ / ↓, et glisser-déposer si la fiche le juge sûr | non | 49-1c |
| 49-3-titres-de-section | Titre de section, sans montant : saisie, rendu PDF, reprise dans les avoirs ; **ouvre `heading`** | non (posée par 49-1a) | 49-1a, 49-1b, 49-1c |
| 49-4-designation-longue-pdf | Ajoutée par Guy le 2026-11-19 : désignation longue sur le PDF de facture et d'avoir, retour à la ligne au lieu de la troncature à 44 caractères (#1363) ; capacité du PDF contrôlée avant la validation (#1364) ; caractères que la police n'a pas — signe moins U+2212, traits d'union U+2010/U+2011, grec, cyrillique — signalés au lieu de disparaître (#1400, rattachée le 2026-11-19, décision de Guy) ; s'applique aux factures émises à partir de la 0.20.0, le PDF archivé fait foi pour les anciennes | non | 49-1b |

La clé `49-3-titres-et-sous-totaux` du kickoff est renommée `49-3-titres-de-section` le 2026-11-18, les sous-totaux étant
repoussés (décision de Guy, #1330).

### 49-1 — Types de lignes, découpée en 49-1a à 49-1d

**Splitting préventif appliqué** (règle du `CLAUDE.md`) : la 49-1 entière touchait **neuf modules** au grain de la crate
(`kesh-db`, `kesh-api`, `kesh-qrbill`, `kesh-report`, `kesh-seed`, `kesh-i18n`, frontend `invoices` et `credit-notes`,
manuels et `docs/`, `scripts/`). Découpage en une story zéro (49-1a : la migration et le modèle, **sans rien rendre
visible**), puis le PDF (49-1b), l'écran (49-1c) et le gate de recette (49-1d). Pas de dérogation : chaque étape se teste
seule. Constat et compte : `kesh-gate-logs/e491/decoupage-49-1.md`.

**La garde d'API** (`LineType::open_to_api()`) rend chaque coupure cohérente : la 49-1a n'accepte que `priced` ; la 49-1c
ouvre `comment` ; la 49-3 ouvre `heading`. Le dépôt accepte tous les types dès la 49-1a (les tests de la 49-1b et de la
49-3 créent leurs lignes par le dépôt). La 49-1c exige que l'édition d'un brouillon **renvoie le type de chaque ligne**
(sans quoi un `PUT` ferait redevenir chiffrée une ligne de commentaire).

**Règles de sortie, propres au découpage** :
- **la 49-1a ne sort jamais sans la 49-1d** : une release qui porte une migration exige la sauvegarde d'avant la mise à
  jour au manuel d'administration et la recette de montée, toutes deux dans la 49-1d ; 49-1a et 49-1d sortent ensemble, ou
  aucune des deux ;
- **la 49-1c ne sort pas sans la 49-1b** : l'écran offrirait un type que le PDF imprimerait en `0 × 0.00 = 0.00` ;
- **la 49-3 ne sort pas sans la 49-1b et la 49-1c**.
- **la 49-1c ne sort pas sans la 49-4** (décision de Guy du 2026-11-19) : sans elle, un commentaire long rendrait une
  facture validée impossible à imprimer (#1364) — même risque qu'aujourd'hui avec de nombreuses lignes chiffrées (#1110),
  mais que la 49-1c étendrait au texte libre.

**Contrat d'API** : `quantity`, `unitPrice`, `vatRate` de `CreateInvoiceLineRequest` deviennent optionnels à la
désérialisation (obligatoires pour `priced`) ; leur absence rend `400 VALIDATION_ERROR` nommant la ligne, documenté dans
`docs/api-external.md`.

**`kesh-core` ne change pas** (écart à la déclaration du kickoff) : `vat_breakdown_by_rate` écarte déjà tout taux `<= 0`
(`kesh-core/src/accounting/vat.rs:132`).

### 49-2 — Réordonnancement

En brouillon seulement : boutons ↑ / ↓ sur chaque ligne (accessibles au clavier) ; glisser-déposer si la fiche le juge sûr
(accessibilité, tests Playwright). L'ordre envoyé au serveur devient la position (`update` réinsère les lignes) ; une
facture validée ne se réordonne pas. Frontend pour l'essentiel ; sa partie écran part de la 49-1c ; E2E du parcours
« créer toutes les lignes, puis réordonner ».

### 49-3 — Titres de section

Un titre de section est une ligne de texte **sans montant**, hors du total et des écritures, rendue en évidence sur
toute la largeur du PDF ; il se place, se déplace (49-2) et se reprend dans un avoir comme un commentaire. L'API accepte le
type `heading` à partir de cette story (retrait de la garde) ; elle dépend des 49-1a, 49-1b et 49-1c.

**Sous-totaux : hors de la 0.20.0** (décision de Guy du 2026-11-18). Beat tient une comptabilité assujettie à la TVA : un
sous-total doit être juste en HT et TTC et par taux, ou attendre. Ils feront l'objet d'une release ultérieure — issue
#1330 (sous-totaux de section, milestone « Plus tard ») ; le sens proposé par #1242 (total de la section, hors total et
écritures) y sera repris.

## Dépendances entre stories et avec l'Epic 48

```
49-1a ──► 49-1b ──► 49-1c ──► 49-2 (écran), 49-3
  └─────► 49-1d (gate ; rejouée au dernier commit de code de la release)
```

- La 49-1b et la 49-1c s'écrivent **en parallèle** une fois la 49-1a fusionnée (fichiers distincts) ; la 49-1c ne retire la
  garde qu'après la fusion de la 49-1b.
- **Avec la 48-13** : `InvoiceForm.svelte` (#1233, chargement du client). **Arbitrage de Guy du 2026-11-18 : la 48-13 se
  fusionne d'abord** ; les stories de l'Epic 49 qui touchent ce fichier (49-1c, 49-2, 49-3) partent ensuite de la branche
  à jour. La 48-15 ne touche aucun fichier de l'éditeur de lignes : elle ne bloque pas l'Epic 49.
- **Avec la PR #1327 de Beat** : elle touche `kesh-qrbill/src/largeurs.rs` et `lib.rs` (et les catalogues) ; la 49-1b en
  tient compte si elle est fusionnée avant.

## Fichiers et modules touchés (déclarés au kickoff, mis à jour au découpage)

| Story | Backend | Frontend | Docs, i18n, tests |
|---|---|---|---|
| 49-1a | `kesh-db/migrations/` (la migration), `test-schema/0001_schema_squash.sql`, `post_restore.rs` (triage P7), `backup.rs` ; entités `invoice.rs`, `credit_note.rs` ; dépôts `invoices.rs`, `credit_notes.rs`, `invoice_settlements_write.rs` et les autres lecteurs de `invoice_lines` / `credit_note_lines` ; `kesh-api` : `routes/invoices.rs` (`validate_line`, garde), `routes/credit_notes.rs`, `exports/csv_tables.rs`, `exports/global.rs`, `mcp/outils/factures.rs`, `errors.rs` ; `kesh-report/src/vat_report.rs` (un filtre) ; `kesh-seed/src/pieges.rs` (un littéral) | — | `docs/migrations-idempotence-audit.md`, `docs/api-external.md`, manuel d'administration (FR, seul existant) (« Pour les intégrations »), tests d'intégration `kesh-db` / `kesh-api` |
| 49-1b | `kesh-qrbill/src/types.rs`, `pdf.rs` ; `kesh-api/src/routes/invoice_pdf_service.rs`, `issued_invoice_pdf.rs`, `credit_notes.rs` (PDF de l'avoir) | — | test avant / après du rendu |
| 49-1c | `kesh-api/src/routes/invoices.rs` (retrait de la garde pour `comment`, `verify_vat_rates_against_db`), `routes/vat.rs` | `components/invoices/InvoiceForm.svelte`, `ProductPicker.svelte` ; `features/invoices/` ; `features/credit-notes/` ; `routes/(app)/invoices/`, `routes/(app)/credit-notes/` | `kesh-i18n` ×4, manuel utilisateur (FR, seul existant), Vitest, E2E |
| 49-1d | — (aucun code de production) | — | `scripts/recette-montee.sh`, `docs/testing.md`, manuel d'administration (FR, seul existant) (« Passer à la 0.20.0 ») |
| 49-2 | probablement aucun (ordre préservé par `update`) | `components/invoices/InvoiceForm.svelte`, `features/invoices/` | i18n, manuel utilisateur, Vitest, E2E |
| 49-3 | `kesh-api/src/routes/invoices.rs` (garde `heading`) ; `kesh-qrbill/src/pdf.rs` (rendu du titre) | `components/invoices/InvoiceForm.svelte`, `features/invoices/`, `features/credit-notes/` | i18n, `docs/api-external.md`, manuel utilisateur, tests |

Modules de l'épique : `kesh-db` (migration, entités, dépôts, sauvegarde), `kesh-api` (routes des factures et avoirs, PDF,
exports, MCP), `kesh-qrbill` (PDF de la facture et de l'avoir), `kesh-report` (rapport TVA), `kesh-seed`, `kesh-i18n`,
frontend (`components/invoices`, `features/invoices`, `features/credit-notes`, routes `invoices` et `credit-notes`),
`scripts/recette-montee.sh`, manuels FR/DE/IT/EN (`.tex` seulement), `docs/api-external.md`, `docs/testing.md`,
`docs/migrations-idempotence-audit.md`. **`kesh-core` n'en fait plus partie.**

## Fichiers communs (avec l'Epic 48 et les PR de Beat)

| Fichier | Qui d'autre | Règle |
|---|---|---|
| `lib/components/invoices/InvoiceForm.svelte` | 48-13 (#1233) | **la 48-13 se fusionne d'abord** (arbitrage de Guy du 2026-11-18) ; 49-1c, 49-2, 49-3 partent ensuite de la branche à jour |
| `crates/kesh-qrbill/src/pdf.rs`, `types.rs` | PR #1327 de Beat (`largeurs.rs`, `lib.rs`) | le second arrivé résout |
| manuel d'administration (FR, seul existant) | 49-1a (« Pour les intégrations ») et 49-1d (« Passer à la 0.20.0 ») | la 49-1d complète ce que pose la 49-1a |
| catalogues i18n (4 locales), `sprint-status.yaml`, `sitesTotal`, totaux de l'audit des migrations, schéma de test | toutes | blocs par épique ; compteurs recomptés après fusion |
| manuels `.tex` | toutes ; le menu (v0200-menu-sous-groupes) fusionne en dernier | PDF jamais commités |

## Ordre de sortie à la coupure

La date ne bouge pas (kesh:D11). Si le temps manque, dans l'ordre : la **49-3**, puis la **49-2**, puis la **49-1c**
(la 49-1b peut rester seule : la garde tient `comment` fermé), puis la **49-1b** — qui emporte la 49-1c si celle-ci est
restée. La
**49-1a porte la migration et ne sort jamais sans la 49-1d** : si l'une n'est pas close à la coupure, les deux sortent
**avec** la migration (« une story sortie emporte sa migration ») et la 0.20.0 n'en porte aucune — à dire en tête des
notes.

## Décomptes au kickoff

Au 2026-11-18 : 496 issues ouvertes, 247 bugs, 14 P1 au kickoff (507 et 257 le soir, plan de la release). L'Epic 49 porte
**#1242** (`enhancement`, P2). Les sous-totaux en sortent (#1330) : #1242 reste ouverte, avec ce qui manque écrit au corps,
si une story de l'épique sort ; sinon elle se ferme avec la dernière livrée, #1330 portant les sous-totaux.
