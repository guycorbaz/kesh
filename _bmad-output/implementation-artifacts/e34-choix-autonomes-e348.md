# Epic 34 — choix autonomes de la Story 34-8 (clé e348)

*Registre tenu en autonomie (Opus 5.5) le 2026-11-10, au développement. Worktree `/home/gcorbaz/devel/kesh-e348`, branche
`story/34-8-fournisseur-relecture-apres-commit`, base `665de151`. Les choix de la fiche sont au registre du groupe,
`e34-choix-autonomes-e343.md` (C-e343-9 à 12 et 15). Mise à jour par union, jamais par écrasement.*

## C-e348-1 — l'encadré d'issue inconnue, sous les refus du cycle

- **Contexte.** L'AC-8 veut un encadré persistant « sous les gestes », `data-testid="supplier-unknown-outcome"`. La fiche a
  ses gestes à plusieurs hauteurs : le cycle (valider, repasser en brouillon) en haut, le document au milieu, le paiement et
  l'annulation sous la table des lignes.
- **Retenu.** Un seul encadré, rendu juste après les refus du cycle (`validateError`, `deleteError`, `unvalidateError`), hors
  des blocs de statut. C'est l'endroit où la revue P1 de la v017-4b a déjà placé les refus qui doivent survivre à une
  relecture : la facture relue change souvent de statut, et un encadré logé dans le bloc du paiement disparaîtrait avec lui.
  `role="status"`, `data-outcome` pour les tests, couleurs de l'avertissement du lot confirmé (`border-warning`).
- **Écartées.** Un encadré par bloc (sept emplacements, dont la moitié disparaissent à la relecture) ; une notification
  (l'AC-8 l'exclut : elle passe).
- **Réversible.** Oui.

## C-e348-2 — joindre un document : la relecture échouée ferme le sélecteur

- **Contexte.** Validation P5, 34-8-F2 (LOW) : l'état `reread-failed` de la branche `attach` n'a pas de lieu d'affichage
  écrit, alors que le sélecteur est ouvert.
- **Retenu.** Le sélecteur se ferme et l'encadré `reread-failed` (C-e348-1) dit de recharger la page. La liste du sélecteur
  n'est plus sûre (le document a peut-être été joint), et la garder ouverte inviterait à réessayer sur elle.
- **Écartées.** Garder le sélecteur ouvert avec le texte `reread-failed` (deux messages, dont l'un dans un panneau qui ment
  peut-être sur la liste) ; les deux à la fois.
- **Réversible.** Oui.

## C-e348-3 — l'issue inconnue d'`attach` qui remonte au sélecteur est une `ApiError`

- **Contexte.** L'AC-8 : relue sans document, « l'erreur remonte au sélecteur avec le texte de l'issue inconnue ». Le
  sélecteur (`AttachDocumentPicker.svelte:88-90`) n'affiche le message que d'une `ApiError`, et « Erreur inattendue » sinon.
- **Retenu.** La fiche lève un objet `ApiError` (`{ code, status, message }`) dont le message est l'avis « vérifiez », le code
  et le statut ceux de l'erreur reçue (`UNKNOWN_OUTCOME` et `0` hors `ApiError`). Le statut n'est ni 404 ni 409 : le
  sélecteur ne relit pas sa liste, et le document reste sélectionnable.
- **Écartée.** Changer le sélecteur pour lire un texte d'une `Error` : une seconde règle d'affichage pour un seul appelant.
- **Réversible.** Oui.

## C-e348-4 — deux tests Vitest de la 30-2c passent de 500 à 403

- **Contexte.** `supplier-document-page.test.ts` affirmait qu'un 500 au rattachement ou au détachement n'est pas relu et
  s'affiche comme un refus. La story change précisément ce comportement : un 5xx est une issue inconnue (AC-8).
- **Retenu.** Les deux tests gardent leur objet (un refus qui ne dit rien des données ne relit rien, le bouton redevient
  actif) avec un **403**, refus certain. Le 5xx est couvert par `supplier-unknown-outcome-page.test.ts`.
- **Écartée.** Supprimer ces tests : leur propos (pas de relecture sur un refus « hors données ») reste juste pour un 4xx.
- **Réversible.** Oui.

## C-e348-5 — `document()` passe au support partagé `fournisseur_rapproche.rs`

