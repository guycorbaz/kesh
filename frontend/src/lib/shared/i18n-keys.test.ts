/**
 * **GARDE B — toute clé demandée par le frontend existe au catalogue.**
 *
 * Story 23-1a (#316). Pendant de la garde A (`kesh-i18n/src/loader.rs`,
 * `parity_between_locales`), qui vérifie que les quatre catalogues déclarent le même
 * ensemble de clés.
 *
 * ⚠️ **Le défaut que ces deux gardes ferment est SILENCIEUX.** `i18nMsg(clé, repli)`
 * retombe sur son repli — du français en dur — quand la clé manque, et `loader.rs`
 * charge `fr-CH` comme base de repli des trois autres locales. Un oubli de traduction
 * ne produit donc ni erreur, ni clé brute à l'écran : il produit **du français
 * correct**, servi à un germanophone, tous gates au vert.
 *
 * ⚠️ **L'INVERSION (D4-ter).** Cette garde n'énumère PAS les formes d'appel qui
 * marchent. Cinq passes de revue l'ont tenté, et chacune a trouvé une forme de plus :
 * littéral, gabarit, relais, appel multi-ligne, clé portée par une table, ternaire dans
 * le premier argument, gabarit affecté à une variable, clé fabriquée par une fonction.
 * *Une énumération de formes est ouverte par nature.* La garde inventorie donc le
 * **complément** — les sites qui ne résolvent PAS en littéral — un ensemble clos et
 * comptable, dont l'assertion de cardinalité ne peut pas être défaite par une forme
 * imprévue.
 */

import { describe, it, expect } from 'vitest';

import { IMPORT_ERROR_LABELS } from '$lib/features/imported-supplier-invoices/error-label';
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
// Seul import de production autorisé (AC7 / D1-bis) : un utilitaire de lecture, sans
// clé ni catalogue, partagé avec le moissonneur de la story 23-1b.
import { findCallSites, findRelays, masquerCommentaires } from './i18n-literal-reader.js';
// ⚠️ La règle de périmètre vivait ici en DEUX copies inline, à côté de la fonction du module
// partagé — trois exemplaires d'une même règle. Ils coïncidaient au caractère près, ce qui est
// la situation d'avant la dérive, pas une garantie contre elle. (Revue de code 23-1b, passe 3.)
import { dansLePerimetreDeFichier } from './i18n-harvest.js';
import { DETTE_CONNUE } from './i18n-dette-connue.js';

const LOCALES = ['fr-CH', 'de-CH', 'it-CH', 'en-CH'] as const;
const RACINE_FTL = '../crates/kesh-i18n/locales';

