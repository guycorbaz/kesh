# Story 24.4c : Le verrou de période — arrêter les livres à une date

## Status

ready-for-dev

## Story

**As a** personne qui tient les livres,
**I want** arrêter les écritures à une date, indépendamment de la clôture annuelle,
**so that** une période déjà déclarée ou remise au fiduciaire ne bouge plus dans mon dos.

## ⛔ Le défaut a CHANGÉ de nature depuis la 24-4b, et la note de planification est périmée

Le sprint-status annonçait, le 2026-08-28 : *« une écriture de janvier reste réécrivable en
décembre, montant de TVA compris »*. **Ce n'est plus vrai.** La 24-4b a supprimé
`journal_entries::update` et refuse le `DELETE` : une écriture enregistrée n'est plus
modifiable, dans aucune période, par personne.

**Ce qui reste ouvert, et que cette story ferme, c'est l'ANTIDATAGE.**

`create_in_tx` ne contrôle qu'une chose sur la date (`journal_entries.rs:273`) :

```rust
if new.entry_date < fy_start || new.entry_date > fy_end {
    return Err(DbError::DateOutsideFiscalYear);
}
```

⇒ **Tant que l'exercice est ouvert, on peut créer aujourd'hui une écriture datée du 15 janvier.**
Elle entre dans un trimestre déjà déclaré, elle change les totaux de TVA de ce trimestre, et
**rien ne s'y oppose** — ni au journal manuel, ni par la validation d'une facture antérieure, ni
par un rapprochement bancaire.

⚠️ **Le mode d'échec est celui de toute la vague : il est silencieux.** Le rapport TVA se
recalcule à la volée (`kesh-report/src/vat_report.rs:83`, borné par un simple couple de dates) ;
il rendra donc, sans le signaler, un chiffre différent de celui qui a été déclaré à l'AFC. Rien
ne rougit, et l'écart ne se découvre qu'au contrôle.

## D1 — Le verrou est une DATE posée à la main, PAS un décompte TVA dérivé

L'issue #380 demande « un verrou de période plus fin que l'année, **au minimum le trimestre TVA
déclaré** ». ⛔ **On ne peut pas le dériver du décompte TVA, parce que le décompte n'existe pas
comme objet.**

Vérifié au sol : il n'y a **ni entité, ni table, ni colonne** portant un décompte déposé —
`ls crates/kesh-db/src/entities/ | grep -i vat` ne rend que `vat_rate.rs`, et les trois
migrations TVA (`vat_rates`, `vat_rates_crud`, `vat_accounts_config`) ne portent que des taux et
des comptes. `ReportPeriod` (`kesh-report/src/period.rs:16`) est une **struct de rapport**, deux
dates, sans persistance ni granularité.

⇒ **Le verrou est une date de la société : `books_locked_through`.** *« Les écritures jusqu'au
31.03.2026 inclus sont verrouillées. »* Une seule valeur, nullable, qui n'avance que vers
l'avant sauf déverrouillage explicite.

⚠️ **Pourquoi une date et non une liste de périodes** : une période « déclarée » ne se ferme
jamais toute seule au milieu — on déclare le T1 après le T1, jamais avant. Une **borne**
exprime donc exactement ce qu'on veut dire, avec un champ au lieu d'une table, et elle se lit
sans ambiguïté à l'écran comme au refus. ⛔ Ne PAS créer une table `locked_periods` : elle
autoriserait des trous (T1 et T3 verrouillés, T2 ouvert) qui n'ont aucun sens comptable.

⚠️ **Ce que cela n'interdit pas plus tard** : quand un décompte TVA déposé existera comme objet,
il pourra **proposer** la date à verrouiller — la borne reste le mécanisme, le décompte n'en
devient qu'une source de suggestion. *Le verrou ne dépend pas de la TVA ; c'est la TVA qui en
est le premier usage.*

## D2 — Le refus vit dans `create_in_tx`, le point de passage unique

⛔ **Vingt-deux sites du dépôt créent une écriture** (`grep -rn "journal_entries::create_in_tx\|journal_entries::create("`,
hors du module et hors tests) : facture validée, avoir, facture fournisseur ×3, règlement,
rapprochement ×5, ouverture, journal manuel. **Tous passent par `create_in_tx`.**

