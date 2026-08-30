# Story 24.4c : Le verrou de période — arrêter les livres à une date

## Status

review

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

`create_in_tx_inner` ne contrôle qu'une chose sur la date (`journal_entries.rs:273`) :

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

## D2 — Le refus vit dans `create_in_tx_inner`, le point de passage unique

⛔ **Douze sites de production créent une écriture** — facture validée, avoir, facture
fournisseur ×3, règlement, rapprochement ×5, journal manuel.

⚠️ **Le grep brut rend 22 lignes, et DIX d'entre elles ne créent rien** : neuf doc-comments,
plus un message `tracing::error!` (`trial_balance.rs:125`) qui cite le nom de la fonction sans
l'appeler. Le tri `| grep -vE ":\s*(///|//!|//)"` en écarte neuf et **laisse le dixième** — il
se retire à la main. *Un implémenteur qui compte les lignes du grep sans les trier croira avoir
vérifié vingt-deux chemins dont dix n'existent pas.*

⛔ **LE POINT DE PASSAGE UNIQUE EST `create_in_tx_inner`, PAS `create_in_tx`.** Vérifié :
`create_in_tx` (`:187`) n'est qu'un mince wrapper qui appelle `create_in_tx_inner` (`:194`) ;
`create_opening_entry` passe par le wrapper, mais **`reverse` appelle `create_in_tx_inner`
DIRECTEMENT** (`:1385`). Poser la garde dans le wrapper laisserait donc un second point d'entrée
l'éviter — celui que la 24-4a a ajouté il y a deux jours.

⚠️ **Et il ne faut PAS que la contre-passation évite la garde : il faut qu'elle la FRANCHISSE.**
Aujourd'hui elle passerait dans les deux cas — sa date est celle du jour, et l'AC 3 impose une
borne strictement passée. Mais « elle passe parce qu'elle contourne » et « elle passe parce que
sa date est valide » ne sont pas la même propriété : la première se casse au premier refactor,
la seconde tient par construction. ⇒ **garde dans `create_in_tx_inner`, et un test qui vérifie
que la contre-passation la traverse.**

⇒ Sur le modèle exact d'`enforce_postable` : **une seule ligne à écrire, et aucun chemin ne peut
la contourner**. Une garde posée aux routes en laisserait onze ouverts.

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

⛔ **La séparation par RÔLE ne suffit pas : il faut une garde par VALEUR, sinon le verbe
« avancer » suffit à reculer.** Rien, dans une distinction de rôles, n'empêche un Comptable
d'appeler l'endpoint de pose avec une date **antérieure** à la borne courante. La borne
reculerait alors sans motif, sans rôle Admin, et le journal d'audit écrirait `books.locked` — un
retrait **maquillé en pose**, c'est-à-dire précisément le mode d'échec silencieux que cette story
existe pour fermer, reproduit dans son propre mécanisme de garde.

⇒ **`lock_books` refuse toute date `<=` à la borne courante non nulle**, en 400. Avancer veut
dire avancer.

⚠️ **L'asymétrie est le cœur de la mesure.** Verrouiller est un geste d'hygiène, qu'on doit
pouvoir faire souvent et sans cérémonie. **Déverrouiller défait une garantie** : c'est ce geste
qui doit coûter, se justifier et se retrouver dans le journal d'audit — exactement comme la
réouverture d'un exercice clôturé.

⛔ **Ne pas confondre avec la clôture annuelle.** Les deux verrous coexistent et ne se
remplacent pas : la clôture est un **état d'exercice** avec ses règles propres (LIFO, report
à-nouveau, réouverture Admin) ; le verrou de période est une **borne mobile** à l'intérieur d'un
exercice ouvert. Un exercice clos reste clos quelle que soit la borne.

## D4 — La précédence des refus, figée ici

Sur une création, l'ordre **réel du code** est :

`NoFiscalYear` (400 `NO_FISCAL_YEAR`, au handler) → `FiscalYearClosed` (400) →
`DateOutsideFiscalYear` (400) → **`PeriodLocked`** (400)

⚠️ **Cet ordre a été vérifié au sol, et il n'est pas celui qu'on suppose** : le `match` de
`create_in_tx_inner` rend `FiscalYearClosed` en `:263`, **avant** le contrôle de bornes en
`:274`. Une première rédaction de cette spec avait écrit la flèche dans l'autre sens.

⛔ **`DATE_OUTSIDE_FISCAL_YEAR` est INATTEIGNABLE par la route de saisie manuelle.** Le handler
résout d'abord l'exercice couvrant la date (`routes/journal_entries.rs:512-524`) : une date qui
ne tombe dans aucun exercice rend **`NO_FISCAL_YEAR`**, jamais `DATE_OUTSIDE_FISCAL_YEAR` — ce
dernier est la garde **défensive** de `create_in_tx_inner`, pour les appelants internes qui
passent un `fiscal_year_id` explicite. *Un test d'AC écrit sans cette distinction obtiendra
`NO_FISCAL_YEAR` et sera « ajusté » jusqu'à passer.*

⚠️ **Le verrou parle en DERNIER, et c'est délibéré** : une date hors exercice ou dans un
exercice clos est un problème plus grave et plus ancien, dont le message existe déjà et renvoie
ailleurs. Dire « période verrouillée » à quelqu'un qui s'est trompé d'exercice l'enverrait
corriger la mauvaise chose.

