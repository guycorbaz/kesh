# Prompt — passe 1 de `bmad-code-review`, Story 25-3-c

*Versionné le 2026-09-26. Trois lentilles en contexte frais (Sonnet), orthogonales à l'auteur du
code (Opus 5.5). Protocole du workflow : Blind Hunter, Edge Case Hunter, Acceptance Auditor.*

*(Fichier réécrit à l'identique après la passe : il avait disparu du disque entre son usage par
les lentilles et le commit — cause non établie.)*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-3-c-annuler-facture-fournisseur`, tête `999a66e4`.
**Diff revu** : `git diff main..HEAD` sur le code et la documentation (PDF et `_bmad-output/` exclus),
enregistré dans
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/ca5ce2e2-67a3-4eeb-817f-c2de35620a1c/scratchpad/25-3-c-code.diff`.
Fiche : `_bmad-output/implementation-artifacts/25-3-c-annuler-facture-fournisseur.md` — ses AC (1 à
10), ses arbitrages (2026-09-24, et **Q1 du 2026-09-26 : le règlement d'une facture payée annulée
est DÉTACHÉ** — ne pas les contester, en contester la mise en œuvre) et son Dev Agent Record.

Objet : annuler une facture fournisseur **même payée** (#454). `supplier_invoices::cancel` passe
désormais par le socle `journal_entries::reverse_owned_in_tx` avec une autorité neuve
`ReversalAuthority::SupplierPurchase` ; motifs composés par `supplier_invoice_cancel_blocker`
(rang 1 déjà annulée, queue commune sur l'écriture d'ACHAT, lot `generated` en dernier) ;
colonnes de règlement remises à `NULL` ; trois champs de lecture ; écran à bloc unique.

⛔ **Premiers suspects** : (1) l'ordre verrous / lectures de `cancel_in_tx` sous `REPEATABLE READ`
(leçon de la 25-3-b) ; (2) la factorisation de l'étape 1-bis du socle — l'autorité **règlement**
de la 25-3-a-2 est-elle intacte ? (3) le détachement : quelque chose suppose-t-il encore qu'une
facture `cancelled` ou `paid` porte ses colonnes de règlement ?

## Lentille 1 — Blind Hunter (diff SEUL)

Revue adversariale générale : logique, erreurs, concurrence, multi-tenant (`company_id` sur
**chaque** requête neuve), tests qui ne prouvent rien ou passent à vide, incohérences code /
commentaires / textes, clés i18n, replis mot pour mot.

## Lentille 2 — Edge Case Hunter (diff + dépôt)

Chaque branche du code neuf, en lisant l'appelé et l'appelant. Priorités :
1. **L'instantané et les verrous** de `cancel_in_tx` : reste-t-il une lecture non verrouillante
   avant le verrou de l'écriture d'achat et de son exercice — dans le code ou dans ce qu'il
   appelle ? Le rang « lot » (`in_generated_batch`, non verrouillant) voit-il un
   `payment_batches::create_batch` concurrent ? Et `confirm_batch` (qui règle une facture d'un lot
   `generated`) contre une annulation concurrente ?
2. **L'ordre des verrous** contre `pay`, `cancel_settlement_in_tx`, `payment_batches::create_batch`
   / `confirm_batch` / annulation de lot, `fiscal_years::close` / `reopen`, `accounts::archive`,
   la contre-passation manuelle d'une écriture : interblocage possible ?
3. **Le détachement** : qui lit `settlement_journal_entry_id`, `settlement_type`, `paid_at` d'une
   facture fournisseur (rapports, tableau de bord, export, lots, `imported_supplier_invoices`,
   frontend liste et fiche) — et se trompe-t-il pour une facture `cancelled` qui a été payée ?
4. **La factorisation 1-bis** : `SupplierSettlement` et `ClientSettlement` inchangés en
   comportement ? Une autorité `SupplierPurchase` sur l'écriture d'achat d'une AUTRE facture ?
5. **Le frontend** : le bloc unique (statuts `open`, `paid`, `cancelled`, `cancellable` `null`),
   double clic, refus au clic, la fiche après l'annulation d'une facture payée.

## Lentille 3 — Acceptance Auditor (diff + fiche + documentation)

Chaque AC (1 à 10) contre le diff. Les décomptes du Dev Agent Record **recomptés** depuis la source
(2472 = 2463 + 9 ; 7 tests de dépôt neufs + 1 réécrit ; +2 HTTP ; +6 vitest ; +1 Playwright ;
`sitesTotal` 1719 ; `CLES_RELEVEES` 210 ; 9 clés neuves ×4). Les textes : quatre locales, replis
**mot pour mot** le FTL fr-CH, liste des sites de l'AC 6 **close** (grep de la clé, du code et de
la phrase). ⛔ **Le manuel** `docs/manual/fr/user-manual.tex` **et son PDF aplati**
(`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, vers le scratchpad) :
reste-t-il une phrase qui dit qu'une facture fournisseur ne s'annule qu'ouverte, qu'il faut annuler
le règlement d'abord, ou qui contredit le détachement ? Cherche large : c'est en vérifiant le
manuel contre le code qu'on trouve les chemins oubliés. `docs/api-external.md`, `CHANGELOG.md`
(section `[0.12.1]` seulement), `README.md`. ⚠️ Cette story **ferme #454** : reste-t-il dans le
dépôt une phrase ou un commentaire qui décrit l'ancien `cancel` ?

## Ce que tu rends

- Findings : sévérité (CRITICAL / HIGH / MEDIUM / LOW), `fichier:ligne`, **preuve** (code lu,
  commande et résultat), correction. Pour tout CRITICAL ou HIGH affirmant qu'une chose est absente
  ou présente : la commande `grep -nF` exécutée et son résultat.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.**

## Interdits

⛔ N'écris aucun fichier du dépôt, n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante — `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make` dans `docs/manual/`, tout `git commit`/`push`/
`add`/`stash`/`reset`/`rebase`/`checkout`/`switch`/`worktree`, `sqlx migrate`, `cargo test`/`cargo nextest`,
`npm run`, `npx playwright`. Lecture, `grep`, `git log`/`show`/`diff`, `pdftotext` (vers le
scratchpad) et `cargo check` sont autorisés.
