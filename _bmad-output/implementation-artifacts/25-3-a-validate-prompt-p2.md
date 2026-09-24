# Prompt — passe 2 de `bmad-create-story validate`, Story 25-3-a

*Versionné le 2026-09-24. Deux lentilles en contexte frais (Haiku 4.5) — cycle Sonnet → Haiku →
Opus. Elles partagent le préambule et les interdits ; leurs axes diffèrent. Elles lisent la fiche
**telle qu'elle est** (un seul fichier, pas de diff multi-commit).*

Ton objet est `_bmad-output/implementation-artifacts/25-3-a-annuler-reglement.md`, dépôt
`/home/gcorbaz/devel/kesh`, branche `story/25-3-a-annuler-reglement`.

Contexte : fille de la 25-3 (mère `25-3-annuler-reglement-et-rapprochement.md`, statut `split`,
source des faits) ; socle `reverse_in_tx` **mergé**. La passe 1 (deux lentilles Sonnet) a remonté
1 HIGH et 3 MEDIUM, **corrigés** ; puis Guy a **corrigé un arbitrage** et la fiche a été remaniée —
la section « Arbitrages rendus » et le Change Log disent quoi. ⛔ **Le motif mesuré sur ce projet :
la sévérité se déplace vers ce qu'on vient d'écrire.** Les remaniements sont le premier suspect.

⚠️ **Ne conteste pas les arbitrages de Guy** listés en « Arbitrages rendus » (sauf **Q5**, qui est
explicitement une *position de la fiche* et peut être discutée). Conteste leur **mise en œuvre**.

## Lentille A — la cohérence de la fiche après remaniement

1. **Contradictions internes.** Chaque AC, tâche, Dev Note et Référence est-il cohérent avec les
   arbitrages **actuels** ? Cherche les restes de l'état antérieur : « deux gestes », « pas de
   passage direct », une Q4 encore ouverte, un renvoi à un AC renuméroté (l'AC 3-bis est neuf ;
   les renvois « AC 13 » / « AC 15 » pointent-ils le bon critère ?).
2. **L'AC 3-bis** (borne exercice clos, précédence des motifs) : est-il cohérent avec l'AC 1 (le
   socle ne change pas), l'AC 6 (statut), l'AC 7 (rapprochement), l'AC 11 (`cancelBlockedBy`),
   l'AC 13 (textes à l'écran) et l'AC 14 (tests) ? Un motif de la précédence manque-t-il d'un
   texte, d'un code, d'un test ?
3. **Frontière avec les stories voisines** : 25-3-b (#418), 25-3-c (annulation d'une facture
   fournisseur dans tous les cas, absorbe #454 — cf. `sprint-status.yaml`), #455, #456, Epic 15
   (lettrage). Un critère de la 25-3-a empiète-t-il, ou laisse-t-il un trou qu'aucune ne couvre ?
4. **Les tâches** couvrent-elles tous les AC, y compris 3-bis ?

## Lentille B — l'exactitude des parties neuves contre le code

1. `DbError::FiscalYearClosed` et son mapping (`crates/kesh-api/src/errors.rs`) : existent-ils
   tels que cités ? Le texte du repli générique convient-il au contexte d'une annulation, ou
   l'écran doit-il le nommer autrement (AC 13) ?
2. **Comment savoir que l'exercice de l'écriture de règlement est clos** : quelle colonne, quel
   statut (`fiscal_years`), quelle requête existante (cf. `delete_in_tx` dans
   `journal_entries.rs`) ? La fiche le dit-elle assez pour qu'un développeur ne l'invente pas ?
3. `SupplierInvoiceResponse` (`crates/kesh-api/src/routes/supplier_invoices.rs`) et
   `InvoiceResponse` : les champs neufs prescrits (AC 11) s'y intègrent-ils sans casser un autre
   appelant (liste, export, autre écran qui désérialise la réponse) ?
4. **L'AC 6 et le cas « facture créditée »** : vérifie dans `credit_notes.rs` la séquence
   « règlement partiel puis avoir » que l'AC 14 demande de produire par le **vrai** chemin. Est-elle
   réellement réalisable par les fonctions publiques ?
5. **Les sites i18n** : la famille `invoice-settlement-cancel-blocked-*` existe-t-elle déjà
   partiellement ? Les gardes vitest (`frontend/src/lib/shared/i18n-*.test.ts`) : que diront-elles ?

## Ce que tu rends

- **Les findings** : sévérité (CRITICAL / HIGH / MEDIUM / LOW), l'endroit exact, **la preuve** (le
  code lu, la commande exécutée et son résultat), ce qu'il faut changer.
- ⛔ Pour tout finding CRITICAL ou HIGH qui affirme qu'une chose est **absente** ou **présente** dans
  un fichier : cite la commande `grep -nF "<chaîne>" <fichier>` que tu as exécutée et son résultat.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un « 0 finding » sans
  cette liste ne compte pas.

## Interdits

⛔ N'écris aucun fichier du dépôt, n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante — `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make` dans `docs/manual/`, tout `git commit`/`push`/
`add`/`stash`/`reset`/`rebase`/`checkout`, `sqlx migrate`, `cargo test`/`cargo nextest`, `npm run`.
Lecture, `grep`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` et `cargo check` sont autorisés.
