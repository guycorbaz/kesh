# Story 15.1a-i : La marque du lettrage — schéma, primitive, routes

## Status

done *(créée le 2026-10-09 au découpage de la 15-1a en validation P3 — C124 ; validation close en P5 ;
développement ouvert le 2026-10-09 sur `dc4bc58b` ; revue de code close après trois passes (P1 Sonnet ×3,
P2 Opus ×3, P3 ciblée Haiku) ; **intégrée le 2026-10-09 sur `origin/main` `803f3e15`** (15-13a, 15-7b1,
15-13b, 15-6c) — gate de référence sur l'état rebasé : `scripts/test-fast.sh` à huit threads 3135/3135,
Vitest 1149/1149, E2E 246 / 7 KF-029 + 1 pollution rejouée verte. ⛔ refs #518, pas closes : la 15-1a-ii
suit, et aucun tag v0.13.0 ne se pose avant son merge — C124)*

## Story

**As a** indépendant, PME ou fiduciaire qui tient ses comptes dans Kesh,
**I want** pouvoir marquer comme se soldant entre elles des lignes d'un même compte,
**so that** ce que le logiciel affirme sur ce qui reste ouvert repose sur une marque unique, posée et
retirée par un seul chemin.

Première moitié du **socle** du lettrage (issue **#518**, P1 ; FR85, FR86), issue du découpage de la
15-1a (C124, couture écrite à C118) : **la marque** — le schéma, le code, la primitive de lettrage unique
et sa règle des périodes, les routes manuelles, l'audit, l'exposition et l'export. La seconde moitié,
**`15-1a-ii-gardes-du-lettrage.md`**, pose les gardes : le gel des écritures lettrées (`ENTRY_LETTERED`)
et le lettrage `reversal` de la contre-passation (R6).

Ordre de développement : **15-12a → 15-12b → 15-1a-i → 15-1a-ii → 15-1a2 → 15-1b → 15-1c** (la
**15-12a**, clôture dans l'ordre, est le **prérequis** réel ; la 15-12b passe avant de préférence — Dev
Notes, C112).

⚠️ **Numérotation conservée** (C124) : les décisions R1–R7, les critères AC1–AC15 et les tâches T0–T12
gardent le numéro qu'ils avaient dans la 15-1a ; cette fiche porte R1–R5 et R7, AC1–AC7, AC10–AC14 et la
part (i) d'AC15 ; R6, AC8, AC9 et la part (ii) d'AC15 sont dans la 15-1a-ii. Table de correspondance :
`15-1a-socle-lettrage.md` (index).

⛔ **Dépendance résiduelle, et la règle de publication qui la ferme** : livrée seule, cette story laisse
une écriture **manuelle** lettrée par l'API modifiable et supprimable (son partenaire resterait « soldé »
seul) — le gel est l'AC8 de la 15-1a-ii. Aucun écran ne lettre encore (15-1c), mais l'API est ouverte aux
clés. Donc : la **15-1a-ii suit immédiatement**, et **la v0.13.0 ne se tague pas entre les deux merges**
(C124).

## Reprise du 2026-10-08 — pourquoi la 15-1a a été réécrite

*(Section reprise de la 15-1a, commune à ses deux moitiés.)* La fiche précédente (huit passes de `validate`, 2026-08-25) supposait **une facture payée = une
ligne au compte débiteurs**, et une marque **par paire**. Depuis :

| fait nouveau (vérifié sur `origin/main` `9cb5083b`) | conséquence pour le socle |
|---|---|
| **Story 24-2** — `invoice_settlements` : un règlement client produit son écriture (un **crédit** à la créance), une facture se règle **en plusieurs fois** (`invoice_settlements_write.rs:48`, `invoice_settlements.rs:198-240`) | une facture soldée porte **1 + N** lignes au compte débiteurs : la **paire** ne suffit plus |
| **Story 25-4-d2a** — le solde du reste (`write_off`) est une ligne de `invoice_settlements` avec **un** crédit à la créance | une ligne de plus dans le groupe, de même nature |
| **Stories 15-8a/15-8b** — modifier/supprimer une écriture tant que l'exercice est ouvert ; `update_in_tx` fait **DELETE + INSERT** des lignes (`journal_entries.rs:1459/1480`) ; garde `modification_guard` (`:1009`) qui réutilise `reversal_blockers` (`:1922`) | les écritures d'une **pièce** sont déjà gelées ; seules les écritures **sans pièce** peuvent perdre une marque — un motif `Lettered` suffit, évalué **hors** du drapeau `enforce_ownership` (AC8, 15-1a-ii) |
| **Garde-fou d'inventaire** `every_reference_to_an_entry_is_sorted` (`crates/kesh-db/tests/journal_entries_modification.rs:189`) : **aucune** table ne doit référencer `journal_entry_lines` (point 1 bis), et les colonnes des lignes sont **closes** (`COLONNES_DES_LIGNES`, `:78`) | une table `letterings` **avec FK depuis les lignes** reste possible, mais une table qui viserait les lignes est interdite ; la colonne neuve doit y être déclarée |
| **Import d'installation** (`kesh-api/src/admin_backup/import.rs:118-136`) : l'inventaire des tables d'une sauvegarde doit être **identique** dans les deux sens ; une colonne **nullable** neuve, en revanche, passe `check_schema_compat` (`:195-238`) | ⛔ **une table neuve rendrait inimportable TOUTE sauvegarde antérieure** — c'est précisément pourquoi la 25-4-d2a a choisi une colonne (`20261001000004_invoice_settlements_write_off.sql:13-15`). La table `letterings` arbitrée le 2026-08-25 **coûte désormais l'export de sauvegarde (#386) que la v0.12.1 vient de livrer** |
| **Stories 15-5e1/15-5e2** — rejeu sur interblocage, registre à deux colonnes (`crates/kesh-api/tests/audit_route_registry.rs`) | les routes neuves s'y inscrivent, enveloppées |
| **Contre-passation** écrite **une seule fois** (`reverse_in_tx_inner`, `journal_entries.rs:2175`), par où passent toutes les annulations (règlement client et fournisseur, facture fournisseur, dé-rapprochement) | un seul site pour la règle « contre-passation ↔ lettrage » (R6, 15-1a-ii) |

Les choix de la reprise sont consignés au registre `epic-15-choix-autonomes.md` (entrées
**C90 à C99**), révisés en validation P1 par **C101 à C106**, en validation P2 par **C113 à C118**, en validation P3 par **C124 à C127** (dont le découpage, C124). Ce qui suit les met en œuvre ; les décisions de la fiche d'août qui tiennent
toujours sont reprises **en le disant**.

## Les quatre questions du dégel (2026-08-28), tranchées

| question | réponse | registre |
|---|---|---|
| **Lettrage partiel ?** | **Non.** Un lettrage est un **groupe** de ≥ 2 lignes d'un même compte dont la somme `Σ(débit − crédit)` est **exactement nulle**. Un règlement partiel laisse ses lignes **ouvertes**, et la vue (15-1b) les regroupe par pièce avec leur reste. Pas d'état « partiellement lettré » | C92 |
| **Proposé ou manuel ?** | **Trois origines.** `document` : posé par Kesh quand une **pièce** est soldée (15-1a2) — le rattachement a été **décidé par l'utilisateur** en saisissant le règlement, Kesh ne devine rien. `reversal` : posé par Kesh entre une ligne et sa contre-passation (`reverses_entry_id`, lien déclaré). `manual` : choisi par l'utilisateur ; Kesh **propose** des paires (15-1b), **n'écrit jamais** sans validation — règle du `CLAUDE.md` « un appariement automatique propose, il ne crée jamais » | C93 |
| **Lettrage à cheval sur deux exercices ?** | **Permis à cheval.** **Lettrer comme délettrer exige qu'au moins UNE ligne du groupe soit « en période ouverte »** — exercice ouvert, aucun exercice postérieur clôturé, date postérieure au verrou de période (R7) : un groupe entièrement en période close ne se crée ni ne se défait (révision de D3 d'août et de C94 en validation P1, étendue au verrou de période et à l'état hérité en validation P2 — la vue « au » ignore la date du lettrage, si bien que lettrer sur une période close réécrirait ses postes ouverts autant que délettrer). Repose sur « les clos forment un préfixe » : la **15-12a est un prérequis** | C94, C105, C113 |
| **Depuis quel écran ?** | Un **écran dédié « Postes ouverts »** (15-1c), atteint depuis le menu et depuis le **Grand livre** d'un compte lettrable ; le code de lettrage est **visible sur la fiche d'écriture**. Pas d'écran « compte » à créer (il n'en existe pas : `/accounts` est le plan comptable) | C95 |

## Décisions de la reprise

### R1 — La marque est portée par la LIGNE, en deux colonnes, sans table (C90)

```sql
ALTER TABLE journal_entry_lines
    ADD COLUMN lettering_key BIGINT NULL
        COMMENT 'Clé du groupe de lettrage = plus petit id de ligne du groupe ; NULL = ligne ouverte',
    ADD COLUMN lettering_origin VARCHAR(10) NULL
        COMMENT 'Origine du groupe : document, reversal, manual',
    ADD CONSTRAINT chk_jel_lettering_origin
        CHECK (lettering_origin IS NULL OR lettering_origin IN ('document', 'reversal', 'manual')),
    ADD CONSTRAINT chk_jel_lettering_pair
        CHECK ((lettering_key IS NULL) = (lettering_origin IS NULL));

CREATE INDEX idx_jel_account_lettering ON journal_entry_lines (account_id, lettering_key);
CREATE INDEX idx_jel_lettering ON journal_entry_lines (lettering_key);

UPDATE _kesh_version SET kesh_version_min_required = '0.13.0' WHERE id = 1;
```

*(Forme reprise du seul précédent d'`ALTER TABLE journal_entry_lines` du dépôt,
`20260702000001_projects_analytics.sql:38-42` : `ADD COLUMN` + `ADD CONSTRAINT`, puis
`CREATE INDEX` séparé.)*

⛔ **Pourquoi pas la table `letterings` arbitrée le 2026-08-25** : elle rendrait **toutes** les
sauvegardes `.keshbackup` antérieures inimportables (inventaire strict, `kesh-api/src/admin_backup/import.rs:118-136`),
alors qu'une colonne nullable passe `check_schema_compat`. Ce que la table apportait se
retrouve autrement :

| la table apportait | sans table |
|---|---|
| un compteur `seq` par société, sérialisé | **inutile** : la clé du groupe est le **plus petit `id` de ligne** du groupe — unique par construction (une ligne n'appartient qu'à un groupe), sans compteur, sans verrou de compteur, sans `MAX()` |
| `created_by`, `created_at` | le **journal d'audit** (`lettering.created`, AC10), qui les porte déjà pour toute mutation |
| un filet `UNIQUE (company_id, code)` | sans objet : deux groupes ne peuvent avoir le même plus petit `id` |

⛔ **Aucune clé étrangère sur `lettering_key`**, et c'est voulu : le point (1 bis) du garde-fou
d'inventaire interdit toute référence vers `journal_entry_lines` (une FK auto-référente y
tomberait), parce que la modification d'écriture **réinsère** ses lignes. L'intégrité est tenue
par la primitive unique (AC3) et par le gel des lignes lettrées (AC8, 15-1a-ii) ; un test d'invariant la
contrôle (AC13).

**Migration BREAKING au sens de P1 — relève `kesh_version_min_required` à `'0.13.0'` en
dernière instruction (P2)** *(révision de C90 en validation P1 — C101)*. Aucune opération de la
liste P3 n'y figure ; c'est la **définition** P1 qui s'applique (« un binaire antérieur ne peut
plus consommer correctement »), comme pour le précédent
`20260814000001_contacts_client_number_canonical.sql:30-36` (« surtout changement de SÉMANTIQUE
d'écriture »). Le risque, écrit au plus juste *(réécrit en validation P2 — F-1, C115 : la version
précédente disait la v0.12.1 « seul binaire antérieur publié » ; c'est faux)* :

- **Tout binaire publié qui passait le contrôle de version** — v0.10.0, v0.11.0, v0.11.1, v0.12.0,
  v0.12.1 : tous portent `min_required = '0.10.0'` au plus et passeraient le contrôle sans ce bump
  (les v0.1.0 à v0.9.0, publiées aussi, sont déjà refusées par `min_required = '0.10.0'` — L1 de P3) —
  **ignore le lettrage** :
  - **v0.10.0 à v0.11.1** **modifient et suppriment des écritures manuelles** (`PUT`/`DELETE
    /journal-entries/{id}` routés, `v0.11.1:crates/kesh-api/src/lib.rs:337-338`, sans
    `ENTRY_IS_POSTED`) : une écriture lettrée y serait réécrite ou supprimée, et son partenaire
    resterait « soldé » seul ; **et annulent** aussi (au moins la facture fournisseur,
    `v0.10.0:crates/kesh-api/src/lib.rs:405-406`, `POST /supplier-invoices/{id}/cancel`) ;
  - **v0.12.0 et v0.12.1** refusent la modification (`409 ENTRY_IS_POSTED`,
    `v0.12.1:crates/kesh-api/src/routes/journal_entries.rs:585`, code à `errors.rs:3044`) mais **annulent un règlement
    client, un paiement fournisseur ou un rapprochement** **sans dissoudre le groupe** posé sur la
    pièce (15-1a2) : la facture redevient due et le grand livre la dit encore **soldée** — le faux
    rattachement muet que le `CLAUDE.md` désigne comme le pire mode d'échec.
- ⚠️ **Ce que le bump n'est pas — la seule barrière** : `sqlx` refuse déjà de démarrer un binaire
  qui ne connaît pas une migration appliquée (`Migrator::run`, `ignore_missing = false` par défaut :
  `MigrateError::VersionMissing`, `kesh-api/src/main.rs:138`), et l'import de la v0.12.1 refuse une
  colonne inconnue (`check_schema_compat`, condition (c1), `unknownColumns`). Le bump rend ce refus
  **explicite, précoce et indépendant** de ces deux mécanismes : `check_downgrade_protection`
  (`main.rs:103`) parle avant `sqlx` en nommant les versions, et le manifeste d'une sauvegarde porte
  `min_required`. C'est l'objet de P1/P2, qui ne présument pas de `sqlx`.

L'état que l'ancien binaire ne sait pas respecter naît **ici** (les colonnes et leur sémantique, et les
groupes `manual` des routes) ; la 15-1a-ii y ajoute les groupes `reversal` de la contre-passation (R6),
la 15-1a2 les groupes `document`, dans la **même release**. D'où le bump dans **cette** migration, à la version **cible** de la
release (`CHANGELOG.md:11 ## [0.13.0] — Non publié`), selon le patron de `20260714000002` (« version
cible de la PR / release »).

**L'en-tête de la migration** — à écrire **une fois**, P8 l'interdisant ensuite (texte arrêté ici
pour que le développeur n'ait pas à le reformuler ; **réécrit en validation P3 — L1** : la version de P2
se lisait comme si v0.10.0–v0.12.1 étaient les seuls binaires publiés, et laissait croire que les
v0.10.0–v0.11.1 n'annulaient rien) :

```sql
-- BREAKING (P1) : relève kesh_version_min_required à 0.13.0 (dernière instruction).
-- Tout binaire publié qui passait le contrôle de version (v0.10.0 à v0.12.1) ignore le
-- lettrage : tous peuvent annuler une pièce dont l'écriture est lettrée sans dissoudre son
-- groupe, et les v0.10.0 à v0.11.1 jusqu'à modifier et supprimer des écritures manuelles.
-- Le bump les refuse au démarrage et à l'import d'une sauvegarde, en le nommant (les
-- v0.1.0 à v0.9.0 le sont déjà par min_required = 0.10.0).
```

**Les deux moitiés de la même action de version (P2-bis)** : dans le **même commit** que la
migration, les **dix** `crates/*/Cargo.toml` passent de `0.12.1` à `0.13.0` (et `Cargo.lock` est
régénéré par la compilation) ; les autres sites qui citent `0.12.1` (`grep -rn "0\.12\.1" crates`
— au 2026-10-09, **six** fichiers outre les `Cargo.toml` : `kesh-api/src/errors.rs`, `kesh-db/src/errors.rs`,
`kesh-report/src/aged_receivables.rs`, `kesh-db/src/repositories/invoice_settlements_write.rs`,
`kesh-db/tests/invoice_amount_due_parity.rs`, `kesh-db/tests/invoice_settlement.rs` — à recompter au
développement) sont **triés** à la main : ce sont des mentions historiques (« depuis la v0.12.1 »), à **ne pas** réécrire. Le **gate
runtime complet** est obligatoire avant `done` — `admin_backup_e2e`, `admin_full_import_e2e`,
`migrations_fresh_install` en font partie (P2-bis : le défaut est invisible en revue statique).

⚠️ **Ce que le bump anticipé fait à la release** — `scripts/prepare-release.sh` (lu, **non
exécuté**) relit la version de `crates/kesh-api/Cargo.toml` et **refuse** si elle égale la cible
(« version cible identique à la version actuelle. Rien à bumper. », `exit 1`) — **avant** son
pré-vol. `scripts/prepare-release.sh 0.13.0` refusera donc, et avec lui **ne s'exécuteront pas** :
la datation de `## [0.13.0] — Non publié` dans le CHANGELOG, ni surtout le **contrôle des
exemptions périssables** (`check_perishable_exemptions`), seul filet de ces justifications. La
release v0.13.0 se fera comme la v0.10.0 (commit `a80a36c1`, crates déjà bumpés par la 22-1) :
CHANGELOG daté à la main, **et** contrôle périssable lancé à la main
(`cargo run -q -p kesh-db --example perishable_exemptions`, puis, pour chaque ligne rendue, la
recherche d'un tag publié dans l'intervalle telle que le script la fait). ⛔ À **signaler à
l'orchestrateur** : soit le script apprend le cas « déjà bumpé » (sauter l'étape 1, garder le
pré-vol), soit la procédure manuelle est écrite dans la checklist de release — décision hors de
cette story (C101).

**Triage P7** : la dernière instruction est un `UPDATE` → le détecteur la voit
(`post_restore.rs`, classement sur le premier mot-clé). Entrée **`EXEMPT_MIGRATIONS`**,
`ExemptionBasis::Durable`, sur le patron exact des deux bumps précédents
(`post_restore.rs:512-525`) : « UPDATE sur _kesh_version (bump kesh_version_min_required), table
système jamais restaurée. Le reste de la migration est du DDL pur. » ⚠️ Pas de commentaire
`/* */` (`migrations_contain_no_block_comment`). ⚠️ En tête de migration, le commentaire
`-- BREAKING (P1) :` est le texte arrêté plus haut, tel quel (patron `20260814000001:30-36`).

**Restauration d'une sauvegarde antérieure** : les deux colonnes reviennent à `NULL`
(`check_schema_compat`, `kesh-api/src/admin_backup/import.rs:195-238`, ne retient que les colonnes obligatoires) — aucun
groupe n'existait à l'époque ; les paires de contre-passation et les pièces soldées historiques
restent non lettrées jusqu'au rattrapage de la 15-1a2 (C98), qui est rejoué après restauration.

### R2 — Le code affiché est la clé, écrite en lettres (C91)

`code = base26_bijective(lettering_key)` : `1 → A`, `26 → Z`, `27 → AA`, `52 → AZ`, `53 → BA`,
`702 → ZZ`, `703 → AAA`. Fonction **pure** en `kesh-core`, avec son **inverse** (`AA → 27`,
pour une recherche par code ; entrée mise en majuscules par **`to_ascii_uppercase`**, **puis**
validée octet par octet sur `A-Z` — refus de tout autre caractère). ⛔ **Jamais `str::to_uppercase`**
avant la validation *(validation P4 — F4-2)* : la casse Unicode ferait de `ß` un `SS`, du s long `ſ` un
`S` et du i sans point `ı` un `I`, si bien que `GET /letterings/ß` résoudrait le groupe `SS` au lieu de
rendre 404. `to_ascii_uppercase` laisse ces caractères tels quels, et la validation les refuse.

⚠️ **Ce que ce choix coûte, écrit d'avance** : les codes ne sont **pas** `A, B, C…` par société
(la fiche d'août le voulait, pour la dictée) ; ils dérivent d'un identifiant de ligne
**global** — 4 lettres jusqu'à 475 254 lignes, 5 au-delà de 475 254 et jusqu'à ~12 millions.
Ils restent courts, stables, dictables. ⚠️ **Réémission** : un groupe dissous puis reformé avec
la même plus petite ligne reprend le **même** code. L'audit (AC10) distingue les deux par leur
horodatage ; un code n'est jamais porté **en même temps** par deux groupes.

### R3 — Le groupe : définition et primitive unique (C92)

Un **groupe de lettrage** est un ensemble de lignes tel que :

1. au moins **deux** lignes ;
2. toutes de la **même société** (par jointure `journal_entries.company_id`) ;
3. toutes sur le **même compte**, et ce compte est **lettrable** (R4) ;
4. aucune n'est déjà lettrée ;
5. `Σ(débit − crédit) = 0` **exactement** (`DECIMAL(19,4)` est exact — la fiche d'août l'avait
   vérifié, P2-6 réfuté) ;
6. toutes portent `lettering_key = MIN(id)` du groupe et la **même** `lettering_origin`.

⛔ **UNE seule fonction écrit la marque** : `letterings::create_group_in_tx(tx, company_id,
line_ids, origin, mode, actor)` — et **une seule** la retire : `letterings::dissolve_group_in_tx(tx,
company_id, key, mode, actor)`. `actor` est `(user_id: i64, actor_api_key_id: Option<i64>)`,
audité par `NewAuditLogEntry::for_actor` (`entities/audit_log.rs:194`) — les routes neuves sont
ouvertes aux clés d'API. ⚠️ La contre-passation ne porte que `user_id` (`reverse_in_tx`,
`journal_entries.rs:2094`) : le lettrage `reversal` passe `None` pour la clé, écart nommé.
`mode` est `Manual` (les routes) ou `System { held_open_fiscal_year_id: i64 }` (contre-passation,
pièces de la 15-1a2) — cf. R7.

**La lettrabilité (R4) est exigée à la CRÉATION d'un groupe, jamais à sa dissolution**
*(C104)*. Un compte lettrable peut cesser de l'être : retypage d'un compte mouvementé
(`kesh-db/src/repositories/accounts.rs:448-558`, `confirm_retype`, Story 25-2-a) ou rattachement ultérieur d'un
`bank_accounts` (`kesh-db/src/repositories/bank_accounts.rs:237`). Les groupes **existants** restent alors intacts — le
solde qu'ils constatent reste vrai — et **dissolubles**. L'invariant d'AC13 ne porte **pas** sur
la lettrabilité, par décision. Ce que la 15-1b en montre (compte devenu non lettrable) est sa
question. Les trois origines (manuel, contre-passation, pièce) passent par
elles.

**Le partage pur / base, dans l'ordre des refus d'AC3** *(précisé en validation P2 — R2-3)* : les
conditions ci-dessus ne sont pas numérotées comme les **rangs** d'AC3, et AC3 intercale des contrôles
en base entre des contrôles purs. `kesh-core::lettering` expose donc **une fonction pure par
cause**, que la primitive appelle **à son rang** : `check_line_ids` (rang 1 : ≥ 2 identifiants,
sans doublon — condition 1), `check_same_account` (rang 3 — condition 3), `check_none_lettered`
(rang 6 — condition 4), `check_balanced` (rang 7 — condition 5). La société (rang 2, condition 2),
la lettrabilité (rang 4), la règle des périodes (rang 4 bis) et l'appartenance à une pièce (rang 5)
se vérifient en base. ⛔ Pas de `validate_group` unique : appelé d'un bloc, il rendrait les rangs 6
et 7 avant 4, 4 bis et 5. Chaque fonction pure est testée sans base.

⛔ **Interdit** : écrire `lettering_key` ou `lettering_origin` — par `UPDATE`, ou par un `INSERT`
qui les nommerait (`INSERT … VALUES`, `INSERT … SELECT` qui recopierait une marque) — ailleurs que
dans ces deux fonctions. **Exceptions, nommées** *(validation P2 — F-9)* : (i) la migration de
rattrapage de la 15-1a2 (SQL de migration) ; (ii) la **restauration d'une sauvegarde**
(`kesh-db/src/backup.rs`, colonnes lues dynamiquement) : elle rétablit les marques **telles
qu'exportées**, jamais recalculées — un groupe restauré est celui qui existait ; (iii) les
suppressions en bloc qui retirent les lignes **avec** leurs marques — `delete_all_by_company`
(appelants de test seulement) et `kesh-seed` (`reset_demo`, `DELETE FROM journal_entry_lines`,
`kesh-seed/src/lib.rs:257`) : un groupe disparaît entier, jamais à moitié. Test lexical T9.

### R4 — Les comptes lettrables (C96)

Un compte est **lettrable** s'il est de type `Asset` ou `Liability` **et** qu'aucun
`bank_accounts.journal_account_id` ne le désigne — **archivé ou non** *(validation P3 — F3-7, C127 :
`bank_accounts` porte `archived` ; un compte qui a été celui d'un compte bancaire relève de la
réconciliation, et ne redevient pas lettrable à l'archivage ; la 15-1b emploie la même fonction,
`is_letterable_account`)*. Couvre : comptes débiteurs et fournisseurs
(quels que soient les réglages — le compte de créance de la vente peut être **tout** compte
`Asset`, `routes/company_invoice_settings.rs:302-310`), comptes de passage, acomptes, avances,
comptes courants d'associés. Exclut : charges et produits (le lettrage n'y a pas de sens), et
les comptes bancaires — **gérés par la réconciliation** (`bank_transactions.status`), frontière
que l'écran de la 15-1c énonce (D4/D6 d'août).

