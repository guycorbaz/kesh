# Prompt — passe 1 de `bmad-create-story validate`, Story 25-3-a

*Versionné le 2026-09-24. Deux lentilles en contexte frais (Sonnet), orthogonales à l'auteur de la
fiche (Opus). Elles partagent le préambule et les interdits ; leurs axes diffèrent.*

Ton objet est `_bmad-output/implementation-artifacts/25-3-a-annuler-reglement.md`, dépôt
`/home/gcorbaz/devel/kesh`, branche `story/25-3-a-annuler-reglement` (issue de `main` à `ac1f2829`).

⛔ **C'est une fille du découpage de la 25-3.** La fiche mère
`25-3-annuler-reglement-et-rapprochement.md` (statut `split`) reste la source des faits ; le socle
`25-3-zero-reverse-in-tx.md` est **mergé** (`reverse_in_tx`, `crates/kesh-db/src/repositories/journal_entries.rs`).
La sœur **25-3-b** (#418, dé-rapprochement) n'est pas spécifiée : elle appellera le geste écrit ici.

⚠️ **Ne conteste pas les arbitrages de Guy** : ligne `invoice_settlements` **retirée** et non marquée
annulée ; **deux gestes** côté fournisseur (annuler le règlement, puis la facture) ; lot pain.001
confirmé **laissé tel quel** ; `supplier_invoices::cancel` hors périmètre (**#454**) ; le découpage
lui-même. Conteste leur **mise en œuvre**.

## Lentille A — le socle et le dépôt

1. **L'exemption de l'AC 1 est-elle suffisante ET étroite ?** Lis `reversal_blocker` et
   `reverse_in_tx`. Énumère **tous** les états réels d'une écriture de règlement (client : manuel,
   par rapprochement ; fournisseur : manuel, par lot confirmé, puis éventuellement rapprochée par
   un des **cinq** sites de `reconciliation.rs` qui posent `matched_entry_id`) et dis, pour chacun,
   ce que la spec prescrit et si c'est juste. Cherche un état qu'elle n'a pas vu.
2. **L'AC 2** (l'autorité fournisseur ne couvre pas l'écriture d'achat) : la garde prescrite
   est-elle la bonne, et le seul trou de ce genre ?
3. **Transactions et verrous.** Ordre des verrous de l'annulation contre celui de `settle_invoice`,
   de `accept_one_invoice` (rapprochement) et de `pay_in_tx`. Interblocage possible ? Course entre
   deux annulations, entre annulation et encaissement concurrent ?
4. **`paid_at`, `version`, résiduel (AC 5, 6)** contre `invoice_settlements_write.rs` et
   `reconciliation.rs` : la projection prescrite est-elle exacte, y compris avec un avoir, un
   trop-perçu, une facture réglée avant `20260827000001` ?
5. **Tout lecteur de l'état** qu'une annulation rend incohérent si la spec l'oublie : cherche par
   `grep` les lecteurs de `paid_at`, de `supplier_invoices.status = 'paid'`, de
   `settlement_journal_entry_id`, des lignes `payment_batch_items` d'un lot confirmé (rapports,
   exports, échéancier, relances, dévalidation, export de souveraineté). Inventorie les sites **non
   traités** par la spec, et dis pour chacun s'il est sain.
6. **Exactitude** : chaque `fichier:ligne`, nom de fonction et décompte de la fiche, vérifié depuis
   la source.

## Lentille B — l'API, les écrans, les registres, la documentation

1. **Les routes (AC 11)** : chemins, rôle, emplacement dans `lib.rs`, forme de réponse — cohérents
   avec les routes voisines ? La route `GET .../settlements` est-elle la bonne réponse au manque
   (ou un champ de `GET /invoices/{id}` serait-il plus juste) ? Clés API (PAT) : que prescrit la
   spec, et est-ce cohérent avec les jumelles d'écriture ?
2. **Les registres (AC 10, 11)** : `audit_labels.rs`, `audit_label_registry.rs`
   (`SITES_INDIRECTS`), `audit_route_registry.rs` (totaux et message de ventilation). Les valeurs
   de départ citées sont-elles exactes ? Qu'est-ce qui rougira, et la spec le dit-elle ?
3. **Les motifs de refus (AC 12)** : inventaire **clos** des sites des clés
   `journal-entries-reverse-blocked-settlement` / `-supplier-invoice` et des codes
   `OWNED_BY_SETTLEMENT` / `OWNED_BY_SUPPLIER_INVOICE` — FTL ×4, replis serveur et Svelte, tests,
   manuels, `api-external.md`. Le nouveau texte prescrit est-il juste pour **toutes** les écritures
   que ces codes couvrent (écriture d'achat fournisseur comprise) ?
4. **Les écrans (AC 13)** : contre le code réel des deux fiches. Gardes vitest i18n
   (`i18n-keys.test.ts`, `i18n-un-repli-par-cle.test.ts`, `i18n-libelle-en-dur.test.ts`) : ce qui
   rougira. Les E2E Playwright existants qui rougiront (inventaire par `grep`), au-delà de
   `invoices_echeancier.spec.ts`.
5. **Le manuel** : liste **close** des passages à modifier. Contrôle le **PDF aplati**
   (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`), et cherche un site
   que la fiche ne nomme pas — `README.md`, `website/`, `docs/api-external.md`, `CHANGELOG.md`,
   manuel admin.
6. **Les tests (AC 14)** : chaque garde nommée a-t-elle un test **et** une mutation qui la fait
   rougir ? Un test prescrit peut-il passer à vide ?
7. **Périmètre** : la story reste-t-elle sous le seuil de découpage (5 modules) ? Empiète-t-elle sur
   la 25-3-b, ou lui laisse-t-elle un trou ?

## Ce que tu rends

- **Les findings** : sévérité (CRITICAL / HIGH / MEDIUM / LOW), l'endroit exact, **la preuve** (le
  code lu, la commande exécutée et son résultat), ce qu'il faut changer. Un reproche au code de ne
  pas encore faire ce que la story prescrit n'est **pas** un défaut de la spec.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un « 0 finding » sans
  cette liste ne compte pas.

## Interdits

⛔ N'écris aucun fichier du dépôt, n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante — `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make` dans `docs/manual/`, tout `git commit`/`push`/
`add`/`stash`/`reset`/`rebase`/`checkout`, `sqlx migrate`, `cargo test`/`cargo nextest`, `npm run`.
Lecture, `grep`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` et `cargo check` sont autorisés.