⚠️ **Les DEUX PREMIERS refus sont mutuellement exclusifs, et aucun test combiné n'est
possible entre eux.** Le `match` de `create_in_tx_inner` (`journal_entries.rs:261-277`) rend
`FiscalYearClosed` **dans son bras de garde**, avant d'avoir regardé la moindre date : un
exercice clos n'atteint jamais le contrôle de bornes. ⛔ Ne pas chercher à reproduire ici le
test « les deux causes à la fois » de la 24-4b — il n'est constructible que pour les couples
faisant intervenir `PeriodLocked` (AC 9).

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
3. La borne est **strictement antérieure à aujourd'hui** (`< today`) : poser une date future
   **ou celle du jour** est refusé en **400**.
   ⛔ **« Aujourd'hui » est `Utc::now().date_naive()`, la MÊME horloge que la contre-passation.**
   Sans cette précision, une pose faite en Suisse entre minuit et 2 h locales — où la date locale
   vaut la date UTC + 1 — serait refusée alors qu'elle vise la veille. *L'écart d'un jour est
   exactement ce que cette AC corrige ; le laisser implicite le réintroduirait sous une autre
   forme.*
   ⛔ **Le « ou celle du jour » n'est pas un excès de prudence, c'est la correction d'un défaut
   réel.** La contre-passation est datée du **jour** (`journal_entries.rs:1371`,
   `Utc::now().date_naive()`) et le seuil de l'AC 2 est **inclusif** : une borne posée à la date
   du jour refuserait donc toute contre-passation faite le même jour, en violation directe de
   l'AC 5 et de l'invariant I2. ⚠️ Et le test de l'AC 5, s'il est écrit avec une borne
   franchement passée, **ne le verrait pas** — le défaut n'apparaîtrait qu'en production, le jour
   où un administrateur verrouille « jusqu'à aujourd'hui » après une clôture.
4. Le refus vaut pour **tous les chemins de création**, pas seulement le journal manuel : un
   test l'exerce sur la **validation d'une facture** antidatée et sur un **rapprochement
   bancaire** antidaté, et le message nomme la pièce.
5. La **contre-passation d'une écriture d'une période verrouillée aboutit** (201) — elle est
   datée du jour, donc après la borne. ⛔ Un test le verrouille : c'est la propriété qui empêche
   les livres de devenir incorrigibles, et rien d'autre ne la protège.
6. Avancer la borne est permis à **Admin et Comptable** ; la reculer ou la retirer est réservé à
   **Admin** et exige un **motif non vide** (blancs seuls refusés), sinon **400**.
   ⛔ **`lock_books` refuse toute date `<=` à la borne courante NON NULLE** (400) : sans cette
   garde de **valeur**, la garde de **rôle** est contournable — un Comptable reculerait la borne
   par l'endpoint d'avancement, sans motif et sous une entrée d'audit `books.locked` mensongère.
   ⚠️ **Le « non nulle » n'est pas décoratif** : à la première pose la borne vaut `NULL`, et
   c'est le seul cas où cette garde doit se taire. Un test dédié l'exerce **avec un jeton
   Comptable**.
7. Consultation reçoit **403** sur les deux gestes.
8. Chaque pose écrit une entrée d'audit `books.locked`, chaque retrait une entrée
   `books.unlocked` portant le **motif** et l'**ancienne** valeur.
   ⛔ **`books.unlocked` a UN SEUL producteur** : le déverrouillage délibéré par un Admin. La
   restauration de sauvegarde écrit **`books.restored`** (AC 14) — confondre les deux rendrait
   le filtre d'audit inutilisable pour le réviseur qui cherche qui a déverrouillé.
9. La précédence est testée **par la route** : une date qui ne tombe dans **aucun exercice**,
   période verrouillée ou non, rend **`NO_FISCAL_YEAR`** ; une date d'un **exercice clos** rend
   **`FISCAL_YEAR_CLOSED`** — jamais `PERIOD_LOCKED` dans les deux cas.
   ⚠️ **Ne pas écrire de test attendant `DATE_OUTSIDE_FISCAL_YEAR` par la route** : il est
   inatteignable par ce chemin (cf. D4).
10. La clôture annuelle et le verrou de période sont **indépendants** : clôturer un exercice ne
    touche pas la borne, et la borne n'empêche ni la clôture ni la réouverture.
11. **Écran** : Réglages → Comptabilité expose la borne et ses deux gestes, le déverrouillage
    sous confirmation avec motif ; la liste des écritures porte un bandeau quand un verrou
    existe ; le champ date du formulaire porte un `min`.
12. Les libellés d'écran sont dans les **quatre** locales.
13. La date de verrou entre dans l'**export de souveraineté** — `serialize_company_csv`
    (`csv_tables.rs:127`, **au singulier** : c'est la seule des **dix-neuf** fonctions d'export à le
    porter, et un `grep serialize_companies_csv` rend zéro). ⚠️ **Cet export n'a AUCUN
    importeur dans le dépôt** : la colonne y va parce qu'un export de souveraineté doit être
    complet, **pas** parce qu'elle protégerait d'une restauration — cf. AC 14, qui traite le
    vrai chemin.