⚠️ **Pas de borne par rôle** (`singleton_role`) : la fiche d'août bornait la vue aux rôles
`Receivable`/`Payable` ; #518 vise aussi les **comptes de passage** et les **écritures
manuelles** (acomptes, avances, compensations), qui n'ont pas de rôle. ⚠️ Un compte **archivé**
reste lettrable : ses lignes existent et se soldent.

### R5 — Les lignes d'une pièce ne se lettrent pas à la main (C93)

Une ligne dont l'**écriture appartient à une pièce** — motifs `OwnedByInvoice`,
`OwnedByCreditNote`, `OwnedBySupplierInvoice`, `OwnedBySettlement` de `reversal_blockers`
(`journal_entries.rs:1922`, rangs 3 à 6) — est **refusée** au lettrage manuel. Son lettrage est
celui de sa pièce (origine `document`, 15-1a2) : **une facture est soldée si et seulement si
son reste dû est nul**, et le grand livre ne doit pas dire autre chose que la fiche facture, la
balance âgée, les relances et le rapprochement — qui lisent tous le reste dû ou `paid_at`.

⛔ **Réutiliser `reversal_blockers`**, pas une seconde liste : deux inventaires des propriétaires
divergeraient (même motif que C-15-8-5).

⚠️ **Ce qui reste lettrable à la main** : lignes d'écritures manuelles, de soldes de départ, de
rapprochement bancaire hors facture (`accept_one_split`, `accept_one_rule`, `post_manual`,
`post_split` — motif `MatchedBankTransaction`, qui n'est pas une pièce), contre-passations dont
l'origine était déjà lettrée (R6), et le **règlement fournisseur détaché** d'une facture annulée
(`kesh-db/src/repositories/supplier_invoices.rs:1046-1050` au 2026-10-09 — `:992-996` jusqu'en P3 —, plus rattaché à aucune pièce).

⚠️ **Ce que la règle coûte** : une facture client payée par une écriture **manuelle** (au lieu
d'un règlement enregistré sur la facture) ne se lettre pas avec ce paiement. Le geste juste est
d'enregistrer le règlement sur la facture (type « compte interne » au besoin) ; le manuel le dit
(15-1c).

⚠️ **Et un cas hérité qu'elle laisse ouvert** *(C106)* : le règlement d'une facture **créditée**
(cas antérieur à la v0.12.1 ou restauré d'une sauvegarde) garde sa ligne `invoice_settlements` —
motif `OwnedBySettlement`, donc refusé au lettrage manuel — et le groupe `document` ne peut pas le
prendre (facture créditée **et** réglée : somme ≠ 0 au compte débiteurs). Le manuel
(`user-manual.tex:1171`, `:1711`) et `api-external.md` (`:325`, `:386`, refus `INVOICE_CREDITED`)
le disent aujourd'hui « paiement **à lettrer** » : promesse rendue fausse par R5. ⚠️ **Et l'écran le
dit aussi** *(validation P4 — R4-1, C130 : C106 n'avait inventorié que le manuel et l'API)* : le
motif du refus d'annulation — deux clés i18n dans les quatre locales, leurs replis Rust et
frontend, quatre tests frontend qui assertent le texte, un doc-comment —, inventaire complet à AC15
part i. **Cette story** les réécrit tous, dans le commit qui pose R5 : l'écran dirait sinon à
l'utilisateur de lettrer ce que l'API lui refuse. Texte de remplacement : « … ce règlement **reste
ouvert au compte débiteurs**, il ne s'annule pas » — un constat, sans geste promis. **Le traitement du
cas est renvoyé à la 15-1a2** (section « Reçu de la 15-1a ») ; si elle le traite par une exception à R5
ou un groupe `document`, c'est elle qui réécrira ce motif.


### R6 — La contre-passation ne défait jamais un groupe ; elle lettre ce qui est libre (C97) — **dans la 15-1a-ii**

Décision portée par la 15-1a-ii (lettrage `reversal` en fin de `reverse_in_tx_inner`, relecture de la
réponse `201`). Ce qu'elle attend d'ici : la primitive en mode `System { held_open_fiscal_year_id }`
(R3, R7 point 3), l'origine `reversal` admise par la contrainte `chk_jel_lettering_origin` (R1), et la
cause 5 d'AC3 non appliquée en mode `System`.

### R7 — Les verrous, et leur ordre *(réécrite en validation P1 — C103 ; points 2, 3 et la règle des périodes révisés en validation P2 — C113, C114 ; point 2 réécrit en validation P3 — C125)*

`create_group_in_tx` et `dissolve_group_in_tx` suivent la même séquence :

1. **Premier acte : une lecture VERROUILLANTE qui découvre et verrouille à la fois.** Aucune
   lecture ordinaire avant elle — doctrine du dépôt, « le verrou d'abord » (`update_in_tx`,
   `journal_entries.rs:1301-1309` ; `delete_in_tx`, `:1655-1657`) : une lecture ordinaire ouvrirait
   la vue `REPEATABLE READ` sur un état que le verrou ne garantit plus.
   - création : `SELECT jel.id, jel.entry_id, jel.account_id, jel.debit, jel.credit,
     jel.lettering_key, jel.lettering_origin, je.fiscal_year_id, je.entry_date FROM
     journal_entry_lines jel JOIN journal_entries je ON je.id = jel.entry_id WHERE jel.id IN (…)
     AND je.company_id = ? ORDER BY jel.id FOR UPDATE` (`je.entry_date` sert la règle des périodes,
     ci-dessous) ;
   - dissolution : la même, `WHERE jel.lettering_key = ? AND je.company_id = ?` (parcours de
     `idx_jel_lettering` : le verrou de clé suivante empêche aussi une **reformation** concurrente
     sous la même clé).

   ⚠️ **Verrous d'intervalle de `idx_jel_lettering`, des deux côtés** *(validation P4 — F4-1 ;
   raisonné sur le moteur, non exécuté, à constater en T0)* : en `REPEATABLE READ`, la lecture
   `FOR UPDATE` par égalité sur cet index **non unique** (dissolution) verrouille chaque entrée de la
   clé, le trou qui la précède **et le trou qui suit la dernière** (jusqu'à la clé suivante, ou le
   supremum). Côté **création**, l'`UPDATE … SET lettering_key = k` **insère** l'entrée `(k, id)` dans
   ce même index : il attend si `k` tombe dans un trou tenu. Les deux gestes se gênent donc sur
   l'index, et non seulement sur les lignes — cf. « Interblocages résiduels », troisième puce.

   ✅ **Constat T0 (2026-10-09, MariaDB 10.11.16 — C-15-1a-i-1) : la moitié « création » est DÉMENTIE.**
   Pendant une dissolution tenue ouverte, l'`UPDATE … SET lettering_key = k` d'une création **n'attend
   pas**, que `k` tombe après la plus grande clé ou dans le trou suivant la clé dissoute ; un `INSERT`
   direct d'une ligne portant `k`, lui, attend (1205) — le trou est bien tenu, mais la mise à jour d'un
   index secondaire ne s'y heurte pas. L'autre moitié (F3-5, insertion d'une ligne **ouverte** pendant la
   dissolution de la plus petite clé) est **confirmée** : 1205 au bout de 3 s, sur une écriture sans
   rapport.

   ⚠️ Plan à vérifier par `EXPLAIN` au développement : le parcours doit partir de la clé primaire
   des lignes (création) ou d'`idx_jel_lettering` (dissolution), faute de quoi le verrou d'une
   lecture jointe couvrirait les en-têtes de toute la société.

   Une lecture `FOR UPDATE` jointe pose un verrou exclusif sur les lignes lues **des deux tables** :
   lignes **et** en-têtes sont tenus à l'issue de ce seul acte, sans seconde lecture à comparer —
   ce qui a été lu sous verrou ne change plus avant la validation. Moins de lignes que demandées
   (création) ou aucune (dissolution) → **404** (AC11), avant toute autre cause. ⚠️ Cela rétablit
   l'invariant « on ne touche une ligne qu'en tenant son en-tête » (conduite (b) de P8-2 d'août).

   ⚠️ **Limite assumée** *(validation P2 — F-11)* : le filtre `je.company_id` s'applique **après**
   le parcours ; en `REPEATABLE READ`, InnoDB garde le verrou des lignes lues puis écartées. Un
   identifiant d'une autre société est donc verrouillé (ligne et en-tête) le temps de la
   transaction — et une requête qui le vise **attend** si l'autre société le tient, ce qui la
   distingue, par sa durée, d'un identifiant inexistant ; un identifiant au-delà du maximum pose un
   verrou de clé suivante sur le supremum (toute insertion de ligne attend). Le **contenu** du 404
   reste indiscernable (AC11) ; la **durée** ne l'est pas. Bref, local à une installation
   mono-société en pratique : écrit, non corrigé. ⚠️ Ce n'est pas le seul verrou qui déborde la
   société : la dissolution du groupe de **plus petite clé** verrouille le trou de
   `idx_jel_lettering` où s'insèrent **toutes** les lignes neuves, toutes sociétés confondues
   (F3-5 de P3, « Interblocages résiduels » ci-dessous) — même nature (durée, pas contenu), même
   décision.
2. **Les exercices — en mode `Manual` seulement — un par un, par clé primaire, dans l'ordre
   chronologique lu sans verrou ; les seuls exercices du groupe** *(révisé en validation P2 — C114 ; puis
   en validation P3 — R3-2, C125, sur la forme de la 15-12a, C119)*. ⛔ **Un `ORDER BY` ne fixe pas
   l'ordre d'acquisition des verrous** : InnoDB verrouille au fil du **parcours** choisi par
   l'optimiseur, puis trie. C'est vrai d'une liste d'`id` (`WHERE id IN (…) ORDER BY start_date, id FOR
   UPDATE`, forme de P1 : elle verrouillait dans l'ordre des `id`) comme d'un parcours d'intervalle
   (`start_date >= ? ORDER BY start_date ASC FOR UPDATE`, forme de P2 : l'ordre y dépend du plan, que le
   dépôt a vu passer par un `filesort` qui verrouille tous les exercices de la société,
   `opening_complement.rs:278-281`). L'ordre est donc tenu par le **code** :
   - (a) **lecture non verrouillante** des exercices du groupe : `SELECT id, start_date, name FROM
     fiscal_years WHERE id IN (<fiscal_year_id distincts des lignes, lus à l'acte 1>) AND company_id = ?
     ORDER BY start_date ASC`. `start_date` est **immuable** (seuls `close`, `reopen` et `update_name`
     écrivent `fiscal_years`, et `update_name` ne touche que `name` — `fiscal_years.rs:400`) : la lire
     sans verrou est sûr, et c'est l'étape (a) de la clôture de la 15-12a (AC 3). `name` sert
     l'affichage (AC6, AC10 — C127) ;
   - (b) **verrou de chacun, un par un, dans cet ordre**, par une boucle Rust — constante
     `LOCK_LETTERING_FISCAL_YEAR_SQL` : `SELECT id, start_date, status FROM fiscal_years WHERE id = ? AND
     company_id = ? FOR UPDATE` ; le `status` est lu **sous le verrou** (état validé le plus récent),
     c'est lui que juge le (i) de la règle des périodes. Absent → `DbError::Invariant` (*fail-loud*) :
     l'exercice d'une écriture tenue par l'acte 1 ne peut disparaître (clé étrangère
     `fk_journal_entries_fiscal_year`, sans cascade). Un verrou par clé primaire ne pose ni verrou
     d'intervalle ni verrou de fin d'intervalle : la société suivante n'est jamais touchée (L10 de P3) ;
   - (c) **« un exercice postérieur est clos »** — le (ii) de la règle des périodes — se lit **sans
     verrou**, par la variante existante `fiscal_years::find_later_closed` (`fiscal_years.rs:678`, texte
     partagé `FIND_LATER_CLOSED_SQL`, `:668-671`), sur `&mut **tx`, pour chaque exercice **ouvert** du
     groupe (un exercice clos rend ses lignes closes sans autre lecture). **Pourquoi c'est sûr** — la
     preuve de la 15-12b pour sa création d'écriture (AC 8, (α) et (β)) : **(α)** sous la 15-12a, aucune
     transition ne **crée** l'état « X ouvert, postérieur clos » — la clôture d'un postérieur Y verrouille
     ses antérieurs un par un (15-12a AC 3 (b')) puis relit sous verrou (AC 3 (d)) et refuse tant qu'un
     antérieur est ouvert ; **(β)** le lettrage tient X par (b) jusqu'à son `COMMIT` : la clôture de Y
     attend X, puis le trouve ouvert et refuse. Une lecture périmée ne peut donc **manquer** aucune
     clôture postérieure ; elle ne peut que voir **encore** clos un postérieur rouvert entre-temps (état
     hérité) — refus à tort, sans dommage, que l'utilisateur rejoue. Un exercice **créé** pendant ce temps
     naît `Open` : sans effet sur le (ii). Aucun fantôme ne peut changer le verdict : **pas de relecture
     verrouillante** (C119 la réserve à ce cas).

   **Pourquoi les seuls exercices du groupe** *(R3-2 de P3 ; C114 verrouillait « tous les
   postérieurs »)* : les verrous au-delà du plus récent du groupe ne gardaient aucune transition (point
   (c)), et coûtaient une sérialisation avec toute écriture de l'exercice du jour (`create_in_tx_inner`
   en prend la ligne à son étape 1, `journal_entries.rs:308-313`), un cycle neuf avec la contre-passation
   d'une écriture sans rapport datée d'un exercice postérieur, et un verrou de fin d'intervalle sur la
   société suivante. « Les exercices du groupe » sont ceux qui portent **une ligne** du groupe — tous
   entre le plus ancien et le plus récent ; un exercice **intermédiaire** sans ligne du groupe n'est pas
   verrouillé : aucune de ses lignes ne change, aucun verdict n'en dépend. Les tenir tous, plutôt que le
   seul plus récent, c'est la règle commune à tous les écrivains de lignes (création, `PUT`, `DELETE`
   tiennent l'exercice de leur écriture) et ce qui sérialise le lettrage avec leur clôture (P8-1 d'août).
   *Remarque* : le verdict du groupe ne dépend que de son exercice le plus récent M — si une ligne d'un
   exercice X plus ancien est en période ouverte, X est ouvert, aucun postérieur à X n'est clos (donc M
   est ouvert et aucun postérieur à M ne l'est), et une ligne de M est datée après celle de X, donc après
   la borne. L'implémentation peut s'en servir ; les tests restent ceux de la règle par ligne.

   ⚠️ **(a) ouvre la vue `REPEATABLE READ` avant l'attente des verrous de (b)** *(F3-4 de P3 — la
   version précédente affirmait « la doctrine “le verrou d'abord” est tenue », inexact)*. L'acte 1 est
   verrouillant et n'ouvre pas de vue : (a) est la **première lecture ordinaire** de la transaction. Ce
   qui se lit ensuite ordinairement lit donc l'état d'avant l'attente — et chaque cas est tolérable :
   (c), par (α)-(β) ; la lettrabilité (rang 4), parce qu'un compte qui cesse d'être lettrable laisse ses
   groupes valides (C104) ; l'appartenance à une pièce (rang 5, `reversal_blockers`), parce qu'aucune
   écriture existante ne devient après coup celle d'une pièce (une pièce crée son écriture dans sa propre
   transaction ; le rapprochement d'une écriture existante n'est pas une pièce, R5) ; la borne
   `books_locked_through`, par la tolérance écrite plus bas. La doctrine « le verrou d'abord » est tenue
   pour ce que le lettrage **écrit** ou **juge sous verrou** — lignes et en-têtes (acte 1), `status` des
   exercices (b) —, pas pour ces lectures-là : limite écrite.

   ⚠️ **Les tests reconnaissent ces requêtes à leur texte** *(F3-6 de P3)* : le doc-comment de la
   primitive le dit, comme celui de `FIND_LATER_CLOSED_SQL` (`fiscal_years.rs:664-667`). Motifs à
   employer (T12) : acte 1 de la création `["jel.id IN", "FOR UPDATE"]`, de la dissolution
   `["jel.lettering_key = ", "FOR UPDATE"]`, verrou d'exercice
   `["SELECT id, start_date, status FROM fiscal_years WHERE id = ", "FOR UPDATE"]` — la liste de colonnes
   l'oppose à `LOCK_EARLIER_BY_ID_SQL` (`SELECT id FROM …`) et à l'étape (c) de la clôture
   (`SELECT id, company_id …`) de la 15-12a. ⛔ Ne **pas** réutiliser `["je.fiscal_year_id", "FOR
   UPDATE"]` (`journal_entries_modification.rs:466`, étape 2 de `delete_in_tx`) : l'acte 1 du lettrage le
   contient aussi ; de même `["ORDER BY start_date ASC", "FOR UPDATE"]` (`:410`, `:511`), que la forme de
   P2 contenait.

   C'est **ce** verrou qui empêche une clôture concurrente de passer entre la lecture « exercice
   ouvert » et l'écriture (P8-1 d'août, conservé). ⛔ Pas de verrou **partagé** : un S suivi d'un X
   sur la même ligne d'exercice (dissolution système puis contre-passation dans une même
   transaction, 15-1a2) est un interblocage d'escalade **structurel** (F7 de P1). ⛔ Pas d'ordre par
   `id` : des `id` non chronologiques existent dès qu'un exercice antérieur est créé après coup.
   L'ordre ascendant est celui de la clôture (15-12a AC 3 (b') puis (c) et (d)), de `reopen` (Y puis les
   postérieurs) et de la garde 15-8a : **aucun cycle** entre eux et le lettrage sur les exercices (la
   clôture verrouille **tous** ses antérieurs, le lettrage un sous-ensemble, dans le même sens).
   ⚠️ Sur `main`, `close_fiscal_year` est `SansEcritureAuJournal` (`audit_route_registry.rs:255`),
   donc **non rejouée** ; la **15-12a**, prérequis, l'enveloppe (son AC 6) — si bien qu'un
   interblocage avec elle ne rendrait plus un 500. L'ordre reste la défense de premier rang, le
   rejeu la seconde (Pattern 5).
