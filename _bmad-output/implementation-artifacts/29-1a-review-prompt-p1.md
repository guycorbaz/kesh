# Prompt — revue de code P1, Story 29-1a (préparer la clôture : contrôles préalables, lecture seule)

*Versionné le 2026-10-14. Trois lentilles **Sonnet** (B, E, A), contexte frais chacune, lecture seule.*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-29-1`, branche `story/29-1`.
**Le diff à revoir** : `git diff fcd2ae75 7c643eba -- . ':(exclude)_bmad-output'` — `fcd2ae75` = `origin/main`
(kickoff de l'Epic 29) ; `7c643eba` = **dernier commit de code** (les commits qui le suivent ne portent que la fiche,
le registre, `sprint-status.yaml` et ce prompt). Le PDF du manuel se contrôle à `7c643eba` (régénéré à `bf5f8238`).
⚠️ Pathspec `':(exclude)…'` et **non** `':!…'`. Fiche : `_bmad-output/implementation-artifacts/29-1-preparer-cloture.md`
— **partie a seulement** : AC1 à AC9 et AC3-bis ; tâches Ta1-Ta8 ; Dev Notes § *Concurrence* (puce 29-1a), § *État
actuel des fichiers modifiés*, § *Tests* (T-a1 à T-a16) et § *Mutations* (M-a1 à M-a22) ; et le **Dev Agent Record**,
section *29-1a* (tests rouges, gates, mutations, décomptes, captures, écarts). Choix C-29-1-32 à C-29-1-42 au
registre `epic-29-choix-autonomes-29-1.md` (nés du développement ; C-29-1-1 à 31 sont ceux de la fiche). Règles :
`CLAUDE.md` du dépôt — § « Recompter ses propres comptes rendus », § « Propagation post-patch », § « Le prompt d'une
passe doit NOMMER le manuel », § « Une passe qui ne déclare pas ses axes exercés ne compte pas », § « Inventorier les
sites NON RÉSOLUS », § *Migration breaking policy* (aucune migration attendue : à vérifier). Issue : #232 — la PR de
la 29-1a **ne la ferme pas** (`refs #232` partout ; aucun `closes` / `fixes`, même en prose, dans un message de
commit : à greper, `git log fcd2ae75..HEAD --format=%B | grep -niE "close[sd]?|fix(e[sd])?" `).

⛔ **DONNÉES PERSONNELLES — DÉPÔT PUBLIC.** N'ouvre aucun fichier hors du dépôt et de
`/home/gcorbaz/devel/kesh-gate-logs/`. Les captures `kesh-gate-logs/29-1a-captures/*.png` sont synthétiques (IBAN de
test) : tu peux les ouvrir.

## Axes communs aux trois lentilles (chacune les déclare exercés ou non)

