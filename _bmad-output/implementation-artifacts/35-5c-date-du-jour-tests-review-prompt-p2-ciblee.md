# Prompt — revue de code P2 ciblée de la Story 35-5c (#929), une lentille, braquée sur la remédiation de la P1

Passe **ciblée**, une seule lentille, contexte frais, **Sonnet** (P1 : Opus, trois lentilles), en lecture seule. Worktree
`/home/gcorbaz/devel/kesh-e355c`, branche `story/35-5c-date-du-jour-tests`. La P1 a rendu 2 MEDIUM (A1 = B1 = E1, B2) et une
douzaine de LOW (rapports `/home/gcorbaz/devel/kesh-gate-logs/e355c-review-p1-{A,B,E}.md`). Déjà tracés, à ne pas rapporter de
nouveau : #1125 (tests à dates fixes de 2026, dont les deux rouges d'`admin_full_import_e2e`), #1111 et #1112 (intermittents des
deux voies), #1126 (test MCP intermittent). L'E2E et `gate-ci` se jouent à la clôture (consignes de l'Epic 35) : leur absence
n'est pas un défaut.

## Périmètre : le seul commit `7fcede7e8`

`git -C /home/gcorbaz/devel/kesh-e355c show 7fcede7e8` (11 fichiers). Compte rendu : le commit qui le suit, `git log --oneline
7fcede7e8..HEAD` (Change Log « Remédiation P1 », annotations du Dev Agent Record, registre C-e355c-4, -5, -6 annotés, C-e355c-8 et
-9 neufs). Journaux sous `/home/gcorbaz/devel/kesh-gate-logs/e355c/` : `p1-rouge-garde-rust.log`, `p1-rouge-dates.log`,
`p1-rouge-garde-front.log`, `p1-cible-verts.log`, `p1-verts-front-cible.log`, `p1-mut-MP1.log` à `p1-mut-MP6.log`,
`p1-test-fast.log`, `p1-front-{check,lint-i18n-ownership,test-unit,build}.log`.