3. **En mode `System { held_open_fiscal_year_id }`, la primitive ne verrouille AUCUN exercice** :
   l'appelant tient **déjà**, `FOR UPDATE`, un exercice **ouvert** — la contre-passation, celui du
   jour (`find_open_covering_date`) ; la 15-1a2, celui qu'elle établit pour chaque site — et le
   passe. La primitive vérifie qu'au moins une ligne lue à l'étape 1 porte ce `fiscal_year_id` —
   **ce contrôle-là** se fait sans requête, sur les lignes déjà lues ; sinon `DbError::Invariant`
   (*fail-loud* : c'est un défaut de l'appelant, pas une course).
   ⚠️ **Une lecture d'exercice, une seule, et elle ne verrouille rien** *(validation P4 — R4-2 de la
   15-1a-i = R4-1 de la 15-1a-ii, C128 ; « sans requête » valait pour toute la primitive jusqu'en P3, et
   laissait sans source le `fiscalYearName` que l'audit exige pour toutes origines — AC10)* : après
   l'acte 1, la primitive lit le **nom** des exercices du groupe par une lecture **ordinaire, non
   verrouillante** — constante `LETTERING_FISCAL_YEAR_NAMES_SQL` : `SELECT id, name FROM fiscal_years
   WHERE company_id = ? AND id IN (<fiscal_year_id distincts des lignes, lus à l'acte 1>)` — pour la
   seule réponse et l'audit (AC6, AC10). Sans effet sur l'ordre des verrous : elle n'en pose aucun, et
   ne prend donc pas les exercices de l'origine **après** celui du jour. Ce qu'elle lit ne juge rien :
   seul `name` en est retenu, et un `update_name` concurrent (`fiscal_years.rs:400`, qui n'écrit que
   `name`) ne ferait qu'auditer le nom d'avant — tolérable pour un libellé, la clé `fiscalYearId`
   restant exacte. ⛔ Pas de jointure de `fiscal_years` dans l'acte 1 : sous `FOR UPDATE`, elle
   verrouillerait les exercices hors de l'ordre de la clôture (MariaDB n'a pas de `FOR UPDATE OF`). En
   mode `Manual`, le nom vient de la lecture (a) du point 2, qui le porte déjà : **pas** de seconde
   lecture. **Pourquoi** aucun verrou : la contre-passation tient l'exercice du jour quand elle lettre ;
   reprendre les exercices de l'origine (antérieurs) **après** lui inverserait l'ordre
   `start_date` contre la clôture de la 15-12a. Le cycle serait rattrapé — la clôture devient
   rejouée avec la 15-12a (AC 6), la contre-passation l'est déjà — mais l'ordre l'évite, ce qui vaut
   mieux qu'un rejeu *(motif corrigé en validation P2 : la version précédente disait la clôture « non
   rejouée », ce que la 15-12a rend faux)*.