⇒ La garde s'y place, sur le modèle exact d'`enforce_postable` : **une seule ligne à écrire, et
aucun chemin ne peut la contourner**. Une garde posée aux routes en laisserait vingt et un
ouverts.

⛔ **La garde s'applique aux flux AUTOMATIQUES aussi, et c'est le point qu'il ne faut pas
adoucir.** Valider une facture datée du 15 janvier après le verrou produirait exactement
l'écriture antidatée que la story interdit — le fait qu'elle vienne d'une pièce n'y change rien.
Le refus doit donc porter le **numéro de la pièce** et la **date de verrou**, faute de quoi
l'utilisateur ne saura pas quoi corriger.

⚠️ **Une exception, et une seule : la contre-passation ne peut PAS être bloquée** — et elle ne
l'est pas, gratuitement. La 24-4a la date **du jour** (`find_open_covering_date(today)`, D4 de la
24-4a) ; comme le verrou est nécessairement dans le passé (cf. AC 3), la contre-passation tombe
toujours après lui. ⛔ **C'est ce qui garantit qu'une écriture d'une période verrouillée reste
corrigeable** : le verrou arrête ce qui entre dans le passé, jamais ce qui corrige depuis le
présent. *Aucune garde à écrire pour cela — mais un test à écrire, sans quoi la propriété se
perdra au premier refactor.*

## D3 — Poser, avancer, reculer : qui, avec quel motif, et ce qui est tracé

Le gabarit existe et se copie : `fiscal_years::reopen` (`fiscal_years.rs:779`) — `SELECT … FOR
UPDATE` scopé `(id, company_id)`, **motif obligatoire**, instantané avant/après au journal
d'audit.

| geste | rôle | motif | audit |
|---|---|---|---|
| **poser / avancer** la date | Admin **et** Comptable | facultatif | `books.locked` |
| **reculer / retirer** la date | **Admin seul** | ⛔ **obligatoire** | `books.unlocked` |

⚠️ **L'asymétrie est le cœur de la mesure.** Verrouiller est un geste d'hygiène, qu'on doit
pouvoir faire souvent et sans cérémonie. **Déverrouiller défait une garantie** : c'est ce geste
qui doit coûter, se justifier et se retrouver dans le journal d'audit — exactement comme la
réouverture d'un exercice clôturé.

⛔ **Ne pas confondre avec la clôture annuelle.** Les deux verrous coexistent et ne se
remplacent pas : la clôture est un **état d'exercice** avec ses règles propres (LIFO, report
à-nouveau, réouverture Admin) ; le verrou de période est une **borne mobile** à l'intérieur d'un
exercice ouvert. Un exercice clos reste clos quelle que soit la borne.

## D4 — La précédence des refus, figée ici

Sur une création, l'ordre est :

`DateOutsideFiscalYear` (400) → `FiscalYearClosed` (400) → **`PeriodLocked`** (400)

⚠️ **Le verrou parle en DERNIER, et c'est délibéré** : une date hors exercice ou dans un
exercice clos est un problème plus grave et plus ancien, dont le message existe déjà et renvoie
ailleurs. Dire « période verrouillée » à quelqu'un qui s'est trompé d'exercice l'enverrait
corriger la mauvaise chose.

⛔ **`PeriodLocked` est un 400, pas un 409** — c'est une **donnée d'entrée invalide** (la date
fournie), pas un conflit d'état de la ressource visée. C'est l'asymétrie que la 24-4a a figée en
D5-bis et que la 24-4b a suivie : `ENTRY_IS_POSTED` porte sur l'écriture qu'on veut changer,
`PERIOD_LOCKED` porte sur la date qu'on propose.

**Le message nomme les deux bornes** : la date refusée et la date de verrou. *« Les écritures
sont verrouillées jusqu'au 31.03.2026 ; cette écriture est datée du 15.01.2026. »*

## D5 — L'écran

- **Réglages → Comptabilité** : la date de verrou, son état, et les deux gestes. Le
  déverrouillage demande le motif dans une confirmation, comme la réouverture d'exercice.
- **Le formulaire de saisie** : le champ date porte un **`min`** à la date de verrou + 1 jour
  quand elle existe. ⚠️ **Ce n'est qu'un confort** — le refus qui fait autorité est celui du
  serveur, et il doit être testé sans passer par l'écran.
