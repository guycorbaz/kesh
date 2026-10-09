# Prompt — revue de code P1, Story 15-7b3

*Versionné le 2026-10-09. Trois lentilles (Sonnet), contexte frais chacune, en lecture seule (bmad-code-review :
Blind Hunter, Edge Case Hunter, Acceptance Auditor). Rotation D6 : passes complètes Sonnet ↔ Opus ; Haiku en passe ciblée.*

Worktree `/home/gcorbaz/devel/kesh-15-7b3`, branche `story/15-7b3-reparation-des-installations-atteintes`. **Le diff à
revoir** : `git -C /home/gcorbaz/devel/kesh-15-7b3 diff e892dcfa..ccac9b9b -- . ':(exclude)_bmad-output'` (un seul diff
aplati ; les fiches `_bmad-output/` sont le contexte). **Fiche** :
`_bmad-output/implementation-artifacts/15-7b3-reparation-des-installations-atteintes.md` (AC, tâches, T0 et ses douze
écarts, Dev Agent Record). Sœur mergée : 15-7b2 (`reattach_orphan_principals_in_tx`, `insert_stub`, `reset_demo`).
Issues (par `gh api repos/guycorbaz/kesh/issues/N`) : #528, #542 ; refs #546. Registre : `epic-15-choix-autonomes.md`
(C-15-7-*, C-15-7b3-1 à 3). Règles : `CLAUDE.md`. Mutations : `/home/gcorbaz/devel/kesh-gate-logs/157b3-mutations.log`.

Code touché : `companies::repair_installation_in_tx` (kesh-db) et `company_referencing_columns` — ordre de l'AC 1,
suppression de chaque société provisoire superflue sous `SAVEPOINT`, entrée `installation.repaired` ; appel en tête
d'`ensure_admin_user` (démarrage, échec journalisé non bloquant) et à l'étape 5-quater de `run_backup_and_restore`
(échec = import annulé) ; action d'audit ×4 locales ; manuels (dont trois textes réécrits hors liste, C-15-7b3-3),
api-external, CHANGELOG ; deux aides `rendre_principaux_orphelins` et `poser_declencheur_en_echec` ajoutées à
`kesh_db::test_fixtures` (C-15-7b3-1).

Axes : **une réparation touche des données réelles au démarrage** — que supprime-t-elle exactement, et peut-elle
supprimer une société qui porte des données (la liste `company_referencing_columns` est-elle COMPLÈTE contre le schéma :
29 clés étrangères vers `companies` — inventorie-les toi-même) ? une installation saine est-elle strictement intacte
(no-op prouvé) ? deux démarrages concurrents, un démarrage pendant une restauration ? l'échec non bloquant au démarrage
laisse-t-il un état partiel (le SAVEPOINT le garantit-il) ? à la restauration, l'ordre avec le rejeu post-restore ?
⚠️ **`kesh_db::test_fixtures` est compilé avec la production** (endpoint de test) : y ajouter une aide qui pose un
DÉCLENCHEUR SQL ou écrit des données arbitraires est-il acceptable (les gardes lexicales du lettrage balaient
`crates/*/src`) ? Les textes du manuel réécrits hors liste sont-ils vrais ? le **manuel** (.tex et PDF aplati).

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

Rapport complet dans `/home/gcorbaz/devel/kesh-gate-logs/15-7b3-review-p1-<B|E|A>.md` : findings numérotés, sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` pour toute affirmation de présence ou
d'absence ; code cité relu), correction proposée ; ⛔ **axes exercés ET non exercés**. Dernier message : le chemin, le
bilan par sévérité, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans une base, ni
aucune commande qui compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`), `make`,
`latexmk`, `git commit`/`add`/`checkout`/`stash`/`apply`, `sqlx`, `cargo`, `npm`, `npx`,
`gh issue create`/`comment`/`edit`, SQL d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`,
`gh api` en lecture, `pdftotext` vers la sortie standard.