4. L'`UPDATE` final (`… WHERE id IN (…) AND lettering_key IS NULL` pour la création, `… WHERE
   lettering_key = ?` pour la dissolution) vise des lignes **tenues** : son nombre de lignes
   affectées ne peut pas différer de l'attendu sans défaut. S'il diffère quand même, c'est un
   refus **métier**, pas un 500 : `DbError::LetteringConcurrentChange` → **409
   `LETTERING_CONCURRENT_CHANGE`** (« le groupe a changé entre-temps ; réessayez »), jamais
   `DbError::Invariant`.
   ⚠️ **Ce que `rows_affected()` compte ici** *(validation P4 — F4-4, C131)* : sqlx pose
   `CLIENT_FOUND_ROWS` à la connexion (`sqlx-mysql` 0.8.6, `connection/stream.rs:46`,
   `Capabilities::FOUND_ROWS`) ; le nombre rendu est celui des lignes **trouvées** par le `WHERE`, non
   des lignes **changées**. C'est ce qu'on veut comparer. Conséquence pour le test : la comparaison vit
   dans une fonction **pure**, `kesh_core::lettering::check_rows_affected(expected, actual)`, appelée
   par les deux primitives — car aucun montage de test ne peut faire différer le compte sans crochet
   de production. Les lignes sont tenues depuis l'acte 1, si bien qu'aucun écrivain concurrent n'y
   parvient ; et un déclencheur `BEFORE UPDATE` posé par le test, qui remettrait `NEW.lettering_key`
   à `NULL`, **ne change pas** le nombre de lignes **trouvées** — il ne ferait que fausser la marque
   en silence (un déclencheur ne peut ni écarter une ligne déjà trouvée ni écrire dans sa propre
   table, erreur 1442).

**La règle des périodes, symétrique** *(révision de C94 — C105 ; étendue au verrou de période et à
l'état hérité en validation P2 — F-3, C113)*. Une ligne est **« en période ouverte »** si, et
seulement si :

- (i) son exercice est `Open` ;
- (ii) **aucun exercice postérieur** à son exercice n'est `Closed` — sans objet dans un état sain
  (les clos forment un préfixe, 15-12a), décisif dans l'**état hérité** (sauvegarde v0.12.x
  restaurée, SQL direct : « N ouvert, N+1 clos ») que ni la 15-12a ni la 15-12b ne gardent pour le
  lettrage ; lu **sans verrou** (point 2 (c), C125) ;
- (iii) sa date est **strictement postérieure** à `companies.books_locked_through` (ou la borne est
  `NULL`) — seuil inclusif comme partout ailleurs (`journal_entries.rs:351`, `:1117`, `:1737`).

⚠️ *Vocabulaire* : « en période ouverte » ne se confond pas avec « ligne ouverte » (non lettrée,
R1, 15-1b) — d'où ce nom.

En mode `Manual`, **lettrer comme délettrer** exige qu'**au moins une** ligne du groupe soit en
période ouverte ; un groupe dont **toutes** les lignes sont en période close **ne se crée ni ne se
défait** — 409 **`LETTERING_ALL_LINES_IN_CLOSED_PERIODS`** dans les deux cas (une seule clé ; le code
`LETTERING_FISCAL_YEARS_CLOSED` de la validation P1 est **retiré**, il ne disait pas le verrou de
période). Motif (F6 de P1) : la vue « au » d'une date (15-1b) ne connaît pas la date du lettrage ; un
groupe entièrement daté ≤ X y est compté soldé à X **quelle que soit sa date de pose**. Lettrer deux
lignes de 2025 (clos) ferait disparaître des postes ouverts « au 31.12.2025 » une ligne que la même
vue montrait ouverte la veille. **Le même raisonnement vaut pour le verrou de période** (F-3 de P2) :
lettrer aujourd'hui deux lignes du 15 mars, période verrouillée jusqu'au 31 mars, réécrirait l'état
« postes ouverts au 31.03 » tiré pour le fiduciaire — exactement ce que le verrou existe pour
empêcher (manuel, `user-manual.tex:578-583` : « verrouiller est le geste qui fige un trimestre
déclaré »). **Et pour l'état hérité** : un groupe tout entier dans N ouvert, sous N+1 clos, réécrirait
les postes ouverts « au » d'une date de N+1, que le bilan clos de N+1 reprend.

**Lecture de la borne** : `books_locked_through` se lit **ordinairement**, après le point 2, dans la
vue ouverte par sa lecture (a) (limite écrite au point 2) — la
même tolérance qu'à la création et au `PUT` (`journal_entries.rs:288-306`, `:1200-1203`) : une pose
de borne commitée pendant l'attente des verrous n'est pas vue ; la borne ferme le passé, pas
l'instant présent. Aucun verrou sur `companies` (pas de sentinelle, ci-dessous).

En mode `System`, la règle n'est **pas** évaluée par la primitive : l'exercice tenu (point 3) couvre
une ligne que l'appelant **vient d'écrire** (le miroir d'une contre-passation, l'écriture d'un
règlement), et cette écriture a passé les gardes de la création — exercice ouvert, verrou de
période (`PeriodLocked`, étape 0-bis de `create_in_tx_inner`), et, avec la 15-12b, le filet
« exercice postérieur clos ». ⚠️ Sans la 15-12b (ordre préféré, non requis — C112), le (ii) n'est
pas garanti en mode `System` dans l'état hérité : limite écrite, c'est celle des écritures du journal
elles-mêmes, que la 15-12b ferme.

**Interblocages résiduels, nommés** *(complété en validation P2 — R2-7, F-5 ; revu en validation P3 —
C125, L2, F3-5)* :

- avec `update_in_tx` / `delete_in_tx` (en-tête puis lignes) sur une écriture commune, l'acte 1
  peut prendre une ligne avant l'en-tête — l'acte 1 verrouille, ligne par ligne du parcours, la ligne
  **puis** son en-tête : ordres opposés, cycle possible ; c'est la garde `lettering_guard` de la 15-1a-ii
  qui le rencontre (son AC8 dit désormais que la défense est le rejeu, non l'ordre — F-4 de sa P4) ;
- avec la **contre-passation** (R6, 15-1a-ii) : `reverse_in_tx_inner` tient l'en-tête **et
  l'exercice** Z de l'origine (jointure `FOR UPDATE`, étape (1)), puis verrouille les **lignes** de
  l'origine (lecture étendue de R6) et parcourt les exercices depuis le premier
  (`find_open_covering_date`, `fiscal_years.rs:550`) ; un lettrage `Manual` tient lignes et en-têtes
  puis les exercices **de son groupe** — cycle possible sur les lignes (une ligne commune) ou sur les
  exercices (Z est un exercice du groupe, et le lettrage tient un exercice plus ancien que le parcours
  de la contre-passation demande). ⚠️ Depuis C125, **plus** de cycle avec la contre-passation d'une
  écriture **sans rapport** datée d'un exercice postérieur : le lettrage ne verrouille plus les
  postérieurs (R3-2 de P3) ;
- avec **toute insertion de ligne d'écriture**, le temps de la dissolution du groupe de **plus petite
  clé** *(F3-5 de P3 ; raisonné sur le moteur, non exécuté)* : l'acte 1 de la dissolution parcourt
  `idx_jel_lettering` par égalité, et InnoDB verrouille en `REPEATABLE READ` les entrées de la clé
  **et le trou qui les précède** ; les lignes ouvertes (`lettering_key = NULL`) sont rangées en tête de
  l'index, par `id`, si bien qu'une ligne neuve `(NULL, id_max + 1)` s'insère **juste avant** la plus
  petite clé non nulle. Toute insertion — de **toute société** — attend donc la fin de cette
  dissolution ; dans la même société, l'attente devient un cycle si l'écrivain tient l'exercice que la
  dissolution `Manual` demande à son point 2. Brève (une dissolution), rare (la plus petite clé est un
  vieux groupe), absorbée par le rejeu des deux côtés (toutes ces routes sont `Rejouee`) : écrite, non
  corrigée — et elle contredit le « local à une installation mono-société » de la limite F-11 du point 1,
  qui le dit désormais ;
- ~~avec **toute création de groupe**, le temps de la dissolution du groupe de **plus grande clé**~~
  — ⛔ **démenti au T0** (C-15-1a-i-1) : l'`UPDATE` d'une création ne se heurte pas au trou tenu par la
  dissolution ; le texte raisonné ci-dessous est gardé pour l'historique —
  *(validation P4 — F4-1 ; raisonné sur le moteur, non exécuté)* — et ce cas-là est **courant**, non
  rare : c'est le geste « j'annule le lettrage que je viens de poser ». La dissolution tient aussi le
  trou **qui suit** la dernière entrée de sa clé, ici `(Kmax, +∞)` jusqu'au supremum ; or une clé neuve
  est le plus petit `id` de lignes récentes, donc presque toujours **supérieure** à toutes les clés
  existantes : l'`UPDATE` d'une création concurrente, qui insère `(k, id)` dans l'index, attend — **de
  toute société**. Cycle possible, même nature que le précédent : la dissolution tient le trou puis
  attend un exercice à son point 2 ; la création tient cet exercice (point 2 (b)) puis attend le trou à
  son `UPDATE`. Les deux routes sont `Rejouee` : le rejeu absorbe, jamais un 500. Même cause, effet
  mineur : un `DELETE /letterings/{key}` sur une clé **inexistante** pose, le temps de sa transaction
  (brève : 404 aussitôt), un verrou sur le trou `(clé précédente, clé suivante)`. Écrit, non corrigé ;
  **constaté en T0** ;
- ces routes sont **toutes** `(Traced, Rejouee)` : `PUT` et `DELETE /journal-entries/{id}`
  (`audit_route_registry.rs:199/203`), `reverse_journal_entry` (`:205`),
  `unvalidate_invoice_handler`, et les **deux** routes neuves (AC12) — **six** routes d'écriture
  directe, de dévalidation et de lettrage *(« quatre » jusqu'en P2 : décompte faux, L2 de P3)* —, plus
  les **quatre** annulations qui contre-passent (`cancel_supplier_invoice`,
  `cancel_supplier_invoice_settlement`, `cancel_invoice_settlement_handler`,
  `post_cancel_reconciliation` — relevé de la 15-12a, AC 3) : le rejeu absorbe le cycle ;
- avec la clôture (15-12a, exercices croissants, un par un par clé primaire) : aucun cycle sur les
  exercices (même sens ; le lettrage en prend un sous-ensemble), et la clôture ne touche ni aux lignes
  ni aux en-têtes ; elle devient rejouée de toute façon (15-12a AC 6).

⚠️ **Pas de sentinelle `companies`** : sans compteur, elle n'a plus rien à protéger. La défense
contre un interblocage résiduel est le **rejeu** (AC12), conformément au Pattern 5
(`docs/MULTI-TENANT-SCOPING-PATTERNS.md:291`, `:311`, `:338` : l'ordre réduit la fréquence, le
rejeu est la défense — `:298-304` jusqu'en P4, qui est le bloc de l'ordre global des verrous, R4-6).

## Critères d'acceptation

**AC1 — Schéma.** La migration R1, telle quelle (nom indicatif `2026100X000001_journal_entry_lines_lettering.sql`,
date à prendre **au développement**, postérieure à la dernière migration de `main`). **Breaking
au sens de P1** : relève `kesh_version_min_required` à `'0.13.0'` en dernière instruction (P2),
avec le commentaire `-- BREAKING (P1) :` **arrêté à R1**, recopié tel quel en tête ; **dans le même commit**, les dix
`crates/*/Cargo.toml` passent à `0.13.0` et `Cargo.lock` suit (P2-bis) ; entrée
`EXEMPT_MIGRATIONS` `ExemptionBasis::Durable` sur le patron de `post_restore.rs:512-525` (P7 —
`every_data_backfill_migration_is_triaged` doit rester vert) ; **gate runtime complet** avant
`done` (`admin_backup_e2e`, `admin_full_import_e2e`, `migrations_fresh_install`). Tests touchés :
`crates/kesh-db/tests/migrations_fresh_install.rs:244` (`assert_eq!(min_required, "0.10.0")`) et
son doc-comment `:230` (« le DERNIER bump breaking du repo — `'0.10.0'` ») passent à `0.13.0` ;
**et deux tests de `crates/kesh-db/tests/migrations_upgrade_path.rs` qui lisent le `min_required`
réel de la base migrée rougissent au bump** *(validation P2 — R2-1 = F-2 ; la version précédente
désignait comme « à exécuter » le seul test insensible)* :
`downgrade_protection_aligned_when_binary_equals_min` (`:509`, appel `:513` avec `"0.10.0"`,
message `:517`) passe à un binaire `"0.13.0"` → `Aligned` ;
`downgrade_protection_binary_ahead_when_binary_greater` (`:524`, assertion `:530`) passe à
`db_min == "0.13.0"` et un binaire `"0.14.0"` → `BinaryAhead` ; leurs commentaires `:510` et
`:525-526` (« depuis le bump breaking de la Story 22-1 ») nomment désormais la 15-1a.
`downgrade_protection_rejects_old_binary` (`:484`) pose lui-même `0.99.0` : insensible au bump, il
reste une simple non-régression. `grep -rn "0\.10\.0" crates/*/tests` trié à la main (valeur, pas
formulation) — au 2026-10-09 il rend exactement ces sites (`migrations_upgrade_path.rs:510, 513,
517, 525, 526, 530` ; `migrations_fresh_install.rs:230, 244`), plus le squash de test, régénéré. Le squash de test est **régénéré** (`scripts/regen-test-schema.sh`, jamais édité),
la migration est inscrite à `crates/kesh-db/migrations.sha384` (`published_migrations_keep_their_checksums`),
et sa ligne est ajoutée à `docs/migrations-idempotence-audit.md` (P5) avec les **deux** sites du
total et les compteurs de partition **recomptés depuis le tableau** (au 2026-10-08 : 75 →
76, `tracked-by-sqlx` 67 → 68 — valeurs à recompter au développement, d'autres stories pouvant
avoir ajouté des migrations entre-temps). P6 : `grep -rn "migrations.len()\|apply_migrations_up_to" crates/`
et inspection de chaque site.

**AC2 — Le code.** `kesh_core::lettering::code_from_key(u64) -> String` et
`key_from_code(&str) -> Option<u64>` (R2), avec les cas limites nommés (`1, 26, 27, 52, 53, 702,
703`) et un aller-retour sur 1..=100 000. Entrée invalide (vide, chiffre, `Ä`, espace, et les trois
caractères que la casse Unicode ramènerait à l'ASCII — `ß`, `ſ`, `ı` : R2, F4-2) → `None` ; minuscules
ASCII acceptées (`aa` → `Some(27)`).
**Débordement** *(validation P2 — F-8)* : `key_from_code` calcule en arithmétique **vérifiée**
(`checked_mul`, `checked_add`) et rend `None` au-delà de `i64::MAX` (la colonne est un `BIGINT`
signé) — sans quoi un code assez long (vingt lettres) déborde `u64` et **boucle silencieusement** en release
(`overflow-checks` éteint) et pourrait désigner une clé réelle. Même borne pour le `{key}` numérique
de la route (AC6). Cas de test : vingt `Z` → `None` ; `"CRPXNLSKVLJFHG"` (`i64::MAX`, quatorze
lettres) → `Some(i64::MAX)` ; `"CRPXNLSKVLJFHH"` → `None`.

**AC3 — La primitive.** `create_group_in_tx` refuse, **dans cet ordre**, avec un `DbError` dédié
par cause :

| ordre | cause | erreur | HTTP / code (route manuelle) |
|---|---|---|---|
| 1 | moins de 2 identifiants, ou doublon | `LetteringTooFewLines` | 400 `LETTERING_TOO_FEW_LINES` |
| 2 | une ligne introuvable **ou d'une autre société** | `NotFound` | **404**, indiscernable (AC11) |
| 3 | comptes différents | `LetteringAccountsDiffer` | 409 `LETTERING_ACCOUNTS_DIFFER` |
| 4 | compte non lettrable (R4) | `LetteringAccountNotLetterable` | 409 `LETTERING_ACCOUNT_NOT_LETTERABLE` |
| 5 | (manuel seulement) ligne d'une pièce (R5) | `LetteringLineOwnedByDocument { blocker, document_id, document_label }` | 409 `LETTERING_LINE_OWNED_BY_DOCUMENT`, `details.documentId` / `details.documentNumber` |
| 6 | ligne déjà lettrée | `LetteringLineAlreadyLettered { code }` | 409 `LETTERING_LINE_ALREADY_LETTERED`, `details.code` |
| 7 | somme non nulle | `LetteringUnbalanced { difference }` | 409 `LETTERING_UNBALANCED`, `details.difference` (décimal en chaîne) |

et sinon pose `lettering_key = MIN(id)` et l'origine sur **toutes** les lignes, en **un**
`UPDATE … WHERE id IN (…) AND lettering_key IS NULL`, dont le nombre de lignes affectées est
**vérifié** (≠ attendu → `DbError::LetteringConcurrentChange`, 409
`LETTERING_CONCURRENT_CHANGE`, R7 point 4 — jamais un succès partiel, **jamais** un 500).

Entre les causes 4 et 5 s'intercale, **en mode `Manual`**, la règle des périodes de R7 : aucune
ligne « en période ouverte » (exercice clos, exercice postérieur clos, ou date ≤ verrou de période)
→ 409 `LETTERING_ALL_LINES_IN_CLOSED_PERIODS` (rang **4 bis**). En
mode `System`, la cause 5 ne s'applique pas (la contre-passation d'une pièce est légitime) et le
rang 4 bis est remplacé par le contrôle de l'exercice tenu (R7 point 3, `Invariant`).

⚠️ La cause 6 couvre la **ré-apparition** d'une ligne déjà lettrée — AC10 de la fiche d'août,
conservé : sans elle, l'ancien partenaire resterait « soldé » seul.

**AC4 — Lettrer exige au moins une ligne en période ouverte** *(révision de D3 et de C94 —
C105 ; étendue au verrou de période et à l'état hérité — C113)*. Permis à cheval (une ligne en
période close, une en période ouverte) ; **refusé** quand **toutes** les lignes sont en période
close (409 `LETTERING_ALL_LINES_IN_CLOSED_PERIODS`), quelle que soit la raison de chacune. Tests
nommés :
`lettering_across_closed_and_open_is_accepted`, `lettering_all_in_closed_years_is_refused`,
`lettering_all_in_locked_period_is_refused` (deux lignes datées ≤ `books_locked_through`, exercice
ouvert), `lettering_at_the_lock_boundary` (une ligne datée **exactement** de la borne → close ; du
lendemain → ouverte : seuil inclusif), `lettering_across_locked_and_unlocked_is_accepted`,
`lettering_under_a_later_closed_year_is_refused` (état hérité « N ouvert, N+1 clos » posé par SQL
direct, comme les tests de la 15-8a ; groupe tout en N → refus).
⚠️ D3 (« lettrer toujours permis ») était un arbitrage de Guy (août) : cette révision, prise en
autonomie, est signalée pour sa revue finale.

**AC5 — La dissolution.** `dissolve_group_in_tx(…, mode)` remet `lettering_key` et
`lettering_origin` à `NULL` sur **toutes** les lignes du groupe (nombre vérifié). En mode
`Manual` (la route) :

dans cet ordre :

1. groupe d'origine `document` → refus 409 `LETTERING_IS_DOCUMENT` *(son lettrage suit la pièce :
   annuler le règlement, pas délettrer)* ;
2. groupe `reversal` dont **une** ligne appartient à une pièce (motifs `OwnedBy*` de R5, rangs 3 à
   6 de `reversal_blockers`, évalués sur l'écriture de chaque ligne) → refus 409
   `LETTERING_LINE_OWNED_BY_DOCUMENT`, mêmes `details` que la cause 5 d'AC3 *(C106 : sinon la
   paire resterait ouverte pour toujours, R5 interdisant de la relettrer)* ;
3. **aucune** ligne en période ouverte (R7 : exercice clos, exercice postérieur clos, date ≤
   verrou de période) → refus 409 `LETTERING_ALL_LINES_IN_CLOSED_PERIODS` (R7, verrou **exclusif**
   sur les exercices **du groupe**, un par un par clé primaire dans l'ordre des `start_date` — R7
   point 2, C125) — tests
   `dissolution_all_in_closed_years_is_refused`, `dissolution_all_in_locked_period_is_refused`,
   `dissolution_under_a_later_closed_year_is_refused` ;
4. sinon (`manual`, `reversal` sans ligne de pièce) : dissous.

⛔ **La dissolution n'exige jamais la lettrabilité** *(C104)* : un groupe dont le compte a été
retypé ou rattaché à un compte bancaire depuis sa création se dissout normalement.

En mode `System { held_open_fiscal_year_id }` (appelé par la 15-1a2 pour une pièce) : pas de
refus 1 ni 2, aucun verrou d'exercice pris par la primitive, et la présence d'une ligne sur
l'exercice tenu est vérifiée (R7 point 3 ; manquement → `DbError::Invariant`, *fail-loud* : défaut
de l'appelant).

**AC6 — Routes.**

| méthode | chemin | corps / paramètres | réponse | rôle |
|---|---|---|---|---|
| `POST` | `/api/v1/letterings` | `{ "lineIds": [i64, …] }` (2 à 200) | `201 { key, code, origin: "manual", accountId, lines: [ { id, entryId, entryNumber, fiscalYearId, fiscalYearName, date, debit, credit } ] }` | Comptable, Admin |
| `GET` | `/api/v1/letterings/{key}` | — | `200` même forme, `origin` réel | Consultation et plus |
| `DELETE` | `/api/v1/letterings/{key}` | — | `204` | Comptable, Admin |

⚠️ **`fiscalYearId` et `fiscalYearName` par ligne** *(validation P3 — F3-8, C127)* : le numéro
d'écriture **repart à 1 à chaque exercice** (`journal_entries.rs:2304-2306`) ; dans un groupe à cheval,
deux lignes peuvent porter « écriture n° 12 » de deux exercices. Le nom vient de la lecture (a) de R7
point 2 (création, dissolution en mode `Manual`), de la lecture ordinaire de R7 point 3 (mode `System` —
C128), ou d'une jointure ordinaire (`GET`).

⚠️ `{key}` accepte **la clé numérique ou le code** (`27` ou `AA`) — l'inverse d'AC2 ; une valeur
invalide → 404, y compris une clé numérique ≤ 0 ou au-delà de `i64::MAX` et un code qui déborde
(AC2). ⛔ **Scoping du `GET`/`DELETE`** : `… FROM journal_entry_lines jel JOIN
journal_entries je ON je.id = jel.entry_id WHERE jel.lettering_key = ? AND je.company_id = ?` —
aucune ligne → **404** (le `GET` en lecture ordinaire, sans verrou ; le `DELETE` par le premier acte
**verrouillant** de R7). ⛔ **Ordre d'évaluation du `POST`** (garantie d'AC11) : les lignes se
chargent par **une** requête scopée — le premier acte **verrouillant** de R7 (`… WHERE jel.id IN
(…) AND je.company_id = ? … FOR UPDATE`), aucune lecture ordinaire avant ; moins de lignes trouvées
que demandées → 404 **avant** toute autre cause hors 400 de forme (précédent :
`kesh-api/src/routes/products.rs:343`).

Handlers : `letterings::create_lettering` (`POST`), `letterings::get_lettering` (`GET`),
`letterings::delete_lettering` (`DELETE`) — les deux **écrivains** (`create_lettering`,
`delete_lettering`) s'inscrivent à `LIB_ROUTES` (AC12).

⚠️ Le plafond de 200 lignes par groupe borne le corps (et l'`IN (…)`) ; au-delà → 400
`LETTERING_TOO_MANY_LINES`.

**AC7 — Le lettrage manuel n'existe pas encore à l'écran** : la 15-1a-i livre l'API ; l'écran est
la 15-1c. Le CHANGELOG le dit (AC15 part i).

**AC10 — Audit.** `lettering.created` et `lettering.removed`, entité `lettering`, `entity_id` = la
clé ; `details` : `code`, `origin`, `accountId`, `accountNumber`, les lignes (`id`, `entryId`,
`entryNumber`, `fiscalYearId`, `fiscalYearName`, `debit`, `credit` — l'exercice par ligne : C127,
F3-8 de P3 ; l'audit est une trace durable, il doit se lire sans recouper `entryId` ; le nom vient de la
lecture (a) de R7 point 2 en mode `Manual`, de la lecture ordinaire de R7 point 3 en mode `System` —
C128). Toutes origines, dans la transaction du geste (patron
`audit_log::insert_in_tx`, `audit_log.rs:62`). Codes inscrits à `audit_labels.rs`
(`ENTITY_TYPES`, `ACTIONS`, triés) et libellés dans les **quatre** locales
(`audit_label_registry.rs`). Acteur par `NewAuditLogEntry::for_actor` (clé d'API portée,
C-15-8-8, patron de `delete_in_tx`). ⚠️ Pour le lettrage `reversal` (15-1a-ii), l'acteur est
l'auteur de la contre-passation, **sans** clé d'API (`reverse_in_tx` ne la porte pas — écart nommé,
R3) ; son test, `reversal_lettering_is_audited_by_the_reverser`, est dans la 15-1a-ii. Tests nommés
ici : `lettering_created_is_audited_with_its_lines` (présence, `details` complets, acteur et clé
d'API d'un `POST` par clé), `lettering_removed_is_audited`.

**AC11 — Anti-IDOR.** Deux lignes d'une autre société → 404 ; une ligne de chaque société → 404 ;
`GET` et `DELETE` d'une clé d'une autre société → 404. **Aucun** message ne nomme une ligne
étrangère. ⚠️ L'indiscernabilité porte sur la **réponse**, non sur sa **durée** : limite écrite à
R7 point 1 (verrous posés avant le filtre de société, F-11 de P2).

**AC12 — Rejeu et registres.** Les routes `POST` et `DELETE` s'inscrivent à `LIB_ROUTES`
(`audit_route_registry.rs:168`) en `(Traced, Rejouee)` et appellent une `ENVELOPPE` dans le corps
du handler ; les comptes figés de `the_replay_partition_is_what_the_story_declares` et de
`the_registry_partition_is_what_the_story_declares` sont **recomptés**. Le `GET` n'écrit rien.
Handlers inscrits : `letterings::create_lettering` (`post`) et `letterings::delete_lettering`
(`delete`).
Une ligne est ajoutée au tableau « Where This Applies » de
`docs/MULTI-TENANT-SCOPING-PATTERNS.md` (séquence de R7).

**AC13 — Le garde-fou d'inventaire est trié, pas contourné.** `COLONNES_DES_LIGNES`
(`journal_entries_modification.rs:78`) gagne `lettering_key` et `lettering_origin`, avec un
commentaire qui renvoie au gel des lignes lettrées (motif `Lettered`, AC8 de la 15-1a-ii — tant
qu'elle n'est pas mergée, le commentaire le dit) ; le test de mutation
`the_inventory_guard_turns_red_on_each_mutation` (`:197`), qui ajoute aujourd'hui une colonne
factice **nommée `lettering_id`** (`:220`), garde son sens (le nom ne collisionne plus — la vraie
colonne s'appelle `lettering_key` — mais le vérifier). Le point (1 bis) reste **vide** (R1). Et un
**test d'invariant** (`lettering_invariants`) parcourt la base d'un scénario mêlé (groupes `manual`
ici ; la 15-1a-ii y ajoute des groupes `reversal`) et vérifie :
tout groupe a ≥ 2 lignes, un seul compte, une seule origine, une seule société, une somme nulle,
et `lettering_key = MIN(id)`. ⛔ Il ne contrôle **pas** la lettrabilité du compte — par décision
(C104 : un compte peut cesser d'être lettrable après coup). Le doc-comment
`journal_entries_modification.rs:181`, qui annonce `journal_entry_lines.lettering_id`, est corrigé
en `lettering_key`.

**AC14 — Exposition et export.** `JournalEntryLine` (`entities/journal_entry.rs:151`) et les
sites qui la construisent gagnent les deux champs — inventaire relevé par `grep` (P1), **à
recompter au développement** : `LINE_COLUMNS` (`journal_entries.rs:80`, qui sert les **six**
`query_as::<_, JournalEntryLine>` de `:492`, `:708`, `:918`, `:1442`, `:1520`, `:1755` — `:1442` et
`:1755` sont les lectures « avant » d'`update_in_tx` et de `delete_in_tx`, qui alimentent
`entry_snapshot_json` : c'est là que la vérification ci-dessous porte *(R2-5 ; quatre étaient cités
jusqu'en P2)*), la liste littérale de `list_all_lines_by_company` (fonction `:1824`, requête
`:1828`), et les deux littéraux `JournalEntryLine {` de `kesh-api/src/routes/journal_entries.rs:834`
et `kesh-api/src/exports/csv_tables.rs:1687` (`sample_line`). Le `SELECT` de
la contre-passation (`:2266`) est un tuple, étendu pour R6 et non pour la struct. ⚠️
`kesh-report` n'est **pas** touché : ses requêtes sont des agrégats (`project_report.rs:227-264`,
aucun `JournalEntryLine` — vérifié). Puis :
`JournalEntryLineResponse` (`kesh-api/src/routes/journal_entries.rs:102`) expose `letteringKey`,
`letteringCode`, `letteringOrigin` (nuls si ouverte) ; le type frontend
`journal-entries.types.ts:11` suit, les trois champs **requis et nullables** (`letteringKey: number |
null`, …) — l'API les rend toujours ; les déclarer optionnels mentirait sur le contrat *(validation P4 —
R4-3 = F4-3, C131)*. ⚠️ **Fixtures typées à compléter** (sans quoi `npm run check` rougit — relevé
`git grep -n "lineOrder:" origin/main -- frontend/src frontend/tests` au 2026-10-09) :
`JournalEntryForm.edit.test.ts:66-67` (`ENTRY: JournalEntryResponse`, deux lignes) ;
`form-helpers.test.ts:29` (`mockEntry`, dont le retour est typé `JournalEntryResponse`) et les trois
littéraux passés à `lineResponseToDraft` (`:38-44`, `:50-56`, `:62-68`) — trois champs `null` chacun.
Les autres littéraux de `form-helpers.test.ts` (`:89-116`, `:185-186`) passent par `mockEntry`, dont
le paramètre a un type local (`:10-17`) : complétés une fois à `:29`, sans autre retouche. ⚠️ `is_no_op_change` (`:1130`) et `entry_snapshot_json`
(`:943`) : vérifier qu'ils n'en sont pas faussés — le premier compare champ par champ, le second
construit son JSON à la main (P8 d'août) ; une écriture lettrée n'atteindra plus l'update une fois
le gel posé (AC8, 15-1a-ii). ⛔ **Export CSV** (`csv_tables.rs:350`) : `lettering_key` et `lettering_origin` **exportées**
— l'export existe pour que l'utilisateur vérifie ce que Kesh affirme ; header littéral et
struct changent **dans le même commit** (la garde `chaque_colonne_du_schema_est_exportee_ou_ecartee`
le contrôle désormais au niveau du schéma). L'**export global** consomme
`serialize_journal_entry_lines_csv` : il gagne les deux colonnes **par** ce changement, sans code
propre. ⚠️ **Sauvegarde et `reset_demo`** : **rien** à faire — l'export de sauvegarde lit les
colonnes dynamiquement (`kesh-db/src/backup.rs:106-159`), et c'est le bénéfice de R1 (colonnes d'une table déjà
inventoriée), à **vérifier** par leurs gardes respectives et non à supposer. Tests nommés :
`journal_entry_line_response_exposes_lettering` (les trois champs, nuls sur une ligne ouverte,
`letteringCode` = `code_from_key`), `csv_export_carries_lettering_columns` (header et valeurs).

**AC15 — Documentation, part (i) : ce qui décrit la marque et ses routes** *(la part (ii) — réserve
« lettrée » sur la modification et la suppression, refus `ENTRY_LETTERED`, contre-passation qui lettre,
entrées #532 du CHANGELOG, sixième condition du manuel — est dans la 15-1a-ii)*.
`docs/api-external.md` : les trois routes, leurs corps (lignes avec `fiscalYearId`/`fiscalYearName`,
C127), et **le tableau de leurs refus** (patron des sous-sections « règlements » `:317` et « solde »
`:334`), dont `LETTERING_ALL_LINES_IN_CLOSED_PERIODS` et la règle des périodes. **Sites
d'`api-external.md` à toucher, nommés** *(aucun test ne lit ce fichier : rien ne rougira s'il en manque
un)* :

| site (au 2026-10-09, `5e4bec50`) | geste |
|---|---|
| `:212` (matrice des ressources du § 7) | une ligne « Lettrages » : `GET /letterings/{key}` en lecture, `POST /letterings`, `DELETE /letterings/{key}` en écriture |
| la réponse d'une écriture (section des écritures, avant `:223`) | les trois champs neufs des lignes, `letteringKey`, `letteringCode`, `letteringOrigin` (nuls si la ligne est ouverte, AC14) |
| `:325`, `:386` | « à lettrer » retiré (ci-dessous) |
| `:485` (catalogue des codes) | les dix codes `LETTERING_*` de T10 |
| `:488` (routes rejouées, « Ouvertes aux clés `read-write`, ce sont : … ») | `POST /letterings` et `DELETE /letterings/{key}` |

**CHANGELOG `[0.13.0]`** *(réécrit en validation P3 — L8, F3-2, C127)* : la section *Ajouté* **existe
déjà** sous `## [0.13.0] — Non publié` (`CHANGELOG.md:13`, entrée #429, au 2026-10-09) — la
**compléter** (la créer, en tête, seulement si elle a disparu) avec l'entrée « le lettrage, par l'API »
(l'écran viendra en 15-1c, qui complétera l'entrée), **et** l'avertissement de non-retour (précédent
v0.10.0, commit `a80a36c1`) : la base porte `kesh_version_min_required = 0.13.0`, un binaire antérieur
refuse de démarrer contre elle et refuse d'importer une sauvegarde qu'elle a produite — faire une
sauvegarde **avant** la mise à jour pour pouvoir revenir. Sous ***Modifié*** (patron C123 : un
changement de contrat d'API se cherche là), une entrée « ⚠️ Changement de contrat pour une intégration
par clé d'API » : les lignes d'écriture portent trois champs neufs (`letteringKey`, `letteringCode`,
`letteringOrigin`) dans toutes les réponses qui les exposent, et l'export CSV des lignes (export global
compris) gagne deux colonnes, `lettering_key` et `lettering_origin`. Les deux autres changements de
contrat (`ENTRY_LETTERED`, réponse `201` de la contre-passation) sont annoncés par la 15-1a-ii.