Il remédie :
- **A1 = B1 = E1 (MEDIUM)** — `docs/testing.md:119` : la suite suit `KESH_TEST_INSTANT` depuis la 35-5c ; deux exceptions
  nommées (#1125 ; le binaire `lib` de `kesh-db`, module de tests de `journal_entries.rs`) ; la règle des specs.
- **B2 (MEDIUM) = E6** — garde « horloge réelle d'un test » dans `crates/kesh-api/tests/date_du_jour_garde.rs` (`HORLOGE_REELLE`,
  `HORLOGE_REELLE_ADMISES`, `horloge_reelle_d_un_test`, `sonde_l_horloge_reelle_d_un_test_rougit`) : liste close de **deux**
  entrées (`journal_entries.rs` 1, `middleware/auth.rs` 1 — l'orchestrateur en attendait une, C-e355c-8).
- **A5 = B4 = E3** — `ajouterJoursIso` (`frontend/tests/e2e/helpers/dates.ts`) refuse année hors de 1000 à 9999, jour impossible,
  `n` non entier ; trois tests (`dates.test.ts`).
- **E4, E5 = B3, B5** — `local-date-guard.test.ts` : appariement exact pour les specs (`auditer(…, exact)`), quatre motifs de plus,
  angles morts écrits.
- **E2** — `accounts_e2e.rs` : le jour lu une fois par test, passé à `mk_fiscal_year` et `mk_entry`.
- **B6** — alias `todayZurich()` / `today()` retirés de cinq specs.
- **A2, A3 = E7, A4** — comptes rendus seulement.

## Ce que tu vérifies

1. **`docs/testing.md:119`** : chaque phrase neuve est-elle vraie au code ? Lis `crates/kesh-api/tests/common/jour.rs`,
   `crates/kesh-db/tests/support/jour.rs`, `crates/kesh-report/tests/support/jour.rs`, `journal_entries.rs:3246`,
   `admin_full_import_e2e.rs:2007-2011` et `:2281-2285` (bornes `2026-03-31`, `2026-06-30`), `helpers/api-fixtures.ts:390`. « La
   garde n'admet plus, hors de ses deux sites permanents, aucune forme » : exact pour l'inventaire (`EXEMPTIONS`) ? La phrase entre
   parenthèses de fin (MariaDB, module de tests au jour réel) est-elle juste ? Une autre phrase de `docs/`, de `CLAUDE.md`, d'un
   commentaire de `crates/` ou de `frontend/tests/` annonce-t-elle encore la 35-5c au futur ou des « montages non convertis »
   (`git grep -nE "non convertis|pas convertis|ne la suit pas|35-5c" -- docs CLAUDE.md crates frontend/tests frontend/src`) ?
2. **La garde Rust neuve** : le motif `HORLOGE_REELLE` attrape-t-il les formes annoncées (qualifiée `chrono::Utc::now()`,
   `::chrono::Utc::now()`, coupée sur plusieurs lignes) et rien de légitime (`HorlogeFixe(instant_de_test(…).unwrap_or_else(Utc::now))`
   de `kesh-db/tests/support/jour.rs:18-21`, `jour_suisse(self.maintenant())` de `kesh-core/src/horloge.rs:32`) ? `\s` du crate
   `regex` couvre-t-il `\n` ? Le compte sur `sans_commentaires(t)` (texte entier, non par ligne) et `ecarts` à égalité stricte : une
   entrée de la liste qui tombe à 0 rougit-elle ? `tous_les_ecarts` inclut-il la garde, si bien que les sondes existantes
   (`vert(…)`) la voient ? Les deux entrées admises : lis `journal_entries.rs:3240-3256` et `middleware/auth.rs:296-303` ; la
   justification écrite est-elle la ligne lue ? L'angle mort de l'en-tête est-il complet pour les formes que tu trouves ?
   La portée (tout `src` et `tests`, non les seuls `tests/` ni les seuls `mod tests`) est-elle cohérente avec C-e355c-8 ?
3. **`ajouterJoursIso`** : pour chaque test neuf, l'attendu est-il celui du code (calcule à la main : `'1000-01-01'` − 1,
   `'9999-12-31'` + 1, `'2025-02-29'`, `'2026-04-00'`, `0.5`, `Infinity`) ? Le contrôle aller-retour peut-il refuser un jour réel
   (année 1000 à 1899, bissextile séculaire `1900-02-29` / `2000-02-29`) ? `formater` sans `padStart` sur l'année : sûr pour 1000
   à 9999 ? Les specs existantes qui appellent `ajouterJoursIso` (`git grep -n ajouterJoursIso -- frontend/tests/e2e`) passent-
   elles toujours des entrées admises (aucune ne rougirait à l'E2E de la clôture) ?
4. **La garde des specs** : avec `exact`, les trois entrées d'`ALLOWLIST_SPECS` égalent-elles toujours leur ligne
   **sans indentation** (`git grep -nF` sur chacune) ? Les quatre motifs neufs relèvent-ils une ligne actuelle de `tests/e2e`
   (l'audit vert de `p1-front-test-unit.log` le dit ; recoupe par `git grep -nP` sur chaque motif) ? Faux positifs plausibles :
   `(?<![\w$])Date\s*\(\s*\)` contre un appel de méthode nommée `Date` précédé de `.` (`x.Date()`), un type générique ; `\.format()`
   contre une autre API `format()` sans argument. La moitié `src` de la garde garde-t-elle son appariement par extrait
   (`auditer(relever(sources), ALLOWLIST)`) ?
5. **`accounts_e2e.rs`** : une seule lecture par test (`git grep -n aujourd_hui_test -- crates/kesh-api/tests/accounts_e2e.rs`) ;
   `mk_fiscal_year`, `mk_entry` et l'assertion `Exercice {}` prennent-ils le même `jour` ? Plus aucun `current_year()` ?
6. **B6** : aucune référence restante à `todayZurich` ou à un `today()` local (`git grep -nE 'todayZurich|\btoday\(\)' --
   frontend/tests/e2e`) ; l'import de `todayInZurich` présent dans chaque fichier touché ; aucune ligne de spec autre que les
   alias et leurs appels modifiée (`git show 7fcede7e8 -- frontend/tests/e2e/*.spec.ts`).
7. **Rouges et mutations** : chaque rouge (`p1-rouge-*.log`) l'est-il pour la bonne raison (la garde à liste vide nomme les deux
   sites réels ; les trois tests de dates rouges sur le `dates.ts` de `40090cdd3` ; les deux sondes frontend rouges sans
   l'extension) ? Chaque mutation (`p1-mut-MP*.log`) a-t-elle été appliquée sur l'arbre commité `7fcede7e8` (en-tête, `git status`
   avec le seul fichier muté), tuée par le test nommé, fichier restauré (`restauré : 0 fichier(s) modifié(s)`) ? MP1 : un
   `jour_suisse(chrono::Utc::now())` dans `credit_card_e2e.rs:1063` ; MP2 : le motif cassé ; MP3 à MP5 : `dates.ts` ; MP6 :
   l'appariement exact retiré.
8. **Le compte rendu et les gates** : `p1-test-fast.log` au commit de code, bases remises à neuf, chiffre exact ; frontend
   (`p1-front-*.log`) au même commit ; `gate-ci` et E2E déclarés « à la clôture », non affirmés ; entrées antérieures du Change
   Log annotées **sans réécriture** (`git show` du commit de compte rendu, lignes `-`) ; registre C-e355c-8 et -9 justes au code ;
   aucun mot-clé de fermeture dans les messages (`git log ebd5b654c..HEAD --format=%B | /usr/bin/grep -niE
   '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`).
9. **Attendus retouchés** : `git show 7fcede7e8`, lignes `-` dans les tests — un attendu existant affaibli ou retiré sans motif
   (hors les alias de B6) ?

## Interdits absolus

Lecture seule : aucun fichier écrit hormis ton rapport (outil Write). **Aucune redirection, sous aucune forme** — ni `>`, ni `>>`, ni
`tee`, ni `2>`, ni `2>&1`, ni `2>/dev/null`, ni `> /dev/null` —, **aucune substitution de processus** (`<(…)`), aucun fichier
temporaire. Des relecteurs de cette epic ont écrit `2>/dev/null` ou `2>&1` « par réflexe » : relis chaque commande avant de la lancer.
Aucun `rm`, `pkill`, `kill`, aucun `git` qui écrit. Aucun interpréteur ni script (`python3`, `perl`, `node`, `npm`, `npx`, `bash -c`,
`sh -c`, `jq`, `scripts/*` dont **`scripts/prepare-release.sh`**, les scripts de `kesh-gate-logs`, `make`, `latexmk`, `cargo`,
`docker`, `mariadb`, `curl`, **`claude`**, `sed -i`, `vitest`, `playwright`), aucune commande `gh`, aucun accès réseau. Ne lis rien
sous `~/kesh/` ni `~/kesh-test/`. Permis : `git show/diff/log/grep/ls-files`, `/usr/bin/grep`, `sed -n`, `awk` sans redirection ni
`system()`, `cat`, `ls`, `find` sans `-exec`/`-delete`, `wc`, `sort`, `uniq`, `cmp`. ⚠️ `grep` est une fonction vers `ugrep` :
emploie `/usr/bin/grep` ou `git grep`. Toute affirmation de présence ou d'absence s'appuie sur une sortie copiée au rapport. Un écart
se déclare.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/e355c-review-p2-ciblee.md` : findings numérotés (`C1`…), sévérité (CRITICAL/HIGH/MEDIUM/LOW),
lieu, ligne lue copiée, preuve, remède ; **axes exercés et non exercés** ; décompte final recompté depuis la liste.
