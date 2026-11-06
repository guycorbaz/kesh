# Story 33-10 : l'outillage de release : le gate en mode CI, et la montée et le retour de version dans un script du dépôt (#925 ; B1, B3 de la rétrospective 0.17.4)

Status: ready-for-dev

*Fiche rédigée le 2026-11-06 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e33`, branche `story/33-0-kickoff`, numéros de
ligne de `4ef19901`. Release **0.17.5**, Epic 33. **Aucune migration.** Registre : `e33-choix-autonomes-e334.md`, C-e334-21 à 25. Commits en
`refs` ; à la PR : `closes #925`. **Le script de montée servira à la 0.17.5, à migration (33-1a), première à passer par #847 en production.** ⚠️ La table
du fichier d'epic porte aussi **B2** ; la rétrospective (réponses de Guy) n'attribue à cette story que B1 et B3, et B2 est tenue par `prompts/e33-commun.md`
(*Fin de story*) : rien dans cette fiche (C-e334-25).*

## Story

En tant qu'orchestrateur d'une release, je veux lancer d'une seule commande les tests **tels que la CI les lance**, et la recette de montée et de retour **telle que la release
l'exige** ; pour ne plus découvrir en CI un défaut invisible au gate local, ni réécrire à chaque release, hors dépôt, un script que rien ne garde.

## Ce que le code dit (prémisses établies)