14. ⛔ **L'import d'une sauvegarde `.keshbackup` peut faire RECULER la borne, et cela doit se
    voir.** `companies` figure dans `TABLES_TO_TRUNCATE` (`crates/kesh-db/src/backup.rs`) et
    l'import la vide puis la ré-insère depuis l'archive, colonne par colonne selon le schéma :
    `books_locked_through` y voyage donc tout seul, et une archive antérieure à la pose du
    verrou restaure une borne plus ancienne ou `NULL`.
    ⇒ **Ce n'est pas un défaut à interdire, c'est un geste à tracer.** Un `.keshbackup` restaure
    l'installation **entière** : si les livres reviennent à l'état de l'archive, il est cohérent
    que la borne les suive — refuser produirait une installation dont le verrou ne correspond
    plus aux écritures. Ce qui serait grave, c'est que la restauration devienne un
    **déverrouillage silencieux**. ⇒ l'import écrit une entrée d'audit **`books.restored`**
    portant l'ancienne et la nouvelle valeur **dès que la borne recule**.
    ⛔ **L'ancienne valeur se relève AVANT le restore, et c'est le point le plus facile à
    manquer.** `companies` figure dans `TABLES_TO_TRUNCATE` (`backup.rs:76`) et
    `restore_tables_in_tx` la **vide** avant de la ré-insérer ; à l'endroit où l'entrée d'audit
    survit — après le restore, dans la même transaction, à côté de `admin.full_import` —
    l'ancienne borne **n'existe plus nulle part**. Elle se lit donc avant, sous le `FOR UPDATE`
    déjà pris, et se compare après. *Écrite naïvement, la trace rapporterait l'ancienne valeur
    comme étant la nouvelle : une trace qui ne trace rien, sur le chemin même qu'elle existe
    pour couvrir.*
    ⛔ **Et l'action est `books.restored`, PAS `books.unlocked`.** Il n'existe aucun acteur
    « système » — `ActorType` ne connaît que `User` et `ApiKey` — et l'audit d'import est
    attribué au `MIN(id)` des admins **du dataset restauré**, un administrateur qui n'a rien
    déverrouillé. Réutiliser `books.unlocked` mêlerait, dans le filtre d'un réviseur, les
    déverrouillages délibérés et les restaurations, sous la signature d'un innocent.
    ⚠️ **Cas à trancher à l'implémentation** : une archive venue d'une **autre installation**
    n'a pas de `companies.id` comparable. La comparaison porte alors sur la société courante
    unique ; si l'inventaire ne permet pas de l'apparier, `books.restored` est écrite **sans
    ancienne valeur**, avec la mention explicite.

## Invariants testables

- **I1 — Rien n'entre sous la borne.** Après une suite complète, aucune écriture de la société
  n'a d'`entry_date` ≤ `books_locked_through` **et** un `created_at` postérieur à la pose du
  verrou. C'est l'invariant qui prouve que la garde tient sur **tous** les chemins, et non
  seulement sur celui qu'on a testé.
- **I2 — Le verrou n'enferme pas.** Pour toute écriture d'une période verrouillée que la 24-4a
  déclare `reversable`, `POST /{id}/reverse` rend **201**.
- **I3 — La borne ne recule pas SANS TRACE.** Aucune opération ne diminue
  `books_locked_through` sans écrire une entrée d'audit : ni la clôture, ni la réouverture, ni
  la pose (qui la refuse, AC 6), ni l'import de sauvegarde (qui la trace, AC 14).
  ⚠️ **Un cinquième chemin existe, ATTÉNUÉ et non absent** : `reset_demo` fait
  `DELETE FROM companies` (`kesh-seed/src/lib.rs`). Il est hors d'atteinte d'une installation
  susceptible de porter une borne — le handler refuse dès `step_completed >= 7` et hors mode
  démo au-delà de l'étape 2 (`onboarding.rs:256-278`) — mais un invariant qui se dit universel
  doit **nommer** le cas et sa garde. Suivi par #377 et #279.

  ⚠️ **La première rédaction disait « ne recule pas toute seule » et prétendait que l'import
  était couvert** — il ne l'était pas, et rien dans la story ne l'en empêchait. Un invariant qui
  énonce l'objectif ne démontre pas qu'il est atteint : c'est l'AC qui doit le faire.

## Tasks / Subtasks

- [x] **T1 — La migration et ses garde-fous** (AC 1)
  - [x] `books_locked_through DATE NULL` sur `companies` — `ADD COLUMN` nullable, donc **non breaking** (P1) : ni bump `min_required`, ni bump Cargo
  - [x] ligne dans `crates/kesh-db/migrations.sha384` (P8)
  - [x] ligne + les **cinq** compteurs de `docs/migrations-idempotence-audit.md`, recomptés depuis la source (P5)
  - [x] `crates/kesh-db/test-schema/0001_schema_squash.sql` aligné
  - [x] ⛔ **DEUX nombres à bumper, pas un** (P6) : `assert_eq!(total, 64)` → `65` (`migrations_upgrade_path.rs:96`) **ET** le `N` de `total - N` de `30` → `31`, pour que la **frontière reste à 34**. Le fichier le dit lui-même (`:108-115`) : bumper le total seul élargirait la fenêtre d'upgrade en silence
  - [x] contrôle P7 : DDL pur, aucune donnée écrite → ni registre ni exemption
- [x] **T2 — La garde, au point de passage unique** (AC 2, 4, 9)
  - [x] `DbError::PeriodLocked { locked_through, attempted }` → **400** `PERIOD_LOCKED`, message nommant les deux dates
  - [x] ⛔ contrôle dans **`create_in_tx_inner`** (`:226`) et non dans le wrapper `create_in_tx` — `reverse` appelle l'inner directement (`:1385`)
  - [x] contrôle placé **après** les gardes d'exercice, dans l'ordre réel du code (D4)
  - [x] ⛔ vérifier au sol les **douze** chemins de production — puis **trier** : dix des vingt-deux lignes du grep ne créent rien
- [x] **T3 — Poser et lever** (AC 3, 6, 7, 8)
  - [x] `companies::lock_books` / `unlock_books`, gabarit `fiscal_years::reopen` (`:779`)
  - [x] refus d'une borne **future ou du jour** (AC 3) ; motif obligatoire et non blanc au déverrouillage
  - [x] ⛔ **`lock_books` refuse une date `<=` à la borne courante** — la garde de valeur sans laquelle la garde de rôle est contournable (AC 6)
  - [x] routes sous `comptable_routes` (pose) et `admin_routes` (levée) ; audit `books.locked` / `books.unlocked`
- [x] **T4 — L'écran** (AC 11, 12)
  - [x] Réglages → Comptabilité ; bandeau sur la liste ; `min` sur le champ date
  - [x] clés dans les **quatre** locales, `data-testid` (jamais un libellé traduit — KF-043)
- [x] **T5 — L'export et la RESTAURATION** (AC 13, 14)
  - [x] ⛔ **le vrai chemin** : l'import `.keshbackup` écrit **`books.restored`** — et **jamais** `books.unlocked`, qui a un seul producteur (AC 8) — dès que la borne recule ; `companies` est dans `TABLES_TO_TRUNCATE`, la colonne y voyage toute seule
  - [x] `books_locked_through` dans **`serialize_company_csv`** (`csv_tables.rs:127` — au singulier, contrairement aux autres), **avec son test** — ⛔ la 24-4a a montré que cet export perd une colonne **en silence**
