# Release 0.17.2 — registre des choix autonomes

Registre de la release, mis à jour **par union** (une section par story, jamais d'écrasement ; un doublon se renumérote).
Chaque choix : contexte, option retenue, alternatives écartées, réversibilité. Numéros de ligne de `d7ec2ea2`.

---

## Story v0172-annulees — repasser en brouillon une facture fournisseur annulée (#868)

Fiche : `v0172-annulees.md`. Clé `v0172a`. Issue #868, en `refs` seulement.

### C-v0172a-1 — Aucune migration

- **Contexte** : la 0.17.1 tient des livres réels ; toute migration rendrait le retour à la 0.17.1 dépendant de la
  sauvegarde (consigne de la release).
- **Retenu** : aucune migration. Le statut `draft`, la colonne `purchase_journal_entry_id` nullable, la contrainte
  `chk_supplier_invoices_entry_iff_not_draft` et l'audit existent ; le geste n'écrit que des états que la 0.17.1 connaît.
- **Écartées** : une colonne « annulée puis repassée en brouillon » ou une table d'historique — l'audit porte déjà
  l'histoire (`previousStatus`), et la colonne imposerait P1–P8 pour rien.
- **Réversible** : oui (retour à la 0.17.1 sans restauration ; les brouillons obtenus y restent brouillons).

### C-v0172a-2 — Annulée → brouillon par suppression des deux écritures, contre-passation d'abord, par le socle

- **Contexte** : `delete_in_tx` refuse une écriture contre-passée (étape 3-bis) et la FK `reverses_entry_id` est `RESTRICT`.
- **Retenu** : `delete_in_tx(contre-passation)` puis `delete_in_tx(achat)`, `enforce_ownership = false`, dans la transaction
  de la dévalidation ; `delete_in_tx` n'est pas modifiée. Chaque écriture subit ainsi les gardes d'exercice, d'exercice
  postérieur, de période et de lettrage, et laisse son `journal_entry.deleted`.
- **Écartées** : (a) un `DELETE` SQL direct de la paire — contourne les gardes et l'instantané d'audit ; (b) un drapeau
  « supprimer aussi la contre-passation » dans `delete_in_tx` — élargit le socle que d'autres appelants partagent ;
  (c) garder l'annulée et en créer une copie brouillon — doublon de facture, numéro du fournisseur en double.
- **Réversible** : oui (code).

### C-v0172a-3 — Lettrage : seules les paires `reversal` de l'annulation se dissolvent, en mode `System`

- **Contexte** : la contre-passation a lettré chaque ligne libre de l'achat sur compte lettrable avec son miroir (groupe
  `reversal`) ; la dissolution manuelle en est refusée (ligne possédée par une pièce).
