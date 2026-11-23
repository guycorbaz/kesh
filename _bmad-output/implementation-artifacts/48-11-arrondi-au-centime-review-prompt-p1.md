# Prompt — revue de code P1 de la story 48-11 « Arrondi au centime » (#1245, #1430), trois lentilles, première passe complète

Tu relis la story **48-11** de Kesh (comptabilité suisse ; Rust/Axum, MariaDB, SvelteKit), release du **vendredi 2026-11-27**, Epic 50.
Kesh tient les livres réels du Project Lead, qui a **déjà émis de vraies factures remises sur papier** : rien de ce qui existe
(facture, avoir, lignes, totaux, TVA, écritures, numéros, **PDF de facture**) ne doit changer. **Aucune migration.** Deux effets :

- **Rapports (PDF et CSV)** : `{:.2}` sur un `Decimal` **coupe** les chiffres (`10.0050` → `10.00`, `-0.0040` → `-0.00`) ; les
  rapports passent désormais par `kesh_core::types::format_two_decimals` (arrondi au centime, l'équidistant loin de zéro, zéro sans
  signe), comme l'écran (big.js). `round_for_display` déménage de `kesh-report/src/pdf.rs` vers `kesh-core/src/types/money.rs`
  (**#1430** : le modèle du bilan ne dépend plus du module de rendu).
- **Export global** : plus d'arrondi ni de coupe ; **chaque colonne `DECIMAL` s'écrit à l'échelle de sa colonne dans le schéma**
  (`fmt_decimal_scale`, constantes `DEC_19_4`, `DEC_18_2`, `DEC_7_2`, `DEC_5_2`, `DEC_20_10`) — décision de Guy du 2026-11-20 : la
  **forme** des 25 colonnes à quatre décimales change (`150.25` → `150.2500`), aucune valeur enregistrée.
- Une **garde** (`crates/kesh-api/tests/arrondi_centime_garde.rs`) fige, par fichier et à égalité stricte, les précisions de format
  et les troncatures restantes de `crates/*/src`.

Worktree `/home/gcorbaz/devel/kesh-48-11`, branche `story/48-11-arrondi-au-centime`, partie de `epic/48-corrections-0-20-0`.
Diff à relire, **aplati** : `git -C /home/gcorbaz/devel/kesh-48-11 diff 15ae0368a~1 <DERNIER_COMMIT_DE_CODE>` (commits de code :
`15ae0368a` tests rouges, `9a6c80f81` correctif et garde, `e704d397a` tests de zéros négatifs ; les suivants ne portent que la fiche
et ce prompt — vérifie par `git diff --stat <DERNIER_COMMIT_DE_CODE> HEAD`). Langue du rapport : français. **Modèle : Opus** (première
passe : jamais Haiku), contexte frais, trois lentilles en parallèle.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/48-11-arrondi-au-centime.md` en entier (D1 à D9, AC1 à AC13, tableau des mutations,
Note de version, **Dev Agent Record**, **Change Log**) et le dernier rapport de validation
`/home/gcorbaz/devel/kesh-gate-logs/e4811/validation-p3.md`. Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le
avec les journaux hors dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e4811/` : `dev/rouge-report.log`,
`dev/rouge-api.log` (les rouges), `dev/vert-1.log` (le vert ciblé), `dev/ac8.txt`, `dev/ac10.txt`, `dev/ac12.txt`, `mut/` (script
`muter.py`, un journal par mutation `mut-m1.log` … `mut-m8.log`, `mut-m2-survivant.log` = le premier M2, qui a survécu, et
`mut-m2-sante.log`), `dev/gate-*.log` et `dev/gate-exit.txt` (`test-fast --ci`, `gate-ci`). **L'E2E complet n'a pas tourné** : il
tourne à la clôture, après la revue (la story n'ajoute aucun contrôle visible : D9, pas de captures).

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, nulle part ;
  **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`). Un fichier créé par erreur se **déclare** ; il ne se supprime pas.
- **Aucun `rm`**, sous aucune forme ; ni `pkill`, ni `killall`, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`/`commit`/`reset`/`merge`/`rebase`/`push`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`,
  `jq` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`,
  `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` (dont `mut/muter.py`, `dev/gate-review.sh`,
  `dev/lourd.sh` : ils se **lisent**) ; ni `make`, `latexmk`, `pdflatex`, `cargo`, `docker`, `mariadb`, `sqlx`, `curl`, **`claude`**,
  `sed -i`, `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`, `awk` (sans
  redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, et, pour un PDF, la seule
  forme `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`.
- ⚠️ `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport ; tout chemin
  cité se vérifie (`git cat-file -e <ref>:<chemin>` ou `ls`).

## Les axes obligatoires, pour chaque lentille

1. **Les AC.** Chaque AC (AC1 à AC13) et chaque décision (D1 à D9) contre le code, les tests et les journaux. Un AC sans test qui
   peut rougir doit être déclaré filet, avec sa raison. Les écarts à la fiche sont-ils déclarés au Dev Agent Record ? (Écarts connus :
   M1 ne rougit pas `exports_global_e2e`, l'export global n'arrondissant pas ; M2 a **survécu** au premier jeu de tests, parce que
   `round_dp_with_strategy` rend déjà un zéro positif pour `-0.0040`, et n'est tué que par les zéros négatifs **construits** ajoutés
   en `e704d397a` — la garde `if r.is_zero()` est défensive : juge si ce rôle est réel.)
2. **Le manuel** : `docs/manual/fr/admin-manual.tex` (section « Écart admis pour un règlement de plusieurs factures », `\subsubsection{Passer
   à la 0.20.0}` et son paragraphe « Pour les intégrations ») et `docs/manual/fr/user-manual.tex` (§ *Contenu du ZIP*) disent-ils ce
   que fait le code (quatre décimales, `150.2500` ; deux pour les taux de TVA, frais de rappel, montants et soldes de relevé ; dix
   pour les cours ; rapports arrondis au centime) ? Les manuels DE/IT/EN n'existent pas (`ls docs/manual/{de,it,en}`). **Le PDF
   n'est pas régénéré dans la branche** : contrôle, forme unique, aplati :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c 'tronque à deux décimales'` — le PDF
   de la branche porte **l'ancien** texte (attendu non nul, à dire, non à compter comme défaut). `git grep -nE '1245|tronqu'
   -- docs/manual` ne doit plus citer la phrase périmée. Le README (feuille de route) et `website/` : rien de promis qui manque.
3. **Les mutations M1 à M8** (script `mut/muter.py`, journaux `mut/mut-m*.log`) : chacune est-elle tuée **pour la bonne raison**, par
   le test que la fiche nomme ? Chaque journal montre-t-il le diff de la mutation, le rouge et « == restaure » ? M7 nomme-t-elle
   `invoice_lines.line_total` ? M8 rougit-elle sur le décompte ? Un test vert par construction (attendu dérivé du même extracteur,
   comparaison de vide à vide) ?
4. **L'inventaire par le symptôme** (CLAUDE.md § *Inventorier les sites NON RÉSOLUS*) : refais **toi-même** la recherche — toute
   précision de format sur un `Decimal` (`{x:.N}`, `{:.N}`, `{x:>10.N}`, `{x:.p$}`, `{:.*}`), toute fonction `Decimal → String`,
   toute troncature (`trunc`, `ToZero`, `MidpointTowardZero`, `to_i64` après ×100, `floor`), tout `format!("{}", montant)` d'un
   montant dans un rapport ou un export — dans `crates/*/src` **et** `crates/*/tests`, et le **frontend** (`toFixed`, `Intl`,
   `Math.round`/`trunc`/`floor`). Chaque site est-il corrigé, sain ou angle mort écrit ? La liste close de la garde (22
   occurrences dans 8 fichiers ; 2 troncatures) est-elle juste, **ligne lue** ? Une famille que la garde ne voit pas et qui coupe un
   montant (par exemple un `Decimal` écrit par `{}` dans un rapport, ou `rust_decimal::Decimal::round_dp` au pair, #1441/#1442/#1443) ?
5. **L'export global, colonne par colonne** : les 38 colonnes `DECIMAL` (25 à quatre, 10 à deux, 3 à dix) du schéma
   `crates/kesh-db/test-schema/0001_schema_squash.sql` ont-elles toutes la bonne constante à leur site (`csv_tables.rs`) ? Le test de
   pleine échelle (`export_global_chaque_colonne_decimal_a_l_echelle_du_schema`) lit-il l'échelle d'une source **autre** que le code
   testé (`information_schema`) ? Ses dérogations (contraintes `CHECK`) sont-elles justes ? Un consommateur de l'export (le serveur
   MCP, l'import d'installation, la restauration d'une sauvegarde, `post_restore.rs`) lit-il ces CSV et pourrait-il casser sur
   `150.2500` ? (`git grep` des lecteurs de `journal_entry_lines.csv` & co.)
6. **Factures réelles** : `git diff 15ae0368a~1 <DERNIER_COMMIT_DE_CODE> --stat -- crates/kesh-qrbill crates/kesh-api/src/routes/invoice_pdf_service.rs crates/kesh-db 'frontend/src/routes/(app)/invoices' frontend/src/lib/features/invoices`
   doit être **vide**. Aucun rapport ne partage-t-il un formateur avec le PDF d'une facture ou d'un avoir ? Un total de facture
   (`rounding_amount`, arrondi aux 5 centimes) exporté à quatre décimales reste-t-il juste ?
7. **La fiche et la Note de version** : le Dev Agent Record dit-il vrai (journaux et commits cités ; décomptes recomptés depuis le
   diff) ? La Note de version dit-elle vrai, issue par issue ? Les gates annoncés ont-ils un journal (`dev/gate-*.log`) ?
8. **Code visible** : la story ne modifie aucun contrôle visible (D9) ; confirme-le par `git diff --stat -- frontend`.

## Lentilles

- **B — Blind Hunter** : le diff seul d'abord (sans la fiche), défauts de code, d'état, de régression ; puis les axes.
- **E — Edge Case Hunter** : toutes les branches et états limites, axes 3, 4 et 5 surtout : montants négatifs, zéros négatifs,
  échelle 0 à 10, très grands montants (`Decimal::MAX` : `round_dp_with_strategy` déborde-t-il ?), `Option` vides, devises.
- **A — Acceptance Auditor** : chaque AC et chaque décision contre le code et les journaux, axes 1, 2, 6 et 7 surtout.

## Rapport

Chaque lentille écrit son rapport (le seul fichier permis) dans
`/home/gcorbaz/devel/kesh-gate-logs/e4811/review-p1-{B,E,A}.md` : findings classés CRITICAL / HIGH / MEDIUM / LOW, chacun avec
**la ligne lue citée** ; puis **la liste des axes exercés et des axes non exercés**, avec la raison (un « 0 finding » sans cette
liste ne compte pas comme passe) ; puis les chemins vérifiés (E9). Tout finding CRITICAL ou HIGH qui affirme une absence ou un
anti-pattern se vérifie d'abord par `/usr/bin/grep -nF`.