1. **Les sept contrôles contre l'AC1** (`crates/kesh-db/src/repositories/closing_checks.rs`). Chaque requête de
   `ALL_SQL` : scope `company_id` (y compris dans les sous-requêtes et les `EXISTS`), bornes `BETWEEN` inclusives,
   population de K3 (archivés, clos avant Y, trois `EXISTS` d'activité **séparés**, quatrième `EXISTS` sans borne), la
   table de précédence de K3 telle que codée en Rust (motifs 1 à 5, borne `MIN(Y.end_date, closed_on)`), K6 (une
   écriture comptée une fois, deux motifs possibles, liste bornée à 20, ordre `entry_date, id` ; la table dérivée des
   totaux est-elle juste pour une écriture **sans aucune ligne** ?), K7 et K1 par `find_next_after` /
   `find_earliest_earlier_open`. Le `count` de K1 et K7 (C-29-1-33). La forme JSON (camelCase, montants en chaînes,
   `CheckDetails` `untagged` : deux variantes de même forme pourraient-elles se confondre à la lecture côté client ?).
2. ⛔ **B5 — verrous et naissance de la vue : AUCUNE lecture ne verrouille, l'instantané est UNIQUE.** La route
   (`routes/fiscal_years.rs::get_closing_checks`) ouvre **un** `ReadSnapshot`, y fait **toutes** les lectures —
   l'exercice Y compris (`FISCAL_YEAR_SQL`), aucune lecture préalable sur le pool —, puis `finish()`. Vérifie : aucune
   lecture de `closing_checks` ni des fonctions **externes** qu'il appelle (`bank_accounts::balances_by_bank_account`
   et son `BALANCES_SQL`, `balances_from_row`, `is_shared`, `statement_gap`, `fiscal_years::find_earliest_earlier_open`
   et `EARLIER_OPEN_SQL`, `find_next_after` et `NEXT_AFTER_SQL`) ne porte `FOR UPDATE`, `LOCK IN SHARE MODE` ni `FOR
   SHARE`, et chacune s'exécute sur l'**exécuteur reçu** (jamais le pool) — c'est l'**angle mort assumé** de T-a2
   (AC2), vérifié ici en revue, `grep -nF` à l'appui. Sur le chemin d'erreur, l'instantané est-il bien annulé ? Et
   `fiscal_years::close` : le texte exécuté à l'étape (d) est-il **octet pour octet** l'ancien (`earlier_open_locking_sql()`,
   T-a15) ; aucun verrou ni aucune lecture ajouté ou déplacé dans `close` ?
3. **L'extraction de la requête du tableau de bord (AC3).** `BALANCES_SQL` : la borne est-elle **dans** `st`, avant
   le `ROW_NUMBER()` ; l'ordre des sept liaisons ; `list_by_company_with_balances` rend-elle exactement ce qu'elle
   rendait (compare au `git show fcd2ae75:crates/kesh-db/src/repositories/bank_accounts.rs`) ; le `match` déplacé, non
   recopié ; `statement_gap` (arrondi des **deux** termes, `MidpointAwayFromZero`, `None` si nul) contre l'ancien
   `homepage.ts` (`git show fcd2ae75:frontend/src/lib/features/homepage/homepage.ts`) ; `statementGap` dans
   `BankAccountWithBalance` et sa lecture par `bank-accounts.api.ts` / `homepage.ts` (`toCentime` supprimé, carte
   inchangée). Les tests existants cités par l'AC3 sont-ils **inchangés** au diff ?
4. **La garde statique T-a2 (AC2, C-29-1-26, C-29-1-35).** Ses deux volets tels que codés (`closing_checks.rs`,
   module `tests`) : un site non résolu peut-il lui échapper (une requête construite hors `query*`, un `Executor::
   execute(&str)`, `fetch_*` sur une chaîne, un alias `use sqlx::query as q`) ? Le cas de contrôle voit-il la forme
   dominante ? L'inventaire `nb_constantes_sql == ALL_SQL.len()` peut-il passer à vide ? Le décommentage (C-29-1-35)
   affaiblit-il la garde ?
5. **Le panneau et le rappel (AC5, AC6, AC7).** `FiscalYearChecks.svelte`, `closing-checks-labels.ts`, la page
   `settings/fiscal-years/+page.svelte` : colonne d'actions pour tous les rôles, gestes existants sous leurs
   conditions **inchangées**, `colspan` 5 ; trois états jamais confondus (attente, indisponible, réponse — M-a10,
   M-a22) dans le panneau **et** le rappel ; bouton *Clôturer* actif pendant le chargement et après un échec ;
   ce que compte « N point(s) » (`info` compris, `notCheckable` exclu) ; les liens (K2 à K7), leur rôle ; une réponse
   tardive pour un autre exercice (jeton de requête, C-29-1-37). i18n : 53 clés × 4 locales (`grep -c
   '^fiscal-years-checks-' crates/kesh-i18n/locales/*/messages.ftl`), traduites réellement, replis égaux à fr-CH,
   `lint-i18n-ownership`, et les bornes recomptées de `i18n-keys.test.ts` (2107 → 2162) et
   `i18n-libelle-en-dur.test.ts` (63 → 68, 49 → 54).
