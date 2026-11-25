# Story 46-4 : outils MCP du bilan et du compte de résultat par groupes

Status: done

Epic : `_bmad-output/planning-artifacts/epic-46-rapports-groupes.md` — D8, Q5 ; FR-46-4, FR-46-11 ; NFR-46-6. Release
visée **0.21.0** (prévision). Approuvée par le responsable de l'epic et par Guy (revue 1, 2026-11-19) ; fiche écrite le 2026-11-20, à
valider en boucle. **Suit la 46-1 ; précède la 46-6a.** Aucune migration. La balance par groupes au MCP est la 46-6a.

## Story

En tant qu'utilisateur d'un assistant branché sur Kesh,
je veux demander le bilan ou le compte de résultat par groupes, au niveau, avec les groupes ouverts ou fermés et la
place des sous-totaux de mon choix,
afin d'obtenir une synthèse au bon niveau, comme à l'écran, sans tout le détail.

## Modules touchés (règle de splitting préventif)

Trois : `kesh-api/mcp/outils/rapports.rs`, les tests MCP (`crates/kesh-api/tests/mcp_outils_e2e.rs`, `mcp_contrat.rs`),
le manuel d'administration (`docs/manual/fr/admin-manual.tex`, tableau des outils MCP). `mcp/sortie.rs` n'est **pas**
touché : l'énumération de genre se déclare dans `rapports.rs` par `champ_enum!`, comme `postes_ouverts.rs:75`.

**Ordre** : suit la 46-1 ; **précède la 46-6a**, dont le MCP de la balance reprend, sans copie, les arguments, le
schéma et la fonction de conversion posés ici (validation P1).

## Critères d'acceptation

1. **AC-1 — Arguments.** `kesh_bilan` et `kesh_compte_de_resultat` (`ArgumentsPeriode`, `deny_unknown_fields`
   conservé) gagnent quatre arguments facultatifs aux **noms de l'API** : `detailLevel` (entier 1–20 ou `"all"`),
   `subtotalPosition` (`"header"` | `"footer"`), `open`, `closed` (tableaux d'entiers, au plus 500).
   - *Lecture des arguments* : `arguments()` n'est que `serde_json::from_value` et rend `(arguments)` invalide sans
     jamais recopier le message de `serde` ; le schéma JSON n'est **pas** appliqué à l'exécution. `detailLevel` se lit
     donc par un type écrit à la main (entier ou `"all"`, tout autre JSON refusé), `open` et `closed` par
     `Option<Vec<i64>>`. **Aucun schéma MCP existant ne porte de tableau ni de `oneOf`** (relevé du 2026-11-20,
     `git grep` sur `src/mcp`) : c'est une première, que `mcp_contrat` (clause K7, forme de `inputSchema`) doit
     continuer d'accepter — relevé à T1.
   - *Pas de `#[serde(flatten)]`* : il est incompatible avec `deny_unknown_fields`. Les quatre champs se déclarent dans
     `ArgumentsPeriode` (et, à la 46-6a, dans `ArgumentsBalance`) ; **une seule fonction** les convertit en la demande
     typée de la 46-1 (`DemandeGroupes`) et en appelle la validation — **sans** passer par les chaînes séparées par des
     virgules de l'API REST (la 46-1 expose une entrée typée, à côté de l'analyse des chaînes de la route).
   - *Refus* : les bornes (`detailLevel` 1–20, au plus 500 identifiants), « `subtotalPosition`, `open` ou `closed`
     sans `detailLevel` » et « un identifiant dans `open` et `closed` » sont celles de la 46-1 (entrée typée, T3 de la
     46-1). Un identifiant nul ou négatif n'est **pas** un refus de la 46-1 : il est ignoré comme un identifiant inconnu
     (le `minimum: 1` du schéma n'est qu'une indication, le schéma n'étant pas appliqué). Tout refus — `serde`
     (`(arguments)`) ou service (`AppError::Validation` → `(demande)`, `mcp/textes.rs:315`) — rend à l'agent le même
     texte fixe « Arguments invalides. » (`textes.rs:127`, `:249-252`), qui ne nomme jamais l'argument : la description
     de l'outil énonce donc ces contraintes ; aucune validation locale de plus n'est écrite dans l'outil (une seule
     validation, celle de la 46-1).
   - *Schéma* : `SCHEMA_PERIODE` ne sert qu'à ces deux outils (la balance a son schéma en ligne,
     `rapports.rs:208-209`) ; il est remplacé par `SCHEMA_PERIODE_GROUPES`, `additionalProperties: false` gardé :
     `detailLevel` = `{"oneOf":[{"type":"integer","minimum":1,"maximum":20},{"type":"string","enum":["all"]}]}`,
     `subtotalPosition` = enum, `open` / `closed` = `{"type":"array","items":{"type":"integer","minimum":1},"maxItems":500}`. *(Amendé le 2026-11-22, revue de code P1 L1 : le `minimum: 1` des éléments de `open` et `closed` est retiré du schéma ; un identifiant nul, négatif ou inconnu est ignoré.)*
