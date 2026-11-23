# Prompt — revue de code P2 de la story 48-11 « Arrondi au centime » (#1245, #1430), passe complète, Sonnet

Tu relis la story **48-11** de Kesh (comptabilité suisse ; Rust/Axum, MariaDB, SvelteKit), release du **vendredi 2026-11-27**, Epic 50.
Kesh tient les livres réels du Project Lead, qui a **déjà émis de vraies factures remises sur papier** : rien de ce qui existe
(facture, avoir, lignes, totaux, TVA, écritures, numéros, **PDF de facture**) ne doit changer. **Aucune migration.** Deux effets :

- **Rapports (PDF et CSV)** : les rapports passent par `kesh_core::types::format_two_decimals` (arrondi au centime, l'équidistant loin
  de zéro, zéro sans signe) au lieu de `{:.2}`, qui **coupe** (`10.0050` → `10.00`) ; `round_for_display` déménage de `kesh-report` vers
  `kesh-core/src/types/money.rs` (#1430).
- **Export global** : chaque colonne `DECIMAL` s'écrit **à l'échelle de sa colonne** (`fmt_decimal_scale`, constantes `DEC_*`) :
  `150.25` → `150.2500`, aucune valeur enregistrée ne change.
- Une **garde** (`crates/kesh-api/tests/arrondi_centime_garde.rs`) fige les précisions de format et troncatures de `crates/*/src`.

Worktree `/home/gcorbaz/devel/kesh-48-11`, branche `story/48-11-arrondi-au-centime`, partie de `epic/48-corrections-0-20-0`.
**Passe complète, modèle Sonnet, contexte frais.** Un seul agent exerce **successivement les trois lentilles B, E et A** sur **toute
la story et sur la remédiation de la revue P1**. Diff à relire, aplati :
`git -C /home/gcorbaz/devel/kesh-48-11 diff 15ae0368a~1 <HEAD de code>` (le commit de code de la remédiation est nommé au Change Log
de la fiche ; `git log --oneline eb0c66c97..HEAD` le montre) ; la remédiation seule : `git diff eb0c66c97 <HEAD de code>`.
Langue du rapport : français.

Lis d'abord : la fiche `_bmad-output/implementation-artifacts/48-11-arrondi-au-centime.md` en entier (D1 à D9, AC1 à AC13, mutations,
Note de version, Dev Agent Record, **Change Log, dont l'entrée « revue P1 remédiée »**), le rapport de validation
`/home/gcorbaz/devel/kesh-gate-logs/e4811/validation-p3.md` et les trois rapports de la P1
`/home/gcorbaz/devel/kesh-gate-logs/e4811/review-p1-{B,E,A}.md`. Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le
avec les journaux (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e4811/` : `dev/rouge-*.log`, `dev/vert-*.log`, `dev/ac*.txt`,
`mut/` (journaux `mut-m1.log` … `mut-m8.log`, `mut-m2b.log`, scripts `muter.py` et `muter2.py`), `dev/gate-*.log`, `dev/gate-exit.txt`.
**L'E2E complet n'a pas tourné** (clôture) ; la story n'ajoute aucun contrôle visible (D9).

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, nulle part ;
  **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`). Un fichier créé par erreur se **déclare** ; il ne se supprime pas.
- **Aucun `rm`**, ni `pkill`, `killall`, `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`/`commit`/`reset`/`merge`/`rebase`/`push`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`,
  `jq` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`,
  `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` (dont `mut/muter.py`, `mut/muter2.py`, `dev/lourd.sh`,
  `dev/remed.sh` : ils se **lisent**) ; ni `make`, `latexmk`, `pdflatex`, `cargo`, `docker`, `mariadb`, `sqlx`, `curl`, **`claude`**,
  `sed -i`, `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`, `awk` (sans
  redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, et, pour un PDF, la seule
  forme `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`.
- ⚠️ `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport ; tout chemin
  cité se vérifie (`git cat-file -e <ref>:<chemin>` ou `ls`).

## Les axes obligatoires, pour chaque lentille

1. **Les AC et les décisions.** Chaque AC (AC1 à AC13) et chaque décision (D1 à D9) contre le code, les tests et les journaux. Un
   AC sans test qui peut rougir est déclaré filet. **Nouveau depuis la P1** : l'AC6 et l'AC1 testent-ils de **vrais zéros négatifs**
   (`-Decimal::ZERO`, `-Decimal::new(0, 2)`, et le solde `-0` saisi, chemin `manual.rs:790` → `bank_statements_manual.rs:354`) ?
   `git grep -nF 'dec!(-0)' -- crates` ne doit plus rendre un cas qui prétende tester un zéro négatif (`dec!(-0)` est un zéro positif).
2. **Le manuel et le guide de l'API** : `docs/manual/fr/admin-manual.tex` (section « Écart admis pour un règlement de plusieurs
   factures », `\subsubsection{Passer à la 0.20.0}` et son paragraphe « Pour les intégrations », dont la phrase sur les cours de change)
   et `docs/manual/fr/user-manual.tex` (§ *Contenu du ZIP*) **et `docs/api-external.md`** (paragraphe « L'export global change de forme »,
   *depuis la v0.20.0*) disent-ils ce que fait le code (quatre décimales ; deux pour taux de TVA, frais de rappel, montants et soldes de
   relevé ; dix pour les cours) ? Les manuels DE/IT/EN n'existent pas (`ls docs/manual/{de,it,en}`). **Le PDF n'est pas régénéré dans
   la branche** : contrôle, forme unique, aplati :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c 'tronque à deux décimales'` (le PDF porte
   **l'ancien** texte : attendu, à dire, non à compter comme défaut ; même forme pour `user-manual.pdf`). `git grep -nE '1245|tronqu'
   -- docs/manual` : recompte et lis chaque ligne. README et `website/` : rien de promis qui manque.
3. **Les mutations M1 à M8 et M2b** (scripts `mut/muter.py`, `mut/muter2.py`, journaux `mut/mut-m*.log`) : chacune est-elle tuée **pour
   la bonne raison**, par le test que la fiche nomme ? Chaque journal montre-t-il le diff, le rouge et « == restaure » ? **M2b** (retrait
   de `if r.is_zero()`) rougit-elle sur les vrais zéros négatifs, dont le solde `-0` du relevé saisi ? Un test vert par construction
   (attendu dérivé du même extracteur, comparaison de vide à vide, valeur qui n'est pas celle qu'on croit construire) ?
4. **L'inventaire par le symptôme** (CLAUDE.md § *Inventorier les sites NON RÉSOLUS*) : refais **toi-même** la recherche — toute précision
   de format sur un `Decimal` (`{x:.N}`, `{:.N}`, `{x:>10.N}`, `{x:.p$}`, `{:.*}`, **`{:.1$}`**), toute fonction `Decimal → String`, toute
   troncature, tout `format!("{}", montant)` dans un rapport ou un export — dans `crates/*/src` **et** `crates/*/tests`, et le frontend.
   Chaque site est-il corrigé, sain ou angle mort écrit ? La liste close de la garde (22 occurrences dans 8 fichiers ; 2 troncatures)
   est-elle juste, **ligne lue** ? La sonde positive (7 formes) couvre-t-elle la forme positionnelle ?
5. **L'export global, colonne par colonne** : les 38 colonnes `DECIMAL` (25 à quatre, 10 à deux, 3 à dix) du schéma
   `crates/kesh-db/test-schema/0001_schema_squash.sql` ont-elles la bonne constante à leur site (`csv_tables.rs`) ? Le test de pleine
   échelle lit-il l'échelle d'une source **autre** que le code testé ? **Nouveau** : le `debug_assert!(d.scale() <= scale)` de
   `fmt_decimal_scale` peut-il paniquer sur une valeur réelle lue en base (échelle supérieure à celle de la colonne, valeur calculée) ?
   Un consommateur des CSV (MCP, import d'installation, restauration, `post_restore.rs`) casse-t-il sur `150.2500` ?
6. **Factures réelles** : `git diff 15ae0368a~1 <HEAD de code> --stat -- crates/kesh-qrbill crates/kesh-api/src/routes/invoice_pdf_service.rs crates/kesh-db 'frontend/src/routes/(app)/invoices' frontend/src/lib/features/invoices`
   doit être **vide**. Aucun rapport ne partage un formateur avec le PDF d'une facture ou d'un avoir ?
7. **La fiche, la Note de version, les gates** : le Dev Agent Record dit-il vrai (journaux et commits cités ; décomptes recomptés) ?
   Le gate complet annoncé à la remédiation a-t-il un journal (`dev/gate-*.log`, `dev/gate-exit.txt`) au commit de code nommé ?
   **La propagation** : la remédiation a-t-elle greppé le symptôme (CLAUDE.md § *Propagation post-patch*) ?
8. **Code visible** : aucun contrôle visible modifié (D9) ; confirme par `git diff --stat -- frontend`.

## Lentilles (exercées l'une après l'autre par le même agent)

- **B — Blind Hunter** : le diff seul d'abord (sans la fiche), défauts de code, d'état, de régression ; puis les axes.
- **E — Edge Case Hunter** : toutes les branches et états limites, axes 3, 4 et 5 surtout : négatifs, zéros négatifs, échelle 0 à 10,
  `Decimal::MAX`, `Option` vides, devises, le `debug_assert!` nouveau.
- **A — Acceptance Auditor** : chaque AC et chaque décision contre le code et les journaux, axes 1, 2, 6 et 7 surtout.

## Rapport

Écris le rapport (le seul fichier permis) dans `/home/gcorbaz/devel/kesh-gate-logs/e4811/review-p2.md` : une section par lentille,
findings classés CRITICAL / HIGH / MEDIUM / LOW, chacun avec **la ligne lue citée** ; puis **la liste des axes exercés et des axes non
exercés**, avec la raison (un « 0 finding » sans cette liste ne compte pas comme passe) ; puis les chemins vérifiés. Tout finding
CRITICAL ou HIGH qui affirme une absence ou un anti-pattern se vérifie d'abord par `/usr/bin/grep -nF`.
