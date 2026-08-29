# Story 24.4b : Le gel d'une écriture — refuser la réécriture et la destruction

## Status

ready-for-dev

## Story

**As a** personne qui tient les livres,
**I want** qu'une écriture enregistrée ne puisse plus être réécrite ni supprimée,
**so that** la seule correction possible soit celle qui se voit — la contre-passation.

## Le défaut, tel qu'il reste après la 24-4a

La **24-4a** a ouvert la porte de sortie : `POST /api/v1/journal-entries/{id}/reverse` crée
l'écriture inverse et laisse l'origine intacte. **Elle n'a rien fermé** — c'était voulu, et
l'ordre était contraint : geler avant que la contre-passation existe aurait rendu Kesh
incorrigible.

Aujourd'hui, donc, les deux voies coexistent :

- le `PUT` réécrit date, journal, libellé **et la totalité des lignes** — `DELETE FROM journal_entry_lines` puis réinsertion (`journal_entries.rs:1090`, dans `update`). L'état antérieur **disparaît des tables comptables** ;
- le `DELETE` est une **suppression physique** (`:1307`) ; aucun `deleted_at` n'existe au schéma ;
- l'unique verrou reste la **clôture annuelle** — `update` (`:1001`) et `delete_in_tx` (`:1259`) ne gardent que `fy_status == "Closed"`.

⚠️ **Art. 958f CO et Olico art. 3 : l'exigence n'est pas qu'on ne se trompe jamais, c'est que
la correction soit APPARENTE.** Tant que la réécriture destructive existe, la contre-passation
n'est qu'une option parmi deux — et c'est celle qui ne laisse pas de trace qui est la plus
rapide sous la main.

⛔ **Le trou défait ce que la vague vient de construire.** Le résiduel d'une facture se calcule
depuis `invoice_settlements.amount`, **jamais depuis l'écriture** (`amount_due`,
`invoice_settlements.rs:129-141`). Réécrire l'écriture d'un règlement fait donc diverger le
grand livre et le résiduel **en silence** — le mode d'échec exact du défaut fondateur que la
24-2 a corrigé.

⛔ **Et le risque explicitement accepté par la 24-4a est encore ouvert** : rien n'empêche de
réécrire par `PUT` une écriture **déjà contre-passée**, ni de réécrire ou supprimer **la
contre-passation elle-même**. Une origine réécrite après coup rend sa contre-passation
**fausse**, sans que rien ne le détecte. La 24-4a a tracé cette garde comme sa **première
exigence** ; elle est reprise ici, et le refus universel la couvre par construction (cf. D6).

## Ce que cette story fait, et ce qu'elle ne fait PAS

⛔ **Elle enlève.** Le `PUT` et le `DELETE` de `/api/v1/journal-entries/{id}` cessent
d'aboutir, **sans exception et sans condition** : toute écriture est comptabilisée dès son
insertion. La correction passe désormais par la contre-passation, livrée par la 24-4a.

⚠️ **Elle ne pose AUCUN verrou de période** — le seul verrou temporel reste la clôture
annuelle. C'est la **24-4c**, indépendante.

⚠️ **Elle n'introduit aucun statut brouillon.** Arbitrage du Project Lead : l'écriture reste
définitive dès l'insertion. Le brouillon pourra s'ajouter plus tard **sans rien défaire** —
et D1 montre qu'il sera même moins cher à ajouter plus tard que maintenant.

## D1 — AUCUNE colonne `status`, et c'est un choix, pas un oubli

L'issue #380 demande « un statut `comptabilisé` qui gèle l'écriture ». **Cette story livre
l'effet, pas la colonne.**

⛔ **Une colonne qui ne prend qu'une seule valeur n'est pas une donnée, c'est une décoration.**
Sans statut brouillon, `status` vaudrait `'posted'` sur 100 % des lignes, à la création comme
à la lecture : elle n'apprendrait rien à personne, et aucune requête ne pourrait la consulter
utilement. Le gel est un **comportement** — deux verbes refusés —, pas un état à stocker.

⚠️ **Et l'ajouter maintenant coûterait PLUS cher que l'ajouter plus tard**, ce qui retourne
l'argument de prudence :

