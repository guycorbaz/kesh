# Release 0.17.2 — registre des choix autonomes

Registre de la release 0.17.2, **mis à jour par union** (`docs/agents-orchestration.md` § 1) : chaque story y tient sa
section, numérotée par sa clé ; une section d'une autre story ne se réécrit pas. Chaque choix : contexte, option retenue,
alternatives écartées, réversibilité.

Un doublon se renumérote ; numéros de ligne de `d7ec2ea2` sauf mention.

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
- *Annoté le 2026-11-04 (validation P4, V4-7)* : la valeur ci-dessous est la **fr-CH** ; les trois autres suivent le
  vocabulaire d'AC-15 — de « Lieferantenrechnung in den Entwurf zurückgesetzt » (au lieu de « … entvalidiert »), it
  « Fattura fornitore riportata in bozza » (au lieu de « Fattura fornitore annullata (convalida) », qui dit *annulée* :
  contresens), en « Supplier invoice returned to draft » (au lieu de « … unvalidated »).
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
- *Annoté le 2026-11-04 (validation P4, V4-5)* : il l'est aussi par une page restée ouverte pendant qu'une autre session
  validait puis annulait le brouillon (`+page.svelte:613`, statut lu au chargement ; `delete_draft_in_tx` relit sous
  verrou) ; course rare, message générique assumé.
- **Écartées** : un motif `SUPPLIER_INVOICE_MUST_BE_REDRAFTED_FIRST` — code et clé neufs dans quatre locales pour un
  chemin qu'aucun écran ni aucune clé n'atteint ; changer le texte de `ILLEGAL_STATE_TRANSITION` — il est commun à tous
  les gestes.
- **Réversible** : oui.

### C-v0172a-26 — `docs/api-external.md` garde « dévalidation » comme nom de la route (validation P4, V4-2)

- **Contexte** : le document de contrat des intégrateurs emploie « dévalidation » neuf fois (`:300 460 572 574 604 610
  869 961 965`, relevé à `d7ec2ea2`) ; la route s'appelle `POST …/unvalidate` pour les deux sortes de factures ; le
  vocabulaire de l'écran fournisseur devient « repasser en brouillon » (C-v0172a-20).
- **Retenu** : « dévalidation » reste le **nom de la route** (comme le nom de code) ; changent les phrases devenues
  fausses (`:572`, `:604`, partie fournisseur de `:869`) et celles qui recopient la consigne d'un message (`:574`,
  `:961`) ; restent `:300`, `:965` (génériques) et `:460`, `:610` (clientes).
- **Écartées** : renommer partout — le document décrirait une route `unvalidate` sous un autre nom que son chemin, et les
  passages génériques couvrent aussi les factures clientes, qui gardent « dévalider ».
- **Réversible** : oui.

### C-v0172a-27 — Le paiement détaché se lit par `CAST(JSON_VALUE(…) AS SIGNED)` (développement)

- **Contexte** : `JSON_VALUE` rend un `LONGBLOB` que sqlx refuse de décoder en `String` (constaté au premier vert ciblé,
  six tests rouges en `ColumnDecode`) ; `modification_guard` contourne en liant l'identifiant **en chaîne**.
- **Retenu** : la requête de la dévalidation enveloppe le fragment partagé `DETACHED_SETTLEMENT_ENTRY` dans
  `CAST(… AS SIGNED)` et lit des `Option<i64>` ; une valeur illisible vaut 0, qu'aucune écriture ne porte (ignorée).
- **Écartées** : décoder en `Vec<u8>` puis analyser — plus de code pour le même résultat ; une seconde copie du chemin
  JSON — contredit le contrôle DRY d'AC-6.
- **Réversible** : oui.

### C-v0172a-28 — Le pré-contrôle d'exercice clos ne joue que pour une annulée (développement)

- **Contexte** : la tâche 3.5 range le pré-contrôle parmi les refus des rangs 3 à 6 sans dire s'il vaut pour une
  validée ; il existe pour le mode `System` de la dissolution, que seule l'annulée emprunte (C-v0172a-4).
- **Retenu** : pré-contrôle sur la seule branche annulée ; une validée garde l'ordre d'avant (son exercice clos parle par
  `delete_in_tx`, même code) — les tests `unvalidation_precedence_*` restent verts sans réécriture.
- **Écartées** : le jouer aussi pour une validée — il imposerait un verrou d'écriture et d'exercice plus tôt sur ce
  chemin, sans changer aucune réponse.
- **Réversible** : oui.

### C-v0172a-29 — `document_id` de `DetachedPayment` = l'écriture de paiement ; l'avertissement du lot reste aux validées (développement)

- **Contexte** : AC-8 garde `details.documentId` sans dire ce qu'il désigne pour le motif neuf ; la fiche montrait
  l'avertissement du lot confirmé pour une `open` seulement.
- **Retenu** : `documentId` = `entryId` (l'écriture de paiement, l'objet à défaire) ; sur la fiche, l'avertissement du
  lot confirmé ne s'affiche toujours que pour une validée (le bloc s'ouvre aussi pour payée et annulée, sans lui) ; la
  boîte de confirmation le garde si la facture en porte un.
- **Écartées** : `documentId` = la transaction rapprochée — elle manque quand le paiement n'est pas rapproché.
- **Réversible** : oui.

### C-v0172a-30 — Les captures se prennent par un script hors dépôt, sur la base E2E jetable (développement)

- **Contexte** : les refus « période verrouillée » et « exercice clos » demandent de verrouiller ou de clore, ce que la
  suite E2E partagée ne doit pas faire (les autres specs en dépendent) ; aucun fichier ne se supprime (consignes).
- **Retenu** : `kesh-gate-logs/v0172a/captures-v0172a.mjs`, Playwright chargé par `createRequire` depuis le
  `frontend/` du worktree, contre le backend du port 3172 sur `kesh_e2e_v0172a` **après** l'E2E complet ; états posés
  par l'API, le verrou et la clôture par SQL sur la seule base jetable, rétablis après chaque capture.
- **Écartées** : une spec Playwright versionnée de plus — elle clorait un exercice dans la base partagée de la suite.
- **Réversible** : oui (hors dépôt).

---


## Story v0172-correctifs (clé `v0172b`) — #876, #840, #870, #851, #882, #880, #883, #885

Fiche : `v0172-correctifs.md`. Numéros de ligne de `d7ec2ea2`. Issues en `refs` seulement.

### C-v0172b-1 — Découpage *(révisé deux fois : validation P1, trois → quatre sous-stories ; validation P2, #876 extraite en fiche sœur)*

*Révision de la validation P2 (2026-11-04) : la sévérité stagnait (12 MEDIUM puis 11, six des onze nés de la remédiation P1, les
défauts recyclés concentrés sur #876). #876 passe dans la fiche sœur `v0172-session.md` (clé `v0172-session`, choix
`C-v0172s-k`) ; `v0172-correctifs` garde A2, B et C, dans cet ordre. Signal déclaré au Project Lead par l'orchestrateur.*

