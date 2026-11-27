# Story 46-1 : arbre des groupes — modèle et API

Status: done

Epic : `_bmad-output/planning-artifacts/epic-46-rapports-groupes.md` — D1 à D10, Q2, Q3, Q4, Q5, Q7 ; FR-46-1, FR-46-7,
FR-46-8, FR-46-9 (contrat), FR-46-11 ; NFR-46-1 à NFR-46-4, NFR-46-7. Release visée **0.21.0** (prévision). Approuvée
par le responsable de l'epic et par Guy (revue 1, 2026-11-19) ; fiche écrite le 2026-11-20, à valider en boucle. **Aucune migration.**
Part de la branche d'epic après la reprise de `main` du 2026-11-20 (`d29aa557` : #1346, #1327, #1351 fusionnées).

## Story

En tant que comptable,
je veux que le bilan et le compte de résultat puissent se calculer par groupes de comptes, avec un sous-total par
groupe, un niveau de détail et des groupes ouverts ou fermés à mon choix,
afin de présenter mes comptes comme mon plan les range, sans perdre aucun total.

Cette story pose **le modèle et le contrat de l'API** que liront l'écran (46-2), les exports (46-3), le MCP (46-4) et
la balance (46-6a). Aucun écran, aucun rendu PDF ou CSV.

## Modules touchés (règle de splitting préventif)

Quatre : `kesh-report/groupes` (module neuf, avec l'enveloppe `AvecGroupes<T, S>`), `kesh-api/services/rapports`,
`kesh-api/routes/reports`, la documentation (`docs/api-external.md`). **`BalanceSheet` et `IncomeStatement` ne changent
pas** (C-46-1-4) : ni `balance_sheet.rs`, ni `income_statement.rs`, ni les littéraux des tests de `pdf.rs`, `csv.rs`
et de `benches/export.rs` (zones de la 48-11 et de la 47-3). Tests : `kesh-report` (unitaires),
`kesh-api/tests/reports_e2e.rs`. Aucun changement de `kesh-db` : l'arbre se lit
par `accounts::list_by_company(pool, company_id, true)` (`crates/kesh-db/src/repositories/accounts.rs:688`).

## Le contrat (source unique pour les stories 46-2 à 46-6b)

**Paramètres de requête**, sur `GET /api/v1/reports/balance-sheet` et `GET /api/v1/reports/income-statement` :

| Paramètre | Valeurs | Absent |
|---|---|---|
| `detailLevel` | entier `1` à `20`, ou `all` | **réponse d'aujourd'hui, à l'octet** (ni `grouping` ni `groupedRows`) |
| `subtotalPosition` | `header` (sous-total sur la ligne du titre du groupe) ou `footer` (ligne de sous-total sous les comptes du groupe) | `header` |
| `open` | identifiants de comptes de groupe ouverts à la main, séparés par des virgules | aucun |
| `closed` | identifiants de comptes de groupe fermés à la main, séparés par des virgules | aucun |

Refus `400 VALIDATION_ERROR` (message qui nomme le paramètre) : `detailLevel` hors de `1`–`20` et différent de `all`
(`0`, `-1`, `2.5`, `ALL`, vide) ; `subtotalPosition`, `open` ou `closed` **sans** `detailLevel` ; `subtotalPosition`
autre que `header`/`footer` ; un élément non entier dans `open` ou `closed` ; un même identifiant dans `open` **et**
`closed` ; plus de 500 identifiants dans l'une des listes. Un identifiant inconnu, d'une autre société, ou d'un compte
qui n'est pas un groupe de l'arbre affiché (groupe présent dans une section après dissolution, AC-4 — qu'il soit
visible ou caché sous un ancêtre fermé) est **ignoré** sans erreur (l'écran peut garder un état d'avant une
modification du plan) — et n'est pas renvoyé dans l'écho. `GET /reports/trial-balance` partage `ReportQuery` : jusqu'à
la 46-6a, ces paramètres y sont ignorés comme tout paramètre inconnu aujourd'hui.

**Règle d'ouverture** (décision 4 de l'orchestrateur sur la remarque 4 de Guy) : le niveau fixe l'état de départ, les
ouvertures et fermetures manuelles priment ensuite. Un groupe affiché de profondeur `p` est **ouvert** si et seulement
si `id ∈ open`, ou bien `id ∉ closed` et (`detailLevel = all` ou `p < detailLevel`). Un groupe fermé montre son
sous-total et **cache** toute sa descendance ; un groupe ouvert sous un ancêtre fermé n'est pas montré.
*Exemple* : niveau 2, groupe `100` (profondeur 2) fermé par le niveau ; `open=<id de 100>` l'ouvre et montre ses
comptes ; `closed=<id de 10>` ferme `10` et cache `100`, ouvert ou non.

**Réponse** (seulement quand `detailLevel` est donné) — champs ajoutés au niveau racine, ceux d'aujourd'hui inchangés :

- `grouping` : `{ detailLevel: 1…20 | "all", subtotalPosition, maxDepth, open: [ids], closed: [ids] }` — l'écho des
  choix **retenus** (identifiants ignorés retirés, listes triées), et `maxDepth`, la plus grande profondeur d'un groupe
  ou d'un compte **de l'arbre affiché**, sur toutes les sections, **indépendamment de l'ouverture** (au niveau 1, il
  vaut la même chose qu'à `all` : le sélecteur de niveau de la 46-2 ne se replie pas sur lui-même) ; `0` pour un
  rapport vide, `1` pour un plan sans groupe.
- `groupedRows` : un objet par section — `{ assets, liabilities, equity }` au bilan, `{ revenues, expenses }` au
  compte de résultat. *(D8 parlait d'un champ « par section » : les sections étant des tableaux JSON, le champ se
  porte à la racine, une clé par section.)* Les deux champs viennent de l'enveloppe `AvecGroupes<T, S>` (C-46-1-4),
  sérialisée **après** les champs du rapport (`#[serde(flatten)]`), absents (non `null`) sans `detailLevel`.
