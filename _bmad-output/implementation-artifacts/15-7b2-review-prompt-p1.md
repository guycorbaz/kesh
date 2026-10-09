# Prompt — revue de code P1, Story 15-7b2

*Versionné le 2026-10-09. Trois lentilles (Sonnet), contexte frais chacune, en lecture seule (bmad-code-review :
Blind Hunter, Edge Case Hunter, Acceptance Auditor). Rotation D6 : passes complètes Sonnet ↔ Opus ; Haiku en passe ciblée.*

Worktree `/home/gcorbaz/devel/kesh-15-7b2`, branche `story/15-7b2-remise-a-zero`. **Le diff à revoir** :
`git -C /home/gcorbaz/devel/kesh-15-7b2 diff 056997b0..HEAD -- . ':(exclude)_bmad-output'` (un seul diff aplati ; les
fiches `_bmad-output/` sont le contexte). **Fiche** : `_bmad-output/implementation-artifacts/15-7b2-remise-a-zero.md`
(AC, tâches, T0 et ses écarts, Dev Agent Record et ses angles morts). Sœurs : 15-7b1 (mergée), 15-7b3 (à venir :
réparation des installations déjà atteintes — ce qu'elle prescrit n'est pas un manque ici). Issues (par
`gh api repos/guycorbaz/kesh/issues/N`) : #434 (P1), #279 ; refs #528 #542 #534 #540 #538 #550. Registre :
`epic-15-choix-autonomes.md` (C-15-7-*, C-15-7b2-1 à 4). Règles : `CLAUDE.md`. Mutations : 
`/home/gcorbaz/devel/kesh-gate-logs/15-7b2-mutations.log`.

Code touché : `kesh_seed::reset_demo(pool, acteur, drapeau)` — une transaction par essai, rejouée sur interblocage,
connexion dédiée fermée ; trois gardes sous le verrou de la transaction qui efface (finalisation, production au-delà
de l'étape 2, `KESH_PRODUCTION_RESET`) ; liste des tables vidées dérivée de la liste canonique (33) et test qui lit les
clés étrangères du schéma ; société remise à l'état provisoire au même `id` ;
`companies::reattach_orphan_principals_in_tx` (clés d'API orphelines révoquées puis rattachées) ; démarrage qui
n'ajoute plus de société provisoire (#542) ; action d'audit `installation.reset` ; registre des routes 108/4/2 sur 114 ;
manuels, api-external, CHANGELOG.

Axes : **une remise à zéro efface — rien ne doit survivre qui fausse la suite, rien ne doit disparaître qui ne le
devait pas** : tables de la société oubliées (lettrage dans `journal_entry_lines`, sauvegardes, pièces jointes,
fichiers sur disque `/data/documents`, `/data/backup`, inbox), données d'une autre société, utilisateurs et clés d'API
(révoquées, jamais réveillées), séquences et numéros de pièce ; les gardes sont-elles évaluées sous le bon verrou et
dans le bon ordre (course avec `start-production`, avec un import, avec un autre reset) ? le rejeu sur interblocage
reprend-il de zéro sans état partiel ? la connexion fermée ne fuit-elle rien ? un refus rend-il le bon code ? le
**manuel** et la recette de sortie de la démonstration.

## Lentilles

- **B — Blind Hunter** : le diff seul, sans la fiche. Défauts de correction, régressions, erreurs de concurrence
  (verrous, REPEATABLE READ, ordre d'acquisition), erreurs rendues au client, tests qui passeraient à vide (un test qui
  ne mord pas sur la mutation qu'il prétend couvrir), code mort, duplication (DRY), doc-comments devenus faux.
- **E — Edge Case Hunter** : chaque branche et chaque borne du code modifié — entrées vides, nulles, multiples, en
  doublon, d'une autre société, archivées ; chemins par clé d'API ; lots partiellement en échec ; locales ; et les
  **chemins NON modifiés qui devraient l'être** (inventorier les sites non résolus de la même famille par `grep`).
- **A — Acceptance Auditor** : chaque AC de la fiche contre le code ET les tests (un AC sans test qui le prouve est
  un finding) ; le Dev Agent Record ne déclare-t-il que ce qui a tourné (chiffres recomptés : `grep -c '#\[sqlx::test\]\|#\[test\]\|#\[tokio::test\]'` aux deux bornes) ;
  le **manuel** (`docs/manual/fr/*.tex` **et PDF aplatis** : `pdftotext -nopgbrk f.pdf - | tr '\n' ' ' | tr -s ' '` vers la sortie
  standard, ligatures ﬀ/ﬁ/ﬂ normalisées), `docs/api-external.md`, CHANGELOG, i18n 4 locales.

## Ce que tu rends

Rapport complet dans `/home/gcorbaz/devel/kesh-gate-logs/15-7b2-review-p1-<B|E|A>.md` : findings numérotés, sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` pour toute affirmation de présence ou
d'absence ; code cité relu), correction proposée ; ⛔ **axes exercés ET non exercés**. Dernier message : le chemin, le
bilan par sévérité, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans une base, ni
aucune commande qui compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`), `make`,
`latexmk`, `git commit`/`add`/`checkout`/`stash`/`apply`, `sqlx`, `cargo`, `npm`, `npx`,
`gh issue create`/`comment`/`edit`, SQL d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`,
`gh api` en lecture, `pdftotext` vers la sortie standard.