**Le verrou de période fige aussi le lettrage** *(C113)* : `user-manual.tex:578-583`
(§ « Arrêter les livres à une date ») dit qu'aucune écriture de la période ne peut plus
« disparaître ni être modifiée ». Il reçoit la phrase symétrique : « Le **lettrage** des lignes de
la période se fige avec elles : un groupe dont toutes les lignes sont dans la période verrouillée
(ou dans des exercices clôturés) ne se lettre ni ne se délettre. » — et `api-external.md` dit la
même règle dans le tableau des refus des routes de lettrage
(`LETTERING_ALL_LINES_IN_CLOSED_PERIODS`).

**Le vocabulaire** *(F-15)* : le glossaire (`user-manual.tex:2323` au 2026-10-09, `5e4bec50` ; `:2308`
jusqu'en P3) définit « Lettrage » comme « rapprochement d'une facture avec son paiement » — trop étroit
pour un groupe à somme nulle sur tout compte d'actif ou de passif. Il est réécrit **dans ce commit** :
« Marque posée sur des lignes d'un même compte d'actif ou de passif dont la somme est nulle, pour dire
qu'elles se soldent entre elles (une facture et ses règlements, une écriture et sa contre-passation…).
Dans cette version, le lettrage manuel et le délettrage se font par l'API (`/api/v1/letterings`) ;
l'écran viendra. » — la dernière phrase est retirée par la 15-1c (« Reçu de la 15-1a » de sa fiche).

**Sites qui promettent un lettrage manuel interdit par R5** *(C106 ; inventaire complété en
validation P4 — R4-1, C130 : C106 ne nommait que le manuel et l'API, alors que l'écran dit la même
chose)*. Relevé **par la valeur**, dans les quatre langues, au 2026-10-09 (`5e4bec50`) :
`git grep -nE "à lettrer|noch zuzuordnen|zuzuordnende|to be matched|da abbinare" origin/main -- crates
frontend docs website CHANGELOG.md README.md ':(exclude)*.pdf'`. Tous réécrits **dans cette story**,
sans promettre de lettrage à la main — le traitement du cas est celui de la 15-1a2 :

| site | geste |
|---|---|
| `crates/kesh-i18n/locales/fr-CH/messages.ftl:783` (`invoices-settlement-cancel-blocked-credited`) et `:818` (`reconciliation-cancel-blocked-credited`) ; les mêmes clés en `de-CH`, `en-CH`, `it-CH` (`:735`, `:770`) | texte de remplacement ci-dessous, **quatre** locales |
| replis Rust `crates/kesh-api/src/errors.rs:2966` (annulation de règlement) et `:3495` (dé-rapprochement) | identiques au catalogue FR |
| replis frontend `frontend/src/lib/features/invoices/settlement-cancel.ts:29`, `frontend/src/lib/features/reconciliation/reconciliation-cancel.ts:50` | identiques au catalogue FR |
| tests frontend qui assertent le texte : `InvoiceSettlements.test.ts:78`, `settlement-cancel.test.ts:22`, `reconciliation-cancel.test.ts:23` (fragment `'paiement à lettrer'` → `'reste ouvert au compte débiteurs'`) ; `frontend/src/lib/shared/utils/settlement-cancel-blocked.test.ts:47` (titre) et `:49` (texte entier) | réécrits ; ils restent les gardes du texte |
| doc-comment de `SettlementCancelBlocker::InvoiceCredited`, `crates/kesh-db/src/errors.rs:327` (« un paiement **à lettrer** (Epic 15) ») | « reste ouvert au compte débiteurs ; il ne se lettre pas à la main (R5 de la 15-1a-i), son traitement est la 15-1a2 » |
| `docs/manual/fr/user-manual.tex:1171` et `:1711` (`:1169` et `:1696` jusqu'en P3) (« paiement \emph{à lettrer} ») | « le règlement reste ouvert au compte débiteurs --- il ne s'annule pas » |
| `docs/api-external.md:325` et `:386` (refus `INVOICE_CREDITED`, « paiement **à lettrer** ») | « le règlement reste ouvert au compte débiteurs » |

Texte de remplacement, arrêté ici (une seule formulation par clé, le repli recopiant le FR) :

| clé | fr-CH | de-CH | en-CH | it-CH |
|---|---|---|---|---|
| `invoices-settlement-cancel-blocked-credited` | Cette facture a été créditée par un avoir : ce règlement reste ouvert au compte débiteurs, il ne s'annule pas. | Diese Rechnung wurde durch eine Gutschrift ausgeglichen: Diese Zahlung bleibt auf dem Debitorenkonto offen, sie kann nicht storniert werden. | This invoice has been credited by a credit note: this settlement stays open on the receivables account, it cannot be cancelled. | Questa fattura è stata stornata da una nota di credito: questo pagamento resta aperto sul conto debitori, non si annulla. |
| `reconciliation-cancel-blocked-credited` | La facture de ce rapprochement a été créditée par un avoir : son règlement reste ouvert au compte débiteurs, il ne s'annule pas. | Die Rechnung dieses Abgleichs wurde durch eine Gutschrift storniert: Ihre Zahlung bleibt auf dem Debitorenkonto offen und wird nicht storniert. | The invoice of this reconciliation has been credited by a credit note: its settlement stays open on the receivables account, it is not cancelled. | La fattura di questa riconciliazione è stata stornata da una nota di credito: il suo pagamento resta aperto sul conto debitori e non si annulla. |

⚠️ **Trié, non touché** : `CHANGELOG.md:74` (« le règlement est alors un paiement à lettrer ») est sous
`## [0.12.1] — 2026-10-07`, **publié** — des notes de version ne se réécrivent pas ; le changement de
texte est annoncé sous *Modifié* de `[0.13.0]` (ci-dessous). `supplier-invoices-cancel-confirm-paid`
(`de-CH:1918`, `en-CH:1924`, `it-CH:1919` « to be matched » ; `fr-CH:1945` « à rattacher ») parle du
**paiement fournisseur détaché**, qui **reste** lettrable à la main (R5, dernier alinéa) : promesse vraie.
L'en-tête de `20260827000001_invoice_settlements.sql:16` est une migration (P8) et ne promet rien.

**CHANGELOG `[0.13.0]`, *Modifié*** : une ligne — « Le motif d'une annulation refusée sur une facture
créditée (règlement, rapprochement) ne promet plus un lettrage : le règlement reste ouvert au compte
débiteurs. »

Toucher le `.tex` impose de **régénérer et versionner** le PDF (`make fr` dans `docs/manual/`), puis
de le contrôler **aplati** (`pdftotext … - | tr '\n' ' ' | tr -s ' '`) : zéro « à lettrer », la phrase
du verrou de période et le glossaire présents. ⚠️ **Et le contrôle porte aussi hors du PDF** *(R4-1)* :
le `git grep` du relevé ci-dessus, relancé au dernier commit, ne rend plus que les sites triés —
**cinq lignes** : `CHANGELOG.md:74`, `supplier-invoices-cancel-confirm-paid` en `de-CH`, `en-CH` et
`it-CH`, et la migration `20260827000001:16`. Les deux clés réécrites existent déjà : aucun compte figé
des gardes i18n ne bouge pour elles.

## Tasks

- [x] **T0 (part i)** (R7) — Relevés au sol sur le `main` du moment — la migration de T1 appliquée
      (index `idx_jel_lettering`), **avant** T3 : `EXPLAIN` de l'acte 1 (création et dissolution) ;
      résultats au Dev Agent Record. Constater que la 15-12a est mergée (prérequis : `grep -n
      "LOCK_EARLIER_BY_ID_SQL\|EarlierFiscalYearOpen" crates/kesh-db/src/repositories/fiscal_years.rs`),
      et si la 15-12b l'est. *(Plus d'`EXPLAIN` des exercices : un verrou par clé primaire n'en dépend
      pas — C125.)* **Constat des verrous d'intervalle** *(F4-1)* : une dissolution du groupe de plus
      grande clé tenue ouverte dans une transaction, puis une création concurrente sur d'autres lignes —
      son `UPDATE` est-il vu en attente (`attendre_une_requete_en_cours`, motif `["SET lettering_key",
      "lettering_key IS NULL"]`) ? Résultat au Dev Agent Record, et R7 corrigé s'il dément le
      raisonnement. *(Le code d'erreur de la sonde `NOWAIT` n'est **pas** à relever : 1205, déjà mesuré
      sous MariaDB 10.11 par la Story 15-5d — `test_fixtures::sonde_verrou_nowait`, F4-5.)*
- [x] **T1** (AC1) — Migration R1 avec le bump `min_required = '0.13.0'` en dernière instruction ;
      **dans le même commit** les dix `Cargo.toml` à `0.13.0` (+ `Cargo.lock`) ; entrée
      `EXEMPT_MIGRATIONS` (`Durable`) ; squash régénéré ; `migrations.sha384` ; ligne d'audit
      d'idempotence et compteurs recomptés — **et la décomposition par story de la ligne `Total`**
      (`docs/migrations-idempotence-audit.md:101`, « 75 migrations (26 historiques + … + 1 Story
      25-6-b) ») gagne « + 1 Story 15-1a-i », sans quoi sa parenthèse ne somme plus au total (F4-7) ;
      inspection P6 ; `migrations_fresh_install.rs:230/244`
      à `0.13.0` ; `migrations_upgrade_path.rs:509` (binaire `0.13.0` → `Aligned`) et `:524`
      (`db_min` `0.13.0`, binaire `0.14.0` → `BinaryAhead`) et leurs commentaires `:510`,
      `:525-526` ; en-tête `-- BREAKING (P1) :` recopié de R1.
- [x] **T2** (AC2, R3, R7) — `kesh-core::lettering` : `code_from_key`, `key_from_code` (arithmétique
      vérifiée, borne `i64::MAX`, `to_ascii_uppercase` puis validation `A-Z` — F4-2), **une fonction
      pure par cause** (`check_line_ids`, `check_same_account`, `check_none_lettered`,
      `check_balanced`), et `check_rows_affected(expected, actual)` (R7 point 4 — F4-4), tests
      unitaires.
- [x] **T3** (AC3, AC4, AC5, R4, R7) — `kesh-db/src/repositories/letterings.rs` :
      `create_group_in_tx`, `dissolve_group_in_tx`, `find_group`, `is_letterable_account` (tout
      `bank_accounts` qui désigne le compte, **archivé compris** — C127), `enum Mode { Manual,
      System { held_open_fiscal_year_id } }`, erreurs `DbError::Lettering*` (dont
      `LetteringConcurrentChange` et `LetteringAllLinesInClosedPeriods`). Séquence de R7 (premier acte
      verrouillant avec `je.entry_date` ; en `Manual`, exercices du groupe lus sans verrou triés par
      `start_date`, puis verrouillés **un par un** par `LOCK_LETTERING_FISCAL_YEAR_SQL` ; « postérieur
      clos » par `find_later_closed` ; règle des périodes (i)-(iii) ; exercice tenu en `System`, et
      **noms** des exercices lus sans verrou par `LETTERING_FISCAL_YEAR_NAMES_SQL` — C128 ; compte
      des lignes trouvées par `check_rows_affected`)
      **documentée dans le doc-comment**, étape par étape, avec la phrase « ces requêtes sont reconnues
      à leur texte par les tests » et leurs motifs (R7 point 2).
- [x] **T6** (AC6, AC11, AC12) — Routes `kesh-api/src/routes/letterings.rs` (`create_lettering`,
      `get_lettering`, `delete_lettering`), enveloppe de rejeu, registres, ligne du Pattern 5.
      ⚠️ Montage dans `kesh-api/src/lib.rs` *(axe signalé non exercé par la lentille F de P2)* :
      `POST`/`DELETE` dans `comptable_routes` (`:347`), **avant** son `route_layer` (`:723`), `GET`
      dans `authenticated_routes` (`:728`) — `route_layer` n'enveloppe que les routes déjà
      enregistrées (avertissement écrit au bloc admin, `:321-332`) : une route chaînée après
      échapperait au RBAC. Les tests 403 d'AC6 (Consultation au `POST` et au `DELETE`) le vérifient.
- [x] **T7** (AC10) — Audit par `for_actor` + labels + 4 locales ; lignes des `details` avec
      `fiscalYearId`/`fiscalYearName` (C127).
- [x] **T8** (AC13, AC14) — Struct, `LINE_COLUMNS` et liste de `list_all_lines_by_company`, les deux
      littéraux `JournalEntryLine {` de `kesh-api`, DTO, type frontend `journal-entries.types.ts:11`
      (champs requis et nullables) **et ses fixtures typées** (`JournalEntryForm.edit.test.ts:66-67`,
      `form-helpers.test.ts:29`, `:38-44`, `:50-56`, `:62-68` — AC14, R4-3 = F4-3), export CSV ; `COLONNES_DES_LIGNES` (+ commentaire renvoyant au gel `Lettered` de la 15-1a-ii, AC8),
      vérification du test de mutation `:197/:220`, doc-comment `journal_entries_modification.rs:181`
      (`lettering_id` → `lettering_key`).
- [x] **T9** (R3) — Test structurel, sur le patron des tests lexicaux du dépôt : dans
      `crates/*/src/**/*.rs`, **hors** blocs `#[cfg(test)]` (les tests de dépôt vivent aussi dans des
      `mod tests` de `src/`) et hors `kesh-db/src/repositories/letterings.rs`, aucun statement
      `UPDATE` **ni** `INSERT` (toutes formes : `VALUES`, `SELECT`) qui nomme `lettering_key` ou
      `lettering_origin`. Les exceptions de R3 n'y apparaissent pas (migration en `.sql` ; la
      restauration nomme ses colonnes dynamiquement) — le dire au doc-comment du test. **Et** dans
      `letterings.rs`, chacune des deux primitives appelle `check_rows_affected` (R7 point 4 — F4-4 :
      c'est la seule garde contre la mutation « retirer la vérification »).
- [x] **T10 (part i)** (AC3, AC4, AC5, R7) — i18n, **dix** clés dans les **quatre** locales, chacune
      avec son repli FR identique au catalogue (le 404 d'AC11 n'a **pas** de message dédié) :

      | code | clé | texte FR |
      |---|---|---|
      | `LETTERING_TOO_FEW_LINES` | `error-lettering-too-few-lines` | Un lettrage réunit au moins deux lignes distinctes. |
      | `LETTERING_TOO_MANY_LINES` | `error-lettering-too-many-lines` | Un lettrage réunit au plus { $max } lignes. *(plafond passé en variable — revue P1, E-4)* |
      | `LETTERING_ACCOUNTS_DIFFER` | `error-lettering-accounts-differ` | Les lignes d'un lettrage doivent toutes porter sur le même compte. |
      | `LETTERING_ACCOUNT_NOT_LETTERABLE` | `error-lettering-account-not-letterable` | Ce compte ne se lettre pas : seuls les comptes d'actif et de passif qui ne sont pas des comptes bancaires se lettrent. |
      | `LETTERING_LINE_OWNED_BY_DOCUMENT` | `error-lettering-line-owned-by-document` | Une de ces lignes appartient à une pièce : elle ne se lettre ni ne se délettre à la main. |
      | `LETTERING_LINE_ALREADY_LETTERED` | `error-lettering-line-already-lettered` | Une de ces lignes est déjà lettrée (code { $code }). |
      | `LETTERING_UNBALANCED` | `error-lettering-unbalanced` | Ces lignes ne se soldent pas : écart de { $difference }. |
      | `LETTERING_ALL_LINES_IN_CLOSED_PERIODS` | `error-lettering-all-lines-in-closed-periods` | Toutes ces lignes sont dans une période close — exercice clôturé, exercice suivi d'un exercice clôturé, ou période verrouillée : le lettrage n'y change plus. |
      | `LETTERING_IS_DOCUMENT` | `error-lettering-is-document` | Ce lettrage est celui d'une pièce : annulez le règlement plutôt que de délettrer. |
      | `LETTERING_CONCURRENT_CHANGE` | `error-lettering-concurrent-change` | Le lettrage a changé entre-temps ; réessayez. |

      Ventilation, recomptée : AC3 **6** (causes 1, 3, 4, 5, 6, 7), plus `LETTERING_TOO_MANY_LINES`
      (AC6) **1**, `LETTERING_ALL_LINES_IN_CLOSED_PERIODS` (AC4 et AC5, une seule clé) **1**,
      `LETTERING_IS_DOCUMENT` (AC5) **1**, `LETTERING_CONCURRENT_CHANGE` (R7) **1** → **10**. La
      onzième clé de la 15-1a, `ENTRY_LETTERED` (`journal-entries-modify-blocked-lettered`, AC8), est
      dans la 15-1a-ii. Le refus 2 d'AC5 réutilise la clé de la cause 5 d'AC3 — d'où son texte
      **neutre** *(validation P2 — R2-9)*. ⚠️ **Deuxième lot, hors de ces dix** *(R2-8)* : les libellés
      d'audit de T7 — une entité (`lettering`) et deux actions (`lettering.created`,
      `lettering.removed`), préfixes `entity`/`action` d'`audit_labels.rs` (`ENTITY_TYPES` `:24`,
      `ACTIONS` `:65`), × 4 locales. Les gardes `i18n-keys.test.ts` / `i18n-un-repli-par-cle.test.ts`
      et `audit_label_registry.rs` voient leurs comptes figés **recomptés pour les deux lots**, chacun
      depuis sa source. ⚠️ **Hors des deux lots, deux clés existantes réécrites** (R5, C130) :
      `invoices-settlement-cancel-blocked-credited` et `reconciliation-cancel-blocked-credited`, quatre
      locales, replis Rust et frontend — texte arrêté à AC15 part i ; aucun compte figé n'en bouge.
- [x] **T11 (part i)** (AC15 part i) — `api-external.md` (routes, refus, et **tous** les sites du
      tableau d'AC15 part i : `:212`, réponse d'une écriture, `:325`, `:386`, `:485`, `:488`),
      CHANGELOG (*Ajouté* **complétée** + avertissement de non-retour ; *Modifié* : champs et colonnes
      neufs ; le motif « facture créditée » qui ne promet plus de lettrage), manuel —
      `user-manual.tex:578-583` (verrou de période), glossaire `:2323`, `:1171`, `:1711` — et PDF
      régénéré puis contrôlé aplati, `MULTI-TENANT-SCOPING-PATTERNS.md` ; **et tout l'inventaire
      « à lettrer » d'AC15 part i** (i18n quatre locales, replis Rust et frontend, quatre tests
      frontend, doc-comment `kesh-db/src/errors.rs:327`), contrôlé par son `git grep` (cinq lignes
      triées restantes) — R4-1, C130.
