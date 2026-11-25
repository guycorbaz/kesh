# Prompt — validation P1 de la fiche 48-11 « Arrondi au centime » (#1245, #1430), passe complète

Tu valides la **fiche** de la story **48-11** de Kesh (comptabilité suisse ; Rust/Axum, MariaDB, SvelteKit), release **0.20.0** (coupure
le jeudi 2026-11-26 au soir). Kesh tient les livres réels du Project Lead, qui a **déjà émis de vraies factures** : rien de ce qui existe
(facture, avoir, lignes, totaux, TVA, écritures, PDF) ne doit changer. **Aucune migration.** Première passe : **complète**, contexte frais,
modèle **Opus ou Sonnet** (jamais Haiku). Langue du rapport : français.

Worktree `/home/gcorbaz/devel/kesh-48-11`, branche `story/48-11-arrondi-au-centime`, partie de `epic/48-corrections-0-20-0` à
`08c47a6cc`. Fiche : `_bmad-output/implementation-artifacts/48-11-arrondi-au-centime.md`. Journaux d'inventaire (lecture seule) :
`/home/gcorbaz/devel/kesh-gate-logs/e4811/` (`inv-*.txt`, `garde-simulation-*.txt`).

## À lire d'abord

1. `CLAUDE.md` du worktree (§ *Review Iteration Rule*, *Inventorier les sites NON RÉSOLUS*, *Migration breaking policy*, *Priorités des
   défauts*) ; `/home/gcorbaz/devel/kesh-gate-logs/prompts/e0200-fiche.md` et `e0200-commun.md`.
2. Le fichier d'épique `_bmad-output/planning-artifacts/epic-48-corrections-0-20-0.md`, § *48-11* et le tableau des fichiers (ils font foi
   pour le périmètre), et `release-0.20.0.md`.
3. Les issues, en lecture seule : `gh issue view 1245 --json number,title,labels,milestone,body,comments`, idem 1430, 813, 822.
4. La fiche en entier.

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

- **A1 — périmètre** contre le fichier d'épique : sites nommés au kickoff, sites « à trier », fichiers déclarés ; tout écart (sites
  ajoutés : `company_invoice_settings.rs`, `balance_sheet.rs` ; décision D5 sur l'export global) est-il dit et justifié ?
- **A2 — prémisses relues au code** : chaque `chemin:ligne` cité est-il juste à `08c47a6cc` (ligne copiée) ? Le troncage de
  `rust_decimal` 1.43 (`~/.cargo/registry/src/*/rust_decimal-1.43.0/src/str.rs`, `decimal.rs` : `Display`, `round_dp`, `rescale`) est-il
  bien établi, signe de `-0.0040` compris ? Le mode de big.js côté écran ?
- **A3 — inventaire par le symptôme** : rejoue R1 à R5 (commandes de la fiche) et cherche ce qu'elles **omettent** : un formatage de
  montant sans arrondi par une autre forme (`write!`, `format_args!`, précision `.*`, `to_string` puis découpe, `Decimal::to_f64`, montant
  passé en `f64` puis `{:.2}`, `printpdf`/`csv` qui sérialiseraient un `Decimal` eux-mêmes, `serde` d'un montant dans un fichier produit).
  L'ensemble est-il **clos** (chaque site corrigé ou assumé avec une raison vérifiable) ? Les décomptes (30 occurrences / 10 fichiers avant,
  23 / 8 après ; appels par fonction) se recomptent-ils ?
- **A4 — la garde (D6)** : n'est-elle pas vraie par construction ? La regex attrape-t-elle toutes les formes (largeur, alignement,
  précision nommée, `.*`, chaîne seule sur sa ligne) ? La sonde positive prouve-t-elle quelque chose ? Les mutations M4 à M6 la font-elles
  rougir ? Les angles morts sont-ils tous écrits ?
- **A5 — AC constatables** : chaque AC dit-il une commande ou un test, une valeur attendue, un fichier ? AC5 (PDF par la route) est-il
  tenable ?
- **A6 — preuve du rouge** : chaque test neuf rougit-il sans le correctif (ou par une mutation nommée, un journal chacune) ? Les filets
  sont-ils déclarés ?
- **A7 — factures réelles** : la fiche garantit-elle que `kesh-qrbill`, `invoice_pdf_service.rs` et les données ne bougent pas (AC10) ?
  L'inventaire a-t-il raison de dire que le PDF des factures n'a pas le défaut ?
- **A8 — manuel, PDF compris** : `docs/manual/fr/admin-manual.tex:3915` et D8 ; cherche dans **les quatre langues**, `.tex` **et PDF**
  (`pdftotext docs/manual/<l>/<f>.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o '.\{150\}1245.\{150\}'`, et `tronqu|truncat|abgeschnitten|
  tronca`) toute phrase que la story rend fausse.
- **A9 — décisions** : chaque D a-t-elle son option écartée ? D5 (arrondir l'export global plutôt que l'écrire à l'échelle de la colonne)
  est-elle correctement soumise à Guy (Q1) ? #1430 se ferme-t-elle vraiment (D2, AC8) ?
- **A10 — splitting et modules** : le décompte (quatre modules, règle de la 48-15) est-il juste ?
- **A11 — note de version, Dev Notes** : numéros de release, jamais d'épique ; `refs #N` ; issues à ouvrir I1, I2 fondées au code.

## Interdits

Lecture seule. N'écris **aucun** fichier du worktree ni du dépôt principal ; aucun `Edit`/`Write` hors de ton rapport. Aucun `cargo`,
`npm`, `make`, `docker`, `mariadb`, `sqlx`, aucun `scripts/*` (en particulier ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`,
ni `scripts/gate-ci.sh`, ni `scripts/mem-guard.sh`). Aucun `git commit`, `git push`, `git checkout`, `git stash`, `git merge`. Aucun `gh`
qui écrit (`issue create/edit/comment/close`, `pr …`). Rien sous `~/kesh/`. `python3 -I` en lecture pour recompter est admis.

## Rapport

Écris **un seul fichier** : `/home/gcorbaz/devel/kesh-gate-logs/e4811/validation-p1.md`, avec :
- le **verdict chiffré** : CRITICAL / HIGH / MEDIUM / LOW ;
- chaque finding numéroté `V1-n`, sévérité, prémisse **citée au code** (`chemin:ligne`, ligne copiée), remède proposé ;
- la liste des **axes exercés** et **non exercés** (A1 à A11), avec ce qui a été lancé pour chacun ; un « 0 finding » sans cette liste ne
  compte pas comme passe ;
- les commandes lancées et leurs sorties utiles.
