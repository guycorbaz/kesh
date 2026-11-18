# Prompt — revue de code P1 de la Story 43-1b, trois lentilles (B, E, A)

Tu relis la **Story 43-1b** de Kesh (comptabilité suisse, Rust/Axum, MariaDB 10.11), release **0.19.0** : **le
règlement groupé côté fournisseurs** (#823). Un débit du relevé règle plusieurs factures fournisseur, de tiers
éventuellement différents, en **une** écriture : une ligne de banque (au crédit), une ligne de dette par facture (au
débit), au plus une ligne d'arrondi et une ligne d'écart (compte choisi par l'utilisateur). S'y ajoutent la **vue**
d'un groupe fournisseur (`view_members`, `difference_account_of`), son **dé-rapprochement** (le bras fournisseur de
`grouped_settlements::cancel_in_tx`, qui remplace l'`Invariant` laissé par la 43-1a), et le **détachement** d'une
facture du groupe par son annulation. Le côté client (43-1a), l'API (43-2) et l'écran (43-3) sont **hors périmètre**,
sauf les lignes du bras client que la 43-1b touche (registre C-e431b-10).

Worktree `/home/gcorbaz/devel/kesh-43-1b`, branche `story/43-1b-reglement-groupe-fournisseurs`. Le diff à relire,
**aplati** : `git -C /home/gcorbaz/devel/kesh-43-1b diff 338a136f1 HEAD -- crates` (`338a136f1` = la base : tête de la
43-1a avec les fiches fusionnées, avant le développement). Langue du rapport : français. Passe **complète** (première
passe), contexte frais, modèle **Sonnet** ou **Opus** (jamais Haiku pour une P1).

## Ce qu'il faut lire d'abord

- La fiche `_bmad-output/implementation-artifacts/43-1b-reglement-groupe-fournisseur.md` en entier (AC1 à AC8, AC6 bis).
- Le registre de la story `_bmad-output/implementation-artifacts/e0190-choix-autonomes-e431b.md` (C-e431b-1 à 11) —
  plusieurs choix **s'écartent de la lettre de la fiche** (C-e431b-1 binaire de test à part, C-e431b-3 ordre des refus
  de la fonction réemployée, C-e431b-4 compte d'écart relu sur l'écriture et non sur les membres, C-e431b-8 code
  d'erreur de l'AC8) : juge-les, ne les prends pas pour acquis.
- Le contrat de la 43-1a : `/home/gcorbaz/devel/kesh-gate-logs/e43/t3.md`, et son registre
  `_bmad-output/implementation-artifacts/e0190-choix-autonomes-e431a.md` (C-e431a-5 à 9, 14 à 16).
- La règle de la différence : `/home/gcorbaz/devel/kesh-gate-logs/e43/decision-difference.md` (le tableau fait foi).
- Le fichier d'épique `_bmad-output/planning-artifacts/epic-43-transaction-plusieurs-factures.md` (C-43-1 à C-43-26).
- Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le avec les journaux hors dépôt (lecture seule) sous
  `/home/gcorbaz/devel/kesh-gate-logs/e431b/` : `rouges.log`, `t1-cible-1.log`, `t1-cible-2.log`,
  `test-fast-review.log`, `gate-ci-review.log`, `mut/campagne.log` (le script `mut/campagne.py` est à **lire**, jamais
  à exécuter).

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS.

## Les propriétés centrales

1. **La méthode de l'unitaire fournisseur** (`accept_one_supplier_invoice`, `crates/kesh-api/src/routes/reconciliation.rs`) :
   chaque facture verrouillée et admise par `supplier_invoices::lock_for_bank_settlement_in_tx` (**réemployée**, non
   recopiée), par identifiant croissant ; `UPDATE supplier_invoices … AND version = ? AND status = 'open'` à la version
   lue sous ce verrou, `rows_affected() == 1` ; transaction `UPDATE … AND version = ?` ; `recheck_open_locked_in_tx`
   en dernière acquisition. Ordre des verrous face à `pay_in_tx` (fiche) et `create_batch` : facture, compte, lot.
2. **Point de sauvegarde propre** (D6 de la 43-1a) : tout `Ok(Err(refus))` rendu après `ROLLBACK TO SAVEPOINT
   grouped_settlement`, `RELEASE` au succès ; un `Err(DbError)` jamais traduit en refus.
3. **L'écriture** : banque au crédit de |montant| ; dette par facture au débit de son `total_amount` (contrôlé égal à la
   ligne de crédit de l'achat, C-e431b-2), projet de la facture sans validation ; `r` et `e` du côté qui équilibre
   (fournisseur : positifs au débit) ; attribution ligne → facture par position ; `settlement_journal_entry_line_id`
   posée ; lettrage par facture (son achat, sa ligne).
4. **La différence** : la **même** `group_difference` que le client ; ordre des refus — plafond d'abord, puis compte
   d'arrondi, puis compte d'écart requis, puis banque, manquant/archivé, non imputable, dette du groupe.
5. **La vue** (AC6 bis) : membres fournisseur (famille, numéro interne à défaut celui du fournisseur, `total_amount`)
   par identifiant croissant ; compte d'écart relu par position, y compris après un **détachement** (C-e431b-4 : le
   bloc des dettes lu sur l'écriture) ; motifs d'annulation : tête fournisseur par facture, queue une fois.
6. **Le dé-rapprochement** : refus avant toute écriture (`Some(true)` → `ReconciliationCancelChanged`, jamais un retrait
   seul) ; lien défait, délettrage, factures rouvertes par `CLEAR_SETTLEMENT_SQL` (ligne comprise) à la version tenue,
   contre-passation de l'écriture **entière** (ligne d'une facture détachée comprise), audit avec l'acteur (clé d'API).
