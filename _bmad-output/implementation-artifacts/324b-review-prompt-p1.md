# Prompt — revue de code P1 de la Story 32-4b (rollout des lectures : factures clientes, échéancier, règlements, factures fournisseur), trois lentilles, Sonnet

Tu relis le développement de la **Story 32-4b** de Kesh (comptabilité suisse, Rust/Axum, MariaDB) : *la couche de
services — rollout des lectures : factures clientes, échéancier, règlements, factures fournisseur*. Worktree
`/home/gcorbaz/devel/kesh-32-4b`, branche `story/32-4b`, partie de la fiche validée `9f29880f` (3 passes de validation,
terminée), elle-même sur `main` `592ee430`. Le diff à relire, **aplati** (un seul diff, pas la suite des commits) :
`git -C /home/gcorbaz/devel/kesh-32-4b diff 9f29880f...HEAD`. Langue du rapport : français. Modèle : **Sonnet**.
Contexte frais : ne t'appuie sur aucune passe antérieure. Trois lentilles, **une par agent** : B, E, A.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/32-4b-services-factures.md` — *Les leçons de la 32-4a*, *Ce que le code dit*
(faits 1 à 10), *Le rollout*, *Inventaire clos des lectures*, AC1 à AC18, tâches T0 à T7, *Ce que les gardes ne
voient pas*, *Dev Notes*, et le **Dev Agent Record** et l'entrée « Développement » du **Change Log** — et le registre
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-324b.md` (C-32-4b-1 à C-32-4b-18, plus les entrées du
développement, C-32-4b-19 et suivantes). Le motif que ce rollout applique : `crates/kesh-api/src/services/mod.rs`
(règles R1 à R8) ; le modèle direct : la 32-4a (`services/{ecritures,postes_ouverts,carnet}.rs`). Le Dev Agent Record
déclare des gates, des rouges et des mutations : **ne les prends pas pour argent comptant**, recoupe-les avec les
journaux `/home/gcorbaz/devel/kesh-gate-logs/324b-*.log`, `324b-mut-*.{log,diff}`, `324b-mutations-resultats.txt`,
`324b-mutations-campagne.log`, `324b-ac14-fermeture.txt` et `324b-ac14-verrous-et-macros.txt` (lecture seule). La story
n'ajoute aucun contrôle visible : pas de capture attendue.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans le scratchpad, ni ailleurs ; **aucune redirection** (`>`, `>>`, `tee`), pas même vers
  un fichier « jetable » : compare par des commandes enchaînées (pipe), lis les PDF vers la sortie standard seulement.
  Un fichier créé par erreur se **déclare** au rapport ; il ne se supprime pas (supprimer, c'est écrire).
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`, `add`).
- **N'exécute rien qui écrit, compile ou exécute du code** : ni **`scripts/prepare-release.sh`**, ni
  `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni `scripts/install-hooks.sh`, ni aucun script de
  `kesh-gate-logs` ou du scratchpad (pas même `324b-mutations-pilote.py`, `324b-ac14-fermeture.py` ni
  `324b-detecteur-state-pool.py` : lis-les, ne les lance pas), ni `make`, ni `latexmk`, ni `cargo` (aucune
  sous-commande, `cargo fmt` compris), ni `rustc`, ni `npm`/`npx`/`node`, ni `docker`, ni `mysql`/`mariadb`, ni `sqlx`,
  ni `curl`, ni `python3`, ni `sed -i`, ni `perl`.
- **Aucune commande `gh`**, sous aucune forme (pas même `gh issue view`).
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection),
  `cat`, `ls`, `find`, `wc`, `diff` sur des substitutions de processus, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée.
- **Données réelles** : aucune ; les fixtures sont synthétiques. N'ouvre rien sous `~/kesh-test/`.

## Lentille B — Blind (le diff seul, sans la fiche)

Lis le diff aplati **avant** la fiche, comme un relecteur qui ne connaît pas l'intention. L'état d'avant est
`git show 592ee430:<fichier>`.

1. `crates/kesh-api/src/services/factures.rs`, `lister` : l'ordre `exiger` → société → `limit` → `offset` → `search` →
   `status` → `dateFrom > dateTo` est-il **exactement** celui qu'a retiré `list_invoices`
   (`git show 592ee430:crates/kesh-api/src/routes/invoices.rs`, l. 597-677) ? Les cinq messages sont-ils identiques au
   caractère près (les aides `limite`, `decalage`, `recherche`, `periode` reproduisent-elles le texte et
   l'interpolation `{LIMITE_MAX}`, `{RECHERCHE_MAX}`) ? Les défauts (`unwrap_or_default()`, `SortDirection::Desc`,
   `payment_status`/`due_before` à `None`, `paused` recopié) ? La société passée au dépôt est-elle celle du contexte ?
2. `requete_echeances` contre l'ancien `build_due_dates_query` (même fichier d'avant, l. 1044-1138) : sept refus dans
   l'ordre, plafond `> 1_000_000`, `status` forcé, `payment_status` défaut `Unpaid`, `paused` forcé à `None`, tri
   `DueDate`/`Asc`. Les commentaires B1, B11, B16, B21 et l'invariant D10 ont-ils suivi ? `echeances` : le résumé reçoit-il
   la requête **sans** `payment_status` (B15) ? `tokio::join!` sur la `Lecture` ?
3. `detail` : aucune lecture de société ; `find_by_id_with_lines` sous `acces.company_id()` ; `amount_settled` puis
   `amount_due` ; le réglage d'arrondi lu **pour un brouillon seulement**. Le handler `get_invoice` compose-t-il la même
   réponse qu'avant (`with_settlement`, puis `with_rounding_preview` sur un brouillon) ? `with_draft_rounding_preview`
   n'est-elle plus appelée par lui, et l'est-elle encore par les trois mutants ?
4. `reglements` : société, facture (404), **une** transaction `lecture.pool().begin()` portant `list_for_invoice` et,
   par règlement, `settlement_cancel_blocker`, refermée par `commit` — comme le handler d'avant (l. 1427-1471) ? La
   garde `reglements_lisent_dans_une_transaction` (`mod tests` de `factures.rs`) peut-elle passer **à vide** (corps mal
   délimité par `"\npub "`, motif sans blancs mal écrit, `tx.commit()` présent ailleurs) ? Compare-la à son patron
   `get_journal_entry_reads_in_one_transaction` (`services/ecritures.rs`) : un contrôle du patron a-t-il été
   **relâché** ?
5. `services/factures_fournisseur.rs` : société d'abord, `unwrap_or(50).clamp(1, 200)`, `unwrap_or(0).max(0)`, valeurs
   écrêtées rendues ; `detail` → `get_settlement_view` → `NotFound`. `SupplierInvoiceResponse::from_view`
   (`routes/supplier_invoices.rs`) : la mise en forme est-elle **exactement** celle de l'ancienne
   `load_with_settlement_cancellation` (champ par champ, `source_document` compris) ? Sa signature a-t-elle bougé ?
6. Les six handlers minces : statuts, corps (`ListResponse`, `DueDatesResponse`, `InvoiceResponse`,
   `InvoiceSettlementResponse` champ par champ, `SupplierInvoiceResponse`), ordre des erreurs identiques ? La conversion
   `demande_echeances` est-elle **une** fonction, partagée par la page et l'export ? L'export CSV : sa lecture de société
   reste-t-elle **avant** `requete_echeances`, avec `limit`/`offset` à `None` ? Imports morts ? Les `mod tests` de
   `invoices.rs` et `mod scan_qr_tests` de `supplier_invoices.rs` inchangés (`sed -n '/^#\[cfg(test)\]/,$p'` en pipe sur
   `git show` des deux bornes) ?
7. **Chaque `.pool()` des deux fichiers neufs** (R8 : la revue les lit, aucune garde ne le prouve) : n'emploie-t-on
   `lecture.pool()` qu'à lire ? aucun autre chemin vers le pool ?
8. `crates/kesh-api/tests/services_rollout.rs` : un test peut-il passer **à vide** (un `Ok` attendu qui serait aussi celui
   d'un contrôle absent, une table vide, un témoin qui ne témoigne de rien, une fixture qui n'atteint pas la branche —
   `Cas::Inexistant` qui n'appellerait pas l'entité voulue, la fixture `poser_factures` dont les dix ordres de tri ne
   seraient pas distincts) ? Recalcule à la main les dix ordres de `ORDRES_DES_FACTURES` depuis la table de
   `poser_factures`.
9. `crates/kesh-api/tests/services_motif.rs` : la garde étendue (ii), (iii) — `corps_des_aides`, l'égalité du fichier
   handlers + aides —, (vi), (viii), et la garde (ix) neuve (`usages_interdits_de_state`, `FORMES_PERMISES_DE_STATE`) :
   faux vert possible (une forme permise qui en couvrirait une interdite, `State(state)` compté deux fois, un corps vide,
   une aide d'une ligne qui avalerait le fichier jusqu'au `}` suivant) ?
10. Les textes Q11 (`services/mod.rs`, `poignees.rs`, `lib.rs`, `tests/common/lecture_seule.rs`) : disent-ils juste la
    décision, sans en ajouter ?

## Lentille E — Edge cases

1. Demandes limites au service : `limit` à `i64::MIN`/`i64::MAX`, `offset` à `i64::MIN`/`i64::MAX` (liste et
   échéancier), `search` de blancs Unicode, de 100 caractères multioctets, de 101 caractères dont des blancs en tête ;
   `status` en majuscules ; `dueBefore` seul ; `dateFrom == dateTo == dueBefore`. La liste fournisseur à `limit = 0`,
   `i64::MIN`, `offset = i64::MIN`.
2. Canal MCP : portées vides ; chacune des deux autres seule ; les trois ; un contexte MCP d'une **autre** société sur
   chacune des six lectures (AC12 (b)) — y a-t-il une lecture qui filtrerait par une autre société que celle du contexte
   (`company.id` contre `acces.company_id()`, `facture.company_id` du réglage d'arrondi) ? Un jeton orphelin : quelle
   réponse pour chacune des six (fait 5) ?
3. Lecture seule : quelles branches de la fermeture les fixtures n'atteignent-elles **pas** (motifs d'annulation
   présents, pièce source, lot confirmé, facture fournisseur réglée, filtres des listes) ? Le Dev Agent Record les
   nomme-t-il ? Relève toi-même la fermeture d'au moins **un** chemin transitif
   (`settlement_cancel_blocker_unlinking` → `settlement_entry_cancel_blocker` → …, ou
   `get_settlement_view` → `supplier_invoice_cancel_blocker` → …) et confronte-la à `324b-ac14-fermeture.txt` : un
   appel manque-t-il (le script ne résout que les appels qualifiés par module, les noms du même module et les `use` de
   `crate::repositories`/`crate::util::search` — lis-le) ? Un `INSERT`, `UPDATE `, `DELETE`, `FOR UPDATE`, `LOCK IN
   SHARE MODE` ou `get_or_create` y est-il atteignable ?
4. Les **mutations** de l'AC15 (100, dont M21d équivalente) : chaque mutation déclarée tuée l'est-elle par une
   **assertion nommée** (`324b-mutations-resultats.txt`, `324b-mut-*.log`, diffs `324b-mut-*.diff`) — et non par une
   erreur de compilation ou de montage ? Le diff de chaque mutation est-il celui que décrit la fiche ? M21d est-elle
   vraiment équivalente (`kesh-db/src/repositories/invoices.rs:937-1024`) ? Une mutation plausible **non jouée**
   survivrait-elle — p. ex. `detail` qui passerait `acces.user_id()` au lieu de `company_id()`, `round_to_5_centimes`
   lu sur `acces.company_id()` au lieu de `facture.company_id`, `echeances` qui recevrait la requête de la page pour le
   résumé, `from_view` qui perdrait `source_document`, la conversion `PausedParam → PausedFilter` permutée ? Raisonne
   sur le code, n'exécute rien.
5. Le comportement **par la route** est-il tenu là où seul le service est testé, et inversement : `paused` (liste),
   `paymentStatus` (échéancier) traversent-ils les enums de route (`From`) — un `From` qui permuterait deux variantes
   serait-il vu ?

## Lentille A — Acceptance (chaque AC contre le dépôt)

Pour chaque AC1 à AC18, rends « tenu », « non tenu » ou « non vérifiable en lecture seule », avec la preuve :

- **AC1** : deux modules `pub mod` (ordre alphabétique) ; `grep -rnE "axum|AppState|CurrentUser" crates/kesh-api/src/services | grep -vE '^[^:]+:[0-9]+:[[:space:]]*//'`
  → exactement les lignes de `RESIDUS_VI` ; dix fichiers ; aucun `forbid` redéclaré ; `\bTrace\b|\btrace\b` absent
  des lignes de code des deux fichiers ; doc-comments ; `Portee::InvoicesRead` documentée.
- **AC2** : `exiger` en première instruction de chacun des six services ; `EXIGENCE_*` aux valeurs du *Rollout*.
- **AC3-AC7** : chaque service contre son AC (ordre, défauts, refus et non écrêtage pour les listes clientes, écrêtage
  pour le fournisseur, erreurs, types rendus, garde de transaction).
- **AC8** : les quatre contrôles (`grep` des trois constantes, `clamp(`, compte des `AppError::Validation` → 22,
  `grep -rnF 'build_due_dates_query' crates/kesh-api/src`) recopiés au Dev Agent Record — refais-les.
- **AC9** : `git diff --name-only $(git merge-base HEAD origin/main) HEAD -- crates/kesh-api/tests` → exactement les
  quatre fichiers ; les lignes ôtées de `services_rollout.rs` (`git diff 592ee430 HEAD -- … | grep -E '^-[^-]'`)
  sont-elles toutes de celles que l'AC9 (2) permet ? Les fichiers et comptes réellement exécutés recopiés au Dev Agent
  Record, contre les journaux.
- **AC10-AC12** : chaque cas présent dans `services_rollout.rs`, assertion nommée ; les « sans objet » dits au Dev Agent
  Record ; chaque filtre, chaque tri, chaque défaut de l'AC11 (d)-(f) a sa ligne **et** sa mutation (leçon 1).
- **AC13** : (ii) douze handlers ; (iii) cinq lignes, 13 / 15 et 7 / 8, aides, compte du fichier ; (vi) dix ; (viii)
  cinq ; (ix) et ses témoins ; (i), (v-1), (v-2), (vii) inchangées. Refais le relevé 30 = 28 + 2 et 11 = 11 + 0 à la
  main sur un fichier au moins (`awk` en pipe).
- **AC14** : inventaire au doc-comment de `services/mod.rs` et au Dev Agent Record, contre `324b-ac14-fermeture.txt`.
- **AC15** : mutations (cf. lentille E, point 4) ; décompte 100 = 16 + 7 + 15 + 44 + 4 + 4 + 5 + 5.
- **AC16** : `kesh-db` non touché (`git diff 592ee430...HEAD --stat -- crates/kesh-db` → vide) ; chiffres du Dev Agent
  Record (gate ciblé, gate complet, frontend, E2E et liste nominative des échecs) contre les journaux ; gate et E2E au
  **dernier commit de code** (révision des journaux contre `git log --oneline 592ee430..HEAD`) ; échecs E2E jugés
  fichier par fichier contre `docs/testing.md` § *Les échecs attendus*.
- **AC17** : **le manuel et l'API externe** — `docs/api-external.md` l. 215, 497, 552-566, 667, 723-757 contre le code
  déplacé (champs de la fiche fournisseur, `null` = non calculé, règlements) ; `docs/manual/fr/*.tex` **et** les PDF
  aplatis (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, puis `admin-manual.pdf`) : aucun
  texte ne décrit autrement les bornes des listes de factures ni l'accès à l'échéancier ; la note de
  `docs/MULTI-TENANT-SCOPING-PATTERNS.md` ; le contrôle Q11 sur `crates/` (motif de l'AC17 (3)) → rien ; les
  doc-comments de module des deux fichiers de routes.
- **AC18** : `grep -nE 'let \(_?company_id, _?admin_id\) = seed_base' crates/kesh-api/tests/invoice_echeancier_e2e.rs`
  → rien ; 8 lignes de diff.
- **Décomptes** : tests ajoutés recomptés (`git show 592ee430:<f>` contre `HEAD`, `grep -cE '#\[(sqlx::|tokio::)?test'`),
  périmètre déclaré.
- **Messages de commit** : `git log 592ee430..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`
  doit ne rien rendre ; chaque commit du développement porte `refs #723`.
- **Registre et statut** : clé `32-4b-services-factures` de `sprint-status.yaml` à `review` ; registre complété par
  union (C-32-4b-19 et suivantes), sans réécrire C-32-4b-1 à 18 ; Change Log : l'entrée « Développement » ajoutée,
  les entrées des validations P1 à P3 intactes.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/324b-review-p1-<B|E|A>.md` (selon ta lentille) :

- bilan en tête : CRITICAL / HIGH / MEDIUM / LOW ;
- findings numérotés `B1…`/`E1…`/`A1…`, chacun avec sévérité, fichier:ligne, **preuve copiée** (sortie `grep -nF`,
  `sed -n` ou `git diff`), correctif proposé ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été** — un « 0 finding » sans cette liste ne compte
  pas comme passe ;
- rien d'autre n'est écrit nulle part.