/**
 * Valeurs de contrôle, mesurées le 2026-08-19. Un écart se recompte, il ne s'ajuste pas.
 *
 * ⚠️ `sitesTotal` est passé de **1493 à 1497** puis à **1502** à la story 23-3, et chaque
 * changement est DÉLIBÉRÉ : les cinq derniers sites viennent de la passe 4 — trois pour le
 * statut de facture, qui s'affichait en français dans les quatre langues, et deux pour les
 * `placeholder="Qté"` restés en dur. **La borne a rougi la première, avant toute relecture.**
 * quatre chaînes françaises étaient **en dur** dans les écrans de factures fournisseurs — deux
 * « Chargement… », un « Qté », un « TVA » —, invisibles au moissonneur (qui ne lit que les
 * `i18nMsg`) comme à l'allowlist. Les couvrir crée quatre sites d'appel. C'est une **hausse
 * assumée**, pas une dérive : la borne exacte a fait exactement son travail en l'exigeant.
 *
 * ⚠️ **1502 → 1525 à la story 23-3b (#316), et les 23 se ventilent** — ce sont les libellés
 * qui n'appelaient AUCUNE fonction i18n, l'angle mort #255 : 9 pour `payment-batch-helpers`
 * (3 statuts + 6 codes d'échec), 3 pour `credit-note-helpers`, 3 + 4 pour les deux écrans de
 * factures (dont le titre du dialogue de validation), 3 pour le type d'organisation des
 * réglages, 1 pour `getGroupLabel`. **Là encore, la borne a rougi la première.**
 *
 * ⚠️ **1525 → 1529 à la story 23-4 (#316), et les QUATRE se ventilent** : ce sont les libellés
 * français qui étaient **en dur** dans les écrans que ce rollout achève de traduire — deux
 * `Chargement…` (`payment-batches` liste et fiche), un `Chargement...` à points ASCII (onboarding)
 * et le `<title>Paramètres - Kesh</title>` des réglages, dont la clé `settings-title` était
 * **déjà utilisée quatre lignes plus bas**. ⚠️ **Aucune garde ne pouvait les voir** :
 * `i18n-libelle-en-dur.test.ts` écarte les nœuds de texte de balisage, et elle le dit dans son
 * préambule. C'est la borne exacte qui les a fait apparaître — son travail.
 * ⚠️ Un cinquième site a été corrigé sans créer de site : le « (optionnel) » en dur d'onboarding
 * a été remplacé par `bank-accounts-labels-qr-iban`, déjà traduite pour ce champ exact.
 *
 * ⚠️ **1529 → 1556 à la story 23-7 (KF-044, #328), soit +27 sur UN seul écran** —
 * `settings/invoicing`, 306 lignes pour 5 appels `i18nMsg`. Recompté depuis la source aux deux
 * bornes : `git show HEAD:…invoicing/+page.svelte | grep -c 'i18nMsg('` rend **5**, le fichier
 * routé en rend **32**. ⚠️ Le décompte des *remplacements* en annonçait 25 : deux d'entre eux
 * créent **deux** appels chacun (le paragraphe des placeholders, scindé en `format-help` +
 * `seq-range` ; le bouton, scindé en `common-saving` + `settings-invoicing-save`). Compter les
 * gestes n'est pas compter les sites — c'est la borne qui l'a montré.
 *
 * ⚠️ **1556 → 1568, +12 pour le PROLONGEMENT du même écran** : `invoice-number-format.ts`
 * portait douze messages de validation en français en dur — « Le format de numérotation est
 * vide », « Padding {SEQ:11} invalide »… — affichés sous les deux champs de cet écran. Le
 * fichier est un module `.ts` **pur**, hors de portée de toutes les gardes de balisage ; ses
 * treize tests n'assertent que sur `.ok`, jamais sur le texte, donc rien ne les tenait.
 * ⚠️ Sept d'entre eux sont **interpolés** : le repli JS porte lui aussi les `{ $x }`, car
 * `i18nMsg` interpole `raw` — qui vaut le catalogue OU le repli. Un repli déjà rendu
 * afficherait « {30} » le jour où la clé manque.
 *
 * ⚠️ **Ce que cet écran a révélé, et qui vaut plus que le chiffre.** Ses **douze** clés
 * `settings-invoicing-*` **existaient déjà dans les quatre catalogues, traduites**, et
 * **aucune n'était demandée par le code**. C'est le **miroir** de #316 : là, le code réclame
 * des clés absentes ; ici, les clés attendent un code qui ne les appelle pas. Le symptôme à
 * l'écran est identique — du français servi à tous — mais **aucune garde ne voit ce sens-là** :
 * le moissonneur ne relève que les clés *demandées et absentes*, et la parité ne compare que
 * les catalogues entre eux. Une clé traduite que personne n'appelle est invisible des deux.
 *
 * ⚠️ **1568 → 1569, +1 pour un en-tête de tableau qui n'existait pas** : la colonne d'actions
 * de `BankProfileList.svelte` portait un `<th></th>` **vide**, qu'`axe` relève en
 * `empty-table-header` — un lecteur d'écran annonce une colonne sans nom. Le libellé est
 * masqué visuellement (`sr-only`) mais lu, sur le patron de `ReconciliationProposals:238`.
 * ⚠️ **Ce n'est PAS un libellé déplacé : c'est un libellé qui manquait.** Le site est donc neuf,
 * et la clé `bank-import-profile-labels-actions` aussi — relevée et non inventée, les quatre
 * formes étant attestées par `reconciliation-cols-actions`. Trouvé par le scan axe de
 * `bank-csv-import.spec.ts`, qui échouait depuis assez longtemps pour que la violation
 * s'installe (issue #107, KF-030).
 *
 * ⚠️ **33 → 34 sites non résolus, et c'est LÉGITIME** : `getGroupLabel`
 * (`routes/(app)/+layout.svelte`) est le symétrique exact de `getItemLabel`, deux lignes plus
 * haut et déjà de la liste — la clé est portée par la donnée, pas par le site d'appel. Un
 * dispatcher n'est pas une clé manquante.
 *
 * ⚠️ **1569 → 1602, +33 pour le grand livre (Story 24-1)**, et le décompte se ventile —
 * il n'a pas été relevé puis recopié : 19 sites dans `GeneralLedgerView.svelte` (l'écran
 * neuf), 10 dans `routes/(app)/reports/+page.svelte` (contrôles, onglet, export), 3 dans
 * `BalanceSheetView.svelte` et 1 dans `TrialBalanceView.svelte` — ces quatre derniers
 * étant l'infobulle du lien qui mène du solde à son détail. La somme vaut le delta, ce
 * qui est la seule vérification qui distingue un ajout légitime d'un ajout compensant
 * un retrait silencieux ailleurs.
 *
 * ⚠️ **1602 → 1605, +3 pour le filtre par compte (#374)** — les trois sites de
 * `routes/(app)/journal-entries/+page.svelte` : l'étiquette du filtre, et deux fois
 * « Tous » (le déclencheur du select et son option). La ventilation vaut le delta.
 *
 * ⚠️ **1605 → 1607, +2 pour l'écriture d'encaissement (Story 24-2, #371)** — les deux
 * sites de `routes/(app)/invoices/[id]/+page.svelte` : « Déjà réglé » et « Reste dû », les
 * deux lignes qui n'apparaissent au pied de la facture que **si** un règlement existe. Le
 * badge « Partiellement payée », lui, n'ajoute **aucun** site : `PaymentStatusBadge` résout
 * sa clé par une table indexée sur le statut, et ce dispatcher était déjà relevé.
 *
 * ⚠️ **1607 → 1613, +6 pour le règlement hors banque (Story 24-3, #372)**, et la ventilation
 * n'est pas monotone — c'est ce qui la rend informative :
 *
 * | site | delta |
 * |---|---|
 * | `SettleInvoiceDialog.svelte` (18) remplace `MarkPaidDialog.svelte` (7) | **+11** |
 * | `routes/(app)/invoices/[id]/+page.svelte` — le dialogue de dé-marquage disparaît | **−6** |
 * | `routes/(app)/invoices/due-dates/+page.svelte` | **+1** |
 *
 * Le dialogue en demande davantage parce qu'un **règlement a une contrepartie et un
 * montant**, là où un marquage n'avait qu'une date. Et la fiche en perd six parce que
 * « Dé-marquer payée » n'existe plus : annuler un règlement demande une contre-passation
 * (issue #414), pas un retrait de drapeau.
 *
 * ⚠️ **1613 → 1629, +16 pour la contre-passation (Story 24-4a, #380)**, tous sur
 * `routes/(app)/journal-entries/[id]/+page.svelte`, un écran qui n'appelait `i18nMsg`
 * nulle part jusqu'ici :
 *
 * | site | delta |
 * |---|---|
 * | `blockedLabel` — les **huit** codes de blocage rendus par le serveur | **+8** |
 * | le bouton, le succès, l'erreur | **+3** |
 * | les deux renvois croisés (« contre-passe » / « contre-passée par ») | **+2** |
 * | le dialogue : titre, corps, annuler, confirmer | **+4** |
 *
 * Les huit branches de `blockedLabel` comptent chacune pour un site : le serveur rend un
 * **code**, jamais une phrase, et c'est ici que la traduction se fait — un dispatcher
 * indexé par table n'aurait compté que pour un, mais n'aurait pas fait rougir le
 * type-check à l'ajout d'un code.
 *
 * ⚠️ **1629 → 1630, +1 en passe 1 de revue de code** : le huitième code,
 * `ACCOUNT_ARCHIVED`. Il manquait au recensement des empêchements, si bien que la
 * fiche affichait un bouton « Contre-passer » qui échouait une fois cliqué —
 * l'AC 11 exige l'inverse. Le code, le type TypeScript et le `switch` étaient
 * silencieusement alignés sur l'omission, ce qui explique qu'aucun gate ne l'ait vu.
 *
 * ⚠️ **1630 → 1619 à la Story 24-4b (#380), le gel de l'écriture** — une BAISSE, la première
 * depuis longtemps, et elle se ventile exactement :
 *
 * | mouvement | sites |
 * |---|---|
 * | les **onze clés** de l'édition, de la suppression et de la modale de conflit, retirées des quatre catalogues avec les chemins d'écran qui les appelaient | **−11** |
 * | le **troisième site** de `journal-entry-form-cancel`, qui vivait dans le bouton « Annuler » de cette même modale de conflit (il en reste deux) | **−1** |
 * | `journal-entry-open`, le lien de la liste vers la fiche qui remplace les deux boutons ✎ et 🗑 | **+1** |
 * | **total** | **−11** |
 *
 * ⛔ La baisse est **délibérée** : à la 24-4b, le `PUT` et le `DELETE` d'une écriture ne
 * rendaient plus que 409 `ENTRY_IS_POSTED` (jusqu'aux Stories 15-8a et 15-8b, qui les ont
 * rouverts sur la fiche), donc les écrans de la liste qui les appelaient n'existaient plus. Une borne qui
 * rougit sur une baisse fait le même travail que sur une hausse — elle exige qu'on dise
 * pourquoi.
 *
 * ⚠️ **1619 → 1630 à la Story 24-4c (#380), le verrou de période** — une hausse, et elle se
 * ventile exactement :
 *
 * | mouvement | sites |
 * |---|---|
 * | `journal-entries-books-locked-banner` — le bandeau de la liste des écritures | **+1** |
 * | `settings-books-lock-set` — employée **trois** fois (l'étiquette du champ, le bouton de pose, l'étiquette du champ de déverrouillage) | **+3** |
 * | `settings-books-lock-release` — employée **deux** fois (le bouton qui déplie, celui qui soumet) | **+2** |
 * | `settings-books-lock-{title,current,none,motif,motif-required}` — un site chacune | **+5** |
 * | **total** | **+11** |
 *
 * ⚠️ Les deux clés à **plusieurs** sites sont la raison pour laquelle ce compteur ne se déduit
 * pas du nombre de clés ajoutées : neuf clés neuves, onze sites. *Un total doit être cohérent
 * avec sa propre ventilation, et la ventilation se recompte depuis l'écran.*
 *
 * ⚠️ **1630 → 1631 à la Story 25-1a (#377), la piste de contrôle** — une hausse d'un seul
 * site, et elle se ventile :
 *
 * | mouvement | sites |
 * |---|---|
 * | `demo-reset-forbidden` — le toast du `DemoBanner` quand le backend rend 403 | **+1** |
 * | **total** | **+1** |
 *
 * ⛔ Le **masquage** du bouton hors rôle Admin, ajouté par la même story, n'ajoute AUCUN
 * site : `demo-banner-reset` existait déjà et n'est que déplacée sous un `{#if}`. Un
 * `{#if}` change qui voit la clé, jamais combien de fois le code la demande — et c'est
 * bien le second que ce compteur mesure.
 *
 * ⚠️ **1631 → 1638 à la Story 25-2-a ([#382], [#274]), l'avertissement bloquant au
 * retypage d'un compte mouvementé** — sept sites, et ils se ventilent :
 *
 * | mouvement | sites |
 * |---|---|
 * | `accounts-retype-title` — le titre de la modale d'avertissement | **+1** |
 * | `accounts-retype-warning` — sa description, **appel multi-ligne** | **+1** |
 * | `accounts-retype-closed-years` — le bloc des exercices clos, **appel multi-ligne** | **+1** |
 * | `accounts-retype-confirm` — le bouton qui confirme le retypage | **+1** |
 * | `accounts-updating` — l'état « en cours » de ce bouton | **+1** |
 * | `common-cancel` — le bouton d'annulation de la modale | **+1** |
 * | `error-unexpected` — la branche d'échec de `confirmRetype` | **+1** |
 * | **total** | **+7** |
 *
 * ⛔ **Deux de ces sept sites emploient une clé qui EXISTAIT DÉJÀ** — `common-cancel` et
 * `error-unexpected`. Le catalogue, lui, ne gagne que six clés, dont une
 * (`error-account-has-entries`) que le frontend n'emploie nulle part puisqu'elle sert le
 * message du backend. *Le nombre de clés au catalogue et le nombre de sites dans le code
 * sont deux grandeurs distinctes, et c'est la seconde que ce compteur mesure.*
 *
 * ⛔ **`accounts-updated` n'ajoute AUCUN site** : l'appel a été déplacé de `submitEdit`
 * vers `editSucceeded`, extrait pour que la confirmation et la soumission initiale
 * partagent le même chemin de succès. Le diff montre une ligne supprimée et une ajoutée —
 * un déplacement, pas une demande de plus.
 *
 * ⚠️ Cette ventilation a dû être **recomptée deux fois**. Le premier relevé, fait au
 * `grep` sur `i18nMsg\('<clé>'`, en annonçait cinq : il ne captait que les appels dont la
 * clé tient sur la même ligne, et manquait donc les deux appels multi-lignes. *Un
 * détecteur trop étroit sur un compteur qu'on s'apprête à corriger fait écrire un nombre
 * faux avec l'assurance de l'avoir mesuré.*
 */