- ⛔ **Un bandeau sur la liste des écritures** quand un verrou est posé : *« Livres verrouillés
  jusqu'au … »*. Sans lui, l'utilisateur découvre le verrou **au refus**, et la 24-4a a établi
  qu'un refus découvert après le clic est un défaut, pas une fonctionnalité.

## Critères d'acceptation

1. La société porte une date de verrou `books_locked_through` (`DATE NULL`), `NULL` par défaut —
   **aucune installation existante ne change de comportement** à la migration.
2. Créer une écriture dont la date est **antérieure ou égale** à la date de verrou rend **400
   `PERIOD_LOCKED`** ; une écriture datée du **lendemain** de la borne passe.
3. Poser une date de verrou **dans le futur** est refusé : la borne est ≤ aujourd'hui. ⛔ Sans
   cette garde, un verrou futur bloquerait la contre-passation et rendrait les livres
   **incorrigibles** — le mode d'échec exact que l'ordre 24-4a → 24-4b existait pour éviter.
4. Le refus vaut pour **tous les chemins de création**, pas seulement le journal manuel : un
   test l'exerce sur la **validation d'une facture** antidatée et sur un **rapprochement
   bancaire** antidaté, et le message nomme la pièce.
5. La **contre-passation d'une écriture d'une période verrouillée aboutit** (201) — elle est
   datée du jour, donc après la borne. ⛔ Un test le verrouille : c'est la propriété qui empêche
   les livres de devenir incorrigibles, et rien d'autre ne la protège.
6. Avancer la borne est permis à **Admin et Comptable** ; la reculer ou la retirer est réservé à
   **Admin** et exige un **motif non vide**, sinon **400**.
7. Consultation reçoit **403** sur les deux gestes.
8. Chaque pose écrit une entrée d'audit `books.locked`, chaque retrait une entrée
   `books.unlocked` portant le **motif** et l'**ancienne** valeur.
9. La précédence est testée : une date **hors exercice** dans une période verrouillée rend
   `DATE_OUTSIDE_FISCAL_YEAR`, et une date d'un **exercice clos** rend `FISCAL_YEAR_CLOSED` —
   jamais `PERIOD_LOCKED`.
10. La clôture annuelle et le verrou de période sont **indépendants** : clôturer un exercice ne
    touche pas la borne, et la borne n'empêche ni la clôture ni la réouverture.
11. **Écran** : Réglages → Comptabilité expose la borne et ses deux gestes, le déverrouillage
    sous confirmation avec motif ; la liste des écritures porte un bandeau quand un verrou
    existe ; le champ date du formulaire porte un `min`.
12. Les libellés d'écran sont dans les **quatre** locales.
13. La date de verrou entre dans l'**export de souveraineté** (`serialize_companies_csv`) —
    ⛔ sinon une restauration rouvrirait silencieusement une période verrouillée.

## Invariants testables

- **I1 — Rien n'entre sous la borne.** Après une suite complète, aucune écriture de la société
  n'a d'`entry_date` ≤ `books_locked_through` **et** un `created_at` postérieur à la pose du
  verrou. C'est l'invariant qui prouve que la garde tient sur **tous** les chemins, et non
  seulement sur celui qu'on a testé.
- **I2 — Le verrou n'enferme pas.** Pour toute écriture d'une période verrouillée que la 24-4a
  déclare `reversable`, `POST /{id}/reverse` rend **201**.
- **I3 — La borne ne recule pas toute seule.** Aucune opération autre que le déverrouillage
  explicite (Admin + motif) ne diminue `books_locked_through` — ni la clôture, ni la
  réouverture, ni l'import de sauvegarde.

## Tasks / Subtasks

- [ ] **T1 — La migration et ses garde-fous** (AC 1)
  - [ ] `books_locked_through DATE NULL` sur `companies` — `ADD COLUMN` nullable, donc **non breaking** (P1) : ni bump `min_required`, ni bump Cargo
  - [ ] ligne dans `crates/kesh-db/migrations.sha384` (P8)
  - [ ] ligne + les **cinq** compteurs de `docs/migrations-idempotence-audit.md`, recomptés depuis la source (P5)
  - [ ] `crates/kesh-db/test-schema/0001_schema_squash.sql` aligné
  - [ ] ⛔ **DEUX nombres à bumper, pas un** (P6) : `assert_eq!(total, 64)` → `65` (`migrations_upgrade_path.rs:96`) **ET** le `N` de `total - N` de `30` → `31`, pour que la **frontière reste à 34**. Le fichier le dit lui-même (`:108-115`) : bumper le total seul élargirait la fenêtre d'upgrade en silence
  - [ ] contrôle P7 : DDL pur, aucune donnée écrite → ni registre ni exemption