6. **Les tests prouvent-ils ce qu'ils nomment.** T-a1 à T-a16 contre la fiche (`crates/kesh-db/tests/closing_checks.rs`,
   `crates/kesh-api/tests/closing_checks_e2e.rs`, `bank_accounts_e2e.rs`, `bank_accounts.rs`, `fiscal_years.rs`,
   Vitest, E2E `fiscal-years.spec.ts`) ; la campagne M-a1 à M-a22 (journal
   `kesh-gate-logs/291a-mutations.log`, script cité au Dev Agent Record) — cherche la **mutation plausible qui
   resterait verte** (exemples : K2 sans le filtre `status = 'pending'` ; K3 sans `bi.company_id = ba.company_id` ;
   `closed_on >= ?` en `>` ; K6 `too_few_lines` borné aux écritures **de Y** retiré ; le `count` de K2 pris sur le
   nombre de comptes ; la liste K4 sans `ORDER BY` ; le rappel qui compterait les `notCheckable`). Les fixtures des
   exclusions passent-elles à vide (assertions de montage) ?
7. ⛔ **LA DOCUMENTATION — MANUEL UTILISATEUR (PDF compris), `docs/api-external.md`, CHANGELOG.** Chaque phrase
   neuve ou modifiée dit-elle vrai **au code** ? Au manuel utilisateur (`docs/manual/fr/user-manual.tex`) : § *Préparer
   la clôture* (`\label{sec:preparer-cloture}`) — les sept contrôles, « ne bloque rien », les liens, les **limites**
   (K5 non borné, CSV / sans lien / lien partagé « non vérifiable », population de K3, relevé « au plus tard le dernier
   jour », écart sous le centime, K6 hérité seulement, lecture à un instant) ; la phrase corrigée « que celui-ci soit
   clôturé ou non » (ex-« si clôturé ») ; la marche de clôture (rappel du dialogue) ; le résumé annuel. Cherche une
   phrase des manuels, du README ou de `website/` que la story rend **fausse** et n'a pas touchée (`grep -n "clôtur"
   docs/manual/fr/user-manual.tex`, `grep -n "écart" docs/manual/fr/user-manual.tex`). Contrôle le **PDF aplati** :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` vers
   `/home/gcorbaz/devel/kesh-gate-logs/29-1a-review-p1-<lettre>-user-manual-flat.txt` — ⚠️ `pdftotext` rend « ﬀ »
   (ligature) et « ’ » (apostrophe typographique) : un `grep` sur « l'exercice » rend un faux négatif. Le PDF est-il
   celui du HEAD (régénéré après la dernière modification du `.tex`) ? `docs/api-external.md` : la ligne
   *Exercices comptables* du tableau (réouverture fermée aux clés : vrai ?) et la note ¹⁰ (forme, ordre, codes, `count`,
   remarque sur K5, `statementGap`). CHANGELOG : **une seule** section `## [0.14.0] — Non publié` (forme attendue par
   `scripts/prepare-release.sh`, sans l'exécuter), entrée de #232 sans annoncer la 29-1b.
8. **Gardes et décomptes.** `lint-i18n-ownership`, `i18n-keys.test.ts`, `i18n-libelle-en-dur.test.ts`, G8 / G8-bis /
   G11 (`kesh-i18n/src/loader.rs`), G4-bis, G9, G18 (`textes_coherents.rs`) — C-29-1-40 a reformulé trois textes pour
   G8, G11 et G18 : la reformulation dit-elle encore vrai ? **Recompte depuis la source** tout décompte du Dev Agent
   Record (tests neufs par fichier aux **deux** bornes `fcd2ae75` / dernier commit de code, clés i18n, fichiers au
   `git diff --stat`, mutations tuées) et vérifie le **périmètre** déclaré de chaque nombre.

## Lentilles

- **B — Blind Hunter : bugs et états**, sans présupposer que la fiche a raison. SQL : `HAVING` sur alias sans `GROUP
  BY` (MariaDB), `CAST(… AS SIGNED)` de `EXISTS`, `LIMIT ?` lié en `i64`, `? IS NULL` sur une date `None` ; types sqlx
  décodés (un `SUM` rendu `DECIMAL` décodé en `i64` ?) ; une société sans compte bancaire, sans exercice suivant, avec
  un exercice Y **gappé** ; un relevé dont `period_to` < `period_from` ; le panneau ouvert deux fois de suite, fermé
  avant la réponse, rouvert sur un autre exercice ; le rappel quand le dialogue de clôture est fermé puis rouvert
  vite ; l'indentation des blocs déplacés dans la page (rendu inchangé ?).
- **E — Edge Case Hunter : bords, et les tests prouvent-ils ce qu'ils nomment.** Jours limites (premier, dernier,
  veille, lendemain) pour K2, K4, K6 ; K3 : compte clos **le** premier jour de Y, clos après Y, relevé arrêté **le** jour
  de la clôture du compte, deux relevés au même `period_to` ; écart `-0.005` ; rejoue mentalement la campagne et
  cherche la mutation verte (axe 6) ; un test qui passerait à vide (sélecteur absent → `toHaveCount(0)`, liste vide →
  `every` vrai) ; l'E2E (`fiscal-years.spec.ts`, préréglage `with-company`, `fys[0]` = l'exercice voulu ?, nettoyage).
