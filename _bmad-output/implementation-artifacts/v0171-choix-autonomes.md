# Release 0.17.1 — registre des choix autonomes de la Story v0171-correctifs

Registre propre à la story (`docs/agents-orchestration.md` § 1), à verser **par union** au registre de la release.
Fiche : `v0171-correctifs.md`. Numérotation `C-v0171-k`. Issues : #854 (P2), #866 (P4), #871 (P4), #878 (P4), #879 (P4), en `refs` seulement.
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

*Remplacé le 2026-11-03 par C-v0171-13 : l'issue demandée est #878, intégrée à la story sur décision du Project Lead.*

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

*Remplacé le 2026-11-03 par C-v0171-14 : l'issue demandée est #879, intégrée à la story sur décision du Project Lead.*

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

---

*Amendement du 2026-11-03 (périmètre élargi à #878 et #879, demande du Project Lead).*

### C-v0171-13 — #878 : `readableAmount(raw, 0)` aux deux sites d'`InvoiceForm`

- **Contexte** : le champ quantité de la facture cliente est alimenté par `initLines` (`InvoiceForm.svelte:113`) et par `reloadInvoice` (`:610`),
  tous deux en recopie brute. Le formulaire fournisseur est déjà conforme (`formLineFromResponse`). Aucun autre champ de saisie de quantité
  (inventaire clos : deux champs ; les avoirs n'ont pas de formulaire).
- **Retenu** : le même helper que #866 (C-v0171-6), aux deux sites ; aucun changement au payload (le champ est envoyé tel quel,
  `"1"` est déjà la valeur d'une ligne neuve). La précision saisie est conservée : seuls les zéros de fin tombent.
- **Écartées** : (a) ne corriger que `initLines` — `reloadInvoice` ramènerait `1.0000` après un conflit de version ; (b) normaliser à l'envoi
  — inutile, le serveur lit un décimal.
- **Réversible** : oui.

### C-v0171-14 — #879 : dix-sept sites des écrans clients, quatre catégories d'exceptions closes, les autres écrans en issue

*Recompté le 2026-11-03 (validation P1, P1) : « seize » et « huit exceptions » étaient faux ; le texte de ce choix dit désormais dix-sept sites et quatre catégories (i) à (iv).*

- **Contexte** : listes et fiches des factures, des avoirs, des relances et des échéances (§ 10 de la fiche). Les écrans du journal, des
  exercices, de l'import bancaire et du rapprochement affichent aussi l'ISO (et des tests le figent) mais ne sont pas ceux de #879.
- **Retenu** : `formatSwissDate` aux dix-sept sites (horodatages après `.slice(0, 10)`) ; le grep de contrôle rend 0 site non enveloppé, hors les
  exceptions (champs `type="date"`, valeurs envoyées à l'API et calculs, tests de nullité, dont `emailedAt` aux `:1000, :1014`) ; les écrans cités ci-dessus se **tracent en issue**
  (« on trace toujours »).
- **Écartées** : tout l'application en une fois — une release corrective ne change pas le journal et les relevés bancaires, et leurs tests
  figent l'ISO.
- **Réversible** : oui.

### C-v0171-15 — #879 : les tests qui figeaient l'ISO sur les sites corrigés sont mis à jour, pas contournés

- **Contexte** : `invoice-pdf-page.test.ts:171-173, :220-222` assertent `"2026-03-02"`, `"2026-04-01"` pour `invoice-pdf-frozen-at`.
- **Retenu** : les assertions deviennent `"02.03.2026"` et `"01.04.2026"` ; c'est le changement voulu, consigné au Change Log.
- **Écartées** : laisser le `data-testid` lire une valeur ISO cachée — l'écran est ce qui compte.
- **Réversible** : oui.

### C-v0171-16 — Les phrases à argument `{ $date }` reçoivent la date formatée, non une chaîne ISO

- **Contexte** : « dernier le { $date } », « Annulé le { $date } », « Confirmer la suppression de la facture du { $date } » : l'argument est
  formaté **avant** `i18nMsg`, comme `supplier-invoices-settlement-cancel-batch-warning`. Les catalogues des quatre locales ne changent pas.
- **Retenu** : formater l'argument ; aucune clé i18n nouvelle, aucun texte modifié (pas de PDF de manuel à régénérer pour #878 et #879).
- **Réversible** : oui.

### C-v0171-17 — Un test par écran, avec un cas qui couvre les deux horodatages d'une même page

- **Contexte** : la fiche d'une facture cliente porte trois horodatages ; l'historique d'un rappel en porte deux.
- **Retenu** : les tests lisent chaque horodatage (mutation M7 : retirer le helper d'un seul site rougit exactement ce test).
- **Écartées** : un test unique par page qui ne lit qu'une date — laisserait les autres sans filet.
- **Réversible** : oui.

### C-v0171-18 — Issue demandée : dates ISO des autres écrans

- **Contexte** : journal (`journal-entries/+page.svelte:551`, `[id]:590`), exercices (`settings/fiscal-years`), import bancaire,
  rapprochement (§ 10 iv de la fiche).
- **Retenu** : l'orchestrateur ouvre l'issue (P4) ; ces écrans sont hors de #879. Les tests du rapprochement qui figent l'ISO sont à
  réviser avec elle.
- **Réversible** : oui.

---

*Remédiation de la validation P1 (2026-11-03).*

### C-v0171-19 — #878 : le prix unitaire du brouillon client est formaté avec la quantité (`readableAmount(raw, 2)`)

- **Contexte** : validation P1, P5. `InvoiceForm.svelte:114` et `:611` recopient `unitPrice: l.unitPrice` brut vers
  `<Input … bind:value={line.unitPrice} />` (`:803`) ; `unit_price DECIMAL(19,4)` (`20260416000001_invoices.sql:47`), échelle ≤ 4 comme la quantité
  (`MAX_DECIMAL_SCALE`, `invoices.rs:499`). Même défaut, mêmes deux sites que #878.
- **Retenu** : l'intégrer à #878 ; `readableAmount(l.unitPrice, 2)`, le formatage du formulaire fournisseur (`supplier-invoice-form.ts:247`) :
  deux décimales au moins, jamais de précision perdue (`100.2000` ⇒ `100.20`, `33.3333` gardé). Test et mutation M6-ter. `"100.00"` et
  `"100.0000"` sont de même valeur : le total ne change pas.
- **Écartées** : une issue distincte (C-v0171-7 l'a déjà fait pour la quantité, et #878 couvre le « champ de saisie d'une ligne ») ;
  `readableAmount(…, 0)` pour le prix (`100` au lieu de `100.00`, pire lisibilité d'un montant).
- **Réversible** : oui.

### C-v0171-20 — M6-bis retirée (mutation équivalente) ; « tel quel » de C-v0171-1 déclaré non testable

- **Contexte** : validation P1, P3 et P13. Sur le domaine du serveur (`DECIMAL(19,4)`, échelle ≤ 4, `MAX_QUANTITY` 1 000 000),
  `Number(x).toString()` rend la même chaîne que `readableAmount(x, 0)` ; et une normalisation `\r\n` → `\n` avant le parseur donnerait les
  mêmes résultats que le passage direct.
- **Retenu** : retirer M6-bis, au lieu de l'envoyer chercher un cas qui ne rougit pas ; déclarer que « le payload part tel quel » n'a pas de test
  (la garantie est la relecture du diff, `parse_spc_payload(p)`, `qr_decode.rs:262`), sans effet fonctionnel.
- **Écartées** : une valeur illisible (`NaN`) pour tuer M6-bis — comportement hors besoin ; un test d'espionnage du parseur — du code de
  production ajouté pour un test.
- **Réversible** : oui.
