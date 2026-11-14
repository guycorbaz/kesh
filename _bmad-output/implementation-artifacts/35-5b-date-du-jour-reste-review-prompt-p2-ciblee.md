# Prompt — revue de code P2 ciblée de la Story 35-5b (#929), une lentille, braquée sur la remédiation de la P1

Passe **ciblée**, une seule lentille, contexte frais, **Sonnet** (P1 : Opus, trois lentilles), en lecture seule. Worktree
`/home/gcorbaz/devel/kesh-e355b`, branche `story/35-5b-date-du-jour-reste`. La P1 a rendu 2 MEDIUM (E1 = B1, A1) et une quinzaine de
LOW (rapports `/home/gcorbaz/devel/kesh-gate-logs/e355b-review-p1-{B,E,A}.md`). Déjà tracés, à ne pas rapporter de nouveau : #1053
(fenêtre de 30 jours de la réconciliation au jour UTC), #1125 (tests à dates fixes de 2026), #1126 (test MCP intermittent). Ce qui
relève de la 35-5c par décision (tests et specs sans attendu lié à un site de cette story) n'est pas un défaut de la 35-5b.

## Périmètre : le seul commit `73dc06b1e`

`git -C /home/gcorbaz/devel/kesh-e355b show 73dc06b1e` (7 fichiers). Compte rendu : le commit qui le suit, `git log --oneline
73dc06b1e..HEAD` (Change Log « Remédiation P1 », annotations du Dev Agent Record, registre C-e355b-5 et -6, annotation de C-e350-25,
PDF du manuel). Journaux sous `/home/gcorbaz/devel/kesh-gate-logs/e355b/` : `p1-cibles.log`, `p1-mut-*.log` et `p1-mut-resume.log`,
`p1-nuit-jour.log`, `p1-test-fast.log`, `p1-manuel-make.log`, `p1-pdf-aplati.log`, et le § 4/§ 5 ajoutés à `nuit.log`.