const ATTENDU = {
	// Story 25-4-b1 (#416) : 1750 → 1751, soit **+1**, recompté depuis la source
	// (`grep -o 'i18nMsg('`, `main` contre l'arbre) — l'en-tête de la colonne
	// « Reste dû » de l'échéancier (`invoices/due-dates/+page.svelte` 29 → 30).
	// Fusion 25-1c-b1 + 25-3-c : 1709 + 31 + 10 = 1750 — les deux hausses portent sur des
	// fichiers disjoints et se cumulent.
	// Story 25-1c-b1 (#378), revue P1 : 1738 → 1740, soit **+2** — le message de plage
	// inversée (`audit-log-error-date-range`) et le « Précédent » de l'état vide.
	// Story 25-1c-b1 (#378) : 1709 → 1738, soit **+29**, recomptés depuis la source
	// (`grep -o 'i18nMsg('`, `main` contre l'arbre) — tous dans
	// `routes/(app)/audit-log/+page.svelte` (0 → 29) : titre ×2 (en-tête et
	// `<h1>`), sous-titre, huit libellés de filtres dont « Tous » / « Toutes » et
	// « Réinitialiser », l'export, six en-têtes de colonnes, « clé API »,
	// Afficher / Masquer, vide, erreur ×2 (liste et vocabulaire), Précédent /
	// Suivant, la plage « X–Y sur N », et les replis `common-loading` /
	// `common-error`. ⚠️ L'entrée de menu ne compte pas ici : c'est une DONNÉE
	// (`i18nKey`), résolue par `FAMILLES_RESOLUES['nav-']`, où `'audit-log'` entre.
	// `sitesNonResolus`, `relais` et `sitesGabarit` ne bougent pas : l'écran
	// n'appelle aucune clé à gabarit (le vocabulaire est traduit par la route).
	// Story 25-3-c (#454) : 1709 → 1719, soit **+10**, recomptés depuis la source
	// (`grep -o 'i18nMsg('`, `main` contre l'arbre) :
	//   • `features/supplier-invoices/invoice-cancel.ts` 0 → 6 : les six motifs ;
	//   • `supplier-invoices/[id]/+page.svelte` 35 → 39 : le complément « payée »
	//     de la confirmation, le toast, l'état « annulée », le repli `common-error`.
	//   ⚠️ `journal-entries/[id]/+page.svelte` ne bouge pas : son repli change de
	//   TEXTE, pas de nombre d'appels.
	// Story 25-3-b (#418) : 1692 → 1709, soit **+17**, recomptés depuis la source
	// (`grep -o 'i18nMsg('`, `main` contre l'arbre) :
	//   • `features/reconciliation/reconciliation-cancel.ts` 0 → 6 : les six motifs ;
	//   • `features/reconciliation/CancelReconciliationDialog.svelte` 0 → 6 : le titre,
	//     la lecture, les deux confirmations, Fermer, Confirmer ;
	//   • `bank-import/[id]/+page.svelte` 0 → 3 : la colonne, le lien, le bouton ;
	//   • `features/invoices/InvoiceSettlements.svelte` 9 → 10 : le bouton du rapprochement ;
	//   • `invoices/[id]/+page.svelte` 68 → 69 : le toast.
	//   ⚠️ `journal-entries/[id]/+page.svelte` (17) et `settlement-cancel-blocked.ts` (4)
	//   ne bougent pas : leur repli change de TEXTE, pas de nombre d'appels.
	// Story 25-3-a-2 (#414) : 1685 → 1692, soit **+7**, recomptés depuis la
	// source (`grep -o 'i18nMsg('`, `main` contre l'arbre) :
	//   • `supplier-invoices/[id]/+page.svelte` 29 → 35 (+6) : le bouton, la
	//     confirmation, l'avertissement de lot, le toast, le lien vers l'écriture
	//     de règlement, le repli `common-error` ;
	//   • `features/supplier-invoices/settlement-cancel.ts` 0 → 1 : la tête
	//     fournisseur. La queue commune, partagée, n'ajoute aucun site.
	//   ⚠️ `journal-entries/[id]/+page.svelte` reste à 17 : le repli
	//   d'`OWNED_BY_SUPPLIER_INVOICE` change de texte, aucun site n'y naît.
	//
	// Story 25-3-a-1 (#414) : 1665 → 1685, soit **+20**, recomptés depuis la
	// source (`grep -o 'i18nMsg('` aux deux bornes, HEAD contre l'arbre) :
	//   • `lib/shared/utils/settlement-cancel-blocked.ts` 0 → 4 : la queue
	//     commune des motifs d'annulation (exercice clos, rapprochement, compte
	//     archivé, pas d'exercice du jour) ;
	//   • `features/invoices/settlement-cancel.ts` 0 → 1 : la tête client
	//     (facture créditée) ;
	//   • `features/invoices/InvoiceSettlements.svelte` 0 → 9 : titre, quatre
	//     en-têtes, deux modes de règlement, lien vers l'écriture, bouton ;
	//   • `invoices/[id]/+page.svelte` 62 → 68 (+6) : le dialogue de
	//     confirmation (titre, corps, « Retour » par `common-back`, bouton), le
	//     toast de succès, et le repli `common-error`.
	//   ⚠️ `journal-entries/[id]/+page.svelte` reste à 17 : deux REPLIS y changent
	//   de texte (`OWNED_BY_SETTLEMENT`, `OWNED_BY_INVOICE`), aucun site n'y naît.
	//
	// Story 25-2-b-2 (#440) : 1638 → 1665, soit **+27**, ventilés — recomptés
	// depuis la source (`grep -c 'i18nMsg('` aux deux bornes), non incrémentés :
	//   • `invoices/[id]/+page.svelte` 41 → 62 (**+21**) : la modale de
	//     dévalidation qui naît — titre, corps, numéro conservé, bouton, plus son
	//     « Annuler » qui passe par `common-cancel`, **déjà en service ailleurs**
	//     et non une clé neuve —, ses deux messages de retour, le bouton de la
	//     barre d'action, `invoice-delete-numbered-warning` (neuve), et **5 sites
	//     qui cessent d'être codés en dur** sur des clés qui dormaient au
	//     catalogue : `invoice-delete-confirm-body` ×2, `invoice-delete-button`,
	//     `invoice-delete-confirm-title`, `invoice-deleted-success` — plus
	//     l'« Annuler » de cette modale-là, resté en dur à vingt-cinq lignes de
	//     sa jumelle —, **plus le bouton « Supprimer » de la barre d'action du
	//     brouillon**, en dur lui aussi et dans le bloc même que cette story
	//     entoure d'une garde de rôle ;
	//   • `invoices/+page.svelte` 7 → 13 (**+6**), sur l'écran de LISTE — le
	//     second chemin de suppression d'un brouillon, que quatre passes de
	//     validation n'avaient pas énuméré : l'avertissement sur le trou de
	//     séquence, le titre de sa modale, son toast de succès, ses deux boutons,
	//     et son corps, qui devient `invoice-delete-confirm-body-context` (clé
	//     **neuve et paramétrée** : la formulation de la liste nomme la date et
	//     le contact, que la clé de la fiche ne porte pas).
	//
	// ⛔ **Et un QUATRIÈME reste, trouvé en passe 3** : la modale de
	// **validation**, dont cette story avait câblé le titre et lui seul. Son
	// corps, ses deux boutons et son toast de succès restaient en français en
	// dur — alors que `invoice-validate-confirm-body`, `invoice-validate-button`
	// et `invoice-validate-success` dormaient, traduites, dans les quatre
	// catalogues. ⚠️ Le toast **invalidait rétroactivement** l'affirmation « plus
	// aucun toast codé en dur » d'un commit précédent : elle était plus large
	// que ce qui avait été vérifié. *(Une clé a dû être créée pour le cas sans
	// numéro : `invoice-validate-success` impose `{ $invoiceNumber }`.)*
	//
	// ⛔ **Ce compteur a donc été corrigé QUATRE FOIS, et chaque correction a
	// laissé un reste** : d'abord une ventilation fausse sous un total juste ; puis deux
	// clés câblées sur **un seul** des deux écrans, alors que la fiche de la
	// story prévenait qu'« un symptôme se grepe sur les DEUX écrans » ; puis les
	// **boutons**, que le contrôle repli/FTL ne voyait pas parce qu'il ne compare
	// que ce qui passe déjà par `i18nMsg` ; puis la modale de validation, qu'aucun
	// des trois contrôles précédents ne regardait parce qu'ils partaient tous
	// des clés `invoice-delete-*` énumérées par la fiche. *Un détecteur ne
	// trouve rien là où l'appel n'existe pas encore : chercher les clés mal
	// traduites ne révèle jamais celles qui ne sont pas appelées du tout — et
	// partir d'une liste de clés ne révèle jamais celles que la liste omet.*
	//
	// Story 25-4-c (#420) : **1751 → 1752**, recompté par ce test et par
	// `grep -c "i18nMsg("` aux deux bornes de `ReconciliationProposals.svelte`
	// (17 → 18) — la mention `reconciliation-labels-amount-due-of` (« reste dû
	// sur … »), clé neuve dans les quatre catalogues.
	//
	// Story 25-4-c3-a1 (#476) : **1752 → 1756**, recompté par ce test et par
	// `grep -o "i18nMsg("` aux deux bornes de `settings/invoicing/+page.svelte`
	// (+4) — la section « Différences d'arrondi » : titre, aide, libellé du
	// sélecteur (trois clés neuves `settings-invoicing-rounding-*` dans les quatre
	// catalogues) et son option vide, qui réemploie `settings-invoicing-select-none`.
	//
	// Story 25-5-b (#385) : **1756 → 1758**, recompté par ce test et par
	// `grep -c "i18nMsg("` aux deux bornes de `reports/TrialBalanceView.svelte`
	// (11 → 13) — la note « mouvement de la période » retirée (−1, clé supprimée des
	// quatre catalogues), l'en-tête « Solde » remplacé par « Ouverture » et
	// « Clôture » (+1, deux clés neuves `reports-column-opening` / `-closing`), et
	// le libellé de la ligne calculée, qui réemploie les deux clés du bilan (+2).
	//
	// Story 25-6-a (#388, #389) : **1758 → 1765**, recompté par ce test et par
	// `grep -o "i18nMsg(\|msg('"` aux deux bornes. La page d'accueil
	// `routes/(app)/+page.svelte` pesait **17** sites — ses 15 appels par son relais
	// `msg()`, plus la déclaration et le corps du relais —, elle en pèse **24** : 1 dans
	// la page et 23 dans les trois tuiles extraites (`features/homepage/`
	// `RecentEntriesCard` 5, `OpenInvoicesCard` 8, `BankAccountsCard` 10), qui appellent
	// `i18nMsg` directement. Le relais disparaît : `relais` **7 → 6**,
	// `sitesNonResolus` **34 → 32** (sa déclaration et son corps).
	//
	// Story 25-6-a, revue de code P1 : **1765 → 1767** — `BankAccountsCard` passe de 10 à
	// 12 appels (`grep -o "i18nMsg("`) : l'état d'échec de la tuile
	// (`homepage-bank-unavailable`) et l'écart non calculable d'un compte du grand
	// livre partagé (`homepage-bank-gap-unavailable`), deux clés neuves × 4 locales.
	// Story 25-6-b (#387) : **+9** (1758 → 1767 sur sa branche), recompté par ce test et par
	// `grep -o "i18nMsg("` aux deux bornes de `invoices/[id]/+page.svelte`
	// (69 → 78) — la mention « Document figé le », le bouton « Refiger le
	// document » (deux sites : l'en-tête et le pied du dialogue), le titre, le
	// corps et l'avertissement du dialogue, son « Annuler », le succès, et le
	// repli générique du refus — neuf sites. Le bouton PDF, extrait dans un snippet pour servir aussi la facture
	// annulée, n'ajoute aucun site : il n'est écrit qu'une fois.
	//
	// Story 25-6-b, revue P2 : **−2**, recompté par ce test et par
	// `grep -o "i18nMsg("` aux deux bornes — fiche facture 78 → 76, fiche avoir
	// 16 → 14, et le module partagé `shared/utils/pdf-error.ts` +2. Les deux
	// fiches résolvaient chacune le refus de PDF (`i18nMsg(clé, err.message)` et
	// le générique) ; elles délèguent désormais à `pdfErrorMessage`.
	//
	// Story 25-6-b, revue P3 : **+1** — l'avertissement « refigé, mais la
	// fiche n'a pas pu être relue » de la fiche facture (76 → 77), clé neuve
	// `invoice-pdf-refreeze-reload-failed` dans les 4 catalogues.
	//
	// Fusion de `main` (25-6-a) dans la 25-6-b, 2026-10-05 : les deux branches partaient de
	// 1758 ; 25-6-a +9, 25-6-b +8 (+9 −2 +1) ⇒ **1775**. `sitesNonResolus` : 32 (25-6-a)
	// − 1 (25-6-b, deux sites des fiches devenus un dans `pdf-error.ts`) ⇒ **31**.
	// Story 25-4-c3-b (#476) : **1756 → 1757**, recompté par ce test et par
	// `grep -o "i18nMsg("` aux deux bornes de `SettleInvoiceDialog.svelte`
	// (18 → 19) — le refus d'un montant à plus de deux décimales,
	// `invoice-error-amount-scale`, clé neuve dans les quatre catalogues.
	//
	// Fusion de `main` dans la 25-4-c3-b, 2026-10-05 : base commune 1756 ; `main` +19
	// (25-5-b, 25-6-a, 25-6-b), 25-4-c3-b +1 ⇒ **1776**. `sitesNonResolus` et `relais`
	// suivent `main` (31, 6) — la c3-b n'y touche pas.
	// Story 25-4-c4-b (#494) : **1757 → 1766**, recompté par ce test et par
	// `grep -o "i18nMsg("` aux deux bornes — `invoices/[id]/+page.svelte` (69 → 76 :
	// le récapitulatif passé en libellés traduits, sept sites dont la ligne
	// « Arrondi » et ses variantes « estimé ») et `settings/invoicing/+page.svelte`
	// (36 → 38 : la case « Arrondir à 5 centimes » et son aide).
	//
	// Fusion de `main` (via la 25-4-c3-b) dans la 25-4-c4, 2026-10-05 : la c4 ajoute
	// +9 à une base désormais de 1776 ⇒ **1785** ; `sitesNonResolus` et `relais` suivent
	// la fusion (31, 6) — la c4 n'y touche pas.
	// Story 25-4-e (#495) : **1766 → 1769**, recompté par ce test et par
	// `grep -o "i18nMsg("` aux deux bornes de `settings/invoicing/+page.svelte`
	// (38 → 41) — la section « Montant minimum » : titre, libellé, aide.
	//
	// Fusion dans la 25-4-e, 2026-10-05 : la story ajoute +3 à une base
	// désormais de 1785 ⇒ **1788** ; `sitesNonResolus` et `relais` suivent la fusion (31, 6).
	// Story 25-4-d1 (#384) : **1769 → 1775**, recompté par ce test et par
	// `grep -o "i18nMsg("` aux deux bornes de `settings/invoicing/+page.svelte`
	// (41 → 47) — la section « Solde du reste » : titre, aide, trois libellés de
	// compte, et le « — Sélectionner — » de son sélecteur.
	//
	// Fusion dans la 25-4-d1, 2026-10-05 : la story ajoute +6 à une base
	// désormais de 1788 ⇒ **1794** ; `sitesNonResolus` et `relais` suivent la fusion (31, 6).
	// Story 25-4-d2a (#384) : **1775 → 1777** — le motif « un solde existe »
	// (`INVOICE_WRITTEN_OFF`), traduit dans les deux tables qui le reçoivent :
	// `features/invoices/settlement-cancel.ts` (1 → 2) et
	// `features/reconciliation/reconciliation-cancel.ts` (6 → 7).
	//
	// Fusion dans la 25-4-d2a, 2026-10-05 : la story ajoute +2 à une base
	// désormais de 1794 ⇒ **1796** ; `sitesNonResolus` et `relais` suivent la fusion (31, 6).
	// Story 25-4-d2b (#490) : **1777 → 1807**, recompté par ce test et par
	// `grep -o "i18nMsg("` aux deux bornes de chaque fichier touché : la fiche
	// facture `invoices/[id]/+page.svelte` (76 → 85 : bouton, notifications et
	// dialogue d'annulation d'un solde, « Soldé »), `InvoiceSettlements.svelte`
	// (10 → 12 : libellé d'un solde, « Annuler le solde »), `WriteOffDialog.svelte`
	// (0 → 13) et `write-off.ts` (0 → 6 : quatre natures, deux aides).
	// Revue de code P1 de la 25-4-d2b : **1807 → 1808** — `WriteOffDialog.svelte`
	// (13 → 14) : « Reste dû en cours de calcul… », le dialogue restant monté
	// quand le reste n'est pas calculé.
	//
	// Fusion dans la 25-4-d2b, 2026-10-05 : la story ajoute +31 à une base
	// désormais de 1796 ⇒ **1827** ; `sitesNonResolus` et `relais` suivent la fusion (31, 6).
	// Story 25-4-d2c (#384) : **1808 → 1811** — `reports/VatReportView.svelte`
	// (9 → 12) : la section des diminutions de contre-prestation (titre, total
	// de la TVA des soldes, TVA due nette).
	//
	// Fusion dans la 25-4-d2c, 2026-10-05 : la story ajoute +3 à une base
	// désormais de 1827 ⇒ **1830** ; `sitesNonResolus` et `relais` suivent la fusion (31, 6).
	// Story 25-7 (#445) : **1830 → 1864** — `settings/opening-balances/+page.svelte`
	// (26 → 60) : totaux actif / passif et montant à porter, avertissement du report
	// à-nouveau (trois variantes), grille de complément (titre, introduction, date,
	// contrepartie, bouton, confirmation, succès) et six raisons d'indisponibilité.
	// Relevé du test, recoupé par `grep -o` sur le fichier aux deux bornes (+34).
	// Revue de code P3 de la 25-7 : **1864 → 1868** — un nom accessible sur les quatre
	// champs de montant (`opening-balances-debit-for` / `-credit-for`, 60 → 64).
	// Story 15-8a (#532) : **1868 → 1876** (+8), la modification d'une écriture
	// depuis sa fiche, ventilée par fichier (recompté par `grep -o` aux deux bornes) :
	//   - `features/journal-entries/JournalEntryForm.svelte` (28 → 31, +3) : le mode
	//     édition rétabli par inversion du gel — exercice clos en édition
	//     (`journal-entries-modify-blocked-fiscal-year-closed`), conflit de version
	//     (`journal-entries-edit-conflict`), compte inutilisable sur une ligne
	//     (`journal-entries-line-account-unusable`). ⛔ La modale de conflit de la 3.3
	//     est ÉCARTÉE de l'inversion : ses trois clés et le troisième site de
	//     `journal-entry-form-cancel` ne reviennent pas (C-15-8-21) ;
	//   - `routes/(app)/journal-entries/[id]/+page.svelte` (17 → 10, −7) : les huit
	//     `journal-entries-reverse-blocked-*` partent dans `blocker-messages.ts` (−8),
	//     le bouton « Modifier » arrive (`journal-entry-edit`, +1) ;
	//   - `features/journal-entries/blocker-messages.ts` (0 → 12, +12) : les huit
	//     motifs de contre-passation déplacés, plus les quatre motifs propres à la
	//     modification (`journal-entries-modify-blocked-*`).
	// Story 15-5c (#492) : **1876 → 1903 (+27 ; 1868 → 1895 avant le rebase sur la 15-8a)** — `reconciliation/failed-proposal-label.ts`
	// (0 → 27) : un site par code de `failed[]` (25 `case`, `FISCAL_YEAR_INVALID` et
	// `RECONCILIATION_FISCAL_YEAR_CLOSED` partageant le leur), plus la variante sans
	// numéros d'`ACCOUNT_NOT_POSTABLE` et le repli des codes inconnus.
	// `ReconciliationProposals.svelte` reste à 18 (compteur et échecs partiels déplacés,
	// non dupliqués). Relevé du test, recoupé par `grep -o "i18nMsg("` aux deux bornes ;
	// `sitesNonResolus`, `relais`, `sitesGabarit` inchangés (aucun site dynamique ajouté).
	// Revue de code P1 de la 15-5c (E1) : **1903 → 1904** (1895 → 1896 avant le rebase sur la 15-8a) — `failed-proposal-label.ts`
	// (27 → 28) : libellé dédié à la raison `payment_date_before_invoice_date` de
	// `RECONCILIATION_INVOICE_NOT_ELIGIBLE`. Recoupé par `grep -c "i18nMsg("` aux deux bornes ;
	// la doublure `ModalSuccessStub.test.svelte` est hors collecte (suffixe `.test.`).
	// Story 15-8b (#532) : **1876 → 1885** (+9 ; **1904 → 1913** sur l'état rebasé après la 15-5c), la suppression depuis la fiche et
	// l'historique, tous dans `routes/(app)/journal-entries/[id]/+page.svelte`
	// (10 → 19, recompté par `grep -o` aux deux bornes) : le bouton « Supprimer »
	// (`journal-entry-delete`), les quatre textes de la confirmation
	// (`journal-entry-delete-confirm-{title,message,cancel,delete}`), le toast
	// (`journal-entry-deleted`), le repli d'erreur du refus (`error-unexpected`),
	// la mention « Modifiée » (`journal-entry-modified`) et le lien « Historique »
	// (`journal-entry-history`).
	// Story 15-5d (#429, choix C34) : **1913 → 1915** (+2), le compte créanciers dans
	// `routes/(app)/settings/invoicing/+page.svelte` (47 → 49, recompté par `grep -o`
	// aux deux bornes) : son libellé (`settings-invoicing-payable-account`) et l'option
	// vide de son sélecteur (`settings-invoicing-select-none`). `sitesNonResolus`,
	// `relais`, `sitesGabarit` inchangés (aucun site dynamique ajouté).
	// Story 15-12a (#543) : **1915 → 1916** (+1), le `title` du bouton « Clôturer »
	// désactivé sous un exercice antérieur ouvert (`fiscal-year-close-blocked-earlier-open`)
	// dans `routes/(app)/settings/fiscal-years/+page.svelte` (56 → 57, recompté par
	// `grep -oE "\b(msg|i18nMsg)\("` aux deux bornes, `5e4bec50` et la branche).
	// `sitesNonResolus`, `relais`, `sitesGabarit` inchangés (un littéral).
	// Story 15-6b (#474) : **1916 → 1922** (+6 ; 1915 → 1921 avant le rebase sur la 15-12a), recompté par `grep -o "i18nMsg("` aux deux
	// bornes, fichier par fichier : `payment-batches/payment-batch-helpers.ts` (9 → 11 : les deux
	// codes neufs de `failedItemLabel`) ; `reconciliation/failed-proposal-label.ts` (28 → 30 :
	// `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT`, deux cas selon `details.role`) ;
	// `invoices/SettleInvoiceDialog.svelte` (19 → 20) et
	// `routes/(app)/supplier-invoices/[id]/+page.svelte` (39 → 40) : le message de liste vide
	// des comptes bancaires. `sitesNonResolus`, `relais`, `sitesGabarit` inchangés (aucun site
	// dynamique ajouté) ; la doublure `SettleInvoiceDialogHost.test.svelte` est hors collecte.
	// Story 15-12b (#543) : **1922 → 1925** (+3 ; 1916 → 1919 avant le rebase sur la 15-6b) — `failed-proposal-label.ts` (30 → 32 ; 28 → 30 avant le rebase) : le
	// libellé `LATER_FISCAL_YEAR_CLOSED` avec et sans nom d'exercice
	// (`reconciliation-failed-later-fiscal-year-closed{,-generic}`) ;
	// `routes/(app)/settings/fiscal-years/+page.svelte` (57 → 58) : le bandeau de l'état
	// hérité (`fiscal-year-out-of-order-warning`). Recompté par
	// `grep -oE "\b(msg|i18nMsg)\("` aux deux bornes (`012fc430` et la branche).
	// `sitesNonResolus`, `relais`, `sitesGabarit` inchangés (trois littéraux).
	// Story 15-6c (#474) : **1925 → 1919** (−6 ; 1922 → 1916 avant le rebase sur la 15-12b), la suppression de
	// `features/bank-accounts/BankAccountList.svelte` (code mort, choix C-15-6-22), qui
	// portait 6 `i18nMsg(` — recompté par `git show f8b2accd:<fichier> | grep -o "i18nMsg("`.
	// Les écrans touchés par la story n'en ajoutent aucun (filtres seuls). `sitesNonResolus`,
	// `relais`, `sitesGabarit` inchangés (six littéraux).
	// Story 15-1a-ii (#518) : **1919 → 1920** (+1) — `features/journal-entries/blocker-messages.ts`
	// (12 → 13) : le motif `ENTRY_LETTERED` (`journal-entries-modify-blocked-lettered`).
	// Recompté par `grep -o "i18nMsg("` aux deux bornes (`0724904c` et la branche). Les
	// replis réécrits (dialogue de contre-passation, soldes de départ) n'ajoutent aucun site.
	// `sitesNonResolus`, `relais`, `sitesGabarit` inchangés (un littéral).
	// Story 15-1a2-0 (#518) : **1920 → 1923** (+3) — un `i18nMsg(` neuf par famille
	// d'annulation, le motif du rang 2 bis (`*-cancel-blocked-lettering-closed`) :
	// `shared/utils/settlement-cancel-blocked.ts` (4 → 5),
	// `features/reconciliation/reconciliation-cancel.ts` (7 → 8) et
	// `features/supplier-invoices/invoice-cancel.ts` (6 → 7). Recompté par
	// `grep -o "i18nMsg("` aux deux bornes (`f9b6b199` et la branche).
	// `sitesNonResolus`, `relais`, `sitesGabarit` inchangés (trois littéraux).
	sitesTotal: 1923,
	sitesNonResolus: 31,
	relais: 6,
	sitesGabarit: 10,
	litterauxMin: 1050,
	clesDepuisTsMin: 5
} as const;

