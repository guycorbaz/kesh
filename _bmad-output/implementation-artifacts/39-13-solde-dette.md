# Story 39-13 : solde de la dette de l'Epic 39 (Wise)

Status: done

*Fiche rédigée le 2026-11-22 au gabarit de `bmad-create-story`, branche `story/39-13` partie d'`epic/39-wise`
(`0a414376f`). Epic 39. Story de solde (`CLAUDE.md` § *Priorités des défauts*, « une story de solde à la fin de chaque
epic ») : elle ne prend que la dette des stories de l'epic. **Aucune migration**, aucune route neuve, aucun texte
d'interface, **aucun comportement visible changé**.*

## Plafond

Le fichier d'epic (`_bmad-output/planning-artifacts/epic-39-wise.md`) **ne fixe pas** de nombre d'issues pour la story
de solde : le plafond est pris à **deux** points de dette, faute de mieux (décision de l'orchestrateur, à confirmer par
le responsable de l'epic). Choisis parmi la dette de l'Epic 39 — ligne de registre de la 39-13, entrées « (39-…) » et
« (intégration 39/40) » de la liste des issues à ouvrir, LOW des Dev Agent Records des 39-11x et de la 39-14 — selon
la consigne « les plus faciles et sans risque d'abord ».

## Story

En tant que **développeur de Kesh**, je veux solder deux dettes laissées par les stories Wise — un contrôle de devise
quadratique et deux fonctions de lecture des doublons dont les commentaires ne disent plus vrai, l'une dupliquant
l'autre —, afin que le code de l'epic parte en PR sans ces restes.

## État de départ (relevé au code, `0a414376f`)

1. **`check_single_currency`** (`crates/kesh-import/src/wise.rs:530-541`, Story 39-11a, LOW L2 de la revue P2) :
   pour chaque ligne, `Vec::contains` sur les devises déjà vues — coût en O(lignes × devises distinctes), et une
   allocation par fichier même à une seule devise ; les devises étant des codes de trois lettres validés
   (`wise.rs:686-691`), le gain réel est mince : c'est surtout un chemin normal linéaire et sans allocation. Le refus
   `WiseCsvError::MixedCurrencies(Vec<String>)` porte les devises **dans l'ordre de leur première apparition** dans le
   fichier (doc de la variante, `wise.rs:298-301`), ordre relu par `routes/wise_imports.rs:358` (détail
   `currencies` du `422 WISE_CSV_INVALID`) et figé par `tests/wise_csv.rs:308-314` (deux devises seulement).
2. **`existing_transaction_ids` et `existing_transactions_by_ids`** (`crates/kesh-db/src/repositories/bank_transactions.rs:131-218`,
   relevé par la 39-14, § *Fichiers communs et points relevés*) :
   - `existing_transaction_ids` dit servir « la recherche de doublon de l'import PayPal » : depuis la 39-14, le cœur
     commun (`routes/provider_imports.rs:677`) lit `existing_transactions_by_ids` pour **tous** les prestataires ;
     `existing_transaction_ids` n'a plus d'appelant dans le dépôt (`git grep`), et aucun test ne l'exerce ;
   - `existing_transactions_by_ids` dit que « l'import Wise juge un doublon » et renvoie à 39-12a3 (sortie de l'epic) ;
   - les deux portent **la même requête** recopiée (paquets de 500, `QueryBuilder`, filtre société et compte) :
     duplication contraire à DRY.

## Décisions

- **D1 — Contrôle de devise linéaire, même erreur.** Comparer chaque ligne à la devise de la première ligne ; tant
  qu'elles sont égales, aucune collection. À la première différence, construire la liste des devises distinctes dans
  l'ordre de première apparition : un `Vec` de sortie, poussé quand `HashSet::insert` des devises vues rend vrai (un
  passage ; jamais l'itération du `HashSet`, qui perdrait l'ordre). Sortie **identique** à l'actuelle,
  pour tout fichier.
