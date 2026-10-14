# Epic 29 — registre des choix autonomes de la Story 29-8

*Registre propre à la story, à verser **par union** au registre de l'epic (`epic-29-choix-autonomes.md`). Fiche :
`29-8-transaction-reglee-ailleurs.md`. Issue : #670. Relevés sur `fcd2ae75`.*

---

### C-29-8-1 — Les prémisses de #670 sont exactes

- **Contexte** : #670 affirme qu'une transaction réglée depuis la fiche, puis rejetée, reste `pending` et bloque à
  jamais la clôture du compte bancaire.
- **Retenu** : établies au code une à une (fiche, § *Les prémisses*) — toutes exactes. Aucune correction à porter à
  l'issue.
- **Réversible** : sans objet.

### C-29-8-2 — Aucune migration : `matched_entry_id` et `reconciled` suffisent

- **Contexte** : rattacher demande un lien transaction → écriture.
- **Retenu** : la colonne `bank_transactions.matched_entry_id` et le statut `reconciled` existent depuis la Story 8-1b ;
  le rattachement les pose comme les cinq autres chemins.
- **Écartée** : une colonne d'origine du lien (« rattachée » contre « créée ») — elle seule permettrait un « détacher
  sans annuler » (L1), au prix d'une migration ; hors périmètre minimal.
- **Réversible** : oui (une migration ADD COLUMN nullable plus tard).

### C-29-8-3 — Piste (a), restreinte aux règlements clients par virement

- **Contexte** : #670 propose (a) rattacher, (b) choisir une facture à la réconciliation (#666), (c) un statut
  « écartée ».
- **Retenu** : (a), restreinte aux lignes `invoice_settlements` de type `bank_transfer` : l'état produit est celui
  d'une proposition « facture » acceptée, que toutes les gardes en aval traitent déjà (rang 3 de l'annulation,
  dévalidation, suppression d'import, *Annuler le rapprochement*).
- **Écartées** : (b) ne couvre pas la 29-6 (trop-perçu refusé par `classify_payment`) ni les règlements déjà saisis,
  et exige une seconde borne (#548) ; (c) migration breaking et faux rattachement **muet** (une transaction jamais
  comptabilisée sortirait des contrôles de clôture) ; (a) généralisée à toute écriture — *Annuler le rapprochement*
  contre-passerait une écriture saisie au journal (`ReconciliationKind::Entry`) : il faudrait d'abord C-29-8-2
  écartée.