2. **AC-2 — Sans argument neuf.** La réponse est celle d'aujourd'hui **au seul champ `groupes: null` près** (AC-3 ;
   `sortie!` n'omet aucun champ, C-37-9 ; décision C-46-4-5) : un test compare le texte rendu à un **littéral écrit au
   commit de base** (avant T1) sur la société d'essai, pour chacun des deux outils — non à la sortie de la fonction qui
   le produit (un « avant et après » ne se rejoue pas dans un seul commit). Le littéral **reste tel qu'écrit à T0** ;
   le test en analyse la partie JSON, y ajoute la clé `groupes` à `null` au niveau supérieur, la re-sérialise par le
   même chemin que la sortie (`serde_json::to_value` puis `to_string_pretty`, `sortie.rs:489-493`), et compare le texte
   rendu **entier**, en-tête compris, à ce texte : toute autre différence rougit. ⚠️ La société d'essai (`societe_piegee`) est datée du jour de l'horloge de test
   (`tests/common/jour.rs`, `aujourd_hui_test`), que la recette fait varier (`KESH_TEST_INSTANT`) : les dates de la
   période (et tout identifiant qui ne serait pas stable) s'écrivent dans le littéral en **jetons** (`{debut}`,
   `{fin}`), remplacés par les valeurs de `a.p` avant la comparaison ; tout le reste est littéral.
3. **AC-3 — Réponse groupée.** Avec `detailLevel`, la sortie gagne un champ `groupes` (`null` sinon, convention
   C-37-9) : `{ niveau, tousLesNiveaux, positionSousTotaux, profondeurMax, ouverts, fermes, <sections> }` — l'écho
   `grouping` de la 46-1 (`niveau` à `null` et `tousLesNiveaux` à `true` pour `"all"` : un champ de `sortie!` ne porte
   qu'un type ; `positionSousTotaux` est, comme `genre`, un miroir admis par `champ_enum!`, valeurs `header`/`footer`) —
   et, par section, les lignes groupées (`actifs`, `passifs`, `fondsPropres` ; `produits`, `charges`).
   Une ligne : `genre`, `profondeur`, `compteId`, `numero` et `nom` en `TexteNonFiable`, `solde` en `Montant`, `devise`
   en `Code<Devise>` et `soldeDevise` (`null` hors devise unique), `ouvert` (`null` hors groupe).
   - **`genre` suit la convention des outils** (`postes_ouverts.rs:75-110`) : un **miroir** local, énumération sans
     champ qui dérive `Serialize` en `camelCase`, construite par un `match` exhaustif depuis le genre de la 46-1, admise
     par `champ_enum!` — valeurs `group`, `account`, `own`, `subtotal`, **celles de l'API** (un seul vocabulaire ; le
     français `groupe|compte|propre|soustotal` est celui du CSV, 46-3). `sortie!` n'admet aucun attribut de champ.
   - La sortie lit l'enveloppe `AvecGroupes<…>` de la 46-1 (C-46-1-4) : `grouping` et `groupedRows` d'elle, les
     listes et totaux plats de `.rapport`.
   - Les lignes de groupée ne portent ni `typeDeCompte`, ni `actif`, ni `role` : le détail d'un compte se lit sans
     `detailLevel` (listes plates). La 47-3 n'ajoute aucun champ au MCP (C-47-3-3) : elle ne change que des descriptions ; une ligne de compte miroir n'a donc rien de propre.
   - Les listes plates (`actifs`, `passifs`, `fondsPropres`, `produits`, `charges`) sont alors **vides** (`[]`) : la
     réponse reste sous le plafond de taille (C-32-9-8) ; la description de l'outil le dit, dit que la règle de rendu
     (montant sur le titre ou sur le sous-total) est celle de l'écran, et énonce les contraintes des arguments (AC-1).
     Totaux, report, résultat et équation : inchangés.
4. **AC-4 — Résumé d'appel.** Le résumé (`resume_periode`, `Resume::outil`) porte `detailLevel` (`entier`, ou
   `code("detailLevel", "all")`) et `subtotalPosition` (`code`, chaînes statiques) quand ils sont donnés, et le
   **nombre** d'identifiants de `open` / `closed` sous les clés `openCount` / `closedCount` (pas la liste, et pas sous
   la clé `open`, qui dirait une liste). Les méthodes de `Resume` (`sortie.rs:404-458`) sont `entier`, `date`,
   `booleen`, `code` et `recherche_fournie` ; seules les quatre premières servent ici.