/**
 * Les 8 préfixes dynamiques et leurs valeurs, **écrites en dur**.
 *
 * ⚠️ Les lire depuis le code de production ferait qu'une carte vidée par erreur rendrait
 * le test vert à vide — le mode d'échec du test muet. La contrepartie est l'assertion de
 * cardinalité : elle rougit dès que la carte de production évolue sans le test.
 *
 * ⚠️ **Deux exceptions à cette contrepartie, et elles sont ASSUMÉES** :
 *  - `vat-category-*` : la colonne `vat_rates.category` n'a **aucune contrainte CHECK**
 *    (décision de la story 11-1, migration `20260613000001`). Un administrateur qui crée
 *    un taux étend l'espace des clés **sans toucher au code** : aucune assertion ne peut
 *    rougir. Les 5 catégories seedées sont déclarées, les catégories créées ne sont
 *    couvertes par aucune garde.
 *  - `bank-import-info-*` : les valeurs sont poussées par le BACKEND Rust
 *    (`kesh-api/src/routes/bank_imports.rs:693` et `:1668`), le frontend ne recevant
 *    qu'un `informational: string[]`. Il n'existe aucune carte à confronter. Un
 *    troisième code informationnel s'afficherait en `snake_case` brut, dans les quatre
 *    langues, sans qu'aucune garde ne bouge.
 */
