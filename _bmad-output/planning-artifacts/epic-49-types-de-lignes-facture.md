# Epic 49 — Types de lignes de la facture client (#1242)

*Kickoff du 2026-11-18, tenu en autonomie par l'orchestrateur (Opus 5.5), worktree `kesh-k0200`, branche
`chore/kickoff-0.20.0`. Code lu à `main` = tag **v0.19.0** (`773df80da`). Release **0.20.0** : publication le **vendredi
2026-11-27**, coupure le **jeudi 2026-11-26 au soir** (plan : `release-0.20.0.md`). Décisions de Guy du 2026-11-18 ;
demande et réponses de Guy du 2026-11-17 au corps de #1242. Révisé le même jour sur les réponses de Guy aux points
ouverts du kickoff : sous-totaux repoussés, ordre de fusion avec la 48-13, règlements de l'orchestrateur ; puis sur le
complément de Guy du même jour (factures réelles déjà émises, § *Contraintes de production*).*

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
  taux, ou attendre. Issue à ouvrir par l'orchestrateur, « sous-totaux de section », milestone « Plus tard » ;
- **réordonnancement** des lignes à la main **avant la validation** (brouillon).

Les **avoirs** reprennent les lignes de commentaire (et les titres) de la facture d'origine. Le **PDF** rend commentaires
et titres (pleine largeur, sans colonnes de quantité ni de prix).

C'est une **épique de fonctionnalité** : elle n'est pas tenue à la borne de six stories, et en compte trois.

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

1. **La migration conserve à l'identique chaque facture et chaque avoir existants.** Toute ligne existante reçoit le type
   « ligne produit » ou « sans produit » selon qu'elle vient du catalogue ou non — **règle à établir à la fiche, jamais
   devinée à tort** : en cas de doute, un **type unique « chiffré »** qui se rend exactement comme aujourd'hui. Lu au
   code : `invoice_lines` et `credit_note_lines` **ne gardent aucun lien** vers le catalogue (pas de colonne d'article ;
   la ligne recopie les valeurs) ; l'origine d'une ligne existante n'est donc vraisemblablement pas établissable, et le
   type « chiffré » le cas probable — la fiche le prouve ou le réfute. Totaux, TVA, écritures et numéros **inchangés**.
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

Portée par la **49-1**, elle pose **d'emblée les quatre types de la release** — produit, sans produit, commentaire,
**titre** — et leurs contraintes, pour que la 49-3 n'ait pas à migrer et puisse sortir à la coupure sans emporter de
migration (décisions de Guy du 2026-11-18). **Le type sous-total n'y entre que si la fiche de la 49-1 établit qu'il ne
préjuge pas de son sens TVA** (HT, TTC, ventilation par taux) ; sinon il viendra avec sa propre migration, dans la
release des sous-totaux.

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
| 49-1-types-de-lignes | Types produit, sans produit, commentaire ; la migration des types (titre compris) ; rendu PDF ; avoirs qui reprennent commentaires et titres | **oui** | — |
| 49-2-reordonnancement-lignes | Réordonner les lignes d'un brouillon : ↑ / ↓, et glisser-déposer si la fiche le juge sûr | non | 49-1 |
| 49-3-titres-de-section | Titre de section, sans montant : saisie, rendu PDF, reprise dans les avoirs | non (posée par 49-1) | 49-1 |

La clé `49-3-titres-et-sous-totaux` du kickoff est renommée `49-3-titres-de-section` le 2026-11-18, les sous-totaux étant
repoussés (décision de Guy).

### 49-1 — Types de lignes

Le choix du type à l'ajout d'une ligne ; la ligne produit recopie l'article sans lien (comportement actuel à confirmer au
code) ; la ligne sans produit prend TVA et compte des réglages ; la ligne de commentaire est un texte seul, hors total et
écritures. Le serveur valide chaque type (requête de création et de mise à jour, API externe documentée dans
`docs/api-external.md`) ; la validation de la facture ignore les lignes non chiffrées ; un avoir recopie commentaires et
titres à leur position. Le PDF rend un commentaire sur toute la largeur. Le type titre, déjà admis par le schéma, reste
**refusé par l'API** jusqu'à la 49-3 (pour qu'une 49-3 sortie à la coupure ne laisse pas un type accepté sans écran ni
rendu) ; de même le type sous-total, s'il entre au schéma.

**Splitting préventif** (règlement de l'orchestrateur du 2026-11-18) : la 49-1 touchera vraisemblablement plus de cinq
modules (`kesh-db`, `kesh-core`, `kesh-api`, `kesh-qrbill`, `kesh-report`, frontend `invoices` et `credit-notes`,
`kesh-i18n`) ; **la règle s'applique à la fiche** : découpage avant `bmad-create-story` — par exemple **49-1a**
(migration, modèle, API, validation, avoirs) et **49-1b** (écran et PDF) — ou section `Dérogation règle de splitting`
si le découpage crée des fusions intermédiaires intestables.

### 49-2 — Réordonnancement

En brouillon seulement : boutons ↑ / ↓ sur chaque ligne (accessibles au clavier) ; glisser-déposer si la fiche le juge sûr
(accessibilité, tests Playwright). L'ordre envoyé au serveur devient la position (`update` réinsère les lignes) ; une
facture validée ne se réordonne pas. Frontend pour l'essentiel ; E2E du parcours « créer toutes les lignes, puis
réordonner ».

### 49-3 — Titres de section

Un titre de section est une ligne de texte **sans montant**, hors du total et des écritures, rendue en évidence sur
toute la largeur du PDF ; il se place, se déplace (49-2) et se reprend dans un avoir comme un commentaire. L'API accepte le
type titre à partir de cette story.

**Sous-totaux : hors de la 0.20.0** (décision de Guy du 2026-11-18). Beat tient une comptabilité assujettie à la TVA : un
sous-total doit être juste en HT et TTC et par taux, ou attendre. Ils feront l'objet d'une release ultérieure — issue à
ouvrir par l'orchestrateur, « sous-totaux de section », milestone « Plus tard » ; le sens proposé par #1242 (total de la
section, hors total et écritures) y sera repris.