- [x] **T6 — Les tests**
  - [x] la garde sur le journal manuel, **la validation de facture** et **le rapprochement**
  - [x] ⛔ **poser une borne à la date du jour rend 400** (AC 3) — la garde neuve que rien n'exercerait sinon
  - [x] ⛔ **le test de l'AC 5 pose la borne À LA VEILLE**, valeur maximale admise — écrit avec une borne franchement passée, il ne verrait pas le défaut d'un jour
  - [x] ⛔ **un jeton Comptable ne peut pas reculer la borne** par l'endpoint d'avancement (AC 6), et la garde se tait quand la borne est `NULL`
  - [x] ⛔ **l'import qui fait reculer la borne écrit `books.restored`** avec l'ANCIENNE valeur (AC 14)
  - [x] la contre-passation qui aboutit (AC 5) **en traversant la garde** ; la précédence (AC 9) ; I1, I2, I3
- [x] **T7 — La doc**
  - [x] manuel **utilisateur** : le verrou, ses deux gestes, et ce qu'il ne remplace pas
  - [x] manuel **admin** : ⛔ la section *Conformité OLICo Art. 9* — le verrou de période **renforce** l'argument, comme le gel l'a fait ; ne pas le sous-déclarer
  - [x] README : la feuille de route v0.12.0
- [x] **T8 — Les gates** (⛔ complets, ciblage interdit — migration **et** repository)
  - [x] base remise à zéro (KF-039), puis `scripts/test-fast.sh`
  - [x] `npm run check` / `lint-i18n-ownership` / `test:unit` / `build`
  - [x] suite Playwright complète — ⚠️ **reconstruire `kesh_e2e`**, pas seulement la migrer (leçon 24-4b) ; comparer à la baseline de `docs/testing.md`, dont le compte **dépend de l'heure** (KF-045 #421)

## Hors périmètre

- **Le décompte TVA comme objet** (déposé, daté, opposable) : il n'existe pas, et cette story ne
  l'invente pas. Quand il existera, il **proposera** la date de verrou (D1).
- **Un verrou par journal ou par compte** : la borne est company-wide.
- **La suppression d'une facture validée**, qui détruit son écriture — issue ouverte à la 24-4b.
- **#381**, les trous de numérotation.

## Dev Notes

### Règle de splitting — examinée, non déclenchée

La story touche cinq zones (`kesh-db`, `kesh-api`, `kesh-i18n`, `frontend`, `docs/`) pour **un
seul mécanisme** : une borne, une garde, deux gestes. Le seuil de la § *Règle de splitting
préventif* — « plus de 5 modules **distincts** » — n'est pas franchi. ⚠️ Le second critère, la
non-convergence de sévérité, ne peut s'évaluer qu'à partir de la passe 2 ; c'est lui, et non le
décompte de zones, qui décidera d'un split si la sévérité cesse de reculer.

### Ce que la 24-4b a changé au périmètre de celle-ci

⛔ **Relire le § « Le défaut a CHANGÉ de nature » avant toute chose.** La note de planification
du 2026-08-28 décrivait un défaut de **réécriture** ; la 24-4b l'a fermé. Ce qui reste est
l'**antidatage**, et c'est un autre mécanisme : il ne se garde pas au même endroit
(`create_in_tx_inner` et non `update`), ni avec le même statut (400 et non 409), ni contre le même
geste. *Une story qui hérite d'une note de planification doit vérifier que la note dit encore
vrai.*

### Fichiers à toucher

| fichier | nature |
|---|---|
| `crates/kesh-db/migrations/<version>_companies_books_lock.sql` | NEW |
| `crates/kesh-db/migrations.sha384` · `test-schema/0001_schema_squash.sql` | UPDATE |
| `crates/kesh-db/src/entities/company.rs` | UPDATE — `books_locked_through: Option<NaiveDate>` |
| `crates/kesh-db/src/repositories/companies.rs` | UPDATE — `lock_books` / `unlock_books` |
| `crates/kesh-db/src/repositories/journal_entries.rs` | UPDATE — la garde dans **`create_in_tx_inner`** (`:226`), PAS dans le wrapper `create_in_tx` |
| `crates/kesh-db/src/errors.rs` · `crates/kesh-api/src/errors.rs` | UPDATE — `PeriodLocked` → 400 |
| `crates/kesh-api/src/routes/companies.rs` · `lib.rs` | UPDATE — les deux routes |
| `crates/kesh-api/src/exports/csv_tables.rs` | UPDATE — ⛔ `serialize_company_csv` (`:127`, **au singulier**), la colonne **et son test** |
| `crates/kesh-i18n/locales/{fr,de,en,it}-CH/messages.ftl` | UPDATE |
| `frontend/src/routes/(app)/settings/…` · `journal-entries/+page.svelte` · `JournalEntryForm.svelte` | UPDATE |
| `crates/kesh-api/src/routes/admin.rs` | UPDATE — ⛔ l'AC 14 : relever la borne avant le restore, écrire `books.restored` après |
| `crates/kesh-api/tests/admin_full_import_e2e.rs` | UPDATE — le test de l'AC 14 |
| `crates/kesh-api/tests/period_lock_e2e.rs` | NEW |
| `docs/manual/fr/{user,admin}-manual.tex` (+ PDF) · `README.md` | UPDATE |

### Pièges vérifiés au sol

- ⚠️ **`create_in_tx_inner` ne connaît pas la société directement** : elle la lit dans
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
  a payé ce piège ; ne pas le repayer. ⚠️ Noter le **singulier** — c'est la seule des **dix-neuf**
  fonctions d'export à le porter, et un `grep serialize_companies_csv` rend zéro.
