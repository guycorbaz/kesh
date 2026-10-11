# Prompt — validation P3 de la spec, Story 26-3 (manuels et garde de kesh-restore)

*Versionné le 2026-10-10. Passe **complète**, deux lentilles (**Sonnet**), contexte frais chacune, en lecture seule.
Rotation (D6, rétro 25) : P1 Sonnet → P2 Opus → P3 Sonnet. Passe complète et non ciblée : la remédiation P2 change
une **règle de test** (la garde se prouve désormais contre un faux `docker`).*

Dépôt `/home/gcorbaz/devel/kesh-26-3`, branche `story/26-3` (base `fe162504`, = v0.13.0 + kickoff de l'Epic 26).
Fiche : `_bmad-output/implementation-artifacts/26-3-manuel-et-restauration-synology.md`. **Remédiation à relire :
commit `f9d5a107`** (`git show f9d5a107`), qui applique la P2 (rapports
`/home/gcorbaz/devel/kesh-gate-logs/26-3-validate-p2-{R,F}.md`) ; la P1 avait été remédiée en `f5c4a958`. Bilan des
deux passes au Change Log de la fiche. Registre : `epic-26-choix-autonomes.md` (C-26-3-1 à 17 ; **13 à 17 nés de la
P2**, 10 remplacé par 13). Epic : `_bmad-output/planning-artifacts/epic-26-recette-v0-13.md`. Issues #616, #617, #619
(`gh api repos/guycorbaz/kesh/issues/N` et `.../N/comments`). Règles : `CLAUDE.md` (§ Review Iteration Rule, § Le
prompt d'une passe doit NOMMER le manuel, § Inventorier les sites NON RÉSOLUS, § Propagation post-patch, § Recompter
ses propres comptes rendus). Fiches voisines : `../kesh-26-1`, `../kesh-26-2` (lecture seule).

**Contexte** : la P2 a levé un signal D5 (recyclage : la garde lit tout l'hôte, que la recette ne maîtrise pas).
L'orchestrateur a tranché **par simplification** (C-26-3-13) : la section neuve de la recette n'emploie aucun vrai
conteneur. La P3 doit dire si cette simplification **tient** et si elle n'a pas, à son tour, introduit ses défauts.

Axes prioritaires :

1. **Le faux `docker` (AC 6, C-26-3-14) — exécution raisonnée.** Sans rien exécuter : la fonction bash exportée
   (`export -f docker` dans le sous-shell de chaque cas) est-elle bien importée par le `bash` qui lance
   `kesh-restore.sh`, et appelée **avant** `/usr/bin/docker` malgré `export PATH=…` (`kesh-restore.sh:54`) ? Le
   script appelle-t-il `docker` ailleurs que par son nom nu (`command docker`, chemin absolu, `env docker`, sous-shell
   `sh -c`) — ce qui contournerait le faux ? `set -euo pipefail` : une fonction qui rend 97 dans `LISTE=$(docker …)`
   fait-elle sortir le script ? Le faux reconnaît-il exactement la chaîne `"$*"` du `--format` (guillemets de
   `{{.Label "com.docker.compose.service"}}` après découpage des arguments) ? L'émulation de la forme **d'origine**
   (`--filter label=… --format {{.State}}`) est-elle assez fidèle pour la mutation (O) ?
2. **Les cas G1-G17 et leurs classes** : déroule chaque cas contre la garde **prescrite** par l'AC 5 (boucle
   `IFS='|' read -r ETAT IMAGE SERVICE NOM RESTE`, compteur, filet `^(docker\.io/)?gcorbaz/kesh([:@].*)?$`, sortie
   sur zéro, « illisible ») : la classe annoncée est-elle la seule possible ? L'ordre des lignes compte-t-il (refus
   dans la boucle avant la sortie sur zéro) ? `printf` d'une ligne vide, d'une sortie vide (G5), d'une empreinte de
   64 hex ; G15 (angle mort assumé) et G16 (image voisine) — justes ? Le montage commun (`$W/garde`, `.cnf` factice,
   dump fabriqué par `printf … | gzip` et son `.sha256`) suffit-il pour que le cas PASSE atteigne l'étape 3
   (`kesh-restore.sh:56-93` : `$1` résolu, `.cnf`, `kesh-dump.sh` voisin, verrou, `sha256sum -c`, `gzip -t`, lecture
   de `CREATE DATABASE`) et s'y arrête sur « recrée la base » ?
