# Epic 34 — choix autonomes du groupe e343 (34-5, 34-6, 34-7, 34-8)

*Registre tenu en autonomie (Opus 5.5) le 2026-11-10, aux fiches. Worktree `/home/gcorbaz/devel/kesh-e34`, branche
`chore/epic-34-kickoff`, commit de référence `a75835db`. Mise à jour par union, jamais par écrasement.*

## C-e343-1 — #951 : version plutôt que clé d'idempotence

- **Contexte.** La question ouverte du fichier d'epic : la protection contre le double règlement passe-t-elle par une version ou
  par une clé d'idempotence ? `settle_invoice` ne lit pas la version (`invoice_settlements_write.rs:120-135`), alors qu'il
  l'incrémente déjà à chaque règlement, partiel compris (`:357-374`) ; le solde, lui, la contrôle (`:537-539`).
- **Retenu : la version de la facture**, exigée par `POST /invoices/{id}/settlements`, comparée sous le verrou de l'étape (1),
  `409 OPTIMISTIC_LOCK_CONFLICT` si elle diffère.
- **Pourquoi.**
  - Elle couvre les quatre cas de #951. Un second envoi après une réponse perdue porte l'ancienne version, et le premier l'a
    déjà incrémentée. Deux onglets ouverts sur la même facture n'écrivent qu'un règlement. Un client de l'API qui relance
    n'écrit pas deux fois. Un envoi qui attend le verrou du premier lit la version incrémentée.
  - **Une clé d'idempotence ne couvre pas les deux onglets** : chaque onglet tire sa clé, et les deux règlements passent.
    C'est le cas que #951 nomme en premier.
  - **Aucune migration.** Une clé demande une colonne et un index unique sur `invoice_settlements` (ou une table), la ligne
    d'audit d'idempotence, P6, un test de montée depuis une 0.17.5 peuplée, et une sauvegarde d'avant la mise à jour en tête
    des notes, sur une installation de production.
  - **Le patron existe** dans le même fichier (`write_off_invoice`, `:537`), dans l'acceptation d'un rapprochement
    (`accept_one_invoice`, `UPDATE … AND version = ?`) et dans les envois MCP.
  - Les deux écrans qui appellent la route ont déjà la version : `InvoiceResponse` sur la fiche, et `InvoiceListItemResponse`
    sur l'échéancier (`routes/invoices.rs:418`, alias `DueDateItemResponse`, `:974`).
- **Coût assumé.**
  - Un rejeu après une réponse perdue rend 409 et non la réponse d'origine. Une clé aurait rendu le règlement écrit.
    L'écran relit donc après un 409 et montre ce qui est écrit. Un client de l'API relit la facture par `GET`.
  - Deux règlements partiels **légitimes** lancés en même temps depuis deux onglets : le second est refusé et doit être
    relancé après relecture. C'était permis jusqu'ici (doc-comment de `SettleInvoiceRequest`, `routes/invoices.rs:1056-1059`,
    à récrire).
  - Tout geste qui incrémente la version entre l'ouverture de l'écran et le clic, comme une suspension des rappels, rend 409.
- **Écartées.**
  - Clé d'idempotence (ci-dessus).
  - **Les deux** : la clé n'ajoute que la réponse rejouée, au prix de la migration.
  - Empreinte du corps (montant, date, contrepartie) sur une fenêtre de temps : elle refuserait deux acomptes égaux et
    légitimes.
- **Réversible.** Oui, techniquement : une clé peut s'ajouter plus tard sans retirer la version. Mais le contrat publié
  dans une release ne se retire plus sans casser les clients.

## C-e343-2 — #951 : la version est obligatoire

- **Retenu.** `version: i32` sans `Option`, comme au solde (`WriteOffInvoiceRequest`, `routes/invoices.rs:1187-1190`). Un corps
  sans version est refusé par l'extracteur (`422`, `docs/api-external.md:294`), sans rien écrire.
- **Écartée.** Une version facultative, qui garderait les clients actuels : elle laisserait ouvert, pour tout client qui ne
  l'envoie pas, le P1 que la story ferme. Kesh n'a qu'un utilisateur réel, qui passe par l'interface.