- **Retenu** : dissoudre par `letterings::dissolve_group_in_tx` en mode `System`, exercice tenu = celui de la
  contre-passation (verrouillé `FOR UPDATE`, ouvert), chaque groupe d'origine `reversal` dont toutes les lignes sont de
  l'achat ou de sa contre-passation. Tout autre lettrage reste, et la garde 3-quinquies de `delete_in_tx` refuse
  (`ENTRY_LETTERED`, « délettrez-la d'abord »).
- **Écartées** : dissoudre tout groupe des deux écritures — effacerait un lettrage manuel qui relie une ligne à une autre
  pièce, en silence.
- **Réversible** : oui.

### C-v0172a-4 — Pré-contrôle de l'exercice clos avant la dissolution

- **Contexte** : le mode `System` suppose que l'appelant tient un exercice **ouvert** ; la dissolution vient avant
  `delete_in_tx`, qui juge l'exercice.
- **Retenu** : après les verrous des deux écritures et de leurs exercices, un exercice `Closed` → `FiscalYearClosed`
  (le code que `delete_in_tx` rendrait), avant toute écriture. *Annoté le 2026-11-04 (validation P1, V1-3)* : la mutation
  M12 n'est **pas** équivalente — sans le pré-contrôle, un achat dans un exercice clos dont la contre-passation est sous le
  verrou de période rend `PERIOD_LOCKED` (la contre-passation parle d'abord) ; elle est tuée par la paire *P4* d'AC-5.
- **Écartées** : s'en remettre au rollback — juste en résultat, mais viole le contrat écrit du mode `System`.
- **Réversible** : oui.

### C-v0172a-5 — Annulée alors que payée : refus `DetachedPayment` tant que le paiement existe non contre-passé

- **Contexte** : le gel du paiement détaché (`modification_guard`, palliatif de #541) joint l'audit à une facture
  `status = 'cancelled'` ; repasser l'annulée en brouillon le lèverait en silence, rendant supprimable un paiement réel.
- **Retenu** : refus, avec le numéro de l'écriture de paiement et, si elle est rapprochée, la date de la transaction ;
  passe si le paiement est contre-passé (gardé alors par l'étape 3-bis) ou absent. Lecture d'audit **partagée** avec
  `modification_guard` (une fonction).
- **Écartées** : (a) refuser toute annulée qui a été payée, sans issue — aucun chemin pour une saisie fautive payée par
  erreur ; (b) laisser passer — perte silencieuse du gel ; (c) déplacer le gel sur une colonne — migration, et c'est #541.
- **Réversible** : oui.

### C-v0172a-6 — Payée dont le règlement est rapproché : motif `PaidReconciled`

- **Contexte** : « annulez d'abord son règlement » envoie vers un geste que la fiche refuse quand le règlement est
  rapproché ; le chemin réel est *Annuler le rapprochement* (`reconciliation_cancel`).
- **Retenu** : variante neuve, code `SUPPLIER_INVOICE_PAID_RECONCILED`, `document_id` = transaction, date au détail.
- **Écartées** : réutiliser `MatchedBankTransaction` — son texte parle de l'écriture d'achat ; un détail optionnel sous
  `SUPPLIER_INVOICE_PAID` — deux états du monde sous un code, que l'écran devrait départager.
- **Réversible** : oui.

### C-v0172a-7 — Les messages nomment l'objet à défaire (lot, transaction, écriture)

- **Contexte** : le Project Lead veut que le refus dise quoi faire d'abord ; les textes suffixaient le numéro de la facture.
- **Retenu** : `details.transactionDate`, `entryNumber`, `batchId` ; clés FTL à arguments ; le suffixe du numéro de la
  facture retiré des messages qui nomment leur objet. *Annoté le 2026-11-04 (validation P1, V1-11)* : cinq champs, dont
  `entryId` et `entryDate` (le numéro d'écriture repart à 1 à chaque exercice) — C-v0172a-22.
- **Écartées** : des liens cliquables dans le message — refonte de l'affichage des refus, hors périmètre.
- **Réversible** : oui.

### C-v0172a-8 — Le bouton s'affiche sur validée, payée et annulée ; le refus vient du serveur au clic

- **Contexte** : le Project Lead ignorait l'existence du geste ; il nomme « payée » parmi les refus de l'action.
- **Retenu** : bouton pour `open`, `paid`, `cancelled` (rôles qui écrivent) ; aucune pré-lecture des empêchements.
- **Écartées** : une pré-lecture `unvalidateBlockedBy` au `GET` (patron d'*Annuler la facture*) — exige une lecture qui
  rejoue les gardes de `delete_in_tx` sur deux écritures, donc une seconde précédence à tenir égale à la première ; à
  rouvrir si l'usage le demande.
- **Réversible** : oui.

### C-v0172a-9 — « Dévalider » devient « Repasser en brouillon », côté fournisseur seulement

- **Contexte** : le mot du Project Lead ; le libellé « Dévalider » ne disait pas ce que fait le geste.
- **Retenu** : renommer les valeurs des clés existantes (`supplier-invoices-unvalidate`, `…-confirm-title`) dans les quatre
  locales ; garder les clés, les `data-testid`, la route et le nom de code `unvalidate`.
- **Écartées** : renommer aussi la fiche client — hors de la décision ; renommer les clés — churn sans effet visible.
- **Réversible** : oui.

### C-v0172a-10 — Même action d'audit `supplier_invoice.unvalidated`, `details` enrichis

- **Contexte** : un libellé d'action neuf exigerait un libellé au registre `audit_labels.rs` et des traductions.
- **Retenu** : même action ; `previousStatus` et, pour une annulée, `reversalJournalEntryId`.
- **Écartées** : `supplier_invoice.redrafted` — un second nom pour le même geste.
- **Réversible** : oui (les traces écrites restent lisibles).

### C-v0172a-11 — Rôles inchangés

- **Contexte** : la dévalidation est ouverte au Comptable et à l'Administrateur (route `unvalidate`, Comptable+, clés
  `read-write`) ; la suppression d'un brouillon à l'Administrateur seul.
- **Retenu** : repasser en brouillon = Comptable+ (comme la dévalidation) ; supprimer le brouillon = Administrateur.
- **Écartées** : réserver l'annulée à l'Administrateur — le geste est sous les mêmes gardes que la dévalidation, et la
  suppression, elle, reste réservée.
- **Réversible** : oui.

### C-v0172a-12 — Le document rattaché reste joint au brouillon

- **Contexte** : le mandat demande « le même sort que pour la dévalidation d'une `open` » ; lu au code : `unvalidate_in_tx`
  ne touche pas au document ; `delete_draft` le rend « à traiter ».
- **Retenu** : idem pour l'annulée.
- **Écartées** : détacher au passage en brouillon — le brouillon se revalide le plus souvent avec sa pièce.
- **Réversible** : oui.

### C-v0172a-13 — Historique de lot : le brouillon ne se supprime toujours pas

- **Contexte** : `delete_draft_in_tx` refuse un brouillon présent dans un lot de paiement, quel qu'en soit le sort
  (limite L8, `SupplierInvoiceHasPaymentBatchHistory`) ; une annulée qui a figuré dans un lot devient un tel brouillon.
- **Retenu** : la règle L8 (`SupplierInvoiceHasPaymentBatchHistory`) est inchangée ; le manuel le dit.
- **Écartées** : la lever pour une annulée — l'ordre transmis à la banque se garde (`RESTRICT`).
- **Réversible** : oui.

### C-v0172a-14 — Factures clientes : écart constaté, rien ne change

- **Contexte** : `invoices::unvalidate` refuse `cancelled` ; une cliente n'est annulée que par un avoir (document
  numéroté, souvent envoyé), déjà refusé par le motif `Credited`.
- **Retenu** : ne rien changer ; issue demandée à l'orchestrateur pour le tracer.
- **Écartées** : supprimer un avoir pour rouvrir la facture — décision de facturation que le Project Lead n'a pas prise.
- **Réversible** : oui.

### C-v0172a-15 — Limite L1 : les messages d'exercice, de période et de lettrage parlent de « l'écriture d'achat »

*Annoté le 2026-11-04 (validation P1, V1-2) : la contre-passation parle avant l'achat pour les refus de `delete_in_tx`
(C-v0172a-19) ; la limite vaut pour les refus 6 et 7a du tableau.*

- **Contexte** : pour une annulée, le refus peut venir de la contre-passation — seulement si l'achat est daté après son
  annulation.
- **Retenu** : textes inchangés, limite écrite.
- **Écartées** : des clés par écriture et par statut — six textes de plus pour un cas marginal.
- **Réversible** : oui.

### C-v0172a-16 — Annulée sans contre-passation liée : `Invariant`

- **Contexte** : avant la Story 25-3-c, l'annulation réécrivait sa contre-passation sans `reverses_entry_id`.
- **Retenu** : `reversed_by` vide → `DbError::Invariant`. Hors fenêtre : ces données ne viennent que d'une sauvegarde
  antérieure à la v0.12, que l'import refuse (inventaire des tables différent).
- **Écartées** : chercher la contre-passation par l'audit — code mort qui paraît fonctionner.
- **Réversible** : oui.

### C-v0172a-17 — Le chemin d'une payée par la réconciliation se prouve par un test d'intégration `kesh-db`

- **Retenu** : T10 de bout en bout (refus → dé-rapprochement → brouillon → suppression) ; l'E2E vérifie l'écran
  (bouton et message) sur le parcours de réconciliation déjà présent dans la spec.
- **Écartées** : tout le parcours en E2E — import, réconciliation et suppression : lent et redondant avec T10.
- **Réversible** : oui.

### C-v0172a-18 — Lecture du droit : écrite dans la fiche et résumée au manuel, renvoi à la fiduciaire

- **Retenu** : CO 957a al. 2 ch. 5, CO 958f, Olico art. 3 et 9 ; « rien ne disparaît sans trace », paire nette à zéro,
  avant clôture, hors période verrouillée, audit et sauvegarde.
- *Annoté le 2026-11-04 (validation P1, V1-13)* : le manuel ne cite que CO 957a, CO 958f et OLICo art. 9 al. 1 let. b,
  que le dépôt appuie déjà ; l'art. 3 de l'Olico reste dans la fiche, marqué non vérifié, jusqu'à confirmation du
  Project Lead ou de sa fiduciaire.
- **Écartées** : affirmer la conformité sans réserve — Kesh n'est pas conseil juridique.
- **Réversible** : oui.

### C-v0172a-19 — Précédence composée par écriture, contre-passation d'abord (validation P1, V1-2, V1-10)

- **Contexte** : la fiche énonçait un ordre total des refus ; deux appels successifs de `delete_in_tx` rendent les refus
  de la contre-passation avant ceux de l'achat, et le lot `generated` est refusé avant la lecture d'audit.
- **Retenu** : rangs 0 à 6 par le dépôt (statut, version, lot, paiement détaché, rapprochement, pré-contrôle d'exercice
  clos), puis `delete_in_tx` sur la contre-passation, puis sur l'achat ; écrit au tableau, testé par paires montées pour
  que les deux refus jouent (AC-5, P1 à P6), conséquence *P5* assumée.
- **Écartées** : étendre le pré-contrôle à l'exercice postérieur et au verrou des deux écritures — une seconde copie de
  la précédence de `delete_in_tx`, qui divergerait.
- **Réversible** : oui.

### C-v0172a-20 — Vocabulaire : « repasser en brouillon » dans tous les textes fournisseur (validation P1, V1-6)

- **Contexte** : le bouton change de nom ; les messages qui disent quoi faire renvoyaient à « dévalider », geste que
  l'utilisateur ne trouve plus sous ce nom.
- **Retenu** : quinze valeurs fournisseur modifiées et cinq clés neuves (AC-15), dans les quatre locales (fr « repasser
  en brouillon », de « in den Entwurf zurücksetzen », it « riportare in bozza », en « return to draft ») ; manuel
  utilisateur côté fournisseur ; glossaire. « Dévalider » reste le mot des factures clientes et le nom de code.
- **Écartées** : garder « dévalider » dans les messages — renvoie à un geste introuvable, ce que la décision du Project
  Lead (point 3) vise.
- **Réversible** : oui.

### C-v0172a-21 — Payée : appel direct, sans boîte de confirmation (validation P1, V1-14)

- **Contexte** : la boîte de la dévalidation annonce une suppression ; pour une payée, le serveur refuse toujours.
- **Retenu** : pour `paid`, le bouton appelle la route sans boîte ; aucune écriture ne peut suivre (statut refusé au
  rang 1 ; une facture redevenue validée ailleurs a une version incrémentée par l'annulation du règlement,
  `supplier_invoices.rs:2427`, refusée au rang 2).
- **Écartées** : une boîte dont le texte annonce une suppression qui n'aura pas lieu ; masquer le bouton — le Project
  Lead veut que le geste se trouve.
- **Réversible** : oui.

### C-v0172a-22 — Cinq champs de `details`, toujours présents (validation P1, V1-11)

- **Contexte** : les messages nomment un lot, une transaction, une écriture ; le numéro d'écriture repart à 1 à chaque
  exercice (`journal_entries.rs:2908-2915`).
- **Retenu** : `batchId`, `transactionDate`, `entryId`, `entryNumber`, `entryDate`, `null` quand ils ne s'appliquent
  pas ; l'écriture se nomme « n° N du JJ.MM.AAAA ».
- **Écartées** : le nom de l'exercice — plus long, et la date suffit à lever l'ambiguïté.
- **Réversible** : oui.