| moment | migrations | conséquence |
|---|---|---|
| **maintenant** (`ENUM('posted')`) puis le brouillon plus tard | 2 — dont un `MODIFY COLUMN` pour étendre l'ENUM | ⛔ le second est **breaking** (P3) → bump `kesh_version_min_required` **et** bump Cargo du workspace (P2-bis) |
| **plus tard**, d'un seul geste (`ENUM('draft','posted') NOT NULL DEFAULT 'posted'`) | 1 — `ADD COLUMN` avec défaut | non breaking, aucun bump |

⇒ **Cette story ne touche AUCUNE migration.** Les garde-fous P1, P5, P6, P7 et P8 ne se
déclenchent pas, et `test-schema/0001_schema_squash.sql` reste inchangé.

⛔ **Le gate complet reste OBLIGATOIRE malgré tout.** La § *Review Iteration Rule* du
`CLAUDE.md` interdit le ciblage dès qu'un patch touche `crates/kesh-db/migrations/`,
`post_restore.rs` **ou un repository** — et celle-ci réécrit `journal_entries.rs`. L'absence de
migration ne lève pas l'exception `kesh-db`.

## D2 — Où le refus se place : dans le repository, jamais dans le handler

⛔ **Un garde-fou posé à la route ne protège que la route.** Le refus vit dans
`crates/kesh-db/src/repositories/journal_entries.rs`, au plus près de l'écriture SQL, pour
qu'aucun appelant futur ne puisse le contourner par inadvertance.

### (a) Le `DELETE` — un paramètre, pas une garde muette

⛔ **`delete_in_tx` a DEUX appelants, et le second ne doit PAS être gelé.**

| appelant | chemin | gel ? |
|---|---|---|
| `delete_by_id` (`journal_entries.rs:1210`) | la route `DELETE /api/v1/journal-entries/{id}` | **oui** |
| `invoices::delete` (`invoices.rs:1338`) | suppression d'une facture **validée** (#219) — la facture part, son écriture avec | **non** — cf. D5 |

⇒ `delete_in_tx` gagne un paramètre **`enforce_immutability: bool`**, sur le modèle exact
d'`enforce_postable` de `create_in_tx`. `delete_by_id` passe `true` ; `invoices::delete` passe
`false`, **avec un commentaire nommant le résidu et son issue**.

⛔ **Ne PAS poser la garde dans `delete_by_id` en laissant `delete_in_tx` nu.** Un futur
appelant de `delete_in_tx` obtiendrait alors le contournement **sans le savoir et sans que rien
ne rougisse** — c'est le mode d'échec du test muet, transposé au garde-fou.

**⚠️ La PRÉCÉDENCE des refus est figée ici, et elle n'est pas cosmétique** — elle décide du
message que l'utilisateur lit :

`FiscalYearClosed` (400) → **`ENTRY_IS_REVERSED`** (409) → **`ENTRY_IS_POSTED`** (409)

⛔ **`ENTRY_IS_REVERSED` passe AVANT, et c'est le point le plus facile à casser de toute la
story.** La garde de la 24-4a (`delete_in_tx`, étape 3-bis) dit à l'utilisateur *« cette
écriture a été contre-passée »*. Si le gel s'exprimait le premier, il répondrait *« corrigez-la
par une contre-passation »* — **un conseil faux, sur une écriture déjà contre-passée**. Et
`ENTRY_IS_REVERSED` deviendrait **injoignable par toute route**, sa garde et son test de la
24-4a se mettant à mesurer le vide.

### (b) Le `PUT` — la fonction `update` disparaît