const MOTIFS_DYNAMIQUES: Record<string, readonly string[]> = {
	'journal-': ['achats', 'ventes', 'banque', 'caisse', 'od'],
	'account-type-': ['asset', 'liability', 'revenue', 'expense'],
	'due-dates-filter-': ['all', 'unpaid', 'overdue', 'paid'],
	'reports-filename-': ['balance-sheet', 'income-statement', 'trial-balance', 'journals', 'vat'],
	// ⚠️ angle mort assumé — colonne libre, cf. le commentaire ci-dessus
	'vat-category-': ['normal', 'reduced', 'special', 'exempt', 'custom'],
	'reminders-error-': [
		'invoice-not-found', 'invoice-not-validated', 'invoice-already-paid', 'dunning-paused',
		'no-next-level', 'contact-archived', 'contact-email-missing', 'content-empty',
		'content-too-long', 'not-pdf-ready', 'rate-limited', 'database-error', 'smtp-failed',
		'sent-but-gone', 'sent-not-recorded'
	],
	'imported-supplier-invoices-error-': [
		'unsupported-file-type', 'file-too-large', 'symlink-rejected', 'duplicate',
		'no-qr-code-found', 'invalid-spc-payload', 'invalid-iban', 'pdf-render-error',
		'file-read-error', 'field-too-long'
	],
	// ⚠️ angle mort assumé — valeurs produites par le backend Rust, déclarées APRÈS
	// transformation (`info.replace(/_/g, '-')`), donc en tirets et non en snake_case.
	'bank-import-info-': ['bank-csv-profile-auto-matched', 'bank-csv-multiple-profile-matches']
};

