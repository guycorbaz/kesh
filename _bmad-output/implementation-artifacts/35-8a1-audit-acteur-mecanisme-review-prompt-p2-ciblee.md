# Prompt — revue de code P2 ciblée de la Story 35-8a1 (refs #431, refs #892), une lentille, braquée sur la remédiation de la P1

Passe **ciblée**, une seule lentille, contexte frais, **Sonnet** (P1 : Opus, trois lentilles), en lecture seule. Worktree
`/home/gcorbaz/devel/kesh-e358a1`, branche `story/35-8a1-audit-acteur-mecanisme`. La P1 a rendu 1 MEDIUM (B1 = E1 : la garde ne
contrôlait ni la raison ni les appelants des fonctions exemptées) et 14 LOW (rapports
`/home/gcorbaz/devel/kesh-gate-logs/e358a1-review-p1-{B,E,A}.md`). Déjà tracés, à ne pas rapporter de nouveau : #1130 (None positionnel
des 26 fonctions), #1131 (annulation d'un rapprochement sans clé), #1132 (`pieges.rs`).

## Périmètre : le seul commit `b3a14f752`

`git -C /home/gcorbaz/devel/kesh-e358a1 show b3a14f752` — sept fichiers : `crates/kesh-api/tests/audit_acteur_garde.rs` (+419),
`crates/kesh-api/tests/common/source_assainie.rs`, `crates/kesh-db/src/entities/audit_log.rs` (doc-comment d'`Acteur` seulement : à
vérifier), le manuel d'administration (`.tex` et PDF), le CHANGELOG, le registre `e35-choix-autonomes-e358a1.md` (C-e358a1-3). Compte
rendu : `git show e86709a34` (entrée P1 du Change Log, annotations du Dev Agent Record). Journaux sous
`/home/gcorbaz/devel/kesh-gate-logs/e358a1/` : `p1-references-releve.log`, `mut/M7.log`, `mut/M8.log`, `mut/M2-garde.log`,
`test-fast-ci-p1.log`, `p1-clippy.log`, `p1-gardes-vert.log`.

Le remède retenu au MEDIUM est le plus fort : une table `REFERENCES` tient, pour chaque fonction qui porte un site de raison finale (hors
`Definition`), la **liste close de ses références** jusqu'à la route ; deux tests neufs la comparent au code dans les deux sens ; une sonde
positive `sonde_i_…` ; angles morts 6 à 9 écrits.

## Ce que tu vérifies

1. **Le mécanisme de `REFERENCES`** : lis le code qui compte les références. Ce qu'il compte (chemin qualifié, appel nu dans le fichier de
   définition ou dans un fichier qui importe nommément `module::fonction`) — et ce qu'il manque, au-delà de l'angle mort 8 écrit :
   `use crate::repositories::x::{self, …}` puis `x::f(` ; `use super::*` ; un appel via `Self::` ou une méthode ; une macro ; un nom de
   fonction qui est le **suffixe** d'un autre (`delete` dans `delete_draft`, `update` dans `update_settings`) — la recherche est-elle
   bornée à un identifiant entier ? Un même nom de fonction défini dans deux modules (`update`, `create`, `delete`) : une référence de l'un
   est-elle comptée pour l'autre ? Cite au rapport chaque cas trouvé au code (`git grep -n`), ou dis que tu n'en trouves pas.
2. **La comparaison dans les deux sens** : une référence déclarée qui n'existe plus rougit-elle ? Une référence de plus rougit-elle (M7 au
   journal, message nommé) ? Une fonction de raison finale **sans** entrée `REFERENCES` rougit-elle
   (`chaque_site_de_raison_finale_a_sa_liste_de_references`) ? La raison elle-même (`Raison`) est-elle désormais lue, ou toujours ignorée
   par `_` — et, si oui, l'en-tête de la garde le dit-il ?