- **D2 — `existing_transaction_ids` écrite sur `existing_transactions_by_ids`.** Elle garde sa signature publique
  (`HashSet<String>` des identifiants tels que stockés) — `epic/40-stripe` l'appelle encore
  (`routes/provider_imports.rs:504` sur cette branche, relevé le 2026-11-22), si bien que la supprimer casserait la
  compilation de sa prochaine reprise ; la suppression reste possible quand plus aucune branche ne l'appelle — mais ne
  porte plus sa propre requête : elle projette les identifiants du résultat de `existing_transactions_by_ids`. Même
  filtre SQL, mêmes paquets, même collation ; deux colonnes lues en plus, sans effet.
- **D3 — Commentaires rendus vrais** : `existing_transactions_by_ids` nomme son appelant réel (le cœur commun des
  imports de prestataire, Story 39-14, doublons par (identifiant, sens) via `ExistingKeys`), sans renvoi à 39-12a3, et
  porte **seule** le contrat commun (transaction de l'import et ses verrous, index
  `idx_bank_transactions_account_txid`, collation sans casse, valeurs telles que stockées, paquets de 500) — le renvoi
  « même contrat que `existing_transaction_ids` » (`bank_transactions.rs:139`), devenu circulaire, disparaît ;
  `existing_transaction_ids` renvoie à ce contrat, dit qu'elle n'a plus d'appelant dans `kesh-api` depuis la 39-14 et
  pourquoi elle reste (lecture par identifiant seul, sans sens, encore appelée par une branche d'epic).

## Critères d'acceptation

- **AC-1** — `check_single_currency` ne fait qu'un passage sur les lignes, sans `Vec::contains` ; un fichier à une
  devise ne construit aucune liste.
- **AC-2** — Un fichier dont les lignes portent, dans l'ordre du fichier, CHF, USD, EUR, USD, CHF (une devise non
  initiale répétée, un retour de la première, un ordre qui n'est pas l'alphabétique) est refusé par
  `MixedCurrencies(vec!["CHF", "USD", "EUR"])`. Test neuf dans `crates/kesh-import/tests/wise_csv.rs` (fixture CHF,
  dont les lignes 0 et 4 sont déjà en CHF : lignes 1 à 3 changées en USD, EUR, USD par trois `set` emboîtés). **Test de caractérisation** : l'ancien code rend le même résultat ; il vaut
  contre les mutations (a) et (b), qui doivent le faire rougir.