5. **AC-5 — Tailles mesurées, et seuil écrit.** *Le plafond est celui de la réponse (24 576 o, refus pour taille,
   `MCP_ENVOI_OCTETS_MAX`) ; le bilan plat tient 57 comptes en devise en 20 156 o (`rapports.rs:7-13`), soit ≈ 350 o
   la ligne, et une ligne groupée est du même ordre. Le plan PME (`pme.json`, 86 comptes) ne se rend pas en entier
   dans un seul rapport : le bilan n'en porte que les 31 comptes de bilan **feuilles** (18 actifs, 13 passifs), le compte de résultat
   les 34 de produits et de charges feuilles (9 et 25) — les 21 autres sont des groupes, qui ne portent pas d'écriture
   propre ici (relevé du 2026-11-20 sur `pme.json`) —, et seulement ceux à solde non nul. Calcul du 2026-11-20 (tous les comptes à solde non nul, `footer`,
   `2 Passifs` dissous au bilan, `6`, `7`, `70`, `8`, `80` au compte de résultat) : **43 lignes groupées au bilan et 52
   au compte de résultat à `all`**, soit ≈ 15 et ≈ 18 Ko — sous le plafond attendu, à mesurer. (La remédiation P1
   affirmait le contraire en comptant les 86 comptes dans un seul rapport : corrigé en P2.)* À T3, des mesures sur
   une fixture qui pose des `parent_id` par SQL (les fixtures des tailles créent des comptes sans parent, donc des
   racines) : (a) plan PME complet, tous ses comptes de bilan, de produits et de charges à solde non nul **en
   devise**, **niveaux 2 et `all`**, `header` puis `footer` : réponse **sous 23 552 o** (marge des autres outils) ; si
   `all` en `footer` dépasse, le chiffre mesuré s'écrit et la description le dit ; (b) sur une fixture synthétique
   (comptes en devise, noms de 34 caractères, groupes posés par SQL), niveau `all`, `footer` : la taille de l'élément
   et le **nombre de lignes groupées** au-delà duquel la réponse est refusée pour taille (même refus que
   `plafond_au_maximum`) ; ce nombre s'écrit dans la doc du module (`rapports.rs:1-13`, avec sa date) et dans les descriptions des deux outils
   (« au-delà d'environ N lignes, choisissez un niveau plus bas » : N n'est connu qu'après la mesure, les descriptions
   se complètent donc à T3) ; (c) un test affirme le refus au-delà du seuil.
   Le seuil « environ 60 comptes » du bilan plat est conservé.
6. **AC-6 — Gardes réelles.** Verts : `mcp_contrat` (forme de `inputSchema`, K7), `mcp_gardes_lexicales`
   (`champ_sur_en_un_lieu` : aucun `ChampSur` ni `scelle::` dans `rapports.rs` ; `sorties_sans_serialisation_parallele` :
   le miroir ne sérialise que par `donnees` ; `donnees_construites_par_la_politique_seule`), `mcp_matrice_manuel`
   (n'apparie que **portée × outil** : la phrase du manuel n'est contrôlée par aucune garde, la revue la lit),
   `mcp_outils_e2e` (`bornes_des_schemas` inchangé : le bilan et le compte de résultat ne sont pas paginés ;
   `proprietes_du_schema_acceptees` exige que l'argument « complet » de `TEMOINS`
   (`tests/common/mcp_pieges.rs:186` ; entrées `kesh_bilan` `:246` et `kesh_compte_de_resultat` `:254` sur
   `story/46-1` à `de0a582f`) nomme **chaque** propriété du schéma et soit servi : celui de `kesh_bilan` et de
   `kesh_compte_de_resultat` gagne `detailLevel`, `subtotalPosition`, `open` et `closed` — deux listes disjointes —,
   `mcp_enveloppe_e2e` parcourt `TEMOINS` mais ne lit que ses **jeux** (`for (outil, jeux, _)`, `:654`, `:974`, `:1291`),
   jamais l'argument « complet » : il n'est pas touché) ; un cas
   groupé ajouté à `mcp_outils_e2e.rs` (arguments valides, un refus par contrainte de l'AC-1, `groupes` non nul, listes
   plates vides, témoin hors groupe où `groupes` vaut `null`). *(« Longueur des descriptions » : aucune garde ne la
   contrôle — retiré, validation P1.)*
7. **AC-7 — Documentation.** Manuel d'administration FR, tableau des outils MCP (troisième colonne de `kesh_bilan` et
   `kesh_compte_de_resultat`) : une phrase sur les arguments de groupement et le seuil de l'AC-5 ; `.tex` seul.

## Tâches / sous-tâches

- [x] **T0** — Reprise de la branche d'epic (46-1 fusionnée ; `Rapport<AvecGroupes<…>, U>`, appelants du MCP à `:305`, `:357` selon la 46-1) ; état de la 47-3 (champ de miroir) ; ligne lue citée.
- [x] **T1 — arguments et schéma** (AC 1, 4) : type de `detailLevel`, tableaux, `SCHEMA_PERIODE_GROUPES`, fonction
      de conversion unique (réutilisée par la 46-6a).
- [x] **T2 — sortie** (AC 2, 3) : `GenreLigne` (miroir + `champ_enum!`), `sortie!` de la ligne groupée et de `groupes`,
      conversion depuis la ligne de la 46-1 ; littéral de base du témoin (AC-2) **écrit avant T1**.