- **A — Acceptance Auditor : conformité, décomptes, documentation.** Chaque AC de la partie a (AC1 à AC9, AC3-bis)
  contre le code : satisfait, partiel, absent ; T-a1 à T-a16 et M-a1 à M-a22 présents et conformes à la fiche ; les
  écarts déclarés au Dev Agent Record sont-ils tous déclarés (tests écrits après le code, ajout par `cat >>`) ; les
  captures (AC5 : panneau et rappel, clair et sombre — le « sombre » est une copie, C-28-1-7, C-29-1-39) ; les chiffres
  de gates et d'E2E déclarés correspondent-ils aux journaux `kesh-gate-logs/291a-*` (dernier gate au dernier commit de
  code) ; les choix C-29-1-32 à 42 : justifiés, ou défaut ? `sprint-status.yaml` : clé `29-1a-controles-prealables` en
  `review`, commentaire daté.

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/29-1a-review-p1-<B|E|A>.md` (ta lettre). Findings avec sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code relu), correction proposée. Pour
tout finding affirmant qu'un code ou un texte est absent ou présent : la sortie d'un `grep -nF` copiée.
⛔ **La liste des axes exercés ET non exercés** (les huit axes communs et ceux de ta lentille) — un « 0 finding » sans
elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/` (et le texte aplati du PDF au
même endroit) ; aucune commande qui écrit dans le dépôt ou dans une base, ni qui compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`, `latexmk`,
`xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`reset`/`rebase`/`restore`/`push`/`tag` — **aucun `git
checkout` ni `git switch` sous aucune forme**, pas même `--detach` —, `sqlx`, `cargo` (aucune sous-commande), `npm`,
`npx`, `node`, `deno`, `bun`, `python`/`python3` (pas même le script de mutations), `perl`, `ruby`, `php`, `sed -i`,
`awk` en écriture, tout autre interpréteur, `mariadb`, `mysql`, `docker`, `curl` et `wget`, `gh` en écriture,
`pkill`/`kill`, aucune requête SQL. Aucun fichier hors du dépôt. Autorisés : lecture, `grep`, `sed -n`, `wc`, `sort`,
`cmp`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers `/home/gcorbaz/devel/kesh-gate-logs/`, l'ouverture
des PNG de `kesh-gate-logs/29-1a-captures/`.