- ⛔ **Le compte de migrations passe de 64 à 65, et il vit à QUATRE endroits** : les **deux**
  sites du total de `docs/migrations-idempotence-audit.md` (l'en-tête de section et la ligne
  `Total`), sa **partition** (`yes` / `tracked-by-sqlx` / `no`, dont la somme doit valoir le
  total), et l'assertion de `migrations_upgrade_path.rs:96`. ⚠️ Les compteurs de partition ne
  valent **pas** le total — les aligner dessus casserait l'invariant qu'ils servent à tenir.

### Références

- `journal_entries.rs:266-274` (la seule garde de date aujourd'hui) · `:187` (`create_in_tx`, le wrapper) · `:226` (`create_in_tx_inner`, **le point de passage réel**) · `:1385` (`reverse`, qui appelle l'inner directement)
- `fiscal_years.rs:779` (`reopen` — le gabarit : motif, `FOR UPDATE` scopé, audit)
- `kesh-report/src/vat_report.rs:83` · `period.rs:16` (`ReportPeriod` — deux dates, sans persistance)
- Stories **24-4a** et **24-4b** — la contre-passation datée du jour, et le gel
- Issue **#380** ; Epic `_bmad-output/planning-artifacts/epic-24-vague1-livres-justes.md`
- `CLAUDE.md` §§ *Migration breaking policy* (P1, P5-P8), *Review Iteration Rule*, *Test Locally First*

## Dev Agent Record

### Agent Model Used

Opus 5 (`claude-opus-5[1m]`) — implémentation du 2026-08-30.

### Completion Notes List

⛔ **Ce que le GATE a trouvé et que quatre passes de revue de spec n'avaient pas vu.** Les
listes de colonnes de `companies` **écrites à la main** existent dans **quatre** endroits, pas
un : la spec n'avait nommé que `serialize_company_csv`, alors que `routes/onboarding.rs` en
portait **deux** et `kesh-seed/src/lib.rs` **une**. Les trois manquantes faisaient échouer
l'onboarding en **500**, et **seule l'exécution réelle les a révélées** — c'est exactement ce
que les garde-fous P6 et P7 disent de ce mode d'échec : *il ne naît ni du code écrit ni de la
spec, mais de l'interaction avec ce que la story ne touche pas.*

⚠️ **Deux garde-fous hors périmètre ont parlé, et c'était leur travail.**
`admin_pat_denied_e2e` tient la liste des couples de routes Admin **et** leur répartition entre
les deux gardes ; ajouter la route de déverrouillage a fait rougir les deux assertions. La
seconde (`(5, 20)` → `(5, 21)`) est celle qu'on aurait pu « ajuster » sans réfléchir : elle dit
que le déverrouillage est arrêté par le **gate de portée** de la clé et non par la couche de
rôle, ce qui est le comportement voulu.

⛔ **J'ai réintroduit un défaut que j'avais corrigé quatre fois plus tôt le même jour** : le
`\og … \fg{}` de LaTeX, **indéfini dans ce manuel**, qui fait échouer la compilation. C'est
le symptôme même que les passes 1 à 3 de cette story ont nommé — *corriger au site et laisser
le geste se reproduire ailleurs*. Deux titres d'encadré ont aussi dû être protégés par des
accolades : une virgule dans un `[title=…]` casse l'analyse `pgfkeys`.

⚠️ **La borne se lit tôt et s'évalue tard**, et il faut le comprendre pour ne pas le
« simplifier » : l'ordre des **verrous** (companies en premier, Pattern 5) et l'ordre des
**refus** (le verrou parle en dernier) ne sont pas le même. Lire la borne après le lock
`fiscal_years` créerait une inversion ABBA ; l'évaluer avant les gardes d'exercice enverrait
l'utilisateur corriger la mauvaise chose.

