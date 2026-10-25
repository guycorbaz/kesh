# Prompt — validation P3, Story 32-5 (les clés MCP et les réglages du serveur, en base)

*Versionné le 2026-10-25. Passe **complète** (troisième passe ; P1 Sonnet : R 0/2/3/7, F 0/0/5/4 ; P2 Opus :
R 0/0/3/7, F 0/1/4/10). **Deux lentilles, R et F**, chacune en contexte frais, **Sonnet**. Lecture seule stricte.*

Dépôt `/home/gcorbaz/devel/kesh-32-5`, branche `story/32-5` (partie de `main` `e59d63e8`). Fiche :
`_bmad-output/implementation-artifacts/32-5-cles-et-reglages-mcp.md` (quatre parties : 32-5a, 32-5b, 32-5c, 32-5d) ; registre :
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-325.md` (C-32-5-1 à 27). Contexte de l'epic :
`_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` (§ *32-5*, § *Ce que le code dit déjà*, § *Points
d'attention*, § *Questions pour Guy* et **§ *Réponses de Guy (2026-10-24, soir)***) ; spike :
`_bmad-output/planning-artifacts/epic-32-spike-mcp.md` (§ 4.3, § 4.4, § 5.3, § 6, § 6.1, § 6.2, § 7 et ses
réponses) ; registres `epic-32-choix-autonomes.md` (C-32-0-6 à 9, 12, 13) et `epic-32-choix-autonomes-323.md`
(C-32-3-4) ; fiche `32-3-services-motif.md` (le motif `ContexteAcces`, `Portee`, `Lecture`, `Trace`). Règles :
`CLAUDE.md` (§ *Migration breaking policy* P1 à P8, § *Review Iteration Rule*, *La passe ciblée* — une boucle ne se
clôt que sur une passe sans finding au-dessus de LOW —, *Règle de splitting préventif*, *Recette visuelle*, *Pendant
une boucle de revue* et son exception `kesh-db`, *Le prompt d'une passe doit NOMMER le manuel*, *Inventorier les sites
NON RÉSOLUS*, *Recompter ses propres comptes rendus*), `docs/agents-orchestration.md` § 3, § 6 et § 6.1.

**Contraintes de la story, à tenir pour vraies et à contrôler** : migration **non breaking** (`CREATE TABLE`) ; aucune
donnée écrite par la migration ; **Q11** : lecture seule applicative + mode de session `READ ONLY`, **ni utilisateur
MariaDB ni variable `.env` de plus** — la fiche ne doit rien supposer d'autre ; **Q12** : politique de validation par
clé, portée en base ici, appliquée par les stories suivantes ; **Q4** : la limite (sauvegarde importable seulement par
une version aux mêmes tables) s'écrit au manuel et au CHANGELOG ; **Q3, Q5, Q7** sans réponse — hypothèses de travail
écrites comme telles ; au plus cinq modules **par partie** ; aucune donnée réelle.

**Ce que les passes précédentes ont trouvé et ce que la remédiation a changé** : lis les entrées *Validation P1* et
*Validation P2* du Change Log de la fiche, et les rapports `/home/gcorbaz/devel/kesh-gate-logs/325-validate-p2-R.md` et
`325-validate-p2-F.md` (ceux de la P1, `325-validate-p1-{R,F}.md`, au besoin). ⚠️ **La sévérité se déplace vers ce
qu'on vient d'écrire** (`CLAUDE.md`, *La passe ciblée*), et la P2 a **déclaré du recyclage** (cinq findings nés de la
remédiation P1) : relis en priorité ce que la remédiation P2 a ajouté ou réécrit — AC-a6 (`populate`), AC-a9 (les
justifications et la garde `post_restore.rs:1178`), AC-a10 (le test d'ordre, les neuf appelants de `replay_retired`),
AC-a13 (1) à (5) (Compatibilité, base vide, *Reprises*, *Ce qui reste hors du fichier*, *Restaurer sur une nouvelle
machine*, format 1), AC-a14, **AC-a15** (neuf), AC-b2, AC-b3 (α et β, port fermé), AC-b4 (a), AC-b5 (`tests/common/`),
AC-c2, AC-d1, AC-d3 (manuel utilisateur), l'ordre **a → b → d → c** (C-32-5-27), C-32-5-26, les faits 10, 17 et 19,
le recompte des modules, et l'annotation datée du fichier d'epic (§ *Réponses de Guy*, puce Q4) ainsi que les reports
inscrits à ses sections 32-6 et 32-7 — et vérifie que **chaque finding de la P2 est réellement remédié**, non
seulement déclaré.

**#722, recopié ici** (aucune commande `gh` n'est permise à cette passe) : CR « **la sauvegarde emporte tout ce qu'il
faut pour redémarrer, hors `.env`** ». Il demande un **inventaire clos** de ce qui vit hors du `.keshbackup` —
réglages de l'Epic 32 compris (prompts, activation du serveur, clés MCP), et le réglage de la sauvegarde automatique
(`_kesh_backup_schedule`), qui en est exclu aujourd'hui — et, pour chaque élément, de **l'emporter** ou de **dire
pourquoi il reste à part**, au manuel d'administration § restauration. Juge la cohérence de la fiche avec ce texte ;
ne prétends pas avoir lu l'issue elle-même.

## Lentille R — la spécification contre le code

Pour chaque affirmation de la fiche sur le code — faits 1 à 18 réécrits compris —, ouvre le site cité (`sed -n`, `grep -nF`) et **copie la sortie**.
Axes, tous à exercer :

1. **Les faits 1 à 18** (§ *Ce que le code dit*) : chaque numéro de ligne, chaque citation. En particulier : le
   middleware de l'API rend-il **vraiment** 401 à un jeton `kesh_mcp_…` (chemin `jwt::decode`, et chemin cookie) ? Le
   compte de 39 tables (refais la commande) ; la règle 4 et les règles 1 et 3 du test de partition, et ce que chacune
   devient quand `mcp_keys` est conservée et `mcp_server_settings` vidée ; les **quatre** consommateurs de
   `OrphanPrincipals` et ses tests à égalité JSON stricte — sont-ils les seuls (`grep -rn "api_keys_revoked\|api_keys_repointed\|OrphanPrincipals"
   crates`) ?
2. **La fenêtre de P7 (AC-a9, AC-a10)** : relis `post_restore.rs` en entier autour du registre, des exemptions et des
   tests de fenêtre. Est-il exact que la création de `mcp_keys` fait de `20261025000001` la nouvelle
   fenêtre (`creates_application_table`, `TABLES_TO_TRUNCATE`) et que `20261010000001` en sort ? Quelles autres
   exemptions, entrées retirées ou commentaires deviennent faux (lis les **vingt** justifications) ? Quels tests, hors
   des cinq nommés, lisent le registre de production ou une entrée précise (`grep -rn "POST_RESTORE_BACKFILLS\|
   20261010000001\|LETTERING_BACKFILL" crates`) ? La conversion à `replay_retired` est-elle faisable pour chacun des
   quatre tests (signature, type rendu, manifeste disponible) sans les faire passer à vide ?
