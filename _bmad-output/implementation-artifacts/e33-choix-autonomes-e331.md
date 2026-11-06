# Epic 33 — registre des choix autonomes, stories 33-1a et 33-1b (clé `e331`)

Rédigé le 2026-11-06, à la rédaction des fiches. Numéros de ligne de `4ef19901`. Se verse par union. Les décisions de Guy
(`epic-33-numerotation-et-defauts.md`, #926) ne se rouvrent pas ici : ces choix tranchent ce qu'elles laissent ouvert.

## C-e331-1 — Le compteur est une colonne de `fiscal_years`, non une table

- **Contexte.** La mécanique naturelle serait une table `supplier_invoice_number_sequences`, sœur de `credit_note_number_sequences`. Or
  `parse_and_verify` exige l'égalité exacte entre le manifeste et `TABLES_TO_TRUNCATE` (`admin_backup/import.rs:250-263`) : toute table neuve fait refuser
  les sauvegardes 0.17.4, alors que la consigne est qu'elles reçoivent les numéros ; elle referme aussi la fenêtre d'importabilité (`post_restore.rs:21-27`).
- **Retenu.** `fiscal_years.supplier_invoice_next_number BIGINT NOT NULL DEFAULT 1`. Aucun verrou neuf : la ligne est déjà tenue `FOR UPDATE` à l'étape (3) de `validate_in_tx`. Le
  compteur se restaure avec l'exercice.
- **Écartées.** Table de séquence (ci-dessus). Import tolérant à une table absente : plus large, et c'est ce qu'a retiré la 32-12. `MAX(seq)+1` : réattribue le numéro d'un brouillon supprimé,
  à moins d'interdire la suppression. JSON de compteurs sur `company_invoice_settings` : sans clé étrangère ni contrainte.
- **Réversible.** Oui avant publication. Après, déplacer le compteur exigerait une migration.

## C-e331-2 — La concurrence d'attribution

- **Contexte.** Un numéro ne doit jamais être attribué deux fois, y compris entre deux exercices rendant la même chaîne.
- **Retenu.** Rien de neuf : réglages `FOR UPDATE` (2 bis), exercice `FOR UPDATE` (3). Lecture du compteur et contrôle de collision **verrouillants** (une lecture ordinaire lirait l'instantané
  d'avant le verrou). `UNIQUE (company_id, internal_number)` en filet. L'interblocage reste couvert par le rejeu (`routes/supplier_invoices.rs:870`).
- **Écartées.** Verrou applicatif ; capture de l'erreur d'unicité avec nouvel essai dans la transaction.
- **Réversible.** Oui.

## C-e331-3 — L'exercice dont le nom n'est pas une année

- **Retenu.** `{YEAR}` = année de `start_date`, comme `credit_notes.rs:559-572`. Exercice 2025/2026 commencé en juillet 2025 : `FF-2025-001`. `{FY}` donne le nom.
- **Écartée.** Année de fin, ou année de la date de facture : divergerait des deux autres familles. Une même séquence de l'exercice porterait alors deux années.
- **Réversible.** Oui pour les numéros à venir ; les numéros attribués ne se réécrivent pas.

## C-e331-4 — Ce que veut dire « configurable » pour la séquence

- **Contexte.** Guy : « dans l'idéal configurable ».
- **Retenu.** Le format réglable : préfixe, année, largeur par `{SEQ:NN}`. Rien d'autre.
- **Écartées.** Valeur de départ du compteur ; choix du mode de remise à zéro. Personne ne les demande ; un compteur modifiable à la main permet de réattribuer un numéro.
- **Réversible.** Oui, un réglage s'ajoute plus tard. À confirmer à la recette.

## C-e331-5 — Le brouillon numéroté qu'on supprime

- **Retenu.** Suppression permise ; le numéro se perd, le trou est accepté, jamais de réattribution ; l'audit garde l'instantané ; l'écran avertit, le manuel dit quoi faire du papier.
- **Écartée.** Interdire la suppression d'un brouillon numéroté : forcerait à revalider puis annuler pour se débarrasser d'une saisie erronée, au prix de deux écritures au journal.
- **Réversible.** Oui.

## C-e331-6 — Les factures hors de tout exercice

- **Établi.** Une facture non brouillon a toujours une écriture, et donc un exercice (`chk_supplier_invoices_entry_iff_not_draft`, `fiscal_year_id NOT NULL`, clés `RESTRICT`). Un brouillon peut être daté hors exercice : la validation le refuse déjà.
- **Retenu.** Migration par l'exercice **de l'écriture**, non par la date ; aucun cas résiduel. Aucun numéro avant une validation réussie.
- **Écartée.** Jointure par la date : inutilement fragile.
- **Réversible.** Oui.

## C-e331-7 — Le format changé en cours d'exercice

- **Retenu.** Vaut pour les numéros à venir, le compteur continue ; aucun numéro attribué n'est réécrit ; l'aide de l'écran le dit.
- **Écartées.** Interdire le changement une fois un numéro attribué : empêche de corriger un format malheureux. Renuméroter : invalide le classement papier.
- **Réversible.** Oui.

## C-e331-8 — Le numéro déjà pris

- **Contexte.** Deux exercices commençant la même année civile rendent `FF-2026-001` chacun, et rien ne les interdit (`initial_schema.sql:49-51`). Un échec d'`UNIQUE` **à la migration** empêcherait la production de démarrer.
- **Retenu.** Migration : rang par société et **année de début**, ces exercices se suivent. Validation : saut au premier numéro libre, borné. Seule entorse à « repart à 1 », écrite au manuel.
- **Écartées.** Exiger `{FY}` dans ces cas : refus d'écran obscur. Faire échouer la validation : bloque sans issue claire.
- **Réversible.** Non pour les numéros attribués.

## C-e331-9 — Le brouillon numéroté redaté dans un autre exercice

- **Retenu.** Il garde son numéro, quel que soit l'exercice de revalidation ; le compteur de l'exercice d'arrivée ne bouge pas. C'est la décision de Guy (« garde le sien »), et le papier est déjà classé. Le manuel dit l'écart possible entre l'année du numéro et celle de l'exercice.
- **Écartée.** Numéro neuf à la revalidation dans un autre exercice : le papier mentirait.
- **Réversible.** Oui pour la suite.

## C-e331-10 — La forme du libellé

- **Contexte.** Guy : les nouvelles écritures portent le numéro. Il dit aussi que le numéro du fournisseur sert aux recherches.
- **Retenu.** `Facture fournisseur FF-2026-001 (ABC-123) - Dupont SA` ; parenthèse absente si le numéro du fournisseur l'est.
- **Écartée.** Remplacer sans le garder : la recherche au journal par numéro du fournisseur disparaîtrait pour les écritures neuves.
- **Réversible.** Oui pour les écritures à venir. À confirmer à la recette.

## C-e331-11 — Pas de `CHECK` « non brouillon ⇒ numéro »

- **Retenu.** Colonne nullable sans contrainte croisée ; contrôle par test.
- **Écartée.** Le `CHECK` : l'import d'une sauvegarde 0.17.4 restaure les lignes avant le rejeu et échouerait ; toute fixture SQL aussi.
- **Réversible.** Par une migration ultérieure ; sans objet tant que les sauvegardes 0.17.4 s'importent.

## C-e331-12 — Le backfill préserve `updated_at` et `version`

- **Retenu.** `SET updated_at = updated_at`, `version` intacte, sur `supplier_invoices` et `fiscal_years`. La numérotation n'est pas une modification de la pièce, et une version intacte n'invalide pas un écran resté ouvert.
- **Réversible.** Sans objet.

## C-e331-13 — P7, classe B

- **Retenu.** Sentinelle `supplier_invoices.internal_number`, ajoutée dans le même fichier que le backfill. Rejeu seulement si la sauvegarde précède la migration. Premier occupant du registre depuis la 32-5a.
- **Écartée.** Classe A : les statements sont gardés, mais rejouer à chaque import sur une sauvegarde 0.17.5 n'apporte rien ; elle y lirait des rangs sans objet.
- **Réversible.** Oui.

## C-e331-14 — Non breaking, pas de bump

- **Établi.** `ADD COLUMN` nullable ou avec défaut, `ADD UNIQUE` ; P3 ne vise pas ces opérations. `kesh_version_min_required` reste à `0.17.0`. Le refus de démarrer de la 0.17.4 vient de `sqlx`, non du seuil.
- **Conséquence.** Retour uniquement par la sauvegarde d'avant la mise à jour, écrit au manuel et répété (AC22).

## C-e331-15 — Les noms

- **Retenu.** `internal_number` (`internalNumber`), `supplier_internal_number_format` (`supplierInternalNumberFormat`), `supplier_invoice_next_number`.
- **Écartée.** `supplier_invoice_number_format` : se lit comme le format de `supplier_invoice_number`, le numéro **du fournisseur**.

## C-e331-16 — `{SEQ}` exigé pour le seul format fournisseur

- **Établi.** `validate_template` accepte `F-{YEAR}` (`invoice_format.rs:151-199`). Avec un saut de collision, un format sans `{SEQ}` ne se terminerait pas.
- **Retenu.** `validate_numbering_template` pour le format fournisseur. Formats client et avoir inchangés, **issue demandée** : le formulaire n'envoyant pas le format d'avoir (`routes/company_invoice_settings.rs:295-308`), le durcir bloquerait tout enregistrement d'une société dont le format d'avoir stocké n'a pas de `{SEQ}`, sans moyen de le corriger à l'écran. Déborde donc d'un correctif court.
- **Réversible.** Oui.

## C-e331-17 — Le rendu impossible à la validation

- **Contexte.** `validate_template` borne `{FY}` à 16 caractères, le nom d'exercice en permet 50 (`invoice_format.rs:171`, `initial_schema.sql:42`). Le rendu peut échouer en `RenderedTooLong`.
- **Retenu.** `DbError::ConfigurationRequired` : 400, message clair. Les sites client et avoir rendent `Invariant`, donc 500 : ils ne changent pas ici, l'issue de C-e331-16 les mentionne.
- **Réversible.** Oui.

## C-e331-18 — Dérogation à la règle de splitting

- **Contexte.** 33-1a touche huit modules, 33-1b plus encore.
- **Retenu.** Garder les deux stories de l'epic. 33-1a est un seul invariant ; 33-1b est le déploiement mécanique de son patron, à revoir fichier par fichier. Signalé au Project Lead par le rapport.
- **Écartée.** Sous-stories : l'epic n'en prévoit pas, et une colonne sans son attribution ne se livre pas seule.
- **Réversible.** Oui : si la validation lève une non-convergence par recyclage, la règle s'applique.

## C-e331-19 — Tri et recherche côté serveur

- **Retenu.** `q`, `sort`, `order` sur la route (la liste est paginée : un tri côté écran ne verrait que la page) ; liste blanche ; `NATURAL_SORT_KEY` (MariaDB ≥ 10.7) ; `q` couvre aussi le numéro du fournisseur et son nom.
- **Écartées.** Colonne `internal_number_seq` pour trier : une colonne pour un seul usage. Tri textuel : `FF-2026-1000` avant `FF-2026-999`.
- **Réversible.** Oui.

## C-e331-20 — Le sort de chaque écran

- **Retenu.** Forme courte dans une phrase ; forme longue pour reconnaître la pièce ; numéro du fournisseur pour comparer à une donnée externe. L'inventaire est à la fiche 33-1b. Champs existants inchangés, `internalNumber` ajouté à côté : un client d'API ne voit pas changer la valeur d'un champ.
- **Écartée.** Remplacer la valeur de `supplierInvoiceNumber` : son nom mentirait.
- **Réversible.** Oui.

## C-e331-21 — Le rapprochement ne compare pas le numéro interne

- **Retenu.** `matching.rs:99` inchangé : un numéro attribué par Kesh ne figure jamais sur un relevé. La règle d'appariement du `CLAUDE.md` ne bouge pas.
- **Réversible.** Oui.

## C-e331-22 — MCP

- **Retenu.** `numero_interne` s'ajoute, `numero` garde le numéro du fournisseur ; en `TexteNonFiable`, le format étant du texte d'administrateur. Aucune écriture.
- **Réversible.** Oui, un champ ajouté se retire avant publication.
