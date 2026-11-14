# Epic 36 — registre des choix autonomes de la story 36-3b (clé `36-3b`)

Choix faits à la rédaction de la fiche `36-3b-facture-prix-unitaire-ambigu.md` (2026-11-14, tête du worktree
`b1cd53d28`), hors de ce que C-e360-17 tranche déjà (clé, phase 1, lignée A, voie 2, réemploi du prédicat et du lecteur
de la 36-3 **sans les modifier**, périmètre borné au prix unitaire de la facture client, serveur inchangé). Chaque choix :
contexte, option retenue, options écartées, réversibilité.

### C-36-3b-1 — Le prix unitaire suit la grammaire du journal, sans variante

- **Contexte** : le serveur admet un prix positif ou nul à quatre décimales au plus, sans normaliser l'échelle
  (`crates/kesh-api/src/routes/invoices.rs:506-520`, `limits.rs:28` `pub const MAX_DECIMAL_SCALE: u32 = 4;`) ; aucune
  règle du centime ne vise la ligne de facture client (#687 ne porte que sur la facture fournisseur). La grammaire du
  journal (`readJournalAmount`, `balance.ts:110`) admet quatre décimales au plus, l'apostrophe des milliers, aucun signe.
- **Retenu** : `readJournalAmount` appelé tel quel ; aucune grammaire propre au prix. Conséquences dites dans la fiche :
  la virgule décimale (« 1,25 ») et l'apostrophe (« 1'250 ») sont désormais admises ; « -5 », « 1e3 », « .5 », « 5. »,
  que `new Big` acceptait ou refusait autrement, sont « illisibles ».
- **Écartées** : (a) une variante à signe pour garder le message « prix unitaire ≥ 0 » — une seconde grammaire pour un
  cas que le serveur refuse de toute façon ; (b) la grammaire de la 35-6 — plus large que la décision de Guy (C-e360-6).
- **Réversible** : oui ; refus à l'écran seulement.

### C-36-3b-2 — Le prix vide reste refusé : une charge propre au prix, non `journalAmountPayload`

- **Contexte** : `journalAmountPayload('')` rend `'0'` (`balance.ts:143-147`) — juste pour un débit vide, faux pour un
  prix : un prix effacé partirait à zéro sans rien dire. Aujourd'hui, `new Big('')` lève et la ligne est refusée
  « valeurs numériques invalides » (`InvoiceForm.svelte:509-518`).
- **Retenu** : une fonction pure `unitPricePayload(raw)` dans `invoice-helpers.ts` — `payload` d'une lecture `clear`,
  `null` sinon (vide compris) ; le vide garde son message d'aujourd'hui.
- **Écartées** : (a) `journalAmountPayload` — envoie 0 en silence ; (b) le modifier — interdit par C-e360-17.
- **Réversible** : oui.

### C-36-3b-3 — Un prix enregistré au millième se rouvre sur quatre décimales

- **Contexte** : `readableAmount(raw, 2)` (`supplier-invoice-form.ts:214-224`) rend « 1.234 » pour « 1.2340 » et
  « 12.345 » pour « 12.3450 » (constaté à l'exécution, `node`, big.js du dépôt) : un brouillon, un rechargement ou un
  article au prix légitime d'un franc vingt-trois et quatre dixièmes de centime serait refusé comme ambigu.
- **Retenu** : `unitPriceFieldValue(raw)` dans `invoice-helpers.ts` : `readableAmount(raw, 2)`, sauf si
  `readJournalAmount` la juge ambiguë — alors `new Big(raw).toFixed(4)` (« 1.2340 »), forme jamais ambiguë (quatre
  décimales). Appelée aux trois sites (`:122`, `:453`, `:650`).
- **Écartées** : (a) modifier `readableAmount` — partagé par la facture fournisseur et trois écrans de détail ;
  (b) toujours quatre décimales — défait la lisibilité de la v0.17.1 (#878, `100.00`).
- **Réversible** : oui.

### C-36-3b-4 — Messages : les clés de la 36-3 sous le champ, deux clés neuves au refus

- **Contexte** : les textes `journal-entry-form-amount-ambiguous` et `-unreadable` (`fr-CH/messages.ftl:313-314`)
  disent vrai d'un prix. Le refus à l'envoi passe par `validateClient`, qui nomme la ligne (`Ligne N : …`), et le bandeau
  `errorMsg` (`:698-701`). Le lint de propriété ne parcourt que `src/lib/features` (`lint-i18n-ownership.js:17`, `:255`) :
  `components/invoices/` n'y est pas soumis.
- **Retenu** : sous le champ, les deux clés de la 36-3 (fallback recopié à l'identique de `JournalEntryForm.svelte`) ; au
  refus, `invoice-line-unit-price-ambiguous-error` et `invoice-line-unit-price-unreadable-error`, avec `{ $line }`, sur le
  patron d'`invoice-line-vat-rate-disabled-error`. Le bouton *Créer la facture* / *Enregistrer* reste actif ; le refus a
  lieu à l'envoi (clic ou *Entrée*), comme pour une quantité invalide.
- **Écartées** : (a) préfixer en dur « Ligne N : » devant la clé de la 36-3 — message à moitié français en de/it/en ;
  (b) désactiver le bouton — nouvel état du bouton, avec son infobulle, pour un refus qui se dit déjà deux fois.
- **Réversible** : oui.

### C-36-3b-5 — Le total d'une ligne refusée vaut 0.00

- **Contexte** : `lineTotal` et `totalAmount` (`:473-477`) lisent le prix par `new Big` ; un prix illisible y compte
  aujourd'hui pour zéro (`invoice-helpers.ts:29-35`, `:44-50`), « 1.250 » pour 1,25.
- **Retenu** : les totaux lisent `unitPricePayload(l.unitPrice) ?? '0'` ; `computeLineTotal` et `computeInvoiceTotal`
  ne changent pas. Une ligne ambiguë affiche 0.00, à côté de son message.
- **Écartées** : (a) garder 1,25 — c'est le défaut ; (b) un tiret — un nouvel état visuel de la colonne.
- **Réversible** : oui.

### C-36-3b-6 — Les doc-comments de la 36-3 ne sont pas retouchés

- **Contexte** : `readJournalAmount` (`balance.ts:96-109`) et `amount-ambiguity.ts:1-13` nomment leurs appelants
  (écriture manuelle, assistant TVA achat, solde d'ouverture) ; C-e360-17 les veut « sans les modifier ».
- **Retenu** : aucun changement dans ces deux fichiers ; le doc-comment des deux fonctions neuves d'`invoice-helpers.ts`
  nomme `readJournalAmount` et la story. Une revue qui demanderait d'y ajouter la facture client le relèvera en LOW
  (`documentation`).
- **Écartées** : ajouter la facture client aux deux doc-comments — touche deux fichiers de la 36-3 pour un commentaire.
- **Réversible** : oui.

### C-36-3b-7 — La charge relit le prix après l'attente des réglages

- **Contexte** : `onSubmit` valide (`:552`), puis attend `getInvoiceSettings()` (`:562`) avant de construire la charge
  (`:588-595`) : un prix retapé pendant l'attente partirait sans contrôle — aujourd'hui en chaîne brute, que le serveur
  juge.
- **Retenu** : la charge prend `unitPricePayload` de chaque ligne ; une valeur `null` arrête l'envoi et affiche
  `validateClient()`, `submitting` remis à faux. Testé (le mock de `getInvoiceSettings` change le prix avant de
  résoudre).
- **Écartées** : envoyer `payload!` sans garde — une course rendrait `"unitPrice": null`.
- **Réversible** : oui.