3. **L'inventaire et l'export** : la place proposée des deux tables dans `TABLES_TO_TRUNCATE` (ordre enfants → parents,
   FK réelles du schéma proposé) ; le compte à 41 ; `TABLES_HORS_EXPORT` et sa garde ; un autre site qui compte les
   tables ou les énumère (`grep -rnE "\b39\b|TABLES_TO_TRUNCATE.len" crates docs`) ; `restore_body` traite-t-il les deux
   tables sans exception (lis `backup.rs:627-730`) ?
4. **Le schéma proposé** (Dev Notes) : faisabilité sous MariaDB 10.11 (`JSON_TYPE`, `JSON_LENGTH` dans un `CHECK` ;
   `JSON NOT NULL` sans défaut) ; aller-retour d'une colonne `JSON` par `decode_cell` et l'import ; `ENGINE` explicite ;
   cohérence avec P3 (aucune opération de la liste) et P8.
5. **L'authentification (AC-b3 à AC-b6)** : faisabilité de `services::cles_mcp::authentifier` sous les gardes
   lexicales de `tests/services_motif.rs` (v-1, v-2, vi) ; la `Trace` et son doc-comment ; `ContexteAcces::mcp` ;
   `Portee` réexportée depuis `kesh-db` — les dérivations et les emplois de la 32-3 (`grep -rn "Portee" crates`) :
   la réécriture prescrite de `services_motif.rs` (`TOUTES`, `autre_portee`, `ac10_vi`) est-elle complète, et garde-t-elle au
   refus de portée sa force ? Le test de clé restaurée (export, suppression, import) est-il montable avec
   les helpers existants d'`admin_full_import_e2e.rs` sans privilège posé à la main (`docs/agents-orchestration.md`
   § 6.1) ?