- **Conséquence.** Changement de contrat cassant pour les intégrations : section *Modifié* du CHANGELOG, avec `⚠️`, et
  `docs/api-external.md`.
- **Réversible.** Oui avant publication.

## C-e343-3 — #951 : où se contrôle la version

- **Retenu.** Juste après la garde de statut, avant la date et avant toute lecture de compte. C'est l'ordre du solde
  (`:526-544` : statut, déjà payée, version, date). Une facture annulée garde son `409 ILLEGAL_STATE_TRANSITION`, qui en dit
  plus qu'un conflit de version.
- **Réversible.** Oui.

## C-e343-4 — #951 : le 409 à l'écran

- **Contexte.** Aujourd'hui, un 4xx laisse le dialogue ouvert avec le message du serveur (`invoices/[id]/+page.svelte:623-626`,
  `due-dates/+page.svelte:298-301`). Après un 409, un nouveau clic partirait avec la même version périmée et serait refusé
  de nouveau.
- **Retenu.** Sur `OPTIMISTIC_LOCK_CONFLICT`, le dialogue se ferme et la facture (ou la liste) est relue par le chemin de
  relecture de la 33-6. Un avis dédié dit « La facture a été modifiée depuis l'ouverture du dialogue : ce règlement n'a pas
  été enregistré. Vérifiez la liste des règlements avant de recommencer. » C'est un état distinct de « issue inconnue »,
  puisque l'issue est connue.
- **Écartée.** Garder le dialogue ouvert après une relecture silencieuse : l'utilisateur relancerait sans avoir vu le
  règlement écrit par l'autre onglet.
- **Réversible.** Oui.

## C-e343-5 — #951 : la relecture d'issue inconnue de la 33-6 reste

- **Retenu.** Le bouton reste indisponible jusqu'à une relecture réussie après une issue inconnue. Le serveur refuse
  désormais le double envoi, mais l'écran doit encore relire pour tenir une version fraîche : sans relecture, le clic suivant
  rendrait 409. Le doc-comment d'`isSettlementOutcomeUnknown` (`frontend/src/lib/shared/utils/unknown-outcome.ts`), qui dit
  « le POST d'un règlement ne porte aucune version », est récrit. Le prédicat, lui, ne change pas.
- **Réversible.** Oui.

## C-e343-6 — #953 : montrer l'écart, garder la saisie

- **Contexte.** Après un 409, les trois éditions de la société (e-mail, coordonnées, adresse) rechargent `data` mais gardent
  la saisie (`settings/+page.svelte:72-87`, `:136-152`, `:201-212`). Le clic suivant envoie la saisie avec la version
  fraîche et écrase l'écriture concurrente sans l'avoir montrée. Le test Vitest `settings-page.test.ts:289-310` et l'E2E
  `company-address.spec.ts:128-157` affirment ce comportement.
- **Retenu.** À l'ouverture de l'édition, l'écran garde une copie des valeurs enregistrées du bloc. Sur un 409, il relit et
  compare les valeurs du bloc à cette copie :
  - **aucun champ du bloc n'a changé** (version incrémentée par un autre bloc ; le verrou de période ne l'incrémente pas,
    `kesh-db/src/repositories/companies.rs:1043` — rectifié à la validation P1) : le message
    actuel, et l'essai suivant est sans danger ;
  - **la valeur enregistrée égale la saisie** (sa propre réponse perdue, ou la même correction faite ailleurs) : l'édition
    se ferme sur « déjà enregistré » ;
  - **sinon** : chaque champ **en conflit** montre sa valeur enregistrée entre-temps. Un champ est en conflit s'il a changé
    depuis la copie **et** diffère de la saisie (validation P1, 34-6-F1), la saisie reste, et un avis dit qu'enregistrer
    de nouveau la remplacera. La copie prend les valeurs relues, si bien qu'un nouveau conflit se compare à elles.
- **Écartées.**
  - Remplacer la saisie par les valeurs relues : le travail serait perdu sans le dire.
  - Un contrôle côté serveur : la version y est déjà. Le défaut est le seul écran.
- **Réversible.** Oui.

## C-e343-7 — #953 : une règle, trois blocs