`journal_entries::update` (`:914`) n'a **qu'un seul appelant** en production : le handler
`update_journal_entry` (`routes/journal_entries.rs:596`, l'appel à `:684`). Vérifié au sol —
`grep -rn "journal_entries::update" crates/` ne rend, hors de ce site, que des **doc-comments**
et des tests.

⛔ **`update` est SUPPRIMÉE, pas neutralisée par une garde en tête.** Poser le refus à la
première ligne laisserait derrière lui cent cinquante lignes qu'aucun chemin n'atteint plus :
la validation des projets, le *grandfathering* des tags archivés, le contrôle de version
optimiste, la garde d'exercice clos. Du code mort qui **paraît vivant** est exactement ce que
ce dépôt paie le plus cher — la passe 3 de revue de la 24-4a a été égarée une journée entière
par un doc-comment périmé de six mots.

⚠️ **Le corps supprimé n'est pas perdu** : il se relit à `git show d2910022:crates/kesh-db/src/repositories/journal_entries.rs`.
Quand le brouillon arrivera, sa fonction d'édition ne sera de toute façon **pas celle-ci** —
elle éditera un brouillon, pas une écriture comptabilisée, et n'aura ni le même contrat ni les
mêmes gardes.

Le handler `update_journal_entry` est réécrit :

1. il **ne désérialise plus de corps** — le paramètre `Json<UpdateJournalEntryRequest>` est
   retiré, et le type de requête avec lui s'il ne sert plus ailleurs ;
2. il résout l'écriture **scopée company** (`find_by_id`) ;
3. `None` → **404** ; `Some` → **409 `ENTRY_IS_POSTED`**.

⚠️ **Le 404 d'abord, toujours** : un `id` d'une autre société doit rendre 404 et non 409, faute
de quoi le code révélerait l'existence de la ressource (convention IDOR du dépôt).

⛔ **Ne pas retirer les routes du montage.** `PUT` et `DELETE` restent déclarés dans
`comptable_routes` (`lib.rs:332-339`). Une route démontée rend **405** — un statut qui
n'apprend rien à un client d'API et qui, à l'écran, ne se traduit par aucun message utile.

## D3 — Le code d'erreur : `ENTRY_IS_POSTED`, en 409

`DbError::EntryIsPosted` → HTTP **409**, code canonique `ENTRY_IS_POSTED`. Le gabarit est
`EntryIsReversed`, livré par la 24-4a — `errors.rs:299` côté `kesh-db`, `:2513` côté
`kesh-api` : le miroir se copie ligne à ligne.

⚠️ **409 et non 400** : c'est un conflit d'**état** de la ressource, pas une donnée d'entrée
invalide — la distinction que la 24-4a a figée en D5-bis, et qui doit rester lisible.

⛔ **Le message nomme le chemin de sortie.** Un refus utilisable dit quoi faire :

> *Une écriture comptabilisée ne se modifie plus. Pour la corriger, contre-passez-la : Kesh
> crée l'écriture inverse et conserve l'originale.*

⚠️ **Un code, jamais une phrase, côté API** (convention `FailedProposal` du `CLAUDE.md`) : la
traduction se fait à l'écran, dans les quatre locales.

## D4 — L'écran : les deux actions de ligne deviennent UN lien vers la fiche

La liste (`frontend/src/routes/(app)/journal-entries/+page.svelte:611-631`) porte aujourd'hui
deux boutons par ligne — ✎ `journal-entry-edit` et 🗑 `journal-entry-delete`. **Les deux
disparaissent**, et la cellule d'actions porte à la place **un lien vers la fiche**
`/journal-entries/{id}`.

⛔ **Ce lien n'est pas un lot de consolation : c'est l'ARBITRAGE DU 2026-08-29, replié ici.**
La liste ne renvoyait vers la fiche par **aucun `href`** — seuls la page d'un avoir et le grand
livre y menaient. Le bouton « Contre-passer » livré par la 24-4a vivait donc sur un écran qu'on
n'atteignait pas depuis la liste : l'AC 18 de la 24-4a était tenue, **la découvrabilité non**.
Cette story touchait déjà exactement cette cellule ; poser le lien ailleurs aurait coûté un
cycle de spec complet pour un `href`.

⇒ **Le chemin de correction devient continu** : liste → fiche → « Contre-passer ».

Conséquences mécaniques, toutes à traiter dans le même patch :

- le formulaire `JournalEntryForm.svelte` perd sa branche d'édition (`initialEntry`,
  `updatePayload`, l'appel `updateJournalEntry` à `:180`) et devient **création seule** ;
- `updateJournalEntry` et `deleteJournalEntry` sortent de `journal-entries.api.ts` ;
- l'état `editingEntry`, le mode `'edit'`, la modale de confirmation de suppression et le
  *toast* `journal-entry-deleted` sortent de la page de liste ;
- ⚠️ **la modale de conflit de version (409 `OPTIMISTIC_LOCK_CONFLICT`) devient injoignable
  depuis cet écran** — le verrou optimiste ne s'exerçait que sur le `PUT`. Retirer le chemin
  mort plutôt que le laisser en embuscade.

## D5 — Ce qui reste destructible, nommé et tracé

⛔ **Le gel n'est pas étanche, et le prétendre serait pire que le trou.** Trois chemins
détruisent encore une écriture :

| chemin | portée | verdict |
|---|---|---|
| `invoices::delete` d'une facture **validée** (#219) | production | ⚠️ **hors périmètre, assumé** — cf. ci-dessous |
| `kesh-seed::reset_demo` (`lib.rs:240`, `:260`) | production, route Admin | hors périmètre — #377 et #279 |
| `delete_all_by_company` | **tests seuls** (dix teardowns) | hors périmètre |

⚠️ **Pourquoi `invoices::delete` n'est PAS gelée ici.** Ce chemin ne supprime pas une écriture :
il supprime **une facture**, et son écriture part avec elle, sous trois gardes déjà en place —
facture non payée, non créditée par un avoir, sans historique de rappels (`invoices.rs:1236-1280`).
Rien ne reste orphelin, et le journal d'audit conserve l'instantané complet. Le geler
reviendrait à **retirer la suppression d'une facture validée**, c'est-à-dire à défaire la
fonctionnalité #219 et à décider ce qui la remplace (l'avoir) — une décision de **facturation**,
pas d'écriture, et un changement de périmètre que cette story n'a pas à trancher.

⇒ **À ouvrir en issue** au moment de la revue : *« Supprimer une facture validée détruit son
écriture — le gel de la 24-4b ne couvre pas ce chemin »*, jalon Vague 1, en renvoyant à #380
et à #381.

## D6 — Ce que le gel referme sans y toucher, et ce qu'il ne referme pas

**Refermé par construction** — le refus étant universel, aucune garde spécifique n'est à écrire :

- ⛔ **le risque accepté de la 24-4a**, dans ses deux moitiés : réécrire une écriture **portant**
  un `reverses_entry_id`, et réécrire ou supprimer une écriture **référencée** par celui d'une
  autre. La 24-4a l'avait tracé comme sa première exigence ; il n'a **pas** besoin d'un test de
  plus que ceux de l'AC 5, mais il a besoin d'être **nommé** dans le journal de la story, faute
  de quoi personne ne saura qu'il a été payé ;
- la réécriture silencieuse d'une écriture de règlement, qui faisait diverger grand livre et
  résiduel.

**NON refermé, et il faut le dire** :

- ⚠️ **#381 — les trous de numérotation.** Le gel supprime **la source principale** (la
  suppression manuelle) mais pas `invoices::delete`. #381 reste ouverte, et le manuel doit
  cesser de décrire la suppression d'écriture comme la cause courante des trous.