**Décomptes, recomptés depuis la source** (périmètre : de `main` au commit d'implémentation) :

| grandeur | avant | après | écart |
|---|---|---|---|
| tests backend (`test-fast.sh`) | 2258 | **2270** | +11 (`period_lock_e2e`) +1 (export) |
| migrations | 64 | **65** | +1, et les **cinq** compteurs de l'audit recomptés (partition 5+60+0 = 65) |
| `assert_eq!(total, N)` · fenêtre `total - N` | 64 · 30 | **65 · 31** | frontière tenue à **34** |
| clés par catalogue (les quatre) | 1678 | **1687** | +9 |
| `ATTENDU.sitesTotal` | 1619 | **1630** | +11 — neuf clés, onze sites |
| tests frontend (`test:unit`) | 740 | **740** | inchangé |

### Gates réellement exécutés

| gate | résultat |
|---|---|
| `cargo fmt --all -- --check` | propre |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 warning |
| `scripts/test-fast.sh`, base remise à zéro (KF-039) | **2270/2270**, 4 skipped |
| `npm run check` · `lint-i18n-ownership` · `test:unit` · `build` | 0 erreur · PASS · **740/740** · OK |
| Playwright, suite **complète**, `kesh_e2e` **reconstruite** | **214 passés / 9 échoués — ZÉRO RÉGRESSION** |

**Les neuf échecs, chacun tranché** : 7 de la KF-029 (#97), 1 de pollution
(`reminders.spec.ts:146`, verte rejouée seule), et 1 de la **KF-046 (#424)**, ouverte à cette
occasion.

⛔ **La KF-046 a été mesurée sur `main`, pas déduite** : frontend de `main` reconstruit en
worktree (`npm ci && npm run build`), servi par `KESH_STATIC_DIR`, base reconstruite —
**l'échec est identique**. Ce n'est pas une régression. ⚠️ Elle passait pourtant seule le matin
même : un test qui dépend de la **route d'atterrissage** est fragile comme celui de la KF-045
dépend de l'heure.

✅ **Et la KF-045 ne s'est PAS déclenchée sur ce run** — il était 14:13 UTC, donc après midi.
C'est la confirmation empirique de son diagnostic horaire.

### File List

| fichier | nature |
|---|---|
| `crates/kesh-db/migrations/20260830000001_companies_books_lock.sql` | NEW |
| `crates/kesh-db/migrations.sha384` · `test-schema/0001_schema_squash.sql` | UPDATE |
| `crates/kesh-db/tests/migrations_upgrade_path.rs` | UPDATE — P6, **deux** nombres (`total` et `N`) |
| `docs/migrations-idempotence-audit.md` | UPDATE — P5, ligne + cinq compteurs |
| `crates/kesh-db/src/entities/company.rs` · `errors.rs` | UPDATE — le champ, `DbError::PeriodLocked` |
| `crates/kesh-db/src/repositories/companies.rs` | UPDATE — `lock_books` / `unlock_books` |
| `crates/kesh-db/src/repositories/journal_entries.rs` | UPDATE — la garde dans `create_in_tx_inner` |
| `crates/kesh-api/src/errors.rs` | UPDATE — mappage **400** `PERIOD_LOCKED` |
| `crates/kesh-api/src/routes/companies.rs` · `lib.rs` | UPDATE — les deux routes, RBAC asymétrique |
| `crates/kesh-api/src/routes/admin.rs` | UPDATE — `books.restored`, borne relevée **avant** le restore |
| `crates/kesh-api/src/routes/onboarding.rs` · `crates/kesh-seed/src/lib.rs` | UPDATE — ⛔ **trois** listes de colonnes manuelles, trouvées par le gate |
| `crates/kesh-api/src/exports/csv_tables.rs` | UPDATE — la colonne **et son test** |
| `crates/kesh-api/tests/period_lock_e2e.rs` | NEW — **11 tests** |
| `crates/kesh-api/tests/admin_pat_denied_e2e.rs` | UPDATE — le couple Admin et la répartition |
| `crates/kesh-i18n/locales/{fr,de,en,it}-CH/messages.ftl` | UPDATE — 9 clés × 4 |
| `frontend/src/lib/features/settings/settings.{api,types}.ts` | UPDATE |
| `frontend/src/routes/(app)/settings/+page.svelte` | UPDATE — la section, `msg()` accepte les arguments |
| `frontend/src/routes/(app)/journal-entries/+page.svelte` | UPDATE — le bandeau |
| `frontend/src/lib/features/journal-entries/JournalEntryForm.svelte` | UPDATE — le `min` du champ date |
| `frontend/src/lib/shared/i18n-keys.test.ts` | UPDATE — `sitesTotal` et sa ventilation |
| `docs/manual/fr/{user,admin}-manual.tex` (+ PDF) · `README.md` · `docs/testing.md` | UPDATE |

## Journal de revue

### Passe 1 — 2026-08-30 · Sonnet 4.6 + Haiku 4.5, contextes frais, orthogonales à l'auteur (Opus 5)

**2 CRITICAL, 1 HIGH, 3 MEDIUM, 2 LOW — soit huit, tous de la lentille Sonnet.**

| # | sév. | ce qui manquait |
|---|---|---|
| S1-C1 | CRITICAL | ⛔ **l'invariant I3 énonçait un objectif et rien ne l'implémentait** : `companies` est dans `TABLES_TO_TRUNCATE`, l'import `.keshbackup` la ré-insère depuis l'archive, et `books_locked_through` y voyage tout seul — la borne pouvait reculer sans motif ni audit. Pire, l'AC 13 prétendait couvrir ce risque par une colonne d'export CSV **qui n'a aucun importeur** |
| S1-C2 | CRITICAL | ⛔ **la garde de RÔLE était contournable par le VERBE** : rien n'empêchait un Comptable d'appeler l'endpoint « avancer » avec une date antérieure. La borne reculait sans motif, sous une entrée `books.locked` mensongère |
| S1-H1 | HIGH | ⛔ AC 3 admettait une borne **égale à aujourd'hui**, et le seuil de l'AC 2 est inclusif : toute contre-passation du même jour — datée du jour (`:1371`) — aurait été refusée, en violation de l'AC 5 et de I2. ⚠️ Un test écrit avec une borne franchement passée **ne l'aurait pas vu** |
| S1-M1 | MEDIUM | l'AC 13 citait `serialize_companies_csv`, **que les Dev Notes du même document déclaraient inexistant** |
| S1-M2 | MEDIUM | défaut de priorisation : tout l'effort portait sur l'export jamais réimporté, aucun sur la restauration réelle |
| S1-M3 | MEDIUM | « vingt-deux chemins de création » : le grep rend 22 lignes dont **neuf sont des commentaires**. Treize sites réels |
| S1-L1 | LOW | la précédence `DateOutsideFiscalYear → FiscalYearClosed` n'est **exerçable par aucun test** : le `match` rend `Closed` avant tout contrôle de date |
| S1-L2 | LOW | la règle de splitting n'était pas auto-évaluée |

⚠️ **Divergence assumée sur le remède de C1.** La lentille proposait de **refuser** une
restauration qui ferait reculer la borne. Un `.keshbackup` restaure l'installation **entière** :
si les livres reviennent à l'état de l'archive, il est cohérent que la borne les suive, et
refuser produirait une installation dont le verrou ne correspond plus aux écritures. Ce qui
serait grave, c'est que la restauration devienne un **déverrouillage silencieux** — d'où l'AC 14,
qui la **trace** plutôt que de l'interdire.

⛔ **La lentille Haiku a rendu ZÉRO finding, et c'est instructif.** Sa note de méthode dit avoir
vérifié « par grep ou lecture directe **de la spec** » : elle a donc contrôlé que la spec dit ce
que la spec dit. Sur le cas de l'import, elle a déclaré le cas traité **en citant l'invariant
I3** — c'est-à-dire l'objectif que S1-C1 démontre non atteint. *Citer l'objectif comme preuve
qu'il est atteint est un raisonnement circulaire, et c'est exactement là que le premier CRITICAL
se cachait.*

⚠️ **Le motif de la passe** : les deux CRITICAL portent sur le **mécanisme de garde lui-même**,
non sur ce qu'il protège — un invariant qui s'énonce sans s'implémenter, et une séparation de
rôles contournable par l'autre verbe. La conception d'ensemble (la borne comme date, la garde au
point de passage unique, la précédence) n'a pas été prise en défaut.

**Prochaine** : passe 2, contexte frais, lentille braquée sur le commit de cette remédiation.

### Passe 2 — 2026-08-30 · Opus 5, contexte frais, passe CIBLÉE sur le seul commit `295f4319`

Prompt versionné dans `24-4c-review-prompt-regression-hunter-p2.md`.
**0 CRITICAL, 4 HIGH, 6 MEDIUM, 3 LOW — soit treize, et les TREIZE portent sur le patch de la passe 1.**

| # | sév. | ce qui était faux |
|---|---|---|
| P2-1 | HIGH | ⛔ **« Tous passent par `create_in_tx` » est FAUX** : `reverse` appelle `create_in_tx_inner` **directement** (`:1385`). Le point de passage réel est l'inner, et la garde posée dans le wrapper aurait laissé un second point d'entrée l'éviter — celui que la 24-4a a ajouté il y a deux jours |
| P2-2 | HIGH | « treize sites » en vaut **douze** (le tri prescrit laisse `trial_balance.rs:125`), et le même paragraphe disait « neuf » puis « dix ». **Trois nombres neufs qui ne se recoupent pas, dans un patch qui corrigeait un décompte** |
| P2-3 | HIGH | ⛔ la flèche de D4 était **inversée** : le code rend `FiscalYearClosed` (`:263`) **avant** `DateOutsideFiscalYear` (`:274`). Le patch avait écrit le fait douze lignes plus bas **sans corriger la flèche qu'il réfutait** |
| P2-5 | HIGH | ⛔ l'AC 14 exigeait « l'ancienne valeur » **là où elle n'existe plus** : `companies` est vidée avant le seul point où l'audit survit. La trace aurait rapporté la nouvelle valeur comme étant l'ancienne |
| P2-4 | MEDIUM | `DATE_OUTSIDE_FISCAL_YEAR` est **inatteignable par la route** — c'est `NO_FISCAL_YEAR` qui répond. L'AC 9 aurait produit un test « ajusté jusqu'à passer » |
| P2-6 | MEDIUM | l'entrée d'audit de l'import aurait été signée par un Admin **de l'archive** (`MIN(id)`), sous `books.unlocked` — mêlant déverrouillages délibérés et restaurations sous la signature d'un innocent |
| P2-7 | MEDIUM | deux résidus du décompte corrigé, dont **la tâche T2** qui demandait encore de vérifier « les 22 chemins » |
| P2-8 | MEDIUM | l'AC 14 n'avait **ni fichier ni test**, quand l'AC 13 — celle qui protège le chemin sans importeur — en exigeait un en gras |
| P2-9 | MEDIUM | la garde neuve de l'AC 3 n'avait **aucun test nommé**, et le patch décrivait lui-même le test muet qui la manquerait |
| P2-10 | MEDIUM | « seize fonctions d'export » : il y en a **dix-neuf** — un décompte non recompté, recopié dans une clause normative |
| P2-11 | LOW | le « non nulle » de la garde de valeur perdu entre D3 et l'AC 6 |
| P2-12 | LOW | `< today` ne disait pas **quelle horloge** — l'écart d'un jour réintroduit sous une autre forme |
| P2-13 | LOW | I3 réécrit en universel omettait `reset_demo` |

⛔ **Le motif du dépôt sous sa forme la plus nette.** P2-2 : trois nombres neufs se contredisent
**à l'intérieur d'un seul paragraphe**, dans le patch qui corrigeait un décompte. P2-3 : le patch
**écrit le fait au sol et laisse la ligne qu'il réfute** douze lignes au-dessus. C'est la
§ *Propagation post-patch* et le § *Recompter ses propres comptes rendus*, tous deux pris en
défaut par la remédiation qui les invoquait.

✅ **La divergence de la passe 1 est CONFIRMÉE juste** — tracer la restauration plutôt que
l'interdire — mais son **exécution** ne l'était pas : l'ancienne valeur détruite avant la trace
(P2-5), et le mauvais signataire (P2-6). *Un arbitrage défendable ne dispense pas de vérifier
qu'il est réalisable.*

⚠️ **Aucune décision de conception antérieure au commit n'a été prise en défaut** : la borne
comme date, la garde au point de passage, l'asymétrie des deux gestes, le 400 plutôt que le 409
tiennent. La sévérité **recule** (CRITICAL → HIGH).

**Prochaine** : passe 3, ciblée, sur un modèle différent — avec un contrôle explicite du tableau
« Fichiers à toucher » contre les AC, que cette passe a trouvé désynchronisé.

### Passe 3 — 2026-08-30 · Sonnet 4.6, contexte frais, passe CIBLÉE sur le seul commit `59a165e6`

Prompt versionné dans `24-4c-review-prompt-regression-hunter-p3.md`.
**0 CRITICAL, 3 HIGH, 0 MEDIUM, 0 LOW — et les TROIS sont le MÊME symptôme.**

| # | sév. | ce qui était faux |
|---|---|---|
| P3-1 | HIGH | ⛔ la correction de « seize » avait **ajouté** « (dix-neuf, recomptées) » **à côté** du chiffre faux au lieu de le remplacer : la phrase affirmait 16 et 19 à la fois — la forme exacte du finding P2-2, recréée par le patch qui venait de la sanctionner |
| P3-2 | HIGH | trois occurrences de `create_in_tx` non corrigées en `create_in_tx_inner`, dont **le tableau des fichiers** et **le piège qui dit où lire `company_id`** — la thèse corrigée, ses applications laissées fausses |
| P3-3 | HIGH | la tâche **T5 prescrivait encore `books.unlocked`** pour l'import, que l'AC 14 du **même commit** venait d'interdire en toutes lettres |

⛔ **LE MÊME SYMPTÔME POUR LA TROISIÈME PASSE CONSÉCUTIVE : corriger la thèse au site nommé et
laisser ses applications ailleurs.** Passe 1→2, passe 2→3, et il ne s'agit plus d'un accident :
c'est le mode d'échec propre à cette story, à verser au Change Log de clôture au-delà du
§ *Propagation post-patch* déjà codifié.

⚠️ **La remédiation a trouvé PLUS que la lentille, et par le geste que le § prescrit.** Elle
nommait **trois** occurrences de `create_in_tx` ; un `grep` du **jeton** sur tout le document en
a rendu **six** — s'y ajoutaient le §22, le **titre même de la section D2**, et le paragraphe des
Dev Notes qui compare la garde à `update`. *Une lentille énumère ce qu'elle a vu ; seul le grep
du jeton énumère ce qui existe.*

✅ **Tout le reste de la passe 2 est confirmé correctement corrigé** après contrôle indépendant :
la flèche de D4, `reverse` → `create_in_tx_inner`, `NO_FISCAL_YEAR` au handler, le compte
« douze », `MIN(id)` et `ActorType`, `backup.rs:76`, `reset_demo` et ses issues, les compteurs de
migrations, le « non nulle » aux deux sites.

⚠️ **Aucune décision de conception n'a été prise en défaut, pour la troisième passe consécutive**,
et **aucun de ces trois findings ne déplace le lieu où le code va s'écrire** — critère que les
passes 1 et 2 avaient tous deux fait échouer. La sévérité tient (HIGH), le volume s'effondre :
8 → 13 → **3**.

**Prochaine** : passe 4, ciblée, quatrième contexte frais.

### Passe 4 — 2026-08-30 · Haiku 4.5, contexte frais, passe CIBLÉE sur le seul commit `0c2c5f54`

Prompt versionné dans `24-4c-review-prompt-regression-hunter-p4.md`.
**0 CRITICAL, 0 HIGH, 0 MEDIUM, 0 LOW — aucun écart.**

⚠️ **Le prompt de cette passe a une FORME différente des trois autres, et c'est une réponse à
l'échec de cette même lentille en passe 1.** Elle y avait rendu zéro finding en vérifiant la spec
**contre elle-même**, laissant passer deux CRITICAL. On ne lui a donc demandé **aucun jugement de
conception** : des **commandes à exécuter**, et le rapport de leurs sorties. *Le mode d'échec
restant était mécanique — une valeur corrigée à un site et laissée fausse ailleurs — donc son
contrôle devait l'être aussi.*

⛔ **Et le verdict a été REFAIT indépendamment avant d'être accepté.** Les trois greps de jetons
ont été rejoués sur le document, hors section « Journal de revue » : cinq occurrences de
`create_in_tx` qui distinguent toutes explicitement le wrapper de l'inner ; sept de
`books.unlocked` qui désignent toutes le déverrouillage délibéré — dont celle de T5, désormais
formulée **en négatif** (« et jamais `books.unlocked` ») ; zéro « seize ». *Un zéro de cette
lentille avait déjà été faux une fois dans cette boucle ; le refaire coûte trente secondes.*

### BOUCLE CLOSE — 2026-08-30

**Quatre passes, cinq contextes frais, rotation complète Sonnet + Haiku → Opus → Sonnet → Haiku.**

| passe | modèle | CRIT | HIGH | MED | LOW | total | nés d'une remédiation |
|---|---|---|---|---|---|---|---|
| 1 | Sonnet 4.6 + Haiku 4.5 | 2 | 1 | 3 | 2 | **8** | — |
| 2 | Opus 5 *(ciblée)* | 0 | 4 | 6 | 3 | **13** | **13 / 13** |
| 3 | Sonnet 4.6 *(ciblée)* | 0 | 3 | 0 | 0 | **3** | **3 / 3** |
| 4 | Haiku 4.5 *(ciblée)* | 0 | 0 | 0 | 0 | **0** | — |

⛔ **VINGT-QUATRE findings, dont SEIZE nés d'une remédiation — et à partir de la passe 2, la
totalité.** Aucune passe après la première n'a pris en défaut une décision de conception : la
borne comme date, la garde au point de passage, l'asymétrie des deux gestes, le 400 plutôt que
le 409 ont tenu de bout en bout.

⛔ **LE MODE D'ÉCHEC PROPRE À CETTE STORY, à verser à la rétrospective.** Ce n'est pas le grep
trop étroit de la 24-4b, c'en est le cousin : **corriger la thèse au site nommé et laisser ses
applications ailleurs dans le même document.** Il s'est produit **deux fois de suite**
(passe 1→2, puis 2→3), et sous sa forme la plus nette en P3-1, où une correction de nombre a été
**ajoutée à côté** du chiffre faux au lieu de le remplacer — la phrase affirmant alors les deux.

⚠️ **Ce qui a fini par le fermer n'est pas une passe de plus, c'est un GESTE** : greper le
**jeton** sur tout le document plutôt que corriger les sites qu'une lentille énumère. À la
passe 3, la lentille nommait trois occurrences de `create_in_tx` ; le grep du jeton en a rendu
**six**, dont le titre même de la section D2. *Une lentille énumère ce qu'elle a vu ; seul le
grep du jeton énumère ce qui existe.*

⚠️ **Deux CRITICAL de la passe 1 visaient le MÉCANISME DE GARDE lui-même**, non ce qu'il protège :
un invariant qui s'énonçait sans s'implémenter, et une séparation de rôles contournable par
l'autre verbe. *Une garde se lit comme le reste : en demandant qui l'empêche d'être contournée.*

**Prochaine** : `bmad-dev-story`.