- [x] **T3 — tests** (AC 2, 5, 6) ; **T4 — manuel** (AC 7).
- [x] **T5 — gate complet** ; E2E complet au dernier commit de code.

## Dev Notes

### État du code (relevé le 2026-11-20 à `origin/main` `d29aa557`)

`crates/kesh-api/src/mcp/outils/rapports.rs` (372 lignes) : `ArgumentsPeriode` (`:40`, bilan et compte de résultat
seuls), `ArgumentsBalance` (`:50`, propre à la balance, paginée, schéma en ligne `:208-209`), `SCHEMA_PERIODE` (`:60`),
`LigneDeSolde` (`:69`), `Bilan` (`:87`), `CompteDeResultat` (`:103`), `ligne`/`lignes` (`:161-177`), `resume_periode`,
`demande`. ⚠️ `ArgumentsPeriode` n'est pas propre au bilan et au compte de résultat : `kesh_balance` en **construit**
une (`:225-229`) depuis `ArgumentsBalance` pour réutiliser `resume_periode` et `demande`. Ajouter quatre champs à
`ArgumentsPeriode` oblige donc à les poser à `None` à `:225` dans cette story ; à la 46-6a, la balance y recopie ses
quatre champs, et la fonction de conversion unique (T1), qui lit `&ArgumentsPeriode`, la sert sans autre code (KISS). Les erreurs de service passent par `depuis_app_error` (`mcp/textes.rs:311`), dont `Validation` rend un
texte sans l'argument. La 47-3, si fusionnée, ne change que les descriptions de ces deux outils (une phrase sur le
miroir) et n'ajoute aucun champ (C-47-3-3) : garder les deux phrases.

### Choix et écarts

- **C-46-4-1** : listes plates vidées en mode groupé, pour la taille ; *écarté* : les deux à la fois (le plat seul
  atteint ≈ 60 lignes).
