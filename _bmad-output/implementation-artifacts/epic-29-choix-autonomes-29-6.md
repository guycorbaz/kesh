# Epic 29 — registre des choix autonomes, Story 29-6

*Registre propre à la story 29-6 (#665), à verser par **union** dans `epic-29-choix-autonomes.md` par l'orchestrateur.
Fiche : `29-6-trop-percu-centimes.md`.*

---

## Story 29-6 — La réconciliation ne propose plus une facture qu'un trop-perçu de quelques centimes fait refuser ([#665])

### C-29-6-1 — Prémisses exactes ; une précision et un doc-comment faux

- **Contexte** : #665 (2026-10-14) décrit la tolérance de ± 5 centimes à la proposition et le refus du trop-perçu à
  l'acceptation.
- **Retenu** : établi au code sur `fcd2ae75`, les quatre prémisses sont exactes. Précision écrite dans la fiche : la
  tolérance est symétrique, et seul son côté « le paiement dépasse le reste » est refusé ; l'autre côté donne un
  règlement partiel accepté. Le doc-comment de `find_unpaid_invoices_for_window` (« la tolérance couvre l'écart à son
  arrondi au centime, au plus 0.005 ») est faux depuis l'origine (0,05 contre 0,005) : réécrit par l'AC2.
- **Réversible** : sans objet (constat).
- *Annotation du 2026-10-14, validation P1 (R6)* : « faux depuis l'origine » était inexact — la phrase date de
  `eecf40cb` (Story 25-4-c), la tolérance de 0,05 de `22fd6d4a` (Story 8-4) : faux **depuis son écriture**.

### C-29-6-2 — Voie 1 : la proposition écarte le trop-perçu ; voie 2 écartée

- **Contexte** : #665 offre deux voies — la proposition écarte les trop-perçus, ou l'acceptation accepte l'écart toléré
  sur le compte d'arrondi (comme `SettlesWithRounding`). Consigne : établir au code les conséquences comptables, choisir
  le périmètre minimal sûr ; une écriture de produit d'arrondi sans compte d'arrondi configuré est exclue.
- **Retenu** : voie 1. Rien de nouveau ne s'écrit ; la proposition ne charge que ce que `classify_payment` admet.
- **Écartée** : voie 2, pour quatre raisons établies au code : (a) sans compte d'arrondi, `rounding_account_for_write`
  refuse encore (`ROUNDING_ACCOUNT_NOT_CONFIGURED`), donc le défaut subsiste sauf à faire lire les réglages par la
  proposition ; (b) le compte d'arrondi reçoit aujourd'hui des écarts d'un demi-centime au plus — un trop-perçu de
  5 centimes est une dette envers le client, pas une différence d'arrondi, et le passer en produit est une décision
  comptable qui revient à Guy ; (c) `classify_payment` est partagée avec le règlement manuel : l'élargir change la
  fiche de la facture, ne pas l'élargir refait deux règles du trop-perçu (#476 les avait réunies) ; (d) elle contredit
  « le trop-perçu est refusé, jamais écrit » (code, `api-external.md`) et « pas de tolérance de montant » (manuel,
  lettrage).
- **Réversible** : oui — la voie 2 peut venir plus tard comme évolution, sur demande.

### C-29-6-3 — La borne basse : une fonction Rust réciproque de `classify_payment`, liée au SQL, avant le `LIMIT 50`

- **Contexte** : `classify_payment(P, R)` rend `Overpayment` ⇔ `R < P − 0,005` (pour `P > 0` au centime, arrondi
  `MidpointAwayFromZero`). Le dépôt interdit un `ROUND` en SQL (une seule définition de l'arrondi).
- **Retenu** : `invoice_settlements::least_due_without_overpayment(paid) = paid − HALF_CENTIME`, à côté de
  `classify_payment`, liée en paramètre du `HAVING` ; un test unitaire exhaustif (grille au dix-millième) tient
  l'équivalence entre les deux lecteurs.
- **Écartées** : (a) filtrer en Rust après la requête par `classify_payment` — des trop-perçus occuperaient des places
  du `LIMIT 50` et feraient disparaître une facture valable au-delà ; (b) `ROUND(amount_due, 2) = ?` en SQL — seconde
  définition de l'arrondi ; (c) une borne basse égale au montant exact — écarterait le reste à quatre décimales soldé
  au centime (10.01 sur 10.0050, `SettlesWithRounding`), que l'acceptation admet.
- **Réversible** : oui (une fonction, un `HAVING`).

### C-29-6-4 — Le côté du paiement partiel ne change pas

- **Contexte** : un reste supérieur au paiement de 1 à 5 centimes est accepté en règlement partiel (`Ordinary`) ; la
  facture reste ouverte pour la différence.
- **Retenu** : inchangé — ce n'est ni un refus ni une écriture fausse, donc pas le défaut de #665. La tolérance de
  5 centimes ne borne plus que ce côté (constante `AMOUNT_TOLERANCE_HUNDREDTHS`, nom conservé, doc réécrit).
- **Écartées** : retirer la tolérance (les clients qui paient le montant QR arrondi d'une facture partiellement réglée
  ne seraient plus reconnus ; hors périmètre) ; l'élargir.
- **Réversible** : oui.

### C-29-6-5 — Signature : paramètre renommé, position conservée

- **Contexte** : `find_unpaid_invoices_for_window(…, tx_amount, amount_tolerance)` a treize sites d'appel : un en production (`routes/reconciliation.rs:656`), douze en test (`reconciliation_repository.rs` dix, `invoice_write_off.rs` deux).
- **Retenu** : renommer le paramètre `underpayment_tolerance` (le nom dit ce qu'il borne désormais), sans changer sa
  position ni son type : aucun site d'appel ne change.
- **Écartée** : une structure `InvoiceAmountWindow` sur le patron d'`InvoiceDateWindow` — plus de surface pour la même
  règle, et plus de conflits de rebase avec la 29-2, qui modifie la même requête et `InvoiceDateWindow`.
- **Réversible** : oui.

### C-29-6-6 — Aucun texte d'écran, aucune clé i18n, aucune capture

- **Contexte** : le libellé de `RECONCILIATION_OVERPAYMENT` (« Le montant de la transaction dépasse le reste dû de la
  facture. ») reste juste pour un appel direct ou une course.
- **Retenu** : le frontend ne change pas ; la frontière HTTP est exercée par le test `spawn_app` d'AC6 ; l'E2E complet
  tourne au dernier commit de code.
- **Réversible** : sans objet.
- *Annotation du 2026-10-14, validation P1 (R1)* : **remplacé en partie par C-29-6-10** — l'écran change (une case,
  une clé i18n, un spec E2E et ses captures). Le libellé de `RECONCILIATION_OVERPAYMENT` reste, lui, inchangé.

### C-29-6-7 — Le contournement écrit au manuel : régler le reste dû, puis rejeter la transaction

- **Contexte** : un client qui a payé quelques centimes de trop n'est plus proposé ; le règlement manuel refuse aussi
  le trop-perçu.
- **Retenu** : le manuel dit d'enregistrer le règlement du reste dû depuis la fiche, de **rejeter** la transaction à la
  réconciliation, et de traiter l'écart (remboursement ou écriture manuelle) — même forme que le contournement de la
  29-2 (C-29-2-8).
- **Réversible** : oui (texte).
- *Annotation du 2026-10-14, validation P1 (R1)* : le geste était **impraticable** à l'écran (aucune case sur une ligne
  sans proposition) ; il le devient par C-29-6-10. Le texte du manuel nomme désormais la case de la ligne «~Aucune
  correspondance~».

### C-29-6-8 — Les angles morts #665 écrits par la 29-3 se retirent au rebase

- **Contexte** : la fiche 29-3 (C-29-3-7) prescrit d'écrire au manuel (`:2049`) et au doc-comment de `candidate_rank`
  qu'une facture que le paiement dépasse est proposée puis refusée (#665). La 29-6 se développe après elle.
- **Retenu** : la 29-6 retire ou réécrit ces phrases au rebase (AC7, § *Zones partagées*), inventaire par
  `grep -n 665`.
- **Écartée** : demander à la 29-3 de ne pas les écrire — elle se merge avant, et ses phrases sont vraies à sa date.
- **Réversible** : sans objet.

### C-29-6-9 — Story ajoutée à l'Epic 29, sortie dans la v0.13.4

- **Contexte** : décision de l'orchestrateur du 2026-10-14 ; #665 relevée pendant la fiche de la 29-3.
- **Retenu** : clé `29-6-trop-percu-centimes` sous `epic-29` ; paragraphe « 29-6 » au fichier d'epic (§ *Les stories*,
  § *Ordre*, § *Releases*) ; ordre de développement 29-2 → 29-3 → 29-5 → 29-6.
