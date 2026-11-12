# Revue de code P1 — Story 34-9 (#969, #974) : le rapport d'import du dossier nomme chaque fichier

Trois lentilles indépendantes, en contexte frais, sur le **même** périmètre : **B** (Blind Hunter), **E** (Edge Case
Hunter), **A** (Acceptance Auditor). Modèle : Sonnet ou Opus (une première passe n'est jamais confiée à Haiku).

## Cadre

- Worktree : `/home/gcorbaz/devel/kesh-e349`, branche `story/34-9-rapport-import-dossier`.
- Périmètre : `git diff 665de151..HEAD` (commit de base `665de151`). Lire le diff **aplati**, non commit par commit.
- Fiche : `_bmad-output/implementation-artifacts/34-9-rapport-import-dossier.md` (AC-1 à AC-8, mutations M1 à M9,
  Dev Agent Record) ; choix : `e34-choix-autonomes-e342.md` (C-e342-10 à 14, 19 à 22) et
  `e34-choix-autonomes-e349.md` (C-e349-1 à 10).
- Consignes : `CLAUDE.md` du worktree ; `/home/gcorbaz/devel/kesh-gate-logs/prompts/e34-commun.md`.

## Rapports

- B : `/home/gcorbaz/devel/kesh-gate-logs/e349-review-p1-B.md`
- E : `/home/gcorbaz/devel/kesh-gate-logs/e349-review-p1-E.md`
- A : `/home/gcorbaz/devel/kesh-gate-logs/e349-review-p1-A.md`

Chaque finding : sévérité (CRITICAL, HIGH, MEDIUM, LOW), fichier et ligne, preuve copiée (sortie de `git grep -nF`,
`git show`, `sed -n`), correction proposée. **Toute affirmation de présence ou d'absence s'appuie sur une sortie
copiée.** Le rapport se termine par la liste des **axes exercés** et des **axes non exercés** : une passe qui ne la
donne pas ne compte pas.

## ⛔ Lecture seule

Aucune écriture hors de votre rapport, aucune redirection vers un fichier, aucun fichier temporaire. Aucune commande
qui écrit, compile ou exécute : ni `cargo`, ni `npm`, ni `npx`, ni `make`, ni `latexmk`, ni `scripts/*.sh` (dont
`scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/gate-ci.sh`), ni `git checkout`, `git switch`,
`git commit`, `git stash`, `git reset`, ni `gh` qui écrit, ni `docker`. Permis : `git diff`, `git show`, `git log`,
`git grep`, `sed -n`, `cat`, `ls`, `/usr/bin/grep` (dans ce shell, `grep` est une fonction : employer `/usr/bin/grep`
ou `git grep`), `pdftotext … -` vers la sortie standard.

## Fichiers du diff

- Serveur : `crates/kesh-api/src/inbox_import.rs` (rapport, `SkippedFile`, `SkipReason`, `push_outcome`,
  `mark_truncated`, `duplicate_details`, `reread_duplicate`, `dispose_failed`, `with_moved_to`, `move_to_failed`,
  `mod tests`), `crates/kesh-api/src/routes/imported_supplier_invoices.rs` (doc-comment),
  `crates/kesh-api/tests/inbox_import_e2e.rs` (sept tests neufs ou étendus, deux montages de course).
- Écran : `frontend/src/routes/(app)/supplier-invoices/import/+page.svelte`, `import-page.test.ts`,
  `frontend/src/lib/features/imported-supplier-invoices/imported-supplier-invoices.types.ts`, `…api.ts` (doc).
- E2E : `frontend/tests/e2e/inbox-import.spec.ts`, `frontend/tests/e2e/helpers/inbox.ts`.
- Textes : `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`.
- Documentation : `docs/api-external.md`, `docs/manual/fr/user-manual.tex`, `docs/manual/fr/admin-manual.tex`,
  `CHANGELOG.md`.

## Manuel, PDF, captures

- **Manuel** : `docs/manual/fr/user-manual.tex`, § *Documents à traiter* (paragraphe « Le rapport d'import » et
  « Ce qui reste refusé ») ; `docs/manual/fr/admin-manual.tex`, § de l'import (`sec:inbox-import`, la description de
  `failed/`) et § *Ce qui reste hors du fichier*. Vérifier chaque phrase **contre le code** : ce que le manuel promet
  (lien, renommage, « se vide à la main », notification, rapport non conservé) doit être ce que fait le code.
- **PDF** : ⚠️ par décision C-e342-20, les PDF ne sont **pas** régénérés dans la story : ils le seront une fois, à
  l'intégration. Le PDF du dépôt (`docs/manual/fr/user-manual.pdf`, `admin-manual.pdf`) est donc celui d'avant la
  story ; le contrôler aplati (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`) pour **constater** qu'il ne porte pas
  encore le texte neuf, et le signaler comme attendu, non comme défaut.
- **Captures** (thème clair, base jetable) : `/home/gcorbaz/devel/kesh-gate-logs/e349/captures/` —
  `avant-rapport-mixte.png` (l'écran du commit de base, le constat), `apres-rapport-mixte.png`,
  `apres-notification.png`, `apres-lien-suivi.png`. Les ouvrir et dire si l'écran montré est celui que décrivent la
  fiche et le manuel.

## Axes par lentille

**B — Blind Hunter** (le diff seul, sans la fiche d'abord) : défauts de logique, de concurrence et de contrat. En
particulier : les deux relectures après `rollback` (`reread_duplicate`) — tiennent-elles une connexion, une
transaction ? ; `with_moved_to` (fusion) ; `move_to_failed` qui rend le nom ; `push_outcome` ; le Svelte (`$derived`,
`{@const}`, ordre de `runImport` et des notifications, `fresh`) ; les ancres `#imported-<id>` et le routeur SvelteKit.

**E — Edge Case Hunter** : chaque branche et chaque bord. Doublon sur chacun des trois sites (`:577` réactivation
manquée, `:581` existante, `UNIQUE` à l'`INSERT`), relecture vide ou en erreur, `completed` sans facture, `discarded` ;
`details` absent, `null`, non objet ; `movedTo` absent (serveur ancien) ; `skipped` absent (serveur ancien) ;
`truncatedAt` `0` ; un import vide ; rechargement de la liste en échec avec et sans message ; noms de fichiers à
accents, majuscules, chiffres (tri) ; un lien vers une ligne absente de la liste (document retiré entre-temps) ; les
montages de course des tests (pool de cinq connexions, `PROCESSLIST`, délais) — peuvent-ils passer à vide ?

**A — Acceptance Auditor** : chaque AC de la fiche (AC-1 à AC-8) contre le code et les tests, ligne lue citée ; les
mutations M1 à M9 et leurs journaux (`/home/gcorbaz/devel/kesh-gate-logs/e349/mut/*.log`) — chaque mutation
rougit-elle le test annoncé ? ; les rouges (`/home/gcorbaz/devel/kesh-gate-logs/e349/rouges.log`) ; les gates déclarés
au Dev Agent Record (journaux sous `/home/gcorbaz/devel/kesh-gate-logs/e349/`) — ne déclarer que ce qui a tourné ;
les décomptes du Dev Agent Record, **recomptés depuis la source** ; les quatre catalogues (clés neuves présentes,
aucune `->`, clés retirées non lues) ; `docs/api-external.md` ; le CHANGELOG (section `## [0.17.6] — Non publié`,
*Corrigé*, `refs #969`, `refs #974`) ; aucun mot-clé de fermeture dans les messages de commit
(`git log 665de151..HEAD --format=%B | /usr/bin/grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'` doit être
vide).
