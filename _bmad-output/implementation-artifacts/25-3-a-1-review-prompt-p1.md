# Prompt — passe 1 de `bmad-code-review`, Story 25-3-a-1

*Versionné le 2026-09-24. Trois lentilles en contexte frais (Sonnet), orthogonales à l'auteur du
code (Opus). Protocole du workflow : Blind Hunter, Edge Case Hunter, Acceptance Auditor.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-3-a-annuler-reglement`. **Le diff revu est
`git diff a1534b63..d8857808`** (le code de la story ; les commits de spec avant `a1534b63` et le
compte rendu `d321a947` en sont exclus), aussi enregistré dans
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/5bef2124-6ec8-41d8-9004-39b0653420a3/scratchpad/25-3-a-1.diff`
(3640 lignes). Fiche : `_bmad-output/implementation-artifacts/25-3-a-1-annuler-reglement-client.md`.

Objet : **annuler un règlement client par contre-passation**. Le socle `reverse_in_tx` gagne une
contre-passation « au titre de son propriétaire » (`ReversalAuthority`, `reverse_owned_in_tx`), qui
lève **un** motif pour **une** pièce nommée et laisse la précédence se poursuivre ; un calcul de
motifs d'annulation (tête client + queue commune sur l'écriture) sert à la fois la lecture
(`GET /invoices/{id}/settlements`) et l'écriture (`POST …/settlements/{sid}/cancel`).

## Lentille 1 — Blind Hunter (diff SEUL)

Tu ne lis **que** le fichier de diff ci-dessus — aucun autre fichier du dépôt, ni la fiche. Revue
adversariale générale (skill `bmad-review-adversarial-general`) : défauts de logique, d'erreur,
de concurrence, de sécurité (multi-tenant : chaque requête est-elle scopée par `company_id` ?),
de tests qui ne prouvent rien (assertions uniquement négatives, états forgés, tests qui passent à
vide), incohérences entre code et commentaires.

## Lentille 2 — Edge Case Hunter (diff + lecture du dépôt)

Méthode `bmad-review-edge-case-hunter` : **parcourir chaque branche et chaque condition limite**
du code neuf, en lisant le code appelé et appelant dans le dépôt. Priorités :
1. `reverse_in_tx_inner` et `ReversalAuthority::exempts` : la recherche du premier motif non
   levé est-elle **strictement équivalente** à l'ancien comportement quand aucune autorité n'est
   donnée ? (Compare avec `git show a1534b63:crates/kesh-db/src/repositories/journal_entries.rs`.)
2. `settlement_entry_cancel_blocker` et `settlement_cancel_blocker` : chaque rang, chaque ordre ;
   la lecture suit-elle **exactement** ce que l'écriture refuse, y compris le verrou de période du
   jour (limite assumée — est-elle écrite là où il faut ?) ?
3. `cancel_settlement_in_tx` : ordre des verrous (facture, règlement, écriture, exercice) face à
   `settle_invoice`, `accept_one_invoice` (`reconciliation.rs`), `create_credit_note` ; que se
   passe-t-il si deux annulations, ou une annulation et un encaissement, se croisent ?
4. Les routes (`routes/invoices.rs`, `lib.rs`) : scoping, rôle, 404, forme de réponse ; la route
   `GET` et la route `POST` au même chemin dans deux routeurs.
5. Le frontend (`InvoiceSettlements.svelte`, fiche `invoices/[id]/+page.svelte`) : états limites
   (liste vide, échec de chargement, double clic, refus au clic, facture non validée).

## Lentille 3 — Acceptance Auditor (diff + fiche + manuel)

Confronte le diff à **chaque AC** de la fiche (1 à 13) : violation, écart d'intention, manque,
contradiction. Vérifie les **décomptes** du Dev Agent Record en les **recomptant** (tests ajoutés,
clés i18n, `sitesTotal`, registre de routes, libellés d'audit). ⛔ **Le manuel** : vérifie
`docs/manual/fr/user-manual.tex` contre le code **et** le PDF aplati
(`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`) — le manuel promet-il
quelque chose que le code ne fait pas, ou tait-il un refus que le code oppose ? Idem
`docs/api-external.md`, `CHANGELOG.md`, `README.md`.

## Ce que tu rends

- **Les findings** : sévérité (CRITICAL / HIGH / MEDIUM / LOW), `fichier:ligne`, **la preuve**
  (code lu, commande exécutée et son résultat), la correction proposée. Pour tout CRITICAL ou HIGH
  qui affirme qu'une chose est absente ou présente : la commande `grep -nF` exécutée et son résultat.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un « 0 finding »
  sans elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier du dépôt, n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante — `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make` dans `docs/manual/`, tout `git commit`/`push`/
`add`/`stash`/`reset`/`rebase`/`checkout`, `sqlx migrate`, `cargo test`/`cargo nextest`, `npm run`,
`npx playwright`. Lecture, `grep`, `git log`/`show`/`diff`, `pdftotext` (vers le scratchpad) et
`cargo check` sont autorisés.