- **Réversible** : oui.

### C-29-6-10 — Le périmètre s'étend à l'écran : une ligne sans proposition se coche, pour *Rejeter* seulement

- **Contexte** : validation P1, MEDIUM R1, **vérifié au code par l'orchestrateur** — une ligne « Aucune correspondance »
  n'a pas de case (`ReconciliationProposals.svelte:318-330`) et `onReject` n'agit que sur les lignes cochées : le geste
  « régler depuis la fiche, puis rejeter la transaction », prescrit par la 29-6 (AC7) et par la 29-2 (C-29-2-8), est
  impraticable. **Décision de l'orchestrateur** (consignée comme telle) : la 29-6 étend son périmètre pour rendre ce
  rejet possible ; *Accepter* reste sans effet ou désactivé pour une telle ligne.
- **Retenu** : AC10. Établi au code d'abord : `POST /reconciliation/reject` ne lit aucune candidate (pré-vol
  `classify_batch_transaction`, `UPDATE` de `reject_batch`), ses refus vont dans `failed[]` en `200` ; le test
  `post_reject_marks_transactions_as_manually_reviewed` rejette déjà des transactions sans facture ; *Annuler le rejet*
  (28-5b) ne lit pas non plus de candidate. Donc **aucun changement d'API** — l'écran seul change.
