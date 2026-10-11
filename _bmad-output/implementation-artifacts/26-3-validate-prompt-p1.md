# Prompt — validation P1 de la spec, Story 26-3 (manuels et garde de kesh-restore)

*Versionné le 2026-10-10. Deux lentilles (Sonnet), contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle.*

Dépôt `/home/gcorbaz/devel/kesh-26-3`, branche `story/26-3` (base `fe162504`, = v0.13.0 + kickoff de l'Epic 26).
Fiche : `_bmad-output/implementation-artifacts/26-3-manuel-et-restauration-synology.md` (commit `a103c9ad`).
Epic : `_bmad-output/planning-artifacts/epic-26-recette-v0-13.md`. Issues #616, #617, #619
(`gh api repos/guycorbaz/kesh/issues/N` et `.../N/comments`). Registre : `epic-26-choix-autonomes.md` (C-26-3-1 à 8).
Règles : `CLAUDE.md` (§ Review Iteration Rule, § Le prompt d'une passe doit NOMMER le manuel, § Inventorier les sites
NON RÉSOLUS, § Propagation post-patch). Fiches voisines en cours d'écriture sur d'autres branches : 26-1 (profil CSV,
réécrit la sous-section « Profils banque » du manuel utilisateur) et 26-2 (description de compte bancaire).

Axes prioritaires :

1. **La garde de `kesh-restore.sh`** (C-26-3-3) : lis le script réel et sa recette de test existante. La garde
   proposée échoue-t-elle FERMÉE dans tous les cas (service renommé, orphelin arrêté, conteneur `restarting`/`paused`,
   plusieurs projets compose, aucune installation) ? Piège `set -e` + sous-shell (`| while … exit 1`). Le filet par
   image `gcorbaz/kesh` : faux positifs (une autre instance Kesh légitime sur l'hôte) ? Les cas de recette prescrits
   sont-ils suffisants et exécutables ; le nettoyage ; les mutations.
2. **Le couplage G16** (`configuration_transmise.rs`, regex sur `service=kesh-api`) et la garde de texte G20 proposée :
   les tests de texte existants qui changeront de sens ; collisions de numérotation.
3. **#616 et #617 au code** : chaque affirmation que la fiche fait écrire dans le manuel est-elle vraie dans le
   frontend actuel (formulaires du plan comptable, `/bank-accounts`, écran d'import, lien comptable requis pour
   l'import ou seulement pour la réconciliation, compte parent modifiable) ? Inventaire CLOS du symptôme : grep de
   « Compte de banque », « toutes les banques », « CAMT.053 » dans `docs/manual/` (`.tex` ET PDF aplatis
   `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`), `website/`, `README.md`, i18n 4 locales, brochure.
4. **Le manuel d'administration** (réseau du dump, `SAUVEGARDE_RESEAU`, recette Traefik, C-26-3-4, C-26-3-7) contre
   `docker-compose.prod.yml` et les scripts.
5. **Implémentable sans deviner**, recompte des AC/tâches/tests, gates prescrits (E2E complet au dernier commit de
   code, puisque l'écran d'import change), CHANGELOG `[0.13.1]`, conflits prévisibles avec 26-1/26-2.

## Lentilles

- **R — Auditeur d'acceptation** : `.claude/skills/bmad-create-story/checklist.md` ; chaque phrase des trois issues
  couverte par un AC testable, chaque AC par une tâche, chaque tâche par un test ; faits cités revérifiés.
- **F — Full-scope adversary** : la conception elle-même ; ce que la garde laisse passer ; ce que le manuel corrigé
  affirmerait encore de faux ; tests existants qui changeront de sens.

## Ce que tu rends

Findings numérotés avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie copiée),
correction proposée. Toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée. ⛔ **La liste des axes
exercés ET non exercés** — un « 0 finding » sans elle ne compte pas. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/26-3-validate-p1-<R|F>.md` ; dernier message : le chemin, le bilan, une ligne
par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh` et les scripts Synology), `make`, `latexmk`, `git commit`/`add`/
`checkout`/`switch`/`stash`/`fetch`/`reset`/`rebase`, `cargo`, `npm`, `npx`, `docker`, `bash` sur un script du dépôt,
`gh` en écriture, SQL. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh api` en lecture,
`pdftotext` vers la sortie standard.