/**
 * ⚠️ **TROISIÈME ensemble ouvert, et il n'a pas de préfixe** : `vat-rates/+page.svelte:52`
 * écrit `i18nMsg(r.label, r.label)` — **la clé elle-même vient de la colonne
 * `vat_rates.label`**, un `VARCHAR` libre sans contrainte `CHECK`. Aucune énumération
 * n'est possible : ce site figure à l'inventaire des non résolus et y restera.
 *
 * Les trois ensembles ouverts de cette story sont donc `vat-category-*`,
 * `bank-import-info-*` et celui-ci. *Aucune garde ne les borne, et c'est écrit plutôt
 * que découvert.*
 */
const ANGLE_MORT_CLE_EN_COLONNE = 'routes/(app)/settings/vat-rates/+page.svelte:52';

/**
 * Cardinalité attendue de chaque préfixe dynamique — **la contrepartie des valeurs
 * écrites en dur**, et elle manquait.
 *
 * ⚠️ Sans elle, `MOTIFS_DYNAMIQUES` peut **rétrécir en silence** : un « nettoyage » qui
 * retire deux valeurs de `journal-` sort deux clés de couverture sans qu'un seul test
 * rougisse — mutation exécutée en passe 3 de revue, **neuf tests verts**. Le docstring
 * de ce fichier promettait pourtant cette protection depuis le premier jet.
 *
 * L'invariant est **à deux places** : pour rétrécir une famille sans bruit, il faudrait
 * éditer la table ET ce compteur. C'est peu, mais c'est exactement ce qui sépare une
 * garde d'une déclaration d'intention.
 *
 * ⚠️ **QUATRIÈME angle mort, écrit ici faute de pouvoir le fermer** : `findRelays`
 * travaille **par fichier**. Un relais qui serait *importé* d'un module partagé ne
 * serait recensé nulle part — et, contrairement aux autres trous, **aucun compteur ne
 * bougerait**. La règle DRY du dépôt pousse vers cette extraction : les six relais
 * actuels sont six copies de la même fonction de trois lignes (sept jusqu'à la 25-6-a,
 * qui a retiré celui de la page d'accueil). À traiter en 23-2.
 */
// ⚠️ La carte des codes d'erreur d'import est LUE depuis la production, plus recopiée :
// tant qu'elle vivait dans le composant, une entrée ajoutée passait tous les gates au vert
// (mutation exécutée en passe 4 de la 23-3 : 111 tests verts sur une 11e entrée fantôme).
// Les six autres motifs restent énumérés — même dette, à extraire de la même façon.
const CARDINALITES: Record<string, number> = {
	'journal-': 5,
	'account-type-': 4,
	'due-dates-filter-': 4,
	'reports-filename-': 5,
	'vat-category-': 5,
	'reminders-error-': 15,
	'imported-supplier-invoices-error-': Object.keys(IMPORT_ERROR_LABELS).length,
	'bank-import-info-': 2
};

/**
 * **Les familles résolues de l'inventaire (D4-ter, étape 4).**
 *
 * Ces clés atteignent `i18nMsg` sans jamais s'écrire comme littéral au site d'appel :
 * elles vivent dans une table de données (`i18nKey`, `labelKey`), dans un index
 * (`LABEL_KEY[status]`), dans un ternaire, ou sont fabriquées par une fonction
 * (`accountRoleKey`) ou par un gabarit affecté à une variable (`AccountingTooltip`).
 *
 * ⚠️ **C'est ce trou qui a laissé QUATRE ENTRÉES DE LA BARRE DE NAVIGATION** —
 * `nav-credit-notes`, `nav-email-templates`, `nav-projects`,
 * `nav-supplier-invoices-import` — s'afficher en français dans les quatre langues,
 * hors de tout décompte, jusqu'à la passe 3 de revue de cette story.
 *
 * ⚠️ **Valeurs EN DUR**, comme celles de `MOTIFS_DYNAMIQUES` et pour la même raison :
 * les lire depuis la production rendrait le test vert à vide si la table se vidait.
 */
