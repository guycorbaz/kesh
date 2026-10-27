# Prompt — validation P2, Story 32-8 (la validation avant envoi — la poignée)

*Versionné le 2026-10-25. Passe **complète** (deuxième passe). **Deux lentilles, R et F**, chacune en contexte frais,
**Opus**. Lecture seule stricte, **sans `gh`**, **sans interpréteur**, **sans fichier temporaire**.*

Dépôt `/home/gcorbaz/devel/kesh-32-8`, branche `story/32-8` (partie de `main` `fa5b3468`). Fiche :
`_bmad-output/implementation-artifacts/32-8-validation-avant-envoi.md` — **quatre parties** depuis la validation P1 :
32-8a (la table), **32-8b0** (le prédicat partagé, neuve), 32-8b (la poignée côté serveur), 32-8c (l'écran et les textes) ;
registre `_bmad-output/implementation-artifacts/epic-32-choix-autonomes-328.md` (C-32-8-1 à 29 ; C-1, 5, 23 annotés,
27 à 29 neufs) ; annotations datées du § 32-8 de `_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` ; entrées `32-8*`
de `_bmad-output/implementation-artifacts/sprint-status.yaml`. La remédiation P1 est le dernier commit de la branche
(`git log -1`, `git show HEAD --stat`) ; l'entrée « validation P1 » du Change Log de la fiche la résume ; les rapports P1
sont `/home/gcorbaz/devel/kesh-gate-logs/328-validate-p1-{R,F}.md` (lis-les : chaque finding a-t-il été traité comme le
Change Log le dit ?).

Contexte : la spike `_bmad-output/planning-artifacts/epic-32-spike-mcp.md` (**en entier** ; § 2.4 (d), § 3, § 5.3.1 E1 à
E6, § 5.3.3 M-1 et M-2, § 6 entrée 32-8, § 6.1, § 6.2, § 7 et ses **réponses** Q10, Q11, Q12) ; la fiche 32-5
`_bmad-output/implementation-artifacts/32-5-cles-et-reglages-mcp.md` (routes de la 32-5c, l. 495-530). Deux fiches
validées mais non fusionnées, **en lecture seule** : `/home/gcorbaz/devel/kesh-32-6/_bmad-output/implementation-artifacts/32-6-endpoint-mcp.md`
(AC-a1, AC-a15 à AC-a24, AC-a30, AC-a31, AC-b1 à AC-b4, § *La frontière avec la 32-8*, Dev Notes) et son registre
`epic-32-choix-autonomes-326.md` ; `/home/gcorbaz/devel/kesh-32-10a/_bmad-output/implementation-artifacts/32-10a-prompts-en-base.md`
(**AC-ii1, l. 583-635**, que la 32-8b0 dit reproduire « exactement » ; AC-ii3, AC-iii1 à AC-iii3) et son registre
`epic-32-choix-autonomes-3210a.md`. Règles : `CLAUDE.md` (§ *Review Iteration Rule* et *La passe ciblée*, *Inventorier les
sites NON RÉSOLUS*, *Règle de splitting préventif*, *Pendant une boucle de revue* et son exception `kesh-db`,
*Migration breaking policy* P1 à P8, *Le prompt d'une passe doit NOMMER le manuel*, *Propagation post-patch*,
*Recompter ses propres comptes rendus*, *Recette visuelle*, *Priorités des défauts*), `docs/agents-orchestration.md`.

**Décisions de l'orchestrateur, à tenir pour vraies et à contrôler dans leur application** : ordre 32-6a, 32-6b, 32-8a,
32-8b0, 32-8b, 32-8c, puis 32-10a ; la 32-8b0 porte seule le prédicat, défini exactement selon l'AC-ii1 de la 32-10a, et
le déménagement du test de `routes/bank_accounts.rs` ; le prédicat des envois **est** ce prédicat (aucune copie) ;
l'export global intégré à la 32-8a, la partie documentaire déplacée à la 32-8c ; la ligne de `migrations.sha384` en
dernier ; l'observation (d) de la spike déclarée comme écart, sans bloquer ; `deposer` reçoit `&CleMcpAuthentifiee` ;
`entree_appel` à signature fixée et sixième clé `envoi`. **Contraintes de la story** : Q10 (c), Q12 par clé, Q11
applicative, Q4 (installation neuve), la propriété centrale (ce que l'utilisateur voit est exactement ce qui part,
octet pour octet), au plus cinq modules par partie, aucune donnée réelle. Ce que la fiche prescrit n'existe pas encore ;
ce qu'elle cite des fiches 32-6 et 32-10a se contrôle **contre ces fiches**.

⚠️ **Le motif mesuré de ce dépôt** : la sévérité se déplace vers ce qu'on vient d'écrire. Regarde d'abord **ce que la
remédiation P1 a ajouté** : la 32-8b0 (fidélité à l'AC-ii1, modules, tests, mutations M01 à M08), l'AC-A12 (export
global), le déplacement documentaire (AC-C11), `entree_appel` (AC-B11), l'AC-B15 (liste close des amendements de la
32-6), la garde `aucun_evenement_ne_nomme_le_contenu` (AC-B12), D24 à D27, MA15, MA16, MB28, MB29 et les reformulations
de MB7, MB9, MB10, MB20.

## Les deux axes obligatoires, pour les deux lentilles

- ⛔ **Motifs et mesures greppés sur le dépôt réel.** Dresse la liste des motifs et mesures écrits dans la fiche
  (`grep -nF '\b' …`, `grep -nF '→ **' …`, `grep -nF 'grep -' …`) ; rejoue **chacun** dans son périmètre et **copie la
  sortie** — dont les mesures neuves : 255 et 117 appels de macros `tracing` (AC-B12), `grep -cwE … cles_mcp.rs` → 0,
  `grep -ni invisible docs/manual/fr/*.tex` → trois lignes, les lignes citées de `exports/global.rs`,
  `migrations_upgrade_path.rs`, `admin_pat_denied_e2e.rs`, `services_motif.rs`, `test_schema_guard.rs`,
  `CHANGELOG.md`, `user-manual.tex`, `bank-accounts-crud.spec.ts`. Une mesure que ton grep ne reproduit pas est un
  finding ; un témoin vert « existant » qui n'existe pas aussi.
- ⛔ **Chaque affirmation sur une dépendance se prouve à sa source**, version du `Cargo.lock` (`~/.cargo/registry/src/*/`) :
  `unicode-properties-0.1.4/src/tables.rs` (**chaque point des deux listes** de l'AC-01 et chaque cas de l'AC-04 —
  catégorie lue, ligne copiée ; `features = ["general-category"]` dans son `Cargo.toml`), `serde_json-1.0.150`,
  `axum-core-0.5.6`, `sqlx-core-0.8.6` si une affirmation en dépend. Une affirmation sur MariaDB 10.11 que la fiche ne
  renvoie pas au test est un finding.

## Lentille R — la spécification contre le code et les fiches

1. **La remédiation P1, finding par finding** : pour R1 à R20 et F1 à F9, ce que la fiche dit avoir fait est-il fait,
   à la ligne et dans tous les sites (propagation) ? Un symptôme corrigé à un endroit et resté ailleurs (« dix
   modules », `est_cache`, `HorsBornes`, `&cle.acces`, « 32-10a-ii avant », « si la 32-10a-iii », `AC-A12` au sens du
   manuel, les compteurs 36 / 47 / D23) est un finding.
2. **La 32-8b0 contre l'AC-ii1 de la 32-10a** : les catégories, les deux listes, l'exception des emojis, la dépendance,
   le refactor de `is_forbidden_in_description`, le test déménagé et ceux qui restent (`bank_accounts.rs`, lignes
   réelles), les tests un par cas ; tout écart non déclaré entre « exactement selon l'AC-ii1 » et ce que la fiche
   écrit ; `is_hidden_in_text` est-il bien l'expression de `is_forbidden_in_prompt` ?
3. **La 32-8a** : P1 à P8 (dont la ligne de `migrations.sha384` et le double geste de `migrations_upgrade_path.rs`),
   `TABLES_HORS_EXPORT` et ses deux tests, le doc-comment `:167`, la partition de la remise à zéro, l'aller-retour
   d'import, le test `decision_modifiee_ecrit_le_texte_recu`, la ligne incohérente de l'AC-A7.
4. **La frontière avec la 32-6** : l'AC-B15 est-elle **close** — chaque changement du code ou des tests de la 32-6 que
   la fiche fait ailleurs y figure-t-il, et rien n'y figure-t-il qui ne soit fait ? `CleMcpAuthentifiee` dans
   `src/mcp/` contre la garde N24 et `mcp_sans_pool` (fiche 32-6, AC-a30) ; `MotifArgument::Invalide` ; `entree_appel`
   contre l'AC-b1/b2 de la 32-6b et l'AC-iii3 de la 32-10a ; la **note pour la 32-10a** est-elle juste et suffisante ?
5. **La spike** : E1, E2, E4, E6-4 et 6, M-1, M-2, § 6.2 : chacun a-t-il toujours son AC et son test après la
   remédiation ? L'écart C-32-8-27 est-il écrit comme la spike le formule ?
6. **Le manuel et la documentation** (axe obligatoire de `CLAUDE.md`) : AC-C10 et AC-C11 contre le code prescrit ;
   les **PDF aplatis** : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE
   ".{60}(MCP|deux tables|export).{80}"`, idem `user-manual.pdf` ; `CHANGELOG.md:1-40` ; `README.md:233`,
   `website/roadmap.html:375`, `docs/api-external.md` (`grep -n "MCP\|montr\|shown"`). Un texte que la 32-8 (dont la
   32-8b0 : description d'un compte bancaire) rend faux et que la fiche ne nomme pas ?

## Lentille F — faisabilité, découpage, sécurité et fausses prémisses

1. **Le découpage révisé** : recompte les modules de chaque partie avec la convention écrite — 5 / 3 / 5 / 4, douze
   distincts. Les fichiers touchés de chaque partie restent-ils dans ses modules (la 32-8b touche-t-elle `kesh-core`,
   `kesh-db`, `exports` ; la 32-8c touche-t-elle un module non compté) ? Le déplacement documentaire (C-32-8-29) est-il
   tenable — un texte faux sur `main` entre deux fusions, un gate qui le lirait (G9, G19, `textes_coherents.rs`) ?
   L'ordre a → b0 → b → c laisse-t-il un état intermédiaire qui casse un gate ou ouvre un accès ?
2. **La propriété centrale** : un octet qui pourrait différer entre la valeur montrée et le second bloc reçu, après la
   remédiation (prédicat déplacé dans `kesh-core`, `rendre_visible` sans `est_cache`, `deposer(cle, …)`).
3. **L'inventaire clos D1 à D27** : un chemin qui n'y est pas ; chaque ligne neuve (D24 à D27) est-elle fermée par ce
   qu'elle dit, ou réellement un angle mort ?
4. **Concurrence, échéance, redémarrage, relecture double, politique changée** : `politique_changee_entre_depot_et_relecture`
   est-il réalisable avec `set_send_validation_in_tx` (`kesh-db/src/repositories/mcp_keys.rs:304`, signature réelle) ?
5. **Les types et les gardes** : la garde `aucun_evenement_ne_nomme_le_contenu` (texte aplati, appariement de
   parenthèses, chaînes contenant des parenthèses, `"{contenu}"`) est-elle exécutable telle qu'écrite et conforme à la
   règle commune (témoins existants vraiment existants) ? `envois_lus_par_leurs_lieux`, `envoi_valide_construit_par_le_service_seul`.
6. **Tests et mutations** : MA15, MA16, M01 à M08, MB7, MB9, MB10, MB20, MB28, MB29 — réalisables, tuées par le test
   nommé, aucune équivalente ? Les témoins de la 32-8b0 (une mutation d'`is_hidden_format` rougit-elle **au crate
   `kesh-core`**, non seulement plus loin) ?
7. **Décomptes** : 42 AC (12 / 4 / 15 / 11), 18 tâches (5 / 3 / 6 / 4), 59 mutations (16 / 8 / 29 / 6), D1 à D27,
   C-32-8-1 à 29, trend P1 0/4/10/15 ; cohérence de `sprint-status.yaml` (cinq entrées `32-8*`, YAML lisible).
8. **Ce que la fiche omet** : un point pour Guy (la 32-8b0 a-t-elle hérité de tous ceux de la 32-10a qui la concernent,
   points 8 et 9 de sa fiche ?), une demande à l'orchestrateur, ce qui est laissé aux 32-7, 32-9, 32-10a, 32-11.

## Ce que tu rends

Findings `R1…` ou `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt ou de la dépendance,
preuve (commande et sortie copiée), correction proposée — et, pour chacun, s'il est **né de la remédiation P1** ou
antérieur. Un bilan d'en-tête **cohérent avec le corps**. ⛔ **La liste des axes exercés ET non exercés** (R1 à R6 ou F1
à F8, et les deux axes obligatoires) — sans elle, « 0 finding » ne compte pas comme passe. Rapport écrit **d'un seul
jet**, un seul fichier par lentille : `/home/gcorbaz/devel/kesh-gate-logs/328-validate-p2-R.md` et
`/home/gcorbaz/devel/kesh-gate-logs/328-validate-p2-F.md`. Dernier message : chemin, bilan par sévérité, une ligne par
finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, **aucun fichier temporaire**, rien dans le scratchpad ; **aucune
redirection** (`>`, `>>`, `tee`), pas même vers un fichier jetable — un fichier créé par erreur se déclare au rapport,
il ne se supprime pas. Aucun `git checkout`/`switch` sous aucune forme ; aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, **`scripts/test-fast.sh`** et
**`scripts/regen-test-schema.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `npm`, `npx`, `docker`, `curl`, `wget`,
**`python3`, `python`, `perl`, `ruby`, `node`** et tout autre interpréteur, `mariadb`/`mysql`, `sed -i`,
`git commit|add|stash|fetch|pull|reset|rebase|tag|push`, et **`gh` sous toutes ses formes** (même en lecture). `pdftotext`
vers la sortie standard seulement. Aucune donnée réelle de Guy (rien de `~/kesh-test/`). Le dépôt principal
`/home/gcorbaz/devel/kesh` est hors de ton périmètre ; les worktrees `kesh-32-6` et `kesh-32-10a` se lisent, ne
s'écrivent pas. Autorisés : lecture, `grep`, `sed -n`, `git log|show|diff|blame|merge-base`, `pdftotext … -`, et les
filtres en lecture seule `tr`, `sort`, `cut`, `head`, `tail`, `awk`, `wc`, `uniq` vers la sortie standard.