- **Écartées** : (a) écrire au manuel que la transaction reste en attente, sans geste (correction (a) de R1) — laisse
  une ligne qu'aucun geste n'écarte, et la 29-2 avec un contournement faux ; (b) ouvrir une issue `enhancement` et ne
  prescrire le geste qu'après (correction (b) de R1) — reporte hors de la v0.13.4 un geste que deux stories de la même
  release prescrivent ; (c) un bouton *Rejeter* propre à chaque ligne sans proposition — un second chemin vers la même
  route, un libellé et un test de plus pour le même effet.
- **Réversible** : oui (un composant, une clé).
- **Annotation du 2026-10-14 (validation P2, R1/F2)** : « pour *Rejeter* seulement », au titre, est inexact — la ligne
  porte aussi *Affecter manuellement* et *Éclater*, et les lignes sans proposition comprennent les débits et le non-CHF
  sans règle (`routes/reconciliation.rs:640-652`). Lire : une ligne sans proposition se coche ; *Accepter* ne la traite
  pas. Le rejet laisse la transaction en attente (#670, C-29-6-17).

### C-29-6-11 — *Accepter* grisé tant qu'une ligne sans proposition est cochée, et la page le dit

- **Contexte** : une fois la case ouverte à toute ligne, *Accepter* doit ne rien faire de faux pour une ligne sans
  proposition. Au sol, `onAccept` lève « No candidate » (`:205-207`), affiché en erreur.
- **Retenu** : `disabled` dès que la sélection contient une ligne sans proposition, garde défensive en tête
  d'`onAccept`, et une indication sous les boutons (« ne peut qu'être rejetée : décochez-la pour accepter les autres »),
  clé `reconciliation-labels-reject-only-hint` dans les quatre catalogues. Le compteur reste `selected.size`.
- **Écartées** : (a) *Accepter* n'envoie que les lignes avec proposition et ignore les autres — le plus simple au code,
  mais **silencieux** : la ligne cochée ne bouge pas et rien ne dit pourquoi, et le compteur « Accepter (n) » mentirait ;
  (b) grisé sans indication — un bouton inerte sans raison visible ; (c) une case grisée pour *Accepter* seulement —
  une case n'a pas deux états de sélection.
- **Réversible** : oui.
- **Annotation du 2026-10-14 (validation P2, R1/F2, F1, F4)** : le texte retenu ci-dessus (« ne peut qu'être
  rejetée ») et la clé `reconciliation-labels-reject-only-hint` sont **remplacés** : « ne peut pas être acceptée »,
  clé `reconciliation-labels-unmatched-not-acceptable-hint`, `data-testid` `reconciliation-unmatched-not-acceptable-hint`
  (C-29-6-16) ; l'indication passe « sous les boutons » → dans la rangée des boutons (C-29-6-18). La garde défensive
  n'est pas inatteignable en Vitest (`fireEvent.click` atteint un bouton désactivé) : éprouvée par M12.