const FAMILLES_RESOLUES: Record<string, readonly string[]> = {
	// routes/(app)/+layout.svelte:154 — table `i18nKey` du menu principal
	'nav-': [
		'home', 'contacts', 'products', 'invoices', 'invoicing-due-dates', 'invoicing-reminders',
		'credit-notes', 'supplier-invoices', 'supplier-invoices-import', 'payment-batches',
		'accounts', 'fiscal-years', 'bank-accounts', 'bank-profiles', 'reconciliation-rules',
		'projects', 'export-global', 'settings', 'opening-balances', 'email-templates',
		'admin-backup', 'admin-restore',
		// Story 25-1c-b1 (#378) — l'écran du journal d'audit.
		'audit-log'
	],
	// routes/(app)/reports/+page.svelte:628 — table `labelKey` des onglets
	'reports-': [
		'balance-sheet', 'income-statement', 'trial-balance', 'journals', 'vat',
		'project-expenses', 'project-return', 'aged-balance'
	],
	// lib/features/invoices/PaymentStatusBadge.svelte:24 — index `LABEL_KEY[status]`
	'payment-status-': ['paid', 'unpaid', 'overdue'],
	// accounts/+page.svelte:62, opening-balances:133, BalanceSheetView:205 —
	// clé FABRIQUÉE par `accountRoleKey()` depuis `ACCOUNT_ROLES`
	'account-role-': [
		'receivable', 'default-revenue', 'payable', 'vat-recoverable', 'vat-payable',
		'vat-settlement', 'equity-capital', 'equity-other', 'retained-earnings',
		'current-year-result', 'none', 'archived-hint'
	],
	// routes/(app)/contacts/+page.svelte:646, :732, :829 — ternaire de littéraux
	'contact-type-': ['personne', 'entreprise'],
	// lib/features/bank-import/BankProfileForm.svelte:256 — ternaire de littéraux
	'bank-import-profile-labels-': [
		'bank-name', 'filename-pattern', 'filename-pattern-help', 'date-format',
		'field-separator', 'decimal-separator', 'encoding', 'header-row-count',
		'column-mapping', 'use-debit-credit-split', 'update', 'create'
	],
	// lib/shared/components/AccountingTooltip.svelte:49, :51 — gabarit affecté à une
	// variable (`const naturalKey = $derived(...)`), invisible en position d'argument
	'tooltip-': [
		'balanced-natural', 'balanced-technical', 'credit-natural', 'credit-technical',
		'debit-natural', 'debit-technical', 'journal-natural', 'journal-technical'
	]
};

/**
 * Les 10 sites de gabarit, **confrontés et non produits**.
 *
 * ⚠️ Le développeur ne doit pas *fabriquer* cette liste par l'extraction — c'est elle
 * qui a déjà échoué cinq fois. Il la *compare*. Un simple compte laisserait d'ailleurs
 * passer un ajout compensant un retrait.
 */
const SITES_GABARIT_ATTENDUS: readonly string[] = [
	'lib/features/bank-import/BankImportUpload.svelte  bank-import-info-',
	'lib/features/journal-entries/JournalEntryForm.svelte  journal-',
	'lib/features/journal-entries/VatPurchaseAssistant.svelte  vat-category-',
	'lib/features/reminders/reminder-error-label.ts  reminders-error-',
	'lib/features/reports/reports.api.ts  reports-filename-',
	'routes/(app)/accounts/+page.svelte  account-type-',
	'routes/(app)/invoices/due-dates/+page.svelte  due-dates-filter-',
	'routes/(app)/journal-entries/+page.svelte  journal-',
	'routes/(app)/settings/vat-rates/+page.svelte  vat-category-',
	'lib/features/imported-supplier-invoices/error-label.ts  imported-supplier-invoices-error-'
];

/**
 * Préfixes dont **toutes** les clés du catalogue sont demandées par le frontend, donc
 * pour lesquels une clé orpheline est un défaut. Repris du second contrôle de
 * `duplicate-i18n-keys.test.ts` (story 22-2b), que cette garde remplace.
 *
 * ⚠️ **Ne se généralise PAS** : le catalogue sert aussi `kesh-qrbill` et `kesh-report`
 * pour les PDF, si bien qu'une clé sans demandeur côté frontend n'est pas orpheline pour
 * autant — `reports-filename-*` en donne le contre-exemple, avec 7 clés déclarées pour
 * 5 valeurs de `ReportType`. La liste s'étend story par story, jamais par défaut.
 */
const PREFIXES_A_COUVERTURE_CLOSE: readonly string[] = ['contact-duplicate-', 'settings-invoicing-'];

// ⚠️ **`settings-invoicing-` entre ici à la story 23-7 (#328), et son cas est INSTRUCTIF.**
// Ses douze clés existaient dans les quatre catalogues, **traduites**, et le code n'en
// demandait aucune : l'écran affichait du français en dur pendant que ses traductions
// dormaient au catalogue. C'est le **miroir** de #316 — là, le code réclame des clés
// absentes ; ici, des clés attendent un code qui ne les appelle pas — et **aucune garde ne
// voyait ce sens-là** : le moissonneur ne relève que les clés *demandées et absentes*, la
// parité ne compare que les catalogues entre eux. La couverture close est le seul dispositif
// du dépôt qui ferme ce sens, et elle ne vaut que par préfixe déclaré.
//
// ⚠️ Le préfixe n'est devenu clos qu'après retrait de `settings-invoicing-format-too-long`,
// dernière orpheline : son message est désormais servi par `invoices-format-error-too-long`
// (le validateur ayant migré dans `features/invoices/`, où le lint d'ownership impose ce
// préfixe). La garder aurait été garder un doublon **traduit quatre fois** et jamais lu.

// ─────────────────────────────────────────────────────────────────────────────

function clesDuCatalogue(locale: string): Set<string> {
	const texte = readFileSync(join(RACINE_FTL, locale, 'messages.ftl'), 'utf-8');
	const cles = new Set<string>();
	for (const ligne of texte.split('\n')) {
		const m = /^([a-zA-Z][\w-]*)\s*=/.exec(ligne);
		if (m) cles.add(m[1]);
	}
	return cles;
}

type Releve = {
	litteraux: Map<string, string>;
	gabarits: string[];
	nonResolus: string[];
	sitesTotal: number;
	clesDepuisTs: Set<string>;
};

