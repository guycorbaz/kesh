# Prompt — validation P4, Story 32-12 (une sauvegarde antérieure reste importable)

*Versionné le 2026-10-24. Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*) : **une seule lentille**, *regression
hunter* sur la spécification, en contexte frais, **Haiku**. Lecture seule stricte. Elle suit la P3 (Sonnet ; R 0/0/2/7,
F 0/0/0/9), remédiée au commit `7d7ade33`. Son périmètre est **le seul diff de cette remédiation** — non la fiche
entière.*

Dépôt `/home/gcorbaz/devel/kesh-32-12`, branche `story/32-12`. Fiche :
`_bmad-output/implementation-artifacts/32-12-import-sauvegarde-anterieure.md` ; registre :
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-3212.md` (C-32-12-30 à 36 neufs, C-32-12-23 et 29
annotés) ; `_bmad-output/implementation-artifacts/sprint-status.yaml` (clé `32-12-import-sauvegarde-anterieure`).
Rapports de la P3 : `/home/gcorbaz/devel/kesh-gate-logs/3212-validate-p3-R.md` et `…-p3-F.md`. Règles : `CLAUDE.md`
§ *Review Iteration Rule* (une boucle ne se clôt que sur une passe sans finding au-dessus de LOW), § *Haiku-specific
guardrails*, § *Propagation post-patch*, § *Recompter ses propres comptes rendus* (et « Recompter n'est pas réécrire
l'historique »), § *Règle de splitting préventif* et son amendement D5.

## Le diff à relire — aplati, un seul

```sh
git diff 8a5a720a 7d7ade33 -- _bmad-output/implementation-artifacts/
```

Un seul diff, du commit d'avant la remédiation à celui qui la porte : **ne lis pas les commits intermédiaires**, il n'y
en a pas, et les numéros de ligne d'un hunk sont ceux du fichier **final** (`git show 7d7ade33:<fichier>` pour le
relire). Pour toute affirmation « X manque » ou « Y est encore là », vérifie par `grep -nF "<chaîne>" <fichier>` avant
de l'écrire (§ *Haiku-specific guardrails*).

## Ce que tu cherches : ce que la remédiation P3 a cassé

La sévérité se déplace vers ce qu'on vient d'écrire. Pour **chaque hunk** du diff, demande : ce texte neuf est-il
**vrai** (au code, aux rapports P3), **cohérent** avec le reste de la fiche, et la valeur qu'il change a-t-elle été
changée **partout** — ni oubliée ailleurs, ni réécrite dans une entrée datée (Change Log P1/P2, registre 1-29 hors
annotations) ?

### Axe 1 — exécuter les commandes de l'AC17 telles qu'écrites (obligatoire)

Recopie **à l'identique**, depuis la fiche, la définition de `S` et les six commandes (G1 à G5 et la valeur), puis
exécute-les depuis la racine du worktree, chacune suivie de `| wc -l`, **sortie vers la sortie standard seulement** :

```sh
cd /home/gcorbaz/devel/kesh-32-12
S="<la portée recopiée de l'AC17, sur une ligne>"
<commande G1 recopiée> | wc -l
…
```

Compare chaque nombre au nombre écrit à l'AC17 (**51 / 7 / 36 / 10 / 18 / 5**), au Change Log « Validation P3 » et à
C-32-12-30 du registre : un écart, une commande qui ne se recopie pas telle quelle, une option qui manque d'une famille
à l'autre est un finding. Puis liste les lignes rendues (sans `wc`) et confronte-les au tableau *Inventaire de la
classe* et à la liste des **14 homonymes nommés** des Dev Notes : une ligne ni au tableau ni parmi les homonymes, un
homonyme qui est en réalité un membre de la classe, ou un décompte d'homonymes qui ne se recompte pas (14) — finding.
Vérifie aussi que G1 ne rend bien **aucun** homonyme, comme la fiche l'affirme.

### Axe 2 — les numéros de ligne et sites neufs

Chaque numéro de ligne introduit ou changé par le diff (`post_restore.rs` 257-279, 272-274, 276-279, 930, 1084,
1095-1096, 433, 436-437, 473-474, 1307, 1355 ; `admin_full_import_e2e.rs` 1091, 1174, 1221, 1314, 1386, 1500 ;
`scripts/prepare-release.sh:128` ; et tout autre du diff) : ouvre le site (`sed -n '<a>,<b>p' <fichier>`) et vérifie
qu'il dit ce que la fiche lui fait dire. `crates/kesh-db/src/post_restore/` existe-t-il, et porte-t-il des extraits ?

### Axe 3 — cohérence des AC remaniés

- AC1 : `created_application_table -> Option<&'static str>`, `table_creators()`, `table_creating_migrations()` (minimum),
  test (i) sur `table_creators()` — le reste de la fiche (Point 1, T2, tableau des sites, M8, registre C-32-12-23 et
  31) dit-il la même chose ? Un `creates_application_table` qui subsisterait comme **nom à venir** (non comme nom
  actuel) est un finding.