### C-29-6-12 — Cinq modules : la limite, pas de découpage

- **Contexte** : la règle de splitting préventif découpe une story qui touche **plus de cinq** modules. Avec l'AC10 :
  `kesh-db`, `kesh-api`, `frontend/…/reconciliation`, `kesh-i18n`, documentation.
- **Retenu** : cinq, non dépassé ; pas de découpage. La 29-6a (écran) / 29-6b (proposition) a été pesée : les deux
  moitiés sont indépendantes au code, mais le manuel et le CHANGELOG de la proposition prescrivent le geste de l'écran —
  aucune ne se livre vraie seule, et l'AC10 tient en un composant sans API.
- **Écartée** : découper — deux PR, deux boucles de revue, et un ordre de merge imposé pour un texte qui ne vaut
  qu'ensemble.
- **Réversible** : oui — si une passe de revue étend encore le périmètre, la story découpe ou écrit une dérogation.
- **Annotation du 2026-10-14 (validation P2, R8/F8)** : granularité déclarée — crate Rust et paquet npm (`kesh-db`,
  `kesh-api`, `frontend`, `kesh-i18n`, documentation à part) : cinq. Au module métier, six ou sept ; si cette
  granularité est tenue pour la bonne, la fiche écrit une dérogation sur l'argument ci-dessus.

### C-29-6-13 — L'invariant « filtre avant `LIMIT 50` » est gardé par un test, pas déclaré

- **Contexte** : validation P1, R4 — aucun test ne tuait un filtre déplacé en Rust après la requête.
- **Retenu** : un second test de dépôt (50 factures trop-perçues puis une valide, la dernière en `(date, id)`) et la
  mutation M8.
- **Écartée** : déclarer l'invariant non gardé — c'est le motif même qui a écarté l'alternative Rust (C-29-6-3) ;
  laisser sans garde la raison d'un choix, c'est laisser le choix se défaire en silence.
- **Réversible** : oui.

### C-29-6-14 — La v0.13.4 réunit 29-2, 29-3, 29-4, 29-5 et 29-6

- **Contexte** : validation P1, R7 — le fichier d'epic (§ *Releases*) omettait la 29-5, la clé `epic-29` du registre
  disait « 29-2..4 ».
- **Retenu** : composition donnée par l'orchestrateur, écrite au fichier d'epic et au commentaire de `epic-29`.
- **Réversible** : oui.

### C-29-6-15 — Un spec E2E neuf, sur le patron de `reconciliation-unreject.spec.ts`

- **Contexte** : l'AC10 change ce qu'un utilisateur peut cocher et envoyer ; seul un E2E voit l'identifiant d'une ligne
  sans proposition traverser la frontière HTTP.
- **Retenu** : `reconciliation-reject-unmatched.spec.ts`, relevé unique, rejet puis *Annuler le rejet*, captures clair et
  sombre hors dépôt.
- **Écartée** : ajouter un scénario au spec de la 28-5b — il rejette par l'API à dessein (il éprouve le détail de
  l'import, pas la liste), et le mêler brouillerait ce qu'il garde.
- **Réversible** : oui.

### C-29-6-16 — L'indication dit « ne peut pas être acceptée », et ne nomme aucun autre geste

- **Contexte** : validation P2, R1 et F2 (convergents, MEDIUM) — « ne peut qu'être rejetée » est faux à l'écran : la
  ligne porte *Affecter manuellement* et *Éclater* ; pour un débit sans règle, le texte poussait vers un rejet sans
  écriture au lieu de l'imputation.
- **Retenu** : dire la seule vérité de la sélection — fr-CH « Une transaction sans correspondance ne peut pas être
  acceptée : décochez-la pour accepter les autres. » (de, it, en au même sens) ; clé
  `reconciliation-labels-unmatched-not-acceptable-hint`, `data-testid` `reconciliation-unmatched-not-acceptable-hint`.
- **Écartées** : (a) la version longue de R1 (« …ou imputez-la par *Affecter manuellement* ou *Éclater* ») — pour un
  paiement de client, la FAQ du manuel (`:2910`) interdit précisément ces deux gestes : l'indication pousserait le cas
  même de #665 vers une imputation fausse ; les boutons sont d'ailleurs visibles sur la ligne ; (b) garder la clé
  `…-reject-only-hint` avec un texte neuf — le nom reproduirait l'erreur.
