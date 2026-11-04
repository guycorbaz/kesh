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
- *Annoté le 2026-11-04 (validation P3, V3-6)* : le partage est un **fragment SQL** constant employé par deux requêtes
  aux `WHERE` propres (AC-6, V2-12), non une fonction ; et la dévalidation examine **chaque** trace
  `supplier_invoice.cancelled` de la facture — une facture peut avoir été annulée, repassée, revalidée, repayée puis
  annulée de nouveau —, le refus nommant la première écriture de paiement non contre-passée (T5e, M6c). Écartée : ne lire
  que la dernière trace, qui laisserait sans gel le paiement d'un cycle antérieur.
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
- *Annoté le 2026-11-04 (validation P3, V3-1)* : l'action garde son nom, son **libellé affiché** change (C-v0172a-24).
- **Écartées** : `supplier_invoice.redrafted` — un second nom pour le même geste.
- **Réversible** : oui (les traces écrites restent lisibles).

### C-v0172a-11 — Rôles inchangés

- **Contexte** : la dévalidation est ouverte au Comptable et à l'Administrateur (route `unvalidate`, Comptable+, clés
  `read-write`) ; la suppression d'un brouillon à l'Administrateur seul.
- **Retenu** : repasser en brouillon = Comptable+ (comme la dévalidation) ; supprimer le brouillon = Administrateur.
- *Annoté le 2026-11-04 (validation P2, V2-3)* : une clé d'API `read-write` peut donc supprimer **deux** écritures d'une
  facture annulée ; l'avertissement du manuel d'administration sur les clés `read-write` (`admin-manual.tex:3010`) le dit.
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
- *Annoté le 2026-11-04 (validation P2, V2-6)* : la phrase précédente est inexacte. Une cliente annulée est refusée par son
  **statut** (`invoices.rs:1524-1528`, `IllegalStateTransition`), avant tout motif ; `Credited` ne vaut que pour une
  facture encore validée qui porte un avoir. L'avoir **est** la contre-passation.
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

- **Contexte** : le mandat (point 2) demande que la fiche vérifie au code le chemin payée → rapprochement annulé →
  validée → brouillon → supprimée, et qu'« un test E2E ou d'intégration » le parcoure, sans changer le refus.
- **Retenu** : T10 de bout en bout (refus → dé-rapprochement → brouillon → suppression) ; l'E2E vérifie l'écran
  (bouton et message) sur le parcours de réconciliation déjà présent dans la spec.
- **Écartées** : tout le parcours en E2E — import, réconciliation et suppression : lent et redondant avec T10.
- **Réversible** : oui.

### C-v0172a-18 — Lecture du droit : écrite dans la fiche et résumée au manuel, renvoi à la fiduciaire

- **Contexte** : le mandat demande que la fiche écrive pourquoi supprimer une écriture contre-passée, avant clôture, avec
  trace d'audit et sauvegarde, reste acceptable (CO 957a), et ce qu'en dit le manuel ; `delete_in_tx` cite l'art. 958f
  pour refuser la suppression d'une écriture contre-passée.
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
- *Annoté le 2026-11-04 (validation P2, V2-1, V2-2)* : **dix-sept** valeurs (deux clés omises :
  `supplier-invoices-validate-confirm-body`, `supplier-invoices-edit-not-draft`) et quatorze replis en dur ; le contrôle se
  fait **par liste de clés** et par racine propre à chaque langue (fr `[Dd][ée]valid`, en `[Uu]nvalidat`, de
  `Freigabe|[Ee]ntvalid`, it `[Cc]onvalida\b`), avec une sonde positive relevée à `d7ec2ea2` (13 valeurs sur 17 dans
  chaque locale) — un motif global attrapait *convalidare* (valider) en italien et manquait les formes fléchies de
  l'allemand.