- [ ] **T2 — La garde, au point de passage unique** (AC 2, 4, 9)
  - [ ] `DbError::PeriodLocked { locked_through, attempted }` → **400** `PERIOD_LOCKED`, message nommant les deux dates
  - [ ] contrôle dans `create_in_tx`, **après** les gardes d'exercice (D4)
  - [ ] ⛔ vérifier au sol que les 22 chemins de création y passent bien — `grep -rn "journal_entries::create"`
- [ ] **T3 — Poser et lever** (AC 3, 6, 7, 8)
  - [ ] `companies::lock_books` / `unlock_books`, gabarit `fiscal_years::reopen` (`:779`)
  - [ ] refus d'une borne future ; motif obligatoire au déverrouillage
  - [ ] routes sous `comptable_routes` (pose) et `admin_routes` (levée) ; audit `books.locked` / `books.unlocked`
- [ ] **T4 — L'écran** (AC 11, 12)
  - [ ] Réglages → Comptabilité ; bandeau sur la liste ; `min` sur le champ date
  - [ ] clés dans les **quatre** locales, `data-testid` (jamais un libellé traduit — KF-043)
- [ ] **T5 — L'export** (AC 13)
  - [ ] `books_locked_through` dans **`serialize_company_csv`** (`csv_tables.rs:127` — au singulier, contrairement aux autres), **avec son test** — ⛔ la 24-4a a montré que cet export perd une colonne **en silence**
- [ ] **T6 — Les tests**
  - [ ] la garde sur le journal manuel, **la validation de facture** et **le rapprochement**
  - [ ] la contre-passation qui aboutit (AC 5) ; la précédence (AC 9) ; I1, I2, I3
- [ ] **T7 — La doc**
  - [ ] manuel **utilisateur** : le verrou, ses deux gestes, et ce qu'il ne remplace pas
  - [ ] manuel **admin** : ⛔ la section *Conformité OLICo Art. 9* — le verrou de période **renforce** l'argument, comme le gel l'a fait ; ne pas le sous-déclarer
  - [ ] README : la feuille de route v0.12.0
- [ ] **T8 — Les gates** (⛔ complets, ciblage interdit — migration **et** repository)
  - [ ] base remise à zéro (KF-039), puis `scripts/test-fast.sh`
  - [ ] `npm run check` / `lint-i18n-ownership` / `test:unit` / `build`
  - [ ] suite Playwright complète — ⚠️ **reconstruire `kesh_e2e`**, pas seulement la migrer (leçon 24-4b) ; comparer à la baseline de `docs/testing.md`, dont le compte **dépend de l'heure** (KF-045 #421)

## Hors périmètre

- **Le décompte TVA comme objet** (déposé, daté, opposable) : il n'existe pas, et cette story ne
  l'invente pas. Quand il existera, il **proposera** la date de verrou (D1).
- **Un verrou par journal ou par compte** : la borne est company-wide.
- **La suppression d'une facture validée**, qui détruit son écriture — issue ouverte à la 24-4b.
- **#381**, les trous de numérotation.

## Dev Notes

### Ce que la 24-4b a changé au périmètre de celle-ci

⛔ **Relire le § « Le défaut a CHANGÉ de nature » avant toute chose.** La note de planification
du 2026-08-28 décrivait un défaut de **réécriture** ; la 24-4b l'a fermé. Ce qui reste est
l'**antidatage**, et c'est un autre mécanisme : il ne se garde pas au même endroit
(`create_in_tx` et non `update`), ni avec le même statut (400 et non 409), ni contre le même
geste. *Une story qui hérite d'une note de planification doit vérifier que la note dit encore
vrai.*

### Fichiers à toucher

