# Prompt — revue de code P1 de la Story 43-1a, trois lentilles (B, E, A)

Tu relis la **Story 43-1a** de Kesh (comptabilité suisse, Rust/Axum, MariaDB 10.11), release **0.19.0** : **un règlement
pour plusieurs factures** (#823) — la migration `20261118000001_grouped_settlements.sql` (non breaking), le lettrage par
ligne (deux familles), le **règlement groupé client** avec la règle de la différence (décision de Guy du 2026-11-17 :
écart admis dans les deux sens au compte choisi, plafond seul refusé, arrondi automatique sous `centime(n × 0.005)`),
le **dé-rapprochement** d'un groupe, la vue, et les garde-fous (inventaire de `journal_entries_modification.rs`,
`TABLES_TO_TRUNCATE`, P5, P6, P8). Le côté fournisseur (`pay_supplier_invoices_in_tx`, `unimplemented!`) est la 43-1b,
l'API la 43-2 : **hors périmètre**.

Worktree `/home/gcorbaz/devel/kesh-43-1a`, branche `story/43-1a-reglement-groupe-modele`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-43-1a diff 7d8859aa3 HEAD -- crates docs` (`7d8859aa3` = dernier commit des fiches
avant le développement). Langue du rapport : français. Passe **complète** (première passe), contexte frais, modèle
**Sonnet** ou **Opus** (jamais Haiku pour une P1).

## Ce qu'il faut lire d'abord

- La fiche `_bmad-output/implementation-artifacts/43-1a-reglement-groupe-modele-client.md` en entier — ⚠️ **D4 bis y est
  remplacée** par la décision de Guy : lis le registre `_bmad-output/implementation-artifacts/e0190-choix-autonomes-e431a.md`
  (C-e431a-1 à 12), qui fait foi pour la différence, les refus et le contrat, puis
  `/home/gcorbaz/devel/kesh-gate-logs/e43/decision-difference.md` et `/home/gcorbaz/devel/kesh-gate-logs/e43/t3.md`.
- Le fichier d'épique `_bmad-output/planning-artifacts/epic-43-transaction-plusieurs-factures.md` (C-43-1 à C-43-24).
- Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le avec les journaux hors dépôt (lecture seule) sous
  `/home/gcorbaz/devel/kesh-gate-logs/e431a/` : `t0-mariadb.log`, `t1-cible.log`, `t5-cible-*.log`, `test-fast-review.log`,
  `gate-ci-review.log`, `mut/*.log` (le script `mut/mutations.py` est à **lire**, jamais à exécuter).

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS.

## Les propriétés centrales

1. **La méthode de l'unitaire** (`accept_one_invoice`, `crates/kesh-api/src/routes/reconciliation.rs`) : aucune lecture
   verrouillante des factures en amont ; version et reste dû lus sur l'instantané ; `UPDATE invoices … AND version = ?`
   par identifiant croissant, refus `race_during_update`. Deux écarts seulement (C-43-19).
2. **Point de sauvegarde propre (D6)** : tout `Ok(Err(refus))` est rendu après `ROLLBACK TO SAVEPOINT grouped_settlement` ;
   un `Err(DbError)` jamais traduit en refus ; un 1305 remonte en `Err` (`is_savepoint_lost`).
3. **Attribution ligne → facture par position (D2)**, lettrage par ligne (AC4), une clé par facture, banque et
   différences non lettrées.
4. **La décomposition** (`group_difference`) : `difference = rounding + gap` ; arrondi automatique ; contrôles du compte
   d'écart dans l'ordre banque, manquant, non imputable, tiers ; l'écriture équilibrée dans tous les cas.
5. **Le dé-rapprochement** défait le groupe en entier, tous les refus avant toute écriture, dans l'ordre de verrous de
   l'unitaire ; l'écriture contre-passée après le retrait de ses propriétaires (D4) ; audit avec l'acteur (clé d'API).
6. **Ce qui doit rester vrai** : un règlement unitaire se crée, se lettre, s'annule et se dé-rapproche comme en 0.18.0
   (`classify`, `entry_cancel_outlook`, `get_view` pour les trois familles d'avant).
7. **Migration** : non breaking au sens du ⚠️ de P1 ; P5 (ligne et compteurs recomptés), P6 (`total - 58`, frontière 34),
   P7 (DDL seul), P8 (checksum à `migrations.sha384`, migration modifiée une fois avant publication — C-e431a-12.4).

## Les trois lentilles

- **B (Blind Hunter)** — le diff seul, sans la fiche : défauts de code, SQL, verrous, erreurs avalées, `unwrap`.
  Rapport : `/home/gcorbaz/devel/kesh-gate-logs/e431a/review-p1-B.md`.
- **E (Edge Case Hunter)** — chaque branche et chaque borne : bornes 2/50, doublons, montant nul, devise, facture
  partiellement réglée, sous-centimes, arrondi automatique à `n` factures, plafond 0 et 100, compte d'écart = arrondi,
  exercice clos, période verrouillée, transaction rapprochée entre-temps, dé-rapprochement concurrent, groupe fournisseur
  (classement sans branche : `Invariant`), vue d'une écriture sans ligne. Rapport : `…/review-p1-E.md`.
- **A (Acceptance Auditor)** — AC1 à AC13 de la fiche, sous la règle du registre ; inventaire AC9 relu ; **manuel
  d'administration** `docs/manual/fr/admin-manual.tex`, § *Passer à la 0.19.0* (`.tex` seulement : le PDF se régénère
  à l'intégration — contrôle sans objet ici) ; captures : sans objet (aucun contrôle visible dans cette story). Rapport :
  `…/review-p1-A.md`.

Chaque rapport : findings classés CRITICAL / HIGH / MEDIUM / LOW, chacun avec le fichier et la ligne lus (sortie de
`grep -nF` copiée pour toute affirmation de présence ou d'absence), et **la liste des axes exercés et non exercés**.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire** ;
  **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, ni `pkill`, ni `killall`. Aucun `git` qui écrit (`checkout`, `switch`, `restore`, `stash`, `commit`).
- **Rien qui compile ou exécute** : ni `cargo`, `python3`, `perl`, `node`, `bash -c`, `jq`, `docker`, `mariadb`, `curl`,
  `make`, `claude`, `sed -i` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`,
  `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs`. **Aucune commande `gh`**, aucun accès réseau.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep`, `sed -n`, `awk` (sans
  redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`. Contrôle d'un PDF,
  s'il le fallait, sous la seule forme `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`.
