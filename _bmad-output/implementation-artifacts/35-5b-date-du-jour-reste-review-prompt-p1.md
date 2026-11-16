# Prompt — revue de code P1 de la Story 35-5b (refs #929), trois lentilles, première passe complète

Tu relis la **Story 35-5b** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5 runes, MariaDB 10.11), release **0.17.7**, Epic 35
(« corrections par impact ») : **la date du jour devient celle de Zurich partout ailleurs dans la production** — l'accueil (#929),
les factures échues (fiche, liste, filtre « échues » et résumé de l'échéancier, son CSV), les rappels (jours de retard, liste « à
rappeler », jour du dernier rappel), les postes ouverts et la balance âgée, la fin d'un rapport de projet cumulé, la date de clôture
d'un compte bancaire par défaut, les noms des fichiers exportés, la démonstration, et le serveur MCP. Le SQL ne lit plus
`UTC_DATE()` : la date est liée. La 35-5a (close) a posé `jour_suisse` et basculé `Horloge::aujourd_hui()` ; la 35-5a0 la plomberie
(`AppState.horloge`, `Lecture` à horloge, `aujourd_hui_test()`, garde `date_du_jour_garde.rs`). **Aucune migration.** Kesh tient des
livres réels : une échéance jugée au mauvais jour fait relancer un client trop tôt, ou montre l'exercice de l'an passé le 1er janvier.