- **L'écart est réel** : `scripts/test-fast.sh:70` lance `cargo nextest` (un processus par test) ; `.github/workflows/ci.yml:195-199`, `cargo test --workspace -j1 -- --test-threads=1` (un processus par binaire). Incident : `release-0.17.4-retro-2026-11-06.md`, § 4.1, 17 tests rouges à la seule CI.
- **La CI** : `DATABASE_URL=mysql://kesh:kesh_dev@127.0.0.1:3306/kesh` (`ci.yml:76`), `kesh` hors `root` (`:71-75`), migrations et seed en `root` (`:178-193`). ⚠️ **Le profil du shell local exporte `DATABASE_URL=…/kesh`, une base vide** : le script force la sienne. Dernier relevé : `rel0174/ci-cargo-test.log`, 204 binaires, 4820 passés, 16 ignorés. **La durée n'est dans aucun journal** : à mesurer et à écrire à `docs/ci.md`.
- **Le reste de #925** : `config.rs:1683-1714`, `EnvGuard` ne restitue que `DATABASE_URL`, alors que `reset_env` en efface une trentaine (`:1717`…) : **court, se corrige ici** : le garde capture `env::vars_os()` et le restitue entier au relâchement. Reste, écrit au doc-comment : un `cargo test` **parallèle** local peut lire l'environnement pendant qu'un test le tient ; ni la CI ni nextest ne sont concernés.
- **Les trois fois où le script a été réécrit** : `kesh-gate-logs/rel0173/boot-{lib,phase1,phase2}.sh`, `rel0174/boot-{lib,phase1,phase2,phase3}.sh`, `reset-db.sh`. Constante à chaque fois : phase 1 (image précédente, base vide, seed, API, empreinte, `.keshbackup`, dump) ; phase 2 (binaire neuf, migrations, empreinte) ; phase 3 (refus de l'image précédente au journal, dump restauré, image relancée, empreinte identique). Rétrospective § 5, A3 ; `git grep "previously applied" v0.17.4 -- scripts` vide.
- ⚠️ **Ce qui varie d'une release à l'autre** ne doit pas aller dans le script : 0.17.4 regardait les jetons et l'alias CAMT ; 0.17.5 regardera la numérotation des factures fournisseur et le libellé des écritures passées. D'où un **crochet** (C-e334-23).
- **Le patron de test** : `scripts/tests/check-dev-grants.test.sh` (`docker` factice, `KESH_DOCKER`, `scripts/check-dev-grants.sh:23,83`), lancé par `crates/kesh-db/tests/check_dev_grants_script.rs`, donc au gate et en CI.
- **Le relevé CAMT synthétique** de `rel0174/camt/` a été lu : IBAN d'exemple, noms et montants inventés.
- **`--network host`, `KESH_TEST_MODE=true`** (`_test/seed`) : repris. `verrous()` de `boot-lib.sh` est propre à la station : reste à l'appelant (C-e334-24).

## Conception

### `scripts/gate-ci.sh <base> [journal]` (C-e334-21)

Un script neuf, **`scripts/test-fast.sh` n'est pas touché**. Refuse toute base hors de `^kesh_[a-z0-9_]+$`, et `kesh` elle-même (partagée) ; recrée la base (`DROP`/`CREATE`, en `root` sur `kesh-mariadb-dev`, jamais un redémarrage) ; `sqlx migrate run` ; `scripts/seed-dev-db.sql` ; `DATABASE_URL` **forcée** ; `cargo test --workspace -j1 -- --test-threads=1` ; sortie en `tee` au journal, code de sortie propagé ; dernière ligne : `gate-ci : N passés, M échoués, K ignorés` **recomptée depuis `test result:`**, et non recopiée. Sous `mem-guard` s'il n'y est pas déjà. ⛔ **Aucun `GRANT`** : droits déjà donnés par `01-dev-grants.sql`.

**Quand** (Q3 de la rétrospective, sans réponse) : avant la PR de release et après toute modification d'un test qui touche l'environnement, **la recommandation de la rétrospective**. Écrit à `CLAUDE.md` § *Test Locally First*, en corrigeant la phrase « ces 4 checks sont exactement ceux de la CI » : nextest n'est **pas** le même mode d'exécution.

### `scripts/recette-montee.sh` (C-e334-22, 23, 24)

```sh
scripts/recette-montee.sh --image gcorbaz/kesh:0.17.4 --binaire target/debug/kesh-api --static frontend/build \
  --base kesh_recette_e33 --port 3190 --journal /…/kesh-gate-logs/rel0175 \
  --migrations-attendues 1 --diff-attendu supplier_invoices,migrations [--apres /…/verifs-0175.sh]
```

**Phases, chacune vérifiée, code de sortie distinct** :

1. **Avant** : base `--base` recréée vide ; image précédente en premier plan **par un `docker run --rm` lancé en tâche de fond, sortie vers un fichier** (le code de retour et le journal viennent du processus, et aucun `docker rm` n'est jamais nécessaire : C-e334-22) ; `/health` ; `_test/seed` `with-company` ; peuplement API, repris de `boot-phase1.sh` : factures clientes validée, payée, brouillon ; fournisseur validée, payée, annulée, brouillon ; compte bancaire et relevé CAMT synthétique ; une session ouverte ; empreinte **avant** (tables, `_sqlx_migrations`, `min_required`, API) ; `.keshbackup` ; dump ; image arrêtée (par son nom propre).
2. **Montée** : binaire neuf ; migrations `avant + N` égales à `--migrations-attendues` ; si `N > 0`, **un fichier dans `<backup>/pre-upgrade/`** (#847), sinon aucun ; aucune erreur ni panique au journal ; empreinte **après**. Seuls les fichiers de `--diff-attendu` peuvent différer ; **tout autre écart échoue**. Crochet `--apres` : un script **hors dépôt**, sourcé, disposant de `SQL`, `api`, `L`, pour les vérifications propres à la release (C-e334-23).
3. **Refus** (si `N > 0`) : l'image précédente sur la base migrée **sort sans répondre**, code non nul, et son journal porte `previously applied but is missing`. Si `N = 0` : ce refus n'existe pas ; le script le **dit**, et saute.
4. **Retour** : dump restauré ; image précédente relancée ; empreinte **retour** **identique à avant**, octet à octet (md5). **Limite assumée, écrite au manuel-outil** : le retour joué ici passe par le dump ; la procédure du manuel, par le `.keshbackup` de `pre-upgrade/` dans une installation neuve, c'est **#920** (B4), qui reste à Guy et à une recette distincte.
5. **Import** (option `--import`) : `.keshbackup` de phase 1 dans le binaire neuf, sur `--base`+`_import`, empreinte comparée sous `--diff-attendu`.

**Rien de la production** (C-e334-24), **refusé avant toute action** : base hors de `^kesh_recette_[a-z0-9_]+$` ; image sans version exacte (`latest` refusé, non `gcorbaz/kesh:X.Y.Z[-suffixe]`) ; chemin de journal ou de binaire sous `$HOME/kesh/`, `$HOME/kesh-test/`, `/run/user/*/gvfs/`, `/volume1/` ; hôte de base autre que `127.0.0.1` (non configurable) ; port déjà pris. Aucun `.env` lu, aucun secret réel : mots de passe de `docs/testing.md`. Données : `Synthetique`, relevé versé dans `scripts/fixtures/recette/`. **Ne fait jamais** : `docker restart|stop|rm|compose` sur `kesh-mariadb-dev`, `GRANT`, `pkill`. Arrêt par PID et nom propre, `trap` de sortie, `ss -ltnp` final.

**Verrous** : à l'appelant, comme le reste des gates : `flock /tmp/station-heavy.lock flock <logs>/.gate.lock scripts/mem-guard.sh scripts/recette-montee.sh …`.

## Acceptance Criteria

1. **AC-1 (B1, #925)** : `scripts/gate-ci.sh` rejoue la commande exacte de `ci.yml` : la garde de T2 compare la ligne du script à celle du workflow et rougit si l'une bouge.
2. **AC-2 — il voit ce que nextest ne voit pas** : sur un test jetable de `config::tests` qui fuit une variable, `gate-ci.sh` rougit, `test-fast.sh --no-lint` non. **Rejoué à la main, relevé au Change Log** (`f5d40d7e` donne l'état d'avant : `ci-repro-lib-avant.log`, 17 échecs sans le correctif, comme point de comparaison).
3. **AC-3** : refus de base (`kesh`, `root`, hors motif) sans aucune action (`docker` factice : zéro appel) ; `DATABASE_URL` hérité **ignoré** ; code de sortie de `cargo` propagé.
4. **AC-4 — `EnvGuard`** : tout l'environnement est restitué ; un test y pose et efface une `KESH_*` et la relit après le relâchement du garde.
5. **AC-5** : `recette-montee.test.sh` (`docker`, `curl`, `sqlx` factices) : chaque refus de sécurité sans aucun appel ; l'enchaînement des phases dans l'ordre ; un écart d'empreinte hors `--diff-attendu` fait échouer ; une image qui **démarre** sur la base migrée fait échouer ; un journal sans `previously applied` fait échouer ; `N = 0` saute la phase 3 en le disant ; absence de fichier `pre-upgrade/` à `N > 0` fait échouer. Lancé par `crates/kesh-db/tests/recette_montee_script.rs`, donc au gate **et** en CI.
6. **AC-6 — recette réelle du script** : sur cette branche (89 migrations, `N = 1` par rapport à une image de 88), `--image gcorbaz/kesh:0.17.2` (elle n'a pas la migration 89 : `rel0173` l'a joué) : les cinq phases passent. Puis `--image gcorbaz/kesh:0.17.4`, `--migrations-attendues 0` : le saut est dit. Journaux cités au Change Log : **aucune image ni base du NAS, aucune donnée réelle**. Fixtures incompatibles avec la 0.17.2 : le dire, et conditionner cette partie.
7. **AC-7** : aucun chemin ni nom de `~/kesh`, du NAS, ni de personne dans `scripts/recette-montee.sh`, `scripts/gate-ci.sh`, `scripts/fixtures/recette/` (`git grep`) ; relevé CAMT relu.
8. **AC-8** : documentation, ci-dessous, et **une ligne que l'orchestrateur reprendra** au prompt d'intégration de la 0.17.5.

## Tâches

- [ ] **T1 — rouges** : `scripts/tests/gate-ci.test.sh`, `scripts/tests/recette-montee.test.sh`, enveloppes Rust, sur le modèle cité (mini-dépôt sous `mktemp -d`, nettoyé par son `trap`, comme le test existant).
- [ ] **T2** : `gate-ci.sh` ; garde `ci.yml` ⇄ script.
- [ ] **T3** : `EnvGuard`, `config.rs` (`#[cfg(test)]`) ; test.
- [ ] **T4** : `recette-montee.sh`, `scripts/fixtures/recette/` (relevés A et B, relus).
- [ ] **T5** : AC-2, AC-6 ; journaux dans `kesh-gate-logs/e3310/`.
- [ ] **T6** : documentation ; CHANGELOG `[0.17.5]` : **uniquement** une ligne *Interne* si la section la tolère ; sinon rien : aucun effet utilisateur (C-e334-25).

### Mutations nommées

P3 : pas d'exigence, mais les gardes se prouvent. **M1** : retirer le refus de `kesh` → rouge AC-3. **M2** : ignorer `--diff-attendu` (tout accepter) → rouge AC-5. **M3** : ne plus exiger `previously applied` → rouge AC-5. **M4** : changer `-j1` en `-j2` dans le script → rouge AC-1. **M5** : `EnvGuard` limité à `DATABASE_URL` → rouge AC-4.

## Gates

`scripts/test-fast.sh` (enveloppes comprises), `scripts/gate-ci.sh kesh_e3310` lui-même, une fois au moins, en grandeur : c'est sa recette. `shellcheck` si disponible : à déclarer. E2E : non touché par cette story, mais le dernier commit de code passe par la règle : à lancer si le frontend ou le backend change hors `#[cfg(test)]`, à déclarer sinon.

## Documentation

- `CLAUDE.md`, § *Test Locally First* : la phrase sur nextest ; le gate de release. § *Règle de commit et push*, liste pré-release : l'item montée et retour.
- `docs/ci.md:90` : le script, durée mesurée. `docs/testing.md` : deux lignes (autre section que 33-9). `docs/agents-orchestration.md` § verrous : l'exemple.
- `docs/manual/fr/admin-manual.tex` : **aucun changement** ; c'est un outil de développement. À contrôler : il ne le cite pas.

## Recoupements avec les autres fiches

**33-1a** doit savoir que `--diff-attendu` et `--apres` existent : ses vérifications (numérotation par date, libellés passés intacts) vont au crochet de la release, non au script. `CLAUDE.md`, `docs/testing.md` avec **33-9** (paragraphes distincts). Aucun code de production partagé.

## Hors périmètre

#920 (procédure de retour du manuel, jouée telle qu'écrite : B4, hôte de Guy) ; passer la CI à nextest (écarté : la CI est la référence, et le sériel est ce qui attrape l'état global) ; les autres actions B ; automatiser l'E2E ou la recette visuelle.

## Change Log

- 2026-11-06 : fiche créée (Opus 5.5).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
