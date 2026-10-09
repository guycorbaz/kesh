# Prompt — revue de code P1, Story 15-1a-ii

*Versionné le 2026-10-09. Trois lentilles (Sonnet), contexte frais chacune, en lecture seule (bmad-code-review :
Blind Hunter, Edge Case Hunter, Acceptance Auditor). Rotation D6 : passes complètes Sonnet ↔ Opus ; Haiku en passe ciblée.*

Worktree `/home/gcorbaz/devel/kesh-15-1a-ii`, branche `story/15-1a-ii-gardes-du-lettrage`. **Le diff à revoir** :
`git -C /home/gcorbaz/devel/kesh-15-1a-ii diff 0724904c..5a54ec01 -- . ':(exclude)_bmad-output'` (un seul diff aplati ;
code, tests, manuels et PDF, i18n, docs — les fiches `_bmad-output/` sont le contexte, pas l'objet). **Fiche** :
`_bmad-output/implementation-artifacts/15-1a-ii-gardes-du-lettrage.md` (AC, tâches, sections « Reçu de … », T0, Dev
Agent Record, Change Log) ; sœur déjà mergée 15-1a-i (`0724904c`) ; à venir 15-1a2 (lettrage des pièces, groupes
`document`) — ce qu'elle prescrit n'est PAS un manque ici. Issue (par `gh api repos/guycorbaz/kesh/issues/518`) :
#518 (refs seulement). Registre : `epic-15-choix-autonomes.md` (C124 et suivants, C-15-1a-ii-1 à 6). Règles :
`CLAUDE.md`.

Code touché : gel `ENTRY_LETTERED` (`lettering_guard`, indépendant d'`enforce_ownership`) sur `DELETE` (étape
3-quinquies), `PUT` (étape 7-bis, avant le court-circuit no-op) et le motif d'écran du `GET` (en dernier), toujours
APRÈS le verrou de période ; 409 et écran ; **contre-passation qui lettre** (R6 : chaque ligne libre et lettrable de
l'origine forme avec son miroir un groupe `reversal` ; une ligne déjà lettrée garde son groupe, son miroir reste
ouvert ; relecture des lignes après le lettrage pour que la 201 porte la marque) ; précédence C117 (reçue de la
15-12b) ; textes « réservés » de la 15-1a-i réécrits pour le `reversal` ; réserve « sauf ligne lettrée » (CHANGELOG,
api-external — tableaux recomptés —, deux manuels + PDF, README, deux clés i18n ×4) ; cycle de verrous « lignes ↔
écriture » nommé (Pattern 5, registre des routes).

Axes : l'ensemble clos des écrivains qui modifient, suppriment, dévalident ou contre-passent une écriture dont une
ligne est lettrée — un chemin échappe-t-il au gel (import, restauration, rejeu, lots, clôture, dévalidation de
pièce, suppression de facture) ? ; l'ordre des refus (période verrouillée, exercice postérieur clos, puis la marque)
sur chaque chemin ; les cycles de verrous avec la 15-1a-i (création/dissolution de groupe), la 15-8a/b et la
clôture ; R6 : contre-passation d'une écriture partiellement lettrée, d'une écriture sur compte non lettrable, d'une
contre-passation (miroir du miroir), dissolution ultérieure d'un groupe `reversal` ; la 201 porte-t-elle vraiment
la marque ; les tests qui passeraient à vide (dont la mutation frontend survivante, déclarée) ; le **manuel**.

## Lentilles

- **B — Blind Hunter** : le diff seul, sans la fiche. Défauts de correction, régressions, erreurs de concurrence
  (verrous, REPEATABLE READ, ordre d'acquisition), erreurs rendues au client, tests qui passeraient à vide, code mort,
  duplication (DRY), doc-comments devenus faux.
- **E — Edge Case Hunter** : chaque branche et chaque borne du code modifié — écriture partiellement lettrée, ligne
  d'une autre société, compte non lettrable, exercice clos ou période verrouillée, miroir d'un miroir, PUT identique ;
  chemins par clé d'API ; locales ; et les **chemins NON modifiés qui devraient l'être** (inventorier par `grep` les
  sites non résolus de la même famille).
- **A — Acceptance Auditor** : chaque AC de la fiche contre le code ET les tests (un AC sans test qui le prouve est un
  finding) ; le Dev Agent Record ne déclare-t-il que ce qui a tourné (chiffres recomptés :
  `grep -c '#\[sqlx::test\]\|#\[test\]\|#\[tokio::test\]'` aux deux bornes) ; la Migration breaking policy point par
  point ; le **manuel** (`docs/manual/fr/*.tex` **et PDF aplatis** : `pdftotext -nopgbrk f.pdf - | tr '\n' ' ' | tr -s ' '`,
  ligatures ﬀ/ﬁ/ﬂ normalisées), `docs/api-external.md`, CHANGELOG, i18n 4 locales.

## Ce que tu rends

Rapport complet dans `/home/gcorbaz/devel/kesh-gate-logs/15-1a-ii-review-p1-<B|E|A>.md` : findings numérotés, sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` pour toute affirmation de présence ou
d'absence ; code cité relu), correction proposée ; ⛔ **axes exercés ET non exercés**. Dernier message : le chemin, le
bilan par sévérité, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans
une base, ni aucune commande qui compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`),
`make`, `latexmk`, `git commit`/`add`/`checkout`/`stash`/`apply`, `sqlx`, `cargo`, `npm`, `npx`,
`gh issue create`/`comment`/`edit`, SQL d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`,
`gh api` en lecture, `pdftotext` vers la sortie standard.
