# Prompt — validation P2 de la spec, Story 29-3 (la proposition affichée suit une priorité écrite, et l'écran la dit)

*Versionné le 2026-10-14. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 était **Sonnet** ; P2 :
**Opus** pour les deux lentilles.*

Dépôt `/home/gcorbaz/devel/kesh-29-3`, branche `story/29-3` (base `16eeed21`, kickoff de l'Epic 29).
Fiche : `_bmad-output/implementation-artifacts/29-3-meilleure-proposition.md`. Registre de la story :
`_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-3.md` (C-29-3-1 à 15).
Commits : `9f4c54a7` (fiche), `471cef30` (prompt P1), **`32cf9bc1` (remédiation P1)** — `git show 32cf9bc1` et
`git diff 471cef30 32cf9bc1` donnent exactement ce que la remédiation a écrit.
Rapports P1 (à lire, pour juger la remédiation, **non** pour les reprendre) :
`/home/gcorbaz/devel/kesh-gate-logs/29-3-validate-p1-R.md` et `…-p1-F.md` (R 0/1/5/7, F 0/0/5/7).
Epic : `_bmad-output/planning-artifacts/epic-29-preparer-cloture.md` § *29-3*. Issues : #527 (l'objet), #529 (la 29-2,
sérialisée **avant**), #665 (trop-perçu de 1 à 5 centimes, C-29-3-7), #76 (KF-026), #548 —
`gh api repos/guycorbaz/kesh/issues/<N>`. Fiche voisine de la 29-2, non mergée, en lecture :
`/home/gcorbaz/devel/kesh-29-2/_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-2.md` (C-29-2-7).
Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites
NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus, « recompter n'est pas réécrire
l'historique » ; § Règle de splitting préventif et son amendement D5 ; § Pattern batch ; § Un appariement
automatique propose, il ne crée jamais) ; `docs/agents-orchestration.md` ; rétrospective de l'Epic 28
(`epic-28-retro-2026-10-14.md` § 4 et § 7).

⚠️ **Ce qui a changé depuis la P1, et pourquoi la P2 doit d'abord regarder là.** Le HIGH R1 a été tranché par
l'orchestrateur (C-29-3-11) : toute facture qui porte un signal d'identité passe devant la règle. La remédiation en a
déduit, au code, que **l'ordre atteignable est inchangé** : la story n'ajoute plus de changement de comportement, elle
écrit l'ordre (fonction pure), le dit à l'écran, le documente pour l'API, et répare un test muet. *La sévérité ne
stagne pas, elle se déplace vers ce qu'on vient d'écrire* (`CLAUDE.md`, § *La passe ciblée*) : la P2 relit d'abord la
remédiation.

## Axes prioritaires (communs aux deux lentilles)

0. **OBLIGATOIRE — Relire ce que la remédiation P1 a écrit** (`git diff 471cef30 32cf9bc1`), ligne par ligne :
   chaque phrase neuve ou réécrite de la fiche, du registre (C-29-3-11 à 15 et les annotations de C-29-3-1, 3, 4, 6,
   7) et de `sprint-status.yaml` est-elle **vraie au code** ? Une remédiation qui a introduit un fait faux, une
   contradiction entre deux sections, un décompte inexact ou un résidu de l'ancienne règle (« facture reconnue »,
   « numéro dans la référence », trois rangs de la première rédaction, « cas 1 rouge au sol ») est un finding — au
   moins **MEDIUM** s'il guiderait le développement de travers. Les entrées **datées** du passé (première entrée du
   Change Log, texte d'origine des choix annotés) sont des relevés : elles ne se reprochent pas, sauf si elles
   étaient fausses **à leur date**.