- Chaque ligne : `{ kind, depth, accountId, accountNumber, accountName, balance, currency?, balanceCurrency?, open?,
  active? }`, déjà **dans l'ordre d'affichage** :
  - `group` — un compte de groupe affiché ; `balance` = son sous-total ; `open` (booléen) toujours présent ;
  - `account` — un compte feuille (ou dont tous les sous-comptes sont absents de la section) ; `active` présent ;
    porte aussi `role` et, si la 47-3 est fusionnée, `mirrorOf` (lus tels quels de `AccountBalance`) ;
  - `own` — le **solde propre** d'un compte de groupe qui porte lui-même des écritures (D5), premier enfant de son
    groupe, `depth` = celle du groupe + 1 ;
  - `subtotal` — seulement en `footer`, et seulement pour un groupe **ouvert** : après sa descendance, même
    `accountId`, `depth` et `balance` que la ligne `group`.
  - **Règle de rendu, unique pour les trois rendus** : une ligne `group` porte son montant à l'affichage si
    `subtotalPosition = header` **ou** si le groupe est fermé ; sinon le montant va à sa ligne `subtotal`. Les lignes
    calculées du bilan (`retainedEarnings`, `equityResult`) restent **hors** de `groupedRows`, après les fonds
    propres, comme aujourd'hui (D4).

## Critères d'acceptation

1. **AC-1 — Fonction pure.** `kesh_report::groupes` (module neuf, sans I/O) expose
   `regrouper(lignes: &[L], arbre: &ArbreComptes, options: &OptionsGroupes) -> Vec<LigneGroupee>` où `L` implémente un
   trait `LigneAgregeable` (identifiant, numéro, nom, devise, **un vecteur de montants** et leurs montants en devise) :
   le bilan et le compte de résultat l'implémentent pour `AccountBalance` (un montant), la 46-6a pour
   `TrialBalanceRow` (cinq). `ArbreComptes::depuis(&[Account])` construit l'arbre depuis `parent_id` seul (D1), jamais
   depuis le numéro ; les frères se trient par numéro, `String::cmp` (D10), la ligne `own` en premier.
2. **AC-2 — Sous-totaux exacts.** Sous-total d'un groupe = Σ **exacte** (`Decimal`, sans arrondi) des montants de sa
   descendance présente dans la section, solde propre compris (D5). Dans chaque section, **à tout niveau et pour tout
   état d'ouverture**, la somme des `balance` des lignes de profondeur 1, **lignes `subtotal` exclues** (en `footer`, le sous-total d'un groupe de profondeur 1 a la profondeur de sa ligne `group`), égale le total de la section (NFR-46-2), et
   chaque ligne d'entrée est comptée **une fois et une seule** : soit en ligne `account`/`own` visible, soit dans le
   sous-total d'exactement un ancêtre visible fermé, soit dans un groupe fermé masqué parce que nul (AC-5), dont le
   zéro reste dans le sous-total de son parent.
3. **AC-3 — Niveau et ouverture.** Profondeur 1 = racines **de l'arbre affiché de la section** (après l'AC-4). Règle
   d'ouverture du contrat ; `all` ouvre tout groupe que `closed` ne ferme pas. Un compte sans parent s'affiche à tout
   niveau (D3). Un niveau supérieur à la profondeur réelle se comporte comme `all`.
4. **AC-4 — Sections légales, groupe à cheval dissous (D4, Q4, art. 959a CO lecture (a)).** Le bilan garde Actifs,
   Dettes, Fonds propres, puis les deux lignes calculées ; le compte de résultat Produits, Charges. L'arbre s'applique
   **dans** chaque section : un groupe dont la descendance présente au rapport (solde propre compris, rangé dans la
   section de son propre compte) tombe dans **plusieurs** sections n'est affiché dans **aucune** ; ses enfants
   remontent à son parent, et leur profondeur diminue d'autant. Exemple du plan PME
   (`crates/kesh-core/assets/charts/pme.json`) : `2 Passifs` (dettes et fonds propres) est dissous ; `20`, `24` sont
   de profondeur 1 dans Dettes, `28` dans Fonds propres. Un test rejoue le plan PME et affirme qu'aucune ligne `group`
   n'apparaît dans deux sections, et que les sections et lignes calculées sont celles du rapport sans paramètre, à
   chaque niveau de 1 à `all`. **La dissolution lit une clé de partition fournie par l'appelant**, distincte de la liste
   de sortie : la section pour le bilan et le compte de résultat ; la **nature** (débitrice ou créditrice) pour la
   balance (46-6a), qui n'a qu'une liste mais ne peut additionner des soldes de signes contraires. Le **solde propre**
   d'un groupe dissous devient une ligne `account` de la section de son propre compte, au rang de son numéro parmi les
   enfants remontés. La dissolution dépend des **lignes présentes** : un groupe peut être dissous à une date d'arrêté
   et non à une autre (p. ex. `2 Passifs` quand aucun compte de fonds propres n'a de solde) ; c'est voulu (règle sur
   les données du rapport, non sur la structure du plan, qui dissoudrait toujours `6`, à cause de `6950 Produits
   financiers`). **Relevé dans `pme.json` (validation P2)** : au compte de résultat, les groupes `6`, `7`, `70`, `8`
   et `80`, dont les feuilles mêlent charges et produits, sont dissous dès que les deux natures portent un solde ; au
   bilan, `2 Passifs` l'est quand un compte de rôle fonds propres a un solde (la partition du bilan est par rôle,
   non par type de compte : `pme.json` ne porte que des passifs sous `2`). Le test PME de l'AC-14 les vérifie une à une.
5. **AC-5 — Groupes non affichés (D6 modifiée, Q2).** Un groupe sans descendance dans la section n'y paraît pas. Un
   groupe **fermé par le niveau** dont le sous-total est nul en CHF **et** en devise n'est pas affiché ; son zéro reste
   dans le sous-total de son parent. Un groupe **fermé à la main** (`closed`) reste affiché, même nul : sinon la ligne
   disparaîtrait sous le clic qui la ferme, et l'écran (46-2) ne pourrait plus la rouvrir. Un groupe **ouvert** à sous-total nul mais à descendance non nulle (+100 / −100) reste
   affiché, avec sa descendance : aucun solde non nul ne disparaît. Aucun total n'est jamais perdu (FR-46-7).
6. **AC-6 — Devise d'un groupe (D7 modifiée, Q3).** `currency` et `balanceCurrency` d'une ligne `group` ou `subtotal`
   sont présents si et seulement si **toutes** les lignes d'entrée de sa descendance (solde propre compris) portent la
   **même** devise étrangère ; `balanceCurrency` en est la somme exacte. Sinon la ligne est en CHF seul, **sans ligne
   supplémentaire** (FR-46-8). Les totaux de section restent en CHF (NFR-37-4-3). *Ici et dans les fiches de l'epic,
