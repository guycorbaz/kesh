# Prompt — revue de code P1, Story 29-3 (la proposition affichée suit une priorité écrite, et l'écran la dit)

*Versionné le 2026-10-14. Trois lentilles — B (Blind), E (Edge cases), A (Acceptance) —, contexte frais chacune,
lecture seule stricte. Modèle : **Sonnet** pour les trois (passe complète ; rotation D6 de la rétrospective 25 :
Sonnet ↔ Opus pour les passes complètes, Haiku pour la seule passe ciblée de fin de boucle).*

Dépôt `/home/gcorbaz/devel/kesh-29-3`, branche `story/29-3` (rebasée sur `main` `c36f39bc`, la 29-2 mergée). Le code
à relire : `git -C /home/gcorbaz/devel/kesh-29-3 diff 0fe08674 HEAD` (`0fe08674` = fiche validée, dernier commit avant
le développement). Langue du rapport : français.

Fiche : `_bmad-output/implementation-artifacts/29-3-meilleure-proposition.md` (AC1 à AC9, Dev Agent Record) ;
registre : `_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-3.md` (C-29-3-1 à 26 ; 19 à 26 neufs au
développement). Issue #527 (`gh issue view 527`) ; voisines #668 (29-5, départage à score égal), #665 (29-6,
trop-perçu de quelques centimes), #666 (choix manuel), #76 (KF-026, une candidate visible). Règles : `CLAUDE.md`
(§ Review Iteration Rule et sous-sections : manuel nommé, axes déclarés, sites non résolus, propagation, recompte ;
§ Pattern batch ; § Un appariement automatique propose, il ne crée jamais) ; `docs/agents-orchestration.md` § 6.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport.
- **N'exécute rien qui écrive, compile ou exécute** : ni `cargo` (aucune sous-commande), ni `npm`, ni `npx`, ni
  `make`, ni `latexmk`, ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni aucun
  script du dépôt ou du répertoire de gate ; ni `python3`, ni `perl`, ni `sed -i`, ni `docker`, ni `mariadb`.
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni de fichier, ni de branche), aucun `git` qui écrit
  (`commit`, `stash`, `reset`, `restore`, `rebase`, `tag`, `push`, `worktree`). Aucune commande `gh` qui écrit ;
  `gh issue view` est permis.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `grep`, `sed -n`, `cat`, `ls`, `wc`, `pdftotext` vers la
  sortie standard uniquement. Les deux PNG de `29-3-captures/` se regardent (lecture d'image), sans être modifiés.

## Axes communs aux trois lentilles

1. **La règle écrite** : `candidate_rank` / `order_candidates` (`crates/kesh-api/src/routes/reconciliation.rs`) —
   `match` exhaustif sans `_`, rang 0 lu sur les sous-scores et jamais sur `score.total`, tri **stable**, appel en
   **dernier** geste sur `candidates` dans `get_proposals`. Une voie atteignable par laquelle une règle passerait
   devant une facture qui porte un signal, ou par laquelle l'ordre affiché changerait par rapport à avant la story
   (C-29-3-11 : il ne doit pas changer), est au moins **HIGH**.
2. **Manuel contre code (obligatoire)** : `docs/manual/fr/user-manual.tex` — § *Algorithme de matching* (vers
   l. 2041, « score conventionnel »), § *Accepter ou rejeter les propositions* (vers l. 2049 : la raison, « La page le
   rappelle au-dessus de la liste », l'angle mort #665), le `\label{sec:faq-aucun-match}` neuf de la FAQ — **et le
   PDF** `docs/manual/fr/user-manual.pdf`, aplati, ligatures normalisées :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | sed 's/ﬀ/ff/g;s/ﬁ/fi/g;s/ﬂ/fl/g;s/ﬃ/ffi/g'`
   (portions de l'AC7 à 1, « depuis la fiche de la facture (issue » à 0, renvoi « (§21.4) » résolu, aucun « ?? »).
   Même exigence pour `docs/api-external.md` (§ *Accepter des propositions de rapprochement*, la phrase sur l'ordre
   de `candidates`) et `CHANGELOG.md` (`## [0.13.4] — Non publié`, sous-titre « Modifié »). Une promesse que le code
   ne tient pas — la phrase d'écran, le manuel, l'API — est au moins **MEDIUM** ; la phrase d'écran ne doit rien
   affirmer de plus fort que le code (C-29-3-15).
3. **Propagation** : greper les **valeurs** (`candidates[0]`, `top-1`, `la mieux scorée`, `les 2 s'affichent`,
   `priority-hint`, `order_candidates`, `#527`) sur `crates`, `frontend/src`, `frontend/tests`, `docs/`,
   `CHANGELOG.md` : reste-t-il un commentaire ou une phrase qui décrit l'ancien ordre comme un effet de bord, ou
   affirme que la règle et la facture « s'affichent » toutes deux ?
4. **Le compte rendu** (Dev Agent Record, Change Log, registre, `sitesTotal` de `i18n-keys.test.ts`) : chaque
   décompte et chaque chiffre de gate se recompte-t-il depuis la source et les journaux
   (`/home/gcorbaz/devel/kesh-gate-logs/293-*.log`, dont `293-mutations.log`) ? Les mutations déclarées tuées le
   sont-elles par les tests nommés ? M1 et sa variante M1b sont-elles démontrées équivalentes ? M9b (déclarée non
   équivalente, C-29-3-22) l'est-elle vraiment ?

## Lentille B — Blind (le diff seul, sans la fiche)

Lis le diff comme un relecteur qui ne connaît pas l'intention : Rust (comparaisons de flottants exactes — sont-elles
sûres sur les valeurs que `matching.rs` pose ?, `sort_by_key` et son coût, visibilité des fonctions), Svelte (la
condition `proposals.some(...)` et sa place dans les branches, `data-testid`, repli i18n égal au catalogue fr-CH),
les quatre catalogues (parité, apostrophes typographiques, termes « règle d'affectation » alignés sur l'écran), tests
qui prouvent ce qu'ils disent (une assertion qui passerait aussi sans le code ? — les trois tests HTTP sont **verts
au sol par construction**, C-29-3-11 : sont-ils pour autant utiles ?), le test muet déplacé (prouve-t-il maintenant
son nom ?), le spec E2E (état partagé de la base E2E : lien du compte bancaire posé puis rendu, facture réglée en cas
d'échec, références du relevé toutes remplacées), code mort, nommage.

## Lentille E — Edge cases

Parcours chaque branche et chaque borne : liste vide, une règle seule, factures seules, facture sans signal
(inatteignable : pourquoi ? une source future la rendrait-elle atteignable sans que rien rougisse ?), égalités de
score, 51 éléments, score total exactement 0,5 (`0,4 + 0,1` en `f64`), contact et référence qui se contiennent
dans un sens ou dans l'autre, transaction non CHF, règle visant un compte archivé ; côté écran : chargement, erreur
au premier chargement, **erreur au rechargement** avec des lignes encore en mémoire, toutes les lignes « Aucune
correspondance », mélange de lignes avec et sans candidate ; côté E2E : échec avant « Accepter », compte bancaire déjà
lié, nom de contact composé (« Pia <nom> »), autres factures de 972,90 de la base partagée. Un cas non couvert par un
test et dont le comportement est faux est au moins **MEDIUM**.

## Lentille A — Acceptance (la fiche contre le livré)

Chaque AC (1 à 9) et chaque tâche (T0 à T7) : livré, testé par un test **qui rougirait**, déclaré honnêtement au
Dev Agent Record ? En particulier : AC1 (doc-comments : règle, pourquoi, rang 2 inatteignable, `candidates[0]`, #665
et #668), AC2 (les onze cas, entrées et sorties en dur, cas 11 par une expression indépendante), AC3 (montages valeur
par valeur, test muet déplacé et retiré de `reconciliation_rules_e2e.rs` avec son renvoi d'une ligne), AC4 (fabrique
à deux candidates, constructeur de règle, les deux tests), AC5 (phrase aux quatre langues mot pour mot, placement,
condition, `sitesTotal` 2107 → 2108 recompté, en-tête du composant), AC6 (montage, nettoyage, captures — et le
constat de C-29-3-25 : le sombre émulé rend comme le clair), AC7 (inventaire clos, quatre ajouts, contrôles du PDF),
AC8 (CHANGELOG et `api-external.md`), AC9 (M1 à M11, avec M1b et M9a/M9b). Les écarts à la fiche (C-29-3-19 : `finally`
au lieu d'`afterAll` ; C-29-3-20 : lien temporaire du compte bancaire ; C-29-3-21 : nom de contact relu ; C-29-3-23 :
label neuf ; C-29-3-24 : « Modifié ») sont-ils justifiés et sans effet sur un AC ? Le rebase sur la 29-2 (T0) a-t-il
laissé une zone partagée mal reprise (`routes/reconciliation.rs`, manuel `:2029-2060`, `docs/api-external.md`,
`CHANGELOG.md`) ?

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/29-3-review-p1-<B|E|A>.md` (selon ta lentille) :

- findings numérotés `B1…` / `E1…` / `A1…`, chacun avec sévérité (CRITICAL / HIGH / MEDIUM / LOW), fichier:ligne,
  preuve (**sortie `grep -nF` ou `sed -n` copiée** pour toute affirmation de présence ou d'absence), correctif proposé ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été** — un « 0 finding » sans cette liste ne
  compte pas ;
- rien d'autre n'est écrit nulle part.
