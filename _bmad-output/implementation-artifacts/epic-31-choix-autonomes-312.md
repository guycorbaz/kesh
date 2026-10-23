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

### C-31-2-10 — #698 intégrée à la 31-2 (choix de l'orchestrateur)

- **Contexte** : la fiche demandait d'ouvrir en P3 le fait que le smoke de `release.yml` interroge l'image publiée avec
  l'utilisateur `root` ; ouverte (#698). La règle de release fixe le plafond au tag v0.15.0 à moins de 136 issues
  ouvertes ; le compte est à 138.
- **Retenu** : intégrer #698 au volet 1 de la 31-2 — le smoke de release applique `scripts/mariadb-init/01-dev-grants.sql`
  (ou un `GRANT` aux droits de la production) puis démarre l'image avec l'utilisateur `kesh`, comme le job `backend` de
  `ci.yml` dans la même story. La PR porte `closes #698`. Ne se vérifie qu'en CI de release : le dire.
- **Écartée** : laisser #698 ouverte — même mécanisme que le volet 1, même correction.
- **Réversible** : oui.
- **Rappel** : #699 (montage relatif) reste ouverte ; le garde du volet 1 la signale sans la prévenir.

### Amendements du 2026-10-23 (validation P1 ; les entrées ci-dessus ne sont pas réécrites)

- **Amende C-31-2-10** (R-2, F6) : la prescription « applique `01-dev-grants.sql` (ou un `GRANT`…) » est **retirée**. Le
  service de `release.yml` crée déjà `kesh` (`MARIADB_USER: kesh`, `release.yml:22-26`) avec `ALL` sur `kesh`.*, soit les
  droits de la production ; la seule modification est `DATABASE_URL` (`release.yml:94`) → `mysql://kesh:kesh_dev@
  127.0.0.1:3306/kesh`, sans fichier de droits ni `GRANT` (le fichier de dev ajouterait `\_sqlx\_test%`.* et `kesh%`.*,
  que la production n'a pas). Porté par l'AC18 et la tâche T4 de la fiche. Ne se vérifie qu'au tag v0.15.0.
- **Amende C-31-2-4** (R-1) : l'« écartée » « modifier `release.yml` » vaut pour `credentials` et le reste de la
  publication, **non** pour le `DATABASE_URL` du smoke, repris par C-31-2-10.
- **Amende C-31-2-9** (F2) : provenance du `GRANT ALL *.*` de la rétrospective 29 — le fichier versionné de `main` n'a
  jamais porté de droit global (seulement deux commits intermédiaires de la 22-5, corrigés en passe 1) ; le montage est
  vide depuis le 2026-10-11 ; le `*.*` a été **posé à la main par un agent** (journal de l'Epic 15, 2026-10-10). D'où
  l'interdit de tout `GRANT`/`REVOKE` manuel sur le MariaDB de dev partagé, ajouté à `docs/agents-orchestration.md` § 2 par la
  fiche (AC5). Le conteneur a été recréé par l'orchestrateur le 2026-10-23 vers 18:13Z depuis le dépôt principal : plus de
  recréation à demander (AC7, T7).

### C-31-2-11 — Docker Hub authentifié aussi pour le build des images (F7)

- **Contexte** : `credentials` ne couvre que les services MariaDB ; les jobs `docker-build` et `e2e-smoke` tirent `rust`,
  `node` et `debian` (Dockerfile) anonymement. Retirer l'avertissement A-C9 aurait laissé un texte qui dit moins que le
  dépôt.
- **Retenu** : `docker/login-action@v3` (mêmes secrets) avant le build de ces deux jobs (`release.yml` le fait déjà) ;
  AC13 compte les étapes de connexion en plus des `credentials`. Les forks échouent bruyamment, comme pour C-31-2-3.
- **Écartée** : réécrire l'avertissement au lieu de le retirer (garde la dépendance anonyme, pour un coût de deux étapes
  YAML). **Réversible** : oui.

### C-31-2-12 — Le garde a des sources injectables (R-5, F1, F3)

- **Contexte** : depuis la recréation du conteneur, aucun état défaillant réel n'existe pour éprouver le garde ; AC3 ne
  portait que sur des `SHOW GRANTS`.
- **Retenu** : acquisition et comparaison séparées pour les droits (`--grants-from-stdin`, lu en `-N -B -r`,
  normalisation minimale qui garde les antislashs, fixture capturée) et pour le montage (`--mount-source <répertoire>`,
  `cmp` avec le fichier du worktree) ; `PATH` sans `docker` testé. Mot de passe par `MYSQL_PWD`.
- **Écartée** : normaliser en retirant les antislashs (rend `_sqlx_test%` non échappé indiscernable du bon droit).
  **Réversible** : oui.
