# Prompt — validation P1, Story 32-4a (la couche de services : rollout des lectures — écritures, postes ouverts, contacts)

*Versionné le 2026-10-24. Passe **complète** (première passe). **Deux lentilles, R et F**, chacune en contexte frais,
**Sonnet**. Lecture seule stricte.*

Dépôt `/home/gcorbaz/devel/kesh-32-4a`, branche `story/32-4a` (partie de `main` `89d68704`, la 32-3 fusionnée). Fiche :
`_bmad-output/implementation-artifacts/32-4a-services-ecritures-contacts.md` ; registre :
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-324a.md` (C-32-4a-1 à 12). **Le motif à appliquer** :
fiche `32-3-services-motif.md` (§ *Le motif*, AC7, AC8, AC10, Dev Notes R1-R8), registre `epic-32-choix-autonomes-323.md`
(C-32-3-34 à 41 surtout), et le code réel `crates/kesh-api/src/services/` et `crates/kesh-api/tests/services_motif.rs`.
Contexte de l'epic : `_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` (§ *32-4*, § *32-9*) ; registre
`epic-32-choix-autonomes.md` (C-32-0-4, C-32-0-7). Issue des LOW de la 32-3 : `gh issue view 738 --json body,comments`.
Règles : `CLAUDE.md` (§ *Review Iteration Rule*, *La passe ciblée* — une boucle ne se clôt que sur une passe sans
finding au-dessus de LOW —, *Règle de splitting préventif*, *Pendant une boucle de revue* et son exception `kesh-db`,
*Le prompt d'une passe doit NOMMER le manuel*, *Inventorier les sites NON RÉSOLUS*, *Recompter ses propres comptes
rendus*), `docs/agents-orchestration.md` § 3, § 6 et § 6.1.

**Contraintes de la story, à tenir pour vraies et à contrôler** : le comportement de l'API ne change pas ; aucune
migration, `kesh-db` non touché ; au plus cinq modules ; seules les lectures que serviront les outils (C-32-0-4) ;
aucune donnée réelle.

## Lentille R — la spécification contre le code

Pour chaque affirmation de la fiche sur le code, ouvre le site cité (`sed -n`, `grep -nF`) et **copie la sortie**.
Axes, tous à exercer :

1. **Les faits 1 à 9** (§ *Ce que le code dit*) : 17 handlers, 7 lectures, 10 mutants (recompte-les) ; les 17 corps
   portent-ils tous `state.pool` ? RBAC des sept lectures dans `lib.rs` ; les trois régimes de pagination — défauts,
   plafonds, et **qui écrête** (handler, dépôt : relis `list_by_company_paginated` des écritures et des contacts, et
   `open_items`) ; l'ordre et le texte des neuf refus de la liste des écritures ; la forme du détail d'une écriture et la
   garde lexicale qui la fige (lis-la entière) ; l'ordre des lectures des contacts et des postes ouverts ; le motif tel
   qu'il est dans `services/` ; les lignes citées de `services_motif.rs` ; `MULTI-TENANT-SCOPING-PATTERNS.md` et
   `api-external.md`.
2. **Inventaire clos des handlers de lecture** : y a-t-il, dans les trois fichiers, une lecture que la fiche ne nomme
   pas ? Et hors de ces fichiers, une lecture des mêmes données que l'outil MCP de la 32-9a/32-9b servirait (les
   personnes d'un contact, une autre route `GET` sur les écritures ou le lettrage, `routes/accounts.rs`) ? Le choix
   C-32-4a-1 (cinq migrées, deux figées) tient-il contre la lettre de l'epic (« aucun handler de lecture des modules
   visés n'appelle plus un dépôt directement ») ?
3. **Comportement inchangé** : pour chacun des cinq handlers, l'ordre des erreurs après déplacement (extracteur `Query`
   d'Axum, analyse, contrôles, société, entité) ; les défauts serde des DTO qui rendraient les constantes du service ;
   le plafond commun des propositions ; la conversion `CoreJournal → DbJournal` ; le libellé des contacts (société et
   `i18n`). Un test existant pourrait-il rougir ? Refais le `grep` de l'AC8 : la liste de seize fichiers et leurs
   comptes `sqlx::test` sont-ils exacts ?
4. **Le contrôle d'accès** : exigences du tableau du *Rollout* contre le RBAC ; `Portee::ContactsRead` et ses
   conséquences sur `services_motif.rs` (`match` exhaustif, `TOUTES`, autres `match` sur `Portee` dans `src/` ou
   `tests/`) ; les cas (a), (b), (c-val), (c-lec), (e) de l'AC9 sont-ils constructibles, non vides, et chacun distinct
   de `Forbidden` sans le contrôle ? Le cas d'ordre de `carnet::detail` rend-il bien `NotFound` aujourd'hui ?
5. **Lecture seule et inventaire des écritures (AC11, AC13)** : relis chaque dépôt appelé — une écriture, un
   `FOR UPDATE`, un `get_or_create`, une transaction que `SET SESSION TRANSACTION READ ONLY` refuserait ? L'AC11 (a)
   exécuterait-il chaque lecteur sur les fixtures annoncées (un compte **lettrable** existe-t-il dans
   `seed_accounting_company`) ?
6. **La garde lexicale (AC12)** : (ii) étendue et son témoin ; (iii) — l'ensemble des douze est-il exact, et la garde
   rougit-elle pour un handler neuf, renommé, ou dont `state.pool` est coupé par rustfmt ? (viii) et son motif
   (`\btrace\b` attrape-t-il un usage légitime, `tracing`, un doc-comment ?) ; la v-2 reste-t-elle à deux fabriques ?
   Le compte de `services/` à huit ?
7. **Mutations (AC14)** : chacune des 33 est-elle réalisable telle qu'écrite et tuée par l'assertion nommée ? Une
   passerait-elle à vide (M7 couverte deux fois ? M8 vraiment équivalente — relis le dépôt ; M14 : l'erreur 1792 remonte-
   t-elle, ou un `?` l'avale-t-il ? M17 : quelle ligne de `rapports.rs` la porte, et la ligne 7 la voit-elle) ?
8. **Manuel et documentation** : la fiche dit « rien de visible ne change ». Vérifie qu'aucun texte ne décrit autrement
   ce qui bouge : `docs/api-external.md` (l. 213, 216-218, 397-460), `docs/manual/fr/user-manual.tex` et
   `admin-manual.tex` (carnet d'adresses, écritures, postes ouverts, pagination), et **les PDF aplatis** :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE ".{80}(postes ouverts|par page|contacts).{80}"`.
   La phrase prévue pour `MULTI-TENANT-SCOPING-PATTERNS.md` sera-t-elle vraie ?