## D7 — ⛔ Le bilan d'ouverture n'est plus ré-éditable, et le manuel décrit la procédure morte

`create_opening_entry` (`journal_entries.rs:508`) refuse dès que la société contient **la
moindre écriture** (`count_by_company > 0`, étape 3). Aujourd'hui, corriger un bilan d'ouverture
faux se fait donc en **supprimant** les écritures puis en revenant sur l'écran « Soldes de
départ ». **Le gel ferme cette voie.**

⛔ **Et le manuel utilisateur l'enseigne, en toutes lettres** (`user-manual.tex:532`) :

> *« pour corriger un solde, modifiez l'écriture d'ouverture directement dans le journal […]
> Pour re-saisir intégralement les soldes, supprimez toutes les écritures de la société puis
> revenez sur l'écran. »*

**Les deux phrases deviennent fausses le jour du merge.**

**La voie qui reste**, et elle est comptablement correcte : **contre-passer** l'écriture
d'ouverture — la 24-4a l'autorise **délibérément**, aucune pièce ne la possède — puis saisir une
**OD manuelle** portant les soldes justes. Trois écritures au lieu d'une, et une piste de
correction lisible : c'est exactement ce que fait un comptable.

⚠️ **Aucune exception n'est ouverte pour l'écriture d'ouverture**, malgré la tentation. Une
dérogation « tant que c'est la seule écriture de la société » serait défendable — mais elle
introduirait dans le mécanisme une condition d'état que rien d'autre ne porte, et le gel
cesserait d'être une règle pour devenir un cas particulier. *L'arbitrage du Project Lead est
« l'écriture reste définitive dès l'insertion » : il est appliqué à la lettre.*

