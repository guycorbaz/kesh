# Prompt — validation P1, Story 31-4 (la répétition à blanc d'une clôture)

*Versionné le 2026-10-24. **Deux lentilles, R et F**, contexte frais, modèle **Opus** (décision de l'orchestrateur : cette
fiche porte la procédure qui a failli remplacer la base de production — R1 HIGH, rétrospective de l'Epic 29 § 4.1).
Lecture seule stricte.*

Dépôt `/home/gcorbaz/devel/kesh-31-4`, branche `story/31-4`. Fiche : `_bmad-output/implementation-artifacts/31-4-repetition-a-blanc.md` ;
registre : `epic-31-choix-autonomes-314.md`. Règles : `CLAUDE.md`, `docs/agents-orchestration.md` § 6. Cahier des charges
d'origine : commentaire de la clé `29-7-repetition-a-blanc` de `_bmad-output/implementation-artifacts/sprint-status.yaml`
(`grep -nF "29-7-repetition-a-blanc" … | cut -c1-6000`).

Rapports, **un seul fichier chacun, écrit d'un seul jet** : lentille R
`/home/gcorbaz/devel/kesh-gate-logs/314-validate-p1-R.md` ; lentille F `/home/gcorbaz/devel/kesh-gate-logs/314-validate-p1-F.md`.

## Lentille R — la raison, les risques, la complétude

Cherche ce qui, dans la procédure que la fiche prescrit, **peut encore détruire des données de la production ou du lecteur**,
et ce que le cahier des charges exigeait que la fiche omet. Axes, tous à exercer :

1. **R1 jusqu'au bout.** Suis chaque chemin par lequel la copie pourrait viser la base ou les dossiers de la production : le
   compose Synology renommé, un `.env` recopié, `DATABASE_URL` hérité de l'environnement du shell, `COMPOSE_PROJECT_NAME`
   absent ou hérité, un `docker-compose.override.yml` préexistant dans le dossier, un `down -v` mal ciblé, un `rm -rf` sur
   variable vide. Pour **chacun**, le contrôle de l'AC3 l'arrête-t-il **avant** le `up` ? Le contrôle lit-il la **sortie finale**
   de `docker compose config` (après override) ? Le critère « seulement alors : `up` » est-il décidable ?
2. **R2.** Les variables `KESH_*_HOST_DIR` : fichier `.env`, shell, `--env-file`, fichier `.env` parent. Le contrôle 6 les couvre-t-il
   toutes ? Qu'arrive-t-il d'un `source:` qui est un chemin relatif résolu hors du dossier ?
3. **Cahier des charges de la 29-7** : relis le commentaire de la clé, point par point (R1, R2, F5, F L9, G19, « .env de la copie
   depuis .env.example », « aucune donnée réelle », « la marche répète ses gestes »). Un point est-il affaibli, omis, ou
   contredit par la fiche ? Les trois ajouts de la fiche (collision de noms, épinglage de version, sauvegarde de sécurité)
   sont-ils justifiés, ou introduisent-ils un risque (override qui épingle une image *plus ancienne* que la sauvegarde) ?
4. **Les gestes de la clôture (AC5)** : sont-ils décrits tels que `main` les fait ? Compare à `docs/manual/fr/user-manual.tex`
   (`sec:preparer-cloture`, `sec:transitoires`, `sec:reouverture-exercice`) et à `crates/kesh-api` (routes de clôture,
   réouverture, contre-passation datée). Un geste, un refus ou un ordre manque-t-il ?
5. **Données réelles** (`docs/agents-orchestration.md` § 8) : la fiche, la fixture, le journal, les captures peuvent-ils
   faire entrer une donnée réelle dans le dépôt ? Le fichier `.keshbackup` réel du lecteur est-il traité comme un secret
   tout au long, destruction comprise ?
6. **Périmètre** (`CLAUDE.md` § *Règle de splitting préventif*, « le défaut voisin ») : la fixture, la garde G22 et le script
   de test sont-ils du périmètre minimal ? Plus de cinq modules touchés ? Un élargissement non mécanique ?
7. **Mutations et tests** : chaque AC vérifiable a-t-il son test ou son journal ? M1 à M7 tuent-elles vraiment ce qu'elles
   disent ? Un test pourrait-il passer à vide (G22 sur une sous-section introuvable, positive adossée) ?
8. **Dépendance à la 31-1c** : la fiche en dépend-elle en réalité, par un détail (dossier `backup/`, nom de fichier) ?
9. **Manuel** : la fiche **nomme-t-elle** le manuel d'administration (`.tex` et PDF), l'emplacement, le label, les renvois ?
   Les PDF se contrôlent aplatis : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`.

## Lentille F — les faits que la fiche affirme

Pour **chaque fait** du tableau « État des lieux », de « Prémisses… » et des AC, vérifie-le par `grep -nF`, `sed -n` ou
`git show` et **copie la sortie**. Axes, tous à exercer :

1. **Compose** : `docker-compose.yml` et `docker-compose.prod.yml` — `DATABASE_URL`, `container_name`, ports, volumes, réseaux,
   lignes citées (l. 140, 152-153 pour les montages). `git diff v0.14.0 HEAD -- docker-compose.yml docker-compose.prod.yml .env.example Dockerfile` :
   « quatre lignes de commentaires » est-il exact ? Les deux fichiers sont-ils bien nommés comme la fiche le dit au manuel
   Synology (`admin-manual.tex` autour de la l. 573) ?
2. **`.env.example`** : `DATABASE_URL=<EDIT…>`, `KESH_JWT_SECRET=<GENERATE_ME…>`, `MARIADB_*` commentées, `KESH_COOKIE_SECURE=true` ;
   le refus des placeholders au démarrage (`crates/kesh-api/src/config.rs`, `is_template_placeholder`).
3. **Manuel** : sections et lignes citées (`Test de restauration périodique` l. 1598, `Restaurer sur une nouvelle machine`
   l. 2013 et son label, `Export/import…` l. 1851, `Quelle méthode…`) ; le manuel utilisateur (l. 685, 792, 819, 843).
4. **G19 et `kesh-style.sty`** : `\keshVersion` vaut `0.14.0` (l. 64) ; la garde à la ligne citée ; la forme exacte de l'URL.
5. **Parcours E2E** : `frontend/tests/e2e/fiscal-year-closing-journey.spec.ts` et `backup-restore-new-install.spec.ts` font-ils
   ce que la fiche dit (appels, IBAN, fixtures, `_test/seed` preset `with-company-no-fy`) ? Le fichier `camt053_v04_year_end.xml` existe-t-il ?
6. **Routes** : `GET /api/v1/admin/full-export`, `POST …/full-import`, `GET /health` (champ `version`) ; l'endpoint `_test/seed`
   n'existe qu'en `KESH_TEST_MODE` (`crates/kesh-api`).
7. **Santé du dépôt** : `scripts/tests/prepare-release.test.sh` existe (modèle du test de script) ; `scripts/` n'a pas déjà un
   `repetition-fixture.sh` ; la garde G22 n'existe pas déjà ; le numéro G22 est libre (`grep -nF "G2" crates/kesh-api/tests/textes_coherents.rs`).
8. **Cohérence interne** : AC1 à AC12, tâches T0-T8, registre C-31-4-1 à 9, renvois « (AC7) », « (AC9) », « n° 3 », nombres
   (sept contrôles, deux pièces, M1 à M7) : une contradiction, un renvoi faux, un nombre non recompté ?
9. **Entrées datées** : la fiche ou le sprint-status réécrivent-ils une entrée passée ? Le registre d'epic
   `epic-31-choix-autonomes.md` est-il touché (il ne doit pas l'être : union au merge) ?
10. **Ce que la fiche omet** : un fichier versionné qui dit comment restaurer, copier ou répéter (`docs/`, `README.md`,
    `DOCKER_START.md`, `scripts/synology/`) et qu'il faudrait tenir d'accord.

## Ce que tu rends

Findings `R1…` / `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, ligne du dépôt, preuve (commande et sortie
copiée), correction proposée. ⛔ **La liste des axes exercés ET non exercés** — sans elle, « 0 finding » ne compte pas. Toute
affirmation de présence ou d'absence s'appuie sur une sortie `grep -nF` copiée. Dernier message : chemin, bilan par sévérité,
une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport. Aucun `git checkout`/`switch` sous aucune forme ; aucune commande qui écrit,
compile ou exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `npm`,
**`docker` (aucune sous-commande, `compose config` compris)**, **`python3`, `python`, `perl`, `ruby`, `node`**, `mariadb`/`mysql`,
`curl`, `git commit|add|stash|fetch|pull|reset|rebase|tag|push`, `gh` en écriture. `pdftotext` vers la sortie standard seulement
(aucun fichier temporaire). Aucune donnée réelle de Guy (rien de `~/kesh-test/`). Autorisés : lecture, `grep`, `sed -n`,
`awk` en lecture, `git log|show|diff|blame`, `pdftotext … -`.
