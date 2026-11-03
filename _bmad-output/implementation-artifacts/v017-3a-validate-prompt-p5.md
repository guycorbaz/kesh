# Prompt — validation P5 (ciblée) de la fiche v017-3a (#671 : rattacher une transaction à une écriture existante), une lentille, Sonnet

Tu valides la **remédiation de la passe P4** de la fiche de la Story v017-3a de Kesh (comptabilité suisse, Rust/Axum,
SvelteKit, MariaDB 10.11). Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*) : **une seule lentille**, braquée sur le
**seul diff** du commit `docs(v017-3a) : remédiation de la validation P4 (refs #671)`. Modèle : **Sonnet** (passe
ciblée, non première passe). **Contexte frais** : ne t'appuie sur aucun rapport antérieur pour juger, ni sur ce que la
fiche dit d'elle-même. Langue du rapport : **français**.

Worktree `/home/gcorbaz/devel/kesh-0173`, branche `story/v017-3-rapprochement-attente`, partie de `c5101b07`.
Historique : P1 0/1/12/12 ; P2 0/0/12/14 (découpage C-v017-3-25, v017-3c = #828) ; P3 0/0/6/12 ; P4 (Opus, ciblée)
0/0/1/8 — le MEDIUM portait sur le point (b) de l'AC-3, réécrit en P3 puis encore en P4.

## À lire

1. Le diff ciblé : `git log --oneline c5101b07..HEAD`, puis `git show <commit de la remédiation P4>` (fiche
   `v017-3a-rattacher-ecriture-existante.md`, registre `v017-3-choix-autonomes.md`).
2. L'AC-3 entier **après** le diff (les trois paragraphes (b) : P2, P3, P4 — le dernier remplace les précédents).
3. Le rapport P4, **après** ton examen : `/home/gcorbaz/devel/kesh-gate-logs/v017-3a-validate-p4.md`.

## La lentille

### Axe obligatoire — relire au code **chaque geste** cité dans le nouveau point (b)

Pour chacun, ouvre la source à `c5101b07` et **cite la ligne lue** :
- le lot d'acceptation (`crates/kesh-api/src/routes/reconciliation.rs:1760-1770`, `with_account_lock` puis
  `accept_batch`) ; `accept_one_invoice` (`:1991`, relecture `:2309-2312`, écriture `:2743`, `UPDATE` `:2810`) ;
  `accept_one_split` (`:2008`, écriture `:3721`, `UPDATE` `:3739`) ; `accept_one_rule` (`:2025`, `:4169`, `:4218`) ;
  `accept_one_supplier_invoice` (`:2041`, `pay_in_tx_as`, `UPDATE` `:3373`) — chacun relit-il la transaction **par le
  compte verrouillé** (`…_for_account(…, bank_account_id, …)`) avant d'écrire, de sorte qu'il n'écrive aucune
  transaction d'un autre compte ?
- le rapprochement manuel (`:4965-4969`, `:5038`, `:5046`) et l'éclatement manuel (`:5455-5459`, `:5502`, `:5509`) ;
- le lettrage du lot (`crates/kesh-db/src/repositories/letterings.rs:243-244`) et le `SAVEPOINT` du lot (`:1900-1921`) ;
- les gestes **sans** verrou nommé : `reverse_in_tx_inner` (`journal_entries.rs:2741-2745`), `reversal_blockers` →
  `document_owners` (`:2212`, appel `:2415`, lecture `:2278` — ordinaire ?), `update_in_tx`, `delete_in_tx`
  (`:1819-1835`), `delete_once` (`routes/bank_imports.rs:1507-1518`) — l'un d'eux prend-il une ligne de
  `bank_transactions` après une ligne de `journal_entries` ?
- **Cherche un geste que (b) ne nomme pas** et qui tient `je` puis verrouille `bt` : grepe `UPDATE bank_transactions`,
  `FROM bank_transactions` suivi de `FOR UPDATE` ou `LOCK IN SHARE MODE` (sur plusieurs lignes : lis la fonction), et pour
  chaque site, remonte à la fonction et à sa route ; dis si elle tient le verrou nommé du compte de cette transaction.
  Confirmation d'un lot de paiement, annulation d'un règlement client ou fournisseur, clôture d'un compte bancaire,
  rejet et annulation du rejet compris.
- La règle écrite (« tout geste qui tient `je` puis verrouille `bt` tient le verrou nommé du compte de cette `bt` ; les
  gestes sans verrou nommé ne prennent jamais `bt` après `je` ») est-elle **vraie** au code, et le critère d'arrêt
  **décidable** ? La phrase sur `ON DELETE SET NULL` (verrou implicite sur l'index de `matched_entry_id`) est-elle juste ?

### Ce que la remédiation a pu introduire

- Les LOW C2 à C8 appliqués sans en créer : la phrase sans motif ; la formule « tant que dure l'obstacle suivant » et
  #827 / #828 propagées (D9, D12, M25, L3, Q4, Q5, AC-13, registre C-v017-3-25 — l'amendement est-il **sous** C-25 ?) ;
  le compte du PDF (49 occurrences, `pdftotext … | tr … | grep -oi "rattach" | wc -l`, sortie standard seulement) ; les
  citations de `LinkSettlementModal.test.ts` (`:124`, `:193`, `:215`, `:319`) et la valeur par défaut du mock ; le montage
  du coût ; les Vitest de M29 et M30.

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois**, à `/home/gcorbaz/devel/kesh-gate-logs/v017-3a-validate-p5.md`. Il
contient :

1. **Les findings**, numérotés `C1`, `C2`…, chacun avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), le lieu
   (`fichier:ligne`), **la ligne lue citée** (sortie `grep -nF` ou extrait copié) et le remède proposé. Une affirmation de
   présence ou d'absence sans sortie `grep -nF` copiée ne compte pas.
2. **Les axes exercés** et **les axes non exercés**, un par un, avec ce que tu as lu ou cherché. Un « 0 finding » sans
   cette liste ne compte pas comme passe.
3. Un décompte final par sévérité, **recompté** depuis ta liste.

## Interdits — lecture seule stricte

- **Aucune écriture**, hors de ton seul rapport : ni `Edit`, ni `Write` sur un autre fichier, ni redirection (`>`, `>>`,
  `tee`) — pas même vers un fichier « jetable ». Un fichier créé par erreur se **déclare** au rapport ; il ne se supprime
  pas.
- Aucun `git checkout`, `git switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase`, sous
  aucune forme.
- Aucune commande `gh` qui écrit ; `gh issue view` seulement.
- Aucune commande qui **compile ou exécute** : ni `cargo`, ni `npm`, ni `make`, ni `latexmk`, ni `python3`, ni `node`,
  ni `sqlx`, ni `docker`, ni `curl`, ni `claude`.
- Jamais `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt.
- Jamais `rm`, `pkill`, `killall`.
- Jamais `~/kesh-test/` ; aucune donnée réelle dans le rapport.
- Rien dans `/home/gcorbaz/devel/kesh-0162` ni dans `/home/gcorbaz/devel/kesh` : lecture seule.
- Commandes permises : `cat`, `sed -n`, `grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `git log`, `git show`,
  `git diff` (en lecture), `pdftotext … -` vers la sortie standard, `gh issue view`.
