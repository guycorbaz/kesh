# Release 0.17.1 — registre des choix autonomes de la Story v0171-correctifs

Registre propre à la story (`docs/agents-orchestration.md` § 1), à verser **par union** au registre de la release.
Fiche : `v0171-correctifs.md`. Numérotation `C-v0171-k`. Issues : #854 (P2), #866 (P4), #871 (P4), en `refs` seulement.
Numéros de ligne de `910e5d5a`. Chaque choix : contexte, option retenue, alternatives écartées, réversibilité.

---

### C-v0171-1 — #854 : le préfiltre accepte les deux séparateurs, le parseur ne bouge pas

- **Contexte** : `first_spc` écarte tout payload qui ne commence pas par `SPC\n0200` (`qr_decode.rs:31, :261`) ; le parseur
  tolère déjà CR+LF (`parser.rs:95`).
- **Retenu** : une règle unique (constante-tableau ou fonction) qui retient `SPC` puis `0200` séparés par `\n` ou `\r\n` ;
  le payload part au parseur tel quel.
- **Écartées** : (a) normaliser le payload (`\r\n` → `\n`) avant le test — modifie ce que voit le parseur et masque
  un `\r` résiduel au lieu de le prouver absent ; (b) supprimer le préfiltre et tout confier au parseur — un QR non SPC
  (lien, autre norme) deviendrait `INVALID_SPC_PAYLOAD` au lieu de « aucune QR-facture ».
- **Réversible** : oui.

### C-v0171-2 — #854 : seuls LF et CR+LF sont admis

- **Contexte** : la norme SIX 2.2 admet LF et CR+LF comme séparateurs de lignes du payload.
- **Retenu** : ces deux-là ; un `\r` seul (`SPC\r0200`) n'est pas retenu (test négatif).
- **Écartées** : tolérer le CR seul — hors norme, et le parseur découpe sur `\n`, il ne verrait qu'une ligne.
- **Réversible** : oui.

### C-v0171-3 — #854 : un `\r` interne à une ligne n'est pas traité

- **Contexte** : le parseur retaille le `\r` **final** de chaque ligne ; un `\r` au milieu d'un champ resterait.
- **Retenu** : hors périmètre ; aucune pièce connue n'en porte, la norme ne l'autorise pas. La story prouve l'absence de `\r`
  **résiduel du séparateur** (AC-3), pas l'absence de tout `\r`.
- **Écartées** : filtrer tous les `\r` de tous les champs — élargit sans cas réel, au risque d'altérer un texte légitime.
- **Réversible** : oui.

### C-v0171-4 — #854 : aucune modification du parseur attendue ; si le test T6 la rend nécessaire, elle est faite et consignée

- **Contexte** : la tolérance du parseur n'a aucun test (`grep` : seul le commentaire de `parser.rs:94`) ; l'orchestrateur l'a
  constatée par exécution sur les pièces réelles.
- **Retenu** : T6 fige cette tolérance sur payload synthétique ; le correctif de production éventuel est mécanique et
  s'intègre (même défaut, même chemin).
- **Écartées** : ouvrir une issue pour un défaut non constaté.
- **Réversible** : oui.

### C-v0171-5 — #854 : le second rendu à haute résolution de la v017-5 reste en place

- **Contexte** : le commentaire de l'issue du 2026-11-03 le dit « utile pour d'autres scans, mais pas en cause ici ».
- **Retenu** : ne pas y toucher ; le manuel (« Faire relire le QR d'un document ») reste vrai.
- **Écartées** : le retirer (coût de la seconde passe pour rien) — sans lien avec le défaut, et une régression possible.
- **Réversible** : oui.

### C-v0171-6 — #866 : `readableAmount(raw, 0)`, importé du module du formulaire fournisseur ; aucun helper créé

- **Contexte** : `readableAmount` (`supplier-invoice-form.ts:213`) fait exactement `1.0000` → `1`, `33.3333` gardé ; la consigne
  est de ne créer aucun helper s'il en existe un. `formatSwissAmount` (décimales fixes, apostrophes) ne convient pas à une quantité.
- **Retenu** : l'appeler aux trois fiches (fournisseur, cliente, avoir) ; les fiches clientes importent donc un module du dossier
  fournisseur. Pas d'apostrophe de milliers (`1250`), comme le champ de saisie fournisseur.
- **Écartées** : (a) le déplacer dans un module commun — remaniement d'import sans effet visible, à faire seul si l'on y tient ;
  (b) un `formatQuantity` neuf — doublon.
- **Réversible** : oui.

### C-v0171-7 — #866 : le champ de saisie d'une facture cliente en brouillon reste hors de la story — issue demandée

- **Contexte** : `InvoiceForm.svelte:113` recopie `l.quantity` brut dans le champ ; à l'ouverture d'un brouillon client le champ
  porte `1.0000`. Même cause, mais c'est un champ de saisie dont la valeur repart au serveur (comparaison de dirty-state,
  payload).