- **C-46-4-2** : `open`/`closed` en tableaux JSON au MCP (listes séparées par des virgules à l'API REST) : même
  sémantique, forme idiomatique de chaque canal ; la validation se fait sur des valeurs typées, une seule fois.
- **C-46-4-3** : **pas de pagination** du bilan ni du compte de résultat groupés (comme à plat) : c'est le niveau
  qui borne la taille, et il est l'objet de la story ; le refus pour taille dit déjà de le baisser. *Écarté* :
  `limit`/`offset` sur les lignes groupées — un groupe et son sous-total en pied tomberaient dans deux tranches, pour
  une réponse qui reste petite aux niveaux utiles. (La balance, dont la ligne pèse 540 o, est tranchée : 46-6a.)
- **C-46-4-4** : le genre de ligne reprend les codes de l'API, non le français du CSV.
- **C-46-4-5** (2026-11-21, validation P1 de la 46-6a, F2) : **`groupes: null` est admis** sans argument neuf, au lieu
  d'omettre le champ : la convention C-37-9 veut un champ présent à `null` (précédent de la 37-4 au MCP). *Écarté* : deux structures de sortie, avec et sans `groupes` (une forme de plus à tenir pour un seul champ). Le littéral du commit de base n'est pas réécrit :
  le test y ajoute la seule clé neuve (AC-2). Même décision à la 46-6a (C-46-6a-5).

### References

Fiche 46-1 (§ *Le contrat*) ; `CLAUDE.md` § *Le prompt d'une passe doit nommer le manuel*.

## Note de version

- **Added** — Les outils MCP `kesh_bilan` et `kesh_compte_de_resultat` présentent, sur demande, le bilan et le compte de
  résultat par groupes de comptes, comme à l'écran : `detailLevel` (1 à 20 ou `"all"`), `open` et `closed` (tableaux
  d'identifiants de groupes), `subtotalPosition` (`header` ou `footer`) ; la réponse porte alors un champ `groupes`
  (lignes groupées par section, écho des choix retenus) et ses listes plates sont vides. Sans ces arguments, la sortie
  ne change qu'au champ `groupes`, présent à `null`. Au-delà d'environ 60 lignes groupées, la réponse est refusée pour
  sa taille : choisir un niveau plus bas.

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (`bmad-dev-story`), 2026-11-21.

### Debug Log References

Journaux hors dépôt, `kesh-gate-logs/s46-4/` : `t0-capture.log` (littéral de base), `cible-*.log` (gate ciblé),
`mutations.log`, `gate-backend-1-env.log` et `gate-backend.log`, `front-*.log`, `e2e*.log`, `gate-ci*.log`.

### Completion Notes List

- **T0** (ligne lue, `rapports.rs` au commit `ed822f21`) : `let r = ctx.afficher(rapport).await.rapport;` aux appelants
  du bilan et du compte de résultat (« Sans groupes jusqu'à la Story 46-4 ») ; `services::rapports` expose
  `bilan_par_groupes`, `compte_de_resultat_par_groupes` et `DemandeGroupes::depuis_valeurs` (entrée typée,
  `NiveauDemande`, `PositionSousTotal`) ; la 47-3 n'est pas dans `epic/46-rapports-groupes` (aucune phrase de miroir
  dans les descriptions à garder). Le **littéral de base** de l'AC-2 a été écrit et commité **avant** tout code
  (`767b81fa`) : `tests/fixtures/mcp-groupes/kesh_bilan.txt` et `kesh_compte_de_resultat.txt`, texte rendu entier
  (en-tête compris), jetons `{debut}`, `{fin}`, `{exercice}` ; le test passait alors, à l'octet, sur le code d'avant.
- **T1** : `ArgumentsPeriode` gagne `detail_level: Option<NiveauArgument>` (`#[serde(untagged)]` : entier JSON, ou
  `Tous::All` = `"all"` ; tout autre JSON refusé à la lecture), `subtotal_position: Option<PositionSousTotaux>`,
  `open`/`closed: Option<Vec<i64>>` ; `deny_unknown_fields` gardé, aucun `flatten`. `kesh_balance` les pose à
  `None`. **Une seule conversion**, `groupes_demandes(&ArgumentsPeriode)`, appelle
  `DemandeGroupes::depuis_valeurs` (aucune validation locale) et passe ses refus par `depuis_app_error` (« Arguments
  invalides. »). `SCHEMA_PERIODE` → `SCHEMA_PERIODE_GROUPES` (`oneOf`, tableaux `maxItems: 500`). Résumé d'appel :
  `detailLevel` (`entier`, ou `code(…, "all")`), `subtotalPosition` (`code`), `openCount`/`closedCount` (nombres).
- **T2** : miroirs `GenreLigne` (`group`/`account`/`own`/`subtotal`) et `PositionSousTotaux` (`header`/`footer`, lu en
  argument et rendu dans l'écho), `match` exhaustifs, admis par `champ_enum!` ; `sortie!` `LigneGroupee`,
  `GroupesBilan`, `GroupesResultat` ; `Bilan.groupes` et `CompteDeResultat.groupes` (`null` sans `detailLevel`) ;
  listes plates vides en mode groupé ; totaux, report, résultat, équation inchangés. L'écho passe par une structure
  privée `Echo` (les deux sorties ne diffèrent que par leurs sections ; `sortie!` n'admet pas `flatten`). La phrase des
  descriptions est commune (`description_groupes!`, `concat!`), le seuil est la constante `SEUIL_LIGNES_GROUPEES` (60),
  contrôlée contre les deux descriptions par un test.
- **T3 — tests** (recomptés : `mcp_outils_e2e.rs` 34 → 39 attributs de test de `ed822f21` à `HEAD` ; `rapports.rs`
  0 → 4 `#[test]`) : `sans_groupes_sortie_du_commit_de_base` (AC-2, le littéral de `767b81fa` + `groupes: null`,
  re-sérialisé par `to_value` puis `to_string_pretty`, texte entier comparé) ; `groupes_par_le_mcp` (AC-3 : témoin
  hors groupe, niveau 1 `header`, `all` `footer` avec un groupe ouvert à la main et des identifiants inconnu, nul et
  négatif ignorés, clés exactes des lignes, totaux inchangés) ; `groupes_arguments_refuses` (AC-1 : 17 refus — dont
  les formes que seule la lecture refuse —, 7 formes valides servies) ; `tailles_groupees_plan_pme` et
  `tailles_groupees_seuil` (AC-5) ; tests unitaires `lecture_de_detail_level`, `conversion_unique`,
  `resume_des_groupes`, `descriptions_des_groupes`. `TEMOINS` (argument « complet ») de `kesh_bilan` et
  `kesh_compte_de_resultat` : `detailLevel: 2`, `subtotalPosition: "footer"`, `open: [K1]`, `closed: [CR]`.
- **AC-5, mesures** (2026-11-21, `cible-nextest.log`) : plan PME (52 comptes imputables sans rôle ni écriture passés en
  USD et mouvementés, les autres tels que la société d'essai les laisse) — bilan niveau 2 : 8 lignes / 3 525 o
  (`header`), 10 / 4 160 o (`footer`) ; `all` : 32 / 11 594 o, 38 / 13 598 o ; compte de résultat niveau 2 : 27 /
  9 879 o, 31 / 11 223 o ; `all` : 39 / 13 965 o, 46 / 16 343 o — tous sous 23 552 o. Fixture synthétique (groupe posé
  par SQL, comptes USD de 34 caractères, `all`, `footer`, un compte ajouté à la fois) : élément 356 o, **67 lignes
  groupées servies au plus (24 389 o)**, refus `TEXTE_ENVOI_TROP_VOLUMINEUX` à la suivante. Seuil écrit : « environ
  60 lignes groupées » (borne basse, comme le bilan plat), dans la doc du module, les deux descriptions, le manuel et
  `docs/api-external.md`. Écart à l'estimation de la fiche (43 et 52 lignes à `all` en `footer`) : 38 et 46 mesurées,
  les comptes à rôle n'ayant pas été mouvementés.
- **Sondes par mutation** (`mutations.sh`, copie `git archive` hors dépôt, cible à part, supprimées ; filtre : tests
  unitaires de `rapports.rs` et tests `/groupes/` de `mcp_outils_e2e`, 7 tests, verts sans mutation) — **9 sur 9
  rouges** : listes plates gardées en mode groupé (`groupes_par_le_mcp`) ; listes plates vidées sans groupes
  (`groupes_par_le_mcp`, `sans_groupes_sortie_du_commit_de_base`) ; `closed` ignoré par la conversion
  (`conversion_unique`, `groupes_arguments_refuses`) ; `subtotal` rendu `account` (`groupes_par_le_mcp`) ;
  `tousLesNiveaux` faux pour `all` (`groupes_par_le_mcp`) ; résumé sous la clé `open` (`resume_des_groupes`) ; seuil
  désaccordé des descriptions (`descriptions_des_groupes`) ; sortie plate changée (`report`)
  (`sans_groupes_sortie_du_commit_de_base`) ; `footer` lu `header` (`conversion_unique`, `groupes_par_le_mcp`).
- **Gate ciblé** (`gate-cible.sh`, au commit de T1–T3) : fmt, clippy workspace `-D warnings` verts ; nextest
  `mcp_outils_e2e`, `mcp_contrat`, `mcp_gardes_lexicales`, `mcp_matrice_manuel`, `mcp_enveloppe_e2e`,
  `mcp_traduction` et les tests unitaires de `rapports.rs` : 110 passés.
- **Gate complet n° 1** (`5de70260` + docs, `gate-backend-1-env.log`) : 6195 exécutés, 6194 passés, **1 échec** —
  `configuration_transmise::lectures`, qui prenait la variable locale `env` (l'enveloppe de la 46-1) pour une lecture
  de l'environnement (C75). Corrigé par renommage (`9cf82948`), aucune exception ajoutée à la garde.
- **T4** : manuel d'administration FR (tableau des outils MCP, `.tex` seul, PDF non régénéré : règle des branches
  d'epic) et `docs/api-external.md` (paragraphe « Au serveur MCP » sous la section de la 46-1).
- **Gates finaux, au dernier commit de code `9cf82948`** (journaux `kesh-gate-logs/s46-4/`) :
  - backend, `scripts/test-fast.sh --ci` (base remise à zéro avant) : fmt et clippy `-D warnings` verts ; nextest
    **6195 exécutés, 6195 passés** (1 instable : `kesh-db::pool_lecture_seule
    g_une_instruction_trop_longue_est_interrompue_par_le_serveur`, rouge au premier essai, vert au second — délai sous
    charge, hors périmètre), 14 ignorés (`gate-backend.log`) ;
  - frontend (inchangé par la story) : `npm run check` 0 erreur (29 avertissements, préexistants),
    `lint-i18n-ownership` vert, Vitest **4627 passés** (278 fichiers), `npm run build` vert (`front-*.log`) ;
  - **E2E complet** (`LANG=fr_CH.UTF-8`, binaire de `9cf82948`) : **478 passés, 10 échecs, 18 ignorés** — les dix
    attendus, jugés un à un : les sept de la KF-029 (#97 : `mode-expert.spec.ts:26`, `:41`,
    `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150`),
    `backup-restore-new-install.spec.ts:262`, `:381` et `opening-balances.spec.ts:156` (`e2e.log`) ;
  - **`scripts/gate-ci.sh`** sur une copie `git archive` de `9cf82948` : passage 1 arrêté au 89e binaire de test sur
    257 par `mcp_transport_e2e::abandon_au_delai_le_permis_reste_pris` (témoin de délai « sonde servie », sensible à
    la charge, hors périmètre ; vert au nextest complet et au passage 4) (`gate-ci-1-arret-transport.log`) ; passages
    2 et 3 **interrompus par l'orchestrateur** (station sous 5 Go libres, mémoire d'échange pleine ; aucun défaut de
    code) ; **passage 4, jusqu'au bout : 257 binaires, 6214 passés, 0 échoué, 18 ignorés** (`gate-ci.log`). ⚠️ Les
    passages 2 à 4 ont joué `cargo test` avec **`--no-fail-fast`**, posé par l'enveloppe `gate-ci-wrap/cargo`, pour
    jouer tout le workspace malgré un binaire rouge ; la commande de la CI est sans ce drapeau, le résultat est le même
    quand tout est vert. Cible du gate-ci supprimée après chaque passage.

### File List

- `crates/kesh-api/src/mcp/outils/rapports.rs` (modifié)
- `crates/kesh-api/tests/mcp_outils_e2e.rs` (modifié)
- `crates/kesh-api/tests/common/mcp_pieges.rs` (modifié)
- `crates/kesh-api/tests/fixtures/mcp-groupes/kesh_bilan.txt` (nouveau)
- `crates/kesh-api/tests/fixtures/mcp-groupes/kesh_compte_de_resultat.txt` (nouveau)
- `docs/manual/fr/admin-manual.tex` (modifié)
- `docs/api-external.md` (modifié)
- `_bmad-output/implementation-artifacts/46-4-groupes-mcp.md`, `_bmad-output/implementation-artifacts/sprint-status.yaml`

## Change Log

- 2026-11-20 — Fiche écrite depuis l'esquisse : arguments aux noms de l'API, sortie groupée sous le plafond de taille,
  résumé d'appel ; la balance passe à la 46-6a. Sept critères. `drafted`.
- 2026-11-20 — **validation P1 (Sonnet 5.5) : 0 C, 1 H, 5 M, 1 L, remédiés.** H : l'AC-5 exigeait le plan PME complet au
  niveau `all` sous le plafond de 24 576 o — inatteignable (≈ 350 o la ligne, 57 comptes = 20 156 o) : mesures par
  niveau, seuil écrit et refus affirmé. M : vocabulaire de `genre` aligné sur les miroirs des outils (`champ_enum!`,
  codes de l'API) ; lecture des arguments (aucun tableau ni `oneOf` au MCP, `serde` seul, pas de `flatten` avec
  `deny_unknown_fields`, refus sans nom d'argument) ; garde « longueur des descriptions » inexistante (gardes réelles
  listées, `mcp_matrice_manuel` n'apparie que portée × outil) ; témoin « avant et après » devenu littéral du commit de
  base ; `SCHEMA_PERIODE` ne sert pas la balance (schéma renommé). Propagation : **46-4 avant 46-6a** déclarée (finding de la 46-6a). L :
  clés du résumé d'appel. Axes exercés : code de `origin/main` (rapports.rs, sortie.rs, outil.rs, textes.rs, tests
  MCP), concordance 46-1 à 46-3, 46-6a, manuel d'administration ; non exercés : compilation, mesures de taille réelles.
- 2026-11-20 — **validation P2 (Claude Opus 5.5, contexte frais, au code de `origin/main` `d29aa557`) : 0 C, 0 H, 3 M,
  4 L, remédiés.** M : prémisse fausse de la remédiation P1 à l'AC-5 (« le plan PME complet au niveau `all` dépasse le
  plafond », en comptant les 86 comptes dans un seul rapport) — le bilan n'en porte que 31, le compte de résultat 34 ;
  calcul : 43 et 52 lignes groupées à `all` en `footer`, sous le plafond attendu → mesures aux niveaux 2 et `all` sur
  le plan PME, seuil sur fixture synthétique ; témoin littéral de l'AC-2 dépendant de la date du jour de l'horloge de
  test (`societe_piegee`, `aujourd_hui_test`, `KESH_TEST_INSTANT`) → dates en jetons ; `proprietes_du_schema_acceptees`
  et l'argument « complet » de `TEMOINS` (`tests/common/mcp_pieges.rs:186`) omis des gardes (AC-6). L : `≥ 1` n'est pas
  un refus de la 46-1 (identifiant ignoré) ; tout refus rend « Arguments invalides. » (`textes.rs:127`), sans argument
  nommé ; `positionSousTotaux` miroir `champ_enum!` ; `ArgumentsPeriode` construit aussi par `kesh_balance`
  (`rapports.rs:225-229`) → champs à `None`, et conversion unique sur `&ArgumentsPeriode` réutilisée par la 46-6a ;
  `Resume` a aussi `recherche_fournie`. Rapport : `kesh-gate-logs/epic-46/validation/46-4-P2.md`.
- 2026-11-20 — **validation P3 ciblée (Claude Sonnet 5.5, contexte frais, sur le seul commit `bb673929`, au code de
  `origin/main`) : 0 C, 0 H, 0 M, 3 L, remédiés.** Recompté et confirmé : 31 et 34 feuilles du plan PME (`pme.json`, 86
  comptes dont 21 groupes), 43 et 52 lignes groupées à `all` en `footer` (script d'arbre avec dissolution : 6, 7, 70, 8,
  80 et `2`), `MCP_ENVOI_OCTETS_MAX` 24 576 (`envois_mcp.rs:55`), marge 23 552 (`plan_comptable.rs:7`),
  `aujourd_hui_test` (`tests/common/jour.rs:14`), `proprietes_du_schema_acceptees` (`mcp_outils_e2e.rs:234`),
  `TEMOINS` (`mcp_pieges.rs:186`), `depuis_app_error` (`textes.rs:311`). L : `mcp_enveloppe_e2e` ne lit pas l'argument
  « complet » de `TEMOINS` (seulement ses jeux) — l'affirmation de la P2 retirée ; « 31 comptes de bilan » précisé
  (feuilles) ; descriptions de l'AC-5 (b) complétées à T3. Rapport : `kesh-gate-logs/epic-46/validation/46-4567-P3-ciblee.md`.
- 2026-11-21 — Propagation de la validation P1 de la 46-6a (F2, `kesh-gate-logs/s46-specs/validation-p1-rapport.md`) :
  l'AC-2 (« réponse identique ») contredisait l'AC-3 (« `groupes` à `null` sinon ») → C-46-4-5 : `groupes: null`
  admis, littéral de base inchangé, la clé ajoutée par le test avant comparaison ; entrées de `TEMOINS` citées
  (F10). Statut inchangé ; la passe P2 de la 46-6a relit ces lignes.
- 2026-11-21 — **Validation** (boucle close) : P1 (Claude Sonnet 5.5, 0 C / 0 H / 0 M au-dessus de LOW ; tendance 0/0/2/10 → 0/0/0/10) puis P2 (Claude Opus 5.5, contexte frais, à `e61c9029`) 0 C / 0 H / 0 M / 10 L sur les trois fiches (46-3 : 4, 46-6a : 5, 46-4 : 1) ; aucun finding au-dessus de LOW. LOW de la P2 appliqués dans la fiche : G5 (note de version), G8 (motif de C-46-4-5), G10 (aucun champ de miroir au MCP, C-47-3-3). Rapport : `kesh-gate-logs/s46-specs/validation-p2-rapport.md`. `ready-for-dev`.
- 2026-11-21 — **Développement** (`bmad-dev-story`, Claude Opus 5.5) : littéral de base de l'AC-2 commité avant le code
  (`767b81fa`) ; arguments, conversion unique, sortie groupée, résumé d'appel, tailles mesurées et seuil écrit (60
  lignes groupées, borne basse ; 67 servies au plus sur la fixture synthétique) (`5de70260`) ; manuel d'administration
  et `docs/api-external.md` (`9833c5a8`) ; variable `env` renommée pour la garde des lectures d'environnement
  (`9cf82948`). Tests : 5 d'intégration et 4 unitaires ajoutés ; 9 sondes par mutation sur 9 rouges. Gates complets
  verts au dernier commit de code (détail au Dev Agent Record). `review`.
- 2026-11-22 — **remédiation de la revue de code P1 (Sonnet 5.5 ; 0 C, 0 H, 1 M, 5 L), commit `e94b391a`.** M1 : test
  `groupes_lignes_fideles_au_plat` (`mcp_outils_e2e.rs`) — chaque ligne de compte groupée (`all`) comparée à la ligne plate
  de même `compteId` (`numero`, `nom`, `solde`, `devise`, `soldeDevise`), dont un compte en USD (100.00), et `ouvert` vrai
  pour un groupe ouvert à la main (bilan, niveau 1) ; **sondes par mutation** dans une copie hors dépôt
  (`kesh-gate-logs/s46-4/mutations-p1.sh`, `mutations-p1.log`) : sans mutation 1 passé ; `solde` mis à zéro dans
  `lignes_groupees` : test rouge ; `devise: None` : test rouge. L1 : `minimum` retiré des éléments de `open` et `closed` du
  schéma (indicatif ; le code et l'API REST ignorent un identifiant inconnu, nul ou négatif). L2 : « une trentaine de
  caractères » écrit à côté du seuil (doc de module) ; la description d'outil est inchangée. L3 : commentaire du test et
  doc de module disent « hors comptes à rôle » (le plan PME mesuré ne les mouvemente pas). L4 : `1e1`, `-0`, `1.0`,
  `9223372036854775808`, `18446744073709551616` ajoutés aux refus de `lecture_de_detail_level`. L5 : doc de module
  réécrite en deux paragraphes (plan PME, fixture synthétique). **Gates** : fmt, clippy workspace `-D warnings`, nextest
  `-p kesh-api` sur `mcp_*`, `textes_coherents` et les tests de lib `mcp::` : 292 passés, 0 échoué (`p1-nextest.log`).
  **Gate complet rejoué ensuite** (voir l'entrée suivante).
- 2026-11-22 — **gate complet au dernier commit de code `e94b391a`** (HEAD `4145cb04`, plus la fiche seule) : `scripts/test-fast.sh --ci`
  (fmt, clippy workspace `-D warnings`, nextest) via `heavy.sh` : **6196 exécutés, 6196 passés (3 lents), 14 ignorés**, rc 0
  (`gate-backend.log`). **`gate-ci.sh` : à la fin de l'epic** (règle des gates de l'epic, 2026-11-22) ; pas d'E2E (aucun
  écran). **Passe P2 ciblée (Haiku, sur `e94b391a` seul) : 0 C, 0 H, 0 M, 2 L** (`revue-p2-rapport.md`). F1 : AC-1 annotée
  (`minimum: 1` retiré). F2 : le test de fidélité ne couvre que les lignes de compte, non les groupes ni les sous-totaux ;
  tracé dans `issues-a-ouvrir.md` (`technical-debt`, P4). **Tendance : P1 (Sonnet) 0 C / 0 H / 1 M / 5 L → P2 ciblée
  (Haiku) 0 / 0 / 0 / 2** ; remédiation P1 sans MEDIUM résiduel. **Story `done`.**