- **Retenu.** Une fonction pure `companyEditConflict(base, stored, input, fields)` dans
  `frontend/src/lib/features/settings/company-conflict.ts`, et un composant d'avis
  `frontend/src/lib/features/settings/CompanyConflictNotice.svelte`, employés par les trois blocs (règle DRY). Les clés
  i18n de conflit sont communes aux trois. La passe 6 de la 16-3a avait voulu des clés dédiées par bloc parce que leur texte
  était générique (`settings/+page.svelte:137-141`) ; ici, le texte nomme le champ, si bien que le contexte voyage avec
  l'argument.
- **Réversible.** Oui.

## C-e343-8 — #945 : le renommage d'un exercice rejoué

- **Retenu.** `update_fiscal_year` appelle `fiscal_years::update_name` dans l'enveloppe `DbError`
  `kesh_db::retry::retry_on_deadlock`, opération `"fiscal_years::update_name"`. C'est le patron de la création, de la clôture
  et de la réouverture (`routes/fiscal_years.rs:286`, `:346`, `:407`). `update_name` est une transaction unique, audit compris,
  sûre à relancer ; ses refus métier sortent en `DbError::Invariant`, qui ne se rejoue pas.
- **La preuve.** Un test déterministe sur le patron de
  `fiscal_year_reopen_is_replayed_when_it_is_the_deadlock_victim` (`rejeu_interblocage_e2e.rs:2486-2529`). Une transaction
  lourde tient l'homonyme. La route renomme l'exercice au nom de l'homonyme : elle tient le sien, puis attend l'homonyme sur
  la lecture de doublon (`kesh-db/src/repositories/fiscal_years.rs:585-596`, index `uq_fiscal_years_company_name`). La lourde demande l'exercice de la
  route, la route est la victime. Après le rejeu, la route rend **400** « Un exercice avec ce nom existe déjà », et non 500.
  C'est le cas exact de C-e331-2.
- **Écartée.** Prendre les verrous dans un ordre fixe (par identifiant) : `update_name` ne connaît l'homonyme qu'après sa
  propre lecture, et l'ordre de la clôture est imposé par la règle des antérieurs.
- **Réversible.** Oui.

## C-e343-9 — #938 : la vue lue dans la transaction d'écriture

- **Retenu.**
  - `get_settlement_view_in_tx(tx, company_id, id)` est extraite de `get_settlement_view` (`supplier_invoices.rs:3002-3038`),
    qui l'enveloppe (règle DRY).
  - Une aide `commit_with_view<T>(tx, company_id, id, result: Result<T, DbError>) -> Result<(View, T), DbError>` lit la vue,
    puis valide la transaction, ou l'annule si l'écriture ou la lecture échoue. C'est une fonction ordinaire, non une macro,
    pour que la garde `syn` puisse la lire.
  - Les quatre enveloppes `validate`, `unvalidate`, `pay` et `cancel` rendent `SupplierInvoiceSettlementView`.
  - `cancel_settlement` rend un type propre, `SupplierSettlementCancelled { invoice: View, reversal_journal_entry_id }`.
    `SupplierSettlementCancellation`, que construit `cancel_settlement_in_tx` et qu'emploie l'annulation d'un rapprochement
    (`reconciliation_cancel.rs:608`), ne change pas. *(Rectifié à la validation P1, 34-8-F1 : ce point disait que
    `SupplierSettlementCancellation.invoice` devenait la vue, ce qui aurait ajouté des lectures à la forme `_in_tx`.)*
  - Les quatre routes qui tiennent leur propre transaction (`update_supplier_invoice_once`, `complete_import_once`,
    `attach_source_document_once`, `detach_source_document_once`) lisent la vue avant leur `commit` et la rendent.
  - `SupplierInvoiceResponse::load_with_settlement_cancellation` (`routes/supplier_invoices.rs:268-277`) n'a plus d'appelant :
    elle est **retirée**, et ses mentions dans les doc-comments sont reprises.