- **Contexte.** T5 : les montages des deux fichiers HTTP sont à réemployer, sans copie. Les tests de réponse joignent,
  détachent et complètent un document, comme `supplier_invoice_draft_e2e.rs`, qui avait sa propre aide `document()`.
- **Retenu.** L'aide `document()` passe dans `crates/kesh-db/tests/support/fournisseur_rapproche.rs` (déjà inclus par
  `#[path]` dans les deux fichiers, `#![allow(dead_code)]`) ; le fournisseur et le compte bancaire viennent de `preparer`.
  Le montage HTTP lui-même (`spawn_app`, connexion) reste propre au fichier, comme dans les quatre-vingt-un autres binaires
  `kesh-api/tests` qui l'écrivent : chaque binaire est un crate, et aucune aide commune ne le porte aujourd'hui.
- **Écartée.** Une aide HTTP commune dans `tests/common/` pour ce seul fichier : un chantier transversal, hors de #938.
- **Réversible.** Oui.

## C-e348-6 — les tests d'égalité comparent la réponse entière

- **Contexte.** L'AC-4 nomme les champs à comparer.
- **Retenu.** `assert_eq!(réponse, GET)` sur l'objet JSON **entier**, patron de la 33-6 (`invoice_settlement_response_e2e.rs`),
  plus des assertions de montage : `settlementCancellable` et `cancellable` booléens, `sourceDocument` et
  `lastConfirmedBatch` présents, lignes non vides. L'égalité entière couvre les champs nommés et ceux qu'on oublierait.
- **Réversible.** Oui.

## C-e348-7 — le second clic refusé relit la facture (revue P2, E-1)

- **Contexte.** Réponse perdue, puis relecture en échec : la fiche reste celle d'avant le geste. Au second clic, le
  serveur refuse (statut changé) ; payer, annuler le règlement et annuler affichaient ce refus en rouge sur la fiche
  d'avant, sans relire — un geste écrit annoncé en échec, le défaut de #938.
- **Retenu.** Une aide `reloadAfterStaleRefusal(err)` : sur un refus d'écran périmé (`isStaleState`, qui gagne
  `SUPPLIER_INVOICE_NOT_PAID`, `SUPPLIER_INVOICE_CANCELLED` et `SUPPLIER_INVOICE_DRAFT`), la fiche relit ; si la relecture
  échoue, l'encadré `reread-failed` revient. Employée par payer, annuler le règlement, annuler, valider et repasser en
  brouillon (les deux derniers relisaient déjà, sans l'encadré). La suppression garde son traitement.
- **Écartée.** Garder l'encadré `reread-failed` jusqu'à une relecture réussie (option b de la revue) : un geste réussi
  ensuite le laisserait à tort, et le refus resterait affiché sur la fiche d'avant.
- **Réversible.** Oui.

## C-e348-8 — la relecture sans rejeu (revue P2, E-2)

- **Contexte.** La relecture (`GET`) se rejouait ~14 s sur une coupure (`api-client.ts`, délais 300 ms à 10 s), pendant
  lesquelles les autres gestes restaient offerts sur la fiche d'avant.
- **Retenu.** `reloadAfterRefusal` lit avec `{ retry: false }` : `apiClient.get` et `getSupplierInvoice` acceptent les
  options d'appel (`CallOptions`). Un échec est aussitôt `reread-failed`, que l'écran traite. Le chargement initial de la
  fiche garde son rejeu.
- **Écartée.** Un drapeau « relecture en cours » qui désactive les sept gestes : plus large, pour une fenêtre que le
  rejeu seul ouvrait.
- **Réversible.** Oui.

## C-e348-9 — joindre : le refus « déjà traité » sur son propre document (revue P2, B-4)

- **Contexte.** Un rattachement dont la requête expire côté client alors que le serveur l'écrit encore : la relecture ne
  montre pas de document, un second clic reçoit `409 IMPORT_NOT_PENDING_COMPLETION` ; la facture relue montre alors ce
  document, et la fiche notifiait un avertissement « déjà traité ».
- **Retenu.** Si le document relu est celui qu'on joignait, la fiche dit « Document joint à la facture. » ; sinon,
  l'avertissement d'avant. Court, dans la zone de la story : traité ici, sans issue.
- **Réversible.** Oui.
