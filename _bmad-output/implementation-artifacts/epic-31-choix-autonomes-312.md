# Epic 31 — choix autonomes de la story 31-2 (registre propre, à verser par union)

### C-31-2-1 — Le montage de dev dépend du répertoire de lancement : un garde, pas un correctif du compose

- **Contexte** : le conteneur `kesh-mariadb-dev` monte `/home/gcorbaz/devel/kesh-chore-tmpfs/scripts/mariadb-init`, un
  répertoire vide (le worktree n'existe plus) ; `kesh` n'a donc que `kesh`.*. Le compose monte `./scripts/mariadb-init`,
  relatif au répertoire de lancement.
- **Retenu** : règle « créé depuis le dépôt principal » écrite + garde `scripts/check-dev-grants.sh` branché sur
  `test-fast.sh` (égalité d'ensembles de droits, dérivée du fichier versionné).
- **Écartées** : chemin absolu en dur (casse les autres postes) ; variable d'environnement de plus (un réglage à
  oublier) ; script `dev-db.sh` qui refuse hors dépôt principal (une story, issue demandée).
- **Réversible** : oui.

### C-31-2-2 — Le smoke tourne contre l'image Docker de la PR

- **Contexte** : #577 demande un job minimal ; deux montages possibles.
- **Retenu** : job `e2e-smoke`, `needs: docker-build`, image reconstruite par le cache `gha`, lancée en
  `--network=host` avec `KESH_TEST_MODE`, `KESH_HOST=127.0.0.1`, `KESH_COOKIE_SECURE=false`, base `kesh` sur le service
  MariaDB en utilisateur `kesh`. Une spec neuve `smoke.spec.ts` (login UI + `GET /auth/me` authentifié + sonde négative
  sans cookie).
- **Écartée** : `cargo build` + artefact `frontend-build` (second build complet, binaire de débogage, pas ce qu'on
  publie). Repli si le cache ne tient pas.
- **Réversible** : oui.

### C-31-2-3 — Docker Hub : pas de contournement conditionnel pour les forks

- **Contexte** : les PR de forks et de Dependabot n'ont pas les secrets.
- **Retenu** : `credentials` inconditionnels sur les trois services ; limite écrite dans `docs/ci.md` ; repli
  `mirror.gcr.io` noté, non adopté. Dépôt à un seul développeur, ni fork ni `dependabot.yml`.
- **Écartée** : expression conditionnelle sur `credentials` (invérifiable hors runner, fragile pour un cas inexistant).
- **Réversible** : oui.

### C-31-2-4 — La CI emploie `kesh` avec les droits du fichier de dev, plus `root`

- **Contexte** : la CI est `root` (plus large que la production) ; un test à privilège excessif y passe.
- **Retenu** : le job `backend` applique `01-dev-grants.sql` en `root` après l'attente de MariaDB, `DATABASE_URL` en
  `kesh` ; migrations et seed restent en `root` (outillage). Non vérifiable hors CI : tout rouge est trié, nommé et
  tracé, jamais annulé en silence.
- **Écartée** : laisser `root` (conserve l'asymétrie que C3 dénonce) ; modifier `release.yml` (publication, issue demandée).
- **Réversible** : oui.

### C-31-2-5 — `.dockerignore` reçoit aussi `/.svelte-kit/`

- **Contexte** : mécanique, même symptôme que le `.gitignore` racine. **Retenu** : ajouté. **Réversible** : oui.

### C-31-2-6 — Ordre de livraison 4, 3, 1, 2 et un commit par volet

- **Retenu** : du plus mécanique au plus coûteux ; un volet défaillant n'arrête pas les autres. **Réversible** : oui.

### C-31-2-7 — Le smoke n'est pas rendu obligatoire par la story

- **Retenu** : aucun réglage de branche protégée modifié ; la décision est demandée à Guy via l'orchestrateur.
- **Réversible** : oui.

### C-31-2-8 — Le garde ne remet jamais le conteneur en état

- **Retenu** : il nomme la remise en état (recréation depuis le dépôt principal) sans l'exécuter : elle efface les
  bases de tous les agents (`docs/agents-orchestration.md` § 2). L'orchestrateur la fait, hors gate. **Réversible** : oui.

### C-31-2-9 — Prémisse corrigée : le conteneur a MOINS de droits que le fichier

- **Contexte** : la rétrospective 29 § 4.2 et l'epic le disaient plus doté (`GRANT ALL *.*`). Constat 2026-10-23 :
  `kesh`.* seul, montage vide. **Retenu** : la fiche corrige ; à noter à la rétrospective 31, aucune issue ne rouvre C3.
