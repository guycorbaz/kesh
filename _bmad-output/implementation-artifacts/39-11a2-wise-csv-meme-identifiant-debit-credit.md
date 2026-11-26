# Story 39-11a2 — Wise CSV : un même identifiant au débit puis au crédit

Status: done

Epic 39 (Wise), phase 1 KISS. Correctif de 39-11a (lecteur) et de 39-11b (import), avant toute publication de
l'Epic 39 (aucune donnée importée à reprendre). Aucune migration, aucun changement d'écran ni de texte.

## Problème

Le lecteur de 39-11a refuse un fichier dont un identifiant paraît deux fois (`WISE_CSV_DUPLICATE_ID`, AC-5 de
39-11a : « une conversion porte le même identifiant dans deux fichiers, jamais deux fois dans un »). Les relevés
CSV CHF réels de 2025 et de 2026 (avec « frais indiqués séparément ») portent pourtant **le même `CARD-<id>` deux
fois, une fois au débit, une fois au crédit** : deux paires dans le relevé CHF 2025, une dans celui de 2026
(mesuré sur les fichiers ; EUR et USD n'en ont aucune). Deux formes, relevées à la recherche
`kesh-gate-logs/epic-39-40-kiss/wise-conversion-partielle-recherche.md`, « Deux formes du relevé que l'API ne
montre pas comme telles » :

- **réserve de carte expirée** : débit le jour du paiement, recrédit une semaine plus tard à l'expiration de la
  réserve, capture ensuite sur un autre solde ;
- **vérification de carte** : débit puis crédit du même petit montant, à quelques secondes d'écart.

Les deux lignes ont réellement mouvementé le solde : le *Running Balance* de chacune est le précédent plus son
montant. Le fichier entier est refusé, et le relevé CHF 2025 comme celui de 2026 ne peuvent pas s'importer.

## Décision

**La clé d'une ligne Wise est (identifiant, sens)** — sens = `Transaction Type`, `DEBIT` ou `CREDIT`.

- **Dans le fichier** (lecteur) : deux lignes de même identifiant et de **sens opposés** sont reçues toutes deux ;
  deux lignes de même identifiant et de **même sens** restent refusées, `DuplicateId { id, lines }`, `lines` = les
  lignes de cet identifiant **dans ce sens**. Un vrai doublon (même identifiant, même sens, même montant, même
  date) est donc refusé, et plus largement tout même identifiant deux fois dans le même sens.
- **À l'import** (route de 39-11b) : une ligne est un doublon quand le compte bancaire porte déjà une transaction
  de **même `transaction_id` et de même sens** ; le sens d'une transaction en base se lit au **signe du montant**
  (le lecteur garantit crédit ≥ 0, débit ≤ 0, `SIGN_MISMATCH` sinon). Une transaction en base de montant **nul**
  compte pour les deux sens (elle ne fait jamais insérer une seconde ligne ; une ligne nulle n'a aucun effet sur le
  solde). `transaction_id` reste l'identifiant Wise tel quel : le rapprochement (39-11c, frais `FEE-<x>` → `<x>`)
  et le téléchargement (39-12a3 (fiche sortie de l'epic, branche `futur/wise-api`), comparaison aux identifiants de l'API) n'en voient pas changer la forme.

**Formes écartées, et pourquoi.**

- *Identifiant + rang d'occurrence dans le fichier* : le rang dépend des bornes du fichier. Un relevé qui commence
  entre le débit et le recrédit donne au recrédit le rang 1, celui du débit déjà importé : le recrédit serait
  ignoré **en silence**, et le solde faux. La clé (identifiant, sens) ne dépend que de la ligne.
- *Identifiant suffixé du sens dans `transaction_id`* : change la valeur que lisent 39-11c et 39-12a3 ; aucun gain
  sur la clé (identifiant, signe du montant), lue sans migration.
- *Identifiant + sens + date + montant* : un même mouvement exporté « avec » puis « sans » frais séparés n'a pas le
  même montant ; la clé l'insérerait deux fois.

**Limite assumée.** Un même identifiant deux fois dans le **même** sens d'un même solde (débit, recrédit, puis
nouveau débit sous le même identifiant, si une capture se faisait sur le même solde après expiration de la
réserve) est refusé en entier, comme avant : refus bruyant, jamais un mouvement perdu. Aucun cas dans les relevés
2025–2026. À tracer en issue (sans `gh` dans cette story : à ouvrir par l'orchestrateur).

## Critères d'acceptation

1. **AC-1 — Lecteur, paire débit/crédit.** Un fichier dont un `CARD-<id>` porte un débit puis un crédit (réserve
   expirée, une semaine d'écart ; vérification de carte, quelques secondes d'écart, même montant) se lit : les deux
   lignes sont rendues, en ordre chronologique, le solde courant contrôlé sur les deux.
2. **AC-2 — Lecteur, vrai doublon refusé.** Même identifiant, même sens (même montant, même date, ou non) →
   `DuplicateId { id, lines }`, `lines` = les lignes de cet identifiant dans ce sens, dans l'ordre du fichier ; le
   premier doublon dans l'ordre du fichier est cité. Les tests existants de `DuplicateId` restent verts.
3. **AC-3 — Import de bout en bout.** Un relevé portant une paire débit/crédit s'importe : les deux transactions
   insérées sous le même `transaction_id`, `inserted` les compte toutes deux ; le réimport du même contenu (autres
   octets) n'insère rien (`duplicates` = toutes les lignes) ; un relevé qui ne porte que le recrédit, après un
   import qui portait le débit, insère le recrédit (et seulement lui).
4. **AC-4 — Dépôt.** Lecture neuve `bank_transactions::existing_transactions_by_ids(conn, company, compte, ids)`
   rendant `(transaction_id, amount, booking_date)` (la forme que 39-12a3 AC-4 prévoit ; elle la reprendra) ;
   `existing_transaction_ids` inchangée (PayPal).
5. **AC-5 — Sonde par mutation**, dans une copie hors dépôt : la clé du lecteur ramenée à l'identifiant seul fait
   rougir AC-1 ; le sens retiré de la clé de l'import fait rougir AC-3 ; la condition « montant nul = les deux
   sens » retirée fait rougir son test.
6. **AC-6 — Relevés réels.** Un test `#[ignore]` (ou un script hors dépôt, non versionné) lit les trois CSV réels
   « avec frais séparés » de 2025 et de 2026 par `parse_wise_csv` sans erreur ; il ne recopie aucune donnée réelle
   et lit le chemin des fichiers dans une variable d'environnement.
7. **AC-7 — Fixtures anonymisées.** Les paires de test sont fabriquées : identifiants, montants et dates inventés
   (dates en 2020), aucune donnée des relevés réels.

## À raccorder (Story 39-14, fusion avec le cœur de la 38-12a)

Le cœur `routes/provider_imports.rs` d'`epic/38-paypal` écarte un groupe dont `group_id` a **déjà paru** dans la
source et lit les doublons par `existing_transaction_ids` (identifiant seul). Avec `group_id = transaction_id`, le
recrédit d'une paire serait **écarté en silence** comme doublon de source, et un relevé ne portant que le recrédit
le perdrait aussi. Au raccord, Wise doit garder la clé (identifiant, sens) : `group_id` = identifiant + sens, et un
test de doublon en base par (identifiant, signe du montant) — par `existing_transactions_by_ids` de cette story.
Le test d'import de l'AC-3 est le garde-fou du raccord : il doit rester vert sans changement d'assertion.

Pour 39-12a3 (AC-4, « règle de base : même `transaction_id` sur le même compte → ligne ignorée ») : la règle de
base devient (identifiant, sens). Pour 39-11c : une paire débit/crédit de carte a le genre `Other` (préfixe
`CARD`), aucune paire de conversion ne se forme sur elle.

## Tasks

- [x] T1 — Lecteur : `check_unique_ids` sur (identifiant, sens) ; doc du module et de `DuplicateId`.
- [x] T2 — Tests du lecteur (AC-1, AC-2, AC-7) dans `crates/kesh-import/tests/wise_csv.rs`.
- [x] T3 — Dépôt : `existing_transactions_by_ids` et son test.
- [x] T4 — Route : doublons par (identifiant, sens) ; test d'import (AC-3) dans `wise_import_e2e.rs`.
- [x] T5 — Relevés réels (AC-6) ; sonde par mutation (AC-5).
- [x] T6 — Gates.

## Dev Agent Record

**Commits** (de `epic/39-wise` à `HEAD`, code) : `51c7601e` lecteur et ses tests, `831db611` dépôt, route et leurs
tests, `9199975d` `cargo fmt` des tests du lecteur (dernier commit de code). Huit fichiers, 633 lignes ajoutées et 16 retirées (dont le Dev Agent Record),
16 retirées (`git diff --stat epic/39-wise...HEAD`, fiche et registre compris).

**Fait.**

- `kesh-import/src/wise.rs` : `check_unique_ids` sur la clé (identifiant, `WiseDirection`) ; `DuplicateId.lines` =
  les lignes de l'identifiant **dans le sens en double** ; `WiseDirection` dérive `Hash` ; doc du module et de la
  variante. Forme de l'erreur et code `WISE_CSV_DUPLICATE_ID` inchangés.
- `kesh-db/src/repositories/bank_transactions.rs` : `ExistingTransaction` et `existing_transactions_by_ids`
  (`transaction_id, amount, booking_date`, par paquets de 500) ; `existing_transaction_ids` inchangée (PayPal).
- `kesh-api/src/routes/wise_imports.rs` : `existing_keys` (sens au signe du montant, nul = les deux sens) ; le filtre
  des doublons de `WiseWriter` sur (identifiant, sens) ; doc du module.
- Aucune migration, aucun écran, aucun texte i18n, aucun changement de contrat de l'API (le compte rendu garde sa
  forme ; seul change le comportement : une paire débit/crédit n'est plus refusée). Le manuel ne dit rien des
  doublons Wise (`grep -i wise docs/manual/fr/*.tex` croisé avec « doublon », « identifiant » : rien) : inchangé.

**Tests ajoutés** (recomptés aux deux bornes, `#[test]` et `#[sqlx::test]`, de `epic/39-wise` à `HEAD`) : sept —
`wise_csv.rs` 30 → 34 (réserve expirée puis recréditée, vérification de carte, vrai doublon même sens-montant-date,
second débit sous le même identifiant : lignes du sens en double seules) ; `wise_import_e2e.rs` 7 → 8
(`import_meme_identifiant_au_debit_puis_au_credit` : débit seul, puis recrédit seul importé, puis le mois entier
n'insère que la vérification de carte, puis réimport sans effet ; solde et lignes en base) ;
`bank_transactions_repository.rs` 5 → 6 ; tests unitaires de `wise_imports.rs` 10 → 11 (`existing_keys`).
Fixtures fabriquées dans les tests (identifiants `CARD-91…`, `CARD-92…`, montants et dates 2020 inventés), aucune
donnée réelle.

**Relevés réels (AC-6)** : le test ignoré existant `real_statements_are_read` (`KESH_WISE_REAL_DIR`, 39-11a), lancé
dossier par dossier : 2025 avec frais séparés (CHF 72, USD 134, EUR 87 lignes), 2026 avec (CHF 89, EUR 148, USD 121),
2025 sans (CHF 61, EUR 72, USD 110) — tous lus ; **avant le correctif** (`wise.rs` d'`epic/39-wise`), 2025 avec :
`WISE_CSV_DUPLICATE_ID`, le constat. Journal : `kesh-gate-logs/s39-11a2/releves-reels.log` (devises et nombres de
lignes seulement). Un second test ignoré, écrit d'abord, doublait l'existant : retiré avant commit.

**Sonde par mutation (AC-5)**, copie `git archive 9199975d` hors dépôt, cible cargo à part, témoin vert, six
mutations, **six rouges** (`kesh-gate-logs/s39-11a2/mutations.log`) : clé du lecteur sans sens (4 tests rouges, dont
l'import de bout en bout) ; lignes citées sans filtre de sens (1) ; clé de l'import sans sens (1, l'import de bout en
bout) ; montant nul exclu du crédit (1) ; exclu du débit (1) ; filtre des doublons retiré (2).

**Gates** (tous au commit `9199975d`, dernier commit de code ; par `heavy.sh`, cible
`<station>/kesh-targets/kesh-wt39a2`, `CARGO_INCREMENTAL=0`) :

- ciblé (fmt `--check`, clippy workspace `-D warnings`, nextest des binaires touchés) : 58 passés, au commit
  `831db611` avant le `fmt` des tests (le `cargo fmt --check` y passait déjà sur l'arbre de travail) —
  `cible-1.log`, `nextest-cible-1.log` ;
- backend complet `scripts/test-fast.sh --ci`, base remise à zéro (`kesh-mariadb-dev5`) : **6348 passés, 16 ignorés**
  — `gate-backend.log` ;
- `scripts/gate-ci.sh` jusqu'au bout, sur `git archive 9199975d` : **6367 passés, 0 échoué, 20 ignorés**, code 0 —
  `gate-ci.log` ;
- frontend et E2E : **non lancés**, aucun fichier du frontend, aucun écran ni aucune forme de route touchés.

**Pour l'orchestrateur.**

- À ouvrir en issue (non ouverte ici, sans `gh`) : la limite assumée — un même identifiant deux fois dans le même
  sens d'un même solde est refusé en entier (aucun cas dans les relevés 2025–2026).
- Raccord 39-14 : voir « À raccorder » ; le test `import_meme_identifiant_au_debit_puis_au_credit` est le garde-fou.
- Les fiches 39-12a3 (AC-4, règle de base des doublons) et 39-11c ne sont pas modifiées par cette story.

## Change Log

- 2026-11-21 — fiche écrite (correctif prioritaire, constat sur les relevés réels).
- 2026-11-21 — développée (`51c7601e`, `831db611`, `9199975d`) ; gates verts au `9199975d` ; passée en `review`.
  Revue de code P1 complète à lancer : prompt `kesh-gate-logs/s39-11a2/revue-p1-prompt.md`.

### Clôture (2026-11-22)

Revue de code **P1** (Sonnet, complète) : **0 CRITICAL · 0 HIGH · 0 MEDIUM · 5 LOW**
(`kesh-gate-logs/s39-11a2/revue-p1-rapport.md`). Boucle close sur P1. LOW corrigés ici (texte seul) : nom de personne et
chemin personnel retirés de la fiche, décompte des lignes recompté. **Limite assumée, complétée (LOW 1)** : une
transaction déjà en base à 0.00 vaut pour les deux sens ; un mouvement non nul de même identifiant serait alors écarté
sans signal — cas absent des relevés réels. **Raccords (LOW 3)** : la règle de base de 39-12a3 (AC-4) et l'origine d'un
`FEE-x` (R1) doivent lire la clé (identifiant, sens) ; à la 39-14, `group_id` = identifiant + sens. LOW 2 (casse au
`IN (...)`, comportement inchangé) : en issue.