1. **La conséquence de C-29-3-11 est-elle exacte ?** Vérifie au code que, sur toute entrée atteignable par
   `get_proposals` (`crates/kesh-api/src/routes/reconciliation.rs:586-845`), une facture qui côtoie la règle porte
   toujours un sous-score de référence ou de contact non nul (`matching.rs:62-161`, `total > 0.0` `:81`, seuil 0,5
   `:725`, `:748-750`, tolérance du dépôt), que le rang 2 est donc vide, et que M1 (retrait de l'appel) est un mutant
   **équivalent**. Cherche une entrée qui l'infirmerait (devise, débit, règle sans facture, facture d'un autre type,
   `filter_map` qui écarte une candidate, `Decimal` et arrondi au centime du reste dû, référence `""` contre `None`).
   Si la conséquence est fausse, c'est au moins **HIGH** (la fiche promet un ordre inchangé). Juge aussi l'arbitrage
   **écrit comme réversible** dans C-29-3-11 (garder la fonction contre l'alternative (b), un commentaire juste et les
   tests HTTP seuls) : l'argument tient-il, au regard du DRY et du périmètre minimal (B3 de la rétro 28) ? Un ajout
   non nécessaire est au moins **MEDIUM** — mais ne décide pas à la place de l'orchestrateur : dis ce qui est vrai.
2. **La règle C-29-3-11 elle-même** : le faux rattachement **symétrique** (risque résiduel écrit) est-il le seul ?
   Une facture de rang 0 peut-elle être refusée à l'acceptation autrement que par #665 et les refus de configuration
   cités (`accept_one_invoice` `:1621`, `RECONCILIATION_SCORE_TOO_LOW` `:1802-1811`) ? La phrase de l'écran (AC5,
   quatre langues) et les ajouts au manuel (AC7) affirment-ils **quoi que ce soit de plus fort que le code** —
   bidirectionnalité de la référence (`matching.rs:132-134`) et du contact (`:148-161`), préfixe de quatre
   caractères, tolérance de 5 centimes ? Les traductions DE/IT/EN sont-elles fidèles au fr-CH et aux termes de
   l'écran (`nav-reconciliation-rules`, `reconciliation-cols-tx-counterparty` des quatre catalogues) ?
3. **Inventaire clos des sites** qui choisissent, ordonnent ou lisent la candidate « première » : la P1 l'a fait et la
   fiche le recopie (§ *Sites qui lisent l'ordre*) — **refais-le** (`grep -rnE`, `crates/`, `frontend/src`,
   `frontend/tests`, `docs/`, `website/`, `README.md`), sans le prendre pour argent comptant.
4. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` § *Algorithme de matching* et *Accepter ou
   rejeter les propositions* (l. 2025-2060), § *Application des règles* (l. 2114-2117), FAQ (l. 2896-2912), renvois
   d'issue (`:1308`). L'inventaire de l'AC7 (cinq sites relus, deux modifiés, trois ajouts) est-il **clos**, et chaque
   verdict « vrai, inchangé » l'est-il réellement sous C-29-3-11 ? Les trois ajouts prescrits seront-ils vrais ?
   Contrôle aussi le **PDF**, aplati **et ligatures normalisées** :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | sed 's/ﬀ/ff/g;s/ﬁ/fi/g;s/ﬂ/fl/g;s/ﬃ/ffi/g'`
   (cette commande n'écrit rien : sortie standard seulement) — le contrôle prescrit par l'AC7 est-il opérant ?
5. **Documentation d'API et CHANGELOG (AC8)** : la phrase prescrite pour `docs/api-external.md:549` est-elle vraie
   (règle présente seulement si aucune facture n'atteint 0,5 ; ordre des factures ; score conventionnel) ? La forme
   `## [0.13.4] — Non publié` est-elle exactement celle qu'attend `scripts/prepare-release.sh` (**lis** le script
   `:234-262`, ne l'exécute **pas**) ? La règle « une seule section admise » est-elle bien reportée ?
6. **Recomptes** — chacun depuis la source : AC (9), tâches (T0-T7), mutations (M1-M10), choix (15), cas de l'AC2 (11),
   tests HTTP de l'AC3 (3), sites du manuel (5 relus, 2 modifiés, 3 ajouts), findings P1 (25, dont 11 MEDIUM et plus) et
   leur ventilation au Change Log, numéros de ligne cités.

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase de #527 (« triées par score, toutes origines confondues — ou une règle de priorité écrite et dite à
  l'écran ») et du § *29-3* de l'epic couverte par un AC testable ; chaque AC par une tâche ; chaque finding P1 réputé
  traité **réellement** traité dans la fiche (confronte le Change Log P1 aux rapports P1, finding par finding — un
  « traité » non suivi d'effet est un finding) ; chaque fait, chemin et numéro de ligne **revérifié** (`sed -n`,
  `grep -nF`) ; les gardes existantes (`lint-i18n-ownership`, `i18n-libelle-en-dur.test.ts`, `i18n-keys.test.ts` et
  son `sitesTotal` `:634`, `textes_coherents.rs`, `jetons-non-definis.test.ts`) ; l'AC4 et le Pattern batch ; le
  § *Zones partagées avec la 29-2* (lis la fiche et le registre de la 29-2 dans son worktree : les zones annoncées
  sont-elles les bonnes ?).
- **F — Faisabilité et exactitude (adversaire de la conception)** : `candidate_rank` et `order_candidates` (AC1)
  réalisables tels quels (types, `match` exhaustif sans `_`, comparaisons `> 0.0` et `== 1.0` sur `f64`, lint
  `clippy` du dépôt) ; **recalcule chacun des onze cas de l'AC2**, et pour **chaque** mutation M1-M10 la liste des
  cas et tests qu'elle fait rougir (la table de l'AC9 et sa note de recalcul sont-elles justes ? une mutation
  déclarée tuée par un cas qui reste vert est au moins **MEDIUM**) ; le cas 11 (51 éléments) tue-t-il plausiblement
  `sort_unstable_by_key` — raisonne sur l'algorithme de la bibliothèque standard en vigueur (`rust-toolchain`,
  `Cargo.toml`), sans rien exécuter, et dis-le comme raisonnement ; les trois tests HTTP de l'AC3 sont-ils montables
  **tels qu'écrits** (`setup_company` et le nom de contact « `<label>` Client », `make_new_tx` et la référence `""`,
  `account_ids`, `create_rule_ok` à ajouter, corps de règle, scores exacts 0,1 et 0,4, règlement partiel 99.99 sur
  100.00, test muet réécrit avec score ≥ 0,5) ; les fabriques Vitest de l'AC4 compatibles avec les types du frontend
  (`ReconciliationCandidate` côté TS) ; la condition `proposals.some(…)` de l'AC5 cohérente avec les états de la page
  (chargement, erreur, vide, toutes lignes sans candidate) ; l'E2E de l'AC6 montable sur la base partagée (aides
  `api-fixtures.ts:30`, `:100`, relevé CAMT, valeurs uniques, nettoyage) ; une mutation évidente manque-t-elle ?

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée ; pour chaque finding, **dis s'il naît de la
remédiation P1** (`32cf9bc1`) ou de la fiche d'origine — c'est ce qui décide du découpage (D5). Toute affirmation de
présence ou d'absence = sortie de `grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés** — un
« 0 finding » sans elle ne compte pas comme passe. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/29-3-validate-p2-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/29-3-validate-p2-F.md` (lentille F) ; dernier message : le chemin du rapport,
le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/` — et là, seulement ton rapport. Aucune commande
qui écrit, compile ou exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`,
`scripts/mem-guard.sh`, `scripts/install-hooks.sh`), `make`, `latexmk`, `xelatex`, **`git checkout` et `git switch`
sous aucune forme** (ni `--detach`, ni sur un fichier, ni dans un autre worktree),
`git commit`/`add`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`/`worktree`/`restore`, `cargo`, `rustc`,
`npm`, `npx`, `node`, **tout interpréteur** (`python3`, `python`, `perl`, `ruby`, `bash`/`sh` sur un script, `awk` qui
écrit), `sed -i`, `docker`, **`mariadb`/`mysql`** et toute requête SQL, `gh` en écriture (`issue`, `pr`, `comment`,
`api -X POST|PATCH|PUT|DELETE`), `pkill`/`killall`. Aucune donnée réelle de Guy ne se lit ni ne se cite. Autorisés :
lecture, `grep`, `sed -n`, `git log`/`show`/`diff`/`blame`, `gh api` en lecture, `pdftotext` vers la sortie
standard, `tr`, `sed` sans `-i` dans un tube.
