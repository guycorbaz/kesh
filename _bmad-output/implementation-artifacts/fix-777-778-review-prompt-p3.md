# Revue de code P3 — issues #777 et #778 : passe ciblée sur la remédiation de la P2

Prompt versionné. **Passe ciblée** : une seule lentille, braquée sur le **seul commit de remédiation** de la P2.
Modèle : **Sonnet**, contexte frais, lecture seule.

## Objet

Worktree : `/home/gcorbaz/devel/kesh-777`, branche `fix/777-778-grand-livre-exercices`.
**Commit à relire : `1ad56c90`** — `git -C /home/gcorbaz/devel/kesh-777 show 1ad56c90`.
Contexte (à lire, non à revoir) : le prompt et le rapport de la P2
(`fix-777-778-review-prompt-p2.md`, même dossier ; `/home/gcorbaz/devel/kesh-gate-logs/777-review-p2-R.md`).

La P2 (Opus, passe ciblée sur `772f7018`) a rendu 0 CRITICAL, 0 HIGH, 2 MEDIUM, 3 LOW. Le commit `1ad56c90`
remédie :

1. **MEDIUM P2-R-2 (production)** — description de l'outil MCP `kesh_grand_livre`
   (`crates/kesh-api/src/mcp/outils/grand_livre.rs`, `fn description`). Elle niait sans condition que `cloture`
   vaille l'ouverture plus les totaux ; or, sur une période d'un seul exercice, `generate` calcule exactement
   ouverture + totaux (`crates/kesh-report/src/general_ledger.rs`, branche `else` de
   `if !is_bilan(acc.account_type) && franchit_un_exercice`). Nouvelle formulation : sur un compte de produits ou
   de charges, `cloture` est le solde de l'exercice en cours à la fin de la période ; il vaut l'ouverture plus les
   totaux, sauf quand la période franchit une fin d'exercice ; sur un compte d'actif ou de passif, toujours
   l'ouverture plus les totaux. Le test `la_description_definit_cloture_et_ruptures` asserte la condition et la
   phrase du bilan.
2. **MEDIUM P2-R-1 (documentation)** — `docs/api-external.md`, paragraphe « Le Grand livre et les fins
   d'exercice » : « `limit` absent » n'était pas une lecture entière (la route plafonne à `MAX_LEDGER_LIMIT` = 500,
   `crates/kesh-api/src/services/rapports.rs`, `fn grand_livre`). Remplacé par : toutes les pages réunies, `lineCount`
   disant combien de lignes ; sans `limit`, une page en porte au plus 500.
3. **LOW** — P2-R-3 (la première phrase du même paragraphe couvre les ruptures de tête et finale) ; P2-R-4
   (`crates/kesh-i18n/locales/de-CH/messages.ftl`, majuscule après le deux-points) ; P2-R-5 (doc-comment de module
   de `general_ledger.rs`, lignes recoupées).

Journal des mutations : `/home/gcorbaz/devel/kesh-gate-logs/777-mutations.log` (M5 : condition retirée de la
description, test tué l. 207).

## Lentille — Regression Hunter

*La sévérité se déplace vers ce qu'on vient d'écrire.* Cherche ce que **ce commit** a cassé ou laissé faux.

- **R1 — la description MCP, contre le code.** Chaque proposition est-elle vraie pour chaque cas de `generate` :
  compte de résultat sur un seul exercice ; à cheval sur deux ou trois exercices ; `from` hors de tout exercice
  (`exercice_initial = None`, `franchit_un_exercice` vrai dès qu'un exercice final existe : `cloture` vaut-il alors
  ouverture + totaux ?) ; `to` après le dernier exercice ; compte de bilan. Cite les lignes de `general_ledger.rs`
  (`franchit_un_exercice`, calcul de `closing`). Une phrase qui sur-promet ou sous-promet est un défaut.
- **R2 — le test.** Les sous-chaînes assertées existent-elles dans la chaîne **après** l'effet des `\` de fin de
  ligne Rust (qui suppriment le saut et l'indentation suivante) ? Le test peut-il passer si la condition disparaît ?
  La mutation M5 est-elle la bonne ?
- **R3 — `docs/api-external.md`.** Les phrases neuves disent-elles vrai au code : plafond de 500 lignes sans `limit`
  (`rapports.rs`), `limit` au-delà de 500 écrêté (`LedgerOptions::limit`, `MAX_LEDGER_LIMIT`), `lineCount` avant
  pagination, ruptures de tête et finale, l'export (`ExportFormat`) qui, lui, rend tout ? La phrase reste-t-elle
  lisible et cohérente avec le reste du paragraphe ?
- **R4 — locale et doc-comment.** La valeur de-CH corrigée reste-t-elle une valeur Fluent valide ? Le doc-comment
  recoupé dit-il la même chose qu'avant ?
- **R5 — propagation.** La négation inconditionnelle (« non l'ouverture plus les totaux » et ses variantes) ou la
  confusion « `limit` absent = lecture entière » subsistent-elles ailleurs : code, `docs/`, manuels
  `docs/manual/fr/*.tex` et leurs PDF aplatis
  (`pdftotext /home/gcorbaz/devel/kesh-777/docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`), CHANGELOG ?
- **R6 — gardes.** La nouvelle description touche-t-elle une garde (`mcp_gardes_lexicales.rs`,
  `mcp_matrice_manuel.rs`, descripteurs figés de `mcp_transport_e2e.rs`, taille des réponses `tools/list`) ?

## ⛔ Interdits

- **N'exécute JAMAIS `scripts/prepare-release.sh`**, ni directement ni par aucun détour. N'exécute aucun script du
  dépôt, ni `cargo`, `npm`, `npx`, `sqlx`, `docker`, `mariadb`/`mysql`, `make`/`latexmk`.
- **Aucune redirection** (`>`, `>>`, `<`, `tee`, here-doc) : compare par pipe, lis vers la sortie standard.
- **Ni `rm` sous aucune forme, ni `bc`.**
- N'écris, ne modifie, ne crée aucun fichier — **sauf ton rapport**, écrit avec l'outil d'écriture de fichiers.
  Aucune commande git ou `gh` qui écrit. Aucun `pkill`/`kill`.
- N'ouvre jamais `~/kesh-test/`.
- Commandes autorisées : `git show`, `git diff`, `git log`, `grep`, `sed -n`, `cat`, `pdftotext` (vers `-`), `tr`,
  lecture de fichiers.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/777-review-p3-R.md` :

1. **Axes exercés** et **axes NON exercés** — R1 à R6, liste explicite. Un « 0 finding » sans cette liste ne compte
   pas.
2. Findings numérotés `P3-R-n`, avec sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier:ligne, description,
   preuve, correctif proposé. **Toute affirmation de présence ou d'absence s'appuie sur une sortie `grep -nF`
   copiée** ; toute affirmation sur le manuel, sur le PDF aplati. Distingue ce que `1ad56c90` introduit de ce qui
   lui préexiste.
3. Verdict : nombre de findings par sévérité, et si la remédiation suivante toucherait du code de production.