Il remédie :
- **E1 = B1 (MEDIUM)** — `paid_at` n'est pas un instant : le CSV de l'échéancier revient à `format_date(&d.date())`
  (`routes/invoices.rs`, colonne de paiement) ; le test `ac2_la_date_de_paiement_du_csv_est_le_jour_suisse` (valeur `23:30:00`
  posée à la main) est remplacé par `ac2_la_date_de_paiement_du_csv_est_celle_du_reglement` (règlement réel par la route, de nuit,
  `settledOn` du 10) ; doc d'`Invoice::paid_at` (`kesh-db/src/entities/invoice.rs`), manuel (note `sec:date-du-jour`), CHANGELOG
  (point #929) corrigés ; mutation M6b.
- **A1 (MEDIUM)** — le témoin de jour d'AC8 (`KESH_TEST_INSTANT=2026-06-15T10:00:00Z`) joué sur les 16 binaires 35-5b.
- **B4 = E2** — `render_reminder` reçoit `today` ; `send_reminder_batch` lit la date une fois et la passe par
  `send_one_batch_reminder` ; l'aperçu lit la sienne.
- **B2, B3 = E5 = A5** — AC2 lit `summary.overdueTotal` et l'export `export.csv?paymentStatus=overdue` (`VueEcheance`) ; M8, M9.
- **E3** (`ac1_l_accueil_la_nuit_du_nouvel_an_sans_exercice_neuf`, M11), **E4** (`ac4_la_fin_donnee_d_un_projet_l_emporte_sur_le_jour`,
  M10), **E6** (`ac2_minuit_pile_en_heure_d_ete`, M12).
- **B6** (commentaire de la garde), **A4** (renvois `\S\ref{sec:date-du-jour}` depuis l'accueil et l'échéancier du manuel), **E7**,
  **A2**, **A3**, **B5** (comptes rendus seulement).

## Ce que tu vérifies

1. **E1** : `git grep -n "paid_at" -- 'crates/*/src/*'` — chaque écrivain de production pose-t-il bien une date à minuit (ligne lue
   citée), ou en reste-t-il un qui pose un instant (`NOW`, `Utc::now`) ? Le commentaire de la colonne et la doc d'`Invoice::paid_at`
   disent-ils vrai, y compris la phrase sur les factures marquées payées avant la v0.12 ? Le nouveau test : la route de règlement
   écrit-elle bien `2026-03-10 00:00:00` (lecture de `invoice_settlements_write.rs`) ; l'attendu `10.03.2026` rougirait-il sur une
   régression plausible (la colonne prenant le jour de la requête : M6b, `p1-mut-M6b.log`) ; le montage (compte bancaire,
   `settledOn` dans l'exercice 2026 ouvert, montant 108.10) est-il cohérent ? Le CHANGELOG et le manuel n'annoncent-ils plus nulle
   part un changement de la date de paiement (`git grep -n` sur « paiement », « paid » dans `CHANGELOG.md` section 0.17.7,
   `user-manual.tex`, `docs/api-external.md`) ?
2. **B4** : `git grep -n "render_reminder\|horloge" -- crates/kesh-api/src/routes/invoice_email.rs` — une seule lecture par requête
   (aperçu, lot), plus aucune dans `render_reminder` ? Signature et appelants cohérents ? Aucun test ne peut le prouver sous une
   horloge arrêtée (la lecture unique et la lecture répétée y sont équivalentes) : le compte rendu le dit-il, sans prétendre à une
   mutation tuée ?
3. **Les tests neufs ou étendus** (`jour_suisse_nuit_e2e.rs`) : `VueEcheance.total_echues` et `export_echues` — comparés aux
   deux instants, avec des attendus distincts (108.10 / 0, présent / absent) ? Le filtre de l'export (`paymentStatus=overdue`) passe-
   t-il bien par `list_for_export` avec la date de la route ? E3 : l'attendu `noOpenFiscalYear`, `asOf` 2026-01-01, `fiscalYear` null
   est-il celui du code (`kesh-report/src/homepage_summary.rs`) ? E4 : l'écriture du 2026-06-01, postérieure au jour, est-elle
   possible par le dépôt sans garde de bornes de projet (lu) ; le total attendu tient-il à la date de fin donnée seule ? E6 : 21:59:59Z
   et 22:00:00Z en juillet sont-ils bien 23:59:59 et 00:00 à Zurich ; le test asserte-t-il ce qui rougit au jour UTC (M12) ?
4. **Mutations** (`p1-mut-resume.log`, `p1-mut-*.log`) : chacune appliquée sur l'arbre commité `73dc06b1e` (en-tête), tuée par le test
   nommé, fichier restauré (`git status` vide après) ? M8 et M9 lient une **autre date** sans forme gardée (la garde textuelle ne
   peut les voir) : le test de comportement seul les tue-t-il ?
5. **A1** (`p1-nuit-jour.log`) : instant `2026-06-15T10:00:00Z`, filtre des 16 binaires 35-5b, commit `73dc06b1e` ; résultat et tri
   d'un éventuel rouge, ligne lue citée ; reporté au § 5 de `nuit.log` ?
6. **Les textes** : `user-manual.tex` (note `sec:date-du-jour`, accueil « Quel exercice », échéancier), le **PDF aplati**
   (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` puis `/usr/bin/grep -o`) : chaque phrase vraie au code ;
   les renvois résolus (pas de `§??`) ; « envoi d'un rappel, paiement » et « date de paiement de l'export » absents.
7. **Le compte rendu et les gates** : `p1-test-fast.log` au commit de code, bases remises à neuf, chiffre exact ; `gate-ci` et E2E
   déclarés « à la clôture », non affirmés ; entrées antérieures du Change Log et `nuit.log` **annotées sans réécriture**
   (`git show` du commit de compte rendu, lignes `-`) ; registre C-e355b-5, C-e355b-6, annotation de C-e350-25 justes au code ;
   aucun mot-clé de fermeture dans les messages (`git log de2626a85..HEAD --format=%B | /usr/bin/grep -niE
   '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`).
8. **Attendus retouchés** : `git show 73dc06b1e -- '*.rs'`, lignes `-` dans les tests — un attendu existant affaibli ou retiré sans
   motif (hors le test d'E1, remplacé par décision) ?

## Interdits absolus

Lecture seule : aucun fichier écrit hormis ton rapport (outil Write). **Aucune redirection, sous aucune forme** — ni `>`, ni `>>`, ni
`tee`, ni `2>`, ni `2>&1`, ni `2>/dev/null`, ni `> /dev/null` —, **aucune substitution de processus** (`<(…)`), aucun fichier
temporaire. Des relecteurs de cette epic ont écrit `2>/dev/null` ou `2>&1` « par réflexe » : relis chaque commande avant de la lancer.
Aucun `rm`, `pkill`, `kill`, aucun `git` qui écrit. Aucun interpréteur ni script (`python3`, `perl`, `node`, `npm`, `npx`, `bash -c`,
`sh -c`, `jq`, `scripts/*` dont **`scripts/prepare-release.sh`**, les scripts de `kesh-gate-logs`, `make`, `latexmk`, `cargo`,
`docker`, `mariadb`, `curl`, **`claude`**, `sed -i`, `vitest`, `playwright`), aucune commande `gh`, aucun accès réseau. Ne lis rien
sous `~/kesh/` ni `~/kesh-test/`. Permis : `git show/diff/log/grep/ls-files`, `/usr/bin/grep`, `sed -n`, `awk` sans redirection ni
`system()`, `cat`, `ls`, `find` sans `-exec`/`-delete`, `wc`, `sort`, `uniq`, `cmp`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
⚠️ `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`. Toute affirmation de présence ou d'absence s'appuie
sur une sortie copiée au rapport. Un écart se déclare.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/e355b-review-p2-ciblee.md` : findings numérotés (`C1`…), sévérité (CRITICAL/HIGH/MEDIUM/LOW),
lieu, ligne lue copiée, preuve, remède ; **axes exercés et non exercés** ; décompte final recompté depuis la liste.
