# Prompt — validation P4, Story 32-3 (la couche de services : le motif, sur deux modules pilotes)

*Versionné le 2026-10-24. Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*) : **une seule lentille, le regression
hunter**, en contexte frais, **Haiku**, braquée sur **le seul diff de la remédiation P3**. Lecture seule stricte.*

Dépôt `/home/gcorbaz/devel/kesh-32-3`, branche `story/32-3`. **Le diff à relire, aplati, et lui seul** :

```sh
git show 1269b07a -- _bmad-output/
```

(commit `docs(32-3): validation P3 remédiée — retrait de la preuve lexicale d'inaccessibilité du pool, …`, un seul
commit : il n'y a pas de diff multi-commit à recomposer). Fiche : `_bmad-output/implementation-artifacts/
32-3-services-motif.md` ; registre : `epic-32-choix-autonomes-323.md` (C-32-3-31 est le choix de cette remédiation) ;
rapports de la P3 qui l'ont motivée : `/home/gcorbaz/devel/kesh-gate-logs/323-validate-p3-R.md` et `-F.md`.

## Ce que la remédiation P3 a fait

Sur décision de l'orchestrateur, la fiche **cesse de promettre une preuve lexicale** que le pool d'une poignée
(`Lecture`, `Trace`) est inaccessible dans `services/` — troisième forme recyclée de ce MEDIUM. Elle garde : la
visibilité Rust (champs privés, `pool()` en `pub(in crate::services)`) ; deux gardes simples à l'AC10 — **(v-1)** aucun
`.pool()` hors de `services/`, **(v-2)** `\bLecture::new\(`/`\bTrace::new\(` au seul câblage de `lib.rs` — avec (i),
(ii), (vi), (vii) ; l'exécution (AC9 (b) sur la `Lecture` en lecture seule, M1, M1-témoin) ; un paragraphe *Ce que les
gardes ne voient pas* qui renvoie la garantie à la 32-6 et à la revue. Elle traite treize LOW (voir le Change Log
« Validation P3 »).

## Ta tâche — chercher la régression que ce patch a introduite

*La sévérité se déplace vers ce qu'on vient d'écrire* : cherche dans **ce diff** ce qu'il casse, contredit ou affirme
à tort. **Chaque affirmation nouvelle sur le code se vérifie au code** — ouvre le site (`sed -n`, `grep -nF`) et
**copie la sortie**. En particulier :

1. **Les affirmations nouvelles sur le code**, une à une :
   - `grep -rnE "\.pool\(\)" crates/kesh-api/src` rend-il bien **rien** aujourd'hui (baseline de (v-1)) ?
     `self.pool.clone()` et `state.pool` échappent-ils au motif ?
   - **E0742** : `pub(in crate::services::rapports)` est-il réellement refusé sur un élément de
     `crate::services::poignees` (règle : le chemin d'un `pub(in …)` doit être un module **ancêtre**) ? Raisonne sur la
     référence Rust ; ne compile pas.
   - `reports.rs:1343-1367` (doc-comment de conventions) et `1368-1371` (doc-comment propre d'`emit_ledger_audit`),
     `emit_report_audit` l. 1407 sans doc-comment ?
   - les six validations avant `span` (l. 330, 462, 660, 970, 1034, 1144 ; spans l. 332, 464, 662, 972, 1036, 1146) et
     les trois exports sans `span` ?
   - `test_fixtures.rs:118` : l'exercice seedé va-t-il de `2020-01-01` à `2030-12-31` ?
   - `config.rs` l. 1691, 1791, 1839 : `unsafe` dans des tests ? `forbid(unsafe` absent de `lib.rs` ?
   - `grep -n '"[^"]*//' crates/kesh-api/src/routes/reports.rs` → rien ?
   - `docs/MULTI-TENANT-SCOPING-PATTERNS.md` l. 152, 160, 244-273 et `_bmad-output/planning-artifacts/architecture.md`
     l. 471-490 disent-ils ce que l'AC14 leur prête ?
   - `gh issue view 431` : #431 ne couvre-t-elle bien que les dépôts de `kesh-db` ?
2. **Les mutations touchées** : M11 refondue (`Lecture::pool()` en `pub(crate)` **et** `state.lecture().pool()` dans
   `get_balance_sheet`) compile-t-elle et rougit-elle (v-1), et seulement elle (pas (i)) ? M1 et M1′ rougissent-elles
   (v-2) — le motif `\bTrace::new\(` voit-il `&Trace::new(self.lecture_m1.pool().clone())` ? M10 sur les deux
   fonctions : la répartition des lignes 1-3-5-7-9 / 2-4-6-8-10 est-elle juste contre la table de l'AC9 (b) ? M3/M3′ et
   l'auxiliaire commun : la phrase est-elle exprimable ?
3. **(vii) hors commentaires** : la règle « présence hors commentaires, absence sur les lignes de code » est-elle
   décidable et sans faux vert ? Un `//` dans une chaîne peut-il produire un faux **vert** dans l'un des deux sens ?
4. **(b′) et la ligne sans bornes** : la phrase nouvelle contredit-elle « La table est une constante unique » ou la
   règle d'`asof` ? (b) : « jamais un `Lecture::new(pool)` sur le pool ordinaire » est-il compatible avec (v-2), qui
   ne regarde que `src/` ?
5. **T2 en deux commits** : le premier commit (montage HTTP, table, (b′), sans référence à `kesh_api::services`) est-il
   constructible et jouable sur le code d'avant T3 ?
6. **Cohérence du retrait** : reste-t-il, **hors des relevés datés** (Change Log P1/P2, corps des choix C-32-3-17 à 30
   sous leurs annotations), une phrase qui promette encore une preuve lexicale (« inventaire clos » des poignées,
   « tout chemin lexical », « attrape tout », `(v-a)`…`(v-e)`, `M11′`, `M11″`, « n'étant appelé que par ») ? Greper la
   **valeur**. L'esquisse, R4, R8, AC3, AC6, *Le motif*, *Hors périmètre* et AC10 disent-ils la même chose ?
7. **Décomptes** du Change Log « Validation P3 » : « R 0/0/1/8, F 0/0/0/10 » (`grep "^### [RF][0-9]"` sur les deux
   rapports), « quatorze distincts (19 − 5) », « un MEDIUM, treize LOW », « P1 6 → P2 5 → P3 1 » : recompte à la source.
8. **Manuel** : le diff touche-t-il une affirmation sur le comportement visible ? (Attendu : non, AC14.) Si oui,
   vérifie le PDF aplati (`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`).

Hors de ta lentille : relire la fiche entière, rejouer les axes R et F des passes P1 à P3. Un défaut d'origine que tu
croiserais se signale, en le marquant « hors diff ».

## Ce que tu rends

Findings `P4-1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt, **preuve (commande et
sortie copiée)**, correction proposée ; pour chacun, dis s'il **naît de la remédiation P3**. ⛔ Tout finding CRITICAL
ou HIGH affirmant qu'un texte « manque » ou « n'a pas été changé » se vérifie par `grep -nF` sur le fichier **final**
avant d'être écrit (garde-fou Haiku de `CLAUDE.md`). Un bilan d'en-tête **cohérent avec le corps**, recompté. ⛔ **La
liste des axes exercés ET non exercés** (points 1 à 8 ci-dessus) — sans elle, « 0 finding » ne compte pas. Ce que la
fiche **prescrit** au développement n'existe pas encore : son absence n'est pas un défaut. Rapport écrit **d'un seul
jet**, un seul fichier : `/home/gcorbaz/devel/kesh-gate-logs/323-validate-p4.md`. Dernier message : chemin, bilan par
sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, **aucun fichier temporaire**, rien dans le scratchpad ; **aucune
redirection** (`>`, `>>`, `tee`), pas même vers un fichier jetable — un fichier créé par erreur se déclare au rapport,
il ne se supprime pas. Aucun `git checkout`/`switch` sous aucune forme ; aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `rustfmt`, `npm`,
`npx`, `docker`, `curl`, **`python3`, `python`, `perl`, `ruby`, `node`**, `mariadb`/`mysql`, `sed -i`,
`git commit|add|stash|fetch|pull|reset|rebase|tag|push`, `gh` en écriture. `pdftotext` vers la sortie standard
seulement. Aucune donnée réelle de Guy (rien de `~/kesh-test/`). Le dépôt principal `/home/gcorbaz/devel/kesh` est en
lecture seule. Autorisés : lecture, `grep`, `sed -n`, `git log|show|diff|blame|merge-base`, `gh issue view|list`,
`pdftotext … -`, et les filtres en lecture seule `tr`, `sort`, `cut`, `head`, `awk`, `wc`, `uniq` vers la sortie
standard.