- **AC-3** — `two_currencies_are_refused` reste vert sans modification.
- **AC-4** — `existing_transaction_ids` ne contient plus de requête SQL ; elle délègue à
  `existing_transactions_by_ids`. Test neuf `#[sqlx::test(migrations = "./test-schema")]` dans
  `crates/kesh-db/tests/bank_transactions_repository.rs`, de caractérisation lui aussi (l'ancien code le passe) :
  identifiant présent au débit et au crédit rendu une fois, identifiant absent non rendu ; **une vraie seconde
  société**, avec son compte et le même identifiant importé, exclue ; un autre compte de la société portant le même
  identifiant exclu. Pour qu'une délégation qui permute `company_id` et `bank_account_id` rougisse, il faut
  qu'**aucune ligne seedée ne porte** (`company_id` = id du compte visé, `bank_account_id` = id de la société visée) —
  des identifiants seulement distincts ne suffisent pas (contre-exemple de la validation P2). Le test le garantit par
  une **assertion de montage** : `existing_transactions_by_ids(conn, account, company, &ids)` (arguments permutés à
  dessein) rend un résultat vide. Ordre de création qui la satisfait : société A (id 1), son autre compte (id 1) puis le
  compte visé (id 2), puis la société B (id 2) et son compte (id 3). C'est l'assertion principale
  (`existing_transaction_ids(conn, company, account, &ids)` contient `CARD-1`) qui tue la permutation ; le montage
  garantit qu'elle le peut. Le test porte lui-même ses assertions de compte et de société.
- **AC-5** — Les commentaires des deux fonctions décrivent leurs appelants réels (D3) ; aucun renvoi à 39-12a3 ni à
  « l'import PayPal » comme seul appelant.
- **AC-6** — Liste des issues à ouvrir (hors dépôt) : les deux entrées soldées barrées, avec le commit.

## Tâches

- [x] T1 (AC-1, AC-2, AC-3) — `check_single_currency` réécrite (D1), test neuf d'ordre et d'unicité.
- [x] T2 (AC-4, AC-5) — `existing_transaction_ids` sur `existing_transactions_by_ids` (D2), commentaires (D3), test
  neuf.
- [x] T3 — Sondes par mutation hors dépôt, une à la fois, tuées sur assertion nommée.
- [x] T4 — Gates : `cargo fmt --check`, clippy workspace `-D warnings`, nextest complet ; frontend non touché ; aucune
  spec E2E ajoutée (règle de la nuit du 22.11 : E2E limité aux specs neuves de la story).
- [x] T5 (AC-6) — Barrer les entrées soldées ; registre `39-13-solde-dette: review` ; prompt de revue P1.

## Stratégie de tests

- Unitaires (`kesh-import`, sans base) : AC-2, AC-3 ; le test `wise_csv.rs` passe par `parse_wise_csv`, chemin public.
- Intégration (`kesh-db`, base éphémère) : AC-4.
- Mutations visées, chacune tuée sur une assertion nommée : (a) ne pas dédoublonner (pousser chaque devise différente
  de la première) → test d'AC-2 ; (b) perdre l'ordre (trier) → test d'AC-2 ; (c) `existing_transaction_ids` qui rend
  un ensemble vide → test d'AC-4 ; (d) filtre de compte retiré de la requête → test d'AC-4 (et celui
  d'`existing_transactions_by_ids`) ; (e) `company_id` et `bank_account_id` permutés à la délégation → test d'AC-4
  (assertion de montage comprise) ; (f) liste tirée de l'itération du `HashSet` → test d'AC-2 (rouge par intermittence :
  jouée plusieurs fois, déclarée comme telle).

## Modules touchés (2)

`kesh-import` (`src/wise.rs`, `tests/wise_csv.rs`), `kesh-db` (`src/repositories/bank_transactions.rs`,
`tests/bank_transactions_repository.rs`).

## Fichiers communs

Aucun fichier de la liste commune (CHANGELOG, PDF, catalogues i18n, compteurs, migrations). `sprint-status.yaml` :
ligne de la 39-13 seule. La revue d'intégration d'`epic/39-wise` en cours lit `routes/provider_imports.rs` et les
fusions : non touchés ici.

## Hors périmètre (dette de l'epic non prise, plafond atteint)

Restent à ouvrir en issues (liste hors dépôt) : 39-11a L2 et L8 (numéro de ligne après une ligne vide, ordre des 100
erreurs) ; 39-11b P3 (rognage du libellé) ; 39-11a2 (même identifiant deux fois dans le même sens) ; 39-11c P1 L5, L8,
L9, M1 ; 39-11e P1 L3 ; 39-14 (`json.paypal.or(json.wise)`) ; 39-11d2 (avertissement d'annulation d'un groupe) ; 39-11d
(garde des codes traduits), 39-11d P2 L1 et L2 ; intégration 39/40 (chaîne Stripe → Wise sans test).

## Note de version

Aucune ligne de CHANGELOG : rien de visible ne change (dette interne).

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (fiche, développement) ; validations : Sonnet (P1, P3 ciblée), Opus (P2).

### Ce qui est livré

- `b10ecf71e` — `check_single_currency` (`crates/kesh-import/src/wise.rs`) : chaque ligne comparée à la devise de la
  première (aucune liste construite pour un fichier à une devise) ; à la première différence, liste du refus en un
  passage (`filter` sur `HashSet::insert`, ordre de première apparition). Sortie inchangée. Test neuf
  `mixed_currencies_are_listed_once_in_file_order` (CHF, USD, EUR, USD, CHF → `[CHF, USD, EUR]`).
- `06001b9f6` — `existing_transaction_ids` (`crates/kesh-db/src/repositories/bank_transactions.rs`) sans requête
  propre : projection des identifiants d'`existing_transactions_by_ids` ; contrat (verrous, index, collation, valeurs
  stockées, paquets de 500) décrit une fois, dans `existing_transactions_by_ids`, qui nomme le cœur commun (39-14) ;
  renvoi circulaire et renvoi à 39-12a3 retirés. Test neuf
  `existing_transaction_ids_returns_each_id_once_within_company_and_account`. Écart assumé à la lettre d'AC-4 : le
  même identifiant sur l'autre compte et dans l'autre société serait **invisible** dans un ensemble qui contient déjà
  `CARD-1` ; le test leur donne des identifiants propres (`CARD-3`, `CARD-4`), rendus par leurs propres comptes et
  absents du compte visé — c'est ce qui fait rougir la mutation (d).
- `10d38b1c4` — hors plafond, documentation : `docs/api-external.md`, ligne `PROVIDER_ACCOUNT_NOT_FOUND`.

### Tests (recomptés, `#[test]` + `#[sqlx::test]`, de `0a414376f` à `06001b9f6`)

`crates/kesh-import/tests/wise_csv.rs` 42 → 43 ; `crates/kesh-db/tests/bank_transactions_repository.rs` 5 → 6 : **deux
tests neufs**, tous deux de caractérisation (l'ancien code les passe).

### Gates (au dernier commit de code `06001b9f6`)

Journal `kesh-gate-logs/s39-13/gate-complet.log` (place 1, `kesh-mariadb-dev2`, base `kesh_wt39s` remise à zéro,
migrée, seedée) : `cargo fmt --check` vert ; clippy workspace `--all-targets -D warnings` vert ; **nextest complet
(profil `ci`) : 6667 tests exécutés, 6667 passés, 17 ignorés**. Frontend non touché : non joué. Aucune spec E2E
ajoutée : aucun E2E (règle de la nuit du 22.11). Pas de `gate-ci.sh` (fin d'epic).

### Sondes par mutation (hors dépôt, `kesh-gate-logs/s39-13/mut.sh` et `mut.py`, copie `git archive` de `06001b9f6`, cible distincte `kesh-targets/kesh-wt39s-mut`, supprimée ; une à la fois, tuée = test nommé `FAILED` sur `panicked at`)

| Mutation | Résultat |
|---|---|
| témoins non mutés (`kesh-import`, `kesh-db`) | verts |
| (a) pas de dédoublonnage | tuée — `left: ["CHF","USD","EUR","USD"]` |
| (b) liste triée | tuée — `left: ["CHF","EUR","USD"]` |
| (f) itération du `HashSet` | tuée 6 fois sur 6 jouées (ordre aléatoire : rouge par intermittence en principe) |
| (c) ensemble vide | tuée — `bank_transactions_repository.rs:483` |
| (d) filtre de compte retiré de la requête | tuée — assertion de montage `:475` |
| (e) arguments permutés à la délégation | tuée — `:483` |

### Fichiers communs et points relevés

Aucun fichier commun touché hors `sprint-status.yaml` (ligne 39-13). Liste des issues à ouvrir (hors dépôt) : les
deux entrées soldées barrées avec leur commit. L'en-tête du module de test `bank_transactions_repository.rs` annonce
encore « 4 tests » (périmé avant la story) : laissé, hors périmètre.

### File List

- `crates/kesh-import/src/wise.rs`
- `crates/kesh-import/tests/wise_csv.rs`
- `crates/kesh-db/src/repositories/bank_transactions.rs`
- `crates/kesh-db/tests/bank_transactions_repository.rs`
- `docs/api-external.md`
- `_bmad-output/implementation-artifacts/39-13-solde-dette.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

## Change Log

- 2026-11-22 — Fiche rédigée (plafond pris à deux, faute de nombre au fichier d'epic).
- 2026-11-22 — Validation P1 (Sonnet, `kesh-gate-logs/s39-13/validation-p1-rapport.md`) : 0 C, 0 H, 2 M, 4 L. M1 :
  séquence d'AC-2 changée (CHF, USD, EUR, USD, CHF), les tests déclarés de caractérisation ; M2 : AC-4 avec une vraie
  seconde société et des identifiants distincts, mutation (e) ajoutée ; L1 : « quadratique » nuancé ; L2 : contrat
  décrit une fois, renvoi circulaire retiré (D3) ; L3 : raison de garder la fonction écrite avec le fait relevé
  (D2) ; L4 : mutations nommées par assertion. Passe P2 à lancer (un MEDIUM rendu).
- 2026-11-22 — Validation P2 (Opus, `kesh-gate-logs/s39-13/validation-p2-rapport.md`) : 0 C, 0 H, 1 M, 3 L. M1 : la
  condition « identifiants distincts » d'AC-4 ne tuait pas la permutation (contre-exemple) : remplacée par la condition
  exacte et une assertion de montage aux arguments permutés ; L1 : ordre gardé par un `Vec` poussé à l'insertion, mutation
  (f) ; L2 : trois `set` (lignes 1 à 3) ; L3 : citation `wise.rs:686-691`. Passe P3 à lancer.
- 2026-11-22 — Validation P3 ciblée sur `a14cf4c5a` (Sonnet, `kesh-gate-logs/s39-13/validation-p3-rapport.md`) :
  0 C, 0 H, 0 M, 2 L, appliqués (ordre de création d'AC-4, assertion qui tue (e)). **Validation close** : trend
  0/0/2/4 → 0/0/1/3 → 0/0/0/2, modèles Sonnet, Opus, Sonnet.
- 2026-11-22 — Hors plafond, documentation seule (demande de l'orchestrateur, LOW-4 de la revue d'intégration
  d'`epic/39-wise`, `kesh-gate-logs/integration/epic-39-fusion-11d2-40-revue.md`) : la ligne `PROVIDER_ACCOUNT_NOT_FOUND`
  de `docs/api-external.md` nomme aussi `GET /wise/review` et `POST /wise/accept` (`10d38b1c4`), relu au code
  (`provider_accept.rs:272` et `:672`, appelés par `routes/wise.rs:193` et `:609`).
- 2026-11-22 — Développée (`b10ecf71e`, `06001b9f6`), gates et sondes ci-dessus ; statut `review`, revue P1 à lancer
  (`kesh-gate-logs/s39-13/revue-p1-prompt.md`).
- 2026-11-22 — Revue de code P1 (Opus, `kesh-gate-logs/s39-13/revue-p1-rapport.md`) : 0 C, 0 H, 0 M, 4 L, tous appliqués
  sans code de production. LOW-1 : décompte du Dev Agent Record corrigé (`bank_transactions_repository.rs` 5 → 6, l'ancien
  grep comptait l'attribut cité dans un commentaire `//!`) ; LOW-2 : la table des erreurs de `docs/api-external.md` nomme
  `POST /paypal-downloads` (`PROVIDER_ACCOUNT_NOT_FOUND`) et `POST /wise/accept` (`PROVIDER_PROPOSAL_STALE`,
  `COUNTERPARTY_CURRENCY_MISMATCH`), relus au code (`routes/paypal_downloads.rs:93`, `routes/wise.rs:839`,
  `provider_accept.rs:1575`) ; LOW-3 : l'assertion de montage du test `kesh-db` lit la base par une requête directe, non
  par la fonction testée ; LOW-4 : en-tête du module de test sans nombre de tests. **Boucle close** : validation P1→P3
  (0/0/2/4 → 0/0/1/3 → 0/0/0/2, Sonnet, Opus, Sonnet), revue de code P1 0/0/0/4 (Opus), aucune ligne de production
  touchée par la remédiation ; gate ciblé `binary(bank_transactions_repository) | binary(wise_csv) | binary(textes_coherents)`
  au commit de clôture. Statut `done`.
