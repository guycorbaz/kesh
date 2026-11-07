# Epic 33 — registre des choix autonomes, stories 33-1a et 33-1b (clé `e331`)

Rédigé le 2026-11-06, à la rédaction des fiches. Numéros de ligne de `4ef19901`. Se verse par union. Les décisions de Guy
(`epic-33-numerotation-et-defauts.md`, #926) ne se rouvrent pas ici : ces choix tranchent ce qu'elles laissent ouvert.

## C-e331-1 — Le compteur est une colonne de `fiscal_years`, non une table

- **Contexte.** La mécanique naturelle serait une table `supplier_invoice_number_sequences`, sœur de `credit_note_number_sequences`. Or
  `parse_and_verify` exige l'égalité exacte entre le manifeste et `TABLES_TO_TRUNCATE` (`admin_backup/import.rs:250-263`) : toute table neuve fait refuser
  les sauvegardes 0.17.4, alors que la consigne est qu'elles reçoivent les numéros ; elle referme aussi la fenêtre d'importabilité (`post_restore.rs:21-27`).
- **Retenu.** `fiscal_years.supplier_invoice_next_number BIGINT NOT NULL DEFAULT 1`. Aucun verrou neuf quand un seul exercice commence dans l'année (`C-e331-2`) : la ligne est déjà tenue `FOR UPDATE` à l'étape (3) de `validate_in_tx`. Le
  compteur se restaure avec l'exercice.
- **Écartées.** Table de séquence (ci-dessus). Import tolérant à une table absente : plus large, et c'est ce qu'a retiré la 32-12. `MAX(seq)+1` : réattribue le numéro d'un brouillon supprimé,
  à moins d'interdire la suppression. JSON de compteurs sur `company_invoice_settings` : sans clé étrangère ni contrainte.
- **Réversible.** Oui avant publication. Après, déplacer le compteur exigerait une migration.

## C-e331-2 — La concurrence d'attribution

- **Contexte.** Un numéro ne doit jamais être attribué deux fois, y compris entre deux exercices de la même année.
- **Retenu.** Réglages `FOR UPDATE` (2 bis), exercice `FOR UPDATE` (3). Lecture des compteurs et contrôle de collision **verrouillants** : l'instantané de la transaction est pris avant l'attente sur les réglages (`supplier_invoices.rs:1356`, `:1421`), une lecture ordinaire y lirait un compteur périmé, ce que prouve AC13 (g). `UNIQUE (company_id, internal_number)` en filet. Les autres exercices de l'année (`C-e331-8`) se lisent en deux temps : leurs `id` par une lecture ordinaire, puis leur compteur `FOR UPDATE` un par un, patron de `close` (`fiscal_years.rs:1067-1072`), une lecture par intervalle verrouillant selon le plan de l'optimiseur. **Aucun verrou neuf quand une seule ligne couvre l'année**, le cas courant.
- **Conséquences assumées.** La ligne des réglages ne sérialise qu'une fois l'exclusif obtenu : deux saisies peuvent s'interbloquer avant (`INSERT IGNORE` en doublon, doc-comment de `validate_in_tx`), et la route rejoue. Avec deux exercices la même année, une clôture ou une réouverture qui croise une validation peut interbloquer ; les trois routes sont rejouées (`routes/supplier_invoices.rs:870`, `routes/fiscal_years.rs:343`, et la réouverture, mise au rejeu ici : `routes/fiscal_years.rs:398`, 33-1a AC12). Le contrôle de collision pose un verrou d'intervalle qui peut faire attendre la création d'un brouillon, sans interblocage. La liste des `id` vient de l'instantané : un exercice créé **et** porteur d'un numéro émis **et** supprimé pendant l'attente du verrou des réglages échapperait au calcul, ce qui n'a pas de chemin réaliste à un utilisateur (`C-e331-23`). **Le renommage n'est pas rejoué** (revue P1, B-L2, E-L1, A-L7) : il tient l'exercice puis son homonyme, et croise de même une validation quand deux exercices commencent la même année ; au pire, un 500 sur un renommage que le pré-contrôle aurait refusé, sans donnée touchée. Candidat déjà nommé à `audit_route_registry.rs`, issue demandée.
- **Écartées.** Verrou applicatif ; capture de l'erreur d'unicité avec nouvel essai dans la transaction.
- **Réversible.** Oui.

## C-e331-3 — L'exercice dont le nom n'est pas une année

- **Retenu.** `{YEAR}` = année de `start_date`, comme `credit_notes.rs:559-572`. Exercice 2025/2026 commencé en juillet 2025 : `FF-2025-001`. Le nom de l'exercice n'entre pas dans le numéro (`C-e331-8`).
- **Écartée.** Année de fin, ou année de la date de facture : divergerait des deux autres familles. Une même séquence de l'exercice porterait alors deux années.
- **Réversible.** Oui pour les numéros à venir ; les numéros attribués ne se réécrivent pas.

## C-e331-4 — Ce que veut dire « configurable » pour la séquence

- **Contexte.** Guy : « dans l'idéal configurable ».
- **Retenu.** Le format réglable : préfixe, séparateur, largeur par `{SEQ:NN}`. Rien d'autre.
- **Écartées.** Valeur de départ du compteur ; choix du mode de remise à zéro. Personne ne les demande ; un compteur modifiable à la main permet de réattribuer un numéro.
- **Réversible.** Oui, un réglage s'ajoute plus tard. À confirmer à la recette.

## C-e331-5 — Le brouillon numéroté qu'on supprime

- **Retenu.** Suppression permise ; le numéro se perd, le trou est accepté, le compteur ne redescend pas et aucun exercice de la même année ne reprend le numéro (`C-e331-8`), hors les trois cas de `C-e331-23` ; l'audit garde l'instantané ; l'écran avertit, le manuel dit quoi faire du papier.
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

## C-e331-8 — La séquence appartient à l'année de début

- **Contexte.** Deux exercices commençant la même année civile rendent `FF-2026-001` chacun, et rien ne les interdit (`initial_schema.sql:49-51`). Un échec d'`UNIQUE` **à la migration** empêcherait la production de démarrer. La validation a montré, en deux passes, trois chemins par lesquels un numéro supprimé revenait, tous dus à `{FY}` et à la portée qui en dépendait : le passage à `{FY}`, le renommage, `{FY}{SEQ}` collés. Arbitrage de l'orchestrateur, proposé à Guy : retirer `{FY}` du format fournisseur et la portée.
- **Retenu.** Migration : rang par société et **année de début**, ces exercices se suivent. Validation : **ces exercices partagent leur séquence, quels que soient le format et le nom** : départ au plus grand compteur de l'année, tous statuts ; le saut d'un numéro pris reste un filet pour une donnée posée hors de l'application, vers le haut seulement, borné, parce que sans lui une telle donnée bloquerait toute validation sans issue à l'écran. Seule entorse à « repart à 1 », écrite au manuel.
- **Écartées.** Faire échouer la validation : bloque sans issue claire. Saut au premier libre : réémet un numéro supprimé. Écrire le compteur de tous les exercices de l'année : une écriture de plus pour un résultat que le plus grand donne déjà. `{FY}` avec une séquence propre à chaque exercice : voir le contexte.
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

## C-e331-16 — Une forme fixe pour le seul format fournisseur

- **Établi.** `validate_template` accepte `F-{YEAR}` (`invoice_format.rs:151-199`). Avec un saut de collision, un format sans `{SEQ}` ne se terminerait pas. Sans `{YEAR}`, `FF-{SEQ:03}` repartirait à 1 à chaque année sur les numéros de l'année précédente : le saut les parcourrait un à un et réémettrait tout trou. Un chiffre libre à côté du compteur fait coïncider deux séquences : `FF-{YEAR}-1{SEQ}` émet `FF-2026-13` au rang 3, et, ce rang supprimé, `FF-{YEAR}-{SEQ}` le réémet au rang 13.
- **Retenu.** `validate_numbering_template` pour le format fournisseur : **`<préfixe>{YEAR}<séparateur>{SEQ}`** ou `{SEQ:NN}`, séparateur non vide et sans chiffre, rien après le compteur, aucun `{FY}`. C'est ce qui rend un numéro **injectif**, l'année s'écrivant sur quatre chiffres (de 1000 à 9999, borne que pose la 33-4 ; `render` ne remplit pas) : le rang est la suite de chiffres finale, le séparateur ce qui la précède sans chiffre, l'année les quatre chiffres d'avant ; deux formats ne peuvent donc rendre la même chaîne que pour la même année et le même rang, et le rang d'une année ne se répète pas. Le texte de la décision (« `{YEAR}`, `{SEQ}` et un préfixe libre ») n'en disait pas tant : **le séparateur sans chiffre et l'interdiction d'un suffixe sont un choix de cette remédiation, signalé au Project Lead.** Formats client et avoir inchangés, **issue demandée** : le formulaire n'envoyant pas le format d'avoir (`routes/company_invoice_settings.rs:295-308`), le durcir bloquerait tout enregistrement d'une société dont le format d'avoir stocké n'a pas de `{SEQ}`, sans moyen de le corriger à l'écran. Déborde donc d'un correctif court. **Limites écrites à la revue P2 (P2-L2, 2026-11-07).** Depuis la remédiation P1, un blanc de tête et toute tabulation sont refusés aussi, mais le message `NotNumberingShape` et la clé `invoices-format-error-numbering` annoncent toujours « un préfixe libre » sans le dire : qui saisit ` FF-{YEAR}-{SEQ}` reçoit un refus juste et mal expliqué. Et `ALLOWED_CHAR_RE` (`invoice-number-format.ts:14`) admet `\s`, donc un saut de ligne, que le serveur refuse : préexistant, tous formats, l'écran dit « valide » et le serveur répond 422. Aucun format enregistré ne devient invalide : la colonne est neuve, son défaut conforme.
- **Réversible.** Oui.

## C-e331-17 — Retiré le 2026-11-06 (le rendu impossible)

- **Motif.** Sans `{FY}`, un format validé se rend toujours (`invoice_format.rs:151-199`), et le nom d'exercice n'est plus passé à `render` (33-1a, AC8). La variante d'erreur et sa clé n'ont plus d'objet ; un format posé hors de l'application rend `Invariant`. Reste à l'issue de `C-e331-16` : les sites client et avoir, où `{FY}` demeure, rendent `Invariant`, donc 500, sur un nom d'exercice trop long ou à accolade.

## C-e331-18 — Dérogation à la règle de splitting

- **Contexte.** 33-1a touche neuf modules, 33-1b plus encore.
- **Retenu.** Garder les deux stories de l'epic. 33-1a est un seul invariant ; 33-1b est le déploiement mécanique de son patron, à revoir fichier par fichier. Signalé au Project Lead par le rapport.
- **Écartée.** Sous-stories : l'epic n'en prévoit pas, et une colonne sans son attribution ne se livre pas seule.
- **Réversible.** Oui : si la validation lève une non-convergence par recyclage, la règle s'applique.

## C-e331-19 — Tri et recherche côté serveur

- **Établi.** `NATURAL_SORT_KEY` apparaît en MariaDB 10.7 (documentation de MariaDB, non exécuté ici). La seule version livrée, testée et en production est 10.11 (`docker-compose.yml:4`, `ci.yml:50`, `docs/ci.md:269-281`) ; `README.md:60`, `docs/ci.md` et la brochure promettent pourtant « compat ≥ 10.6 », et `admin-manual.tex:109` une compatibilité MySQL 8, déjà fausse puisque des migrations emploient `ADD COLUMN IF NOT EXISTS` (de connaissance pour MySQL, non exécuté).
- **Retenu.** `q`, `sort`, `order` sur la route (la liste est paginée : un tri côté écran ne verrait que la page) ; liste blanche ; `NATURAL_SORT_KEY`, **plancher 10.7 assumé**, promesses corrigées ; `q` couvre aussi le numéro du fournisseur et son nom.
- **Écartées.** Colonne `internal_number_seq` pour trier : une colonne pour un seul usage. Tri textuel : `FF-2026-1000` avant `FF-2026-999`. Tri par longueur puis texte : indépendant de la version, mais `FF-2025-1000` passerait après `FF-2026-001`.
- **Réversible.** Oui.

## C-e331-20 — Le sort de chaque écran

- **Retenu.** Forme courte dans une phrase ; forme longue pour reconnaître la pièce ; numéro du fournisseur pour comparer à une donnée externe. L'inventaire, établi depuis le symptôme, est à la fiche 33-1b, où chaque site a son choix : postes ouverts et lettrage en forme courte (la ligne porte déjà le libellé, à la forme longue pour les écritures neuves). Champs existants inchangés, `internalNumber` ajouté à côté (`number` de `DocumentResponse`, `numero` du MCP compris) : un client d'API ne voit pas changer la valeur d'un champ. **Deux exceptions dites** : `details.documentNumber` d'un refus, qui suit son message, et `documentNumber` de l'audit (`C-e331-25`).
- **Écartée.** Remplacer la valeur de `supplierInvoiceNumber` : son nom mentirait.
- **Réversible.** Oui.

## C-e331-21 — Le rapprochement ne compare pas le numéro interne

- **Retenu.** `matching.rs:99` inchangé : un numéro attribué par Kesh ne figure jamais sur un relevé. La règle d'appariement du `CLAUDE.md` ne bouge pas.
- **Réversible.** Oui.

## C-e331-22 — MCP

- **Retenu.** `numero_interne` s'ajoute, `numero` garde le numéro du fournisseur ; en `TexteNonFiable`, le format étant du texte d'administrateur. Aucune écriture.
- **Réversible.** Oui, un champ ajouté se retire avant publication.

## C-e331-23 — Ce qui peut encore faire réémettre un numéro

- **La preuve.** Tant qu'on reste dans l'application, et pour une année de 1000 à 9999 (33-4), une chaîne ne se rend que pour une seule année et un seul rang (`C-e331-16`), et le rang d'une année ne se répète pas : le départ est le plus grand compteur de l'année, le compteur de l'exercice passe au-dessus, aucun compteur ne redescend, aucun exercice ne quitte son année (`start_date` immuable) ni ne disparaît hors réinitialisation. Le nom et le format n'y jouent aucun rôle. Migration et rejeu posent des compteurs au-dessus de leurs rangs (33-1a, AC3).
- **Fermé, testé (33-1a, AC13, AC16).** Deux exercices de même année civile ; brouillon numéroté supprimé, y compris dans un exercice clos ou créé après coup ; format changé puis repris ; renommage ; `{FY}`, format sans année, chiffre à côté du compteur, refusés. *(Annoté le 2026-11-06, revue P1, A-M1 : « créé après coup » n'était pas prouvé en propre jusque-là, le test (c) passant par le filet du saut, comme (d) avant `bfe6eedb`. Renforcé à la remédiation, M19 le tue.)* *(Note du 2026-11-07, P2-L7 : cette remédiation a traversé minuit, code du 2026-11-06 à 21:29, gates du 2026-11-07. L'annotation ci-dessus et les Change Logs, datés du 2026-11-07, désignent le même acte.)*
- **Reste ouvert, limites nommées, les seuls chemins d'un utilisateur.**
  1. **Restauration d'une sauvegarde** : les compteurs reviennent à sa date, comme pour toute numérotation de Kesh ; un numéro attribué après elle peut être réattribué. Le manuel le dit.
  2. **Retour à la 0.17.4 suivi d'une nouvelle montée** : la numérotation repart de l'ordre des dates ; une facture de date antérieure saisie entre-temps décale les suivantes, et le format réglé revient au défaut. Le manuel le dit.
  3. **Réinitialisation des données** : tout repart de 1, papier compris à la charge de qui l'ordonne. Le manuel le dit.
- **Hors de ces cas, pour mémoire, sans chemin réaliste à un utilisateur.** La fenêtre d'instantané de `C-e331-2` : il faudrait créer un exercice, y valider une facture et la supprimer pendant qu'une validation attend, soit quelques millisecondes. Une écriture en SQL direct, ou une sauvegarde retouchée à la main, échappent par nature à la preuve.
- **Écartée.** Mémoriser les numéros supprimés (table : interdite par `C-e331-1` ; lecture de l'audit : sans index, et une sauvegarde partielle l'en priverait).
- **Réversible.** Oui, une table de numéros retirés peut s'ajouter à une version qui accepte de rompre l'import des sauvegardes 0.17.4.

## C-e331-24 — La numérotation de l'existant n'a pas d'entrée d'audit

- **Établi.** `audit_log.user_id` est `NOT NULL` (`20260413000001_audit_log.sql:15` ; sa clé étrangère a été retirée par `20260910000001_audit_log_actor_label.sql:48`) : une migration n'a pas d'auteur, donc pas d'entrée. À l'import, le rejeu en a un : l'entrée `admin.full_import`, signée `MIN(admin)` du jeu restauré, porte la liste des reprises et leur nombre de lignes (`admin.rs:526-540,639`).
- **Retenu.** Pas d'entrée propre à la montée : limite acceptée, écrite au manuel d'administration et au CHANGELOG, avec l'ordre appliqué. Même règle que le lettrage posé par une mise à jour (`user-manual.tex:1390`). Les numéros eux-mêmes figurent à l'instantané des gestes suivants.
- **Écartée.** Attribuer l'entrée au premier administrateur : un auteur inventé.
- **Réversible.** Non pour les numéros déjà posés.

## C-e331-25 — L'audit du lettrage : `documentNumber` passe au numéro interne (à relire par Guy)

- **Contexte.** `documentNumber`, dans les `details` de `lettering.created` et `lettering.removed`, vient de `DocumentRef.number` (`letterings.rs:1035-1076`, `:1882`, `:1983`) : le numéro du fournisseur. La lentille F demandait une décision. Guy veut que le numéro serve au lien avec le papier.
- **Retenu**, sur arbitrage de l'orchestrateur. Les nouvelles entrées d'une facture fournisseur portent le numéro interne, à la **forme longue** du libellé (`FF-2026-001 (ABC-123)`), ce qui garde le numéro du fournisseur ; les entrées passées ne se réécrivent pas ; le manuel le dit. Le sens du champ change donc selon la date, ce qu'une lecture de l'audit doit savoir.
- **Écartées.** Champ `documentInternalNumber` à côté : garde au champ un sens stable, mais laisse `documentNumber` nommer un numéro qui n'identifie pas la pièce. Inchangé : l'audit ne dirait pas le numéro du papier.
- **Réversible.** Oui pour les entrées à venir ; celles écrites entre-temps restent. **À relire par Guy.**

## C-e331-26 — Le détail d'un lot dit le numéro de ses factures

- **Contexte.** `payment-batches/[id]/+page.svelte:137-138` nomme la facture par `#id`, le DTO n'ayant aucun numéro ; `+page.svelte:225` de même pour les échecs.
- **Retenu.** `internalNumber` et `supplierInvoiceNumber` ajoutés à `PaymentBatchItemResponse`, par une seule lecture dédiée, scopée société, réemployée par `from_parts` ; le détail en forme courte ; les échecs depuis `openInvoices`, déjà chargée. Limite courte voisine, corrigée ici selon la règle de l'epic. **Depuis la remédiation P1, `from_parts` ne rend plus d'erreur** : sur un échec de cette lecture, elle écrit un `tracing::warn!` (`batch_id`, erreur) et rend les numéros absents, l'écran disant `#id`, pour qu'une opération enregistrée ne soit pas annoncée en échec. **Ce masquage vaut aussi pour le `GET /payment-batches/{id}`** (P2-L4, 2026-11-07), qui ne suit aucun commit et où un 500 aurait été aussi honnête : même `warn`, seul le journal du serveur en garde la trace. Accepté, une seule définition valant mieux ici qu'un cas par route.
- **Écartées.** Ajouter les colonnes à `PaymentBatchItem` : cinq lectures, et l'entité sert aussi l'export. Une issue : la correction tient en une lecture.
- **Réversible.** Oui.

## C-e331-27 — La liste MCP des factures fournisseur perd quelques lignes par page

- **Établi au développement.** `numero_interne` (AC18, `C-e331-22`) porte l'élément de `kesh_factures_fournisseur` à 372 octets : à 40 par défaut et 70 au maximum, les plafonds mesurés (13 312 et 23 552 octets, `mcp_outils_e2e::tailles_mesurees`) sont franchis, et l'outil refusait la page (« le résultat dépasse ce qu'un envoi peut porter »).
- **Retenu.** 62 au maximum, 34 par défaut, révisés sur la mesure comme la balance âgée (`C-32-9-44`). `limit` à 63-70 devient un refus de paramètre : dit au CHANGELOG. **Ces bornes valent pour un numéro au format par défaut** (revue P1, B-L8, E-L5) : le format admet des préfixes plus longs, et un numéro plus long de quelques caractères peut faire refuser la page la plus large. Le refus est propre, un `limit` plus bas passe ; le manuel le dit.
- **Écartée.** Omettre le numéro à la liste : la fiche demande les deux (AC18). Remonter les plafonds : ils bornent la taille d'un envoi, ils ne se règlent pas ici.
- **Réversible.** Oui.

## C-e331-28 — Deux champs d'erreur changent de nom, jamais de sens

- **Établi au développement.** `ModificationGuard::DetachedSupplierSettlement` et `SettlementBatchContext` portaient un champ `supplier_invoice_number` qui, avec la 33-1b, contient la forme courte (le numéro interne d'abord).
- **Retenu.** Renommés `supplier_invoice_label` : le nom ne ment pas, et ces deux types sont internes (l'API dit `documentNumber` et le message).
- **Écartée.** Garder le nom : c'est le défaut que `C-e331-20` refuse pour `supplierInvoiceNumber`.
- **Réversible.** Oui.

## C-e331-29 — Les montages partiels posent les colonnes le temps du semis

- **Établi au développement.** `draft_migration_preserves_a_populated_base` (`migrations_upgrade_path.rs`) et `lettering_documents_backfill.rs` montent une base antérieure, puis la sèment par le dépôt courant, qui lit désormais `internal_number`, `supplier_internal_number_format` et, pour la validation, `supplier_invoice_next_number`.
- **Retenu.** Le patron déjà en place dans `lettering_documents_backfill.rs` (`colonne_posterieure`, Story 28-4a) : les colonnes sont posées le temps du semis et retirées avant la montée, que la migration les recrée. La migration 33-1a elle-même se teste sur un semis en SQL brut (AC5).
- **Écartée.** Récrire ces semis en SQL : hors sujet, et ces tests prouvent autre chose.
- **Réversible.** Oui.