6. **Les routes (AC-c1 à AC-c7)** : le bloc `admin_routes` et ses deux couches (`lib.rs:221-402`) ; ce que rend chaque
   cas de l'AC-c2 (relis `rbac.rs`, `require_auth`) ; `admin_pat_denied_e2e.rs` et `audit_route_registry.rs` — ce
   qu'il faut y ajouter ; `audit_labels.rs`, `audit_label_registry.rs`, les clés i18n et la garde G9 de
   `textes_coherents.rs` ; `ipnet` réellement dans `Cargo.lock` et compatible.
7. **Le manuel et la documentation** : la fiche change `docs/manual/fr/admin-manual.tex` (l. 2230, 2237-2243, 2246, § 2311-2330 en
   32-5a ; l. 2929 en 32-5c ; l. 1414, 1416, 2461 en 32-5d), `user-manual.tex:3027` et `docs/api-external.md` § 2. Vérifie le texte actuel **et le PDF aplati** :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE ".{80}(condensés des clés|Compatibilité|retrouve, à l'import).{120}"`.
   Un autre passage du manuel (administration **et** utilisateur), du CHANGELOG, du README, de `website/` ou de
   `docs/api-external.md` devient-il faux du fait de la 32-5 — inventaire des tables, restauration, remise à zéro
   (`admin-manual.tex:1414`), clés d'API « toujours `kesh_pat_` », reprises rejouées ? Ce que la fiche renvoie à la
   32-7 (section MCP du manuel) peut-il attendre sans qu'un texte mente entre-temps ?

## Lentille F — faisabilité, découpage et fausses prémisses

1. **Prémisses** : chaque « établi au code » tient-il ? Une prémisse du kickoff ou de la spike reprise sans
   vérification (prémisse 3 : « une clé restaurée reste valide, puisque son condensat revient » ; prémisse 4 ;
   C-32-0-6 « l'endpoint MCP ne lit jamais `api_keys` ») ?
2. **Découpage (C-32-5-1, révisé)** : le compte « dix modules distincts, dix-sept en somme » et sa ventilation 5 / 3 / 4 / 5 — recompte avec la convention
   écrite ; chaque partie est-elle fusionnable seule, gate vert, **sans état intermédiaire qui ouvre un accès** ou fasse
   mentir un texte ? L'ordre imposé a → b → d → c (C-32-5-27) est-il nécessaire et suffisant, et chaque phrase de la fiche qui en dépend est-elle vraie dans cet ordre ? Les clés du registre de sprint (`sprint-status.yaml`, `32-5a` à `32-5d`) correspondent-elles à la fiche ? Les
   fichiers touchés de chaque partie restent-ils dans ses modules ?
3. **Fidélité aux décisions de Guy** : Q11 (rien d'autre supposé : cherche toute mention d'un second utilisateur,
   d'une variable d'environnement, d'un pool neuf) ; Q12 (porté, non appliqué, et les stories qui l'appliquent sont-elles
   nommées juste ?) ; Q4 (le texte prescrit pour le manuel dit-il exactement la règle et la marche à suivre ?) ; Q3 et
   Q7 écrits comme hypothèses ; la consigne `RESET_PRESERVED_TABLES` « comme `api_keys` ».
4. **Tests et mutations** : chaque AC est-il vérifiable tel qu'écrit ? Chaque mutation Ma1 à Md4 (Ma7 neuve) est-elle réalisable et
   tuée par le test **nommé** — une passerait-elle à vide (en particulier Mb3, sur une `Lecture` qui partage le pool de
   production ; Mb6, au pool injoignable ; Ma6, redéfinie ; Md3 et Md4, sur l'orphelin MCP seul) ? Un test converti (AC-a10) peut-il passer à vide ? Une commande d'AC (AC-a1, AC-a11,
   AC-a13, AC-a14, AC-b1) rend-elle ce que la fiche annonce, contre le code actuel **et** contre un correctif
   plausible ?
5. **Décomptes** : recompte à la source — 39 tables, 83 migrations et les compteurs de l'audit d'idempotence (10, 73,
   0), 8 et 20 exemptions, 31 AC (15 / 6 / 7 / 3), 23 tâches (8 / 5 / 5 / 5), 23 mutations (7 / 6 / 6 / 4), C-32-5-1 à 27, les compteurs de `audit_route_registry.rs` (118 → 122, 127 → 131), les « 33 → 39 » et « 124/127 → 128/131 » de l'AC-c2, les modules par partie.
6. **Ce que la fiche omet** : une table, une garde ou un site qui lit l'inventaire, la partition de la remise à zéro,
   le registre de P7 ou les principaux orphelins et que la fiche ne nomme pas ; la recette visuelle (aucun écran ne
   change ?) ; le README et `website/` (une fonctionnalité annoncée ?) ; `scripts/prepare-release.sh` et les exemptions
   `PerishableSince` ; le gate E2E et P8 sur `kesh_e2e_325` ; #722, d'après le texte recopié ci-dessus (la fiche y ajoute-t-elle quelque chose ou en
   retire-t-elle ?).

## Ce que tu rends

Findings `R1…` ou `F1…` (numérotés à neuf, sans reprendre ceux de la P1), sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt, preuve (commande et
sortie copiée), correction proposée. Un bilan d'en-tête **cohérent avec le corps**. ⛔ **La liste des axes exercés ET
non exercés** (R1 à R7, F1 à F6) — sans elle, « 0 finding » ne compte pas. Ce que la fiche **prescrit** au développement
n'existe pas encore : son absence n'est pas un défaut. Rapport écrit **d'un seul jet**, un seul fichier par lentille :
`/home/gcorbaz/devel/kesh-gate-logs/325-validate-p3-R.md` et `/home/gcorbaz/devel/kesh-gate-logs/325-validate-p3-F.md`.
Dernier message : chemin, bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, **aucun fichier temporaire**, rien dans le scratchpad ; **aucune
redirection** (`>`, `>>`, `tee`), pas même vers un fichier jetable — un fichier créé par erreur se déclare au rapport,
il ne se supprime pas. Aucun `git checkout`/`switch` sous aucune forme ; aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`** et **`scripts/regen-test-schema.sh`**), `make`,
`latexmk`, `cargo`, `rustup`, `npm`, `npx`, `docker`, `curl`, **`python3`, `python`, `perl`, `ruby`, `node`**,
`mariadb`/`mysql`, `sed -i`, `git commit|add|stash|fetch|pull|reset|rebase|tag|push`, **toute commande `gh`**, en lecture comme en écriture. `pdftotext`
vers la sortie standard seulement. Aucune donnée réelle de Guy (rien de `~/kesh-test/`). Le dépôt principal
`/home/gcorbaz/devel/kesh` est hors de ton périmètre. Autorisés : lecture, `grep`, `sed -n`,
`git log|show|diff|blame|merge-base`, `pdftotext … -`, et les filtres en lecture seule `tr`,
`sort`, `cut`, `head`, `awk`, `wc`, `uniq` vers la sortie standard.
