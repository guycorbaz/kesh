# Story 25.2-b : Dévalider une facture — pour l'effacer, ou pour la corriger

Status: split

⛔ **DÉCOUPÉE le 2026-09-21** (arbitrage de Guy), en **25-2-b-1** (dépôt, route, empêchements,
audit, numéro conservé, `api-external.md`) et **25-2-b-2** (retrait de la branche `validated` de la
suppression, écran, E2E des deux cycles, manuels, `CHANGELOG`, `closes #440`). Motif : **critère de
non-convergence** de la § *Règle de splitting préventif* — sévérité maximale `CRITICAL → HIGH →
HIGH` sur trois passes de validation, chacune trouvant un défaut dans ce que la précédente venait
d'écrire.

⚠️ **Cette fiche n'est plus à implémenter, et elle ne se périme pas pour autant** : elle reste la
**source des faits établis** par ses trois passes — le tableau des huit empêchements et leur
précédence (AC 3), les deux trous de `main` qu'elle a mis au jour, les arbitrages de Guy, et les
prompts versionnés `25-2-b-validate-prompt-p1|p2|p3.md`. Les deux filles la **référencent** au lieu
de la recopier.

**Issue : [#440]**, ouverte le 2026-09-16. Elle **remplace [#219]** (fermée), dont le besoin est
conservé mais le chemin change. ⚠️ Elle **ne ferme pas [#381]** — cf. § *Ce que cette story ne fait
pas*.

## Story

En tant que **personne qui facture**,
je veux pouvoir **repasser une facture validée en brouillon**, puis soit l'**effacer**, soit la
**corriger et la revalider**,
afin de **réparer une erreur de facturation sans polluer les livres d'un avoir** qui ne correspond à
aucune réalité commerciale.

**Précédent cité par Guy (2026-09-16) : bexio se comporte ainsi.** *(Consigné comme référence de
comportement attendu ; la sémantique exacte de bexio n'a pas été vérifiée ici et ne doit pas être
invoquée comme spécification.)*

## L'arbitrage, et ce qu'il remplace

Guy, 2026-09-16, en deux temps :

1. d'abord « fermer le chemin résiduel de #381 » — l'écriture d'une facture ne doit plus disparaître ;
2. puis, la contre-passation s'étant révélée **inapplicable** (`ReversalBlocker::OwnedByInvoice`,
   `crates/kesh-db/src/errors.rs:63-64` : l'Epic 24 a décidé que l'écriture d'une facture se corrige
   par l'**avoir**, jamais par une extourne) : **« pour supprimer une facture validée, il faut
   pouvoir la repasser en brouillon, puis l'effacer »**, et — demandé explicitement —
   **« dévalider-effacer *et* dévalider-corriger-revalider »**.

⚠️ **Ce que cela ne fait pas, et il faut le dire net** : en deux gestes ou en un, **l'écriture est
supprimée de la même façon**. Cette story ne referme pas #381 ; elle rend la destruction **explicite
et auditable** au lieu d'être un effet de bord caché du verbe « supprimer ». La réattribution muette
des numéros est fermée par la story **25-2-c**, séparément.

**Ce qu'elle remplace** : la suppression directe d'une facture validée (#219, décision produit de Guy
du 2026-07-07 — *« suppression réelle, pas simple annulation »*). Le besoin est conservé ; c'est le
chemin qui change, et il gagne une étape nommée.

## L'état actuel, établi

- **Aucun chemin de retour vers `draft` n'existe.** `validate_invoice`
  (`crates/kesh-db/src/repositories/invoices.rs:1550`) est à sens unique ; les fonctions publiques du
  dépôt ne comportent aucune dévalidation.
- La contrainte `chk_invoices_validated_has_je` **anticipe déjà ce cas** en toutes lettres : *« une
  facture draft peut avoir journal_entry_id NULL (cas nominal) ou exceptionnellement non-null —
  aucun chemin applicatif actuel, mais autorisé par le CHECK »*
  (`20260417000002_invoice_validated_journal_entry_check.sql:8-13`).
- La validation écrit : `status = 'validated'`, `invoice_number` **tiré d'un compteur qui ne
  redescend jamais**, `journal_entry_id`, `version + 1`, et crée l'écriture de vente
  (`invoices.rs:1742-1845`).
- Le seul site du dépôt passant `enforce_immutability = false` à `journal_entries::delete_in_tx` est
  `invoices::delete` (`invoices.rs:1350`).

## Acceptance Criteria

1. **La transition existe.** `POST /api/v1/invoices/{id}/unvalidate`, dans la famille de
   `POST /api/v1/invoices/{id}/validate` (`crates/kesh-api/src/lib.rs:486`). Elle repasse la facture
   en `draft`, met `journal_entry_id` à `NULL`, **puis** supprime l'écriture — dans cet ordre, la FK
   `journal_entry_id … ON DELETE RESTRICT` l'imposant.

2. **Le numéro de facture est CONSERVÉ.** `invoice_number` n'est **pas** effacé par la dévalidation,
   et la revalidation le **reprend tel quel** sans tirer du compteur. ⛔ **Sans ce critère, la story
   transporte la maladie de #381 sur la numérotation des factures** : le compteur ne redescendant
   jamais, chaque cycle dévalider/revalider brûlerait un numéro et creuserait un trou dans la
   séquence des factures.
   ⛔ **Un brouillon numéroté ne change pas d'exercice** — arbitrage de Guy, 2026-09-19. Le numéro
   est tiré du compteur de l'exercice qui couvre la date (`invoice_number_sequences`,
   `UNIQUE (company_id, fiscal_year_id)`), et son année vient de cet exercice : redater une facture
   dévalidée dans un autre exercice lui ferait porter le numéro d'une séquence qui n'est pas la
   sienne, sans aucun signal. `PUT /api/v1/invoices/{id}` **refuse** donc une date hors de l'exercice
   du numéro conservé, sous un code propre (`INVOICE_NUMBER_FISCAL_YEAR_MISMATCH`, `409`).
   ⚠️ **La revalidation ne peut PAS porter « la même garde »** — relevé en passe 3 : `invoices` ne
   stocke aucun `fiscal_year_id`, et un gabarit de numéro peut n'avoir **aucun** marqueur d'année
   (`{SEQ}` seul suffit, `kesh-core/src/invoice_format.rs`), donc l'exercice d'origine n'est pas
   reconstructible depuis le numéro. `validate_invoice` garde ce qu'elle garde déjà : un exercice
   **ouvert** couvrant la date. Une fois le `PUT` gardé, l'état « brouillon numéroté dont la date a
   changé d'exercice » devient **inatteignable par les chemins applicatifs** — comme le motif 7 de
   l'AC 3 : son test se construit en base, à la main. Pour déplacer la facture, on la supprime et on en
   crée une. Test : dévalider, redater dans l'exercice suivant → refus ; redater dans le même
   exercice → accepté.

3. **Les empêchements forment une liste CLOSE, et chacun se justifie.** La dévalidation est refusée,
   avec un **code distinct par motif**, si la facture ou son écriture est dans l'un de ces états.
   Inventaire refait en passe 1 de validation depuis les migrations et le code — toutes les FK vers
   `invoices` et vers `journal_entries`, et tout **état** qui interdit aujourd'hui d'écrire dans une
   période —, non depuis une énumération de cas imaginés :

   | # | Empêchement | Lu dans | Gardé aujourd'hui par `invoices::delete` ? | Code |
   |---|---|---|---|---|
   | 1 | facture **réglée, même partiellement** | **`invoice_settlements`** (existence d'une ligne) **OU** `paid_at IS NOT NULL` | ⚠️ **NON pour un règlement partiel** — la garde lit `paid_at`, posé seulement quand le résiduel tombe à zéro (`invoice_settlements_write.rs:227-232`) ; les règlements partent alors en **cascade** | neuf |
   | 2 | facture **créditée** par un avoir | `credit_notes` (RESTRICT) | oui, code générique | neuf |
   | 3 | facture avec **historique de rappels** | `invoice_reminders` (CASCADE) | oui, code générique | neuf |
   | 4 | facture **envoyée au client** | `emailed_at` | **non** | neuf |
   | 5 | **exercice clos** | `fiscal_years.status` | oui, par `delete_in_tx` | `FISCAL_YEAR_CLOSED` (existant) |
   | 6 | écriture datée d'une **période verrouillée** | `companies.books_locked_through` | ⛔ **NON — trou réel sur `main`** | `PERIOD_LOCKED` (existant) |
   | 7 | écriture **rapprochée** d'une transaction bancaire | `bank_transactions.matched_entry_id` (FK `SET NULL`) | **non** — la FK effacerait le lien en silence | neuf |
   | 8 | écriture **contre-passée** | `journal_entries.reverses_entry_id` | oui, par `delete_in_tx` (inatteignable en pratique : `OwnedByInvoice` interdit de contre-passer l'écriture d'une facture) | `ENTRY_IS_REVERSED` (existant) |

   ⛔ **Deux trous RÉELS de `main`, que cette story ferme en passant** (établis en passe 1 de
   validation, par lecture du code) :
   - **le verrou de période (#6)** — ✅ **sorti en story 25-2-b-zero** (arbitrage de Guy, 2026-09-19),
     livrée **avant** celle-ci : la dévalidation s'appuie sur la garde qu'elle pose dans
     `delete_in_tx`, et n'en réécrit pas. `books_locked_through` n'est contrôlé qu'à la **création**
     d'une écriture (`journal_entries.rs:269-319`) ; `delete_in_tx` ne le lit pas. La 24-4c supposait
     que le gel de la 24-4b refusait toute suppression — mais le chemin `enforce_immutability = false`
     de `invoices::delete` y échappe. Supprimer aujourd'hui une facture validée datée d'un trimestre
     verrouillé **change ses totaux de TVA sans rien signaler**, alors que le manuel promet « aucune
     écriture ne pourra plus y être datée, **par aucun chemin** » (`user-manual.tex:464`). ⚠️ La garde
     se pose **dans `delete_in_tx`**, pour la raison qui y fixe déjà `enforce_immutability` : posée
     chez l'appelant, elle laisserait la fonction nue pour le suivant.
   - **le règlement partiel (#1)** : cf. le tableau. La garde lit l'**existence** d'une ligne de
     `invoice_settlements` **OU** `paid_at IS NOT NULL` — les deux, jamais l'un seul. ⚠️ `paid_at`
     seul rate le partiel ; la table seule rate les factures réglées **avant** sa création
     (`20260827000001`, « DDL PUR », sans rattrapage) : `paid_at` posé, aucune ligne. L'`UPDATE` de
     dévalidation heurterait alors `chk_invoices_paid_at_validated` et rendrait une erreur de
     contrainte opaque au lieu du code métier. Test : une facture payée **sans** ligne de règlement.

   ⛔ **« Un code distinct par motif » n'est PAS obtenu en déplaçant les gardes existantes.** Les
   trois gardes d'`invoices::delete` rendent toutes `DbError::IllegalStateTransition(m)`, mappé sur un
   code **unique** `ILLEGAL_STATE_TRANSITION` dont le message `m` n'est que journalisé, jamais rendu
   (`crates/kesh-api/src/errors.rs`, bras `IllegalStateTransition`). Le motif à suivre existe dans le
   dépôt : **`ReversalBlocker`** (`crates/kesh-db/src/errors.rs:57-104`) — un enum des empêchements,
   chacun avec son `code()` canonique, porté par une seule variante d'erreur. Les motifs 5, 6 et 8
   gardent leurs variantes et leurs statuts existants (`FISCAL_YEAR_CLOSED` et `PERIOD_LOCKED` en
   400, `ENTRY_IS_REVERSED` en 409) : les changer toucherait toutes les routes qui les rendent déjà.
   Les **cinq codes neufs** rendent **409 CONFLICT** — un refus fondé sur l'état de la ressource.

   **Où vit chaque garde, et l'ordre qui en découle.** Les motifs **1, 2, 3, 4 et 7** se contrôlent
   dans `unvalidate`, **avant** l'appel à `delete_in_tx` — le 7 ne peut pas vivre plus loin : la FK
   `bank_transactions.matched_entry_id … ON DELETE SET NULL` (`20260504000001_bank_imports.sql:83`)
   effacerait le lien au moment même de la suppression. Les motifs **5, 6 et 8** vivent dans
   `delete_in_tx`. La précédence réelle est donc **1, 2, 3, 4, 7, puis 5, 8, 6** — et non l'ordre du
   tableau, qui range par nature. Dans `delete_in_tx`, la garde de **période** se place **après** le
   gel (étape 3-ter) : placée avant, elle ferait passer la route gelée `DELETE /journal-entries/{id}`
   d'un `409 ENTRY_IS_POSTED` à un `400 PERIOD_LOCKED` sur une écriture de période verrouillée — un
   changement de contrat qu'aucune story n'a demandé. Un test pose deux empêchements à la fois
   (un de chaque côté de l'appel, p. ex. rappel **et** exercice clos) et vérifie que le premier est
   rendu. L'enum neuf compte **cinq** variantes (motifs 1, 2, 3, 4, 7) ; les motifs 5, 6 et 8 gardent
   leurs variantes existantes.
   **Codes neufs** : `INVOICE_HAS_SETTLEMENTS`, `INVOICE_CREDITED`, `INVOICE_HAS_REMINDERS`,
   `INVOICE_EMAILED`, `ENTRY_IS_RECONCILED` — tous en `409`.
   ⚠️ **Le motif 7 est inatteignable par les chemins de l'application** : les cinq sites qui posent
   `matched_entry_id` (`reconciliation.rs`) le posent sur une écriture **neuve**, jamais sur
   l'écriture de vente. Sa garde reste — défense contre un état posé autrement —, et son test
   construit l'état à la main.

   ⛔ **`emailed_at` : REFUS SEC, non levable par confirmation.** Arbitrage de Guy, 2026-09-16 :
   *« on ne doit pas pouvoir modifier une facture créée et envoyée à un client »*. C'est le seul des
   huit empêchements qui ne se déduise pas du schéma — les autres protègent une donnée interne,
   celui-ci protège un document **sorti de Kesh**, que le client détient. Une fois la facture partie,
   le chemin de correction redevient l'**avoir**, et c'est cohérent : l'avoir existe précisément pour
   corriger ce qu'un tiers a déjà reçu.

   ⛔ **La fenêtre de l'envoi — LIMITE ASSUMÉE, et voici pourquoi le renversement est écarté.**
   `mark_emailed` pose `emailed_at` **après** l'envoi SMTP (`invoices.rs:2067-2110`) : pendant
   l'envoi, une dévalidation voit la facture vierge et passe. Marquer **avant** d'expédier, conduite
   d'abord retenue, est **écarté en passe 3 de validation** : `mark_emailed` pose la marque **et**
   l'entrée d'audit `invoice.emailed` dans la même transaction, et le test existant
   `smtp_failure_returns_500_and_does_not_mark`
   (`crates/kesh-api/tests/invoice_send_email_e2e.rs`) exige qu'un échec SMTP n'écrive **aucun**
   audit. Le renversement imposerait donc de désolidariser marque et audit — un refactor du chemin
   d'envoi, hors de cette story —, et il paierait ce prix par un mode d'échec **pire** : une panne
   entre la marque et son retrait laisserait la facture marquée à tort, donc **indévalidable pour
   toujours**, le motif 4 étant sans confirmation possible. La fenêtre reste donc ouverte, de la
   durée d'un envoi, et la story l'écrit : *pendant l'expédition d'un e-mail, une dévalidation
   concurrente peut encore passer.* Elle se referme d'elle-même le jour où l'envoi deviendra
   asynchrone, avec un état « en cours » à consulter.

   ⚠️ **La garde n'attrape que ce que Kesh sait avoir envoyé.** `emailed_at` est renseigné par
   `mark_emailed` ; une facture dont le PDF a été téléchargé puis transmis à la main ne laisse
   **aucune trace**, et restera dévalidable. La limite est structurelle, pas réparable ici — elle
   s'écrit, elle ne se comble pas.

4. **Deux sorties, et elles sont symétriques.**
   - **effacer** : la facture redevenue `draft` se supprime par le chemin brouillon **existant**, sans
     garde nouvelle — il n'y a plus d'écriture à protéger ;
   - **corriger et revalider** : la facture `draft` se modifie par `PUT /api/v1/invoices/{id}`
     **existant**, puis se revalide par la route de validation existante, qui recrée une écriture
     **neuve** portant un numéro d'écriture **neuf**. C'est correct et voulu : ce n'est pas la même
     écriture.

   ⚠️ **Effacer un brouillon numéroté laisse un trou dans la séquence des FACTURES** — le compteur
   ne redescend pas, et c'est voulu : un numéro émis ne se réattribue pas. Le trou est expliqué par
   le journal d'audit. La confirmation de suppression d'un brouillon **qui porte un numéro** le dit
   (« le numéro F-… ne sera pas réutilisé ») ; celle d'un brouillon jamais validé reste inchangée.

   ⚠️ **Les deux sorties ne sont pas ouvertes aux mêmes rôles — asymétrie arbitrée par Guy le
   2026-09-19.** La suppression d'une facture, brouillon compris, reste réservée à
   l'**Administrateur** (`admin_routes`, décision de #219) et fermée aux clés API. Le Comptable et une
   clé `read-write` peuvent donc **dévalider puis corriger**, non **dévalider puis effacer** :
   effacer creuse un trou définitif dans la séquence, et c'est un geste d'administrateur. L'écran le
   dit au Comptable plutôt que de lui laisser découvrir un `403`.

5. **`invoices::delete` ne traite plus que les brouillons.** Sa branche `status == "validated"` et
   ses trois gardes disparaissent, déplacées vers la dévalidation. ⛔ **Conséquence à vérifier comme
   un critère** : `enforce_immutability = false` n'a plus alors qu'un appelant — le nouveau — et le
   drapeau doit rester **dans `delete_in_tx`**, jamais chez l'appelant, pour la raison déjà écrite à
   `journal_entries.rs` : une garde posée chez l'appelant laisserait la fonction nue pour le suivant.
   ⚠️ **Cinq tests existants portent sur cette branche** et doivent être **réécrits contre
   `unvalidate`**, non supprimés : `test_delete_validated_unpaid_open_fy_removes_invoice_and_je`,
   `test_delete_validated_paid_is_rejected`, `test_delete_validated_in_closed_fy_is_rejected`,
   `test_delete_validated_credited_by_avoir_is_rejected`, `test_delete_validated_with_reminders_is_rejected`
   (`crates/kesh-db/src/repositories/invoices.rs`). Rougiront aussi :
   `crates/kesh-api/tests/invoice_delete_e2e.rs:184` `delete_validated_as_admin_returns_204`, et le
   Playwright `frontend/tests/e2e/invoices.spec.ts:333-367` (« fiche fantôme »), qui supprime une
   facture **validée** par l'API — à réécrire, le premier contre `unvalidate`, le second en
   dévalidant d'abord. Supprimés, ils emporteraient la couverture des
   trois gardes existantes ; conservés tels quels, ils rougiraient. Un test neuf vérifie qu'une
   facture validée passée à `delete` est **refusée** — la branche retirée ne doit pas redevenir un
   chemin silencieux.
   ⛔ **Un SIXIÈME test casse, et il vient de la story prérequise** (relevé en passe 3) :
   `delete_validated_in_locked_period_returns_400_period_locked`, ajouté par la **25-2-b-zero**
   (#443) dans ce même fichier `invoice_delete_e2e.rs`. Il supprime par l'API une facture **validée**
   d'une période verrouillée et attend `400 PERIOD_LOCKED` ; la branche `validated` retirée, il
   recevra le refus générique **avant** d'atteindre `delete_in_tx`. Il se **réécrit contre la
   dévalidation** — c'est le même motif 6, par le nouveau chemin —, il ne se supprime pas : la
   couverture du verrou de période à la suppression ne doit pas disparaître avec lui. ⚠️ **Ils sont
   donc SIX, et non cinq** : le décompte se recompte, il ne se recopie pas.

6. **L'audit nomme le geste.** Code d'action neuf `invoice.unvalidated`, avec un instantané de l'état
   avant (numéro, total, écriture supprimée), en plus du `journal_entry.deleted` que
   `delete_in_tx` journalise déjà. Le registre `crates/kesh-api/src/audit_labels.rs` est sur `main`
   depuis la #439 (`c7de37c4`) : le code entre dans `ACTIONS` et reçoit
   `audit-log-action-invoice-unvalidated` dans les **quatre** locales ; la garde
   `crates/kesh-api/tests/audit_label_registry.rs` l'impose. ⚠️ Si le code d'action est choisi par
   une variable plutôt qu'écrit en littéral au site d'insertion, le site doit entrer à
   `SITES_INDIRECTS` — c'est ce que la 25-2-a a appris au merge (`accounts.rs`). ⚠️
   `repositories/invoices.rs` **y figure déjà** (`audit_label_registry.rs:111`, suspension de relance) :
   il faudrait alors **compléter** son entrée, pas en créer une seconde.
   ⚠️ **Le registre des routes mutantes** s'applique à la route neuve : `audit_route_registry.rs`
   (toute route mutante y est examinée, 25-1b) — ses totaux codés en dur passent de **105 à 106** et
   de **108 à 109** (`:448`, `:458`), et leurs mentions en prose (`:20`, `:24-25`, `:246`) avec eux.
   Montée dans `comptable_routes` (AC 9),
   elle n'entre pas au décompte d'`admin_pat_denied_e2e`.

7. **Les messages sont traduits** dans les quatre locales, convention `error-*`.

8. **Le verrou optimiste s'applique.** La dévalidation prend `version` et rend 409 en cas de conflit.

9. **L'écran.** Le bouton « supprimer » d'une facture validée est remplacé par « dévalider », qui
   explique ce qui va se passer : l'écriture comptable sera supprimée, la facture repassera en
   brouillon, et le numéro sera conservé. Les refus de l'AC 3 s'affichent en nommant leur motif.
   ✅ **Le rôle — arbitré par Guy le 2026-09-19 : Administrateur ET Comptable.** La route se monte
   dans `comptable_routes`, comme `POST /api/v1/invoices/{id}/validate` : qui peut valider peut
   dévalider. Le bouton passe donc de `isAdmin` (`frontend/src/routes/(app)/invoices/[id]/+page.svelte:729`)
   à la garde de rôle des autres actions comptables de la fiche. ⚠️ C'est un **élargissement** :
   jusqu'ici seul l'Administrateur pouvait faire disparaître l'écriture d'une facture.
   ✅ **Les clés API sont ADMISES, comme pour la validation — arbitré par Guy le 2026-09-19 :
   « même approche que Bexio ».** Bexio expose ce geste dans son API publique
   (`POST /2.0/kb_invoice/{invoice_id}/revert_issue`). La route suit donc le régime de
   `comptable_routes` sans garde supplémentaire. ⚠️ C'est un **élargissement**, et il s'écrit : jusqu'ici
   aucune clé ne pouvait détruire l'écriture d'une facture, la suppression vivant dans `admin_routes`.
   `docs/api-external.md` la range parmi les opérations ouvertes aux clés. Test : une clé `read-write`
   dévalide ; une clé `read` reçoit le refus d'écriture habituel.
   ⚠️ *« Même approche que Bexio » ne rouvre pas l'arbitrage `emailed_at` du 2026-09-16* (refus sec,
   AC 3) : il est explicite et postérieur à la citation de Bexio en tête de fiche, et la sémantique de
   Bexio sur une facture envoyée n'a pas été vérifiée — ses pages d'aide ne se chargeaient pas.
   ⚠️ **Deux résidus d'écran, relevés en passe 3.** La branche
   `invoice?.status === 'validated'` de la modale « retaper le numéro »
   (`+page.svelte:908-919,958`) devient **du code mort** : à retirer avec le bouton. Et la garde
   actuelle `isAdmin && !invoice.paidAt` (`:729`) **sous-couvre** le motif 1 : une facture
   *partiellement* réglée n'a pas de `paid_at`, l'écran la montrerait donc dévalidable avant que le
   serveur ne rende `409 INVOICE_HAS_SETTLEMENTS`. L'écran lit le résiduel, ou assume le refus
   serveur — au choix, mais écrit.
   **La friction** : une confirmation simple qui dit ce qui va se passer — l'écriture est supprimée,
   la facture repasse en brouillon, son numéro est conservé. Pas de numéro à retaper : la facture
   reste en place, et c'est la suppression du brouillon, ensuite, qui garde sa propre confirmation.

10. **Les manuels.** Le manuel utilisateur décrit le cycle et ses empêchements. Sites établis en
    passe 1 de validation : `user-manual.tex` §`sec:suppression-facture` (l. 1001-1022, **refonte**,
    pas une retouche), le renvoi de la l. 538 (« suppression définitive d'une facture validée, qui
    emporte son écriture ») et l'aiguillage « avoir ou suppression ? » de la l. 966 ;
    `admin-manual.tex` l. 1759 (liste des routes d'administration) et l. 1957, qui affirme **déjà
    faux** que la garde des rappels « est prévue » alors qu'elle existe. Établis en passe 2 :
    `user-manual.tex` l. 778 (« Brouillon : … pas de numéro définitif ») et l. 1003 (« aucun numéro
    définitif n'a encore été émis ») — **rendus faux** par l'AC 2 ; l. 834 (« strictement
    séquentielle dans un exercice ») — à relire contre l'AC 4 ; l. 502 (« une facture client par un
    avoir ») et l. 618 (« ne se modifie ni ne se supprime plus ») — rendus **incomplets**. Côté i18n
    (AC 7) : `journal-entries-reverse-blocked-invoice` (« corrigez-la par un avoir ») dans les
    quatre locales et son repli dans `crates/kesh-api/src/errors.rs` doivent nommer la dévalidation.
    ⚠️ Le PDF du **manuel d'administration** n'a encore été aplati par aucune passe.
    Relevés en passe 3 : `user-manual.tex:464-465` **énumère** les chemins que le verrou ferme
    (« ni la saisie manuelle, ni la validation… ») — la dévalidation y entre, plutôt que d'être
    seulement relue ; et `admin-manual.tex:1957` ne se corrige pas en remplaçant « prévue » par
    « existe » : **tout son paragraphe** repose sur la prémisse qu'un administrateur peut supprimer
    une facture qui a des rappels, fausse déjà sur `main` — il se réécrit.
    Relevés par la validation de la 25-2-b-zero : `admin-manual.tex:1799` et `README.md:218`
    affirment qu'une écriture enregistrée n'est « ni modifiable ni supprimable […] sans exception » —
    **déjà faux sur `main`** (la branche `validated` d'`invoices::delete` supprime l'écriture), et à
    réécrire ici : après cette story, la dévalidation supprime une écriture, délibérément. La promesse du verrou de
    période (`user-manual.tex:464`, « par aucun chemin ») redevient vraie avec cette story — la
    relire. PDF régénérés et **contrôlés à plat** (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`).

11. **`docs/api-external.md`** porte la route neuve et ses codes d'erreur, et sa note ² (l. 217), qui
    décrit `DELETE /invoices/{id}` comme une « suppression définitive », est rectifiée : la route ne
    supprime plus que des brouillons.

12. **#440 se ferme** — `closes #440` sur la **PR**, `refs` sur les commits intermédiaires, le dépôt
    mergeant en squash. ⚠️ **#219 ne se rouvre pas** : elle est fermée et le reste ; cette story
    remplace son chemin, elle ne revient pas sur sa décision. Et **#381 ne se ferme pas ici** — un
    `closes` posé par mégarde sur elle serait un mensonge que rien ne rattraperait après le merge.

## Tasks / Subtasks

- [x] **T0 — Traçage** — issue **[#440]** ouverte le 2026-09-16 sur `feature_request.yml`
      (labels `enhancement`, `triage`), sur demande explicite de Guy.
- [ ] **T1 — Dépôt : la dévalidation** (AC 1, 2, 3, 5, 8)
  - [ ] `invoices::unvalidate`, transaction unique : verrou `FOR UPDATE` de la facture, empêchements
        1-4 et 7, **un seul `UPDATE`** posant `status = 'draft'`, `journal_entry_id = NULL` et
        `version = version + 1`, puis `delete_in_tx` (motifs 5, 8, 6), puis audit.
        ⛔ Deux `UPDATE` dans l'ordre « `NULL` d'abord » violeraient
        `chk_invoices_validated_has_je` (`status <> 'validated' OR journal_entry_id IS NOT NULL`)
        dès le premier.
  - [ ] Retrait de la branche `validated` de `invoices::delete` ; une facture validée y est refusée
        par un message qui **oriente vers la dévalidation**, non par le générique
        `ILLEGAL_STATE_TRANSITION`. Les doc-comments qui nomment `invoices::delete` comme seul
        appelant à `false` (`journal_entries.rs:937-942`, `:976-981` ; `invoices.rs:1339-1349`) sont
        réécrits.
  - [ ] Un enum des empêchements sur le modèle de `ReversalBlocker`, un `code()` par motif, rendu au
        client — **pas** `IllegalStateTransition`.
  - [ ] ⛔ **Prérequis : 25-2-b-zero mergée** — elle pose la garde du verrou de période dans
        `delete_in_tx` (motif #6). Ici, seulement le test du motif 6 **par la dévalidation**.
  - [ ] Garde du règlement sur l'**existence** d'une ligne `invoice_settlements`, jamais `paid_at`.
  - [ ] Les cinq tests `test_delete_validated_*` réécrits contre `unvalidate` (AC 5).
  - [ ] Tests : un par empêchement de l'AC 3, la précédence, le chemin nominal, le conflit de version.
- [ ] **T2 — Dépôt : la revalidation reprend le numéro** (AC 2, 4)
  - [ ] `validate_invoice` ne tire du compteur **que** si `invoice_number` est absent.
  - [ ] ⛔ Test décisif : valider → dévalider → revalider → **le même numéro de facture**, et le
        compteur n'a pas bougé.
- [ ] **T3 — API, i18n, audit** (AC 1, 6, 7)
- [ ] **T4 — Frontend** (AC 9)
- [ ] **T5 — E2E** — les deux cycles complets, bout à bout.
- [ ] **T6 — Documentation** (AC 10, 11) + `CHANGELOG.md`.
- [ ] **T7 — Gates complets et PR.**

## Dev Notes

### Traçage — ⛔ à faire avant le développement

La dévalidation est une **fonctionnalité neuve**, pas un correctif : la règle de traçage du dépôt
impose une issue GitHub (`feature_request.yml`) **avant** le changement de scope. ✅ **Fait —
[#440]**, ouverte le 2026-09-16 sur demande de Guy, labels `enhancement` et `triage` posés par le
gabarit.

Les trois issues voisines, et pourquoi elles ne se confondent pas :

- **[#219]** — fermée. Elle décrit le chemin que cette story **remplace** ; elle ne se rouvre pas.
- **[#381]** — appartient à la **25-2-c**. Cette story ne la ferme pas.
- **[#440]** — celle-ci.

### Ce que cette story ne fait pas

Elle ne touche **pas** au mécanisme de numérotation des écritures : c'est la **25-2-c** (#381),
**mergée le 2026-09-19** (#442). Une écriture supprimée par la dévalidation laisse donc un **trou**
dans la séquence des écritures — visible, expliqué par le journal d'audit —, et son numéro n'est
jamais réattribué.

### Règle de splitting

Modules touchés : `kesh-db`, `kesh-api`, `kesh-i18n`, `frontend` — quatre, sous le seuil. Mais le
scope est **plus lourd que la 25-2-a** : une transition d'état neuve, huit empêchements, deux cycles
E2E. Si une passe de `validate` remonte une sévérité égale ou supérieure à la précédente, splitter
selon le critère de non-convergence.

### References

- [Source: crates/kesh-db/src/errors.rs:57-104] — `ReversalBlocker`, dont `OwnedByInvoice`.
- [Source: crates/kesh-db/src/repositories/invoices.rs:1205-1360] — `delete` et ses trois gardes.
- [Source: crates/kesh-db/src/repositories/invoices.rs:1550-1845] — `validate_invoice`.
- [Source: crates/kesh-db/migrations/20260417000002_invoice_validated_journal_entry_check.sql]
- [Source: https://github.com/guycorbaz/kesh/issues/219] — la suppression directe, fermée, que cette story remplace.
- [Source: CLAUDE.md § Issue Tracking Rule, § Review Iteration Rule]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Étape | Note |
|---|---|---|
| 2026-09-16 | spec | Story créée sur arbitrage de Guy, après que la contre-passation se soit révélée inapplicable (`OwnedByInvoice`). Deux cycles demandés : dévalider-effacer **et** dévalider-corriger-revalider. ⛔ Issue GitHub non ouverte. |
| 2026-09-16 | traçage | Issue **[#440]** ouverte sur demande de Guy ; la story passe de `draft` à `ready-for-dev`. Le dernier point ouvert est levé. |
| 2026-09-16 | arbitrage | `emailed_at` → **refus sec**, non levable : *« on ne doit pas pouvoir modifier une facture créée et envoyée à un client »* (Guy). Réserve écrite : la garde n'attrape que ce que Kesh sait avoir envoyé. ⚠️ Fait relevé au passage, **contre l'asymétrie supposée** : les factures **reçues** ne sont pas plus faciles à modifier — `supplier_invoices` n'a **aucune** fonction de mise à jour, et son `cancel` **contre-passe** l'écriture d'achat au lieu de la supprimer. Le côté fournisseur est donc plus strict, pas plus souple. |
| 2026-09-19 | validate P1 | **Passe 1, deux lentilles** (Sonnet, Haiku 4.5), prompt versionné `25-2-b-validate-prompt-p1.md` ; branche tirée de `main` au `44c6842f`, après les merges de #439, #441 et #442. Findings **retenus après vérification dans le code** : **1 CRITICAL** — « un code distinct par motif » était inatteignable en déplaçant les gardes, toutes en `IllegalStateTransition` au code unique ; le motif `ReversalBlocker` est désigné. **3 HIGH** — le décompte « quatre non gardés » était faux et contradictoire ; l'AC 6 était périmée (#439 mergée) ; le **rôle** du bouton n'était pas spécifié (ARBITRAGE ATTENDU). **2 MEDIUM** — cinq tests existants cassés non nommés ; les sites des manuels sous-estimés (dont `admin-manual.tex:1957`, déjà faux). ⛔ **Et deux trous RÉELS de `main`, trouvés en vérifiant les findings et vus par AUCUNE lentille** : le **verrou de période** n'est pas contrôlé à la suppression d'une écriture (seulement à sa création), si bien que supprimer une facture validée change aujourd'hui les totaux TVA d'un trimestre verrouillé ; et la garde « payée » lit `paid_at`, qui ignore un **règlement partiel** — dont les lignes partent en cascade. Tous deux entrent dans l'AC 3, qui passe de **sept à huit** empêchements. ⚠️ **Haiku a rendu 2 CRITICAL et 3 HIGH qui visaient le code non encore écrit** (« `validate_invoice` tire toujours du compteur », « aucune garde `emailed_at` ») : c'est le travail que la story prescrit, non un défaut de sa spec — écartés. Sites vérifiés un à un (`user-manual.tex` l. 538, 966, 1001-1022 ; `admin-manual.tex` l. 1759, 1957 ; `+page.svelte:729` et non 730). |
| 2026-09-19 | arbitrage | **Rôle — Guy : « les deux »**, Administrateur **et** Comptable ; route dans `comptable_routes`, symétrique de la validation. Deux conséquences écrites à l'AC 9 : c'est un **élargissement** (seul l'Administrateur supprimait une facture validée), et les **clés API sont refusées par défaut** (`ensure_not_pat`) — défaut posé par l'orchestrateur et signalé, faute de quoi le rôle les aurait ouvertes par effet de bord. Friction : confirmation simple, proposée à Guy et non contestée. |
| 2026-09-19 | arbitrage | **Clés API — Guy : « même approche que Bexio »** ; le défaut de refus posé plus haut est **levé**. Vérifié : l'API publique de Bexio porte `POST /2.0/kb_invoice/{id}/revert_issue`. La route est donc ouverte aux clés comme la validation, élargissement écrit à l'AC 9. L'arbitrage `emailed_at` n'est pas rouvert — la conduite de Bexio sur une facture envoyée n'a pas pu être vérifiée. |
| 2026-09-19 | validate P2 | **Passe 2, une lentille Opus** en contexte frais, prompt versionné `25-2-b-validate-prompt-p2.md`. **0 CRITICAL, 1 HIGH, 8 MEDIUM, 8 LOW** — sévérité maximale en baisse (CRITICAL → HIGH), pas de non-convergence au sens de la règle de découpage ; ⚠️ mais **trois MEDIUM portent sur les passages réécrits en passe 1** (AC 3, T1, AC 9) : le motif du dépôt se vérifie. Chaque point **vérifié dans le code** avant d'être retenu. Corrigés sans arbitrage : la garde du règlement lit `invoice_settlements` **OU** `paid_at` (données antérieures à la table) ; la précédence réelle **1-4, 7, puis 5, 8, 6**, la garde de période **après** le gel pour ne pas changer le contrat de `DELETE /journal-entries/{id}` ; **un seul `UPDATE`** (sinon `chk_invoices_validated_has_je`) ; les cinq codes neufs nommés ; tests et registres cassés inventoriés (`invoice_delete_e2e`, Playwright « fiche fantôme », 105→106 et 108→109) ; six sites de manuel et le message `journal-entries-reverse-blocked-invoice`. ⚠️ **Ma correction de passe 1 `:1350` → `:1351` était fausse** — reprise d'une lentille Haiku sans vérification ; rétablie. **Soumis à Guy** : H1 (revalidation dans un autre exercice), M5 (le Comptable dévalide mais ne supprime pas le brouillon), M6 (course avec l'envoi d'e-mail), M7 (trou dans la séquence des factures), L8 (garde de période en story zéro). |
| 2026-09-19 | arbitrages | Guy : « continue » — **les trois recommandations et les deux choix par défaut sont retenus**. (1) un brouillon numéroté **ne change pas d'exercice** (`PUT` et revalidation refusent, `INVOICE_NUMBER_FISCAL_YEAR_MISMATCH`) ; (2) **asymétrie des sorties** gardée — seul l'Administrateur efface, le Comptable dévalide et corrige ; (3) la garde du **verrou de période** sort en **story 25-2-b-zero**, livrée d'abord. Choix par défaut : le trou laissé par l'effacement d'un brouillon numéroté est dit et signalé à la confirmation ; la fenêtre de l'envoi d'e-mail se ferme en **marquant avant d'expédier**, repli en limite assumée. |
| 2026-09-20 | validate P3 | **Passe 3, une lentille Sonnet** (P2 était Opus), prompt versionné `25-2-b-validate-prompt-p3.md`, sept axes exercés. **2 HIGH, 1 MEDIUM, 2 LOW.** ⛔ **La sévérité maximale ne baisse plus** (P2 : 1 HIGH ; P3 : 2 HIGH) — le critère de non-convergence de la § *Règle de splitting préventif* est atteint, **signalé à Guy**. HIGH 1 — **un sixième test casse**, `delete_validated_in_locked_period_returns_400_period_locked`, que la **25-2-b-zero vient d'ajouter** : mon propre inventaire de passe 2 était déjà périmé le jour où je l'écrivais. HIGH 2 — **le renversement de l'envoi d'e-mail est écarté** : `mark_emailed` couple marque et audit dans une transaction, et un test existant exige qu'un échec SMTP n'écrive aucun audit ; pire, une panne entre marque et retrait rendrait la facture **indévalidable pour toujours**. Repli **déjà accepté par Guy** : la fenêtre devient une limite écrite. MEDIUM — la revalidation ne peut pas porter « la même garde » : aucun `fiscal_year_id` sur `invoices`, et un gabarit de numéro peut n'avoir aucun marqueur d'année ; l'état devient inatteignable une fois le `PUT` gardé. LOW — deux résidus d'écran, deux sites de manuel qui demandent une réécriture et non une relecture. |
| 2026-09-21 | découpage | **Guy tranche pour le découpage**, signalé à la passe 3. Deux filles : **25-2-b-1** (dépôt et API) et **25-2-b-2** (retrait de la suppression, écran, manuels), la seconde derrière la première. Cette fiche passe en `split` et devient la **référence des faits**, non un plan de travail. ⚠️ La 25-2-b-2 corrige d'emblée un décompte de cette fiche : **sept** sites de test cassés, non cinq. |