- **Réversible** : oui (une clé, quatre valeurs).

### C-29-6-17 — La transaction rejetée reste en attente : angle mort assumé, renvoyé à #670

- **Contexte** : validation P2, R2 (MEDIUM) — une transaction rejetée reste `pending`, empêche la clôture du compte
  bancaire (`bank_accounts.rs:1025`, manuel `:1690-1694`), et aucun geste ne la rapproche d'un règlement saisi depuis la
  fiche ; le texte prescrit par la P1 (« — sans quoi la transaction reste en attente ») promettait l'inverse.
  L'orchestrateur a ouvert [#670] (bug P2) ; son intégration à l'epic est une décision attendue de l'auteur.
- **Retenu** : la 29-6 **ne ferme pas** #670 et n'en dépend pas ; elle l'écrit comme angle mort assumé (statut, AC10,
  inventaire (b), exclusions) et réécrit les textes prescrits (manuel `:2029`, `:2057`, `:2910`, CHANGELOG) pour ne
  plus dire que le rejet règle la transaction : il la retire de la liste et empêche qu'elle soit passée une seconde
  fois ; rejetée, elle reste en attente et empêche la clôture du compte bancaire. Au manuel, **sans numéro d'issue**
  (renvoi `\S\ref{sec:clore-compte-bancaire}`) ; au CHANGELOG, avec le lien #670.
- **Écartées** : (a) intégrer #670 à la 29-6 — ni mécanique (geste neuf de rattachement, ou statut neuf compté par la
  clôture), ni décidé, et la règle de découpage est à sa limite (C-29-6-12) ; (b) attendre la décision sur #670 — la
  fiche resterait bloquée sur un arbitrage qui ne change pas son code ; (c) un numéro d'issue au manuel — il périme à la
  fermeture de #670 sans que rien ne le signale.
- **Réversible** : oui — si #670 est intégrée avant le tag, les phrases d'attente se réécrivent avec elle.
- *Annotation du 2026-10-14 (passe ciblée P3, P3-3)* : la décision n'est plus attendue — Guy a intégré #670 à l'Epic 29
  comme **Story 29-8**, qui réécrira « puis rejetez la transaction » en « puis rattachez ». La 29-6 garde ses textes ;
  la 29-8 les remplace.

### C-29-6-18 — L'indication se place dans la rangée des boutons

- **Contexte** : validation P2, F4 — un paragraphe qui apparaît au cochage entre les boutons et la table descend la
  table d'une ligne sous le pointeur ; le clic suivant peut cocher la ligne voisine. R9/F4 : l'ordre avec l'indication
  de la 29-3 n'était pas fixé.
- **Retenu** : dernier enfant du `div` des boutons (`flex-wrap items-center`) ; ordre fixé : rangée des boutons,
  paragraphe de priorité de la 29-3, table. À largeur étroite, un retour à la ligne peut encore décaler la table :
  accepté, vérifié sur les captures.
- **Écartées** : (a) réserver la place en permanence (élément vide rendu sans condition) — la mutation M11 et son test
  (« indication absente ») perdraient leur objet, et une hauteur fixe ne tient pas les quatre longueurs de texte ;
  (b) l'indication sous la table — loin du bouton grisé qu'elle explique.
- **Réversible** : oui.

### C-29-6-19 — Le spec `reconciliation-deleted-import` garde sa règle ; seul son commentaire change

- **Contexte** : validation P2, R4 — son commentaire (`:12-14`) dit que la règle d'affectation rend les transactions
  sélectionnables ; après l'AC10, toute ligne l'est.
- **Retenu** : réécrire le commentaire (la règle n'est plus nécessaire à la sélection ; le ciblage par `data-tx-id`
  isole les lignes du spec) ; garder la règle et le code du spec.
- **Écartée** : retirer la règle — modifie un spec qui éprouve autre chose (un lot dont une transaction a disparu),
  hors du périmètre minimal, pour un gain nul.
- **Réversible** : oui.

[#665]: https://github.com/guycorbaz/kesh/issues/665
[#670]: https://github.com/guycorbaz/kesh/issues/670