- **Réversible** : oui (élargir plus tard, après une colonne d'origine).

### C-29-8-4 — Les quatre conditions du rattachement

- **Contexte** : un faux rattachement doit être impossible ou visible.
- **Retenu** : transaction `pending` crédit CHF ; règlement de la société, `bank_transfer`, **même compte bancaire** ;
  écriture rapprochée d'aucune transaction ; écart `0 ≤ t.amount − M ≤ 0,05`, `M` lu sur le compte du grand livre
  actuel du compte bancaire.
- **Écartée** : comparer `invoice_settlements.amount` (montant réglé, qui peut différer du payé d'un demi-centime
  d'arrondi) — la ligne de banque est l'argent.
- **Réversible** : oui.

### C-29-8-5 — Un écart de 0 à 5 centimes, dans un seul sens, affiché

- **Contexte** : le contournement de la 29-6 enregistre le **montant dû** ; la transaction le dépasse de 1 à
  5 centimes. Un rattachement à montant strictement égal laisserait ce cas bloqué.
- **Retenu** : `0 ≤ écart ≤ AMOUNT_TOLERANCE_HUNDREDTHS` (constante existante) ; l'écart s'affiche avant confirmation
  et s'écrit à l'audit. Négatif exclu (argent aux livres que la banque n'a pas reçu).
- **Écartée** : égalité stricte (29-6 non couverte) ; tolérance symétrique (aucun contournement ne la produit).
- **Réversible** : oui.

### C-29-8-6 — Pas de fenêtre de dates : la proximité trie et s'affiche

- **Contexte** : « dates proches — à établir ».
- **Retenu** : le dialogue de règlement préremplit la date au jour (`SettleInvoiceDialog.svelte:76-90`) et Guy importe
  un relevé par mois : une borne refuserait le cas nominal. Tri par écart puis jours d'écart ; colonne « jours ».
- **Écartée** : une borne de ± 30 ou ± 60 jours.
- **Réversible** : oui.

### C-29-8-7 — Un geste humain, sans présélection, absent des propositions

- **Contexte** : règle *Un appariement automatique propose, il ne crée jamais*.
- **Retenu** : liste de candidats, aucun présélectionné, confirmation explicite ; `GET /reconciliation/proposals`
  inchangé.
- **Écartée** : proposer le rattachement parmi les candidates de la liste (nouveau type de candidate, acceptation en
  lot).
- **Réversible** : oui.

### C-29-8-8 — Deux routes unitaires, sous `/reconciliation/transactions/{id}/`

- **Contexte** : forme de l'API.
- **Retenu** : `GET …/link-candidates`, `POST …/link` `{ settlementId }` ; le compte bancaire se lit sur la
  transaction (comme `…/cancel` et `…/unreject`). Pas un lot : le pattern batch ne s'applique pas.
- **Écartée** : un corps `{ bankAccountId, bankTransactionId, … }` comme `/manual` (redondant avec l'identifiant de
  chemin).
- **Réversible** : oui avant publication.

### C-29-8-9 — Une seule définition du « rattachable »

- **Contexte** : la liste et le refus ne doivent pas diverger.
- **Retenu** : condition 1 en fonction pure de `kesh-db` ; conditions 2 à 4 en **une** requête, rejouée par le `POST`
  restreinte à `s.id`. Mutations M1 à M13.
- **Écartée** : un classifieur Rust des motifs à côté de la requête (deux définitions).
- **Réversible** : oui.

### C-29-8-10 — Un seul code de refus, `409 RECONCILIATION_SETTLEMENT_NOT_LINKABLE`

- **Contexte** : motifs multiples (compte, montant, déjà rapproché, transaction non créditrice).
- **Retenu** : un code, `details.bankTransactionId` et `details.settlementId` ; l'écran n'offre que des candidats, le
  refus ne se voit que par une course ou un appel direct — conduite : relire la liste. Les 404 restent distincts
  (anti-énumération).
- **Écartée** : un `details.reason` par condition — il exigerait le classifieur écarté en C-29-8-9.
- **Réversible** : oui (ajouter un `reason` est compatible).

### C-29-8-11 — Rejouée sur interblocage, classée `SansEcritureAuJournal`

- **Contexte** : la route prend des verrous de ligne sur `invoice_settlements` et `journal_entries`.
- **Retenu** : enveloppe `retry_app_on_deadlock`, comme `post_manual` ; registre `Traced, SansEcritureAuJournal` (le
  registre n'interdit pas l'enveloppe, point (vi)).
- **Écartée** : non rejouée comme `…/unreject` (qui ne verrouille que sa transaction).
- **Réversible** : oui.

### C-29-8-12 — Ordre des verrous imposé, vue née après le dernier

- **Contexte** : B5 de la rétrospective 28 (naissance de la vue).
- **Retenu** : verrou nommé → `bank_transactions` (`t`) → `invoice_settlements` ⋈ `journal_entries` → lecture
  courante de `matched_entry_id` → lectures ordinaires. Ordre `s → je` aligné sur `cancel_settlement_in_tx` ; la ligne
  `invoices` n'est jamais prise. Absence de cycle à justifier au Dev Agent Record.
- **Réversible** : oui.

### C-29-8-13 — *Annuler le rapprochement* d'une transaction rattachée annule le règlement (L1)

- **Contexte** : sans colonne d'origine (C-29-8-2), l'état est indiscernable d'une proposition acceptée.
- **Retenu** : comportement existant gardé et écrit au manuel ; visible (contre-passation, facture rouverte),
  réparable, sans erreur comptable. Issue d'évolution demandée (« détacher sans annuler »).
- **Écartée** : refuser l'annulation d'une transaction rattachée (indiscernable sans migration).
- **Réversible** : oui.

### C-29-8-14 — Le bouton sur la ligne des propositions, pas au détail de l'import

- **Contexte** : où offrir le geste.
- **Retenu** : bouton de ligne *Rattacher à un règlement* sur les lignes crédit CHF de `ReconciliationProposals` ;
  une transaction rejetée passe d'abord par *Annuler le rejet* (l'API admet une rejetée, comme `/manual`).
- **Écartée** : un second point d'entrée au détail de l'import (deux écrans, deux tests E2E).
- **Réversible** : oui.

### C-29-8-15 — « Rattachez » remplace « rejetez » dans les contournements de la 29-2 et de la 29-6

- **Contexte** : les fiches 29-2 (C-29-2-8) et 29-6 (AC7) prescrivent « régler depuis la fiche, puis rejeter » ;
  même release v0.13.4.
- **Retenu** : la 29-8, développée après elles, réécrit ces phrases (manuel, CHANGELOG, `api-external.md` s'il y a
  lieu) et retire les incises « (#670) » devenues fausses. L'indication de l'écran de la 29-6 reste (elle parle des
  boutons de lot).
- **Écartée** : laisser les deux gestes au manuel (le rejet ne règle pas #670).
- **Réversible** : oui.

### C-29-8-16 — Pas de découpage ; inventaire clos des textes « cinq chemins »

- **Contexte** : règle de découpage (> 5 modules).
- **Retenu** : quatre modules de code (`kesh-db`, `kesh-api`, `kesh-i18n`, `frontend/…/reconciliation`) plus la
  documentation : une seule story. Les cinq textes qui supposent « une écriture créée par le rapprochement » sont
  inventoriés (AC7) et réécrits.
- **Réversible** : oui.
