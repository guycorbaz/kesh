# Revue de code P2 — Story v017-4a — passe ciblée sur la remédiation de la revue P1 — #849

*Prompt versionné (`CLAUDE.md` § *La passe ciblée*). **Une lentille**, contexte frais, modèle **Sonnet**. Langue :
français soigné. Elle relit **la seule remédiation** de la revue P1, non la story entière.*

## Le périmètre

- Worktree : `/home/gcorbaz/devel/kesh-0175`, branche `story/v017-4-facture-fournisseur-brouillon`.
- Le diff à relire : `git diff 717b19f7..HEAD` — les commits de la remédiation P1 (`git log --oneline 717b19f7..HEAD`).
- Les rapports de la revue P1, pour savoir ce que la remédiation prétend corriger :
  `/home/gcorbaz/devel/kesh-gate-logs/v0174a-review-p1-{B,E,A}.md`.
- Le Dev Agent Record : `_bmad-output/implementation-artifacts/v017-4a-facture-fournisseur-brouillon-backend.md`,
  § *Revue P1 — remédiation*, et les écarts 4, 9, 10 et 11 ; journaux sous `/home/gcorbaz/devel/kesh-gate-logs/v0174a/`
  (`mutations-p1.log`, `nextest-p1.log`, `p1-cible-1.log`, `inventaire-ac4-ecriture.txt`).

**Ce que la remédiation change.** Des tests (`supplier_invoices_draft.rs` : refus de la validation après changement
d'état, projet du test de M20, numéro blanc ; `mcp_outils_e2e.rs` : un brouillon lu par l'outil ;
`supplier_invoice_draft_e2e.rs` : un doc-comment), des doc-comments (`supplier_invoices.rs`, `letterings.rs`), des textes
(`docs/api-external.md`, `docs/migrations-idempotence-audit.md`, `docs/manual/fr/admin-manual.tex` et son PDF, la fiche
parente, la sous-fiche 4b, le Dev Agent Record) — et **une seule ligne de production** :
`repositories/supplier_invoices.rs`, dans `unvalidate_in_tx`, l'étiquette du refus filtrée (`filter(|n|
!n.trim().is_empty())`, revue P1, E-6). Aucune règle de refus, aucun verrou, aucune migration ne change.

## Les axes obligatoires

1. **Chaque finding de la P1 est-il réellement corrigé**, et sans régression ? Prendre les 18 findings (B1–B4, E-1–E-6,
   A-1–A-8) un par un : ligne lue avant (`git show 717b19f7:<fichier>`) et après, verdict.
2. **Les tests neufs prouvent-ils ce qu'ils disent ?** `validation_rechecks_accounts_and_project_changed_since_the_draft`
   (trois cas, rien d'écrit, codes assertés) ; `un_brouillon_lu_par_les_outils` (`ecritureAchatId` **présent et nul**,
   non omis) ; le test de M20 (le projet fait-il bien bloquer B sur un `FOR UPDATE` que la sonde voit, sous l'ordre
   prescrit comme sous la mutation ?) ; le numéro blanc. Lire `mutations-p1.log` (diff et ligne d'échec) : M-A3b, M-A3c
   et M20 tuées par l'assertion attendue ? **M-A3a déclarée équivalente** (`journal_entries.rs:165-166`) : le vérifier au
   code, et dire si un autre chemin de la validation peut atteindre un compte inactif sans passer par ce refus.
3. **La ligne de production E-6** : `documentNumber` passe de `""` à `null` pour un numéro blanc. Ce changement de forme
   est-il cohérent avec `docs/api-external.md` (« `details { documentId, documentNumber }` ») et avec la facture client
   (`refusal_409`, `InvoiceNotUnvalidatable`), qui ne filtre pas ? Un client de l'API ou l'écran lit-il
   `documentNumber` d'une façon qui casse ?
4. **`api-external.md`** : la liste des refus de la validation contre le code (`validate_in_tx`, puis
   `journal_entries::create_in_tx`) — codes réels, statuts HTTP (`400` / `409`), ordre ; la rangée
   `ACCOUNT_NOT_POSTABLE` et la rangée `INVALID_PAYMENT_DETAILS` contre les routes réelles (le `PUT` d'un brouillon
   **né de la complétion d'un document sans QR** : `new_from_document` rend-il bien `INVALID_PAYMENT_DETAILS` ?) ; la règle
   Q4 (« ouverte ») : `grep -n -i 'ouvert' docs/api-external.md`, trié ; `:608` (« 50 factures ouvertes ») laissé :
   le tri est-il juste ?
5. **Le manuel** : `admin-manual.tex` (lettrage parmi les refus de la dévalidation fournisseur) **et le PDF aplati**
   (`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`).
6. **Les comptes rendus** : les décomptes du Dev Agent Record (29 lignes, 123 lignes, 14 fichiers, 6 et 5 fichiers de
   l'AC-12, 46 tables) **recomptés depuis la source** par les commandes qu'il cite ; le tableau de l'AC-4 contre la
   commande 1 (chaque ligne de `inventaire-ac4-ecriture.txt` y trouve-t-elle sa ligne ?) ; les numéros de ligne cités.
7. **Propagation** : les codes corrigés (`DESIGNATED_ACCOUNTS_NOT_POSTABLE`, `LATER_FISCAL_YEAR_CLOSED`,
   `PERIOD_LOCKED`, `ACCOUNT_NOT_POSTABLE`) grepés sur tout le dépôt ; une copie restée fausse ailleurs (CHANGELOG,
   manuel, fiches, catalogues) ?

## Le rapport

- Écrit dans `/home/gcorbaz/devel/kesh-gate-logs/v0174a-review-p2.md`.
- Chaque finding : sévérité (CRITICAL, HIGH, MEDIUM, LOW), lieu (`fichier:ligne`), **ligne lue citée**, défaut, remède.
  Toute affirmation de présence ou d'absence s'appuie sur une sortie `grep -nF` copiée.
- **Axes exercés et axes non exercés**, listés : un « 0 finding » sans cette liste ne compte pas.
- Décompte final recompté depuis la liste.
- Si la remédiation touche une règle d'exécution que ce prompt ne nomme pas, le dire en tête : la passe suivante
  redevient complète (lentilles B, E et A).

## Les interdits — lecture seule stricte

- Aucune commande qui écrit, compile ou exécute : **ni `cargo`, ni `npm`, ni `make`, ni `python3`, ni `docker`, ni
  `curl`, ni `claude`**, ni aucun script du dépôt (`scripts/*`, dont `scripts/prepare-release.sh`), ni `sed -i`, ni
  `rm`, `pkill`, `killall`.
- **Aucune redirection** (`>`, `>>`, `tee`), aucun fichier temporaire, aucun fichier dans le scratchpad : un fichier
  créé par erreur se déclare au rapport, il ne se supprime pas. Le seul fichier écrit est le rapport, par l'outil
  d'écriture.
- Aucun `git checkout`, `switch`, `reset`, `stash`, `commit`, `push` ; `git diff`, `git show`, `git log` seulement.
- Aucune donnée réelle ; aucune base du NAS.