3. **Les mutations de l'AC 6 (O, F, Z, I, R)** : refais le déroulé de chacune sur les 17 cas ; les ensembles de cas
   rouges annoncés sont-ils **exacts** (ni un de plus, ni un de moins) ? Le journal sous (O) (forme d'origine) est-il
   compatible avec l'oracle « une ligne qui commence par `ps --all` » ?
4. **Ce que la simplification a retiré** : la section 4 réelle exerce-t-elle encore la garde **nouvelle** contre de
   vrais conteneurs (`running`, `restarting`, `paused`) et les sections 5 à 9 la font-elles passer (un `kesh-api`
   arrêté du projet de recette, compteur ≥ 1) ? Le prérequis « hôte » écrit dans `docs/testing.md` remplace-t-il
   honnêtement le pré-vol abandonné ? Reste-t-il dans la fiche ou le registre une prescription de l'ancienne 4-bis
   (pré-vol, `KESH_TAG`, `compose_factice` paramétré, `up --no-start` dans la recette, conteneur hors compose) ?
5. **G16 et G20 amendées (AC 7, AC 9)** : la regex 7.1 « indifférente au séparateur » est-elle écrivable sans
   laisser passer la garde d'origine ? Le tableau de l'AC 7.6 est-il exact clause par clause (lis
   `configuration_transmise.rs:2866-3466`) ? Les expressions de G20 (`\\(?:textbf|emph)\{Compte\s+de\s+banque\}`,
   `«\s*~?\s*Compte\s+de\s+banque\s*~?\s*»`, `[Tt]outes\s+les\s+banques\s+suisses`) : faux positifs sur les six
   « son propre compte de banque » ? faux négatifs ?
6. **Le manuel et les écrans, contre le code — et le PDF** : AC 2.3 réécrite (profil choisi avant le dépôt ;
   Annuler/choisir/redéposer) contre `BankImportUpload.svelte` ; AC 2.3-bis (rapport d'import, doublons) ; cibles
   DE/EN/IT fixées (C-26-3-17) contre le glossaire `:105`/`:151` et `nav-bank-import` ; `<title>` et `aria-label`
   lus par i18n ; colonne « précédent » du glossaire (C-26-3-16). Inventaire du symptôme sur les **PDF aplatis**
   (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`) et les `.tex`.
7. **Propagation et décomptes** : l'AC 11.2 (tableau borné) — rejoue chaque commande sur HEAD et compare à la colonne
   « attendu » (en tenant compte de ce qui ne sera vrai qu'après développement) ; Change Log P2 (R 0H/5M/12L, F
   0H/9M/8L, « 10 MEDIUM distincts », « 8 introduits », trend) recompté contre les rapports P2 ; fiche ↔ registre
   (C-26-3-3/5/8/10/12 amendés ou remplacés) ; 12 AC, tâches T0-T8.

## Lentilles

- **R — Auditeur d'acceptation** : `.claude/skills/bmad-create-story/checklist.md` ; chaque finding P2 réellement
  résolu par `f9d5a107` ou rendu sans objet à bon droit (vérifie-les un par un, avec preuve) ; chaque phrase des trois
  issues couverte par un AC testable, chaque AC par une tâche, chaque tâche par un test ; faits cités revérifiés.
- **F — Full-scope adversary** : ce que le faux `docker` laisse prouver **à vide** (un test vert qui n'établit rien) ;
  ce que la garde laisse passer et qu'aucun cas G ne couvre ; ce que le manuel corrigé affirmerait de faux ; défauts
  **introduits** par `f9d5a107`.

## Ce que tu rends

Findings numérotés avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie copiée),
correction proposée, et pour chacun : **origine** — défaut d'origine, introduit par `f5c4a958`, ou introduit par
`f9d5a107`. Toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée. ⛔ **La liste des axes exercés ET
non exercés** — un « 0 finding » sans elle ne compte pas. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/26-3-validate-p3-<R|F>.md` ; dernier message : le chemin, le bilan, une ligne par
MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/` (pas de dossier temporaire ailleurs ; si `occ` en
demande un, place-le sous `kesh-gate-logs/` et dis-le). Aucune commande qui écrit, compile ou exécute : `scripts/*`
(dont `scripts/prepare-release.sh` et les scripts Synology `kesh-dump.sh`, `kesh-restore.sh`, `recette.sh`), `make`,
`latexmk`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`reset`/`rebase`, `cargo`, `npm`, `npx`, `docker`
(y compris `docker ps`), `bash` sur un script du dépôt ou sur une fonction qui l'imite, `gh` en écriture, SQL. Le
déroulé des axes 1 à 3 est **raisonné**. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh api` en
lecture, `pdftotext` vers la sortie standard.
