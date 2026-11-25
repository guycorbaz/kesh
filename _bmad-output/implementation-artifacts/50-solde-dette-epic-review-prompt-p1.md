# Prompt — revue de code P1 de la story 50-solde-dette-epic (#1492), trois lentilles Opus, première passe complète

Tu relis la story **50-solde-dette-epic** de Kesh (comptabilité suisse ; Rust/Axum, MariaDB, SvelteKit 5 runes), release **0.20.0**, Epic 50
(story de solde, lot B seul). Kesh tient les livres réels du Project Lead, qui a **déjà émis de vraies factures remises sur papier** :
rien de ce qui existe (facture, avoir, lignes, totaux, TVA, écritures, numéros, PDF) ne doit changer. **Aucune migration.** Une
correction de dette, **sans changement de comportement** (#1492) : la règle « ce refus de doublon de facture fournisseur se lève-t-il par
`confirmNotDuplicate` ? » n'était écrite qu'à deux endroits qui concordaient sans se lier (`guard_duplicates_in_tx` dans `kesh-db`,
`supplier_invoice_duplicate_response` dans `kesh-api`). Elle n'a plus qu'une source :
`DuplicateRule::lifted_by_confirmation(self, gesture) -> bool` (`crates/kesh-db/src/repositories/supplier_invoices.rs`), lue par la
couche base (levée du refus) et par la réponse HTTP (`details.confirmable`) ; le code d'erreur se déduit de la règle dans la fonction de
réponse ; une douzaine de doc-comments renvoient à la source.

Worktree `/home/gcorbaz/devel/kesh-50-solde`, branche `story/50-solde-dette-epic`, partie de `epic/50-corrections` = `48a66ca24`. Diff à
relire, **aplati** : `git -C /home/gcorbaz/devel/kesh-50-solde diff 48a66ca24 88905d0ff -- crates` (`88905d0ff` = dernier commit de
code ; le commit `cf8c2c01a` porte les tests rouges). Les commits suivants ne portent que la fiche et ce prompt : vérifie-le par
`git -C /home/gcorbaz/devel/kesh-50-solde diff --stat 88905d0ff HEAD -- crates` (vide attendu). Langue du rapport : français.
**Modèle : Opus**, contexte frais, trois lentilles en parallèle.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/50-solde-dette-epic.md` en entier (« Ce qui se lit au code », D-B1 à D-B3, AC-1 à AC-6, tableau
des mutations M-B1 à M-B3, **Dev Agent Record**, **Change Log**) et le dernier rapport de validation
`/home/gcorbaz/devel/kesh-gate-logs/e50solde/validation-p3.md`. Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le avec
les journaux hors dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e50solde/dev/` : `inv-etape0.txt`,
`rouge-ac1-ac2.log`, `clippy-1.log`, `vert-cible.log`, `mut-b1.log`, `mut-b2.log`, `mut-b3.log` (extraits) avec leurs `mut-bN.full.log`
(sortie nextest entière, ligne « git diff après restauration ») et `mut-bN.diff.txt` (la mutation), `test-fast-ci.log`, `gate-ci.log`,
`gate-ci-journal.log`, et `../alerte.md`. **L'E2E complet n'a pas tourné** (il tourne à la clôture, après la revue) ; aucun fichier du
frontend n'est touché, donc aucune capture (recette visuelle : sans objet, à confirmer au diff).

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, nulle part (ni
  scratchpad, ni `/tmp`) ; **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`, `> /dev/null` compris). Un fichier créé
  par erreur se **déclare** au rapport ; il ne se supprime pas.
- **Aucun `rm`**, sous aucune forme ; ni `pkill`, ni `killall`, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`/`commit`/`reset`/`merge`/`rebase`/`push`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`,
  `jq` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`,
  `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` (ils se **lisent**) ; ni `make`, `latexmk`, `pdflatex`,
  `cargo`, `docker`, `mariadb`, `sqlx`, `curl`, **`claude`**, `sed -i`, `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`, `awk`
  (sans redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, et, pour un PDF, la
  seule forme `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`.
- ⚠️ `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport ; tout chemin
  cité se vérifie (`git cat-file -e <ref>:<chemin>` ou `ls`).

## Les axes obligatoires, pour chaque lentille

1. **Les AC et les décisions.** AC-1 à AC-6 et D-B1 à D-B3 contre le code, les tests et les journaux. Un AC sans test qui peut rougir
   doit être déclaré filet, avec sa raison (l'AC-2 est un filet qui ne rougit pas sur l'ancien code : est-ce dit, et les mutations le
   rattrapent-elles ?). Les écarts à la fiche sont-ils déclarés au Dev Agent Record (M-B3 : six e2e de plus que la fiche ne l'annonce) ?
2. **Le comportement ne change pas, dans les quatre cases.** (règle 1, saisie) → non confirmable ; (règle 2, saisie) → oui ; (règle 1 ou
   2, dévalidation d'une annulée) → oui. Compare, ligne à ligne, le `match` d'avant (`git show 48a66ca24:crates/kesh-db/src/repositories/supplier_invoices.rs`,
   autour de `:658`) à `lifted_by_confirmation` ; le `match` est-il **exhaustif sur le couple**, sans bras `_` ? Le code d'erreur, le
   message (clés `error-supplier-invoice-duplicate`, `-probable-duplicate`, `-duplicate-unvalidate`) et `details.confirmable` sont-ils
   identiques à ceux d'avant dans chacune des quatre cases ? Les trois appelants de `guard_duplicates_in_tx` (`:944`, `:1152`, `:1875` à
   la base) passent-ils toujours le bon geste ? L'audit `supplier_invoice.duplicate_confirmed` s'écrit-il dans les mêmes cas ?
3. **Le choix variante → règle de chaque bras** (`kesh-api/src/errors.rs`, les deux bras `DbError::SupplierInvoiceDuplicate` /
   `…ProbableDuplicate`) : seul ce que le compilateur ne tient pas. Est-il tenu par l'AC-2 et M-B3 ? Un test construit-il l'erreur à
   partir de la **même** association que le code de production (test circulaire : `duplicate_refusal` du module de tests redit la
   correspondance variante ↔ règle) ? Dis si c'est un défaut ou la limite déclarée à D-B2.
4. **Les mutations.** M-B1 (règle 1, `Entry` → `true`), M-B2 (règle 2, `Unvalidation` → `false`), M-B3 (règles des deux bras
   échangées) : chacune est-elle tuée **pour la bonne raison**, par l'assertion que la fiche nomme (littérale, non « relue », pour M-B1
   et M-B2 : « une table ») ? Chaque journal montre-t-il le diff de la mutation (`mut-bN.diff.txt`), le rouge et sa raison, et un
   `git diff` vide après restauration ? Une mutation que les tests existants et les tests neufs ne tuent pas (angle mort) ? La case
   (règle 2, dévalidation) n'était assertée par aucun test HTTP : l'est-elle maintenant ? Un test vert par construction (comparaison
   d'une valeur à elle-même, `relue` qui suit toujours la table) ?
5. **Les doc-comments (D-B3) et la propagation.** Chaque site de la liste de D-B3 renvoie-t-il à `DuplicateRule::lifted_by_confirmation`
   **sans changer de sens** ? Y a-t-il un site qui **récite encore la table sans la nommer** (rejoue le grep du symptôme :
   `git grep -niE "jamais.{0,40}(règle 1|SUPPLIER_INVOICE_DUPLICATE)|lève.{0,30}(deux règles|règle 2)|les deux règles|levé \(règle|aucune\** sortie|sans sortie" -- 'crates/*/src/*'`
   et `git grep -nE "liftable|confirmable|probable \|\||gesture == DuplicateGesture::Unvalidation" -- 'crates/*/src/*'`) ? Un site qui
   **décide** encore (hors `lifted_by_confirmation`) ? Les liens intra-doc `[`DuplicateRule::lifted_by_confirmation`]` se résolvent-ils
   (`kesh-db/src/errors.rs` cite un chemin `crate::repositories::…` en code, non en lien : correct ?) ?
6. **Le manuel et le contrat** (`docs/manual/`, **PDF compris**, aplati avant de greper : `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`) :
   `docs/manual/fr/admin-manual.tex:2585` (section `sec:maj-0-20-0`, `confirmable`) et `docs/api-external.md:1028`, `:1315-1316` récitent
   la table comme contrat. Disent-ils encore ce que fait le code (la story ne change aucune case) ? Le PDF est-il d'accord
   (`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c 'confirmable'`) — il n'est pas régénéré dans la
   branche, le dire sans le compter comme défaut ? Les manuels DE/IT/EN n'existent pas (`ls docs/manual/{de,it,en}`). La Note de version
   (« Interne ») est-elle juste ?
7. **Rien d'autre ne bouge** : AC-4 (`git diff 48a66ca24...HEAD --stat -- crates/kesh-db/tests/supplier_invoice_duplicate.rs
   crates/kesh-db/tests/supplier_invoice_duplicate_lock.rs crates/kesh-api/tests/supplier_invoice_duplicate_e2e.rs` vide), AC-5
   (`kesh-qrbill`, `kesh-report`, `kesh-core`, `kesh-db/migrations`, `frontend`, `kesh-i18n` : vide ; aucune requête SQL modifiée :
   `git diff 48a66ca24...HEAD -- crates/kesh-db | /usr/bin/grep -E '^[+-].*(SELECT|UPDATE|INSERT|DELETE)'` vide). Le frontend lit
   `details.confirmable` (`frontend/src/lib/features/supplier-invoices/supplier-invoice-form.ts:592-603`, `:619`) : inchangé, mais la valeur
   qu'il reçoit l'est-elle ? Les clés i18n et leurs replis Rust (`SUPPLIER_INVOICE_*_FR`) sont-ils intacts ?
8. **La fiche et les décomptes.** Le Dev Agent Record dit-il vrai : 1259/1259 (ciblé), 6270/6270 (`test-fast --ci`), 6289 passés
   (`gate-ci`), huit tests neufs, décomptes **recomptés depuis les journaux**, périmètre de chaque ? Aucun mot-clé de fermeture :
   `git log origin/epic/50-corrections..HEAD --format=%B | /usr/bin/grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'` vide ?

## Lentilles

- **B — Blind Hunter** : le diff seul d'abord (sans la fiche), défauts de code, d'état, de régression ; puis les axes.
- **E — Edge Case Hunter** : toutes les branches et états limites (axes 2, 3, 4 surtout) : un geste ou une règle ajoutés demain ; un
  appelant qui passerait `Unvalidation` pour un autre geste ; `policy == Confirmed` combiné à une règle non levable ; un refus de
  règle 1 à la saisie quand `confirmNotDuplicate: true` est envoyé (le refus reste-t-il 409 `confirmable: false`, sans audit ?).
- **A — Acceptance Auditor** : chaque AC et chaque décision contre le code, les tests, les journaux (axes 1, 6, 7, 8).

## Rapport

Un rapport par lentille : `/home/gcorbaz/devel/kesh-gate-logs/e50solde/review-p1-B.md`, `…/review-p1-E.md`, `…/review-p1-A.md` (le
seul fichier que tu écris). Pour chaque finding : identifiant (`B1`, `E1`, `A1`…), sévérité (CRITICAL, HIGH, MEDIUM, LOW), fichier et
ligne, extrait cité, scénario, correctif proposé. Verdict chiffré en tête. **Liste des axes exercés ET non exercés** (un « 0 finding »
sans elle ne compte pas). Ce qui n'a pas tourné reste écrit : la revue n'exécute rien ; les gates qui ont tourné au passage en revue
sont cités au Dev Agent Record avec leurs journaux.