Worktree `/home/gcorbaz/devel/kesh-e355b`, branche `story/35-5b-date-du-jour-reste`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-e355b diff de2626a85 e3e8256fa` (`de2626a85` = 35-5a close, base ; `777664745` = squelette, au jour
UTC de l'instant de l'horloge ; `e3e8256fa` = dernier commit de code ; le commit suivant ne porte que la fiche, le PDF,
`sprint-status.yaml` et ce prompt). Lis la production en entier, les tests neufs en entier, les conversions de tests en entier (elles
sont courtes). Langue du rapport : français. Modèle : **Opus ou Sonnet** (première passe : jamais Haiku). Contexte frais.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/35-5b-date-du-jour-reste.md` en entier (*Ce qui se lit au code*, D1 à D9, AC1 à AC10,
mutations, *Dev Notes*, **Change Log** — entrée « Développement » — et **Dev Agent Record**) ; la synthèse de clôture de
`35-5a-date-du-jour-horloge.md` ; les registres `e35-choix-autonomes-e350.md` (C-e350-8, -14, -16, -21 à -26, -28, -29) et
`e35-choix-autonomes-e355b.md` (C-e355b-1 à -4). Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le avec les journaux
hors dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e355b/` : `gate-squelette-test-fast.log`, `rouges.log`,
`rouges-ac7.log`, `verts.log`, `mut/M1.log` à `mut/M7.log` et `mut/resume.log`, `gate-test-fast.log`, **`nuit-N1.log`, `nuit-B1.log`,
`nuit-B2.log`** et la synthèse **`nuit.log`**, `gate-ci.log`, `manuel-make.log`, `pdf-aplati.log` (environnement et commit en tête de
chacun). Les scripts `*.sh`, `*.py` sont à **lire**, jamais à exécuter.

**Livrables visibles à contrôler** : le manuel `docs/manual/fr/user-manual.tex` (note *La date du jour est celle de Zurich, même la
nuit*, `\label{sec:date-du-jour}` ; export CSV du journal d'audit) **et son PDF** `docs/manual/fr/user-manual.pdf` — à lire aplati :
`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (lecture seule, sortie à l'écran, sans redirection) ;
`docs/api-external.md` (`closedOn`, la synthèse de l'accueil, `asOf` des postes ouverts) ; `docs/testing.md:119` ; `CHANGELOG.md`,
section `## [0.17.7] — Non publié`, point #929. **Aucune capture** : la story ne change aucun contrôle visible (les écrans affichent ce
que le serveur rend ; vérifie-le). **Aucune clé i18n.**

⚠️ **L'E2E (Playwright) n'a pas tourné** sur cette story : il se joue à la clôture, avec un second `gate-ci`. Le compte rendu le dit.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis tes trois rapports. **Aucun fichier temporaire**, nulle part.
- **Aucune redirection, sous aucune forme** : ni `>`, `>>`, `<`, `tee`, ni `2>`, `&>`, `2>&1`, **ni `2>/dev/null`**, ni `> /dev/null`.
  **Aucune substitution de processus** (`<(…)`, `>(…)`) ni de commande qui écrive. Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni **`pkill`**, ni **`killall`**, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`/`reset`/`commit`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`, `jq` ;
  ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`, `scripts/mem-guard.sh`,
  ni aucun script du dépôt ou de `kesh-gate-logs` ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `sqlx`, `curl`, **`claude`**,
  `sed -i`, `playwright`, `vitest`. `pdftotext` est permis, sortie à l'écran.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep`, `sed -n`, `awk` (sans redirection ni
  `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, `pdftotext … -`.
- ⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport.

## Les axes obligatoires, pour chaque lentille

1. **Une lecture de l'horloge par requête, et la même date partout (D1)** : chaque route et chaque service lit-il la date **une fois**
   et la passe-t-il à tout ce qu'il appelle ? La page de l'échéancier, ses `isOverdue` et son résumé (`Echeancier::aujourd_hui`), la
   liste (`ListeFactures`), la fiche (`FicheFacture::aujourd_hui`, C-e355b-3 — écart à la lettre de D1, qui ne nommait pas la fiche) et
   le CSV (filtre, statut de chaque ligne, nom du fichier) disent-ils le même jour, même à minuit ? Les routes qui reçoivent un
   règlement ou une annulation (`from_parts` après `settle`, `write_off`, `cancel_settlement`, envoi par e-mail, regel du PDF) :
   l'annulation de règlement réemploie la date déjà lue (35-5a0), les autres en lisent une : une requête lit-elle deux fois ?
2. **Le SQL (D2)** : `push_where_clauses`, `due_dates_summary`, `list_for_export` lient-ils la date dans l'**ordre** des `?` — les deux
   liens du `SELECT` de `due_dates_summary` avant ceux du `WHERE` ? Les fragments `INVOICE_*_SQL` et `amount_due_derived_joins()`, écrits
   **après** les liens, ne contiennent-ils aucun `?` (preuve `grep -nF`) ? Le filtre « échues » et le résumé gardent-ils leur sens
   (`due_date < aujourd'hui`, `NULL` jamais échue) ?
3. **Les deux « jours d'un instant » (D4)** : `dunning_eligibility.rs` compare `jour_suisse(lr.and_utc())` au jour passé, et le CSV
   écrit `jour_suisse(d.and_utc())` pour `paid_at` : les colonnes sont-elles bien des instants UTC (`pool.rs`, session `+00:00`) ? La
   **limite connue** (fiche, *Hors périmètre*) — la fenêtre de 30 jours de la réconciliation (`reconciliation.rs:237`,
   `DATE(rmd.last_sent_at)`) reste au jour UTC (#1053) — est-elle écrite au Change Log, au manuel et au CHANGELOG, et nulle part
   contredite ?
4. **La graine et le MCP (D3, D8, AC5, AC6)** : `seed_demo` n'a qu'un appelant de production (`routes/onboarding.rs`), qui lit
   `state.horloge` ; `auth/bootstrap.rs` (`mod tests`) passe une date fixe ; les pièges reçoivent leur date des bancs. Le MCP construit sa
   `Lecture` avec l'horloge de l'état (`lib.rs`) : `banc_avec_horloge` / `routeur_avec_horloge` fixent-ils bien **cette** horloge
   (l'état est-il construit avant le routeur) ? `kesh_factures` n'a pas de filtre « échues » : rien d'autre à juger au jour côté MCP ?
5. **Les tests neufs** (`jour_suisse_nuit_e2e.rs`, fin de fichier : AC1 à AC4 ; `mcp_outils_e2e.rs` : AC5 ; `fiscal_years_e2e.rs` :
   AC6 ; `kesh-db/tests/dunning_eligibility.rs` : `le_jour_du_dernier_rappel_est_celui_de_zurich` ; `routes/invoices.rs` : M7) :
   chaque assertion rougit-elle sur le squelette `777664745` (`rouges.log`, test par test, motif lu) ? Un test qui passe **à vide**
   (attendu vrai sur les deux codes) ? AC3, côté HTTP, ne rougit au squelette que sur les jours de retard (l'assertion d'éligibilité
   n'est pas atteinte) : l'éligibilité est-elle prouvée rouge ailleurs (dépôt, M3) ? Le montage SQL (rappel inséré, délai ramené à 7
   jours, `paid_at` posé à la main) est-il cohérent avec le dépôt ?
6. **Les conversions de tests 35-5b** (`bank_accounts_closed_e2e`, `dunning_reminders_e2e`, `fiscal_years_e2e`,
   `homepage_summary_e2e`, `mcp_outils_e2e`, `onboarding_audit_e2e`, `onboarding_e2e`, `open_items_e2e`, `services_motif`,
   `kesh-db/tests/dunning_eligibility`) et les appelants à paramètre neuf (`invoice_amount_due_parity`, `invoice_ttc_parity`,
   `kf005_fulltext_index_e2e`, `services_rollout`, `common/mcp_pieges`) : une date **de pièce** remplacée par le jour (ce qui changerait
   ce que le test prouve) ? Deux horloges dans un même test ? Le module de tests de `kesh-db invoices.rs` en date fixe (C-e355b-1) : la
   date `2026-06-15` tombe-t-elle dans l'exercice qu'`ensure_fiscal_year` rend, et un test y dépendait-il de l'heure réelle ?
7. **L'inventaire par exécution (AC8, `nuit.log`)** : la nuit réelle reproduite (`nuit-N1.log`, tous les binaires) et les deux
   instants de la fiche sur les binaires 35-5b (`nuit-B1.log`, `nuit-B2.log`) : chaque rouge est-il rattaché, ligne lue, à un fichier
   35-5c ou à un intermittent connu (#1126) ? La **sonde positive** (un rouge connu vu par la méthode) est-elle réelle ? Les angles morts
   sont-ils écrits ?
8. **La garde (D5, AC7)** : plus aucune exemption de production hors `kesh-core/src/horloge.rs` (1) et `admin.rs` (1, `:724`) ; chaque
   fichier 35-5b compte 0 ; le rouge d'AC7 sur la seule liste du squelette (`rouges-ac7.log`) ; les sondes vertes ; M1 la fait rougir.
9. **Docs et manuel (D7, AC9)** : la note du manuel énumère-t-elle chaque site converti, et **aucun** resté en UTC (horodatages, filtres
   du journal d'audit, réconciliation) ? Le PDF porte-t-il le texte du `.tex` (aplati) ? `api-external.md` : une autre phrase « date du
   jour » ou « UTC » qui devrait maintenant dire Zurich ? Les commentaires qui justifiaient l'UTC (liste de la fiche) sont-ils tous
   réécrits — `/usr/bin/grep -n UTC` sur chaque fichier de production touché ? Le CHANGELOG complète-t-il le point #929 sans en écrire un
   second ?
10. **Mutations** : M1 à M7 tuées chacune par le test **nommé** au journal (`mut/`) ? Propose des mutations plausibles non jouées (le
    `push_bind` du `SELECT` de `due_dates_summary` remplacé par `UTC_DATE()` — un seul des deux ; `list_for_export` sans la date ; la
    route des rappels sur l'heure réelle ; `parse_as_of` qui ignore son paramètre).

## Lentille B — Blind (le diff seul, sans la fiche)

Relis `kesh-db/src/repositories/{invoices,dunning_eligibility}.rs`, `kesh-seed/src/{lib,pieges}.rs`, `kesh-api/src/services/{factures,
postes_ouverts,rapports}.rs`, `kesh-api/src/routes/*.rs` touchés, `mcp/outils/factures.rs`, puis les tests. Cherche : une date lue deux
fois ou par deux horloges ; un `?` mal ordonné ; un test qui passe à vide ; une date de pièce prise pour le jour.

## Lentille E — Edge (chaque branche, chaque borne)

Minuit pile à Zurich (22:59:59Z / 23:00:00Z en hiver, 21:59:59Z / 22:00:00Z en été) pour « échue » et pour le rappel ; une échéance
`NULL` ; un rappel envoyé à 23:59:59 heure suisse ; le 1er janvier sans exercice suivant (accueil `noOpenFiscalYear`) ; un rapport de
projet cumulé dont la fin est donnée (le défaut n'agit pas) ; `asOf=` vide ; un CSV sans paiement ; un banc MCP sans horloge fixée. Chaque
cas : comportement au code, test qui le couvre (ou absence, prouvée par `grep -nF`).

## Lentille A — Acceptance (la fiche contre le livré)

Chaque critère AC1 à AC10 : livré, testé, au lieu prescrit ? Les écarts déclarés (C-e355b-1 : module de tests de `kesh-db` en date fixe
dès le squelette ; C-e355b-3 : la fiche d'une facture ; le squelette commité avec un test rouge et une garde rouge, puis corrigé dans le
commit suivant) sont-ils exacts et sans effet sur ce que l'AC veut prouver ? L'étape 0 (sites recomptés, specs E2E reconfirmées) citée
ligne lue ? Les gates du passage en `review` (`gate-test-fast.log`, `gate-ci.log`) : chaque chiffre au journal, au commit `e3e8256fa`,
horodaté après lui ? Le Dev Agent Record n'affirme-t-il que ce qui a tourné (E2E : **non lancé** ; frontend : **non touché**) ?

## Rapport

Écris **un fichier par lentille** : `/home/gcorbaz/devel/kesh-gate-logs/e355b-review-p1-B.md`, `…-p1-E.md`, `…-p1-A.md`. Chaque
rapport : findings numérotés (`B1`, `E1`, `A1`…) avec sévérité (CRITICAL / HIGH / MEDIUM / LOW), lieu (`fichier:ligne`), ligne lue
copiée, rejeu ou preuve, remède proposé ; **la liste des axes exercés ET non exercés** ; le décompte final **recompté depuis la
liste**.