⇒ **Le manuel se réécrit dans le même patch**, et l'écran « Soldes de départ » doit dire ce
qu'il faut faire au lieu de laisser l'utilisateur découvrir un refus.

## D8 — RBAC et audit : rien ne change, et c'est vérifié

- Les routes restent sous `comptable_routes` (Admin + Comptable). **Consultation reçoit 403
  avant tout autre contrôle** — le refus de rôle précède le refus d'état.
- ⛔ **Aucune entrée d'audit n'est écrite pour une tentative refusée.** Le journal d'audit trace
  ce qui s'est produit, pas ce qui a été empêché ; l'y faire entrer le rendrait bruyant et
  contredirait `journal_entry.deleted` / `journal_entry.updated`, qui deviennent **sans
  émetteur** en production.
- ⚠️ Les deux actions `journal_entry.updated` et `journal_entry.deleted` restent **lisibles**
  dans l'historique existant : ne pas les retirer des tables de correspondance ni des écrans
  d'audit — des lignes anciennes les portent.

## Critères d'acceptation

1. `PUT /api/v1/journal-entries/{id}` sur une écriture existante de la société rend **409
   `ENTRY_IS_POSTED`**, quel que soit le corps envoyé — y compris un corps vide ou absent.
2. `PUT /api/v1/journal-entries/{id}` sur un `id` inexistant **ou appartenant à une autre
   société** rend **404**, jamais 409.
3. `DELETE /api/v1/journal-entries/{id}` sur une écriture existante rend **409
   `ENTRY_IS_POSTED`** ; sur un `id` inexistant ou d'une autre société, **404**.
4. Le message porté par `ENTRY_IS_POSTED` **nomme la contre-passation** comme chemin de
   correction.
5. `DELETE` sur une écriture **déjà contre-passée** rend **409 `ENTRY_IS_REVERSED`** — et non
   `ENTRY_IS_POSTED`. ⛔ La précédence est testée **explicitement**, par un test qui monte les
   deux causes à la fois et vérifie **laquelle** répond.
6. `DELETE` sur une écriture d'un exercice **clos** rend le refus d'exercice clos (**400**), qui
   précède les deux 409.
7. Le rôle **Consultation** reçoit **403** sur `PUT` et sur `DELETE`, avant tout autre contrôle.
8. `invoices::delete` d'une facture **validée** supprime toujours la facture **et** son écriture
   — un test le verrouille. ⛔ C'est le seul appelant qui passe `enforce_immutability = false`,
   et sans ce test la régression serait muette.
9. `journal_entries::update` **n'existe plus** dans `crates/kesh-db` ; aucun appelant, aucun
   doc-comment ne s'y réfère au présent.
10. Le bilan d'ouverture ne peut plus être corrigé par suppression : un test vérifie que
    `DELETE` sur l'écriture d'ouverture rend **409**, et que `create_opening_entry` refuse
    toujours sur une société non vierge.
11. **Écran** : la liste des écritures ne porte plus ni ✎ ni 🗑 ; chaque ligne porte un lien
    vers `/journal-entries/{id}`, ciblable par `data-testid`.
12. **Écran** : le formulaire de saisie est en **création seule** — aucun mode édition, aucun
    appel `PUT` ne subsiste dans le frontend.
13. **Écran** : la fiche d'une écriture demeure inchangée quant à la contre-passation
    (24-4a) — aucune régression sur le bouton, ses renvois croisés ni ses motifs de refus.
14. Les libellés d'écran neufs sont dans les **quatre** locales ; les **sept** clés devenues
    orphelines (`journal-entry-edit`, `journal-entry-delete`,
    `journal-entry-delete-confirm-{title,message,cancel,delete}`, `journal-entry-deleted`) sont
    retirées des **quatre** catalogues.
15. Les **deux** entrées mortes de la dette de sélecteurs
    (`journal-entries.spec.ts :: Supprimer` et `:: Annuler`) sont retirées de `DETTE_CONNUE` ;
    `:: Valider` reste, la saisie subsistant.