7. **Le détachement** (AC7) : `supplier_invoices::cancel_in_tx` vide la ligne avec l'écriture, l'audit nomme la ligne ;
   l'écriture redevient `entry` quand plus aucune facture n'y est attachée.
8. **Ce qui doit rester vrai** : le paiement, l'annulation du règlement et le dé-rapprochement d'un règlement
   fournisseur **unitaire** (la tête extraite `supplier_settlement_cancel_head`, C-e431b-5), et tout le bras client
   (aiguillage seulement, C-e431b-10).
9. **Inventaire du compte clos** (`crates/kesh-api/tests/bank_account_closed_inventory.rs`) : les deux fonctions
   neuves classées, décomptes anti-muets recomptés (41 → 45 sites, G 14 → 16, revérifications 9 → 11) — et
   l'`ensure_open_in_tx` ajouté au paiement pour le satisfaire (C-e431b-6) : légitime ou artefact ?

## Les trois lentilles

- **B (Blind Hunter)** — le diff seul, sans la fiche : défauts de code, SQL (portée par société, index, verrous),
  erreurs avalées, `unwrap`, conversions de signe, duplication avec le bras client.
  Rapport : `/home/gcorbaz/devel/kesh-gate-logs/e431b/review-p1-B.md`.
- **E (Edge Case Hunter)** — chaque branche et chaque borne : bornes 2/50, doublons, crédit au lieu d'un débit, montant
  nul, devise, facture `paid`/`cancelled`/`draft`, lot `generated` sur une facture du groupe ou sur une **autre** au
  montant du débit (garde (d)), date au-delà de la tolérance de 7 jours, totaux à quatre décimales (`r ≠ 0`), `e` dans
  les deux sens, plafond 0 et 100, compte d'écart = compte d'arrondi, = dette, = banque, archivé, non imputable ; dette
  d'une facture = banque ; exercice clos, période verrouillée ; dé-rapprochement après détachement de la première, de
  la dernière, de toutes les factures ; facture détachée dont la dette est sur un autre compte (angle mort de
  C-e431b-4) ; concurrence avec `pay`, `create_batch`, `cancel` (détachement) et une clôture.
  Rapport : `/home/gcorbaz/devel/kesh-gate-logs/e431b/review-p1-E.md`.
- **A (Acceptance Auditor)** — AC1 à AC8 et AC6 bis de la fiche, contre le code et les tests ; les mutations demandées
  par la fiche (*Tests*) contre `mut/campagne.log` ; la *Note de version*. **Manuels** : cette story n'a ni route ni
  écran (le manuel utilisateur est la 43-3) ; vérifie qu'aucun `docs/manual/fr/*.tex` ne décrit déjà le paiement
  groupé fournisseur autrement que le code (`git grep -n "plusieurs factures" -- docs/manual`) ; **PDF** : aucun n'est
  touché par la story — contrôle sans objet, à déclarer. **Captures** : sans objet (aucun contrôle visible).
  Rapport : `/home/gcorbaz/devel/kesh-gate-logs/e431b/review-p1-A.md`.

Chaque rapport : findings classés CRITICAL / HIGH / MEDIUM / LOW, chacun avec le fichier et la ligne lus (sortie de
`grep -nF` copiée pour toute affirmation de présence ou d'absence), et **la liste des axes exercés et non exercés**.
Un « 0 finding » sans cette liste ne compte pas.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire** ;
  **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, ni `pkill`, ni `killall`. Aucun `git` qui écrit (`checkout`, `switch`, `restore`, `stash`, `commit`).
- **Rien qui compile ou exécute** : ni `cargo`, `python3`, `perl`, `node`, `bash -c`, `jq`, `docker`, `mariadb`, `curl`,
  `make`, `claude`, `sed -i` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`,
  `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` (dont `e431b/reset-base.sh` et
  `e431b/mut/campagne.py`). **Aucune commande `gh`**, aucun accès réseau.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep`, `sed -n`, `awk` (sans
  redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`. Contrôle d'un PDF,
  s'il le fallait, sous la seule forme `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`.