« CHF » désigne la devise principale de la société ; « devise étrangère » un compte dont `currency` est renseigné.*
7. **AC-7 — Arbre défensif (NFR-46-4).** Un cycle de `parent_id`, un parent d'une autre société ou inconnu (possibles
   par SQL seulement) : le compte est traité en **racine**, `tracing::warn!` une fois par compte (numéro et motif),
   ni boucle ni panique ; la recherche d'ancêtres est bornée (profondeur ≤ nombre de comptes).
8. **AC-8 — Contrat de l'API.** Les quatre paramètres, leurs refus et la réponse du § *Le contrat* sur
   `GET /reports/balance-sheet` et `GET /reports/income-statement`. Sans `detailLevel` : JSON **identique à l'octet**
   à celui de `main` pour le même jeu de données (NFR-46-1), prouvé par un test qui compare le corps à une **fixture
   écrite au commit de base** (T0, avant toute modification ; plan PME, un compte en devise, deux exercices ; base
   `#[sqlx::test]` neuve et montage déterministe, donc identifiants stables — sinon le test normalise les `accountId`) — champs
   `grouping` et `groupedRows` absents, non `null`. Avec `detailLevel`, les champs
   d'aujourd'hui restent identiques (les sections plates sont toujours rendues).
9. **AC-9 — Ordre miroir puis groupes.** Le regroupement s'applique dans `services/rapports.rs` (`bilan`,
   `compte_de_resultat`), **après** la présentation miroir de la 47-3 si elle est fusionnée (`present_balance_sheet`,
   `present_income_statement`), jamais dans `balance_sheet::generate` ni `income_statement::generate`, qui restent
   inchangés (`balance_sheet::generate` appelle `income_statement::generate` pour le résultat de l'exercice). Si la 47-3 n'est pas fusionnée, le point
   d'insertion est commenté ; le second arrivé recâble (fiche 47-3, § *Recâblage à la fusion de l'Epic 46*). Une ligne
   de miroir se range sous le groupe **du miroir** et garde `mirrorOf`.
10. **AC-10 — Une seule requête de plus (NFR-46-3).** Avec `detailLevel`, le service lit l'arbre par une seule
    requête (`accounts::list_by_company(…, true)`, archivés compris : un groupe archivé dont un enfant porte un solde
    reste un groupe) ; sans `detailLevel`, **aucune** requête de plus. Agrégation O(n) en mémoire.