3. **Le contenu de la table** : échantillon d'au moins dix entrées de raison finale (dont `unlock_company_books`, `build_audit_entry`,
   `supplier_invoices::delete_draft`, `company_invoice_settings::update`, une entrée `PreAuth`, une `Mcp`) : la liste correspond-elle au
   code (`git grep -n '<fonction>('`), et **chaque route citée en bout de chaîne est-elle bien réservée à l'administrateur** dans `lib.rs`
   (ou pré-authentification / système), ce qui seul justifie la raison ? Le raccord 51 → 54 (`p1-references-releve.log`) se retrouve-t-il ?
4. **Les sondes et mutations** : `sonde_i_…` rougit-elle pour la bonne raison (sources synthétiques, assertion sur le message) ? M8
   (saut de la définition) tuée par la sonde (h) ? M2-garde : la garde verte sous `Self::user` est-elle bien écrite en angle mort 6 ?
   Propose une mutation plausible du mécanisme `REFERENCES` qui survivrait.
5. **`source_assainie.rs`** et **`audit_log.rs`** : le diff ne change-t-il que des commentaires (`git show b3a14f752 -- <f>`, lignes `-`
   et `+`) ? Si du code change, la garde et les autres utilisateurs du module commun gardent-ils leur comportement ?
6. **Le coût** : la garde passe de 0,3 s à environ 12 s (réassainissement de chaque fichier pour chacune des 50 cibles). Est-ce exact au
   journal ? Un coût de cet ordre dans `test-fast` et en CI (`cargo test -j1 --test-threads=1`) mérite-t-il un finding (mise en cache
   simple) — juge et dis pourquoi.
7. **Les textes** : manuel d'administration (`.tex` et **PDF aplati** : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`
   puis `/usr/bin/grep -o`) et CHANGELOG — la phrase sur « les entrées de ce retour en brouillon » et la liste des gestes encore en
   `'user'` sont-elles vraies au code (chaque geste cité lit bien `NewAuditLogEntry::user(` ou équivalent, ligne lue) ?
8. **Le compte rendu et les gates** au commit `b3a14f752` : test-fast 5409/0/13, clippy, gardes 15/15 et 11/11 au journal, postérieurs au
   commit ; `gate-ci` et E2E déclarés « à la clôture », non affirmés ; l'annotation en tête de `gate-ci.log` ne réécrit pas le relevé ;
   entrées antérieures du Change Log annotées sans réécriture (`git show e86709a34`, lignes `-`).

## Interdits absolus

Lecture seule : aucun fichier écrit hormis ton rapport (outil Write). **Aucune redirection, sous aucune forme** — ni `>`, ni `>>`, ni
`tee`, ni `2>`, ni `2>&1`, ni `2>/dev/null`, ni `> /dev/null` —, **aucune substitution de processus** (`<(…)`), aucun fichier temporaire.
Plusieurs relecteurs de cette epic ont écrit `2>/dev/null` « par réflexe » : relis chaque commande avant de la lancer. Aucun `rm`, `pkill`,
`kill`, aucun `git` qui écrit. Aucun interpréteur ni script (`python3`, `perl`, `node`, `npm`, `npx`, `bash -c`, `sh -c`, `jq`,
`scripts/*` dont **`scripts/prepare-release.sh`**, les scripts de `kesh-gate-logs`, `make`, `latexmk`, `cargo`, `docker`, `mariadb`,
`curl`, **`claude`**, `sed -i`, `vitest`, `playwright`), aucune commande `gh`, aucun accès réseau. Ne lis rien sous `~/kesh/` ni
`~/kesh-test/`. Permis : `git show/diff/log/grep/ls-files`, `/usr/bin/grep`, `sed -n`, `awk` sans redirection ni `system()`, `cat`, `ls`,
`find` sans `-exec`/`-delete`, `wc`, `sort`, `uniq`, `cmp`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`. ⚠️ `grep` est une fonction vers
`ugrep` : emploie `/usr/bin/grep` ou `git grep`. Toute affirmation de présence ou d'absence s'appuie sur une sortie copiée au rapport. Un
écart se déclare.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/e358a1-review-p2-ciblee.md` : findings numérotés (`C1`…), sévérité (CRITICAL/HIGH/MEDIUM/LOW), lieu,
ligne lue copiée, preuve, remède ; **axes exercés et non exercés** ; décompte final recompté depuis la liste.