## Lentille F — faisabilité et fausses prémisses

1. **Prémisses** : chaque « établi au code » tient-il ? Une prémisse de l'epic reprise sans vérification (« `routes/letterings`
   ou `accounts/{id}/open-items` », « 4 avec `services` », « `clamp(1, 200)` ») ?
2. **Périmètre et découpage** : quatre modules — juste (`lib.rs`, `helpers.rs` vraiment non touchés ?) ; aucune
   migration ni dépôt `kesh-db` ; le déplacement de la garde de transaction (C-32-4a-7) et l'extraction de l'aide de
   lecture seule dans `tests/common/` (C-32-4a-9 : chaque binaire qui fait `mod common;` la compile — `dead_code` ?)
   sont-ils sûrs ?
3. **Faisabilité en Rust** : les signatures de l'esquisse compilent-elles sans contorsion (types de `kesh_db` rendus,
   `Company` dans `PageContacts`, la transaction du détail dans un service qui ne reçoit qu'une `Lecture`, `parse_as_of`
   déplacée) ? `services/carnet.rs` et `kesh_db::repositories::contacts` — collision ? Une constante `pub` du service
   lue par une fonction `default_*` serde ?
4. **Décomptes** : recompte à la source chaque nombre — 17 handlers, 7 lectures, 10 mutants, 12 sites non résolus, six
   corps de (ii), 16 fichiers et leurs comptes, neuf refus, huit fichiers de `services/`, 17 AC, 33 mutations, quatre
   modules, C-32-4a-1 à 12.
5. **Cohérence interne** : AC1 à AC17, T0 à T7, registre, *Hors périmètre*, *Fichiers touchés*, Change Log —
   contradiction, critère invérifiable, commande mal formée (essaie mentalement **chaque** commande des AC7, AC8, AC10,
   AC13, AC16 contre le code actuel **et** contre un correctif plausible : rend-elle ce que la fiche annonce ?).
6. **#738** : la décision sur chacun de ses treize points (C-32-4a-10, *Hors périmètre*) est-elle juste — un point
   repris qui n'a rien à faire ici, un point laissé que le rollout rend plus coûteux plus tard ?
7. **Ce que la fiche omet** : un appelant des constantes ou de `parse_as_of` hors de leur fichier ; une garde existante
   qui lit ces sources ou compte des fichiers ; le frontend et l'E2E (un écran lit-il un champ qui pourrait changer) ;
   la recette visuelle ; le CHANGELOG.

## Ce que tu rends

Findings `R1…` ou `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt, preuve (commande et
sortie copiée), correction proposée. Un bilan d'en-tête **cohérent avec le corps**. ⛔ **La liste des axes exercés ET
non exercés** (R1 à R8, F1 à F7) — sans elle, « 0 finding » ne compte pas. Ce que la fiche **prescrit** au développement
n'existe pas encore : son absence n'est pas un défaut. Rapport écrit **d'un seul jet**, un seul fichier par lentille :
`/home/gcorbaz/devel/kesh-gate-logs/324a-validate-p1-R.md` et `/home/gcorbaz/devel/kesh-gate-logs/324a-validate-p1-F.md`.
Dernier message : chemin, bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, **aucun fichier temporaire**, rien dans le scratchpad ; **aucune
redirection** (`>`, `>>`, `tee`), pas même vers un fichier jetable — un fichier créé par erreur se déclare au rapport,
il ne se supprime pas. Aucun `git checkout`/`switch` sous aucune forme ; aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `npm`, `npx`,
`docker`, `curl`, **`python3`, `python`, `perl`, `ruby`, `node`**, `mariadb`/`mysql`, `sed -i`,
`git commit|add|stash|fetch|pull|reset|rebase|tag|push`, `gh` en écriture. `pdftotext` vers la sortie standard
seulement. Aucune donnée réelle de Guy (rien de `~/kesh-test/`). Le dépôt principal `/home/gcorbaz/devel/kesh` est hors
de ton périmètre. Autorisés : lecture, `grep`, `sed -n`, `git log|show|diff|blame|merge-base`, `gh issue view|list`,
`pdftotext … -`, et les filtres en lecture seule `tr`, `sort`, `cut`, `head`, `awk`, `wc`, `uniq` vers la sortie
standard.