- *Annoté le 2026-11-04 (validation P3, V3-1, V3-6)* : le chiffre du *Retenu* ci-dessous (« quinze ») est périmé ; il
  vaut **dix-huit** valeurs modifiées depuis la remédiation P3 (dix-sept à la P2, plus le libellé d'audit, C-v0172a-24).
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

### C-v0172a-23 — Les totaux par période changent : le verrou de période suffit, la confirmation et le manuel le disent (validation P2, V2-5)

- **Contexte** : l'achat est daté de la facture, la contre-passation du jour de l'annulation ; la paire n'est nette
  qu'au cumul. La supprimer change, en sens inverse, la charge, l'impôt préalable et la TVA de la période de l'achat et
  de celle de l'annulation, et tout solde arrêté entre les deux dates ; le rapport TVA se recalcule à la volée.
- **Retenu** : aucune garde neuve. Le **verrou de période** est le seul fait que Kesh connaisse d'une période déclarée
  (« aucune écriture ne peut plus être datée d'une période déjà déclarée », `admin-manual.tex:3054`), et
  `delete_in_tx` l'applique à chacune des deux écritures (étape 3-quater) : une période verrouillée refuse le geste. La
  confirmation d'une annulée, le manuel utilisateur et le CHANGELOG disent que deux périodes changent et qu'il faut
  verrouiller une période déclarée **avant** ; T1 asserte l'effet par période.
- **Écartées** : (a) refuser quand l'achat et la contre-passation tombent dans deux trimestres différents — refuserait
  le cas courant (facture d'un trimestre, annulée au suivant) sans savoir si l'un est déclaré, et Kesh ne le sait que par
  le verrou ; (b) refuser si la date de l'une des écritures est antérieure au dernier rapport TVA produit — Kesh ne garde
  pas trace des rapports produits ; (c) recalculer l'avertissement selon les dates réelles des deux écritures — demande
  au `GET` la date de la contre-passation, pour un texte qui ne change pas la conduite.
- **Réversible** : oui (une garde se rajoute sans migration).

### C-v0172a-24 — Le libellé d'audit de `supplier_invoice.unvalidated` devient « repassée en brouillon » (validation P3, V3-1)

- **Contexte** : le journal d'audit affiche l'action sous `audit-log-action-supplier-invoice-unvalidated` (« Facture
  fournisseur dévalidée », fr-CH `:3053` ; de, it, en de même) ; C-v0172a-20 retire « dévalider » de tous les textes
  fournisseur, et cette clé échappait à la liste `K`.
- **Retenu** : 18ᵉ clé de `K`, valeur « Facture fournisseur repassée en brouillon » dans les quatre locales ; sonde
  refaite à `d7ec2ea2` : `clés=18 verbe=14` dans chaque locale. Le libellé vaut aussi pour les entrées passées (validées
  dévalidées depuis la 0.17.0) : c'est le même geste, rebaptisé.
- **Écartées** : garder le libellé historique au motif que le nom de l'action ne change pas — l'administrateur lirait
  au journal un mot que l'écran ne montre plus, pour un geste qu'il cherche sous l'autre nom.
- **Réversible** : oui.

### C-v0172a-25 — `DELETE` d'une annulée : le refus générique reste (validation P3, V3-7)

- **Contexte** : `delete_draft_in_tx` refuse une annulée par `IllegalStateTransition`, rendu « Transition d'état
  interdite » (`kesh-api/src/errors.rs:4260-4266`) ; la décision du Project Lead veut que tout refus dise quoi faire.
- **Retenu** : pas de motif neuf. La route est réservée à l'Administrateur, **fermée aux clés d'API**
  (`docs/api-external.md:563`), et l'écran ne propose *Supprimer le brouillon* que sur un brouillon (un brouillon ne
  s'annule pas) : le refus n'est atteignable que par un appel direct de session ; le manuel et `api-external.md` disent
  déjà « d'abord brouillon ».
- **Écartées** : un motif `SUPPLIER_INVOICE_MUST_BE_REDRAFTED_FIRST` — code et clé neufs dans quatre locales pour un
  chemin qu'aucun écran ni aucune clé n'atteint ; changer le texte de `ILLEGAL_STATE_TRANSITION` — il est commun à tous
  les gestes.
- **Réversible** : oui.