- **Retenu** : ne pas l'intégrer ; **demander l'issue** (« on trace toujours »), P4, symétrique client de v017-4b AC-23.
- **Écartées** : le corriger ici — non mécanique (touche l'état du formulaire).
- **Réversible** : oui.

### C-v0171-8 — #871 : la prémisse est fausse sur le code publié ; vérifier par exécution, puis commenter l'issue

- **Contexte** : `supplier-invoices/[id]/+page.svelte:231` formate déjà la date ; B2-4 de la revue P2 de la v017-4b l'a remédié
  avant la release, avec `supplier-settlement-page.test.ts:202-205`. L'issue (08:32 le 2026-11-03) décrit l'état d'avant.
- **Retenu** : AC-9 (essai par exécution, texte de la confirmation capturé) ; **commentaire de l'orchestrateur** sur l'issue ; elle
  se ferme à la PR de release si l'essai confirme. Le travail réel de #871 devient l'inventaire des autres dates (C-v0171-9).
- **Écartées** : « corriger » un site déjà correct pour honorer l'énoncé ; ou ne rien dire.
- **Réversible** : oui.

### C-v0171-9 — #871 : les dates des lots de paiement sont corrigées avec celles des factures fournisseur

- **Contexte** : les lots de paiement (`payment-batches/+page.svelte:239`, `[id]/+page.svelte:97, :104`) sont les lots de #871
  (« la date du lot ») ; mêmes helper et cause.
- **Retenu** : `formatSwissDate` à sept sites ; `confirmedAt` passe par `slice(0, 10)` (horodatage naïf).
- **Écartées** : se limiter à la fiche d'une facture fournisseur — laisserait « Confirmé le 2026-01-29 » sur la fiche du lot que la
  confirmation d'annulation cite en `29.01.2026`.
- **Réversible** : oui.

### C-v0171-10 — Dates ISO des écrans clients : issue demandée

- **Contexte** : `invoices/+page.svelte:413`, `invoices/[id]/+page.svelte:1135, :1139`, `credit-notes/+page.svelte:64`,
  `credit-notes/[id]/+page.svelte:153`, `invoices/reminders/+page.svelte:345`, `invoices/due-dates/+page.svelte:450`
  affichent l'ISO. Défaut voisin, hors « factures fournisseur ».
- **Retenu** : ne pas l'intégrer (périmètre minimal, cinq écrans, release corrective) ; **demander l'issue** P4.
- **Écartées** : tout corriger ici.
- **Réversible** : oui.

### C-v0171-11 — Manuel : une phrase ajoutée, aucune retirée

- **Contexte** : le manuel ne décrit ni la quantité ni les dates des écrans fournisseur ; il décrit la lecture du QR et la
  relecture d'un document (`user-manual.tex:1965-2030`).
- **Retenu** : dire la lecture du CR+LF et la conduite pour un document déjà entré ; PDF à régénérer par le développeur.
  Le dire aussi s'il ne change rien : ici, il change.
- **Réversible** : oui.

### C-v0171-12 — Les documents déjà entrés « sans QR » ne sont pas relus automatiquement

- **Contexte** : les deux pièces de Guy sont entrées « Aucune QR-facture exploitable » ; la relecture existe déjà à la
  réactivation (retirer de la liste, redéposer — v017-5), sans doublon.
- **Retenu** : ne rien relire de soi-même au démarrage (pas de migration, pas de tâche de fond sur des livres réels) ; CHANGELOG
  et manuel disent le geste.
- **Écartées** : une relecture au démarrage de tous les documents `qr: none` — écriture sur la base de production au
  premier boot, hors d'une release corrective « sans migration ».
- **Réversible** : oui.
