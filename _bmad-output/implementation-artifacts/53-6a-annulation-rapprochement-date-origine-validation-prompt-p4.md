# Prompt — validation P4 de la fiche 53-6a (#1674, P1), passe CIBLÉE

Tu valides la **fiche** de la story **53-6a** de Kesh (comptabilité suisse en production ; Rust/Axum, MariaDB, SvelteKit), Epic 53.
Quatrième passe : **ciblée**, une seule lentille, contexte frais, modèle **Haiku** (P1 Opus 0/1/1/10 ; P2 Sonnet 0/0/1/6 ; P3 Opus 0/0/1/4, tout remédié).
Langue : français. Worktree `/home/gcorbaz/devel/kesh-53-6`, branche `story/53-6-annulation-rapprochement-exercice`. Fiche :
`_bmad-output/implementation-artifacts/53-6a-annulation-rapprochement-date-origine.md`. Rapport de la P3 : `/home/gcorbaz/devel/kesh-gate-logs/e53/53-6a-validation-p3.md`
(à lire). Code de référence : **`origin/main` à `0e99cd2db`**, lu par `git show origin/main:<chemin>` — jamais le code du worktree, qui est en retard sur `main`.

## Le seul objet de la passe
Le commit de la remédiation P3 : `C=$(git log -1 --format=%H -- _bmad-output/implementation-artifacts/53-6a-annulation-rapprochement-date-origine-validation-prompt-p4.md)`,
diff aplati : `git show $C -- _bmad-output/implementation-artifacts/53-6a-annulation-rapprochement-date-origine.md > /tmp/claude-1000/53-6a-p4.diff`.
Tu ne relis **que** les lignes ajoutées par ce diff (D5 « états par bras », AC-9 « Fixture », AC-13 « Validation P3 (L3) », AC-14 « Site ajouté à la validation P3 »,
annotation des décomptes, Change Log P3).

## Lentille unique : chaque affirmation ajoutée est-elle vraie au code ?
Pour chaque affirmation, `grep -nF` sur le fichier de `origin/main` (extrait sous `/tmp/claude-1000/`), ligne copiée :
1. R3 n'a qu'une écriture de règlement (`letterings.rs`, `discover_supplier_invoice_document`) ; les groupes soldent chaque facture (`grouped_settlements.rs`,
   « garde un reste dû ») ; R2 admet le règlement partiel (aucun groupe si la somme n'est pas nulle, `sync_document_in_tx`).
2. Tolérances : `PAYMENT_BEFORE_INVOICE_TOLERANCE_DAYS` = 1 (`kesh-db/src/repositories/reconciliation.rs:288`), `SUPPLIER_…` = 7 (`:493`), contrôlées au groupe
   (`grouped_settlements.rs:653`, `:2417`) ; `line_open_in_period` (`letterings.rs:609`, `entry_date > borne`). Le montage « facture datée après le paiement, borne sur la
   date du règlement » laisse-t-il bien le groupe non figé et l'écriture sous la borne ? Le chemin d'**acceptation** unitaire (client et fournisseur) applique-t-il la même
   fenêtre (`routes/reconciliation.rs:2754`, `:3642`) ?
3. Les tests nommés existent aux lignes dites et ont l'état dit : `reconciliation_e2e.rs:7519`, `grouped_settlements.rs:1823`, `supplier_settlement_cancel_e2e.rs:806`
   (passe par la fiche, non par le dé-rapprochement), `invoice_settlement.rs` `RANGS` `:1213` et `la_precedence_de_l_annulation_lecture_et_ecriture` `:1419` (règlement de
   40.00 sur 100.00, verrou à `D − 10` pour le 2 bis), `grouped_settlements_supplier.rs` sans test du lettrage figé.
4. Le décompte annoté : rejoue `git grep -c "reverse_in_tx(\|reverse_owned_in_tx(\|cancel_settlement_in_tx(\|cancel_in_tx(\|settlement_cancel_blocker(\|entry_cancel_outlook" origin/main -- 'crates/*/tests/*'`
   et compare (14 fichiers, 73 lignes ; total = somme).
5. `user-manual.tex:4135-4141` et `:1950-1956` de `origin/main` sont bien les deux puces du lettrage figé (dé-rapprochement, puis règlement).
6. Le diff contredit-il une autre partie de la fiche (M6, notes de version, Dev Notes `:455` sur l'attendu `DbError::PeriodLocked`) ?

Un finding CRITICAL ou HIGH qui affirme une absence se prouve par `grep -nF` cité ; sinon il est écarté.

## Ce que tu rends
Rapport français : par finding, sévérité, `chemin:ligne` et ligne copiée, ce qui est faux, la correction. Puis les points 1 à 6 **exercés et non exercés**
(un « 0 finding » sans cette liste ne compte pas), puis le décompte C/H/M/L.

## Interdits
Aucune écriture (ni worktree, ni dépôt principal, ni `~/kesh/`) hors de `/tmp/claude-1000/` ; pas de `git commit`, `push`, `checkout`, `merge`, `stash` ; pas de `gh` ;
ni `cargo`, `npm`, `scripts/test-fast.sh`, `gate-ci.sh`, `prepare-release.sh`, `recette-montee.sh`, `mem-guard.sh`, ni script qui écrit ; ni `docker`, `sqlx migrate`,
`mariadb` ; jamais la base du NAS ; jamais `pkill`, `killall`, ni `kill` par motif.