- [x] **T12 (part i)** — Tests :
      - AC1 : `migrations_fresh_install` à `0.13.0`, `downgrade_protection_aligned_when_binary_equals_min`
        et `downgrade_protection_binary_ahead_when_binary_greater` **mis à jour**,
        `downgrade_protection_rejects_old_binary` (non-régression), `every_data_backfill_migration_is_triaged`
        — exécutés au gate runtime complet ;
      - AC3 : une par cause, dans l'ordre (une requête cumulant deux causes rend la **première**),
        dont le rang 4 bis ;
      - AC2 : débordement (vingt `Z`, `i64::MAX` et son successeur) ;
      - R4 : `archived_bank_account_keeps_its_account_unletterable` (un `bank_accounts` archivé désigne
        le compte → `LETTERING_ACCOUNT_NOT_LETTERABLE`) *(C127)* ;
      - AC4 : les six tests nommés (exercices, verrou de période et sa borne, état hérité) — le dernier,
        `lettering_under_a_later_closed_year_is_refused`, passe par la lecture **non verrouillante** du
        (ii) (C125) ;
      - AC5 : les quatre branches (dont les trois tests du point 3) ; dissolution d'un groupe dont le
        compte a été retypé (C104) ; et, **séparés** *(R3-3 de P3 — la version précédente fondait en un
        seul test un scénario d'ordre qui ne discriminait pas la mutation et un scénario de refus
        indéterminé)* :
        - **sérialisation avec la clôture** — groupe tout dans A (ouvert, **sans** exercice antérieur) ;
          la dissolution, dans une transaction tenue ouverte, a passé R7 point 2 ; `fiscal_years::close(A)`
          lancé **attend** (`attendre_une_requete_en_cours`, motifs de l'étape (c) de la clôture de la
          15-12a, qui verrouille Y par clé primaire : `["SELECT id, company_id", "WHERE id = ", "FOR
          UPDATE"]` — `15-12a-cloture-dans-l-ordre.md:356` ; sans antérieur, l'étape (b') ne prend rien et
          c'est (c) qui bute — R4-5) ; la dissolution valide ; la clôture aboutit. (Le cas
          « clôture validée avant → refus » est `dissolution_all_in_closed_years_is_refused`.) Aucun
          `id` inversé : ce test ne prétend rien de l'ordre ;
        - **ordre d'acquisition** — `lettering_locks_fiscal_years_in_date_order_not_id_order` : A
          (antérieur, `id` **haut**, créé après coup) et B (postérieur, `id` bas), une ligne ouverte dans
          chacun sur le même compte, groupe `manual` {a, b} ; une transaction T tient A (`SELECT id FROM
          fiscal_years WHERE id = A FOR UPDATE`) ; la dissolution — appel **direct** de
          `dissolve_group_in_tx` dans sa propre transaction, **hors** enveloppe de rejeu — se lance et
          bute (`attendre_une_requete_en_cours`, motifs `["SELECT id, start_date, status FROM
          fiscal_years WHERE id = ", "FOR UPDATE"]`) — ⚠️ **observée deux fois, à 100 ms d'écart**
          *(validation P4 — R4-5)* : sous la mutation, la requête qui verrouille B (sans attente, moins
          d'une milliseconde) porte le même motif que celle qui bute sur A ; une seule observation
          tombée dans sa fenêtre sonderait B **avant** qu'il soit pris, et la mutation survivrait. Une
          requête encore en cours 100 ms plus tard est celle qui attend. Puis une troisième connexion
          sonde B par **`kesh_db::test_fixtures::sonde_verrou_nowait(pool, "SELECT id FROM fiscal_years
          WHERE id = ? FOR UPDATE NOWAIT", B)`** (helper existant, Story 15-5d — DRY, F4-5) : `true`
          attendu — la dissolution n'a rien pris au-delà de A —, puis `ROLLBACK` de T ; la dissolution
          aboutit. **Mutation tuée** : « trier par `id` au lieu de `start_date` » — la dissolution prend
          B puis bute sur A, la sonde rend `false` sur-le-champ (`1205`, mesuré par la 15-5d sous
          MariaDB 10.11 ; toute autre erreur fait paniquer le helper, si bien qu'une sonde qui échoue
          pour une autre raison ne passe pas pour un verrou). ⚠️ Pourquoi pas face à `close` : sous la 15-12a, l'entrelacement qui révèle l'ordre
          passe par la clôture, rejouée (son AC 6), qui absorberait l'interblocage — le test resterait
          vert sous la mutation ;
      - AC6 : `POST` → `201` et sa forme (dont `fiscalYearId`/`fiscalYearName` par ligne — C127) ;
        `GET` par clé et par code (`27` et `AA`), clé invalide (`0`, `A1`, `Ä`, `ß` — F4-2 ; le
        cas `ß` ≠ `SS` est prouvé sans base à AC2 —, `99999999999999999999`, vingt `Z`) → 404 ; 201 lignes → 400 `LETTERING_TOO_MANY_LINES` ; rôle
        Consultation → 403 au `POST` et au `DELETE`, 200 au `GET` ;
      - R7 point 4 *(réécrit en validation P4 — F4-4, C131 : ni crochet de production, ni déclencheur
        SQL, qui ne change pas le compte des lignes **trouvées**)* : `check_rows_affected` testée sans
        base — `actual < expected` et `actual > expected` → `LetteringConcurrentChange`, égalité →
        `Ok` ; **mutation tuée** : retirer l'appel dans `create_group_in_tx` ou `dissolve_group_in_tx`
        se voit au test lexical que T9 étend (chaque primitive appelle `check_rows_affected` sur le
        résultat de son `UPDATE`) ; le **409** par le test de correspondance des erreurs de
        `kesh-api/src/errors.rs` (patron des bras existants : `DbError::LetteringConcurrentChange` →
        `409 LETTERING_CONCURRENT_CHANGE`, clé et repli). Le chemin de bout en bout reste un **angle
        mort assumé** : il n'est atteignable que par un défaut ;
      - R7 point 3 : mode `System` avec un exercice tenu qui ne couvre aucune ligne → `Invariant` ;
        mode `System` → la réponse et l'audit portent `fiscalYearName` par ligne (C128 ; le test de
        bout en bout est `reversal_lettering_is_audited_by_the_reverser`, 15-1a-ii — ici, un appel
        direct de `create_group_in_tx` en mode `System`) ;
      - AC2 / R2 : `ß`, `ſ`, `ı` → `None` ; `aa` → `Some(27)` (F4-2) ;
      - AC10 : `lettering_created_is_audited_with_its_lines`, `lettering_removed_is_audited` ;
      - AC11 ; AC13 (`lettering_invariants`, sur un scénario de groupes `manual` — la 15-1a-ii y
        ajoute des groupes `reversal` ; mutation `:197` toujours rouge) ;
      - AC14 : les deux tests nommés ;
      - **concurrence** : deux `POST` simultanés partageant une ligne → exactement **un** 201 et un
        409 `LETTERING_LINE_ALREADY_LETTERED`, **jamais** un 500 (le test ne s'affaiblit pas sur un
        interblocage : le rejeu doit l'absorber). Motifs de T12 : ceux de R7 point 2, jamais
        `["je.fiscal_year_id", "FOR UPDATE"]` (F3-6).

## Dev Notes

- **Gate `kesh-db` : complet, jamais ciblé** (migration + repository, P6/P7) — et **gate runtime
  complet** (P2-bis : bump `min_required` + bump Cargo). Base remise à zéro **avant** le gate
  (KF-039) — par `DROP/CREATE DATABASE` de **ses** bases, jamais par redémarrage du conteneur
  (consignes de l'Epic 15, point 8). E2E complet au dernier commit de code (D7) : le type frontend des
  lignes est touché (AC14).
- **P8** : une fois appliquée, la migration ne se modifie plus, pas même un commentaire.
- **Dépendances** *(motif réécrit en validation P3 — L7)* : ⛔ **la 15-12a (clôture dans l'ordre, C89)
  passe AVANT** *(C105, C112)*. Ce qui en dépend encore — et non « la règle des périodes », qui **garde**
  elle-même l'état hérité par son (ii) (C113) : (1) la **lecture sans verrou** du (ii) (R7 point 2 (c),
  argument (α)) et la **dispense de verrouiller les postérieurs** (C125) ; (2) la clôture **rejouée**
  (15-12a AC 6), que R7 points 2 et 3 prennent en compte ; (3) la vue « au » d'une date de la 15-1b, qui
  repose sur « les clos forment un préfixe ». La **15-12b** (le filet des données héritées) n'est pas
  requise — son filet couvre la création et la suppression d'écritures, pas le lettrage — mais passe
  avant **de préférence** : elle ferme le (ii) en mode `System` (R7) et touche `delete_in_tx`, que la
  15-1a-ii touche aussi (C117). Également : 15-8a et 15-8b (mergées — `modification_guard`,
  `delete_in_tx`), 15-5e1/e2 (mergées — enveloppes). ⚠️ **15-6a** (l'avoir crédite le compte de la
  vente) n'est pas requise ici ; elle l'est pour la 15-1a2. **En aval** : la 15-1a-ii suppose cette
  story mergée, et la v0.13.0 ne se tague pas entre les deux (C124).
- **Règle de découpage** : cette fiche **est** le produit d'un découpage (C124, couture de C118). Modules
  touchés, au grain « crates Rust, packages npm » : `kesh-core`, `kesh-db`, `kesh-api`, `kesh-i18n`,
  `frontend` (le seul type des lignes, AC14) = **5** — seuil (« plus de 5 ») non franchi. Au grain des
  modules métier : `kesh-db/migrations`, `repositories/letterings`, `repositories/journal_entries`
  (listes de colonnes seulement), `routes/letterings`, `routes/journal_entries` (DTO seulement),
  `exports`, `audit_labels`, `kesh-i18n`, `kesh-core/lettering`, `features/journal-entries` (type) — une
  dizaine, dont **cinq mécaniques** (listes de colonnes, DTO, type, export, libellés). Le cœur est la
  primitive et ses routes. Signal **déclaré** au Project Lead ; un second découpage séparerait la
  primitive de ses routes, et la primitive ne s'éprouve qu'à travers elles (AC11, AC12).
- **Manuel d'administration et README : rien ici, par décision** *(validation P4 — F4-8)*. Le
  non-retour du bump est dit au CHANGELOG (AC15 part i) ; `admin-manual.tex:1671` et `:1754`
  (« Toujours sauvegarder avant la mise à jour. Une migration DB peut être irréversible ») le disent
  déjà en général ; le manuel d'administration se relit à la release (CLAUDE.md, liste pré-release
  n° 4), et la ligne `README.md:222` (« v0.13.0 (prévu) ») change à la clôture de l'epic. Les sites
  d'`admin-manual.tex` et de `README.md` qui promettent « modifiable tant que l'exercice est ouvert »
  sont, eux, à la 15-1a-ii (F-2 de sa P4).
- **FR86** (`prd.md:518`, « tant que l'exercice est ouvert ») est **interprété** par C94/C105/C113 :
  délettrer exige **au moins une** ligne en période ouverte (R7), ce qui permet de délettrer un
  groupe à cheval — et l'interdit sous un verrou de période, que FR86 ne connaissait pas. Lecture
  défendable (FR86 ne pensait qu'un exercice), consignée ici ; un CR n'est ouvert que si le Project
  Lead la conteste.
- Références : `journal_entries.rs` (`:80`, `:308-313`, `:492`, `:708`, `:918`, `:1009`, `:1442`,
  `:1520`, `:1755`, `:1824`, `:1828`, `:1922`, `:2094`), `reversal_blockers` et ses codes
  (`kesh-db/src/errors.rs:57`, `:104-111`), `journal_entries_modification.rs:42/78/90/181/189/197/410/466/511`,
  `audit_route_registry.rs:168/199/203/205/255/638/909`, `kesh-api/src/admin_backup/import.rs:118-136/195-238`,
  `post_restore.rs:512-525`, `fiscal_years.rs:400/528-531/550-555/646-671/668-671/678/773`,
  `opening_complement.rs:278-281`, `migrations_upgrade_path.rs:484/509/524`,
  `kesh-api/src/lib.rs:321/347/723/728`. ⚠️ **Tous les numéros se vérifient sur `origin/main`**
  (`5e4bec50` au 2026-10-09 ; ceux de la P2 dataient de `8f9811d8` — `journal_entries.rs` et
  `fiscal_years.rs` n'ont pas bougé entre les deux, les autres sites ont été recalculés), non sur la
  branche de planification, en retard — partir d'un `main` à jour.

## Dev Agent Record

### Agent Model Used

Opus 5.5 (agent de développement), en deux sessions : la première (T0, `bc32f92a` ; implémentation,
`cbaf5a18`) interrompue par un crash de la station au moment du gate ; la seconde (reprise) a intégré
la correction restée non commitée, achevé le contrôle d'AC15 et exécuté les gates.

### Completion Notes List

- **Toutes les tâches de la part (i) sont livrées** (T0, T1, T2, T3, T6–T12) ; R6, AC8, AC9 et la part (ii)
  d'AC15 restent à la 15-1a-ii. Choix consignés au registre : **C-15-1a-i-1** (verrou d'intervalle « plus
  grande clé » démenti par la mesure, R7 corrigé), **-2** (plafond de 200 lignes = refus de forme de la
  route), **-3** (l'acte 1 lit `je.entry_number` ; rang 4 lu après les verrous d'exercice), **-4**
  (vocabulaire DE/EN/IT du lettrage), **-5** (clés `15-13*` au registre de sprint).
- **Reprise après crash** : la session précédente avait laissé non commitée la mise à jour du compte figé
  `EXEMPT_MIGRATIONS.len()` 15 → 16 (`post_restore.rs`). Vérifiée contre T1 (une entrée `Durable` pour
  `20261009000001`, **16** entrées recomptées depuis la source) et commitée (`74d164ca`).
- **Contrôle d'AC15 part i relancé au dernier commit** : le `git grep` du relevé rendait **six** lignes —
  les cinq triées plus un doc-comment neuf de la story (`routes/letterings.rs:45`, « lignes … à lettrer
  ensemble », sans promesse mais hors inventaire). Reformulé (`ee6cb1c3`) : le contrôle rend désormais les
  **cinq** lignes triées (`CHANGELOG.md:96` sous `[0.12.1]`, la migration `20260827000001:16`,
  `supplier-invoices-cancel-confirm-paid` en `de-CH`, `en-CH`, `it-CH`). PDF du manuel contrôlé aplati :
  zéro « à lettrer », la phrase du verrou de période (« se fige avec elles ») et le glossaire
  (`/api/v1/letterings`) présents.
- **Comptes figés des gardes i18n et d'audit** (T10) : `i18n-keys.test.ts` (sites `i18nMsg` du frontend),
  `i18n-un-repli-par-cle.test.ts` (replis frontend) et `audit_label_registry.rs` ne bougent pas — les dix
  clés d'erreur et les trois libellés d'audit n'ont **aucun site frontend** ; recomptés par l'exécution des
  gardes, verts.
- **Mutations rejouées dans la reprise** (chacune restaurée, puis `letterings` + `letterings_lexical`
  rejoués verts, 34/34) : « trier les exercices du groupe par `id` au lieu de `start_date` »
  (`FISCAL_YEARS_OF_GROUP_SQL`) → `lettering_locks_fiscal_years_in_date_order_not_id_order` **rouge** sur
  la sonde `NOWAIT` de B (`letterings.rs:1231`) ; « retirer `check_rows_affected` de
  `dissolve_group_in_tx` », puis de `create_group_in_tx` → `each_primitive_checks_the_rows_its_update_found`
  **rouge** les deux fois. Les autres mutations citées par la fiche (`:197` de
  `journal_entries_modification.rs`) n'ont **pas** été rejouées dans cette session.
- **Tests ajoutés**, recomptés depuis la source, périmètre `dc4bc58b..ee6cb1c3` : `kesh-db/tests/letterings.rs`
  **31**, `kesh-db/tests/letterings_lexical.rs` **3**, `kesh-api/tests/letterings_e2e.rs` **7**,
  `kesh-core/src/lettering.rs` **12**, `kesh-api/src/errors.rs` 26 → 28 (**+2**),
  `kesh-api/src/exports/csv_tables.rs` 16 → 17 (**+1**), `kesh-api/src/routes/journal_entries.rs` 6 → 7
  (**+1**, `journal_entry_line_response_exposes_lettering`), `kesh-db/src/repositories/letterings.rs`
  0 → 3 (**+3**, `mod tests`) — **60** *(le compte rendu d'origine écrivait 56 : les deux derniers
  fichiers manquaient à la ventilation — revue P1, A-L1 ; recompté :
  `git diff dc4bc58b..ee6cb1c3 -- '*.rs' | grep -E '^\+.*#\[(sqlx::test|test|tokio::test)' | wc -l` → 60)* ;
  tests existants mis à jour sans
  changement de nombre : `migrations_upgrade_path.rs`, `migrations_fresh_install.rs`,
  `journal_entries_modification.rs`, `audit_route_registry.rs`, et cinq fichiers de test frontend.
- **AC14 — les vérifications « par leurs gardes » faites, et écrites** *(revue P1, A-L4 = E-1, C-15-1a-i-8)* :
  `is_no_op_change` (`journal_entries.rs:1179`) compare date, journal, libellé puis, ligne à ligne, compte,
  débit, crédit et projet ; `entry_snapshot_json` (`:992`) construit les lignes champ par champ (`lineOrder`,
  `accountId`, `debit`, `credit`, `projectId`). **Ni l'un ni l'autre n'est faussé** par les deux colonnes,
  qu'ils ignorent : la marque n'est pas un contenu de l'écriture. Ce qu'ils ne voient pas — un `PUT`
  identique sur une écriture lettrée serait un no-op, un `PUT` réel détruirait la marque sans la tracer —
  est fermé par la garde de la 15-1a-ii (AC8, étape 7-bis, **avant** l'instantané et le no-op). La
  **sauvegarde** est désormais prouvée par un aller-retour (`full_import_round_trip_keeps_lettering_marks`),
  et non plus supposée de la lecture dynamique des colonnes.
- **Écart connu, écrit au T0, non corrigé** : sur tables **vides**, le plan de l'acte 1 de la création à
  40 identifiants passe par `idx_journal_entries_company_date` (sur-verrouillage des seules bases
  minuscules, sans effet sur l'exactitude : routes rejouées).

**Gates du développement, au dernier commit de code `ee6cb1c3` — sauf `npm run check` et `npm run
test:unit`, rejoués sur `74d164ca`** (revue P1, A-L5 ; bases `kesh_151ai` et `kesh_e2e_151ai` reconstruites par
`DROP/CREATE` + migrations du worktree + `scripts/seed-dev-db.sql` immédiatement avant) :

| gate | résultat |
|---|---|
| `scripts/test-fast.sh --ci` (fmt, clippy `-D warnings`, nextest) | vert — **3086** exécutés, **3086** réussis, 4 ignorés (115,7 s) |
| `npm run check` | 0 erreur, 27 avertissements (préexistants, aucun dans un fichier de la story) |
| `npm run lint-i18n-ownership` | PASS |
| `npm run test:unit` | vert — 114 fichiers, **1139** tests |
| `npm run build` | vert |
| E2E complet (`npm run test:e2e`, backend `0.13.0` sur `:3011`) | **245** réussis, **9** échecs, 19 sautés (11,4 min) |

Les neuf échecs E2E sont **tous** dans la liste des attendus (`docs/testing.md` § « Les échecs
attendus »), jugés fichier par fichier : les **sept** KF-029 (#97 — `mode-expert:26`, `:41`,
`onboarding-path-b:65`, `:92`, `onboarding:57`, `:77`, `:150`) et les **deux** KF-045 (#421 —
`invoices.spec.ts:415` et `:439`, l'ancien `:405`/`:429` décalé ; run achevé à 09:25 UTC, avant 12:00).
Aucune pollution ce run. Le gate runtime P2-bis est donc passé : `migrations_fresh_install`,
`downgrade_protection_*`, `admin_backup_e2e` et `admin_full_import_e2e` dans les 3086, et le backend
`0.13.0` a booté sur une base migrée jusqu'à `20261009000001` (`/health` : `version 0.13.0`).
⚠️ Le frontend n'a pas changé entre `74d164ca` (où `check` et `test:unit` ont tourné) et `ee6cb1c3`
(le diff ne touche qu'un doc-comment Rust) ; `build` et l'E2E ont tourné sur `ee6cb1c3`. Un premier gate
backend, sur `74d164ca`, avait rendu le même 3086/3086.

### Remédiation de la revue de code P1 — 2026-10-09 (Opus 5.5, remédiateur)

Commits : `b335e90a` (code, tests, documentation), `15a67932` (catalogue `fr-CH` rétabli — voir
ci-dessous). Choix au registre : **C-15-1a-i-6** à **-10**.

- **Ce que la remédiation touche en production** : `kesh-db/src/repositories/letterings.rs`
  (`fiscal_year_names` prend une connexion et rend `Invariant` si un exercice manque ;
  `group_account_number`, nouveau, rend `Invariant` si le compte manque ; `find_group` partage ces deux
  lectures) ; `kesh-api/src/errors.rs` (le bras `LetteringTooManyLines` lit `max`) ; la clé
  `error-lettering-too-many-lines` des quatre catalogues (`{ $max }`). Documentation : `CHANGELOG.md`,
  `docs/api-external.md`, `docs/manual/fr/user-manual.tex` et `.pdf` (régénéré, glossaire contrôlé aplati).
- **Tests ajoutés**, périmètre `b4ed4d61..15a67932`, recomptés par fichier : `admin_full_import_e2e.rs`
  33 → 34 (`full_import_round_trip_keeps_lettering_marks`), `letterings_e2e.rs` 7 → 8
  (`a_read_only_key_reads_but_cannot_letter`) — **+2** ; renforcés sans changement de nombre :
  `lettering_invariants` (groupe `System`, seconde société, deux contrôles négatifs),
  `the_detector_sees_writes_and_only_writes` (sept littéraux neufs : cinq écritures `G`, `H`, `I`, `J`, `M`, et deux témoins négatifs `K` `updated_at` et `L` `FOR UPDATE` — « cinq » corrigé en revue de code P2, A2-4), le test de mapping (`max: 7`),
  `form_refusals_are_400` (message rendu).
- **Mutations rejouées** (chacune restaurée puis le binaire touché) :
  colonnes de lettrage exclues de l'**export** (`non_generated_columns`) → `full_import_round_trip_keeps_lettering_marks`
  **rouge** (« lettering_key absent du manifeste ») ; colonnes écartées de l'**INSERT de restauration** → **rouge**
  (« l'import doit rétablir la marque ») ; clause `COUNT(DISTINCT je.company_id)` retirée → `lettering_invariants`
  **rouge** (« clause des sociétés ») ; clause des origines retirée → **rouge** (« clause des origines ») ;
  `REPLACE` retiré du détecteur → `the_detector_sees_writes_and_only_writes` **rouge** (8 ≠ 9) ; message rendu
  par `t` sans la variable → `form_refusals_are_400` **rouge** (`{$max}` non substitué). ⚠️ **Non tuée** : passer
  `200` en dur dans l'argument au lieu de `max` — équivalente tant que le plafond vaut 200, seul plafond de
  production ; le repli, lui, est éprouvé avec `max: 7`.
- ⚠️ **Incident de mutation, rattrapé par le gate** : la mutation du plafond avait d'abord été posée par un
  `sed` sans ancrage sur le catalogue `fr-CH`, qui a réécrit en « 200 » la première variable `{ $max }` de
  **huit** autres clés *(le message de `15a67932` dit « sept » : recompté depuis le diff, `git diff b4ed4d61 b335e90a -- crates/kesh-i18n/locales/fr-CH/messages.ftl`, huit lignes hors lettrage)* (`error-username-too-long`, `invoices-*-too-long`, `invoices-format-error-*`,
  `reconciliation-*`) ; la restauration ne visait que la ligne du lettrage, et le résidu est parti dans
  `b335e90a`. Le gate complet l'a vu (`kesh-i18n loader::tests::format_with_args` rouge) ; catalogue repris
  de `b4ed4d61` dans `15a67932`, diff des quatre catalogues contre `b4ed4d61` réduit à la seule clé du
  lettrage (vérifié). Les autres mutations étaient restaurées par copie du fichier.
- ⚠️ **Environnement** (C-15-1a-i-10) : tmpfs MariaDB plein (`ibdata1` à 3,5 Go). Deux premiers gates
  complets **non concluants** (1523 puis 330 échecs `1114 table is full`, aucun du code) ;
  `innodb_file_per_table` basculé à `OFF` (volatil), mes bases de test résiduelles supprimées, gate rejoué
  à **deux** threads. *(Revue de code P2, A2-2 : les journaux des deux runs non concluants n'ont pas été
  conservés — « aucun du code » n'est donc **pas étayé** par un journal, seulement par le code d'erreur
  observé. Le réglage est **global** : tant que le conteneur n'est pas redémarré, les gates de tous les
  agents tournent sous `innodb_file_per_table=OFF` — sans effet sur les verrous ni la sémantique SQL.
  Ce gate à deux threads n'est pas le gate de référence à huit, celui du `CLAUDE.md` : il est à rejouer
  par l'orchestrateur après redémarrage.)*

**Gates de la remédiation, au dernier commit de code `15a67932`** (bases `kesh_151ai` et
`kesh_e2e_151ai` reconstruites par `DROP/CREATE` + migrations du worktree + seed immédiatement avant) :

| gate | résultat |
|---|---|
| `cargo fmt --all -- --check` | vert |
| `cargo clippy --workspace --all-targets -- -D warnings` | vert, 0 avertissement |
| `cargo nextest run --workspace --profile ci --test-threads=2` *(au lieu de `scripts/test-fast.sh --ci`, 8 threads : même contenu, parallélisme réduit par l'espace disque)* | vert — **3088** exécutés, **3088** réussis, 4 ignorés, 0 flaky (331,6 s) |
| `npm run check` | 0 erreur, 27 avertissements (préexistants) |
| `npm run lint-i18n-ownership` | PASS |
| `npm run test:unit` | vert — 114 fichiers, **1139** tests |
| `npm run build` | vert |
| E2E complet (backend `0.13.0` sur `:3011`, `/health` : `smtpConfigured: true`) | **245** réussis, **9** échecs, 19 sautés (10,2 min) |

Les neuf échecs E2E sont ceux de la liste des attendus, jugés fichier par fichier : sept KF-029 (#97 —
`mode-expert:26`, `:41`, `onboarding-path-b:65`, `:92`, `onboarding:57`, `:77`, `:150`) et deux KF-045 (#421
— `invoices.spec.ts:415`, `:439` ; run achevé vers 10:45 UTC, avant 12:00). Aucune pollution. 3086 → 3088 :
les deux tests ajoutés.

### Remédiation de la revue de code P2 — 2026-10-09 (Opus 5.5, remédiateur)

Choix au registre : **C-15-1a-i-11** à **-13**. Section « Reçu de la 15-1a-i » ajoutée aux fiches
`15-1a-ii-gardes-du-lettrage.md` et `15-1a2-lettrage-des-pieces.md` sur la branche de planification
`story/15-5-gardes-postabilite-serveur` (`fecf18ae`).

- **Ce que la remédiation touche en production** : `kesh-db/src/repositories/letterings.rs` —
  `build_group` rend `Result` et refuse en `Invariant` un exercice de ligne absent des noms lus, au lieu
  du repli `unwrap_or_default()` (B2-2) ; ses trois appelants propagent par `?`. Doc-comments seulement :
  `kesh-api/src/routes/letterings.rs` (`:12`, `:45`) et `kesh-db/src/errors.rs` (`LetteringTooManyLines`)
  renvoient à `MAX_LINES_PER_GROUP` au lieu d'écrire « 200 » (B2-5).
- **Détecteur lexical** (`letterings_lexical.rs`, B2-1 = E2-1) : `neutraliser_echappements` remplace
  chaque séquence d'échappement — la barre oblique inverse et le caractère qui la suit — par deux espaces
  avant le découpage en mots ; l'auto-test gagne cinq littéraux (`N` `\nUPDATE`, `O` `\tINSERT`,
  `P` `\rREPLACE`, `Q` `\0UPDATE`, `R` continuation de ligne) et passe de 9 à 14 écritures vues.
  **Mutation rejouée** : neutralisation retirée (découpage du texte brut) →
  `the_detector_sees_writes_and_only_writes` **rouge** (`left: 10`, `right: 14` — `N`, `O`, `P`, `Q`
  manqués ; `R` vu sans la neutralisation, le saut de ligne séparant déjà les mots : témoin, non preuve) ;
  fichier restauré par copie puis `touch`, binaire relancé vert.
- **Test ajouté au contrôle négatif (2) de `lettering_invariants`** (A2-3) : `find_group` lu par la seconde
  société sur la donnée corrompue rend `Invariant` du **compte** ; l'écriture intruse passée sur un exercice
  de la première société, il rend `Invariant` de l'**exercice** ; état rétabli. ⚠️ **Compilé (clippy
  `--all-targets`), non exécuté** : il exige MariaDB, saturée — au gate complet de l'orchestrateur.
- **Fiche** : T10 en `{ $max }` (A2-5) ; « cinq littéraux » → sept (A2-4) ; gate de la remédiation P1
  qualifié au Status, au Change Log et au Dev Agent Record (A2-2).
- **« 200 » re-grepé par la valeur** sur les fichiers de la story (`git diff dc4bc58b --name-only | xargs grep
  -nE "\b200\b"`) : restent légitimes `MAX_LINES_PER_GROUP = 200` (la définition), le test
  `manual_line_count_is_capped_at_200` (il fige la valeur d'AC6 contre la constante), et la documentation
  publique (`CHANGELOG.md:15`, `api-external.md:288`, `:306`, `:558` — valeur annoncée à l'intégrateur) ;
  `errors.rs:767` et `kesh-api/src/errors.rs:3464` parlent des lignes de facture, hors sujet.

**Gate de la remédiation P2 — gate ciblé, gate complet à rejouer après redémarrage de MariaDB** (tmpfs
plein, C-15-1a-i-10 ; consigne de l'orchestrateur : pas de gate complet) :

| gate | résultat |
|---|---|
| `cargo fmt --all -- --check` | vert |
| `cargo clippy --workspace --all-targets -- -D warnings` | vert, 0 avertissement |
| `cargo nextest run -p kesh-db -E 'binary(letterings_lexical) \| (kind(lib) & test(/repositories::letterings::tests/))'` | vert — 6 exécutés, 6 réussis |

Non exécutés : `kesh-db/tests/letterings.rs` (test A2-3, base requise) et tout ce qui touche la base,
dont les suites qui traversent `build_group` (`letterings`, `letterings_e2e`) ; front et E2E (rien de
frontal touché, mais l'E2E complet est dû au dernier commit de code). **À l'orchestrateur** : après
redémarrage du conteneur, gate complet de référence (`scripts/test-fast.sh --ci`, huit threads) et E2E
complet sur le dernier commit de code de cette remédiation.

### Intégration sur `origin/main` `803f3e15` — 2026-10-09 (Opus 5.5, intégrateur)

**Rebase** de la branche (17 commits, base `dc4bc58b`) sur `origin/main` `803f3e15`, qui porte la 15-13a,
la 15-7b1, la 15-13b et la 15-6c ; sauvegarde préalable `backup/15-1a-i-avant-rebase-803f3e15` (tête
`17d9de95`). Conflits, tous résolus sans toucher au sens de la story (C-15-1a-i-14) :

- `epic-15-choix-autonomes.md` et `sprint-status.yaml` : par **union** ; les clés 15-13* prennent le statut
  de `main` (celui relevé au T0 sur leurs branches, C-15-1a-i-5, était périmé) ; l'en-tête de cette branche
  renuméroté (44), puis (45) à la clôture.
- `crates/kesh-api/src/errors.rs` (module `tests`) : union des tests de la 15-13b et de ceux du lettrage.
- `crates/kesh-api/tests/audit_route_registry.rs` : partition **recomptée** — 114 routes = **107** tracées
  (105 de `main`, dont le peuplement de démonstration de la 15-7b1, + le lettrage et le délettrage) +
  **5** exemptées (`main`, 15-7b1) + 2 sans objet ; registre 117 inchangé.
- `docs/manual/fr/user-manual.pdf` : jamais fusionné ; `.tex` fusionné automatiquement, PDF régénéré par
  `make -B fr` (les PDF d'administration et de la brochure, dont le `.tex` est celui de `main`, rendus à
  leur version de `main`).

**Contrôles de version et de migrations** : `main` n'a reçu **aucune migration** depuis `dc4bc58b`
(dernière : `20261003000001`) ; `20261009000001_journal_entry_lines_lettering.sql` reste la dernière et son
`UPDATE _kesh_version SET kesh_version_min_required = '0.13.0'` est en dernière instruction ; la version
Cargo de `main` est restée `0.12.1`, les dix crates de la branche sont à `0.13.0` (P2-bis tenu).
`ls crates/kesh-db/migrations/*.sql | wc -l` → **76** = `grep -c '^| `20' docs/migrations-idempotence-audit.md`
→ 76 = en-tête et ligne `Total` (76) ; partition recomptée depuis le tableau : `yes` 8 + `tracked-by-sqlx`
68 + `no` 0 = 76. Empreinte `migrations.sha384` de `20261009000001` = `sha384sum` du fichier. **P6** :
`grep -rn "migrations.len()\|apply_migrations_up_to" crates/` — tous les sites résolvent par version
(`migrations_before`) ou portent leur garde-fou (`migrations_upgrade_path.rs`) ; aucune migration neuve de
`main` ne les décale.

**Gate de référence sur l'état rebasé** (tête `f12f4ca2`, dont le dernier commit de code est `f983f5df` —
le `acd19bd8` rebasé ; MariaDB redémarrée, `innodb_file_per_table=ON`, tmpfs 1,3 Go / 4 Go avant et après ;
bases `kesh_151ai` et `kesh_e2e_151ai` reconstruites par `DROP/CREATE` + 76 migrations du worktree + seed
immédiatement avant ; `CARGO_TARGET_DIR` du worktree) :

| gate | résultat |
|---|---|
| `scripts/test-fast.sh` (fmt, clippy `-D warnings`, nextest profil par défaut, **huit threads**) | vert — **3135** exécutés, **3135** réussis, 4 ignorés, 0 flaky (518,4 s) |
| `npm run check` | 0 erreur, 27 avertissements (préexistants) |
| `npm run lint-i18n-ownership` | PASS |
| `npm run test:unit` | vert — 114 fichiers, **1149** tests |
| `npm run build` | vert |
| E2E complet (backend `0.13.0` sur `:3011`, `/health` : `smtpConfigured: true`) | **246** réussis, **8** échecs, 19 sautés (15,2 min) |

Les huit échecs E2E, jugés fichier par fichier contre `docs/testing.md` § « Les échecs attendus » : les
sept KF-029 (#97 — `mode-expert:26`, `:41`, `onboarding-path-b:65`, `:92`, `onboarding:57`, `:77`,
`:150`) et `dunning.spec.ts:59` (« éditer la période de grâce », valeur `5` lue au lieu de `10`), **hors
liste, rejoué seul : vert** — la pollution d'état « qui change d'identité ». KF-045 n'a pas rougi (run
après 12:00 UTC). Aucun `reconciliation_*_e2e` rouge au gate backend.

### File List

Code et tests : `crates/kesh-core/src/lettering.rs` (neuf), `crates/kesh-core/src/lib.rs`,
`crates/kesh-db/migrations/20261009000001_journal_entry_lines_lettering.sql` (neuf),
`crates/kesh-db/migrations.sha384`, `crates/kesh-db/test-schema/0001_schema_squash.sql`,
`crates/kesh-db/src/entities/journal_entry.rs`, `crates/kesh-db/src/errors.rs`,
`crates/kesh-db/src/post_restore.rs`, `crates/kesh-db/src/repositories/journal_entries.rs`,
`crates/kesh-db/src/repositories/letterings.rs` (neuf), `crates/kesh-db/src/repositories/mod.rs`,
`crates/kesh-db/tests/letterings.rs` (neuf), `crates/kesh-db/tests/letterings_lexical.rs` (neuf),
`crates/kesh-db/tests/journal_entries_modification.rs`, `crates/kesh-db/tests/migrations_fresh_install.rs`,
`crates/kesh-db/tests/migrations_upgrade_path.rs`, `crates/kesh-api/src/audit_labels.rs`,
`crates/kesh-api/src/errors.rs`, `crates/kesh-api/src/exports/csv_tables.rs`, `crates/kesh-api/src/lib.rs`,
`crates/kesh-api/src/routes/journal_entries.rs`, `crates/kesh-api/src/routes/letterings.rs` (neuf),
`crates/kesh-api/src/routes/mod.rs`, `crates/kesh-api/tests/audit_route_registry.rs`,
`crates/kesh-api/tests/letterings_e2e.rs` (neuf), `crates/kesh-i18n/locales/{fr-CH,de-CH,en-CH,it-CH}/messages.ftl`,
les dix `crates/*/Cargo.toml` (0.13.0) et `Cargo.lock`.
Frontend : `journal-entries.types.ts`, `JournalEntryForm.edit.test.ts`, `form-helpers.test.ts`,
`invoices/settlement-cancel.ts` et `.test.ts`, `InvoiceSettlements.test.ts`,
`reconciliation/reconciliation-cancel.ts` et `.test.ts`, `shared/utils/settlement-cancel-blocked.test.ts`.
Revue P1 : `crates/kesh-api/tests/admin_full_import_e2e.rs` (en plus des fichiers déjà listés).
Documentation : `CHANGELOG.md`, `docs/api-external.md`, `docs/MULTI-TENANT-SCOPING-PATTERNS.md`,
`docs/migrations-idempotence-audit.md`, `docs/manual/fr/user-manual.tex` et `.pdf`.
Planification : cette fiche, `epic-15-choix-autonomes.md`, `sprint-status.yaml`.

## Change Log

### Intégration sur `origin/main` `803f3e15` — 2026-10-09 (Opus 5.5, intégrateur)

Rebase sur `803f3e15` (15-13a, 15-7b1, 15-13b, 15-6c) ; conflits résolus par union (registre, sprint-status,
tests d'`errors.rs`), partition d'audit recomptée (114 = 107 + 5 + 2), PDF utilisateur régénéré (C-15-1a-i-14).
Aucune migration mergée entre-temps, version Cargo de `main` inchangée (0.12.1), compteurs d'audit 76 = 8 + 68 + 0.
Gate de référence sur l'état rebasé : backend **3135/3135** à huit threads, Vitest **1149/1149**, E2E **246** /
7 KF-029 + 1 pollution rejouée verte. Statut **done**.

### Revue de code P3 ciblée — 2026-10-09 (Haiku ; remédiation par l'orchestrateur, Opus 5.5)

Passe ciblée sur `8cda7041` (prompt `d3bead04`, rapport `/home/gcorbaz/devel/kesh-gate-logs/15-1a-i-review-p3-ciblee.md`) :
**1 MEDIUM**, né de la remédiation P2. **F1** — `neutraliser_echappements` consomme le caractère qui suit toute
barre oblique ; dans une chaîne brute, où `\` n'échappe rien, `\UPDATE` devenait `  PDATE` et le verbe
échappait au détecteur, alors qu'il le voyait avant `8cda7041` ; le doc-comment « elle ne le rétrécit jamais »
était faux. Vérifié par l'orchestrateur à la lecture du code. **Correction** : `ecritures_de_la_marque` cherche
le verbe dans **l'union** de deux lectures — le texte neutralisé (`\nUPDATE` → `UPDATE`) et le texte brut, où
`\` fait séparateur (`\UPDATE` → `UPDATE`) ; doc-comment rectifié ; deux littéraux neufs à l'auto-test
(`S` : `r#"SELECT 1;\UPDATE …"#`, `T` : `r#"x\\\INSERT …"#`), 14 → 16 écritures vues. **Mutation rejouée** :
retirer la lecture brute (`|| ecrit_dans(&l.texte)`) → `the_detector_sees_writes_and_only_writes` rouge
(`left: 14`, attendu 16) ; fichier restauré puis touché, binaire vert. Points 1, 3 et 4 de la lentille
(`build_group` → `Result`, contrôle négatif A2-3, doc-comments `MAX_LINES_PER_GROUP`) : sans finding.

**Gate ciblé** (fichier de test seul, aucun code de production) : `cargo fmt --all -- --check` vert ;
`cargo clippy --workspace --all-targets -- -D warnings` vert ; `cargo nextest run -p kesh-db --test
letterings_lexical` 3/3. **Le dernier commit de code est désormais `acd19bd8`** (un fichier de test est du code, décision D7) : le
gate complet de référence (8 threads) et l'E2E complet se rejouent sur lui, après le redémarrage de MariaDB
(tmpfs saturé), avant la PR.

**Boucle de revue close** : la remédiation de cette passe ciblée ne touche aucune ligne de code de production
(CLAUDE.md § « La passe ciblée »). Trend : P1 (Sonnet ×3) 3 MEDIUM → P2 (Opus ×3) 2 MEDIUM, nés de P1 → P3 ciblée
(Haiku) 1 MEDIUM, né de P2, dans un fichier de test.

### Revue de code P2 — 2026-10-09 (Opus ×3 ; remédiation Opus 5.5)

Trois lentilles Opus en contexte frais sur `dc4bc58b..30398986`, lues d'abord sur la remédiation P1
`b4ed4d61..30398986` (rapports `kesh-gate-logs/15-1a-i-review-p2-{B,E,A}.md`) : **B** 0 CRITICAL / 0 HIGH /
1 MEDIUM / 4 LOW, **E** 0 / 0 / 1 / 3, **A** 0 / 0 / 1 / 4. Après dédoublonnage (B2-1 = E2-1, B2-3 = E2-3,
B2-5 ≈ A2-5) : **2 MEDIUM distincts, tous deux nés de la remédiation P1** (B6 et C-15-1a-i-7), aucun de la
conception d'origine. **Trend** : P1 **3 MEDIUM** → P2 **2 MEDIUM** distincts de ceux de P1 — le motif « la
remédiation introduit le défaut suivant », non une stagnation (pas de signal de découpage au sens de D5).

- **Remédiés** : B2-1 = E2-1 (détecteur aveugle à `\nUPDATE` — échappements neutralisés, mutation tuée,
  C-15-1a-i-11) ; A2-1 (relais des six textes provisoires écrit aux fiches 15-1a-ii et 15-1a2, `fecf18ae`
  sur la branche de planification, C-15-1a-i-11). LOW appliqués : B2-2, B2-5 = A2-5, A2-2, A2-3, A2-4.
  LOW écartés avec motif : B2-3 = E2-3, B2-4, E2-2, E2-4 — C-15-1a-i-12.
- **Code de production touché** : oui — `letterings.rs` (`build_group` rend `Result`, B2-2) ; doc-comments
  de `routes/letterings.rs` et `errors.rs` (B2-5). La boucle ne se clôt donc pas sur cette passe.
- **Gate ciblé seulement** (fmt, clippy, `binary(letterings_lexical)` et tests unitaires de `letterings`,
  6/6) ; le test A2-3 est compilé, non exécuté. Gate complet et E2E complet **à rejouer après redémarrage
  de MariaDB** (C-15-1a-i-13). Détail au Dev Agent Record.

### Revue de code P1 — 2026-10-09 (Sonnet ×3 ; remédiation Opus 5.5)

Trois lentilles Sonnet en contexte frais sur `dc4bc58b..09a9d16b` (rapports
`kesh-gate-logs/15-1a-i-review-p1-{B,E,A}.md`) : **B** 0 CRITICAL / 0 HIGH / 2 MEDIUM / 6 LOW, **E** 0 / 0 / 1 / 7,
**A** 0 / 0 / 0 / 6. Après dédoublonnage (B2 = A-L3, E-1 = A-L4, E-6 ⊂ A-L6, E-5 = B8) : **3 MEDIUM** distincts.

- **Reclassé** : B1 (MEDIUM → **LOW**, décision de l'orchestrateur, C-15-1a-i-6) — modification et suppression
  d'une écriture lettrée : périmètre de la 15-1a-ii, dont AC8 couvre `update_in_tx` (étape 7-bis, avant le
  `DELETE`+`INSERT`) et `delete_in_tx` (étape 3-quinquies, inconditionnelle) ; C124 interdit tout tag entre les
  deux merges.
- **Remédiés** : B2 = A-L3 et A-L2 (textes publics ramenés au code livré, C-15-1a-i-7) ; E-1 = A-L4 (aller-retour
  de sauvegarde, `is_no_op_change` et `entry_snapshot_json` constatés, C-15-1a-i-8). LOW appliqués : E-2, E-4,
  B6, E-6 = A-L6 (première moitié), E-7, E-8, A-L1, A-L5. LOW écartés avec motif : B3, B4, B5, B7, B8 = E-5, E-3,
  A-L6 (seconde moitié) — C-15-1a-i-9.
- **Code de production touché** : oui — `letterings.rs` (E-2), `errors.rs` et les quatre catalogues (E-4). La
  boucle ne peut donc pas se clore sur cette passe (règle de la passe ciblée).
- Gate complet et E2E complet au dernier commit de code `15a67932` : backend 3088/3088 **à `--test-threads=2`,
  `innodb_file_per_table=OFF`** (tmpfs plein, C-15-1a-i-10 — pas le gate de référence à huit threads ;
  qualificatif ajouté en revue de code P2, A2-2), Vitest 1139, E2E 245 / 9 attendus. Détail, mutations et
  incident de catalogue au Dev Agent Record.

### Développement — 2026-10-09 (Opus 5.5, agent de développement)

Implémentation (`cbaf5a18`), correctif du compte figé des exemptions resté non commité au crash
(`74d164ca`), doc-comment reformulé pour que le contrôle d'AC15 rende ses cinq lignes triées
(`ee6cb1c3`). Gate complet au dernier commit de code : backend 3086/3086 (4 ignorés), Vitest 1139/1139,
build vert, E2E 245 réussis / 9 attendus (7 KF-029 + 2 KF-045 du matin). Trois mutations rejouées et
tuées (tri par `id` ; retrait de `check_rows_affected` dans chacune des deux primitives). Détail au Dev
Agent Record. Statut → `review`.

### T0 — relevés au sol sur `dc4bc58b` — 2026-10-09 (Opus 5.5, agent de développement)

**Prérequis constatés** : la **15-12a** est mergée (`LOCK_EARLIER_BY_ID_SQL`, `fiscal_years.rs:178` ;
`DbError::EarlierFiscalYearOpen`, `:22`) ; la **15-12b** aussi (`find_later_closed`, `fiscal_years.rs:887`,
lu sans verrou par le filet de `create_in_tx_inner`). Les deux autres stories citées (15-6b, 15-7a2,
15-11b) ont déplacé des numéros, aucune règle.

**Numéros relocalisés par le texte** (la fiche citait `5e4bec50`) : `LINE_COLUMNS` `journal_entries.rs:80 →
:100` ; `list_all_lines_by_company` `:1824/:1828 → :1872/:1876` ; `reversal_blockers` `:1922 → :1970` ;
`find_later_closed` `:678 → :887`, `FIND_LATER_CLOSED_SQL` `:668-671 → :869-872` ; `LIB_ROUTES`
`audit_route_registry.rs:168 → :196` ; replis « facture créditée » `kesh-api/src/errors.rs:2966/:3495 →
:3172/:3682` ; catalogue `fr-CH:783/:818 → :792/:827` ; `api-external.md:325/:386 → :328/:410`, catalogue des
codes `:485 → §10 (:490-512)`, routes rejouées `:488 → :514` ; manuel : verrou de période `:578-583 →
:566-584`, glossaire `:2323 → :2398`, « à lettrer » `:1171/:1711 → :1203/:1770` ; `CHANGELOG.md:74 → :88`
(sous `[0.12.1]`, trié). Inchangés : `post_restore.rs:512-525`, `journal_entries_modification.rs:78/181/197/220`,
`migrations_upgrade_path.rs:484/509/524`, `migrations_fresh_install.rs:230/244`, `csv_tables.rs:350/1687`.
Recomptés : **six** fichiers citent `0.12.1` hors `Cargo.toml` (les six de la fiche, mentions historiques
non réécrites) ; `0.10.0` dans `crates/*/tests` : les huit sites de la fiche ; migrations **75**,
`tracked-by-sqlx` **67** (avant cette story) ; inventaire « à lettrer » : les sites de la fiche plus le
`CHANGELOG.md:88` trié.

**Mesures sur `kesh_151ai`** (migration de T1 appliquée ; 3000 écritures, 6000 lignes, 500 groupes ;
`ANALYZE`) :
- `EXPLAIN` de l'acte 1 de la **création** (`jel.id IN (3 ids)`) : `jel` par `PRIMARY` (`range`), `je` par
  `PRIMARY` (`eq_ref`) — le parcours part de la clé primaire des lignes, comme R7 le demande ; idem avec 40
  identifiants. De la **dissolution** (`jel.lettering_key = ?`) : `jel` par `idx_jel_lettering` (`ref`), `je`
  par `PRIMARY` (`eq_ref`). ⚠️ **Sur tables vides** (avant le peuplement), le plan à 40 identifiants prenait
  `je` par `idx_journal_entries_company_date` (`ref const`), qui verrouillerait tous les en-têtes de la
  société : sur-verrouillage des seules bases minuscules (bases de test), sans effet sur l'exactitude — les
  routes sont rejouées. Écrit, non corrigé.
- **Verrous d'intervalle** (F4-1) : la moitié « création » est **démentie** (R7 corrigé, C-15-1a-i-1) ; F3-5
  **confirmé**.
- `1205` de la sonde `NOWAIT` : non relevé à la main, par consigne (mesuré par la 15-5d) ; le test d'ordre
  d'acquisition l'exerce à travers `sonde_verrou_nowait` (la mutation « tri par `id` » le fait rendre `false`).

**Écart de fond** : aucun — aucune règle, aucun critère ne change ; seul un cas d'interblocage résiduel,
écrit et non corrigé, disparaît.

**Registre de sprint** : clés `15-1a-i`, `15-1a-ii`, `15-1a2` et les trois `15-13*` ajoutées,
`15-1a-socle-lettrage` passée à `split` (C-15-1a-i-5).


### Remédiation de la validation P4 — 2026-10-09 (Opus 5.5)

**Validation P4** (Sonnet 5.5 ×2, contexte frais, prompt versionné `b53f7278`) : R (chasseur de
régressions) **0 HIGH, 2 MEDIUM, 5 LOW** ; F (adversaire plein périmètre) **0 HIGH, 0 MEDIUM, 8 LOW**
(`target/gate-logs/15-1a-i-p4-{R,F}.md`). Recoupements : R4-3 = F4-3, R4-4 ≈ F4-5. **Distincts : 0 HIGH,
2 MEDIUM** (R4-1, R4-2) **et 11 LOW** (R4-3 à R4-7, F4-1, F4-2, F4-4, F4-6, F4-7, F4-8). **Trend** (15-1a
puis 15-1a-i) : P1 **2 HIGH / 15 MEDIUM** → P2 **0 / 5 / 16 LOW** → P3 **0 / 4 / 16** → P4 **0 / 2 / 11**.
Modèles : P1 Opus ×2, P2 Sonnet ×2, P3 Opus ×2, P4 Sonnet ×2 (rotation D6) ; remédiation Opus 5.5.

⚠️ **Signal D5, déclaré au Project Lead** : R4-2 **naît d'un correctif** — C127 (P3) a exigé
`fiscalYearName` pour toutes origines sans suivre le mode `System`, que C125 laissait « sans requête »
(recyclage). R4-1 est un défaut d'**origine** (inventaire de C106, P1, limité au manuel et à l'API). Un
seul recyclage, réparé par une lecture non verrouillante (C128) : pas de nouveau découpage proposé.
**Signal de périmètre** *(F4-6)* : une dizaine de modules métier dont cinq mécaniques (Dev Notes) —
déclaré, découpage jugé inutile (la primitive ne s'éprouve qu'à travers ses routes).

| finding | décision | où |
|---|---|---|
| R4-1 (MEDIUM) | Promesse « à lettrer » retirée **dans cette story** de tous les sites visibles, inventoriés par la valeur en quatre langues ; texte de remplacement arrêté FR/DE/EN/IT ; `CHANGELOG.md:74` (`[0.12.1]`, publié) trié, non réécrit, le changement annoncé sous *Modifié* ; contrôle `git grep` final | R5, AC15 (i), T10, T11, C130 ; 15-1a2 Reçu point 19 |
| R4-2 (MEDIUM) = R4-1 de la 15-1a-ii | Mode `System` : nom des exercices lu par une lecture ordinaire non verrouillante après l'acte 1 ; « sans requête » levé | R7 point 3, AC6, AC10, T3, T12, C128 ; 15-1a-ii (R6 « Coût », AC10 ii) ; 15-1a2 Reçu point 18 |

**LOW appliqués** : R4-3 = F4-3 (champs frontend requis et nullables ; fixtures nommées — AC14, T8),
R4-4 = F4-5 (`sonde_verrou_nowait` réutilisée, `1205` de la 15-5d ; T0 n'a plus à le relever), R4-5
(requête bloquée observée deux fois à 100 ms ; motifs de la clôture pour le test de sérialisation), R4-6
(Pattern 5 : `:291`, `:311`, `:338`), R4-7 (`:2304-2306`), F4-1 (verrous d'intervalle côté création : R7
point 1 et troisième cas d'interblocage résiduel, constat en T0), F4-2 (`to_ascii_uppercase` puis
validation ; `ß`, `ſ`, `ı`), F4-4 (`check_rows_affected` pure + mapping 409 ; **écart** à la consigne du
déclencheur SQL, motivé : `CLIENT_FOUND_ROWS` — C131), F4-6 (cette ligne de signal), F4-7 (décomposition
de la ligne `Total` de l'audit d'idempotence, T1), F4-8 (manuel d'administration et README : décision
écrite aux Dev Notes).

**Propagation** (grep par la valeur, dépôt entier) : `à lettrer|noch zuzuordnen|zuzuordnende|to be
matched|da abbinare` (inventaire d'AC15 part i) ; `298-304`, `2304-2305`, `code d'erreur relevé`,
`point d'injection` (aucun résidu dans les fiches 15-1a*) ; `sans requête` (seule occurrence restante :
le contrôle de l'exercice tenu, exact).

**Recompte de cette fiche** (`grep` sur le corps, avant `## Change Log`) : inchangé — **13 critères**,
**11 tâches**, **10 clés i18n** neuves au tableau de T10 (plus le lot d'audit, et **2 clés existantes
réécrites**, C130) ; **16 tests neufs nommés** (aucun nom neuf en P4 : les tests de
`check_rows_affected` sont unitaires) ; mutations nommées **3** (`:197`, « trier par `id` », « retirer
`check_rows_affected` »).

### Création au découpage de la 15-1a, et remédiation de la validation P3 — 2026-10-09 (Opus 5.5)

**Validation P3 de la 15-1a** (Opus ×2, contexte frais, prompt `15-1a-validate-prompt-p3.md`) : R
(chasseur de régressions) **0 HIGH, 3 MEDIUM, 10 LOW** ; F (adversaire plein périmètre) **0 HIGH,
2 MEDIUM, 6 LOW** (`target/gate-logs/15-1a-p3-{R,F}.md`). Recoupement : R3-1 = F3-1. **Distincts : 0
HIGH, 4 MEDIUM** (R3-1/F3-1, R3-2, R3-3, F3-2) **et 16 LOW** (R L1–L10, F3-3 à F3-8). **Trend de la
15-1a** : P1 **2 HIGH / 15 MEDIUM** → P2 **0 HIGH / 5 MEDIUM / 16 LOW** → P3 **0 HIGH / 4 MEDIUM /
16 LOW** (distincts). Modèles : P1 Opus ×2, P2 Sonnet ×2, P3 Opus ×2 (rotation D6) ; remédiation Opus 5.5.

⛔ **Signal D5 — recyclage, et découpage** : R3-1, R3-2 et R3-3 naissent de correctifs de P2 sur des
règles métier (C113, C114 et le test de C114). Le déclencheur écrit à C118 est atteint ; l'orchestrateur
a décidé le découpage selon sa couture (C124). Cette fiche porte la part (i) ; la part (ii) est
`15-1a-ii-gardes-du-lettrage.md` ; l'index `15-1a-socle-lettrage.md` garde la table de correspondance et
l'historique des passes.

| finding | décision | où (cette fiche) |
|---|---|---|
| R3-2 (+ L10, F3-4, F3-6) | Exercices du groupe lus sans verrou triés par `start_date`, puis verrouillés **un par un** par clé primaire (forme de C119) ; plus de verrou des postérieurs ; « postérieur clos » par `find_later_closed`, non verrouillant, avec sa preuve (α)-(β) ; vue ouverte par (a) écrite comme limite ; motifs de test discriminants | R7 point 2, T0, T3, T12, Dev Notes, C125 |
| R3-3 | Test d'ordre séparé et discriminant (sonde `NOWAIT` sur B, dissolution hors enveloppe, mutation « par `id` » nommée) ; sérialisation avec `close` sans prétention d'ordre ; scénario « refus » renvoyé au test du point 3 | T12, C125 |
| F3-2 (part i) | CHANGELOG : *Ajouté* complétée (elle existe) ; *Modifié* : champs neufs des lignes et deux colonnes CSV | AC15 (i), T11 (i), C127 |
| R3-1 = F3-1, F3-2 (part ii) | Dans la 15-1a-ii (précédence d'`ENTRY_LETTERED`, entrées #532) | — (C126, C127) |

**LOW appliqués ici** : L1 (en-tête de migration : « tout binaire publié qui passait le contrôle de
version », les v0.10.0–v0.11.1 annulent aussi), L2 (décompte des routes rejouées : six, plus quatre
annulations), L4 (`epics.md:1365-1366` alignés sur C113), L7 (motif de la dépendance à la 15-12a), L8
(*Ajouté* existe ; `kesh-api/src/errors.rs:2794 → :2822` ; manuel `:1169 → :1171`, `:1696 → :1711`,
FAQ `:2209 → :2224`, glossaire `:2308 → :2323` ; `invoices.rs:1659 → :1660`, `:1529-1606 → :1530-1607` ;
`supplier_invoices.rs:992-996 → :1046-1050` ; `CHANGELOG.md:13/23/42 → :17/27/48` pour *Modifié*,
*Corrigé*, *Sécurité*), L10 (sans objet : plus de parcours d'intervalle), F3-4, F3-5 (dissolution du
groupe de plus petite clé : verrou du trou devant les lignes ouvertes, toutes sociétés — R7, limite
F-11), F3-6, F3-7 (compte bancaire archivé : non lettrable, test), F3-8 (`fiscalYearId`/`fiscalYearName`
par ligne, réponse et audit). L3, L5, L6, L9, F3-3 relèvent de la part (ii) (15-1a-ii) ou de la 15-12a
(L5, porté à l'orchestrateur).

**Recompte de cette fiche** (`grep` sur le corps, avant `## Change Log`) : R1–R5 et R7 (R6 en
renvoi) ; **13 critères** (AC1–AC7, AC10–AC14, AC15 part i) ; **11 tâches** (T0 part i, T1, T2, T3, T6,
T7, T8, T9, T10 part i, T11 part i, T12 part i — T0, T10, T11 et T12 n'y portant que leur part i ;
cf. l'index pour le décompte aux deux bornes) ; **10 clés i18n** au tableau de T10 (plus le lot
d'audit).

- **2026-10-09 — validation P5 ciblée (Haiku, prompt `15-1a-i-validate-prompt-p5-ciblee.md`)** : rapport
  `target/gate-logs/15-1a-i-p5-ciblee.md`, **0 finding**, axes exercés et non exercés déclarés. Axe repris par
  l'orchestrateur, que la lentille n'a pas traité explicitement (point 1 du prompt : nom faux ou manquant entre
  l'acte 1 et `LETTERING_FISCAL_YEAR_NAMES_SQL`) : la fiche le couvre (`update_name` concurrent → nom d'avant
  audité, `fiscalYearId` exact) ; un exercice qui porte des lignes ne peut être supprimé. **Validation close.**
  Trend (15-1a puis 15-1a-i) : P1 2 HIGH / 15 MEDIUM → P2 5 MEDIUM → P3 4 MEDIUM (découpage, C124) → P4 2 MEDIUM
  → P5 ciblée 0. Modèles : Opus ×2, Sonnet ×2, Opus ×2, Sonnet ×2, Haiku (ciblée). Développement après la
  15-12a et la 15-12b (C112) ; pas de tag v0.13.0 entre la 15-1a-i et la 15-1a-ii, ni avant la 15-1c tant que
  le message `ENTRY_LETTERED` prescrit un délettrage sans écran.