## Dépendances entre stories et avec l'Epic 48

- **49-2 et 49-3 partent de la 49-1** (même éditeur de lignes, même type de ligne) ; elles peuvent avancer en parallèle
  l'une de l'autre une fois la 49-1 fusionnée dans la branche d'épique, en résolvant `InvoiceForm.svelte`.
- **Avec la 48-13** : `InvoiceForm.svelte` (#1233, chargement du client). **Arbitrage de Guy du 2026-11-18 : la 48-13 se
  fusionne d'abord** (petite, sur le chemin de sa première facture de fin novembre) ; les stories de l'Epic 49 qui touchent
  ce fichier partent ensuite de la branche à jour.
- **Avec la PR #1327 de Beat** : elle touche `kesh-qrbill/src/largeurs.rs` et `lib.rs` (et les catalogues) ; la 49-1 en
  tient compte si elle est fusionnée avant.

## Fichiers et modules touchés (déclarés au kickoff)

| Story | Backend | Frontend | Docs, i18n, tests |
|---|---|---|---|
| 49-1 | `kesh-db/migrations/` (une migration), `test-schema/0001_schema_squash.sql`, `post_restore.rs` (si backfill), `backup.rs` ; `kesh-db/src/entities/invoice.rs`, `credit_note.rs` ; `kesh-db/src/repositories/invoices.rs`, `credit_notes.rs`, et les lecteurs de `invoice_lines` inventoriés ci-dessus ; `kesh-core/src/accounting/vat.rs` ; `kesh-report/src/vat_report.rs` ; `kesh-api/src/routes/invoices.rs`, `credit_notes.rs`, `invoice_pdf_service.rs`, `errors.rs`, `exports/` ; `kesh-qrbill/src/types.rs`, `pdf.rs` ; `kesh-seed` (données de démonstration, pièges) | `components/invoices/InvoiceForm.svelte`, `ProductPicker.svelte` ; `features/invoices/` (types, helpers) ; `features/credit-notes/` ; `routes/(app)/invoices/`, `routes/(app)/credit-notes/` | `kesh-i18n` (4 locales), `docs/migrations-idempotence-audit.md`, `docs/api-external.md`, manuels utilisateur (4 langues) et d'administration (sauvegarde avant mise à jour), tests d'intégration `kesh-db`/`kesh-api`, Vitest, E2E |
| 49-2 | probablement aucun (à confirmer : ordre préservé par `update`) | `components/invoices/InvoiceForm.svelte`, `features/invoices/` | i18n, manuel utilisateur, Vitest, E2E |
| 49-3 | `kesh-db/src/repositories/invoices.rs`, `credit_notes.rs` (acceptation du type titre) ; `kesh-api/src/routes/invoices.rs`, `invoice_pdf_service.rs` ; `kesh-qrbill/src/pdf.rs`, `types.rs` | `components/invoices/InvoiceForm.svelte`, `features/invoices/` | i18n, `docs/api-external.md`, manuel utilisateur, tests |

Modules de l'épique : `kesh-db` (migration, entités, repositories, sauvegarde), `kesh-core` (TVA), `kesh-api` (routes des
factures et avoirs, PDF, exports), `kesh-qrbill` (PDF de la facture), `kesh-report` (rapport TVA), `kesh-seed`,
`kesh-i18n`, frontend (`components/invoices`, `features/invoices`, `features/credit-notes`, routes `invoices` et
`credit-notes`), manuels FR/DE/IT/EN (`.tex` seulement), `docs/api-external.md`, `docs/migrations-idempotence-audit.md`.

## Fichiers communs (avec l'Epic 48 et les PR de Beat)

| Fichier | Qui d'autre | Règle |
|---|---|---|
| `lib/components/invoices/InvoiceForm.svelte` | 48-13 (#1233) | **la 48-13 se fusionne d'abord** (arbitrage de Guy du 2026-11-18) ; 49-1, 49-2, 49-3 partent ensuite de la branche à jour |
| `crates/kesh-qrbill/src/pdf.rs`, `types.rs` | PR #1327 de Beat (`largeurs.rs`, `lib.rs`) | le second arrivé résout |
| catalogues i18n (4 locales), `sprint-status.yaml`, `sitesTotal`, totaux de l'audit des migrations, schéma de test | toutes | blocs par épique ; compteurs recomptés après fusion |
| manuels `.tex` | toutes ; le menu (v0200-menu-sous-groupes) fusionne en dernier | PDF jamais commités |

## Ordre de sortie à la coupure

La date ne bouge pas (kesh:D11). Si le temps manque : la **49-3** sort la première (la migration, déjà posée par la 49-1,
reste ; l'API continue de refuser le type titre), puis la **49-2**. La **49-1 porte la migration** : si elle n'est pas
close à la coupure, elle sort **avec** sa migration (« une story sortie emporte sa migration ») et la 0.20.0 n'en porte
aucune — à dire en tête des notes.

## Décomptes au kickoff

Au 2026-11-18 : 496 issues ouvertes, 247 bugs, 14 P1 (plan de la release). L'Epic 49 porte **#1242** (`enhancement`,
P2). Les sous-totaux en sortent : #1242 ne se ferme donc pas avec l'épique sans que l'issue des sous-totaux soit ouverte
et liée ; elle reste ouverte, avec ce qui manque écrit au corps, si la 49-2 ou la 49-3 sort.