- **Contexte** : huit issues, plus de cinq modules (règle de splitting préventif du `CLAUDE.md`).
- **Retenu** (état en vigueur depuis la validation P2) : **#876** dans la fiche sœur `v0172-session.md` (clé de statut
  `v0172-session`, choix `C-v0172s-k`) ; **trois** sous-stories dans `v0172-correctifs.md` — A2 (import simultané #840), B (écrans :
  #870, #851, #882, #885), C (forme : #880, #883) —, développées dans l'ordre A2 → B → C sur la branche `story/v0172-correctifs`,
  chacune close par ses commits, son gate et sa revue de code ; chacune touche au plus cinq modules (tableau des fichiers dans la
  fiche) ; C est un déploiement mécanique. *Historique* : trois sous-stories à la rédaction (A = #876 + #840), quatre à la
  validation P1 (A1 / A2 séparées, finding P5), puis l'extraction de A1 à la validation P2.
- **Écartées** : (a) une story unique — trop large pour une revue adversariale fiable ; (b) une fiche sœur par sous-story — les
  trois qui restent partagent la contrainte, le CHANGELOG et le manuel, et leurs revues convergeaient ; (c) huit stories — coût de
  boucles de revue disproportionné pour des correctifs de P3/P4.
- **Réversible** : oui — une sous-story peut encore être sortie en fiche sœur si sa revue ne converge pas.

### C-v0172b-2 — #876 : renouvellement côté navigateur, à l'entrée de l'utilisateur, et champ `sessionExpiresIn` *(révisé à la validation P1)*

*Déplacé le 2026-11-04 (validation P2) vers la section `v0172s` : **C-v0172s-1**, qui fait foi ; texte ci-dessous conservé comme historique.*

- **Contexte** : avec 15/15 minutes (défauts), le renouvellement sur 401 arrive toujours après l'expiration du jeton de
  renouvellement (`routes/auth.rs:254, :399` ; cookie d'accès `max_age = jwt_expiry`, `:54`) : la session meurt à 15 minutes,
  en pleine saisie, avec redirection vers `/login` (`api-client.ts:176-177`).
- **Retenu** : le frontend renouvelle **à l'entrée de l'utilisateur**, au plus une fois par pas `T` (C-v0172b-23), tant que la
  session vit ; le serveur rend `sessionExpiresIn` (login, refresh, changement de mot
  de passe, assistant : `refresh_inactivity` ; `/me` : durée restante du jeton du cookie, `null` sinon).
- **Écartées** : (a) relâcher l'expiration au serveur (`expires_at = now + jwt + leeway + inactivity`) — inactivité tolérée
  floue (15 à 31 minutes), saisie longue sans requête perdue quand même ; (b) prolonger à chaque requête — les sondes de fond
  (`BackupScheduleBanner`, `api-health`) rendraient la session éternelle ; (c) ne corriger que le manuel — la perte de saisie
  est le défaut ; (d) déduire l'échéance du renouvellement côté navigateur sans aide du serveur — impossible après un
  rechargement de page, et faux si `KESH_REFRESH_INACTIVITY_MINUTES < KESH_JWT_EXPIRY_MINUTES` ; (e) une minuterie armée avant
  l'échéance qui renouvelle s'il y a eu activité depuis le dernier renouvellement — la première rédaction : borne d'inactivité
  jusqu'à `2N − 1`, et minuterie muette pendant une veille ou ralentie dans un onglet masqué (findings P1-P2 de la validation P1).
- **Réversible** : oui (champ JSON additif, module frontend isolé).

### C-v0172b-3 — #876 : l'activité, ce sont les entrées de l'utilisateur, non les requêtes

*Déplacé le 2026-11-04 (validation P2) vers la section `v0172s` : **C-v0172s-2**, qui fait foi ; texte ci-dessous conservé comme historique.*

- **Contexte** : des composants interrogent le serveur sans geste de l'utilisateur (`BackupScheduleBanner.svelte:34`,
  `api-health.svelte.ts:102`).
- **Retenu** : `keydown`, `pointerdown`, `wheel`, `touchstart` (écouteurs passifs sur `window`) et `visibilitychange` vers
  `visible` (revenir sur l'onglet est un geste) ; posés à l'armement par `authState`, retirés au désarmement (`clearSession`,
  `logout`) ; une saisie longue sans
  enregistrer garde la session ; un onglet laissé ouvert expire après l'inactivité réglée.
- **Écartées** : compter les requêtes (session éternelle sur un écran d'administration) ; compter `mousemove` (un pointeur
  qui frôle l'écran prolongerait la session d'un poste quitté) ; `scroll` (déclenché aussi par le code).
- **Réversible** : oui.

### C-v0172b-4 — #876 : un seul renouvellement pour tous les onglets

*Déplacé le 2026-11-04 (validation P2) vers la section `v0172s` : **C-v0172s-3**, qui fait foi ; texte ci-dessous conservé comme historique.*

- **Contexte** : les onglets partagent les cookies ; un jeton déjà tourné présenté de nouveau révoque toutes les sessions
  (`routes/auth.rs:379`). Le correctif rend le renouvellement fréquent et donc la course réelle, y compris sur le chemin
  401 existant.
- **Retenu** : `navigator.locks.request('kesh:auth:refresh')` autour du renouvellement (à l'entrée **et** sur 401) ; sous le
  verrou, si `localStorage['kesh:auth:renewedAt']` est postérieur ou égal à la décision de l'onglet, `GET /auth/me` au lieu de `POST
  /auth/refresh` ; après succès, écriture de `renewedAt` et message `auth-renewed` sur le canal existant. Sans
  `navigator.locks`, repli sur la seule comparaison de `renewedAt` (course résiduelle déclarée).
- **Écartées** : (a) une période de grâce au serveur pour un jeton tout juste tourné — affaiblit la détection de vol ;
  (b) un onglet « maître » élu — plus complexe, même résultat ; (c) rien — déconnexion de tous les onglets à chaque
  renouvellement simultané.
- **Réversible** : oui.

### C-v0172b-5 — #876 : la recette de session est manuelle, non un E2E

*Déplacé le 2026-11-04 (validation P2) vers la section `v0172s` : **C-v0172s-4**, qui fait foi ; texte ci-dessous conservé comme historique.*

- **Contexte** : la suite E2E partage un backend aux durées par défaut (15 minutes) ; un E2E de session demanderait un
  backend dédié à durées courtes et plusieurs minutes d'attente.
- **Retenu** : Vitest à minuteries simulées pour la logique (AC-A3, A4), tests d'intégration pour le serveur (AC-A1, A2), et
  une recette manuelle à 2/2 minutes sur la base jetable `kesh_v0172b`, déroulé au Dev Agent Record (AC-A5).
- **Écartées** : un projet Playwright dédié à durées courtes — coût récurrent pour un comportement couvert en unitaire.
- **Réversible** : oui.

### C-v0172b-6 — #840 : le verrou nommé du compte, sans migration

- **Contexte** : `create` et `create_csv` (`routes/bank_imports.rs:1157-`, `:2060-`) contrôlent puis insèrent sans
  exclusivité ; l'unicité `(company_id, file_hash)` a été retirée (`20260507000001`) parce que le réimport confirmé est
  légitime.
- **Retenu** : le motif de la saisie manuelle (`bank_statements_manual.rs:211-283`) : `with_account_lock` sur une connexion
  dédiée, la section transactionnelle existante déplacée **telle quelle** sous le verrou, verrou rendu après le commit,
  `MANUAL_STATEMENT_LOCK_TIMEOUT_SECS` réutilisée, `409 RECONCILIATION_ACCOUNT_LOCKED` au délai. L'ordre des refus est
  inchangé.
- **Écartées** : (a) une contrainte d'unicité — migration, et elle interdirait le réimport confirmé ; (b) `SELECT … FOR
  UPDATE` sur la ligne du compte — sérialiserait aussi, mais pas avec la saisie manuelle ni le rapprochement, qui tiennent le
  verrou nommé : deux mécanismes pour une même exclusion ; (c) un verrou en mémoire du processus — faux dès deux instances ou
  un redémarrage.
- **Réversible** : oui (aucun schéma).

### C-v0172b-7 — #870 : pagination et filtre de statut dans l'adresse, sans toucher au serveur

*Révisé à la validation P1 : le motif `initFromUrl` n'est pas repris tel quel, cf. C-v0172b-26 (lecture réactive de l'URL).*

- **Contexte** : le serveur pagine et filtre déjà (`routes/supplier_invoices.rs:640-`, `services/factures_fournisseur.rs:28-30,
  :65, :116`) ; la page ne lit pas l'URL et demande 100 lignes.
- **Retenu** : motif de `invoices/+page.svelte` (`initFromUrl`, `syncUrl`, *Précédent*/*Suivant*) ; `?status=` d'**une** valeur,
  `?offset=`, `?limit=` (50 par défaut, borné à [1, 200]) ; avec un statut, bandeau du filtre et case des annulées masquée ;
  lien K8 vers `?status=draft`.
- **Écartées** : (a) un sélecteur de statut complet qui remplacerait la case des annulées — change l'écran et le manuel de
  la 0.17.0 au-delà du besoin ; (b) un statut multiple dans l'URL — aucun lien ne le demande ; (c) relever la limite à 200 —
  repousse le défaut.
- **Réversible** : oui.

### C-v0172b-8 — #851 : la cause est le débordement de la barre latérale ; reproduire avant de corriger

*Complété à la validation P3 : l'aspect de la barre est gardé — fond et filet portés par un conteneur étiré, la `<nav>` seule `sticky` à hauteur maximale ; AC-B5 ter et sa capture ; état persisté des groupes vidé avant l'oracle, relevé (1) non bloquant (findings P2, P9).*

*Révisé à la validation P2 : la reproduction est un **oracle du symptôme** (entrées atteignables par la molette, avant et après
l'ouverture d'un groupe, sans et avec bandeau), dont le résultat décide du correctif ; le correctif borne la barre par un
**maximum** (`self-start max-h-[…]`) en plus d'`overflow-y-auto`, pour rester juste avec un bandeau. L'oracle de la première
remédiation rougissait par construction (finding P9) et restait rouge sous un code corrigé avec un bandeau (finding P10).*

- **Contexte** : l'issue suppose un rôle momentanément nul ; le code le réfute (remplacement atomique de `_currentUser`,
  garde de `/admin/mcp` qui redirigerait, disparition qui serait totale). La `<nav>` a une hauteur fixe sans `overflow-y`
  (`+layout.svelte:374-376`), pour 33 entrées de 44 px en mode guidé (11 + 4 + 18 ; « 34 » à la rédaction, recompté à la
  validation P1).
- **Retenu** (état en vigueur depuis la validation P4) : un **oracle du symptôme** (entrées atteignables par la molette avant et après
  l'ouverture d'un groupe, sans et avec bandeau, état persisté des groupes vidé) dont le résultat avant correctif décide ; s'il rougit,
  le fond et le filet passent à un `aside` étiré **et minoré** (`min-h-[calc(100vh-3.5rem)]`), la `<nav>` intérieure reste `sticky
  top-14` avec `max-h-[calc(100vh-3.5rem)]` et `overflow-y-auto` ; l'aspect d'aujourd'hui est gardé (AC-B5 ter). S'il ne rougit pas :
  aucune correction, constat au rapport. *(Le `self-start max-h-[…]` de la note de la validation P2, ci-dessus, est remplacé ; le
  `overflow-y-auto min-h-0` de la première rédaction aussi.)*
- **Écartées** : corriger à l'aveugle ; réduire la hauteur des liens (change le mode guidé).
- **Réversible** : oui.

### C-v0172b-9 — #882 : le taux désactivé reste affiché et le refus dit quoi faire ; le serveur ne change pas

- **Contexte** : options du `<select>` limitées aux taux actifs (`InvoiceForm.svelte:54-74`) ; le serveur refuse aussi tout
  taux inactif (`routes/vat.rs`, `verify_vat_rates_against_db`, `active = TRUE`).
- **Retenu** : option `7.70% (désactivé)` pour le taux porté par la ligne, ensemble dérivé des lignes ; message de refus
  explicite — « choisissez un taux actif », **sans** parler de réactivation, que l'écran ne propose pas (`settings/vat-rates/+page.svelte:231-252` ; la route, elle, le permet : C-v0172b-29) ;
  le repli `8.10` limité aux lignes neuves (C-v0172b-27) ; symétrique de `extraRates` du formulaire fournisseur.
- **Écartées** : accepter un taux inactif au serveur — règle TVA, jalon « Vague 2 » reporté, conséquence sur le décompte non
  établie ; remplacer d'office le taux par le premier actif — changement muet d'un montant de TVA.
- **Réversible** : oui.

### C-v0172b-10 — #880 : `formatSwissDate` tolère l'horodatage ; un helper voisin `formatSwissDateTime`

*Révisé à la validation P2 : les horodatages du serveur sont de l'**UTC** (`naive_utc()`, session `+00:00`, `DateTime<Utc>` avec
`Z`) ; les deux helpers lisent un horodatage sans décalage comme UTC et l'affichent **à l'heure de Zurich**
(`Intl.DateTimeFormat`, `timeZone: 'Europe/Zurich'`) ; une date pure reste sans fuseau. Tests sous `process.env.TZ` épinglé,
cas `Z` et décalage compris. « Aucun fuseau appliqué » (première rédaction) est remplacé ; le « Retenu » ci-dessous dit l'état en vigueur.*

- **Contexte** : `formatSwissDate` (`reports.api.ts`) rend un résultat faux sur `2026-11-03T17:40:45` (#883.6) ; plusieurs
  sites de #880 affichent un horodatage (`importedAt`, `lastUsedAt`), et les horodatages du serveur sont de l'UTC.
- **Retenu** (état en vigueur depuis la validation P2, complété à la P3) : une date pure (`AAAA-MM-JJ`) se rend `JJ.MM.AAAA` sans
  fuseau ; un horodatage (sans décalage lu comme UTC, `Z` et `±HH:MM` compris) se rend à l'heure de Zurich, par
  `Intl.DateTimeFormat('fr-CH', { timeZone: 'Europe/Zurich' })` dont seuls les composants sont repris pour composer `JJ.MM.AAAA`
  et `JJ.MM.AAAA HH:mm` (la forme ne dépend pas de la langue) ; `api-keys` délègue. **Exception** : l'expiration d'une clé API est
  une **date de calendrier** saisie, envoyée `T23:59:59Z` (`api-keys/+page.svelte:152`) : elle s'affiche par sa date
  (`value.slice(0, 10)`), sans conversion (finding P1 de la validation P3 ; test du 31.12). **`isExpired`** lit l'expiration comme UTC,
  comme le serveur (`repositories/api_keys.rs:134`) — l'écran dit « expirée » au même instant que le serveur refuse la clé ; AC-C2,
  MC6 ; puce au CHANGELOG, sans issue séparée (corrigé dans cette story ; finding P2 de la validation P5).
- **Écartées** : afficher les horodatages tels quels (heure UTC lue comme suisse, et la veille entre minuit et 1-2 h) ; `Intl` pour
  la forme entière (dépendante de la langue, alors que la 0.17.1 a fixé `JJ.MM.AAAA`) ; un `.slice(0, 10)` à chaque site
  (duplication, heure perdue) ; convertir aussi l'expiration (31.12 affiché 01.01).
- **Réversible** : oui.

### C-v0172b-11 — #880 : l'`aria-label` des propositions de rapprochement est inclus

- **Contexte** : `ReconciliationProposals.svelte:461, :471` passent `bookingDate` brute au message lu par un lecteur d'écran.
- **Retenu** : même forme suisse que la cellule voisine (`:480`) — ce qui est lu doit dire ce qui est affiché.
- **Écartées** : n'y toucher que pour l'affichage visible.
- **Réversible** : oui.

### C-v0172b-12 — #880 : `UnrejectTransactionAction` reste en `Intl`

- **Contexte** : `UnrejectTransactionAction.svelte:39` formate par `Intl.DateTimeFormat(i18nLocale(), { dateStyle: 'medium' })`.
- **Retenu** : conforme — ce n'est pas l'ISO que vise #880 ; l'uniformiser est un choix de présentation hors de l'issue.
- **Écartées** : le passer à `formatSwissDate` — change un texte localisé voulu (C-28-5-15).
- **Réversible** : oui.

### C-v0172b-13 — #883.2 : sans objet

- **Contexte** : « des lignes Svelte dépassent probablement la largeur Prettier ». Le dépôt n'a ni Prettier ni
  configuration de largeur (`frontend/package.json`, `git ls-files`).
- **Retenu** : rien ne change ; l'orchestrateur le dit à l'issue.
- **Écartées** : refluer à la main sans règle — bruit de diff sans critère ; introduire Prettier — hors périmètre d'une
  corrective (reformaterait tout le frontend).
- **Réversible** : oui.

### C-v0172b-14 — #883.4 : un payload précédé d'un BOM reste refusé, et le test le fige

- **Contexte** : `est_payload_spc` (`qr_decode.rs:39-41`) teste `starts_with` ; un BOM en tête le fait échouer.
- **Retenu** : figer le comportement actuel (refus) : la norme SIX n'admet pas de BOM, et aucune pièce connue n'en porte.
- **Écartées** : retirer un BOM avant le test — élargit sans cas réel.
- **Réversible** : oui.

### C-v0172b-15 — #885 : la date « à vérifier » ne bloque pas l'enregistrement

- **Contexte** : #853 venait d'une date du jour restée **sans le dire** ; #885 la veut proposée et visible.
- **Retenu** : avis visible jusqu'à modification ou confirmation (*C'est la bonne date*), sans blocage ; date du jour
  **locale** (`todayLocal()`), non UTC.
- **Écartées** : bloquer tant que non confirmée — un clic obligatoire de plus, que l'issue n'a pas demandé ; garder le champ
  vide — refusé par le Project Lead.
- **Réversible** : oui (une condition à ajouter dans `onSubmit`).

### C-v0172b-16 — #885 : l'échéance suit la date tant qu'elle n'a pas été saisie, à la création seulement

- **Contexte** : le formulaire fournisseur n'emploie pas les conditions de paiement (prémisse de l'issue fausse) ;
  `addDaysIso` et `defaultPaymentTermsDays` existent (#245, facture cliente).
- **Retenu** : à la création (avec ou sans document), échéance = date + délai du fournisseur choisi, ou la date ; recalculée
  à chaque changement de date ou de fournisseur tant que l'utilisateur ne l'a pas saisie ; à l'édition d'un brouillon,
  jamais recalculée.
- **Écartées** : calculer au serveur — l'utilisateur ne la verrait qu'après enregistrement ; recalculer à l'édition —
  écraserait une échéance enregistrée.
- **Réversible** : oui.

### C-v0172b-17 — CHANGELOG commun à la 0.17.2

- **Contexte** : la story `v0172-annulees` (#868) écrit aussi la section `## [0.17.2] — Non publié`.
- **Retenu** : puces de cette story sous *Corrigé* / *Ajouté* ; fusion par union à l'intégration ; l'intro est vérifiée contre
  l'autre story (une migration de sa part changerait « retour sans restauration »).
- **Écartées** : une section par story.
- **Réversible** : oui.

### C-v0172b-18 — #876 : `/me` lit la ligne du jeton de renouvellement, sans écriture

*Déplacé le 2026-11-04 (validation P2) vers la section `v0172s` : **C-v0172s-5**, qui fait foi ; texte ci-dessous conservé comme historique.*

- **Contexte** : après un rechargement, seul `/me` peut dire l'échéance du jeton de renouvellement.
- **Retenu** : `find_active_by_token` sur le cookie `kesh_refresh_token` (chemin `/api/v1/auth`, envoyé à `/me`) ; `null`
  sans cookie, inconnu, révoqué ou expiré ; aucune mise à jour de la ligne.
- **Écartées** : prolonger la ligne à chaque `/me` — écriture par lecture, et `/me` est appelé au démarrage de chaque onglet.
- **Réversible** : oui.

### C-v0172b-19 — Issue demandée : la sélection d'un lot de paiement est limitée à 200 factures validées

- **Contexte** : `routes/(app)/payment-batches/+page.svelte:54` (`listSupplierInvoices({ limit: 200, status: ['open'] })`),
  même famille que #870, non mécanique (sélecteur paginé ou recherche).
- **Retenu** : hors périmètre ; l'orchestrateur ouvre l'issue (« on trace toujours »).
- **Réversible** : —

### C-v0172b-20 — Issue demandée : la « date du jour » calculée en UTC

- **Contexte** : `….toISOString().slice(0, 10)` rend la veille entre minuit et 1 h (2 h en été) en Suisse. Le motif large
  `git grep -nE '\.toISOString\(\)\.slice\(0, ?10\)' -- frontend/src ':!*.test.ts'` rend **16 lignes** (relevé à la validation
  P1 ; la première rédaction n'en comptait que dix, par un motif qui manquait les formes en deux temps) : `InvoiceForm.svelte:110`,
  `SettleInvoiceDialog.svelte:77`, `WriteOffDialog.svelte:64`, `invoice-helpers.ts:124`, `journal-entries/form-helpers.ts:63`,
  `ManualReminderDialog.svelte:46`, `reminders/reminder-sent-at.ts:24`, `reports.api.ts:328`, `SupplierInvoiceForm.svelte:153`
  (corrigé par #885), `invoices/[id]/+page.svelte:106`, `invoices/due-dates/+page.svelte:297`, `payment-batches/+page.svelte:37`,
  `payment-batches/[id]/+page.svelte:26`, `settings/+page.svelte:219`, `settings/api-keys/+page.svelte:51`,
  `supplier-invoices/[id]/+page.svelte:90`. Certaines sont du calcul de dates (à trier), non une « date du jour ».
- **Retenu** : seul le site de #885 passe à `todayLocal()` ; l'issue demandée à l'orchestrateur porte les 15 autres lignes, à trier. *Complété à la validation P3 : l'issue est **#889** ; l'orchestrateur y a ajouté la forme `formatSwissDate(x.slice(0, 10))` sur un horodatage UTC, dont la fiche (§ C.1) liste les **12** lignes à y porter nommément.*
- **Écartées** : les corriger tous ici — des écrans sans lien avec les issues de la story.
- **Réversible** : —

### C-v0172b-21 — Aucune migration ; retour à la 0.17.1 sans restauration

- **Contexte** : contrainte de production (`v017-donnees-a-garder.md`) ; consigne de préférer une solution sans migration.
- **Retenu** : verrou nommé (#840), champ JSON additif (#876), tout le reste au frontend ; 88 migrations avant et après (`ls crates/kesh-db/migrations/*.sql | wc -l` ; « 86 » à la
  rédaction, faux, corrigé à la validation P1).
- **Écartées** : contrainte d'unicité pour #840 (C-v0172b-6).
- **Réversible** : —

### C-v0172b-22 — Les textes neufs passent par l'i18n, quatre locales

- **Contexte** : `InvoiceForm.validate` écrit ses messages en français en dur (`:480-497`) ; les écrans fournisseur et la
  liste emploient `i18nMsg`.
- **Retenu** : chaque texte neuf (bandeau du filtre, pagination fournisseur, libellé « (désactivé) » et son refus, avis « à
  vérifier » et son bouton, lien K8) a sa clé dans les quatre `messages.ftl` ; les messages voisins en dur de `validate`
  restent tels quels.
- **Écartées** : passer tout `validate` à l'i18n — hors périmètre.
- **Réversible** : oui.

### C-v0172b-23 — #876 : pas de minuterie ; l'entrée renouvelle, au plus une fois par pas `T`

*Déplacé le 2026-11-04 (validation P2) vers la section `v0172s` : **C-v0172s-6**, qui fait foi ; texte ci-dessous conservé comme historique.*

- **Contexte** : validation P1, findings P1 et P2 — une minuterie qui renouvelle « s'il y a eu activité depuis le dernier
  renouvellement » laisse la session vivre jusqu'à `2N − 1` après la dernière entrée, et ne sonne pas pendant une veille ni à
  l'heure dans un onglet masqué.
- **Retenu** : à chaque entrée (clavier, pointeur, molette, toucher, retour sur l'onglet), l'horloge murale est relue ; si la
  session vit et que le dernier renouvellement (de cet onglet ou d'un autre) date d'au moins `T = min(60 s, N/4, jwt/4)`, on
  renouvelle. Borne d'inactivité : entre `max(N, jwt) − T` et `max(N, jwt)` après la dernière entrée (14 à 15 minutes par
  défaut). Coût : au plus un renouvellement par minute d'activité et par utilisateur.
- **Écartées** : (a) la minuterie de la première rédaction ; (b) une minuterie de secours en plus de l'entrée — inutile : sans
  entrée, il n'y a rien à prolonger ; (c) un pas fixe de 60 s — trop long pour une inactivité réglée à une minute.
- **Réversible** : oui.

### C-v0172b-24 — #876 : un échec transitoire du renouvellement anticipé ne déconnecte pas

*Déplacé le 2026-11-04 (validation P2) vers la section `v0172s` : **C-v0172s-7**, qui fait foi ; texte ci-dessous conservé comme historique.*

- **Contexte** : `doRefresh` efface la session sur toute erreur (`api-client.ts:96-99, :174-178`), défendable sur 401 (session
  morte), faux quand la session vit encore (finding P3).
- **Retenu** : sur le chemin anticipé, seul `401 INVALID_REFRESH_TOKEN` efface la session ; erreur réseau, 429, 5xx : la
  session reste, l'entrée suivante retente au plus tôt 10 s plus tard tant que la session vit. Le chemin 401 garde son
  comportement, plus la garde « 401 sans entrée depuis le dernier renouvellement : ni renouvellement ni effacement » (sans
  elle, une sonde de fond prolongerait la session dès que l'inactivité réglée dépasse la durée du jeton d'accès).
- **Écartées** : réessayer en boucle sans entrée — prolongerait une session inactive ; effacer sur toute erreur — la perte de
  saisie de #876 par un autre chemin.
- **Réversible** : oui.

### C-v0172b-25 — Limites déclarées, et issue demandée : le jeton de renouvellement stocké en clair

*À la validation P2, le point (c) (manuel d'administration et stockage du jeton) passe à `v0172-session` : **C-v0172s-14**. Restent ici (a) et (b), d'A2.*

- **Contexte** : (a) l'import tient deux connexions d'un pool de 5 (`main.rs:83`) ; (b) au-delà de 5 s sous le verrou, le
  second envoi rend le `409` du rapprochement ; (c) en refermant l'inventaire du manuel d'administration, `admin-manual.tex:2996`
  affirme un hash SHA-256 du jeton de renouvellement, alors qu'il est stocké tel quel (`repositories/refresh_tokens.rs:30-34`,
  `.bind(&new.token)`).
- **Retenu** : (a) et (b) déclarés en Dev Notes, non corrigés (même risque que la saisie manuelle, déjà accepté) ; (c) le manuel
  est corrigé (il dit ce que fait le code) ; le stockage en clair, défaut de sécurité sans lien avec #876, fait l'objet d'une
  **issue demandée** (« on trace toujours »).
- **Écartées** : hacher le jeton dans cette story — touche le dépôt et les sessions ouvertes, hors périmètre d'une corrective.
- **Réversible** : —

### C-v0172b-26 — #870 : l'URL est la source de vérité, lue de façon réactive ; le filtre ne porte que sur le statut

*Complété à la validation P2 (finding P8) : la case des annulées a sa place dans l'URL, `?cancelled=1` ; les assertions `limit: 100` du test de la liste sont nommées et le simulacre de `goto` modifie `page.url`.* *Précisé à la validation P3 : seules `:59`, `:68`, `:71` sont des assertions de requête (`:50, :104, :113, :136` sont des réponses simulées) ; `FiscalYearChecks.test.ts:176-178` change aussi ; le `page` simulé est **réactif**, sur le modèle de `journal-entries/[id]/page-state.test-helper.svelte.ts`, `url` réaffectée par le simulacre de `goto` (findings P3, P5).*

- **Contexte** : validation P1, finding P6 — le motif `initFromUrl` à `onMount` ne relit pas l'URL quand seule la requête
  change ; un lien vers `/supplier-invoices` depuis `?status=draft` laisserait la liste filtrée. Finding P20 : K8 compte les
  brouillons de l'exercice, le lien ouvre ceux de tous les exercices.
- **Retenu** : `status`, `offset`, `limit` dérivés de `page.url` ; les gestes naviguent ; une page au-delà du total ramène à la
  dernière page ; le bandeau, le manuel et le CHANGELOG disent « de tous les exercices ».
- **Écartées** : filtrer aussi par dates depuis K8 — l'issue ne le demande pas, et un sur-ensemble suffit à atteindre chaque
  brouillon ; faire du lien du bandeau un bouton qui remet l'état — le lien du menu resterait sans effet.
- **Réversible** : oui.

### C-v0172b-27 — #882 : le repli `8.10` ne touche plus que les lignes neuves

- **Contexte** : `InvoiceForm.svelte:83-92` réécrit en silence toute ligne à `8.10` quand ce taux n'est plus actif, lignes
  relues comprises (finding P9) — exactement ce que C-v0172b-9 écarte.
- **Retenu** : une marque d'interface `_vatPlaceholder` sur les seules lignes créées avec la valeur provisoire avant le
  chargement des taux (retirée par `stripUiKey`) ; le repli ne touche qu'elles ; une ligne relue ou venue d'un article garde
  son taux, affiché « (désactivé) » s'il est inactif.
- **Écartées** : retirer le repli — une ligne neuve créée avant le chargement garderait `8.10` même absent de la société ;
  le garder tel quel — changement muet de TVA.
- **Réversible** : oui.

### C-v0172b-28 — #840 : la mutation « verrou rendu avant le commit » est retirée ; angle mort déclaré

- **Contexte** : validation P2, finding P6 — dans le test d'entrelacement, c'est la connexion de test qui tient le verrou ; que le
  `POST` rende le sien avant ou après son commit n'est jamais observé ; `tokio::join!` ne la tue que selon l'ordonnancement.
- **Retenu** : MA13 retirée ; l'ordre « commit, puis rendu » est garanti par la forme du code (commit dans `…_under_lock`, appelée
  dans la fermeture de `with_account_lock`, comme `bank_statements_manual.rs:239-249`) et vérifié à la revue de code ; angle mort
  déclaré au Dev Agent Record.
- **Écartées** : un point d'arrêt de test dans le code de production pour inverser les rôles — instrumente la production pour un
  test.
- **Réversible** : oui.


### C-v0172b-29 — #882 : pas de bouton de réactivation ; le message reste « Choisissez un taux actif. »

- **Contexte** : validation P3, finding P4 — `PUT /api/v1/vat-rates/{id}` accepte `active: true` (`routes/vat.rs:371, :406, :425` ;
  `lib.rs:282-283`), mais l'écran *Taux de TVA* ne l'offre pas (`vat-rates/+page.svelte:240`, `{#if r.active}`).
- **Retenu** : la story n'ajoute pas de bouton ; le refus dit « Choisissez un taux actif. » (une action offerte à l'écran) ; le manuel
  dit que l'écran ne propose pas de réactiver un taux, sans affirmer qu'un taux ne se réactive pas.
- **Écartées** : un bouton *Réactiver* — touche l'écran des taux et la règle de chevauchement (`:406`), dans le domaine TVA dont les
  P1 sont reportés ; il ne corrige pas #882, qui porte sur la facture ; à proposer en demande séparée si le Project Lead le
  souhaite.
- **Réversible** : oui.

---

### C-v0172b-30 — #840 : l'appariement des issues du verrou est partagé par les trois chemins d'écriture d'un relevé

*Développement (Opus 5.5, 2026-11-04).*

- **Contexte** : la fiche prescrit de reprendre le motif de `bank_statements_manual::create` (verrou sur une connexion dédiée,
  création dans sa propre transaction, puis un `match` de quatre bras qui apparie les issues aux statuts). Le recopier deux fois
  dans `bank_imports.rs` aurait fait trois copies du même `match`.
- **Retenu** : `bank_imports::create_under_account_lock(pool, société, compte, contexte, création)` porte le verrou et le
  `match` ; `create` (CAMT), `create_csv` et `bank_statements_manual::create` l'appellent. Comportement de la saisie manuelle
  inchangé (ses tests E2 restent verts sans changement d'assertion) ; seul le message de trace porte désormais un champ
  `context`. `take_account_lock` passe dans `tests/common/mod.rs`.
- **Écartées** : recopier le `match` dans chaque chemin (DRY) ; toucher `kesh_reconciliation::mutex` (hors périmètre).
- **Réversible** : oui.

### C-v0172b-31 — #880 : un 25ᵉ site, hors de l'inventaire clos, est corrigé

- **Contexte** : les tests de `ReconciliationProposals` que la fiche nomme au § 26 (`:398, :428, :542`, « figent l'ISO »)
  portent sur le bilan d'un lot (`describeTx`, `ReconciliationProposals.svelte:120-125`), qui rend `t.bookingDate` brute dans
  une fonction, hors du gabarit : les motifs M₁/M₂ ne lisent que les interpolations et ne le voient pas. Une sonde élargie
  (`\.(\w*Date|\w*At|periodFrom|periodTo|settledOn|\w*On)\b` sur les 32 fichiers, appels de mise en forme et liaisons
  exclus) ne rend aucun autre site d'affichage.
- **Retenu** : `formatSwissDate(t.bookingDate)` ; les trois assertions mises en forme suisse, plus une négative. AC-C3 reste
  à 20 lignes (le site n'y entrait pas). Limite déclarée : l'inventaire de la fiche n'est clos que pour les interpolations.
- **Écartées** : le laisser (même écran, même classe de défaut, tests déjà désignés par la fiche).
- **Réversible** : oui.

### C-v0172b-32 — #851 : AC-B5 ter replie le groupe Quotidien d'un clic

- **Contexte** : AC-B5 ter mesure « groupes repliés, sur `/` » ; or `/` est une entrée de Quotidien, que le `$effect`
  d'auto-dépliage (`+layout.svelte`, route active) rouvre à chaque chargement : l'état est inatteignable par `localStorage`.
- **Retenu** : `localStorage` à `false` pour les trois groupes, rechargement, puis un clic sur le titre de Quotidien (geste
  de l'utilisateur) ; la mesure suit.
- **Écartées** : mesurer sur une autre page (la fiche dit `/`).
- **Réversible** : oui.

### C-v0172b-33 — #880 : un utilitaire de test pour épingler le fuseau

- **Contexte** : sept fichiers de test lisent un horodatage et doivent épingler `process.env.TZ` (AC-C2).
- **Retenu** : `src/lib/shared/utils/pin-time-zone.test-helper.ts` (`pinTimeZone(zone)` : `beforeEach`/`afterEach`), sur le
  précédent de `bank-accounts-page.test.ts` ; non inclus par Vitest comme fichier de test (suffixe `.test-helper.ts`).
  Les tests qui fixent aussi l'horloge (clés d'API, formulaire fournisseur) gardent leur montage local.
- **Réversible** : oui.

### C-v0172b-34 — #870 : l'E2E compte les brouillons relativement à l'état de la base

- **Contexte** : la spec `supplier-invoices.spec.ts` est seedée une fois (`beforeAll`) ; un ordre d'exécution différent ou une
  base non remise à neuf pourrait y laisser des brouillons.
- **Retenu** : le test lit le nombre de brouillons avant d'en créer 55, attend `avant + 55` au pied et `avant + 5` en page 2,
  et refuse un montage au-delà de 100.
- **Réversible** : oui.

### C-v0172b-35 — #870 : « Afficher toutes les factures » mène à la liste annulées comprises

*Remédiation de la revue de code P1 (finding E-5), 2026-11-04.*

- **Contexte** : le lien du bandeau de filtre menait à `/supplier-invoices`, liste par défaut **sans** les annulées ; depuis
  `?status=cancelled`, il faisait disparaître les factures qu'on regardait, et le manuel disait « la liste complète ».
- **Retenu** : le lien mène à `/supplier-invoices?cancelled=1` (la case cochée) ; le libellé et le manuel disent vrai. La fiche
  (C-v0172b-7) prescrivait `/supplier-invoices` ; l'écart est écrit ici.
- **Écartées** : renommer le lien « Retirer le filtre » (le libellé, traduit, reste juste ainsi) ; ne corriger que le manuel.
- **Réversible** : oui.

### C-v0172b-36 — #851 : la mutation MB3 bis est équivalente

- **Contexte** : MB3 bis rétablit `h-[calc(100vh-3.5rem)]` à la place de `max-h-[…]` sur la `<nav>` ; la fiche l'attendait
  tuée par la variante « avec bandeau » d'AC-B5. Jouée à la remédiation de la revue P1 : **6 passés**, elle survit.
- **Raison de l'équivalence** : depuis le correctif, le fond et le filet sont portés par l'`aside`, et la `<nav>` défile
  (`overflow-y-auto`) ; qu'elle ait une hauteur fixe ou maximale, ses entrées restent atteignables par la molette (la barre
  défile jusqu'en bas, puis la page de `B` pixels) et l'aspect est celui de l'`aside`. La seule différence est la hauteur de la
  boîte de la `<nav>` quand son contenu est court, invisible à l'écran. Un test qui la mesurerait figerait une implémentation,
  non un comportement : aucun n'est ajouté.
- **Réversible** : —

## Story v0172-session (clé `v0172s`) — #876

Fiche : `v0172-session.md`, extraite de `v0172-correctifs.md` à la validation P2 (2026-11-04). Numéros de ligne de `d7ec2ea2`.
Correspondance : C-v0172s-1 ← C-v0172b-2 ; s-2 ← b-3 ; s-3 ← b-4 ; s-4 ← b-5 ; s-5 ← b-18 ; s-6 ← b-23 ; s-7 ← b-24 ; s-8 à s-13
neufs ; s-14 ← b-25 (c).

**Conception remplacée à la validation P3 (2026-11-04)** : **C-v0172s-1 à 13 sont écartés** (historique ; leur conception — renouvellement
anticipé à l'entrée, verrous, module armé, suspension — est remplacée, cf. C-v0172s-15) ; **C-v0172s-14 reste** (issue du jeton stocké en
clair) ; **en vigueur : C-v0172s-14 à 21**.

### C-v0172s-1 — Renouvellement côté navigateur, à l'entrée de l'utilisateur, et champ `sessionExpiresIn`

- **Contexte** : avec 15/15, le renouvellement sur 401 arrive toujours après l'expiration du jeton de renouvellement
  (`routes/auth.rs:254, :399` ; cookie d'accès `max_age = jwt_expiry`, `:54`) ; la session meurt à 15 minutes, saisie perdue
  (`api-client.ts:176-177`).
- **Retenu** : renouvellement **à l'entrée**, au plus une fois par pas `T`, tant que le jeton de renouvellement vit ; le serveur rend
  `sessionExpiresIn` (login, refresh, changement de mot de passe, assistant : `refresh_inactivity` ; `/me` : reste, `null` sinon).
- **Écartées** : relâcher l'expiration au serveur ; prolonger à chaque requête ; ne corriger que le manuel ; déduire les échéances
  sans aide du serveur ; une minuterie (borne jusqu'à `2N − 1`, muette en veille).
- **Réversible** : oui.

### C-v0172s-2 — L'activité, ce sont les entrées de l'utilisateur, non les requêtes

- **Retenu** : `keydown`, `pointerdown`, `wheel`, `touchstart` sur `window` et `visibilitychange` vers `visible` ; posés à
  l'armement, retirés au désarmement. Les sondes de fond (`BackupScheduleBanner.svelte:34`, `api-health.svelte.ts:102`) ne comptent
  pas.
- **Écartées** : les requêtes ; `mousemove` ; `scroll`.
- **Réversible** : oui.

### C-v0172s-3 — Un seul renouvellement pour tous les onglets

- **Retenu** : `navigator.locks.request('kesh:auth:refresh')` autour de tout renouvellement (à l'entrée, sur 401) **et de la
  déconnexion** (C-v0172s-10) ; sous le verrou, `renewedAt ≥ décision` ⇒ `GET /auth/me` au lieu du `POST` ; message `auth-renewed`,
  ignoré par un onglet désarmé ; repli sans verrou par `renewedAt`.
- **Écartées** : grâce au serveur pour un jeton tout juste tourné ; onglet « maître ».
- **Réversible** : oui.

### C-v0172s-4 — La recette de session est manuelle ; les tests Vitest simulent l'horloge, non les minuteries

- **Retenu** : `vi.useFakeTimers({ toFake: ['Date'] })` + `vi.setSystemTime` (précédent `bank-accounts-page.test.ts:702-718`) ;
  aucun cas ne dépend d'une minuterie ; recette à 2/2 minutes sur `kesh_v0172b` (AC-7). *(L'ancien C-v0172b-5 disait « minuteries
  simulées » : corrigé, finding P21 de la validation P2.)*
- **Écartées** : un projet Playwright à durées courtes.
- **Réversible** : oui.

### C-v0172s-5 — `/me` lit la ligne du jeton de renouvellement, sans écriture

- **Retenu** : `find_active_by_token` sur le cookie `kesh_refresh_token` (chemin `/api/v1/auth`) ; `null` sans cookie valide.
- **Écartées** : prolonger la ligne à chaque `/me`.
- **Réversible** : oui.

### C-v0172s-6 — Pas de minuterie ; pas `T` calculé sur les valeurs reçues

- **Retenu** : `T = min(60 s, sessionExpiresIn/4, expiresIn/4)` à chaque armement ; après `login` ou un renouvellement, ce sont les
  durées pleines ; après `hydrate`, des restes (`/me`), donc un `T` plus court, prudent, jusqu'au renouvellement suivant.
  *(Aligne la fiche et le registre, qui divergeaient — finding P13 de la validation P2.)*
- **Écartées** : exposer au `/me` les durées réglées — un champ de plus pour un gain nul.
- **Réversible** : oui.

### C-v0172s-7 — Échecs du renouvellement anticipé, et chemin 401

- **Retenu** : `401 INVALID_REFRESH_TOKEN` n'efface la session **que jeton d'accès expiré** ; sinon désarmement pour ce jeton et
  session jusqu'à `accessDeadline` ; réseau, 429, 5xx : session conservée, nouvelle tentative au plus tôt 10 s plus tard. Chemin 401
  d'une requête : module armé, renouvellement seulement si une entrée date de **moins de 10 s** ; module non armé, comportement 0.17.1.
- **Écartées** : « aucune entrée depuis le dernier renouvellement » comme critère (une entrée dans le pas `T` laissait une sonde
  prolonger la session — finding P2 de la validation P2) ; effacer sur tout 401.
- **Réversible** : oui.

### C-v0172s-8 — La borne d'inactivité quand `N < jwt` : Kesh ne coupe pas avant l'expiration du jeton d'accès

- **Contexte** : validation P2, finding P1 — avec `N = 1`, `jwt = 15`, la remédiation P1 lançait un renouvellement voué au 401
  après deux minutes de lecture, et l'effaçait : saisie perdue alors que le jeton d'accès valait encore 13 minutes.
- **Retenu** : (1) aucun renouvellement tenté après `sessionDeadline` ; (2) un `401 INVALID_REFRESH_TOKEN` n'efface la session que
  jeton d'accès expiré ; la session vit donc jusqu'à `accessDeadline`. Borne : entre `max(N, jwt) − T` et `max(N, jwt)` après la
  dernière entrée qui a renouvelé — `N` si `N ≥ jwt`, `jwt` si `N < jwt` (et, dans ce cas, une lecture plus longue que `N` fixe la
  fin à `r + jwt` même si l'utilisateur reprend). Écrite au manuel d'administration.
- **Écartées** : couper à `r + N` côté navigateur (même effet que le défaut) ; effacer au premier refus.
- **Réversible** : oui.

### C-v0172s-9 — Après un import complet, le renouvellement est suspendu

- **Contexte** : finding P4 — l'import remplace `refresh_tokens` ; le bilan reste à l'écran jusqu'à *Se reconnecter*
  (`AdminRestorePanel.svelte:170-185`, `admin-manual.tex:2322`) ; un renouvellement rendrait 401 et effacerait le bilan.
- **Retenu** : `sessionRenewal.suspend()` à l'envoi de l'import (`:156`) ; reprise si l'import est refusé.
- **Écartées** : suspendre seulement après réussite — un renouvellement pourrait partir pendant l'import.
- **Réversible** : oui.

### C-v0172s-10 — La déconnexion attend le renouvellement en vol

- **Contexte** : finding P5 — `logout()` (`auth.svelte.ts:169`) révoque la ligne du cookie courant (`routes/auth.rs:306`,
  `refresh_tokens.rs:115`) ; un renouvellement commité avant laisserait une ligne neuve active.
- **Retenu** : `logout()` désarme, prend le verrou `kesh:auth:refresh` (attend tout renouvellement en vol), puis `POST /auth/logout` ;
  la réponse d'un renouvellement arrivée après le désarmement ne réarme rien.
- **Écartées** : révoquer toutes les lignes de l'utilisateur au logout — déconnecterait ses autres appareils, changement de règle
  hors périmètre.
- **Réversible** : oui.

### C-v0172s-11 — La garde du chemin 401 ne vaut que module armé ; les tests existants ne changent pas

- **Contexte** : finding P3 — `api-client.test.ts:205, :246, :270, :327, :488, :567` simulent un 401 sans entrée.
- **Retenu** : module non armé ⇒ comportement 0.17.1 ; ces six tests restent verts **sans modification** (T4, M15). Changement déclaré
  (finding P12) : module armé, une sonde de fond qui reçoit 401 sans entrée récente ne mène plus à `/login` ; l'écran reste jusqu'à
  la première action.
- **Écartées** : réécrire les six tests.
- **Réversible** : oui.

### C-v0172s-12 — L'aperçu `<iframe>` d'un document ne compte pas comme entrée

- **Contexte** : finding P17 — `DocumentPreview.svelte:92`.
- **Retenu** : déclaré au manuel utilisateur ; aucun écouteur dans l'`<iframe>` (document d'une autre origine possible).
- **Réversible** : oui.

### C-v0172s-13 — Coûts déclarés

- **Retenu** : au plus un renouvellement par pas `T` d'activité et par utilisateur ; une ligne `refresh_tokens` chacun, purgée au
  démarrage seulement (`main.rs:230-233`, au-delà de sept jours) ; la table n'est **pas** dans l'export global
  (`exports/global.rs:215-217`) — le finding P16 de la validation P2 l'affirmait, à tort — mais elle est dans le `.keshbackup`.
- **Réversible** : —

### C-v0172s-14 — Manuel d'administration corrigé ; issue demandée : le jeton de renouvellement stocké en clair

- **Retenu** : l'inventaire clos du manuel d'administration (11 sites au tableau, 53 hors classe) est dans `v0172-session.md` ;
  `admin-manual.tex:2996` dira que le jeton est stocké tel quel (`refresh_tokens.rs:30-34`) ; le stockage en clair fait l'objet
  d'une issue demandée.
- **Écartées** : hacher le jeton dans cette story.
- **Réversible** : —

### C-v0172s-15 — Conception remplacée : la fenêtre d'inactivité plus longue que le jeton d'accès, et le chemin 401 existant

- **Contexte** : trois validations de suite (P1, P2 de la fiche mère, P3 de cette fiche) ont rendu des MEDIUM nés de la mécanique du
  renouvellement anticipé (verrous entre onglets, `navigator.locks` absent en HTTP, module inerte sous Vitest, suspension pendant l'import,
  `logout` sous verrou). Lu au code : le chemin 401 existant (`api-client.ts:452-461`, mutex `:185-192`) renouvelle et rejoue ; le
  renouvellement glisse (`routes/auth.rs:421-432`) ; il n'échoue que parce que la ligne expire en même temps que le cookie d'accès
  (`:254`, `:54`, `:399`) ; le 401 précède le handler (`middleware/auth.rs:73-`), le rejeu est sûr.
- **Retenu** : défaut d'inactivité 60 minutes, règle « au moins le double du jeton d'accès » (C-16), grâce de rotation (C-17), repli `/me`
  (C-18), renouvellement silencieux au démarrage (C-19), requêtes de fond (C-20) ; borne entre `N − jwt` et `N` après la dernière requête
  (C-21).
- **Écartées** : la conception C-v0172s-1 à 13 — borne plus serrée, mais huit modules et une mécanique qui recyclait les défauts ; un jeton
  d'accès plus court (5 min) avec la même inactivité — renouvellements trois fois plus fréquents et défauts de test au jeton d'accès
  (`config.rs:2106`) sans gain pour la saisie.
- **Réversible** : oui.

### C-v0172s-16 — Défaut 60 minutes ; une valeur trop courte est portée au double, sans refus de démarrage

- **Retenu** : défaut de `KESH_REFRESH_INACTIVITY_MINUTES` 15 → 60 (`config.rs:735-755`, `.env.example:160`, `docker-compose*.yml`) ; si
  la valeur est inférieure à `2 × KESH_JWT_EXPIRY_MINUTES`, avertissement et double appliqué.
- **Écartées** : refuser le démarrage — un `.env` recopié du gabarit 0.17.1 (15) empêcherait la mise à jour de démarrer ; la règle
  « strictement supérieur » — une marge d'une minute laisserait une saisie de quelques minutes sans requête perdre la session.
- **Réversible** : oui.

### C-v0172s-17 — Grâce de rotation de 30 secondes

- **Contexte** : deux onglets qui reçoivent 401 au même moment présentent le même jeton ; le second déclencherait la détection de vol
  (`routes/auth.rs:379-389`) et la révocation de toutes les sessions.
- **Retenu** : un jeton tourné depuis moins de 30 s rend 401 sans révocation générale ; aucun jeton émis.
- **Écartées** : verrou entre onglets (finding P3 : `navigator.locks` absent hors contexte sécurisé) ; rendre le même jeton neuf au
  second appel — demande de garder le jeton neuf en clair côté serveur.
- **Réversible** : oui.

### C-v0172s-18 — Après un refus de renouvellement, un `GET /auth/me` avant d'effacer

- **Retenu** : si `/me` rend 200, un autre onglet a renouvelé : succès, rejeu ; sinon, comportement actuel.
- **Écartées** : `localStorage` ou `BroadcastChannel` — de l'état partagé de plus, pour un cas que `/me` tranche.
- **Réversible** : oui.

### C-v0172s-19 — `hydrate()` tente un renouvellement silencieux

- **Retenu** : sur 401 de `/me` au démarrage, un `POST /auth/refresh` ; succès ⇒ second `/me` ; échec ⇒ état nul **sans redirection**
  (la page de connexion appelle aussi `hydrate()`).
- **Écartées** : passer par `doRefresh` — il redirige sur échec, ce qui bouclerait sur `/login`.
- **Réversible** : oui.

### C-v0172s-20 — Les lectures de fond ne renouvellent pas

- **Retenu** : `CallOptions.background` ; la lecture périodique du bandeau de sauvegarde (`BackupScheduleBanner.svelte:34`) le porte ;
  sur 401, rejet sans renouvellement ni effacement.
- **Écartées** : laisser la lecture renouveler — session éternelle sur un écran d'administration abandonné (une lecture toutes les
  30 minutes, inactivité de 60) ; supprimer la lecture périodique — le bandeau de sauvegarde en retard perdrait sa mise à jour.
- **Réversible** : oui.

### C-v0172s-21 — La borne écrite : entre `N − jwt` et `N` après la dernière requête ; aucune durée absolue

- **Retenu** : 45 à 60 minutes par défaut, 15 à 30 avec un `.env` 0.17.1 ; une saisie sans requête est sûre tant qu'elle dure moins de
  `N − jwt` ; aucune durée absolue pour un utilisateur actif (cookie de 30 jours reposé à chaque renouvellement), écrit au manuel
  d'administration (finding P13 de la validation P3).
- **Réversible** : —