11. **AC-11 — Trace d'audit inchangée.** Le niveau et l'ouverture sont des choix de présentation : `DetailsTrace`
    (`services/rapports.rs:266`) ne change pas ; `crates/kesh-api/tests/services_motif.rs` et
    `audit_route_registry.rs` restent verts sans modification. Conséquence assumée : chaque demande écrit sa trace
    `report.generated` comme aujourd'hui (`emit_report_audit`, `:1059`), donc chaque redemande de l'écran (clic
    d'accordéon, changement de niveau, 46-2) en écrit une, comme un changement de période aujourd'hui.
12. **AC-12 — Art. 959a CO, lecture (a) (Q7, NFR-46-7).** La conformité tient au squelette de l'AC-4 ; aucune donnée
    neuve, aucun rattachement de compte à une rubrique, aucune migration. `docs/api-external.md` le dit, et dit qu'une
    présentation à un niveau qui replie les comptes (niveau 1 en particulier) est **un affichage de travail, non des
    comptes annuels** ; la même phrase va au manuel utilisateur à la 46-2.
13. **AC-13 — Arrondi.** Aucun formatage de montant n'est écrit dans cette story (le JSON porte des `Decimal` exacts) ;
    aucun `format!("{…:.2}")` neuf dans `kesh-report` — la garde de la 48-11 l'interdit (cf. *Dev Notes*).
14. **AC-14 — Tests.** Unitaires de `regrouper` sans base : plan PME complet, dont les dissolutions de l'AC-4 (`2` au bilan ; `6`, `7`, `70`, `8`, `80` au compte de résultat) ; groupe à solde propre ; groupe à cheval
    (dissous) ; cycle et parent étranger ; devise unique, devises mêlées, CHF mêlé à une devise ; niveaux 1, 2, 3 et
    `all` ; `open` et `closed` (dont un groupe ouvert sous un ancêtre fermé) ; `header` et `footer` ; groupe ouvert à
    sous-total nul ; groupe nul fermé par le niveau (masqué) et fermé à la main (affiché) ; `maxDepth` égal au niveau 1
    et à `all` ; solde propre d'un groupe dissous ; et **un test de propriété maison** (graine fixe, sans dépendance neuve : arbres et soldes générés)
    qui vérifie l'AC-2 pour tous les niveaux et des états d'ouverture tirés. Intégration (`reports_e2e.rs`) sur deux
    exercices : chaque refus de l'AC-8 ; un bilan et un compte de résultat au niveau 2 et à `all` ; l'égalité à l'octet
    sans paramètre ; une sonde négative (la mutation « ne pas dissoudre le groupe à cheval » rougit l'AC-4).
15. **AC-15 — Documentation.** `docs/api-external.md` : les quatre paramètres, les refus, `grouping`, `groupedRows`, la
    règle d'ouverture, la règle de rendu, l'exemple JSON d'un bilan PME au niveau 2 ; aucun numéro de compte hors de
    ceux que livre le plan PME (gardes G4-bis et G4-ter de `crates/kesh-api/tests/textes_coherents.rs`).

## Tâches / sous-tâches

- [x] **T0** — Reprise de `main` ; relire la 48-11 (fusionnée ou non) et la 47-3 (fusionnée ou non) ; noter au Change
      Log l'état trouvé (ligne lue citée). **Avant toute modification**, écrire les fixtures JSON de l'AC-8 (bilan et
      compte de résultat sans paramètre).
- [x] **T1 — `groupes.rs`** (AC 1 à 7, 13) : `ArbreComptes`, `LigneAgregeable`, `OptionsGroupes`
      (`NiveauDetail::{Niveau(u8), Tous}`, `PositionSousTotal::{Titre, Pied}`, ensembles `ouverts`, `fermes`),
      `LigneGroupee` (neutre : un vecteur de montants), `regrouper` ; tests unitaires et de propriété. La **conversion** de
      `LigneGroupee` en ligne JSON à champs nommés (`balance`, `currency`, `balanceCurrency`, `role`, `mirrorOf`…,
      `skip_serializing_if`) est propre à chaque rapport : bilan et compte de résultat la posent ici (`LigneRapport`,
      `Serialize`, camelCase), la balance en 46-6a (cinq colonnes). Déclarer le module dans `crates/kesh-report/src/lib.rs`.
- [x] **T2 — enveloppe** (AC 8, C-46-1-4) : dans `groupes.rs`, `AvecGroupes<T, S>` (`Serialize`, camelCase) —
      `#[serde(flatten)] rapport: T`, puis `grouping: Option<Grouping>` et `grouped_rows: Option<S>` (`S` : les sections
      du rapport), `#[serde(skip_serializing_if = "Option::is_none")]`. `BalanceSheet`, `IncomeStatement` et leurs
      littéraux ne changent pas.
- [x] **T3 — service et route** (AC 8 à 11) : `DemandePeriode` reste ; une `DemandeGroupes` optionnelle passée à
      `bilan` / `compte_de_resultat`, qui rendent `Rapport<AvecGroupes<…>, U>` (appelants : `routes/reports.rs:241`,
      `:253`, `:393`, `:524`, `mcp/outils/rapports.rs:305`, `:357` — l'export et le MCP lisent `.rapport` jusqu'aux
      46-3 et 46-4) ; validation dans le service (un seul endroit, réutilisé par l'export de la 46-3 et
      le MCP de la 46-4) **avec deux entrées** : l'analyse des chaînes de la route REST (`detailLevel=2`,
      `open=1,2`) qui produit la même `DemandeGroupes` typée, et la construction directe depuis des valeurs typées
      (tableaux d'entiers du MCP, 46-4) ; les refus de l'AC-8 vivent dans la seconde, sauf ceux de la syntaxe de la chaîne (`2.5`, `ALL`, vide, élément non entier), que la première refuse à l'analyse ; `ReportQuery` gagne `detail_level`, `subtotal_position`, `open`, `closed`
      (`Option<String>`, validés à la main comme `validate_format`, pour un 400 et non un 422 Axum).
      *(Annotation du 2026-11-21, revue P2 L-1 : description de tâche périmée — la remédiation P1 a retiré ces quatre champs de `ReportQuery` ; les paramètres se lisent dans `groupes_de` (`routes/reports.rs`) par `Query<Vec<(String, String)>>`, qui refuse un paramètre répété par un 400 `VALIDATION_ERROR`.)*
- [x] **T4 — tests d'intégration** (AC 14) dans `crates/kesh-api/tests/reports_e2e.rs`.
- [x] **T5 — documentation** (AC 12, 15).
- [x] **T6 — gate complet** (fmt, clippy, nextest, `scripts/gate-ci.sh`) ; frontend non touché ; E2E complet au
      dernier commit de code (règle du dépôt, même sans écran).

## Dev Notes

### État du code (relevé le 2026-11-20 à `29b9ec27`, reprise de `main` `d29aa557`)

- `crates/kesh-report/src/balance_sheet.rs:74-99` `BalanceSheet` (sections plates `assets`, `liabilities`, `equity`,
  totaux, `retained_earnings`, `equity_result`, `equation_holds`) ; `:152-180` `AccountBalance` (`FromRow`, champs
  `currency`, `balance_currency` `skip_serializing_if`) ; `generate` partitionne les fonds propres **par rôle**
  (`is_equity_role`) et les trie par rang de rôle (CO 959a al. 2). ⚠️ En mode groupé, l'ordre des fonds propres suit
  l'arbre (D4) : `groupedRows.equity` ne reprend **pas** l'ordre par rôle, et les sous-titres de rôle ne paraissent
  qu'en mode à plat (rendus, 46-2 et 46-3).
- `income_statement.rs:21-31` `IncomeStatement`, `fetch_section` (`ORDER BY a.number ASC`, `HAVING balance != 0 OR
  balance_currency != 0` : un compte à solde nul n'est **jamais** une ligne d'entrée).
- `crates/kesh-api/src/services/rapports.rs:199` `DemandePeriode`, `:610-637` `bilan`, `compte_de_resultat`,
  `:595` `details_periode`, `:266` `DetailsTrace`. `routes/reports.rs:64` `ReportQuery`, `:205` `validate_format`
  (modèle du 400 à la main), `:235-255` les deux handlers JSON.
- `accounts.parent_id` : FK `ON DELETE RESTRICT`, aucune contrainte de type entre parent et enfant ; un parent devient
  non imputable (14-3a) mais peut porter des écritures antérieures (D5). Le parent se change par `PUT /accounts/{id}`
  depuis #1351 (refuse un cycle et un parent qui porte des écritures).
- **Aucun rapport ne lit `parent_id` aujourd'hui** : rien à préserver de ce côté.

### Ce qui doit rester

Les générateurs (`generate`) et leur appel croisé (le bilan lit le compte de résultat), les structures `BalanceSheet`
et `IncomeStatement`, le JSON sans paramètre, les sections plates même en mode groupé, l'équation du bilan, la trace d'audit, les rendus PDF et CSV (46-3).

### Algorithme (indicatif)

1. Arbre : `HashMap<id, nœud>` depuis les comptes de la société ; parents résolus avec garde de cycle (marquage en
   cours de visite) ; un parent invalide → racine + `warn!`.
2. Par section : ensemble des comptes présents (lignes d'entrée) ; remontée de leurs ancêtres ; pour chaque ancêtre,
   l'ensemble des sections où tombe sa descendance (calculé sur **toutes** les sections du rapport) → dissous s'il y
   en a plus d'une.
3. Arbre affiché de la section (ancêtres non dissous) ; profondeurs ; sous-totaux par parcours postfixe ; devise
   unique ou non.
4. Parcours préfixe avec la règle d'ouverture ; émission des lignes (`group`, `own`, enfants, `subtotal` en pied) ;
   groupes fermés à zéro omis (AC-5).

### Arrondi au centime — contrainte de la 48-11 (Epic 48, release 0.20.0)

La 48-11 (#1245) doit déplacer `round_for_display` (aujourd'hui `crates/kesh-report/src/pdf.rs:349`, `pub(crate)`)
et une `format_two_decimals` dans `kesh-core`, et poser une garde (annoncée `arrondi_centime_garde.rs`) qui interdit
tout `{:.2}` neuf dans `kesh-report`. ⚠️ **Noms annoncés, non relevés** : au 2026-11-20, la fiche 48-11 est « à
écrire » (`epic-48-corrections-0-20-0.md` sur `origin/epic/48-corrections-0-20-0`, qui dit « fonction commune,
probable » dans `kesh-core/src/types/money.rs`) et aucune branche ne la porte ; les noms exacts se relèvent au T0. Cette
story ne formate rien ; si une valeur arrondie est nécessaire (tests compris), elle passe par les fonctions de
`kesh-core`. Si la 48-11 n'est pas fusionnée au T0, ne rien recopier : le dire au Change Log.

### Ordre avec l'Epic 47 (décision de Guy : miroir puis groupes)

La 47-3 (`story/47-3`, fiche `47-3-miroir-presentation-rapports.md`) applique la présentation miroir dans le service,
générateurs bruts inchangés, et ajoute `mirrorOf` à `AccountBalance`. Le regroupement vient **après** (AC-9). Les
invariants NFR-46-2 se vérifient sur les sections présentées.

### Choix et écarts

- **C-46-1-1** : niveau = profondeur dans l'arbre **affiché** de la section, après dissolution (D3 + D4) ; au niveau 1
  du bilan PME, Actifs montre `1 Actifs`, Dettes `20` et `24`, Fonds propres `28`. *Écarté* : la profondeur dans
  l'arbre brut, qui donnerait des sections sans aucune ligne de profondeur 1.
- **C-46-1-2** : la position des sous-totaux est dans la donnée (`subtotal`), non dans les rendus : écran, PDF, CSV et
  MCP lisent la même liste (D8, FR-46-9).
- **C-46-1-3** : « tout ouvrir / tout fermer » ne passe pas par l'API : c'est `detailLevel=all` ou `1` sans liste
  (46-2) ; les listes ne portent que les écarts manuels, bornées à 500.
- **C-46-1-4** (validation P1, KISS) : une **enveloppe** `AvecGroupes<T, S>` (`#[serde(flatten)]` ; `S` = les sections groupées du rapport, `SectionsBilan { assets, liabilities, equity }` ou `SectionsResultat { revenues, expenses }`, de `LigneGroupee`) plutôt que deux
  champs ajoutés à `BalanceSheet` et `IncomeStatement`. *Pourquoi* : ces structures ont une vingtaine de littéraux dans
  les tests de `pdf.rs` et `csv.rs` (`git grep -n "BalanceSheet {\|IncomeStatement {"`, 22 sites à `d29aa557`), là où
  la 48-11 et la 47-3 écrivent ; les y toucher créait une collision non déclarée, la 46-1 pouvant partir avant la
  48-11. Sans `detailLevel`, la sérialisation est celle du rapport seul (`rust_decimal` en `serde-str`, sans
  `arbitrary_precision` : le `flatten` ne change aucun octet ; l'AC-8 le prouve). Les générateurs restent inchangés,
  et la présentation miroir (47-3) s'applique au rapport avant l'enveloppe. *Écarté* : les champs dans les
  structures (plus direct pour les rendus, mais 22 littéraux et deux fichiers de plus).

### References

- Fichier d'epic (D1–D12, FR, NFR) ; réponses du responsable de l'epic et de Guy (`kesh-gate-logs/epic-46/`, revue 1 du 2026-11-19).
- `CLAUDE.md` § *Review Iteration Rule* (le prompt de chaque passe nomme `docs/api-external.md`) ; § *Migration
  breaking policy* (aucune migration ici).

## Note de version

- **Added** — API : le bilan et le compte de résultat se demandent **par groupes de comptes** (l'arbre du plan
  comptable) : `detailLevel` (1 à 20 ou `all`), `subtotalPosition` (`header` ou `footer`), `open` et `closed` ;
  la réponse gagne `grouping` et `groupedRows`, un sous-total exact par groupe, dans les sections légales (un groupe à
  cheval sur deux sections, comme `2 Passifs`, est dissous). Paramètres facultatifs : **sans eux, la réponse ne change
  pas, à l'octet**. L'écran (46-2), les exports (46-3) et le MCP (46-4) suivront.

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (bmad-dev-story), 2026-11-20.

### Debug Log References

Journaux hors dépôt, `<journaux>/s46-1/`, tous sur la cible propre `kesh-targets/kesh-wt46s1`
(`CARGO_INCREMENTAL=0`) sauf mention :
- `t0-fixture.log` (génération des fixtures, cible partagée abandonnée) et `verif-fixtures-base.log` (contre-épreuve sur
  l'arbre de base, cible à part : passé) ;
- `cible-clippy.log`, `cible-nextest.log` : gate ciblé au commit de remédiation `14230aa3` — fmt, clippy workspace
  verts, **327 passés sur 327**, 1 ignoré (au commit `b3ea6fc0` : 325 sur 325) ;
- `gate-backend.log` : `scripts/test-fast.sh --ci` **au commit `b3ea6fc0`** (avant la remédiation) — 6184 passés,
  14 ignorés ; **à rejouer au dernier commit de code** ;
- `front-check.log`, `front-lint-i18n-ownership.log`, `front-build.log` verts ; `front-test-unit-2.log` : Vitest
  278 fichiers, 4627 tests verts (`front-test-unit.log`, premier passage : un délai dépassé sous charge, frontend non
  touché) ;
- `mutations.log` : relancé et complet, voir ci-dessous ;
- `gate-ci-essai1.log`, `gate-ci-essai2.log`, `gate-ci.log` : trois essais en échec d'environnement (cause plus bas) ;
  rejoué avec succès, voir ci-dessous ;
- **Gates finaux (2026-11-21), dernier commit de code `14230aa3`** (les commits suivants ne portent que des fiches et
  des comptes rendus), tous par `heavy.sh` :
  - `mutations.log` : six sondes, chacune rougit des tests (sans mutation : 20 passés) — « ne pas dissoudre » 8 rouges ;
    M1 « solde propre hors de sa partition » 2 ; M2 « sous-total faux sous la profondeur 1 » 8 ; « groupe nul fermé à
    la main masqué » 1 ; « solde propre oublié » 11 ; « fermes ignorés » 2 ;
  - `gate-backend-final.log` : `scripts/test-fast.sh --ci`, **6186 passés**, 14 ignorés, fmt et clippy verts ;
  - `gate-ci.log` (`scripts/gate-ci.sh`, `cargo test` comme la CI) : **portée entière du workspace, code 0**, 6205 passés,
    18 ignorés. Un premier passage (`gate-ci-run1-zip.log`) s'était arrêté au premier binaire rouge,
    `admin_auto_backup_e2e::disque_sature_pendant_l_ecriture` (panique de `zip` 2.6.1, instabilité connue, fichiers non
    touchés par la branche ; rejoué 3 fois seul, `diag-sature.log`, vert) : `cargo test` s'arrête alors et le reste du
    workspace n'avait pas tourné ; le second passage, complet, est vert ;
  - `e2e.log` : E2E complet, **478 passés, 10 échecs, 18 ignorés** — les dix attendus : sept de la KF-029 (#97 : deux
    `mode-expert`, cinq `onboarding*`), `backup-restore-new-install.spec.ts:262` et `:381` (KF-065, #1282),
    `opening-balances.spec.ts:156` (KF-067, #1284). Aucune autre régression.

### Completion Notes List

**T0 — état trouvé (2026-11-20, branche `story/46-1` à `4c498494`).** 47-3 **non fusionnée** : `grep -rn "mirrorOf\|mirror_of" crates/` ne rend rien
(elle est sur `story/47-3`, en développement). 48-11 **non fusionnée** : `crates/kesh-report/src/pdf.rs:349`
`pub(crate) fn round_for_display(amount: Decimal) -> Decimal {` est toujours là, aucune `format_two_decimals` ; rien
recopié, rien formaté (AC-13). Les fixtures de l'AC-8 (`crates/kesh-api/tests/fixtures/rapports-groupes/`, bilan et
compte de résultat, exercices 2025 et 2026) ont été écrites par le code de base, **avant toute modification** : copie
`git archive 4c498494` plus le seul test du T0, `KESH_ECRIRE_FIXTURES_46_1=1` ; commit `1f75cb29` (test et fixtures
seuls). Les dates de période y sont **fixes** (exercices 2025-01-01→2025-12-31 et 2026-01-01→2026-12-31 créés
explicitement), aucune ne dépend de l'horloge de test : la fixture ne casse pas sous un autre `KESH_TEST_INSTANT`
(correction de l'orchestrateur reprise de la validation P2 de la 46-4). Les identifiants sont stables (base
`#[sqlx::test]` neuve, montage déterministe) : aucune normalisation nécessaire.

**Cible cargo.** La première génération des fixtures et un premier clippy ont tourné sur la cible partagée
`<cible partagée>`, abandonnée le 2026-11-20 (empreintes indépendantes du chemin) ; **tout a été rejoué**
sur la cible propre `kesh-targets/kesh-wt46s1` (`CARGO_INCREMENTAL=0`), et les fixtures contre-éprouvées sur l'arbre de
base avec une cible à part, supprimée ensuite (`verif-fixtures-base.sh`).

**Implémentation.**
- `crates/kesh-report/src/groupes.rs` (neuf, sans I/O) : `ArbreComptes` (`depuis(&[Account])`, `depuis_noeuds`,
  parents validés, cycles coupés au premier compte par numéro, remontées bornées, `warn!` par compte), trait
  `LigneAgregeable` (impl pour `AccountBalance`), `Dissolution::calculer` (clé de partition fournie par l'appelant),
  `regrouper`, `regrouper_sections`, `LigneGroupee` neutre (vecteur de montants), `Grouping`, `LigneRapport`,
  `SectionsBilan`, `SectionsResultat`, `AvecGroupes<T, S>` (`flatten`, `sans_groupes`, `bilan`,
  `compte_de_resultat`). Déclaré et réexporté dans `lib.rs`.
- `crates/kesh-api/src/services/rapports.rs` : `DemandeGroupes` avec ses deux entrées (`depuis_valeurs`, typée, porte
  les refus du contrat ; `depuis_requete`, REST, refuse la syntaxe puis délègue), `NiveauDemande`,
  `MAX_GROUPES_MANUELS = 500` ; `bilan_par_groupes`, `compte_de_resultat_par_groupes` ; `arbre_des_comptes` (une
  requête, `list_by_company(…, true)`, seulement avec des groupes).
- `crates/kesh-api/src/routes/reports.rs` : `ReportQuery` gagne `detail_level`, `subtotal_position`, `open`, `closed`
  (`Option<String>`, 400 et non 422) ; les deux handlers JSON rendent `AvecGroupes<…>` ; les exports lisent
  `&report.rapport`. `trial-balance` et `vat` partagent `ReportQuery` et ignorent les champs.
  *(Annotation du 2026-11-21, revue P2 L-1 : périmé — `ReportQuery` n'a plus que `fiscal_year_id`, `period_start`, `period_end` ; les quatre paramètres se lisent dans `groupes_de`, `trial-balance` et `vat` ne sont pas touchés.)*
- `crates/kesh-api/src/mcp/outils/rapports.rs` : `kesh_bilan` et `kesh_compte_de_resultat` lisent `.rapport` (sans
  groupes jusqu'à la 46-4).
- `docs/api-external.md` : paragraphe « Le bilan et le compte de résultat par groupes de comptes » (paramètres, refus,
  dissolution, règle d'ouverture, réponse, règle de rendu, art. 959a CO et « affichage de travail », exemple JSON du
  plan PME au niveau 2 — **tenu égal à la réponse réelle par un test**).

**Point d'insertion de la présentation miroir (AC-9).** Dans `bilan_par_groupes` et `compte_de_resultat_par_groupes`,
entre `generate_balance_sheet` / `generate_income_statement` et le `match groupes`, un commentaire marque l'endroit :
`present_balance_sheet` / `present_income_statement` (47-3) s'y appliquent au rapport brut, **avant**
`AvecGroupes::bilan` / `AvecGroupes::compte_de_resultat`, qui calculent alors les groupes sur les sections
présentées. Au recâblage : (1) appeler la présentation à cet endroit ; (2) ajouter `mirror_of` à `LigneRapport`
(`skip_serializing_if` vide), lu de `AccountBalance` par `source` dans `LigneRapport::depuis`, sur les lignes
`account` et `own` ; une ligne de miroir se range alors sous le groupe **du miroir**, puisque son `account_id` est
celui du miroir. Les générateurs ne changent pas.

**Interaction avec la 47-3, pour l'intégration (revue P1, sans code ici).** Rien n'oblige un compte miroir à avoir un
parent de son type (`routes/accounts.rs:76`, `check_parent_in_company`). Un miroir de passif rangé sous un groupe
d'actif (p. ex. un découvert `1020.1` sous `10`) fait dissoudre `10` et `1` dans le bilan groupé dès qu'un découvert
est présenté au passif : sommes justes (AC-4, voulu), mais groupes d'actif absents de l'affichage groupé. Point versé
à `kesh-gate-logs/epic-38-kiss/issues-a-ouvrir.md` (n° 20), à trancher avec le responsable de l'epic ; la revue du code d'intégration de
l'Epic 46 avec la 47-3 doit le savoir.

**Écarts à la fiche, déclarés.**
- **E1 — services à quatre arguments gardés.** T3 disait « une `DemandeGroupes` optionnelle passée à `bilan` /
  `compte_de_resultat` » ; mais l'AC-11 exige `services_motif.rs` vert **sans modification**, et ce test appelle
  `rapports::bilan(a, l, demande, u)` (`services_motif.rs:1104`, `:1882`). D'où `bilan` / `compte_de_resultat`
  inchangés en arguments (ils délèguent avec `None`) et `bilan_par_groupes` / `compte_de_resultat_par_groupes`, qui
  prennent la demande de groupes. Le type rendu est bien `Rapport<AvecGroupes<…>, U>` pour les quatre.
- **E2 — signature de `regrouper`.** L'AC-1 donnait trois arguments ; la dissolution (AC-4) exige de voir **toutes**
  les partitions, qu'une liste seule ne porte pas : `regrouper(lignes, arbre, &Dissolution, options)`, et
  `regrouper_sections(sections, arbre, options)` calcule la dissolution avec la section pour clé. La 46-6a appellera
  `Dissolution::calculer` avec la nature pour clé, puis `regrouper` sur sa liste unique.
- **E3 — `active` aussi sur `own`**, et `role` sur `account` et `own` (`null` pour un compte sans rôle, comme dans les
  sections plates) : la ligne `own` est une ligne d'entrée comme `account`.
- **E4 — `open=` (ou `closed=`) vide refusé** (« élément non entier ») : choix strict, dit au guide d'API.
- **Complexité (AC-10)** : agrégation et émission en O(n) par section, mais la construction de l'arbre affiché et la
  dissolution remontent les ancêtres de chaque ligne, O(n × profondeur) — profondeur de 3 au plan PME.

**Tests ajoutés** (de `4c498494` à `HEAD`, recomptés : `grep -c '#\[test\]' crates/kesh-report/src/groupes.rs` → 13 ;
`sqlx::test` de `reports_e2e.rs` 39 → 46, soit 7 ; *annotation du 2026-11-21, revue P2 L-2 : recompté au dernier commit de code `14230aa3`, **14** unitaires et `sqlx::test` 39 → 47, soit **8** d'intégration, le test `groupes_parametre_repete` de la remédiation P1 s'y ajoutant*) : 13 unitaires dans `groupes.rs` (niveaux 1, 2, 3, `all` et 20 ;
`maxDepth` ; `open`/`closed` dont un ouvert sous fermé ; `header`/`footer` et la somme sans `subtotal` ; solde propre ;
groupes nuls masqués ou non ; devise unique, mêlée, CHF mêlé ; cycle, parent inconnu, étranger, lui-même ; groupe à
cheval et son solde propre ; plan PME complet à chaque niveau de 1 à 20 et `all` ; propriété sur 300 arbres tirés,
graine fixe, deux colonnes une fois sur deux ; enveloppe ; contrat JSON) ; 7 d'intégration dans `reports_e2e.rs`
(fixture à l'octet, refus du contrat sur les deux routes — 15 cas chacune —, bilan au niveau 2 et `all` sur deux
exercices, compte de résultat de même, ouverture manuelle, identifiants ignorés et `footer`, exemple du guide).

**Sondes négatives** (`mutations.sh`, `mutations.log`, cible propre) : « ne pas dissoudre » → 6 rouges, dont les quatre
tests d'intégration groupés (AC-4) ; « groupe nul fermé à la main masqué » → 1 ; « solde propre oublié » → 10 ;
« `closed` ignoré » → 2. Fichier restauré après chaque mutation.

**Gate-ci, cause des 37 échecs « Unknown database 'kesh_ci_s461' » (trois essais, `gate-ci-essai1.log`,
`gate-ci-essai2.log`, `gate-ci.log` du 2026-11-21 07:09).** Les deux premiers coïncident avec l'OOM de 00:22 et la
sortie de dev2 (`Exited (137)`). Le troisième est une faute de mon enveloppe : `scripts/gate-ci.sh` lance cargo sous
`env -i`, si bien que `s46-1/gate-ci-wrap/cargo` ne voyait pas `KESH_DB_PORT` et retombait sur 3307 (dev2) pendant que
`gate-ci-wrap/docker`, lancé hors de `env -i`, créait la base sur le serveur de la place (dev4 ou dev5). Corrigé :
`s46-1/gate-ci.sh` écrit le port de la place en dur dans l'enveloppe cargo avant le lancement. Aucun autre gate-ci de
la 46-1 ne tournait (vérifié au `ps`). Aucun défaut du code de la story.

### File List

- `crates/kesh-report/src/groupes.rs` (neuf)
- `crates/kesh-report/src/lib.rs`
- `crates/kesh-api/src/services/rapports.rs`
- `crates/kesh-api/src/routes/reports.rs`
- `crates/kesh-api/src/mcp/outils/rapports.rs`
- `crates/kesh-api/tests/reports_e2e.rs`
- `crates/kesh-api/tests/fixtures/rapports-groupes/bilan-2025.json` (neuf)
- `crates/kesh-api/tests/fixtures/rapports-groupes/bilan-2026.json` (neuf)
- `crates/kesh-api/tests/fixtures/rapports-groupes/resultat-2025.json` (neuf)
- `crates/kesh-api/tests/fixtures/rapports-groupes/resultat-2026.json` (neuf)
- `docs/api-external.md`
- `_bmad-output/implementation-artifacts/46-1-arbre-des-groupes-modele-api.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

## Change Log

- 2026-11-20 — Fiche écrite depuis l'esquisse du 2026-11-18 : contrat complet (paramètres, refus, règle d'ouverture,
  règle de rendu, `grouping`, `groupedRows`), remarques 3 (lecture (a)), 4 (ouverture) et 5 (`header` par défaut, sans
  paramètre réponse inchangée) de Guy, ordre miroir puis groupes, contrainte de la 48-11. Quinze critères. `drafted`.
- 2026-11-20 — Validation P1 (Claude Opus 5.5, contexte frais, au code de `origin/main` `d29aa557`) : 0 C / 0 H / 4 M
  / 4 L, remédiés. M : collision non déclarée des champs ajoutés à `BalanceSheet`/`IncomeStatement` (22 littéraux dans
  les tests de `pdf.rs`/`csv.rs`, zones de la 48-11 et de la 47-3) → enveloppe `AvecGroupes<T, S>` (C-46-1-4, T2, T3,
  modules : quatre) ; `maxDepth` ambigu (lignes émises ou arbre : le sélecteur se repliait au niveau 1) → arbre
  affiché, indépendant de l'ouverture ; groupe fermé **à la main** à sous-total nul masqué (ligne impossible à
  rouvrir) et invariant de l'AC-2 contredit par les groupes nuls masqués → AC-5 (masqué seulement s'il est fermé par
  le niveau), AC-2 ; égalité « avec et sans la story » non exécutable → fixture au commit de base (AC-8, T0). L :
  solde propre d'un groupe dissous et dissolution selon les lignes présentes (AC-4) ; « groupe affiché » de l'écho
  précisé, `trial-balance` qui partage `ReportQuery` ; trace d'audit à chaque redemande dite (AC-11) ; noms de la
  48-11 annoncés et non relevés (*Dev Notes*). Rapport : `kesh-gate-logs/epic-46/validation/46-1-P1.md`.
- 2026-11-20 — Validation P2 (Claude Sonnet 5.5, contexte frais, au code de `origin/main` `d29aa557`) : 0 C / 0 H / 1 M
  / 4 L, remédiés. Vérifié : `rust_decimal` en `serde-str` (`Cargo.toml:18`), aucun `arbitrary_precision` ni
  `preserve_order` dans les `Cargo.toml` ; `flatten` en sérialisation passe par `serialize_map`, ordre et forme du JSON
  sans paramètre inchangés (l'AC-8 le prouve) ; `Rapport<T, U>` générique (`afficher`, `exporter`, `periode()` ne
  dépendent pas de `T`) ; appelants et lignes citées (`routes/reports.rs:241/253/393/524`, `rapports.rs:610-637`,
  `:1059`, MCP `:305/:357`, `accounts.rs:688`). M : `AvecGroupes<T>` annoncé à un paramètre mais `grouped_rows: Option<S>`
  en réclame un second → `AvecGroupes<T, S>`, sections `SectionsBilan`/`SectionsResultat` (T2). L : entrée **typée** de
  la validation pour le MCP (T3) ; conversion en champs nommés propre à chaque rapport (T1) ; groupes dissous du plan PME
  relevés (`6`, `7`, `70`, `8`, `80` au compte de résultat ; `2` au bilan, partition par rôle) (AC-4) ; déterminisme de
  la fixture (AC-8). Rapport : `kesh-gate-logs/epic-46/validation/46-1-P2.md`.
- 2026-11-20 — Validation P3 ciblée (Claude Haiku 5.5, contexte frais, lentille sur le seul commit `8a7f0f73`, au code de `origin/main` `d29aa557`) : 0 C / 0 H / 0 M / 2 L, remédiés. L : le test PME de l'AC-14 n'énumérait pas les dissolutions de l'AC-4 (désormais énumérées dans l'AC-14) ; les refus de syntaxe (`2.5`, `ALL`, vide, élément non entier) étaient attribués au service alors qu'ils se refusent à l'analyse (T3). Rapport : `kesh-gate-logs/epic-46/validation/46-123-P3-ciblee.md`.
- 2026-11-20 — Développement (Claude Opus 5.5, bmad-dev-story) : T0 (fixtures du commit de base, `1f75cb29`), T1 à T5
  (`b3ea6fc0`) ; corrections de l'orchestrateur reprises : AC-2 exclut les lignes `subtotal` de la somme de profondeur
  1 (code, tests, critère) ; dates de période de la fixture fixes, sans horloge de test. Écarts E1 à E4 au Dev Agent
  Record.
- 2026-11-21 — Revue de code P1 (Claude Opus 5.5, complète, une lentille) : 0 C / 0 H / 1 M / 6 L. Remédiation
  `14230aa3` : M-1, test `groupe_dissous_par_son_seul_solde_propre` (la règle « solde propre compris » de la
  dissolution n'était tenue par aucun test) ; L-1, la propriété affirme la somme de la descendance de chaque groupe à
  toute profondeur et l'absence d'un compte dans deux sections ; L-2, L-3, guide d'API (`active`/`role` sur `own`,
  « groupe de l'arbre affiché », groupe nul sans ligne) ; L-4, paramètres de groupes lus dans les paires brutes de la
  requête (`groupes_de`) : un paramètre répété est un 400 `VALIDATION_ERROR` qui le nomme sur le bilan et le compte de
  résultat, et `ReportQuery` revient à ses trois champs, si bien que la balance et la TVA les ignorent comme avant
  (test `groupes_parametre_repete`) ; L-5, l'accès est contrôlé dans le handler avant la validation des groupes ; L-6,
  comptes rendus, `mutations.sh` dans une copie hors dépôt. Interaction avec la 47-3 (miroir de passif sous un groupe
  d'actif) écrite au Dev Agent Record et versée aux issues à ouvrir, sans code. La remédiation touche la production
  (L-4, L-5) : passe ciblée P2 à suivre (`kesh-gate-logs/s46-1/revue-p2-prompt.md`).
- 2026-11-21 : LOW de la revue P2 (L-1, L-2, L-6) corrigés dans les fiches 46-1, 46-3 et 46-6a (texte seulement) ; mutations, gate backend, gate-ci complet (second passage) et E2E complets verts au dernier commit de code ; statut `done`.