16. Le **manuel utilisateur** ne promet plus nulle part la modification ni la suppression d'une
    écriture — **six passages** sont à reprendre, dont la procédure de correction du bilan
    d'ouverture (cf. Dev Notes). Le PDF est régénéré et versionné.

## Invariants testables

- **I1 — Aucune écriture ne disparaît par la route.** Après une suite complète, le nombre
  d'écritures d'une société ne décroît jamais du fait d'un appel à `DELETE
  /api/v1/journal-entries/{id}` : tous ces appels ont été refusés.
- **I2 — Aucune ligne d'écriture n'est réécrite.** Pour toute écriture créée puis soumise à un
  `PUT`, `journal_entry_lines` est **identique** avant et après — comptes, montants, ordre,
  tags de projet — et `version` n'a pas bougé.
- **I3 — La correction reste possible.** Pour toute écriture qu'aucune pièce ne possède, il
  existe un chemin qui aboutit : `POST /{id}/reverse` rend **201**. ⛔ C'est l'invariant qui
  interdit de livrer un gel qui enfermerait l'utilisateur — il se teste sur une écriture d'un
  exercice **clos** et sur l'écriture d'**ouverture**, les deux cas où l'enfermement serait le
  plus coûteux.

## Tasks / Subtasks

- [ ] **T1 — L'erreur** (AC 1, 3, 4)
  - [ ] `DbError::EntryIsPosted` + `"ENTRY_IS_POSTED"` dans `crates/kesh-db/src/errors.rs`, miroir exact d'`EntryIsReversed`
  - [ ] mappage **409** dans `crates/kesh-api/src/errors.rs`, message nommant la contre-passation
- [ ] **T2 — Le `DELETE`** (AC 3, 5, 6, 8)
  - [ ] `delete_in_tx` gagne `enforce_immutability: bool` ; refus **après** `FiscalYearClosed` et **après** `ENTRY_IS_REVERSED`
  - [ ] `delete_by_id` passe `true` ; `invoices::delete` passe `false`, avec le commentaire nommant le résidu (D5)
  - [ ] doc-comments des deux fonctions repris — ⛔ ils décrivent aujourd'hui une suppression qui aboutit
- [ ] **T3 — Le `PUT`** (AC 1, 2, 9)
  - [ ] `journal_entries::update` **supprimée**, avec ses tests unitaires devenus sans objet
  - [ ] handler `update_journal_entry` réécrit : plus de corps désérialisé, `find_by_id` scopé, 404 sinon 409
  - [ ] `UpdateJournalEntryRequest` retiré s'il ne sert plus ; `grep` des doc-comments qui citent `journal_entries::update` au présent (⚠️ `admin_full_import_e2e.rs:1354`, `balance_sheet.rs:28`, `accounts.rs:444`)
- [ ] **T4 — L'écran** (AC 11, 12, 13)
  - [ ] liste : ✎ et 🗑 remplacés par un lien vers la fiche, `data-testid`
  - [ ] `JournalEntryForm.svelte` en création seule ; `editingEntry`, mode `'edit'`, modale de suppression et modale de conflit de version retirés
  - [ ] `updateJournalEntry` / `deleteJournalEntry` retirés de `journal-entries.api.ts`
- [ ] **T5 — i18n et dette de sélecteurs** (AC 14, 15)
  - [ ] clés neuves dans les **quatre** locales ; **sept** clés orphelines retirées des quatre
  - [ ] `journal-entries.spec.ts :: Supprimer` et `:: Annuler` retirées de `DETTE_CONNUE`
  - [ ] ⚠️ `i18n-keys.test.ts` (`ATTENDU.sitesTotal`, **1630**) rougira — ventilation documentée dans son doc-comment, avec le delta et son motif
- [ ] **T6 — Le manuel** (AC 16)
  - [ ] les **six** passages de `docs/manual/fr/user-manual.tex` (cf. Dev Notes), dont `:532`
  - [ ] `make fr` dans `docs/manual/`, PDF commité
- [ ] **T7 — Les tests**
  - [ ] `crates/kesh-api/tests/journal_entry_reversal_e2e.rs` étendu — c'est le fichier que la 24-4a a créé **pour porter aussi cette story**
  - [ ] la précédence de l'AC 5 testée en montant les deux causes ensemble
  - [ ] I1, I2, I3 — I3 sur une écriture d'exercice **clos** et sur l'écriture d'**ouverture**
  - [ ] les quatre tests du bloc `« Page écritures — modification (Story 3.3) »` de `journal-entries.spec.ts` réécrits ou retirés
- [ ] **T8 — Les gates** (⛔ complets, ciblage interdit — exception `kesh-db`)
  - [ ] base remise à zéro (KF-039), puis `scripts/test-fast.sh`
  - [ ] `npm run check` / `lint-i18n-ownership` / `test:unit` / `build`
  - [ ] suite Playwright complète, comparée à la baseline de `docs/testing.md`

## Hors périmètre

- **Le verrou de période** plus fin que l'année : **24-4c**.
- **Le statut brouillon** : arbitrage du Project Lead, et D1 montre qu'il est moins cher plus tard.
- **La suppression d'une facture validée** qui détruit son écriture : issue à ouvrir (D5).
- **Les trous de numérotation** : **#381**, que cette story ne ferme pas (D6).
- **`reset_demo`** : **#377**, **#279**.
- **Annuler un règlement** : **#414**. **Annuler un rapprochement** : **#418**.

## Dev Notes

### Règle de splitting — examinée, non déclenchée

La story touche cinq zones (`kesh-db`, `kesh-api`, `kesh-i18n`, `frontend`, `docs/manual`) mais
**un seul mécanisme** : deux verbes refusés. Aucune règle métier neuve, aucune migration, aucun
schéma. Le critère de la § *Règle de splitting préventif* — « plus de 5 modules **distincts** »
— n'est pas atteint, et le critère de non-convergence ne peut pas l'être avant la première passe.

### Fichiers à toucher

| fichier | nature |
|---|---|
| `crates/kesh-db/src/errors.rs` | UPDATE — `EntryIsPosted` |
| `crates/kesh-db/src/repositories/journal_entries.rs` | UPDATE — ⛔ `update` **supprimée** ; `delete_in_tx` + `delete_by_id` |
| `crates/kesh-db/src/repositories/invoices.rs` | UPDATE — le seul `enforce_immutability = false` |
| `crates/kesh-api/src/errors.rs` | UPDATE — mappage 409 |
| `crates/kesh-api/src/routes/journal_entries.rs` | UPDATE — handler `update_journal_entry` réécrit |
| `crates/kesh-i18n/locales/{fr,de,en,it}-CH/messages.ftl` | UPDATE — 7 clés retirées, les neuves ajoutées |
| `frontend/src/routes/(app)/journal-entries/+page.svelte` | UPDATE — la cellule d'actions devient un lien |
| `frontend/src/lib/features/journal-entries/JournalEntryForm.svelte` | UPDATE — création seule |
| `frontend/src/lib/features/journal-entries/journal-entries.api.ts` | UPDATE — deux fonctions retirées |
| `frontend/src/lib/shared/e2e-selecteurs-traduits.test.ts` | UPDATE — 2 entrées mortes |
| `frontend/src/lib/shared/i18n-keys.test.ts` | UPDATE — `sitesTotal` et sa ventilation |
| `frontend/tests/e2e/journal-entries.spec.ts` | UPDATE — le bloc « modification » |
| `crates/kesh-api/tests/journal_entry_reversal_e2e.rs` | UPDATE — la couverture du gel |
| `docs/manual/fr/user-manual.tex` (+ `.pdf`) | UPDATE — six passages |

### Le manuel, ligne par ligne — ⛔ AUCUN GATE NE LE LIT

C'est la leçon la plus coûteuse de la 24-4a : le manuel affirmait que le logiciel ne savait pas
contre-passer, **le jour où la story livrait la fonction**. Ici, six passages deviennent faux :

| ligne | ce qu'il dit aujourd'hui | ce qu'il faut |
|---|---|---|
| `:328` | « les écritures **déjà enregistrées** sur un tel compte restent **modifiables** » | plus aucune ne l'est |
| `:406` | « tant que l'exercice est ouvert, une écriture peut être **modifiée ou supprimée** » | la phrase entière tombe |
| `:435` | « Kesh **n'empêche pas encore** la modification ni la suppression d'une écriture déjà contre-passée » | c'est précisément ce que cette story fait |
| `:453` | les trous de numérotation viennent de la suppression d'écritures | la cause principale disparaît ; #381 subsiste par `invoices::delete` |
| `:502` | la clôture « verrouille » les écritures de l'exercice | à reformuler : le verrou n'est plus ce qui distingue un exercice clos |
| `:532` | ⛔ « **supprimez toutes les écritures** de la société puis revenez sur l'écran » | la procédure **n'existe plus** — cf. D7 |

⚠️ **Greper la VALEUR, pas la formulation** (§ *Propagation post-patch*) : chercher
`modifi`, `supprim` et `écritur` **séparément** sur les `.tex`, jamais la phrase du site corrigé.
Les manuels DE/EN/IT ne contiennent que des `README.md` — rien à propager.

### Pièges vérifiés au sol

- ⛔ **`delete_in_tx` est `pub(crate)`** : le paramètre neuf ne casse aucun appelant externe, mais les **deux** sites internes doivent être repris dans le même patch, sans quoi le crate ne compile pas — c'est le seul filet automatique de D2 (a).
- ⚠️ **`ENTRY_IS_REVERSED` n'est atteignable QUE par la précédence de D2 (a).** Si un implémenteur place le gel avant, le test de l'AC 15 de la **24-4a** passe au vert **en mesurant autre chose** — il n'échoue pas, il change de sujet. C'est le mode d'échec du test muet, et l'AC 5 existe pour l'attraper.
- ⚠️ **La modale « Conflit de version »** de la liste n'est atteinte que par le `PUT` : la retirer, ou elle devient un chemin mort qu'un futur lecteur croira vivant.
- ⛔ **`journal-entries-delete-blocked-reversed` est DÉJÀ orpheline, et personne ne l'a vu.** Livrée par la 24-4a dans les **quatre** catalogues (fr-CH `:343`), elle n'est consommée **nulle part** — `grep -rn "delete-blocked-reversed\|ENTRY_IS_REVERSED" frontend/src frontend/tests` rend **zéro résultat**. Le message de refus de suppression n'a donc jamais atteint l'utilisateur, et aucun gate ne l'a signalé : une clé **en trop** est muette là où une clé **manquante** fait rougir. ⇒ la retirer des quatre catalogues avec les sept autres, ce qui en porte le total à **huit**.
- ⚠️ **Les tests unitaires de `journal_entries.rs`** (32 dans `mod tests`) comptent plusieurs cas d'`update` : ils partent **avec** la fonction. Recompter le solde depuis la source avant de l'écrire dans le Change Log (§ *Recompter ses propres comptes rendus*), et **déclarer le périmètre** de mesure.
- ⚠️ **`invoices.rs:3764`** fait bien un `DELETE FROM journal_entries` en masse — mais il est dans `mod tests` (helper `cleanup_journal_entries`). Ne pas le confondre avec un chemin de production.
- ✅ **Aucune migration**, donc P5/P6/P7/P8 muets et `test-schema/` intact. ⛔ Le gate complet reste dû (D1).

### Références

- `journal_entries.rs:914` (`update`, à supprimer) · `:1210` (`delete_by_id`) · `:1233` (`delete_in_tx`, garde 3-bis de la 24-4a) · `:508` (`create_opening_entry`, garde « société vierge »)
- `invoices.rs:1236-1280` (les trois gardes de #219) · `:1338` (le second appelant de `delete_in_tx`)
- `routes/journal_entries.rs:596` (handler `update`, l'appel à `:684`) · `:710` (handler `delete`)
- `errors.rs` — `EntryIsReversed`, le gabarit à copier (`kesh-db:299`, `kesh-api:2513`)
- Story **24-4a** `_bmad-output/implementation-artifacts/24-4a-contre-passation-ecriture.md` — D2, D3 et son « Risque accepté »
- Issue **#380** ; Epic `_bmad-output/planning-artifacts/epic-24-vague1-livres-justes.md`
- `CLAUDE.md` §§ *Review Iteration Rule* (exception `kesh-db`), *Propagation post-patch*, *Recompter ses propres comptes rendus*, *Test Locally First*

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Journal de revue