| fichier | nature |
|---|---|
| `crates/kesh-db/migrations/<version>_companies_books_lock.sql` | NEW |
| `crates/kesh-db/migrations.sha384` · `test-schema/0001_schema_squash.sql` | UPDATE |
| `crates/kesh-db/src/entities/company.rs` | UPDATE — `books_locked_through: Option<NaiveDate>` |
| `crates/kesh-db/src/repositories/companies.rs` | UPDATE — `lock_books` / `unlock_books` |
| `crates/kesh-db/src/repositories/journal_entries.rs` | UPDATE — la garde dans `create_in_tx` |
| `crates/kesh-db/src/errors.rs` · `crates/kesh-api/src/errors.rs` | UPDATE — `PeriodLocked` → 400 |
| `crates/kesh-api/src/routes/companies.rs` · `lib.rs` | UPDATE — les deux routes |
| `crates/kesh-api/src/exports/csv_tables.rs` | UPDATE — ⛔ `serialize_company_csv` (`:127`, **au singulier**), la colonne **et son test** |
| `crates/kesh-i18n/locales/{fr,de,en,it}-CH/messages.ftl` | UPDATE |
| `frontend/src/routes/(app)/settings/…` · `journal-entries/+page.svelte` · `JournalEntryForm.svelte` | UPDATE |
| `crates/kesh-api/tests/period_lock_e2e.rs` | NEW |
| `docs/manual/fr/{user,admin}-manual.tex` (+ PDF) · `README.md` | UPDATE |

### Pièges vérifiés au sol

- ⚠️ **`create_in_tx` ne connaît pas la société directement** : elle la lit dans
  `new.company_id`. La borne se charge donc dans la même transaction, sous le verrou de
  `companies` déjà pris par le lock ordering (`companies → projects → fiscal_years`,
  Pattern 5) — ⛔ **ne pas prendre un verrou dans un autre ordre**, sous peine d'ABBA
  inter-flux.
- ⚠️ **Le seuil est INCLUSIF** : `entry_date <= books_locked_through` est refusé. Une borne au
  31.03 verrouille le 31.03. C'est ce que « jusqu'au 31 mars inclus » veut dire, et l'écart d'un
  jour est exactement le genre de défaut qu'aucun test ne rattrape s'il n'est pas nommé ici.
- ⚠️ **L'écriture d'ouverture est datée du premier jour du premier exercice** : poser une borne
  postérieure la rendrait irrémédiablement inatteignable. C'est cohérent — on ne verrouille pas
  avant d'avoir saisi ses soldes de départ —, mais le message de refus doit rester
  compréhensible pour qui le rencontre à la mise en route.
- ⚠️ **`serialize_company_csv` énumère ses dix colonnes à la main** (`csv_tables.rs:127-140`),
  comme `serialize_journal_entries_csv` : aucun test n'impose leur exhaustivité vis-à-vis du
  schéma (`backup_inventory_matches_schema` ne compare que la liste des **tables**). ⛔ La 24-4a
  a payé ce piège ; ne pas le repayer. ⚠️ Noter le **singulier** — c'est la seule des seize
  fonctions d'export à le porter, et un `grep serialize_companies_csv` rend zéro.
- ⛔ **Le compte de migrations passe de 64 à 65, et il vit à QUATRE endroits** : les **deux**
  sites du total de `docs/migrations-idempotence-audit.md` (l'en-tête de section et la ligne
  `Total`), sa **partition** (`yes` / `tracked-by-sqlx` / `no`, dont la somme doit valoir le
  total), et l'assertion de `migrations_upgrade_path.rs:96`. ⚠️ Les compteurs de partition ne
  valent **pas** le total — les aligner dessus casserait l'invariant qu'ils servent à tenir.

### Références

- `journal_entries.rs:266-274` (la seule garde de date aujourd'hui) · `:187` (`create_in_tx`)
- `fiscal_years.rs:779` (`reopen` — le gabarit : motif, `FOR UPDATE` scopé, audit)
- `kesh-report/src/vat_report.rs:83` · `period.rs:16` (`ReportPeriod` — deux dates, sans persistance)
- Stories **24-4a** et **24-4b** — la contre-passation datée du jour, et le gel
- Issue **#380** ; Epic `_bmad-output/planning-artifacts/epic-24-vague1-livres-justes.md`
- `CLAUDE.md` §§ *Migration breaking policy* (P1, P5-P8), *Review Iteration Rule*, *Test Locally First*

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Journal de revue