/** Parcourt `src` et classe chaque site d'appel. */
function relever(): Releve {
	const litteraux = new Map<string, string>();
	const gabarits: string[] = [];
	const nonResolus: string[] = [];
	const clesDepuisTs = new Set<string>();
	let sitesTotal = 0;

	const parcourir = (rep: string) => {
		for (const e of readdirSync(rep, { withFileTypes: true })) {
			const chemin = join(rep, e.name);
			if (e.isDirectory()) {
				parcourir(chemin);
				continue;
			}
			// ⚠️ Les fichiers `.test.*` sont hors collecte (D5-bis) : `i18n.svelte.test.ts`
			// demande `une-cle` et `compteur`, clés FICTIVES qui doivent le rester.
			if (!dansLePerimetreDeFichier(e.name)) continue;

			const texte = readFileSync(chemin, 'utf-8');
			const relatif = chemin.replace(/^src\//, '');
			for (const site of findCallSites(texte)) {
				sitesTotal += 1;
				if (site.arg === null) {
					nonResolus.push(`${relatif}:${site.line}`);
				} else if (site.arg.kind === 'template') {
					const prefixe = site.arg.value.slice(0, site.arg.value.indexOf('${'));
					gabarits.push(`${relatif}  ${prefixe}`);
				} else {
					if (!litteraux.has(site.arg.value)) litteraux.set(site.arg.value, relatif);
					if (chemin.endsWith('.ts')) clesDepuisTs.add(site.arg.value);
				}
			}
		}
	};
	parcourir('src');
	return { litteraux, gabarits, nonResolus, sitesTotal, clesDepuisTs };
}

/** Les clés que le frontend demande RÉELLEMENT : littéraux ∪ expansions des motifs. */
function clesDemandees(r: Releve): Set<string> {
	const toutes = new Set(r.litteraux.keys());
	for (const table of [MOTIFS_DYNAMIQUES, FAMILLES_RESOLUES]) {
		for (const [prefixe, valeurs] of Object.entries(table)) {
			for (const v of valeurs) toutes.add(prefixe + v);
		}
	}
	return toutes;
}

// ─────────────────────────────────────────────────────────────────────────────

describe('garde i18n — les clés demandées existent au catalogue', () => {
	const releve = relever();
	const catalogues = Object.fromEntries(LOCALES.map((l) => [l, clesDuCatalogue(l)]));
	const fr = catalogues['fr-CH'];
	const dette = new Set(DETTE_CONNUE);

	it('la collecte ne passe pas à vide', () => {
		// Bornes anti-test-muet : un motif d'extraction cassé rendrait tout ce qui suit
		// vert sans rien vérifier.
		expect(releve.litteraux.size).toBeGreaterThanOrEqual(ATTENDU.litterauxMin);
		// ⚠️ Borne par EXTENSION, posée à la valeur mesurée et non « mesuré moins une
		// marge » : deux des cinq clés `.ts` viennent d'appels multi-lignes en TypeScript
		// pur (`notify.ts`). Une borne à 3 serait verte sur leur perte.
		expect(releve.clesDepuisTs.size).toBeGreaterThanOrEqual(ATTENDU.clesDepuisTsMin);
		expect(releve.sitesTotal).toBe(ATTENDU.sitesTotal);
	});

	it('toute clé demandée existe au catalogue, ou figure à la dette connue', () => {
		const absentes: string[] = [];
		for (const cle of clesDemandees(releve)) {
			if (fr.has(cle) || dette.has(cle)) continue;
			absentes.push(`${cle}  (${releve.litteraux.get(cle) ?? 'expansion de motif'})`);
		}
		expect(absentes, `clés demandées et absentes des catalogues :\n  ${absentes.join('\n  ')}`)
			.toEqual([]);
	});

	it("l'allowlist de dette ne se fossilise pas", () => {
		const demandees = clesDemandees(releve);
		const obsoletes: string[] = [];
		for (const cle of DETTE_CONNUE) {
			if (fr.has(cle)) obsoletes.push(`${cle} : désormais au catalogue — retirer la ligne`);
			else if (!demandees.has(cle))
				// ⚠️ DEUX causes, et la seconde se vérifie d'abord : la feature a été
				// retirée (retirer la ligne), ou l'extracteur ne la voit plus (le réparer).
				obsoletes.push(`${cle} : plus demandée — feature retirée, OU extracteur cassé`);
		}
		expect(obsoletes, `entrées de dette obsolètes :\n  ${obsoletes.join('\n  ')}`).toEqual([]);
	});

	// ⚠️ **LE VERROU DE CLÔTURE DE L'EPIC 23 (story 23-6, 2026-08-22).** Les deux tests
	// ci-dessus tiennent la liste *cohérente* ; celui-ci la tient **vide**. Sans lui, rien
	// n'empêcherait d'y rajouter une ligne pour faire passer un gate — et le seul usage que
	// cette liste ne doit jamais avoir est précisément celui-là.
	//
	// ⚠️ **Ce n'est pas un test de forme : c'est ce qui rend la garde INCONDITIONNELLE.**
	// Tant que la liste pouvait accueillir une entrée, une clé manquante avait une issue
	// silencieuse ; désormais elle n'en a plus qu'une, écrire la valeur dans les quatre
	// catalogues. La liste a compté **317 clés** au kickoff de l'epic.
	//
	// ⚠️ **Si ce test rougit, la réparation n'est JAMAIS de le modifier.** Le message
	// d'échec nomme les clés : elles se traduisent, elles ne se tolèrent pas. Rouvrir
	// l'allowlist demanderait une décision de projet écrite — pas un patch de gate.
	it("l'allowlist de dette est VIDE — la garde i18n est inconditionnelle", () => {
		expect(
			[...DETTE_CONNUE],
			`la dette i18n est close depuis la story 23-6 : ces clés se TRADUISENT dans les ` +
				`quatre catalogues, elles ne se réinscrivent pas ici`,
		).toEqual([]);
	});

	it('les 6 relais locaux sont recensés — cardinalité assertée', () => {
		// ⚠️ **Ce garde-fou manquait au premier jet, alors que la tâche le déclarait fait.**
		// Sans lui, un huitième relais ajouté sans que le test le sache rendrait TOUTES
		// ses clés invisibles, en silence — le défaut que le recensement des relais avait
		// été créé pour clore, et qui avait déjà coûté 29 clés et un dossier entier.
		const trouves = new Set<string>();
		const parcourir = (rep: string) => {
			for (const e of readdirSync(rep, { withFileTypes: true })) {
				const chemin = join(rep, e.name);
				if (e.isDirectory()) {
					parcourir(chemin);
					continue;
				}
				if (!dansLePerimetreDeFichier(e.name)) continue;
				for (const nom of findRelays(masquerCommentaires(readFileSync(chemin, 'utf-8')))) {
					trouves.add(`${chemin}:${nom}`);
				}
			}
		};
		parcourir('src');
		expect(trouves.size, `relais recensés :\n  ${[...trouves].join('\n  ')}`).toBe(ATTENDU.relais);
	});

	it('le troisième ensemble ouvert est nommé, pas seulement compté', () => {
		// AC7-ter : les trois ensembles ouverts portent un commentaire. Les deux premiers
		// ont un préfixe et vivent dans MOTIFS_DYNAMIQUES ; le troisième n'en a pas — sa
		// clé vient d'une colonne libre —, il est donc nommé par son site.
		expect(releve.nonResolus).toContain(ANGLE_MORT_CLE_EN_COLONNE);
	});

	it('chaque préfixe dynamique porte sa cardinalité — la table ne rétrécit pas en silence', () => {
		expect(Object.keys(CARDINALITES).sort()).toEqual(Object.keys(MOTIFS_DYNAMIQUES).sort());
		for (const [prefixe, valeurs] of Object.entries(MOTIFS_DYNAMIQUES)) {
			expect(valeurs.length, `${prefixe} : ${valeurs.length} valeurs déclarées`).toBe(
				CARDINALITES[prefixe]
			);
		}
	});

	it('les 8 préfixes dynamiques sont déclarés, et leurs 10 sites confrontés', () => {
		expect([...new Set(releve.gabarits)].sort()).toEqual([...SITES_GABARIT_ATTENDUS].sort());
		expect(releve.gabarits.length).toBe(ATTENDU.sitesGabarit);
		const prefixesVus = new Set(releve.gabarits.map((g) => g.split('  ')[1]));
		expect([...prefixesVus].sort()).toEqual(Object.keys(MOTIFS_DYNAMIQUES).sort());
	});

	it('les familles résolues de l\'inventaire couvrent bien leurs clés du catalogue', () => {
		// Chaque famille déclarée doit correspondre à des clés réelles : une famille dont
		// aucune valeur n'existerait au catalogue signalerait une table renommée ou vidée.
		for (const [prefixe, valeurs] of Object.entries(FAMILLES_RESOLUES)) {
			const auCatalogue = valeurs.filter((v) => fr.has(prefixe + v)).length;
			expect(auCatalogue, `${prefixe} : aucune de ses ${valeurs.length} valeurs au catalogue`)
				.toBeGreaterThan(0);
		}
	});

	it("l'inventaire des sites non résolus est borné", () => {
		// ⚠️ **L'assertion qui porte la garantie.** Elle ne dépend d'aucune énumération de
		// formes : un site d'une forme inconnue y tombe automatiquement. Si elle rougit,
		// un nouveau site d'indirection est apparu — il doit être RÉSOLU (ses clés
		// déclarées) ou ÉCRIT comme angle mort, jamais simplement recompté.
		expect(
			releve.nonResolus.length,
			`sites dont le premier argument n'est ni littéral ni gabarit :\n  ${releve.nonResolus.join('\n  ')}`
		).toBe(ATTENDU.sitesNonResolus);
		// ⚠️ **8 de ces 33 sont des DÉCLARATIONS de fonction**, pas des sites d'appel :
		// `function msg(key: string, …)` et la déclaration d'`i18nMsg` elle-même sont
		// capturées par le motif `nom(`. Avec les 7 corps `return i18nMsg(key, fallback)`,
		// **15 des 33 sont du boilerplate de relais**. Vérifié en passe 3 de revue contre
		// les parseurs TypeScript et Svelte : 1485 sites d'appel réels, tous concordants.
	});

	it('aucune clé orpheline sur les préfixes à couverture close', () => {
		expect(PREFIXES_A_COUVERTURE_CLOSE.length).toBeGreaterThanOrEqual(1);
		const demandees = clesDemandees(releve);
		const couvertes = [...fr].filter((k) =>
			PREFIXES_A_COUVERTURE_CLOSE.some((p) => k.startsWith(p))
		);
		expect(couvertes.length).toBeGreaterThanOrEqual(5);
		const orphelines = couvertes.filter((k) => !demandees.has(k));
		expect(orphelines, `clés au catalogue que personne ne demande :\n  ${orphelines.join('\n  ')}`)
			.toEqual([]);
	});
});