- AC2 / M3 : « (iii) impliquée par (iv) » — vrai ? contredit-il une autre phrase de l'AC2 (« (iii) rougit ») ou du
  *Contrat* point 1 ?
- AC3 : ordre défini, table présente sans créatrice, `Inconnue(String)` / `Exigee` / `HorsOrdre` en `&'static str` —
  cohérent avec l'AC5 (messages), l'AC7 et la signature `Result<Vec<&'static str>, InventoryError>` ?
- AC7 (a) : « après toute autre table » — compatible avec la carte de production et la règle du suffixe ?
- AC11 / M7 / *Contrat* point 2 : le prédicat étendu est-il décidable avec ce que la fiche fournit (cible extraite,
  `table_creating_migrations()`) ? L'exigence d'un extrait est-elle exacte au regard de `registry_sql_contains_no_ddl`
  et d'`extract_carries_every_write_statement_of_its_source_migration` ?
- AC20 (ii) / *Contrat* point 3 / AC21 : l'exemple MCP a-t-il disparu de **tout** ce que le manuel devra porter, sans
  rien promettre de la 32-5 ?
- *Dérogation règle de splitting* : les sévérités citées (P1, P2, P3, lentille R puis F) se retrouvent-elles dans le
  Change Log et les rapports ? La coupe de repli (32-12a, 32-12b) couvre-t-elle **tous** les AC, sans doublon ? La
  décision est-elle bien laissée à Guy ?

### Axe 4 — comptes rendus

Change Log « Validation P3 » : décomptes (R 0/0/2/7, F 0/0/0/9), convergences, « dix entrées » (recompte), tendance des
MEDIUM distincts (3 → 3 → 2) ; en-tête de la fiche (C-32-12-1 à 36) ; registre (sept choix neufs, numérotés sans trou) ;
clé du `sprint-status.yaml`. Les entrées P1 et P2 du Change Log sont-elles **intactes** (`git diff 8a5a720a 7d7ade33`
ne doit y montrer aucune ligne retirée) ?

## Ce que tu rends

Findings `P4-1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site, **preuve (commande et sortie
copiée)**, correction proposée. ⛔ **La liste des axes exercés ET non exercés** (axes 1 à 4) — sans elle, « 0 finding »
ne compte pas ; pour l'axe 1, les **six nombres que tu as obtenus**, commande par commande. Bilan par sévérité
**recompté** sur la liste. Rapport écrit **d'un seul jet**, avec l'outil d'écriture de fichier (non par une redirection
du shell), dans **un seul fichier** : `/home/gcorbaz/devel/kesh-gate-logs/3212-validate-p4.md`. Dernier message : chemin,
bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ **Aucune redirection d'écriture** (`>`, `>>`, `tee`, `2>` vers un fichier) — **pas même vers `/tmp` ou un répertoire
d'échange** : les deux lentilles de la P3 ont chacune laissé un fichier temporaire, c'est précisément ce qui est
interdit. **Aucun fichier temporaire**, aucun fichier écrit hors de ton rapport ; pour comparer deux sorties, utilise
`diff <(…) <(…)`. Aucun `git checkout`/`switch` sous aucune forme ; aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont **`scripts/prepare-release.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `npm`, `npx`, `docker`,
`curl`, **`python3`, `python`, `perl`, `ruby`, `node`**, `mariadb`/`mysql`, `unzip` vers le disque (`unzip -l` et
`unzip -p` vers la sortie standard seulement), `sed -i`, `git commit|add|stash|fetch|pull|reset|rebase|tag|push`, `gh`
en écriture. `pdftotext` vers la sortie standard seulement. Aucune donnée réelle de Guy (rien de `~/kesh-test/`).
Autorisés : lecture, `grep`, `sed -n`, `awk` en lecture, `wc`, `diff <(…) <(…)`, `git log|show|diff|blame|ls-tree|tag
--contains`, `gh issue view`, `pdftotext … -`, `unzip -l|-p`.
