# Prompt — passe 4 de `bmad-create-story validate`, Story 25-3-a-2

*Versionné le 2026-09-24. Deux lentilles en contexte frais (Opus) — le cycle reprend après la passe
5 (Haiku) de la sœur. La numérotation continue celle de la mère `25-3-a` (trois passes).*

Ton objet est `_bmad-output/implementation-artifacts/25-3-a-2-annuler-reglement-fournisseur.md`,
dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-3-a-annuler-reglement`.

Contexte : née du découpage de la 25-3-a (`split`, source des faits, trois passes). La sœur
**25-3-a-1** (client) est **validée** (boucle close en passe 5) ; la 25-3-a-2 **réutilise** ses AC 1
à 3 (socle étendu, autorité, table des motifs) et ne les réécrit pas. ⛔ La 25-3-a-2 est **neuve** :
tout y est suspect, et d'abord ce qu'elle **suppose** de la 25-3-a-1 sans le dire.

⚠️ **Ne conteste pas les arbitrages de Guy** (section « Arbitrages de Guy qui s'appliquent ici » —
facture ramenée à `open` ; lot confirmé laissé tel quel ; règlement d'exercice clos refusé,
réouverture comme chemin ; annulation d'une facture payée = 25-3-c). Conteste leur **mise en œuvre**.

## Lentille A — le dépôt, le geste, la dépendance à la sœur

1. **L'autorité fournisseur (AC 1)** : la vérification « l'écriture est bien
   `settlement_journal_entry_id` de cette facture » est-elle suffisante ? Existe-t-il d'autres
   écritures « possédées » par une facture fournisseur ? Comment la 25-3-a-1 a-t-elle écrit l'enum
   d'autorité, et la variante fournisseur s'y insère-t-elle sans deviner ?
2. **La table des motifs (AC 2)** : « étendus ou jumeaux, au choix du développeur » — est-ce
   implémentable sans deviner, ou faut-il trancher ? Le code du rang 1 (`SUPPLIER_INVOICE_NOT_PAID`
   donné en « p. ex. ») est-il acceptable non tranché ? Chaque rang et la paire 1-2 sont-ils
   produisibles par un vrai chemin ? L'ordre des rangs 3-4 suit-il bien le socle ?
3. **Le geste (AC 3)** contre `pay_in_tx`, `cancel`, `guard_not_in_generated_batch`,
   `payment_batches::confirm_batch` / `cancel_batch` : verrous, garde de lot (une facture `paid` peut-elle
   être dans un lot `generated` ?), colonnes vidées, contrainte `chk_supplier_invoices_paid_has_settlement`.
   Que devient une facture annulée puis **remise dans un nouveau lot** — double paiement possible ?
   (L'arbitrage autorise l'annulation ; conteste seulement ce que la fiche **omet de dire**.)
4. **Tout lecteur** de `supplier_invoices.status`, `paid_at`, `settlement_*`, `payment_batch_items`
   (rapports, exports, écrans, export de souveraineté, 25-3-c) que l'annulation rend incohérent.
5. **Exactitude** : chaque `fichier:ligne`, nom, décompte — depuis la source.

## Lentille B — l'API, l'écran, les textes, les tests, la documentation

1. **La route et les champs (AC 5, 6)** : les **cinq** appelants de `SupplierInvoiceResponse::from_parts`
   (vérifie le nombre) ; le constructeur additionnel sur GET, `pay` et l'annulation ; que voit
   `imported_supplier_invoices` ?
2. **L'écran (AC 7)** : contre `supplier-invoices/[id]/+page.svelte` réel ; préfixe de clé et
   `frontend/scripts/lint-i18n-ownership.js` pour `features/supplier-invoices/` (lis le script :
   un nom de dossier **à tiret** se traite-t-il comme un nom simple ?).
3. **Le motif `OWNED_BY_SUPPLIER_INVOICE` (AC 8)** : liste **close** des sites ; le nouveau texte
   est-il juste pour l'écriture d'achat **et** pour celle de règlement — et le restera-t-il après la
   25-3-c ?
4. **Les tests (AC 9)** : un test passe-t-il à vide, ou porte-t-il sur un état impossible ?
5. **La documentation (AC 10)** : manuel (`.tex` **et** PDF aplati :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`), `README.md` (ligne 40),
   `api-external.md`, `CHANGELOG`. Liste close ?
6. **Frontière avec la 25-3-a-1 et la 25-3-c** : rien ne tombe entre, rien n'est écrit deux fois en se
   contredisant.

## Ce que tu rends

- **Les findings** : sévérité (CRITICAL / HIGH / MEDIUM / LOW), l'endroit exact, **la preuve**, ce
  qu'il faut changer. Un reproche au dépôt de ne pas **encore** faire ce que la fiche prescrit n'est
  **pas** un défaut de la spec.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.**

## Interdits

⛔ N'écris aucun fichier du dépôt, n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante — `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make` dans `docs/manual/`, tout `git commit`/`push`/
`add`/`stash`/`reset`/`rebase`/`checkout`, `sqlx migrate`, `cargo test`/`cargo nextest`, `npm run`.
Lecture, `grep`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` (vers le scratchpad) et
`cargo check` sont autorisés.