- **Pourquoi la vue en retour des enveloppes.** `SupplierInvoiceSettlementView` porte `invoice` et `lines` sous les mêmes noms
  que `SupplierInvoiceWithLines` (`:64`, `:2973`). Les appelants en test qui lisent `.invoice.status` ou `.lines` compilent
  sans changement. Ceux qui passent le retour à une fonction typée `SupplierInvoiceWithLines` ne compilent plus : leur
  nombre n'est pas relevé, le compilateur les nomme. Les formes `_in_tx` (`pay_in_tx`, `cancel_in_tx`,
  `cancel_settlement_in_tx`), réemployées par la confirmation d'un lot et le rapprochement, ne lisent rien de plus.
- **Écartées.**
  - Des fonctions jumelles `*_with_view` : elles doublent cinq enveloppes.
  - Une macro `in_own_tx_with_view!` : la garde `syn` ne lit pas les jetons d'une macro (limite écrite à la 33-6), et
    `validate` et `unvalidate` passent aujourd'hui par `in_own_tx!` (`:411-424`).
- **Réversible.** Oui.

## C-e343-10 — #938 : l'instantané de la transaction d'écriture

- **Contexte.** `get_settlement_view` lisait dans une transaction de lecture neuve, pour que la facture et ses motifs de refus
  décrivent le même état (doc-comment, `:2985-3001`). Dans la transaction d'écriture, une lecture ordinaire voit l'instantané
  pris à la **première** lecture ordinaire de cette transaction, avec ses propres écritures.
- **Retenu et assumé.** Ce qui a été écrit par la transaction elle-même est juste : facture, lignes, statut, version. Ce qui
  vient d'autres lignes (lot confirmé, rapprochement, période close d'une autre écriture) peut être plus ancien de la durée
  de la transaction. Les drapeaux `cancellable` et `settlementCancellable` sont indicatifs : le geste suivant les recalcule
  sous verrou et refuse. Le doc-comment de `get_settlement_view_in_tx` le dit. Aucune des lectures ne devient verrouillante
  (règle de `:2996-2998`).
- **Écartée.** Relire après le commit dans une transaction neuve : c'est le défaut de #938.
- **Réversible.** Oui.

## C-e343-11 — #938 : la garde de structure, mise en commun

- **Retenu.** Le visiteur `syn` de la 33-6 (`invoice_settlement_response_e2e.rs:395-516`) passe dans
  `crates/kesh-api/tests/common/autour_appel.rs`, employé par la garde du client et par celle du fournisseur (règle DRY). Ses
  limites écrites passent avec lui, sans changement.
- **Réversible.** Oui.

## C-e343-12 — #938 : l'écran fournisseur après une issue inconnue

- **Contexte.** Le commentaire de Guy sur #938 (2026-11-07) : l'écran fournisseur ne relit pas après une réponse perdue, et
  n'affiche rien sur une erreur hors `ApiError`. Lu : `pay()` (`supplier-invoices/[id]/+page.svelte:215-217`) n'écrit rien
  hors `ApiError`.
- **Retenu.** Sur les cinq gestes de la fiche (payer, annuler le règlement, annuler la facture, valider, repasser en
  brouillon), `isSettlementOutcomeUnknown(err)` (5xx, réponse perdue, erreur hors `ApiError`) mène à une relecture
  (`reloadAfterRefusal`, `:477-483`, qui rend désormais un booléen) et à un avis. Si la relecture a réussi : « L'issue n'est
  pas connue : la facture a été relue, vérifiez-la avant de réessayer. » Si elle a échoué : « … la facture n'a pas pu être
  relue : rechargez la page avant de réessayer ». *(Le second avis est ajouté à la validation P1, 34-8-F2.)* Un 4xx garde
  son traitement. Le prédicat existe déjà, son doc-comment s'élargit au fournisseur.
- **Pas d'état bloquant**, contrairement à la 33-6 : chacun de ces gestes est gardé par le statut ou par la version, si
  bien qu'un second envoi est refusé, sans double écriture. Pour le paiement, la garde est
  `crates/kesh-db/src/repositories/supplier_invoices.rs:1980` (`inv.status != "open"`). #951 la citait à `:1747`, numéro
  d'avant la 33-1.
- **Hors de la story.** Les pages de création et d'édition (`supplier-invoices/new`, `[id]/edit`). Issue demandée pour la
  création, qui n'a ni version ni clé.
- **Réversible.** Oui.
