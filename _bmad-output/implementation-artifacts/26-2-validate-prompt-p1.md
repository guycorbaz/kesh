# Prompt — validation P1 de la spec, Story 26-2 (description d'un compte bancaire)

*Versionné le 2026-10-10. Deux lentilles (Sonnet), contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle.*

Dépôt `/home/gcorbaz/devel/kesh-26-2`, branche `story/26-2` (base `fe162504`, = v0.13.0 + kickoff de l'Epic 26).
Fiche : `_bmad-output/implementation-artifacts/26-2-description-compte-bancaire.md` (commit `1c85e499`), découpée en 26-2a (socle, AC1-8) et 26-2b (écrans, AC9-12), cf. C-26-2-15.
Epic : `_bmad-output/planning-artifacts/epic-26-recette-v0-13.md`. Issue #622
(`gh api repos/guycorbaz/kesh/issues/N` et `.../N/comments`). Registre : `epic-26-choix-autonomes.md` (C-26-2-1 à 15).
Règles : `CLAUDE.md` (§ Review Iteration Rule, § Le prompt d'une passe doit NOMMER le manuel, § Inventorier les sites
NON RÉSOLUS, § Propagation post-patch). Fiches voisines en cours d'écriture sur d'autres branches : 26-1 (profil CSV,
réécrit la sous-section « Profils banque » du manuel utilisateur) et 26-3 (manuels, écran d'import, garde de kesh-restore).

Axes prioritaires :

1. **Inventaire clos des sites** (C-26-2-12, AC11) : refais le grep `bankName` et `iban` dans `frontend/src` et `crates/` ; chaque site classé ? la garde Vitest statique est-elle forte (anti-test-muet, liste d'exclusions qui se périme) ? sites serveur (PDF, exports, messages d'erreur, audit) ?
2. **Migration et requêtes** : les SELECT vers `BankAccount` (constante de colonnes unique, `FromRow`), squash de test `crates/kesh-db/test-schema/`, P5 (recompte), P6 (`migrations_upgrade_path.rs`), P7, P8 ; le verrou de `first_active_bank_account_linked_to` ; course de version de migration avec la 26-1 (profil CSV, autre migration possible).
3. **API** : validation (60 caractères Unicode, contrôle, trim, vide → NULL), `PUT` remplacement complet qui efface — les deux formulaires renvoient-ils toujours le champ ? onboarding inchangé ; audit ; `DbError` `bank_label` et `details.bankName` ; `docs/api-external.md`.
4. **Export CSV et `.keshbackup`** : aller-retour, sauvegarde antérieure sans la colonne (`parse_and_verify`, inventaire des colonnes), `post_restore.rs`.
5. **Frontière 26-2a / 26-2b** : rien entre deux chaises, chaque partie livrable et testable seule, E2E réparti ; manuel (section neuve, `.tex` et PDF aplati), i18n 4 locales (`i18n-keys.test.ts` recompte), CHANGELOG `[0.13.1]` ; recompte AC/tâches/tests/mutations.

## Lentilles

- **R — Auditeur d'acceptation** : `.claude/skills/bmad-create-story/checklist.md` ; chaque phrase de l'issue (et de ses commentaires)
  couverte par un AC testable, chaque AC par une tâche, chaque tâche par un test ; faits cités revérifiés.
- **F — Full-scope adversary** : la conception elle-même ; ce que la garde AC11 laisse passer ; ce que le manuel
  affirmerait encore de faux ; tests existants qui changeront de sens.

## Ce que tu rends

Findings numérotés avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie copiée),
correction proposée. Toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée. ⛔ **La liste des axes
exercés ET non exercés** — un « 0 finding » sans elle ne compte pas. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/26-2-validate-p1-<R|F>.md` ; dernier message : le chemin, le bilan, une ligne
par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh` et les scripts Synology), `make`, `latexmk`, `git commit`/`add`/
`checkout`/`switch`/`stash`/`fetch`/`reset`/`rebase`, `cargo`, `npm`, `npx`, `docker`, `bash` sur un script du dépôt,
`gh` en écriture, SQL. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh api` en lecture,
`pdftotext` vers la sortie standard.
