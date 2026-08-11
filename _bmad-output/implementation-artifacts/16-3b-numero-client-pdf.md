# Story 16.3b : Numéro de client sur le PDF de facture

## Status

in-progress

## Story

**As a** indépendant ou fiduciaire qui facture des clients récurrents,
**I want** que le **numéro de client** que j'attribue à un contact figure sur le PDF, dans le bloc des métadonnées,
**so that** mon client puisse rapprocher la facture de son propre dossier fournisseur, et que je puisse moi-même retrouver un contact depuis une facture papier.

Issue : **#151** (moitié « n° client »). Sous-story de l'Epic 16 « Facturation avancée », cible **v0.9.0**. **Ferme #151.**

## Contexte

L'issue #151 relève trois manques sur le PDF de facture. Deux sont livrés :

| Volet | État |
|---|---|
| Récapitulatif TVA par taux | ✅ PR #267, 2026-07-21 |
| Coordonnées de l'émetteur (téléphone, e-mail, site web) | ✅ Story 16-3a, PR #290, 2026-08-10 |
| **Numéro de client / référence client** | ⬜ **cette story** |

**Le champ n'existe nulle part.** Relevé sur le dépôt au 2026-08-10 :

```sh
grep -rn "client_number\|customer_number\|numero_client\|reference_client" crates/ frontend/src
# → aucune occurrence
```

Il faut donc le créer de bout en bout : colonne, entité, repository, DTO, écran, PDF, i18n. C'est le **même parcours que 16-3a**, mais sur `contacts` au lieu de `companies`, et dans le bloc **droit** (métadonnées) au lieu du bloc gauche (émetteur).

**Ce que 16-3a rend plus facile, et qu'il faut réutiliser** : le patron conditionnel de rendu (`if let Some(...)` — pas de ligne dessinée, pas de descente du curseur si le champ est absent), l'invariant `I18N_KEYS` ↔ `DEFAULT_EN` tenu à la compilation, et la discipline de propagation aux 4 locales.

## Décisions

Ces cinq décisions sont **prises** et n'ont pas à être re-délibérées à l'implémentation. Si l'une doit changer, c'est un CR.

### D1 — Le numéro est **saisi**, pas auto-généré

Colonne `client_number VARCHAR(50) NULL` sur `contacts`, remplie à la main depuis la fiche contact.

*Pourquoi* : l'issue demande « un numéro de client / référence client », pas un système de numérotation. Auto-générer imposerait de trancher un format, une séquence, une politique de reprise pour le parc existant et un comportement en cas de collision — c'est un sujet en soi, et **aucun de ces choix n'est demandé**. `VARCHAR(50)` est aligné sur `contacts.phone` (50) et accueille aussi bien `42` que `CLI-2026-00042` ou une référence imposée par le client.

### D2 — Unicité **par société**

*(La forme SQL exacte de la contrainte est fixée par **D2-bis** : elle porte sur une colonne générée `client_number_uniq`, pas sur `client_number` directement. D2 pose le principe, D2-bis son périmètre.)*

*Pourquoi* : un numéro de client qui désigne deux contacts ne remplit pas sa fonction — il sert précisément à identifier. MariaDB autorise **plusieurs `NULL`** dans un index `UNIQUE`, donc les contacts sans numéro (la majorité au départ) ne se gênent pas entre eux. Le précédent est dans la **même table** : `uq_contacts_company_ide` (`20260414000001_contacts.sql:23`).

⚠️ La violation d'unicité doit remonter une **erreur métier** (**409 CONFLICT**, aligné sur le précédent), pas un 500 opaque sur l'erreur SQLx `1062`. Voir AC4.

**Vérifié empiriquement sur le moteur, pas supposé** (2026-08-10, MariaDB dev) : trois lignes `(company_id = 1, client_number = NULL)` sont acceptées sous cette contrainte, et un doublon non-`NULL` est refusé par `ERROR 1062 (23000) Duplicate entry '1-C-1' for key 'uq'`. C'est l'invariant dont dépend toute la décision — s'il tombait, la majorité du parc, qui n'aura pas de numéro, deviendrait insaisissable.

⚠️ **`''` N'EST PAS `NULL`, et c'est le piège de cette décision.** L'invariant ci-dessus vaut pour `NULL` **littéral**. Une chaîne vide est une valeur comme une autre pour un index `UNIQUE` : deux contacts soumis avec `client_number: ""` — le cas **majoritaire** que D2 prétend protéger — se percuteraient. La parade existe déjà dans le dépôt et doit être appliquée : `normalize_optional()` (`crates/kesh-api/src/routes/contacts.rs:259`), qui trime et effondre le vide en `None`, déjà branchée sur `email` (l. 348), `phone` (l. 360) et `default_payment_terms` (l. 369), et testée (l. 773). Voir AC3.

### D2-bis — Un contact archivé **libère** son numéro : unicité partielle sur les actifs

La contrainte porte sur une **colonne générée**, pas sur `client_number` directement :

```sql
client_number_uniq VARCHAR(50) GENERATED ALWAYS AS (IF(active, client_number, NULL)) VIRTUAL,
CONSTRAINT uq_contacts_company_client_number UNIQUE (company_id, client_number_uniq)
```

C'est la **troisième application** d'un patron déjà éprouvé deux fois dans ce dépôt — MariaDB n'ayant pas d'`UNIQUE` partiel natif, la convention « `NULL` n'est jamais égal à `NULL` » en tient lieu (`20260513000001_reconciliation_rules.sql:19-28`, `20260722000001_accounts_role_postable.sql:20-33`). Pré-requis MariaDB ≥ 10.6 pour un `UNIQUE` sur colonne `VIRTUAL` : le compose est épinglé sur `mariadb:10.11`.

⚠️ **Une version antérieure de cette story décidait l'inverse — contrainte plate, verrou permanent — et sa justification était doublement fausse.**

1. **L'échappatoire invoquée n'existe pas.** Elle disait « vider le champ sur l'archive libère le numéro ». Or **un contact archivé n'est pas modifiable** : `repositories/contacts.rs:421-426` rend `IllegalStateTransition("impossible de modifier un contact archivé")`, doublé par le `WHERE … AND active = TRUE` de l'`UPDATE` (l. 457) ; et il n'existe **aucune route de désarchivage** (`grep -rniE "unarchive|restore_contact|reactivate" crates/kesh-api/src/routes/contacts.rs crates/kesh-db/src/repositories/contacts.rs crates/kesh-db/src/entities/contact.rs` → 0 ligne ; ⚠️ **le périmètre de fichiers est indispensable** — la même commande lancée sur `crates/` entier rend **84** lignes, car `accounts`, `projects` et `reconciliation_rules` ont bien, eux, des routes de réactivation). Le numéro aurait donc été perdu **à vie**, déblocable seulement par un `UPDATE` SQL manuel en base — ce qui n'est pas une échappatoire, c'est une intervention hors produit.
2. **Le dépôt avait déjà rendu ce jugement, par écrit, sur le cas jumeau.** `20260722000001_accounts_role_postable.sql:35-38` : « *Le `active AND` n'est PAS cosmétique : sans lui, un compte archivé squatterait son rôle singleton à vie et son remplaçant actif ne pourrait jamais le recevoir (409 permanent causé par un compte mort).* » La situation des contacts est **strictement plus fermée**, puisqu'ils n'ont même pas de réactivation.

**Et le précédent `uq_contacts_company_ide` ne sauve pas le verrou** : un IDE est un identifiant **attribué par l'État**, jamais réattribuable à une autre entité — le verrou permanent y est sémantiquement juste. Un numéro de client est une **étiquette interne**, recyclable par nature.

**Enfin, le raisonnement par D5 était inversé.** Il prétendait que libérer un numéro le ferait « désigner deux entités selon l'époque ». Le système ne résout **jamais** numéro → contact : une facture désigne son contact par clé étrangère (`entities/invoice.rs`), et le PDF imprime le numéro *courant* de ce contact. Il n'y a donc aucune ambiguïté interne. La seule ambiguïté possible est sur le **papier déjà envoyé** — exactement l'artefact que D5 laisse déjà dériver pour le nom et l'adresse. D5 est un argument **contre** le verrou, pas pour.

### D3 — Rendu dans le bloc **métadonnées (droite)**, sous le numéro de facture

**Position exacte** : **entre la ligne « N° facture » et la ligne « Date »** — pas ailleurs dans le bloc. Le test de delta d'AC6 est insensible au point d'insertion (tout emplacement au-dessus de l'échéance donne le même écart de 4,5), donc **rien ne rattraperait un mauvais choix** : il faut le fixer ici.

*Pourquoi ce bloc* : c'est ce que prescrit la note de planification au sprint-status (« bloc métadonnées du PDF »), et c'est la place usuelle sur une facture commerciale — le numéro de client appartient aux références du document, pas à l'adresse du destinataire. Le patron à copier est `origin_reference` (`pdf.rs:315-324`) : ligne conditionnelle qui ne descend le curseur `my` que si elle est dessinée.

**Et une raison matérielle, plus forte que l'usage** : `pdf.rs:342` pose le bloc destinataire à `y.min(PAGE_H - 55.0)`, soit 55 mm du bord supérieur — **c'est la fenêtre de l'enveloppe** suisse. Y insérer une mention non postale serait fautif à l'impression. Le bloc métadonnées, à `meta_x = 120.0`, est hors fenêtre.

### D4 — Les **avoirs** le portent aussi

*Pourquoi* : symétrie avec 16-3a, dont le manuel utilisateur dit « Les avoirs portent les mêmes coordonnées, à l'identique ». Un avoir est adressé au même client et sert le même rapprochement. `draw_invoice_section` est partagée par les deux documents — le champ suit donc automatiquement **dès lors que le site de construction de la donnée le renseigne** (voir le piège en Dev Notes).

### D5 — La valeur vient du **contact destinataire** de la facture

Résolue à la génération du PDF, comme le nom et l'adresse du débiteur. Pas de copie dénormalisée sur `invoices` : le numéro de client est un attribut du **contact**, et un changement doit se refléter sur les PDF régénérés.

## Acceptance Criteria

**AC1 — La colonne existe et est unique par société, entre contacts ACTIFS.**
Migration `crates/kesh-db/migrations/<date>_contacts_client_number.sql` : `ADD COLUMN client_number VARCHAR(50) NULL`, plus la colonne générée et la contrainte de D2-bis (`client_number_uniq` … `UNIQUE (company_id, client_number_uniq)`).

⚠️ **L'unicité est insensible à la casse.** `contacts` ne déclare aucune collation (`20260414000001_contacts.sql:4-30`) et hérite donc du défaut MariaDB, une collation `_ci` : `CLI-1` et `cli-1` sont **identiques** pour la contrainte. C'est souhaitable — un utilisateur ne doit pas créer deux clients qui ne diffèrent que par la casse — mais ce doit être **décidé**, pas subi. Le précédent d'`ide_number` n'éclaire pas ce point : il est normalisé en majuscules avant stockage, donc sa casse ne varie jamais, alors que `client_number` ne passe que par `normalize_optional()` (trim, pas de casse).

*Preuve*, quatre cas :
1. deux contacts **actifs** de la même société, même numéro → **rejet** ;
2. deux contacts avec `client_number = NULL` → **tous deux acceptés** (l'invariant MariaDB dont dépend D2 — s'il tombait, la majorité du parc deviendrait insaisissable) ;
3. `CLI-1` puis `cli-1` sur deux actifs → **rejet** (la casse) ;
4. **le numéro d'un contact archivé est réattribuable à un contact actif** — c'est l'invariant de D2-bis, et rien d'autre ne le vérifie.

**AC2 — Le champ traverse le repository sans se perdre.**
Les **quatre** listes de colonnes **SQL** de `crates/kesh-db/src/repositories/contacts.rs` portent le champ : `COLUMNS` (l. 28-31), `FIND_BY_ID_SQL` (l. 33-36), l'`INSERT` (l. 201) **et ses placeholders `VALUES`**, l'`UPDATE` (l. 451).

**AC2-bis — L'audit trace la modification du numéro.**
`contact_snapshot_json` (`repositories/contacts.rs:39-58`) est une **cinquième liste de champs écrite à la main**, non-SQL, qui alimente l'audit log. Elle porte déjà `ideNumber` ; `client_number` est de **même nature** — un identifiant du contact, unique par société — et son omission rendrait l'audit aveugle à sa modification **sans que rien ne casse ni ne compile en erreur**.

*Preuve* : test qui modifie le numéro et vérifie que l'entrée d'audit en porte la trace, avant et après.

⚠️ Ce snapshot est **partiel par conception** — les champs structurés de #213 (`first_name`, `address_street`, …) n'y figurent pas. L'argument d'inclusion n'est donc pas « il faut tout y mettre », mais « `ideNumber` y est, et ces deux champs sont fonctionnellement jumeaux ».

**AC2-ter — Modifier UNIQUEMENT le numéro enregistre réellement la modification.**
`is_no_op_change` (`repositories/contacts.rs:377-396`) est une **sixième liste écrite à la main** — 18 comparaisons champ à champ — et c'est la plus dangereuse de toutes.

Son verdict **court-circuite l'écriture** (l. 439-442) :

```rust
if is_no_op_change(&before, &changes) {
    tx.rollback().await.map_err(map_db_error)?;
    return Ok(before);
}
```

Si `client_number` n'y est pas ajouté, alors **le geste central de cette story** — ouvrir une fiche existante pour lui attribuer son numéro, sans rien changer d'autre — est classé « aucun changement » : transaction annulée, aucun `UPDATE`, aucune entrée d'audit, et l'API rend **`200 OK` avec l'ancienne valeur**. L'utilisateur voit son numéro disparaître au rechargement, **sans le moindre message d'erreur**.

C'est pire que l'oubli d'AC2-bis : l'audit aveugle perd une *trace*, celui-ci perd la *donnée* — et perd précisément celle que la story existe pour ajouter.

*Preuve* : `update` ne modifiant **que** `client_number` → `find_by_id` rend la nouvelle valeur **et** `version` a été incrémentée. Le contrôle de `version` n'est pas décoratif : il distingue une écriture réelle d'un `Ok(before)` renvoyé par le court-circuit.
*Preuve* : test d'aller-retour `create` → `find_by_id` → `update` → `find_by_id` qui vérifie la valeur à chaque étape. ⚠️ Un test qui n'utilise que `list` passerait alors même que `FIND_BY_ID_SQL` aurait été oublié — voir Dev Notes, « le piège qui coûterait le plus cher ».

**AC3 — Le champ est saisissable et relisible depuis l'API.**
`ContactResponse` (`crates/kesh-api/src/routes/contacts.rs:151`) et son `impl From<Contact> for ContactResponse` (l. 186), plus `NewContact` (`crates/kesh-db/src/entities/contact.rs:213`), `ContactUpdate` (l. 250) et **les deux** DTO d'entrée (`CreateContactRequest` l. 70, `UpdateContactRequest` l. 108) portent le champ.

⚠️ **Ces noms sont ceux de `contacts`, pas ceux de `companies`.** Une version antérieure de cette story écrivait `ContactJson` et `ContactChanges` — du vocabulaire recopié de 16-3a, où `CompanyJson` est bien le nom réel. Ni l'un ni l'autre n'existe ici (`grep -nF "struct ContactJson"` → 0 ligne).

⚠️ **Le champ passe par `normalize_optional()`** (`contacts.rs:259`), comme `email`, `phone` et `default_payment_terms`. Sans cela, `""` est stocké tel quel et **percute l'unicité** de D2 dès le deuxième contact sans numéro — le cas majoritaire. Une borne de longueur dédiée (`MAX_CLIENT_NUMBER_LEN = 50`) est souhaitable par cohérence avec ses champs voisins (l. 339-374) ; à défaut, le filet générique absorbe le cas proprement (MariaDB `STRICT_ALL_TABLES` → `1406` → `DbError::DataLengthOrRange` → **400** `DATA_LENGTH_OR_RANGE`), avec un message moins précis.

*Preuve* : test E2E API `POST /contacts` avec `clientNumber` → `GET /contacts/{id}` rend la même valeur. **Aucun compilateur ne vérifie cette couture** : omettre la ligne dans `From<Contact>` compile, stocke, et rend `null` pour toujours. C'est le HIGH de la passe 3 de 16-3a, sur la struct jumelle `CompanyJson`.

*Preuve additionnelle, non négociable* : `POST` avec `clientNumber: ""` → stocké `NULL`, **et deux contacts créés ainsi sont tous les deux acceptés**. Sans ce test, la garde de D2 tient sur une convention de politesse du client.

**AC4 — Un doublon rend une erreur métier alignée sur le précédent, pas un 500 ni un code inventé.**
`POST`/`PUT` avec un `client_number` déjà pris dans la même société → **409 CONFLICT**, code `CLIENT_NUMBER_ALREADY_EXISTS`.

⚠️ **Le mapping existe déjà — l'étendre, ne pas en écrire un second.** `map_contact_error` (`crates/kesh-api/src/routes/contacts.rs:461`) intercepte déjà `DbError::UniqueConstraintViolation` sur `uq_contacts_company_ide` et le remappe vers `AppError::IdeAlreadyExists`, qui rend **409 / `IDE_ALREADY_EXISTS`** (`crates/kesh-api/src/errors.rs:1208`). La nouvelle contrainte est **de même nature** — unicité par société sur la même table — et doit donc rendre le **même code HTTP**. Travail réel : une branche de plus dans `map_contact_error`, une variante `AppError::ClientNumberAlreadyExists`, sa ligne dans le `match` de `errors.rs`.

⚠️ **Matcher le nom de CONTRAINTE, jamais le nom de colonne** — le helper le fait déjà et son doc-comment en donne la raison : le format du message MariaDB varie entre versions (10.x / 11.x, schéma préfixé ou non).

*Preuve* : test E2E API qui crée le doublon et assert **le code HTTP et le code d'erreur**. Un test qui n'assert que « ce n'est pas 200 » laisserait passer un 500. **Plus** un test de **non-sur-capture** : le helper matche par `contains`, et avec deux contraintes il faut prouver qu'aucune ne capture l'autre. Le dépôt a déjà ce test pour l'IDE (`contacts.rs:765`, « Doit être mappé en `AppError::Database` (pas `IdeAlreadyExists`) ») — le calquer.

**AC5 — La fiche contact permet de le saisir.**
Champ dans l'écran contact du frontend, avec libellé traduit.
*Preuve 1 (création)* : test E2E Playwright qui saisit un numéro, enregistre, recharge la page et le relit. ⚠️ Un E2E est le **seul** test qui vérifie qu'une valeur traverse réellement la frontière HTTP — Vitest teste la construction du payload, les tests Rust la validation, et ni l'un ni l'autre ne voit une clé qui disparaît entre les deux.

*Preuve 2 (édition) — non négociable, et la preuve 1 ne la remplace pas* : créer un contact **avec** un numéro, puis **modifier un champ sans rapport** (le téléphone), enregistrer, et vérifier que **le numéro a survécu**.

⚠️ **Le chemin d'édition efface le champ si une seule ligne est oubliée, et rien ne le signale.** `PUT /contacts/{id}` est un **full-replace** : `frontend/src/routes/(app)/contacts/+page.svelte:336-341` envoie le **même `payload`** pour la création et l'édition, et `openEdit` (l. 240-258) hydrate le formulaire **champ par champ**, sur dix-huit lignes. Ajouter `clientNumber` au payload **sans** ajouter `formClientNumber = c.clientNumber ?? ''` à `openEdit` ne casse **aucune** compilation (le champ TS est optionnel) — et toute modification d'un contact, même sans rapport, **efface son numéro**. La preuve 1 passe au vert sous cette mutation, puisqu'elle ne parcourt que la création.

**AC6 — Le PDF affiche le numéro quand il existe, et rien quand il n'existe pas.**
`InvoicePdfData` porte `debtor_client_number: Option<String>` ; `draw_invoice_section` dessine la ligne sous le numéro de facture **uniquement** si `Some`, et ne descend `my` que dans ce cas — `my -= 4.5` **à l'intérieur** du `if let`, comme `origin_reference` (`pdf.rs:315-324`).

*Preuve (présence)* : deux cas, `Some` et `None`, comparés par **delta de taille du PDF** — le contenu textuel est hex-encodé dans les opérateurs `Tj`, un `grep` du texte échouerait de toute façon.

⚠️ **Le delta de taille NE PEUT PAS prouver la conditionnalité du décrément, et le dépôt l'a déjà payé.** `pdf.rs:1165-1168` le dit noir sur blanc : « un décalage vertical de 2 mm déplace le texte sans changer sa longueur, donc le PDF pèse le même nombre d'octets. **Mesuré** — une première version de ce test comparait deux générations entre elles et restait **verte** sous la mutation. » La parade de 16-3a fut de mesurer au **seuil de la garde haute**, où 2 mm changent un verdict (`no_identity_line_costs_no_vertical_space`, l. 1182). **Cette parade n'est pas disponible ici** : la garde ne surveille que `y`, la colonne gauche, et les Dev Notes de cette story établissent qu'il ne faut pas en créer une à droite (89,5 mm de marge ⇒ branche inatteignable).

*Preuve (conditionnalité) — le refactor est donc obligatoire*, comme AC5 de 16-3a a imposé le sien : extraire la construction du bloc métadonnées en **fonction pure** rendant la séquence des lignes et leur ordonnée (`Vec<(String, f32)>`), et tester **la position**.

⚠️ **PAS « les deux ordonnées sont identiques » — c'est la signature du mutant.** Avec un code **correct**, la ligne « échéance » est 4,5 mm **plus basse** dans le cas `Some` ; c'est la mutation qui rend les deux ordonnées **égales**. Une assertion d'égalité échouerait donc sur du bon code, et pousserait à implémenter la mutation pour la verdir.

**Une seule formulation, et c'est délibéré** : `assert_eq!(y_none - y_some, 4.5)` — le delta *doit* valoir 4,5 ; la mutation le met à 0. **Ne pas** asserter une ordonnée absolue codée en dur : elle dépend du fixture, le delta est invariant.

Sans ce montage, AC6 est un critère dont aucun test ne peut échouer, c'est-à-dire un test muet — le mode d'échec le plus documenté de ce dépôt.

**AC6-bis — Un numéro trop long est tronqué, pas débordé.**
Le bloc droit dispose de **70 mm** (`meta_x = 120.0` à `PAGE_W − 20.0 = 190.0`, cf. `hline` l. 428) — **moins** que les 100 mm du bloc gauche, qui imposent déjà `IDENTITY_MAX_CHARS = 46`. Or `client_number` est un champ **libre de 50 caractères saisi par l'utilisateur**, contrairement à `origin_reference` qui est un numéro système court : le patron de D3 ne couvre donc pas ce risque.
*Preuve* : constante de troncature **dédiée au bloc droit**, et test de calibrage calqué sur celui du bloc gauche. Tronquer, jamais refuser : refuser une facture pour un champ décoratif serait disproportionné (c'est déjà l'arbitrage de 16-3a, `pdf.rs:266-267`).

**La valeur est donnée, pas laissée à deviner** : en reprenant la méthode de calibrage d'`IDENTITY_MAX_CHARS` (`pdf.rs:180-201` — table AFM Helvetica 9 pt, largeur moyenne des capitales 677/1000 em, 1 pt = 0,3528 mm) :

```
70 / (9 × 0,677 × 0,3528) ≈ 32,6  →  32 caractères
```

soit bien moins que les 46 du bloc gauche, comme attendu de ses 70 mm contre 100. ⚠️ Ne pas partir de 46 en le réduisant « à vue » : pour `IDENTITY_MAX_CHARS`, la valeur juste (46, et non 50) n'a été obtenue qu'**en passe 3 de revue** de la 16-3a.

⚠️⚠️ **LA BORNE PORTE SUR LA LIGNE COMPLÈTE `"{libellé}: {valeur}"`, PAS SUR LA VALEUR SEULE.** C'est le piège le plus coûteux de cette story, parce qu'il fait *recréer* par une exécution fidèle le défaut que cet AC prévient. Le patron du bloc gauche tronque la ligne **formatée** :

```rust
// pdf.rs:271
truncate_display(&format!("{}: {}", i18n.get(key), v), IDENTITY_MAX_CHARS),
```

Les 32 caractères sont un budget de **largeur** (120 mm → 190 mm) : ils couvrent donc nécessairement le libellé. Écrire `truncate_display(&client_number, 32)` puis formater le libellé autour produit une ligne d'environ 43 caractères ≈ **92 mm** à partir de `x = 120`, soit `x ≈ 212` sur une page de **210 mm** — du texte **hors feuille**. Et cet AC établit lui-même, deux lignes plus bas, que **le test de calibrage ne peut pas le voir**.

**Conséquence à connaître, et que personne n'a arbitrée** : appliquée correctement, la borne laisse à la valeur ce que le libellé ne prend pas — « N° client: » (11 caractères) laisse **~21 caractères**, « Kundennummer: » (14) en laisse ~18. Un champ déclaré `VARCHAR(50)` s'imprime donc sur une vingtaine de caractères. C'est acceptable (la troncature est un repli d'affichage, la valeur reste entière en base et dans les réglages), mais cela doit être **su** plutôt que découvert.

⚠️ **Limite du test de calibrage, à connaître avant de s'y fier** : celui du bloc gauche (`an_overlong_contact_detail_is_truncated_not_overflowed`) prouve la **cohérence** de la troncature — deux chaînes longues coupées à la même taille — mais **ne mesure jamais si la valeur tient réellement dans la largeur disponible**. Une constante trop généreuse (40 au lieu de 32) passerait ce test et ne se verrait qu'à l'œil sur un PDF rendu. La justesse de la valeur repose donc sur le **calcul ci-dessus**, pas sur le test.

**AC7 — L'avoir le porte aussi.**
*Preuve* : test au **site de construction de la donnée**, pas dans `pdf.rs`. Facture et avoir partagent `draw_invoice_section` : un test posé dans `pdf.rs` ne peut **structurellement pas** discriminer les deux, et resterait vert sous la mutation « ne pas renseigner le champ pour l'avoir ». ⚠️ Interdit d'hériter d'une fixture de facture par `..base` — c'est ce qui avait rendu le test d'AC5 de 16-3a muet.

**AC8 — L'i18n est complète sur les 4 locales.**
Clé `invoice-pdf-client-number` ajoutée à `I18N_KEYS` **et à la même position dans `DEFAULT_EN`** (`crates/kesh-qrbill/src/types.rs`), plus les 4 locales `fr-CH`/`de-CH`/`it-CH`/`en-CH` pour le libellé du PDF **et** celui de la fiche contact.
*Preuve, côté PDF* : l'assertion de compilation existante (`types.rs:264`) couvre les **longueurs** ; l'**appariement positionnel** ne l'est pas — le vérifier par un test qui résout la clé et compare au libellé attendu (mutation à tuer : décaler une entrée de `DEFAULT_EN` d'un cran ; les longueurs restent égales, le test runtime doit rougir).

⚠️ **La preuve 4-locales doit porter sur les DEUX clés — celle du PDF ET celle de l'écran.** `build_i18n` (`routes/invoice_pdf_service.rs:311-317`) résout **chaque** entrée d'`I18N_KEYS` par `bundle.format`, et `crates/kesh-i18n/src/loader.rs:130-135` charge d'abord **toutes les clés `fr-CH` comme base de repli** des autres locales. Une clé présente seulement en français rend donc le libellé **français sur un PDF allemand**, en silence — `DEFAULT_EN` n'est **jamais atteint en production**, il ne sert que de repli de dernier recours côté crate. La preuve d'appariement positionnel ci-dessus ne voit pas ce cas : elle teste `I18N_KEYS` ↔ `DEFAULT_EN`, un autre mécanisme. Sans preuve dédiée, un PDF de-CH imprimerait « N° client » avec **tous les tests au vert** — c'est le mécanisme exact de la KF #283, appliqué à l'artefact qui donne son titre à la story.

*Preuve, côté fiche contact et côté PDF — sans quoi AC8 ne couvre que la moitié du sujet* : un test qui résout **`invoice-pdf-client-number` et la clé du libellé d'écran** dans les 4 locales, et échoue si l'une retombe sur le repli français. Le loader `kesh-i18n` **replie silencieusement** une clé absente vers le FR (`loader.rs`, `format_missing_key_in_de_falls_back_to_fr`) — c'est un comportement voulu, mais il rend l'absence **invisible** : c'est le mécanisme même de la KF #283 (57 clés déjà manquantes en de-CH / it-CH / en-CH). AC8 avertissait de « ne pas l'aggraver » sans fournir aucune preuve exécutable pour la partie exactement visée ; l'avertissement reposait donc sur la seule discipline manuelle.

**AC9 — Les garde-fous de migration sont honorés.**
Ligne ajoutée au tableau de `docs/migrations-idempotence-audit.md` **à sa place chronologique dans le tableau**, avec les **deux** sites du total et la partition recomptés **depuis le tableau** (`ls crates/kesh-db/migrations/*.sql | wc -l` doit égaler `grep -c '^| \`20' docs/migrations-idempotence-audit.md`). DDL pur → **ni** registre `POST_RESTORE_BACKFILLS` **ni** `EXEMPT_MIGRATIONS` (P7). `ADD COLUMN` nullable → **pas** de bump `kesh_version_min_required` ni de version Cargo (P1/P2).
*Preuve* : le test `every_data_backfill_migration_is_triaged` ne doit jamais sélectionner cette migration. **Et** `upgrade_path_preserves_data` doit repasser au vert après le bump de `assert_eq!(total, …)` à **60** et l'arbitrage de la fenêtre — un rouge laissé là masquerait tout ajout ultérieur.

**AC10 — Le numéro est cherchable.**
La recherche de contacts accepte le numéro de client.

*Pourquoi c'est un critère et non un confort* : le « so that » de cette story promet **deux** bénéfices, et le second est « que je puisse moi-même **retrouver un contact depuis une facture papier** ». Sans recherche, un utilisateur tenant une facture portant « N° client : CLI-2026-00042 » n'a **littéralement aucun moyen** de remonter au contact. Livrer une story dont la moitié du bénéfice annoncé n'est ni spécifiée ni testée est le plus mauvais des choix disponibles.

*(La **colonne** « N° client » dans la liste des contacts a été **écartée** — arbitrage du Project Lead, passe 6. Voir « Ce que cette story ne doit PAS faire ».)*

*Le coût est faible, et le dépôt donne la bonne technique.* La recherche actuelle (`repositories/contacts.rs:170-186`) couvre `name` par `MATCH … AGAINST` (FULLTEXT) et `email` par `LIKE`. Le commentaire de la l. 162-165 explique **pourquoi** `email` est resté en `LIKE` : ses séparateurs cassent les tokens FULLTEXT. Un `CLI-2026-00042` subirait exactement le même sort — donc une branche `OR client_number LIKE ?` à côté de celle d'`email`, et non un ajout à l'index FULLTEXT.

*Preuve* : test API — recherche par numéro exact rend le contact ; recherche par fragment le rend aussi. Test E2E — la colonne est visible dans la liste.

⚠️ Ne pas oublier le **placeholder** du champ de recherche (`frontend/src/routes/(app)/contacts/+page.svelte:441`, « Rechercher par nom ou email… ») : le laisser tel quel ferait mentir l'interface sur ce qu'elle sait faire, sur les 4 locales.

## Tasks / Subtasks

- [x] **T1 — Migration** (AC1, AC9). Créer le `.sql` : `ADD COLUMN client_number`, **`ADD COLUMN client_number_uniq … GENERATED ALWAYS AS (IF(active, client_number, NULL)) VIRTUAL`**, puis `ADD CONSTRAINT … UNIQUE (company_id, client_number_uniq)`. En-tête de commentaire du dépôt : rôle, longueur justifiée, statut non-breaking, mention « DDL pur, ni registre ni exemption », **et le renvoi aux deux précédents du patron** (`reconciliation_rules`, `accounts_role_postable`) avec le pré-requis MariaDB ≥ 10.6. Puis le garde-fou **P6**, dont le relevé est **déjà fait** ci-dessous — inutile de le refaire à l'aveugle, mais **le vérifier** :

- ⚠️ **`upgrade_path_preserves_data` VA ÉCHOUER, et c'est son rôle.** `crates/kesh-db/tests/migrations_upgrade_path.rs:89-94` porte `assert_eq!(total, 59, "59 migrations attendues (58 précédentes + Story 16-3a : companies_phone_website)")`. Il faut donc : passer le compte à **60**, prolonger la **généalogie en commentaire** (l. 86-88, qui énumère chaque story) d'une ligne « + `contacts_client_number` (Story 16-3b, #151) = 60 », **et trancher la question que pose le message d'assertion de `apply_migrations_up_to`** (l. 30-47) : la fenêtre d'upgrade `total − 25` s'élargit-elle d'un cran, ou la frontière doit-elle rester à 34 (auquel cas il faut bumper `total` **et** la taille de fenêtre) ? Cette décision est explicite par construction — le test existe pour l'imposer.
- **Deux définitions distinctes** de `apply_migrations_up_to` coexistent, chacune avec son propre garde-fou `assert!(n <= all.len())` : `tests/common/mod.rs:50` et `tests/migrations_upgrade_path.rs:30`.
- **Les deux suites de backfill sont insensibles à l'ajout** — `accounts_role_backfill.rs` et `invoice_lines_revenue_account_backfill.rs` (une quinzaine d'appels) passent par `migrations_before(<version>, <nom>)`, c'est-à-dire une **résolution par version**, le critère que P6 prescrit de préférence. Rien à y faire — mais le **vérifier** plutôt que le supposer, c'est précisément ce que la Story 16-1a a payé.
- [x] **T2 — Audit d'idempotence** (AC9). Ligne dans le tableau, **recompter** les deux sites du total et les trois partitions. État de départ mesuré le 2026-08-10 : **59** fichiers `.sql` = 59 lignes de tableau = en-tête = total, partitionnés en `54 tracked-by-sqlx + 5 yes + 0 no`. Cette migration porte l'ensemble à **60** et `tracked-by-sqlx` à **55** — mais **recompter depuis le tableau**, ne pas incrémenter ces chiffres de confiance : c'est exactement le geste qui avait laissé dériver le compteur de 7 unités jusqu'à la Story 16-1a. ⚠️ Les compteurs de partition ne valent pas le total ; les aligner dessus casserait l'invariant qu'ils servent à tenir.
- [x] **T3 — Entité et repository** (AC1, AC2). `Contact`, `NewContact` (`entities/contact.rs:213`), `ContactUpdate` (l. 250) — **pas** `ContactChanges`, qui n'existe pas —, puis **les quatre** listes de `repositories/contacts.rs`. Vérifier que le nombre de `?` de l'`INSERT` suit la liste de colonnes. `reconciliation.rs:201` réutilise `COLUMNS` — rien à y faire, mais le vérifier plutôt que le supposer.
- [x] **T3-bis — Audit** (AC2-bis). Ajouter le champ à `contact_snapshot_json` (`repositories/contacts.rs:39-58`) — **cinquième liste de champs écrite à la main**, non-SQL, que les quatre listes de T3 ne couvrent pas.
- [x] **T3-ter — Détection de non-changement** (AC2-ter). Ajouter le champ à `is_no_op_change` (`repositories/contacts.rs:377-396`) — **sixième liste**. Sans elle, modifier le seul numéro est classé no-op : rollback, `200 OK`, donnée perdue en silence.
- [x] **T4 — Tests repository** (AC1, AC2, AC2-bis, AC2-ter). Aller-retour `create`/`find_by_id`/`update`/`find_by_id` ; unicité rejetée **entre contacts actifs** ; **deux `NULL` acceptés** ; **numéro d'un contact archivé réattribuable à un actif** (l'invariant de D2-bis) ; trace d'audit ; et `update` du **seul** `client_number` → nouvelle valeur **et** `version` incrémentée.
- [x] **T5 — Route et DTO** (AC3, AC4). `ContactResponse` (`contacts.rs:151`) + `impl From<Contact> for ContactResponse` (l. 186) + **les deux** DTO d'entrée (`CreateContactRequest` l. 70 et `UpdateContactRequest` l. 108), **avec `normalize_optional()`** sur le champ (l. 259) comme pour `email`/`phone`. Pour l'erreur : **étendre `map_contact_error` (`contacts.rs:461`)** d'une branche, ajouter la variante `AppError::ClientNumberAlreadyExists` et sa ligne dans le `match` de `errors.rs` (409 / `CLIENT_NUMBER_ALREADY_EXISTS`). Ne **pas** écrire un second helper : le repository rend déjà `DbError::UniqueConstraintViolation`, tout le chemin existe.
- [x] **T6 — Tests API** (AC3, AC4). Aller-retour `POST`/`GET` ; doublon → **409** + code d'erreur asserté ; **non-sur-capture** entre les deux contraintes (calquer `contacts.rs:765`).
- [x] **T7 — Frontend** (AC4, AC5, AC8, AC10). Type TS (`contacts.types.ts`, interface `ContactResponse`), champ de la fiche contact, **hydratation dans `openEdit` (l. 240-258) — la ligne dont l'oubli efface le champ à chaque édition**, **placeholder de recherche corrigé** (l. 441), libellés sur les 4 locales. ⚠️ **Pas de colonne dans le tableau de liste** (écartée en passe 6) — donc **ne pas toucher au `colspan`**, câblé en dur à deux endroits. Respecter `lint-i18n-ownership`.
- [x] **T7-ter — Message du 409 à l'écran** (AC4). Le frontend branche les codes **un par un** (`frontend/src/routes/(app)/contacts/+page.svelte:349-358`) : `OPTIMISTIC_LOCK_CONFLICT`, puis `IDE_ALREADY_EXISTS` → `contact-error-ide-duplicate`, **sinon `err.message` brut du backend**. Sans branche dédiée, l'utilisateur qui saisit un doublon reçoit un message non maîtrisé et non traduit, là où le doublon d'IDE a droit au sien. Ajouter la branche `CLIENT_NUMBER_ALREADY_EXISTS` et la clé `contact-error-client-number-duplicate` **dans les 4 locales** — le domaine `contact-*` est aujourd'hui à 59 clés dans chacune, sans dérive : ne pas l'ouvrir.
- [x] **T7-bis — Recherche par numéro** (AC10). Branche `OR client_number LIKE ?` dans `repositories/contacts.rs`, **pas** dans l'index FULLTEXT — le commentaire l. 162-165 explique pourquoi les séparateurs cassent les tokens (et `escape_like` préserve les tirets, donc `CLI-2026-00042` fonctionne). ⚠️ **Il y a DEUX sites `email LIKE`, l. 174 et l. 181** — la première branche sert quand le terme échappé est vide, la seconde le cas courant. N'en traiter qu'un compile, passe les tests dont le terme survit à `escape_boolean_ft`, et **cesse silencieusement de chercher le numéro** quand le terme n'est fait que d'opérateurs FULLTEXT. Échec rare et muet : les deux sites, ou aucun.
- [x] **T8 — E2E fiche contact** (AC5). Saisie → enregistrement → rechargement → relecture. ⚠️ Le fichier **DOIT** être nommé `*.spec.ts` : `playwright.config.ts:35` filtre sur `testMatch: /(.+\.)?spec\.[jt]s/`, et un `*.test.ts` posé dans `tests/e2e/` est **silencieusement ignoré** — il ne rougit jamais, il se tait.
- [x] **T9 — PDF** (AC6, AC6-bis, AC8). `InvoicePdfData.debtor_client_number`, rendu conditionnel calqué sur `origin_reference` (`pdf.rs:315-324`), **extraction de la fonction pure** de construction du bloc métadonnées (sans elle, AC6 est intestable), **constante de troncature dédiée au bloc droit** (70 mm, donc < `IDENTITY_MAX_CHARS`), clé i18n à la **même position** dans `I18N_KEYS` et `DEFAULT_EN`.
- [x] **T10 — Service de génération** (AC6, AC7, D5). Renseigner le champ depuis le contact destinataire dans `invoice_pdf_service.rs` **et** au site de construction de l'avoir. C'est le site que la mutation doit tuer.
- [x] **T11 — Tests PDF** (AC6, AC6-bis, AC7). Présence par delta de taille ; **conditionnalité par assertion de position** sur la fonction pure extraite en T9 — l'assertion est **`y_none − y_some == 4.5`**, et ⚠️ **surtout PAS « les deux ordonnées sont identiques »** (signature du mutant) **ni une ordonnée absolue codée en dur** (fragile au fixture, et fausse dans la variante retirée) — cf. AC6 ; troncature calibrée ; avoir testé au site de construction, sans `..base` d'une fixture de facture.
*(T12 — export CSV du carnet d'adresses : **tranché, exclu**. Voir « Ce que cette story ne doit PAS faire ».)*
- [x] **T13 — Documentation** (règle de synchronisation). Les sites sont **énumérés** plutôt que laissés à l'appréciation :
  - `docs/manual/fr/user-manual.tex:510-520` — la liste des champs à renseigner à la création d'un contact. **C'est la page que lira l'utilisateur cherchant à quoi sert le nouveau champ.**
  - `docs/manual/fr/user-manual.tex:615` — « Vos coordonnées sur la facture », qui appelle un pendant côté client.
  - `docs/manual/fr/user-manual.tex:531` — « Recherchez par nom, IDE, adresse, email, téléphone » (à corriger avec AC10 ; ⚠️ cette phrase **surestime déjà** le code, qui ne cherche que `name` et `email`).
  - `docs/search-patterns.md:116` — le patron de recherche des contacts, à mettre à jour avec AC10.
  - `README.md:213` — ligne v0.9.0 de la feuille de route : cette story **ferme #151**, le libellé doit le refléter.
  - CHANGELOG, et régénération des PDF des manuels.

## Dev Notes

### Ce que cette story ne doit PAS faire

- **Pas de numérotation automatique**, pas de séquence, pas de backfill du parc existant (D1). Les contacts existants restent à `NULL`, et c'est l'état normal.
- **Pas de copie du numéro sur `invoices`** (D5). Le PDF le résout depuis le contact.
- **Pas de garde de capacité symétrique** — voir plus bas, ce serait du code mort.
- **Pas de numéro dans l'échéancier / rapport des débiteurs.** `crates/kesh-report/src/aged_receivables.rs:112` identifie le client par `c.name` seul, et l'export CSV de même (`csv.rs:456`). C'est le rapport où le numéro aurait le plus de sens après la facture — l'omission est **délibérée** et bornée au périmètre de cette story, pas un oubli.
- **Pas de variable `{clientNumber}` dans les modèles d'e-mail.** La liste blanche est à `entities/email_template.rs:53` et `:65` ; aucun modèle n'évoque de numéro de client. L'y ajouter serait une **feature**, pas une complétion.
- **Pas de colonne « N° client » dans la liste des contacts** — arbitrage du Project Lead, passe 6. Le bénéfice annoncé par le « so that » est **entièrement servi par la recherche** (AC10) ; une septième colonne au tableau est une décision d'interface que rien n'a demandée, et son coût est réel (le `colspan` câblé en dur à **deux** endroits, `frontend/src/routes/(app)/contacts/+page.svelte:537` et `:540`, plus un E2E et une clé sur 4 locales). Ne pas y toucher.
- **Pas de numéro dans l'export CSV du carnet d'adresses.** `serialize_contacts_csv` (`exports/csv_tables.rs:314`) a deux listes appariées positionnellement (en-têtes puis valeurs) et est **déjà partielle** : ni `first_name`, ni `address_street`, ni `language` n'y figurent. L'exclusion est donc cohérente avec l'état de cet export, et **tranchée ici** plutôt que laissée à l'appréciation de l'implémentation. Si elle devait être révisée un jour, **les deux** listes devraient être touchées ensemble, sous peine de décaler silencieusement toutes les colonnes suivantes.

### Les angles déjà explorés et rendus VIDES — ne pas les refaire

Relevé en passe 4. Chacun a été ouvert et n'appelle **aucun travail** :

- **Autres portes d'entrée d'un contact : il n'y en a pas.** Seul `create_contact` en construit en production. Les deux `NewContact { }` de `repositories/invoices.rs` sont **à l'intérieur** d'un `#[cfg(test)] mod tests`. `crates/kesh-seed` ne contient **aucune** occurrence de « contact », et `crates/kesh-import` non plus.
- **Fixtures CI** : `test_fixtures.rs:468-479` insère son contact par SQL à colonnes explicites — la nouvelle colonne prend `NULL`, rien à maintenir.
- **Import d'une sauvegarde antérieure à la migration** : sain. `ColumnConstraint::is_required()` (`backup.rs:324-329`) ne déclare obligatoire qu'une colonne `NOT NULL` **sans `DEFAULT`**, et `parse_ndjson_rows` substitue `Null` à toute clé absente.
- **Rappels / relances** : `routes/dunning_reminders.rs` ne produit **aucun PDF** ; le rappel joint la QR-facture, donc le numéro y voyage déjà.
- **`docs/api-external.md`** : ses exemples sont des payloads **partiels**, pas un schéma — aucune mise à jour due.
- **Site web** : aucune page ne décrit le contenu du PDF de facture.

⚠️ **Un défaut de documentation préexistant a été relevé en chemin, hors périmètre** : `user-manual.tex:533-535` annonce un « Contacts → Import CSV » qui **n'existe pas** dans le code. À tracer en issue GitHub, pas à corriger ici.

### Le piège qui coûterait le plus cher

**`FIND_BY_ID_SQL` (l. 33-36) duplique `COLUMNS` (l. 28-31) mot pour mot.** Les deux listes sont identiques aujourd'hui, mais ce sont **deux chaînes distinctes écrites à la main**. Ajouter le champ à `COLUMNS` seul compile, passe les tests de `list` (qui utilisent `COLUMNS`), et rend `find_by_id` **silencieusement amnésique** : la fiche contact affiche un champ vide alors que la base contient la valeur.

C'est pour cela qu'AC2 exige un aller-retour **par `find_by_id`**, et non par `list`.

### Le rayon d'impact réel — une vingtaine de fichiers cassent à la compilation, et c'est normal

`NewContact` et `ContactUpdate` n'ont **pas** de `#[derive(Default)]` (vérifié : `grep -nF "derive(Default)" crates/kesh-db/src/entities/contact.rs` → rien). Tout site qui les construit en **littéral complet** devra donc lister le nouveau champ :

```sh
grep -rl "NewContact {" crates/     # 23 fichiers
grep -rl "ContactUpdate {" crates/  #  3 fichiers
```

**Et `Contact` lui-même n'a pas non plus de `Default`** — au moins quatre fixtures le construisent en littéral complet et casseront de la même façon : `routes/invoice_email.rs:1371`, `routes/invoice_pdf_service.rs:459`, `routes/credit_notes.rs:390`, `kesh-reconciliation/src/matching.rs:244`. Le rayon réel dépasse donc la « vingtaine » annoncée.

**Ce n'est pas un risque de défaut silencieux** — `cargo build` échoue bruyamment sur chaque site, un à un. Mais c'est un dimensionnement à connaître avant de commencer : le précédent 16-3a n'avait que **6** fichiers construisant `Company` en littéral. `contacts` est bien plus diffusé dans le dépôt (factures, avoirs, réconciliation, rapports, fixtures). Corriger au fil des erreurs du compilateur, sans chercher à les anticiper toutes.

### Les sites qui suivent TOUT SEULS — ne pas les « corriger »

- **Le backup et l'import d'installation.** `crates/kesh-db/src/backup.rs` ne liste aucune colonne en dur : `non_generated_columns` (l. 92-110) les lit dans `information_schema`, et le `SELECT` est construit par `format!` à partir de ce résultat (l. 121-145). La nouvelle colonne y entre **sans une ligne de code**. Une passe de revue qui réclamerait une mise à jour de `backup.rs` se tromperait.
- **`reconciliation.rs:201`** réutilise `super::contacts::COLUMNS` — rien à y faire, mais le **vérifier** plutôt que le supposer.
- **`draw_invoice_section`** est partagée par la facture et l'avoir : le rendu suit automatiquement **dès lors que le site de construction renseigne le champ**. C'est précisément pourquoi AC7 se teste au site de construction et non dans `pdf.rs`.
- **Le refactor de testabilité de l'avoir est DÉJÀ ACQUIS** — `build_credit_note_pdf_data` (`crates/kesh-api/src/routes/credit_notes.rs:219`) est une fonction pure. 16-3a avait dû l'extraire ; cette story n'a pas à le redemander. Les deux sites de construction de `InvoicePdfData` sont `invoice_pdf_service.rs:269` et `credit_notes.rs:229`, **tous deux avec le `Contact` déjà en portée** — aucun filetage de paramètre supplémentaire n'est nécessaire pour D5.

À l'inverse, `serialize_contacts_csv` (`exports/csv_tables.rs:314`) liste bien ses colonnes en dur, **deux fois** — voir T12.

### Ce que j'ai vérifié plutôt que supposé — la garde de capacité haute

16-3a a posé une garde `if y < ty + 2.0 { return Err(HeaderOverflow) }` (`pdf.rs:382`). On pourrait croire qu'ajouter une ligne au bloc droit appelle une garde symétrique. **Le calcul dit non** — et il a fallu le refaire deux fois pour l'obtenir juste :

- `ty = PAGE_H - 130.0 = 297 - 130 = **167**`
- `my` part de `PAGE_H - 20.0 = 277`. Le code ne porte que **quatre** décréments de `my`, et les voici tous : `pdf.rs:295` (`-7.0`, après le titre), `:303` (`-4.5`), `:317` (`-4.5`, conditionnel, référence d'origine), `:327` (`-4.5`, conditionnel, échéance). Au **pire cas actuel**, `my` vaut donc `277 − 7 − 3 × 4,5 = ` **256,5**.

La colonne droite dispose donc de **89,5 mm**, soit **une vingtaine de lignes** de marge. Une ligne de plus la porte à 252. La garde est bien **asymétrique** (elle ne surveille que `y`, la colonne gauche), et c'est un fait à connaître — mais il est **sans conséquence pour la hauteur**. Ajouter une garde sur `my` produirait une branche que rien ne peut atteindre, donc intestable : exactement le genre de code qu'une passe de revue signalerait à juste titre.

⚠️ **Deux erreurs ont été commises sur ce calcul, et elles instruisent.** (1) Une version antérieure de ces notes comptait « conditions de paiement » parmi les lignes du bloc droit : c'est faux, `payment_terms` est dessiné dans la colonne **gauche**, près du total, avec un pas différent (`ty -= 8.0`, `pdf.rs:555-556`). (2) Le chiffre **252** qui en résultait a été retrouvé **à l'identique par une passe de revue indépendante**, qui comptait quatre décréments de 4,5 là où le code n'en porte que trois. Deux analyses, le même chiffre, la même erreur — parce qu'aucune des deux n'avait énuméré les décréments **depuis le code**. C'est le mode d'échec que le `CLAUDE.md` décrit pour les compteurs de migrations : *relire une valeur n'est pas recompter sa source*. La conclusion ne change pas ; la marge est même plus large qu'annoncée.

⚠️ **La hauteur n'est pas la largeur.** Ce calcul ne dit rien du débordement **horizontal**, qui est un risque réel et traité par AC6-bis : le bloc droit ne fait que 70 mm.

### Comment tester un PDF dans ce dépôt — lire AVANT d'écrire T11

Le seul précédent du dépôt **écarte explicitement** la comparaison octet à octet (« Plan C : on ne compare pas l'octet exact du PDF »). Et un `grep` du texte échouerait de toute façon : le contenu est **hex-encodé** dans les opérateurs `Tj`. Les techniques retenues par 16-3a, à réutiliser telles quelles :

- **Par le `Result`** — le cas qui doit être refusé l'est, avec la bonne variante d'erreur.
- **Par delta de taille** — deux rendus, l'un avec le champ, l'autre sans ; la différence de taille atteste que quelque chose a bien été dessiné.
- **Au site de construction** — pour tout ce que `pdf.rs` ne peut pas discriminer, l'avoir en particulier.

### Une règle sèche, née de trois erreurs sur le même bloc

**Ne jamais écrire une ordonnée du bloc métadonnées sans la recalculer depuis la chaîne de décréments de `pdf.rs:287-339`.** Trois chiffres faux ont été posés sur ce bloc au fil de la spécification — une marge (`252` au lieu de `256,5`), un décompte de décréments, et une ordonnée (`265,5`, qui est la position de la *date*, pas de l'échéance). Chaque fois par la même faute : un nombre écrit de tête plutôt que dérivé du code. Deux d'entre eux ont même été **reproduits à l'identique par une relecture indépendante**.

### Conventions de test

- Chaque patch de remédiation vient **avec** son test.
- Après un patch, `grep` le **symptôme** sur tout le dépôt — pas seulement le site corrigé : code, story file, doc-comments, tests, les 4 locales, fallbacks Svelte, manuels LaTeX.
- Campagne de mutation attendue sur les sites décisifs : renseigner le champ pour la facture mais pas pour l'avoir (doit tuer T11), retirer la ligne de `From<Contact>` (doit tuer T6), retirer le champ de `FIND_BY_ID_SQL` (doit tuer T4).

### References

- Issue **#151** — les trois volets, dont celui-ci est le dernier.
- **Story 16-3a** (`16-3a-coordonnees-emetteur-pdf.md`) — patron direct : rendu conditionnel, invariant i18n, technique de test PDF, couture DTO non vérifiée par le compilateur. Sa boucle de revue a convergé en 8 passes ; ses Dev Notes valent lecture avant de commencer.
- `crates/kesh-db/migrations/20260414000001_contacts.sql:23` — précédent d'unicité dans la même table.
- `crates/kesh-qrbill/src/pdf.rs:315-324` — patron `origin_reference` à copier.
- `CLAUDE.md` § *Migration breaking policy* (P1, P2, P5, P6, P7) et § *Review Iteration Rule*.
- **KF #283** — 57 clés i18n absentes des locales non-françaises ; ne pas l'aggraver.

## Dev Agent Record

### Agent Model Used

`bmad-dev-story` — **Opus 5 (1M)**, 2026-08-10 (T1-T2) et 2026-08-11 (T3 →).

### Debug Log References

- **T1** — les quatre cas d'AC1 éprouvés **sur le moteur** avant d'écrire la moindre ligne de Rust (deux `NULL` acceptés ; doublon actif refusé en `1062` ; casse refusée, collation `_ci` ; numéro d'un contact archivé réattribuable).
- **T1, garde-fou P6** — arbitrage posé : `total - 25` → `total - 26`. C'est la **frontière (34)** qui est l'invariant voulu, pas la taille de la fenêtre ; garder `-25` aurait déplacé le point de départ du chemin d'upgrade testé **sans que rien ne le signale**. Les deux suites de backfill résolvent par version (`migrations_before`) — vérifié, pas supposé : insensibles à l'ajout.
- **T2** — les 5 compteurs **recomptés depuis le tableau** : `60` fichiers `.sql` = `60` lignes de tableau = en-tête de section = ligne `Total`, partitionnés en `55 tracked-by-sqlx + 5 yes + 0 no`.
- **T3** — `21` colonnes à l'`INSERT` pour `21` placeholders, recomptés par script plutôt qu'à l'œil. `reconciliation.rs:203` interpole bien `super::contacts::COLUMNS` (**vérifié**, comme T3 l'exigeait) ; les quatre `SELECT` de `contacts.rs` interpolent `{COLUMNS}` — **`FIND_BY_ID_SQL` est donc bien la seule liste dupliquée à la main**.
- **T5** — mutation `M4a` : omettre `client_number` d'`impl From<Contact> for ContactResponse` **ne compile pas** (`E0063`). Voir ci-dessous — la spec annonçait l'inverse.
- **T3, rayon d'impact** — `37` insertions dans `26` fichiers, posées par appariement d'accolades sur chaque littéral et **non** par `sed` sur `ide_number:` (d'autres structs portent ce champ : `CreateContactRequest`, `UpdateContactRequest`, `ContactResponse`). `cargo build --workspace --all-targets` propre.

### Ce que l'implémentation a appris, et qui corrige la spec

- **Une SEPTIÈME liste écrite à la main, que la spec ne nommait pas** : le helper de test `contact_to_update` (`repositories/contacts.rs:1500`) reconstitue l'état persisté champ par champ. L'insertion automatique y avait posé `client_number: None` — ce qui aurait fait **perdre le numéro à chaque aller-retour de test**, et rendu vert le test d'AC2-ter pour la mauvaise raison. Corrigé en `c.client_number.clone()`. La spec énumérait six listes ; celle-ci est dans le **module de test**, donc invisible au recensement fait sur le code de production.

- **Une affirmation de la spec RÉFUTÉE à l'exécution.** AC3 et T5 annonçaient : « omettre la ligne dans `From<Contact>` **compile**, stocke, et rend `null` pour toujours », en s'appuyant sur le HIGH de la passe 3 de 16-3a. C'est **faux pour `ContactResponse`** : le littéral de `From` est complet, sans `..Default::default()`, donc l'omission est un `E0063` — le compilateur *vérifie* bien cette couture. La mutation réellement silencieuse est `client_number: None` à la place du champ ; c'est celle qui a été jouée, et le test la tue. L'AC reste juste dans sa conclusion (il faut un test d'aller-retour HTTP), pas dans son motif.

- **AC5 dessinait une séparation que l'écran ne permet pas.** Il posait que la preuve 1 (création) resterait **verte** sous la mutation « hydratation d'`openEdit` retirée », d'où la nécessité de la preuve 2. Mesuré : **les deux tombent**. Motif — la colonne « N° client » de la liste ayant été écartée en passe 6, la boîte d'édition est le **seul** endroit de l'interface où le numéro se relit ; toute preuve de persistance passe donc par l'hydratation. La preuve 2 reste néanmoins distincte par ce qu'elle *mute* (un champ sans rapport, et le `PUT` full-replace qui perd le numéro), et c'est ce qui justifie de la garder.

- **AC6-bis ne tranchait pas le PÉRIMÈTRE de la troncature, et l'étendre aurait été une régression.** Appliquer `META_MAX_CHARS = 32` à **toutes** les lignes du bloc droit couperait « Réf. facture d'origine: F-2026-0042 » (**35** caractères) sur tout avoir français — du contenu existant, en échange d'aucun risque évité : les autres lignes portent des valeurs **engendrées par le système** (n° de facture, date formatée, référence d'origine), bornées par le schéma de numérotation. Le numéro de client est le **seul** champ libre de 50 caractères saisi par l'utilisateur, donc le seul à pouvoir déborder. La borne lui est donc réservée, et un test dédié (`system_generated_meta_lines_are_not_truncated`) fixe ce choix.
- **AC10 portait une clause de preuve devenue insatisfiable, et son abandon n'était pas écrit.** La partie « Preuve » d'AC10 exigeait un « **Test E2E — la colonne est visible dans la liste** », alors que la même AC et les Dev Notes écartent explicitement toute colonne de liste (arbitrage de la passe 6). Les deux ne peuvent pas tenir ensemble : l'implémentation a suivi l'arbitrage — `colspan` laissé à **6** aux deux sites câblés en dur — et la clause est donc **caduque**, non oubliée. La recherche par numéro reste couverte au niveau API par `contacts_are_searchable_by_client_number` (trois termes, dont celui fait uniquement d'opérateurs FULLTEXT). *(Consigné en passe 1 de `bmad-code-review` : c'était la seule clause d'AC sans traitement écrit.)*

- **Le curseur `my` du bloc droit a disparu de `draw_invoice_section`.** Toute la chaîne de décréments vit désormais dans `build_meta_lines`, et rien en aval ne consommait `my` — `clippy` l'a signalé (`value assigned to \`my\` is never read`). C'est une conséquence heureuse de l'extraction : le bloc droit n'a plus deux sources de vérité sur sa géométrie.

### Campagne de mutation — T4

Les trois mutations prescrites par les Conventions de test ont été **exécutées**, pas raisonnées :

| Mutation | Test qui devait mourir | Résultat |
|---|---|---|
| `client_number` retiré de `FIND_BY_ID_SQL` | aller-retour d'AC2 | **FAILED** ✅ |
| `client_number` retiré d'`is_no_op_change` | bump de `version` d'AC2-ter | **FAILED** ✅ |
| `clientNumber` retiré de `contact_snapshot_json` | trace d'audit d'AC2-bis | **FAILED** ✅ |

Restauration contrôlée derrière : 6/6 verts.

### Campagne de mutation — T6

| Mutation | Test qui devait mourir | Résultat |
|---|---|---|
| ligne retirée de `From<Contact>` | aller-retour HTTP | **`E0063`** — attrapée par le compilateur, pas par le test |
| `client_number: None` dans `From<Contact>` | aller-retour HTTP | **FAILED** ✅ *(la mutation réellement silencieuse)* |
| branche retirée de `map_contact_error` | les deux 409 | **FAILED** ✅ |

Restauration contrôlée derrière : 5/5 verts.

### Campagnes de mutation — T7-bis et T8

| Mutation | Test qui devait mourir | Résultat |
|---|---|---|
| une seule des deux branches `LIKE` traitée | recherche par terme fait **uniquement** d'opérateurs FULLTEXT | **FAILED** ✅ (`1 attendu, obtenu 0`) |
| hydratation retirée d'`openEdit` | survie du numéro à l'édition | **FAILED** ✅ — et le `npm run build` reste **vert**, ce qui est tout le propos |

Restaurations contrôlées derrière : 2/2 et 6/6 verts.

### Campagne de mutation — T9 / T10 / T11

| Mutation | Test qui devait mourir | Résultat |
|---|---|---|
| espace consommé alors que la ligne est absente (`y -= META_LINE_STEP` dans le `else`) | conditionnalité du décrément | **FAILED** ✅ — `left: 0.0`, `right: 4.5`, la signature du mutant décrite par AC6 |
| troncature portée sur la **valeur** au lieu de la **ligne complète** | calibrage d'AC6-bis | **FAILED** ✅ |
| l'**avoir** n'est pas renseigné (la facture, si) | AC7, au site de construction | **FAILED** ✅ — et le test jumeau de la facture reste vert, ce qui prouve que les deux sites sont bien discriminés |
| clé `invoice-pdf-client-number` retirée de **de-CH** | AC8, repli silencieux vers le français | **FAILED** ✅ |

⚠️ **Une première mutation a SURVÉCU, et l'erreur était dans la mutation.** `if !out.is_empty() || true` décalait **les deux** cas d'un pas identique : le delta restait 4,5 et le test avait raison de rester vert. La mutation fidèle est celle du tableau. Le fait mérite d'être noté : l'extraction en fonction pure rend la mutation d'origine (« `my -= 4.5` sorti du `if let` ») **structurellement impossible** — le décrément est désormais couplé à l'ajout d'une ligne —, si bien qu'il faut l'écrire à la main pour l'éprouver.

Restaurations contrôlées derrière : `kesh-qrbill` 61/61, `kesh-i18n` 22/22.

### Completion Notes List

- **T1, T2** (2026-08-10) — migration + audit d'idempotence.
- **T13** (2026-08-11) — les **six** sites énumérés par la tâche : liste des champs de création et section dédiée `\label{sec:numero-client}` du manuel utilisateur, phrase sur la recherche (qui **surestimait déjà** le code avant cette story — elle annonçait IDE, adresse et téléphone, que la recherche n'a jamais couverts), `docs/search-patterns.md`, ligne v0.9.0 du README, CHANGELOG, et régénération du PDF (`make fr`, 56 pages). Défaut de doc **préexistant** tracé en **issue #291** plutôt que corrigé au passage : le manuel annonce un « Contacts → Import CSV » qui n'existe ni à l'écran ni en route.
- **T9, T10, T11** (2026-08-11) — `InvoicePdfData.debtor_client_number`, clé `invoice-pdf-client-number` **en fin** des deux tableaux et sur les 4 locales, extraction de `build_meta_lines` (fonction **pure**, sans quoi AC6 est intestable), `META_LINE_STEP` et `META_MAX_CHARS = 32` avec troncature sur la **ligne complète** ; champ renseigné aux **deux** sites de construction. **11** tests neufs (**6** dans `pdf.rs`, 2+2 aux sites, 1 dans `kesh-i18n`), 4/4 mutations tuées.

  ⚠️ **Ce décompte porte sur les tâches T9/T10/T11 seules**, mesurées de `main` au commit de dev — **pas** sur l'écart courant à `main`, que les passes de revue font grossir. Recompte de contrôle : `git show main:<fichier> | grep -c '#\[test\]'` contre le même à `HEAD~n`. *(Décompte corrigé en passe 1 : « 7 » contredisait sa propre ventilation. Puis contesté en passe 2 par une lentille qui avait mesuré jusqu'à `HEAD`, revue comprise, et imputait aux tâches de dev le test ajouté par la passe 1 — d'où cette précision de périmètre, qui lève l'ambiguïté à l'origine des deux erreurs.)*
- **T7, T7-ter, T7-bis, T8** (2026-08-11) — type TS, champ de la fiche, hydratation d'`openEdit`, payload, branche `CLIENT_NUMBER_ALREADY_EXISTS`, placeholder de recherche corrigé ; recherche `OR client_number LIKE ?` sur les **deux** branches ; 3 clés i18n × 4 locales (`contact-*` passe de 59 à **62 dans chacune**, sans dérive). Gate ciblé : `svelte-check` 0 erreur, `lint-i18n-ownership` PASS, Vitest **512/512**, E2E `contact-client-number.spec.ts` **2/2** et `contacts.spec.ts` **8 passed / 2 skipped** (non-régression).
- **T5, T6** (2026-08-11) — DTO d'entrée et de sortie, `normalize_optional`, `MAX_CLIENT_NUMBER_LEN`, branche de `map_contact_error` sur le **nom de contrainte**, variante `AppError::ClientNumberAlreadyExists` → **409 `CLIENT_NUMBER_ALREADY_EXISTS`**. 5 tests E2E API (`contact_client_number_e2e.rs`, nouveau) + 3 tests unitaires dont la **non-sur-capture dans les deux sens** entre les deux contraintes de la table.
- **T3, T3-bis, T3-ter, T4** (2026-08-11) — le champ traverse les **sept** listes ; 6 tests repository, gate ciblé `binary(kesh-db lib)` vert, 3/3 mutations tuées. **Gate complet au push.**

### File List

- `crates/kesh-db/migrations/20260810000001_contacts_client_number.sql` *(nouveau)*
- `docs/migrations-idempotence-audit.md`
- `crates/kesh-db/tests/migrations_upgrade_path.rs`
- `crates/kesh-db/src/entities/contact.rs`
- `crates/kesh-db/src/repositories/contacts.rs`
- `crates/kesh-db/src/repositories/invoices.rs`
- `crates/kesh-api/src/routes/contacts.rs`
- `crates/kesh-api/src/errors.rs`
- `crates/kesh-api/tests/contact_client_number_e2e.rs` *(nouveau)*
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`
- `crates/kesh-i18n/src/loader.rs`
- `crates/kesh-qrbill/src/types.rs`
- `crates/kesh-qrbill/src/pdf.rs`
- `crates/kesh-qrbill/tests/golden_test.rs`
- `docs/manual/fr/user-manual.tex` + `docs/manual/fr/user-manual.pdf`
- `docs/search-patterns.md`
- `README.md`
- `CHANGELOG.md`
- `frontend/src/lib/features/contacts/contacts.types.ts`
- `frontend/src/routes/(app)/contacts/+page.svelte`
- `frontend/tests/e2e/contact-client-number.spec.ts` *(nouveau)*
- `crates/kesh-api/src/routes/credit_notes.rs`
- `crates/kesh-api/src/routes/invoice_email.rs`
- `crates/kesh-api/src/routes/invoice_pdf_service.rs`
- `crates/kesh-reconciliation/src/matching.rs`
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — journal de progression de la story.
- `CLAUDE.md` — deux ajouts de processus portés par cette branche, sans lien avec les AC : la règle « la revue de projet suit la rétrospective » (demande de Guy, commit `de427c62`) et le garde-fou **P8** « une migration appliquée ne se modifie plus » (passe 2 de `bmad-code-review`).
- Fixtures de test mises à jour (littéraux `NewContact` / `ContactUpdate` / `Contact`) : `crates/kesh-db/tests/{credit_notes_repository,invoice_ttc_parity,invoices_line_revenue_account,invoices_validate_vat,kf005_fulltext_index_e2e,payment_batches_repository,reconciliation_repository,supplier_invoices_repository}.rs`, `crates/kesh-api/tests/{admin_full_import_e2e,inbox_import_e2e,invoice_delete_e2e,invoice_echeancier_e2e,invoice_pdf_e2e,invoice_send_email_e2e,reconciliation_e2e,reports_e2e,vat_report_e2e}.rs`, `crates/kesh-report/tests/{aged_receivables,vat_report_reconciliation}.rs`

## Dette technique — deux MEDIUM reclassés par arbitrage du Project Lead

Les deux findings ci-dessous sont sortis de la boucle de revue par **reclassement en dette documentée**, au titre de l'exception de la § *Review Iteration Rule*. Ils ne sont donc **pas** « résolus » : ils sont tracés, avec un propriétaire et une remédiation planifiée. Tous deux portent sur la couche de **comparaison** d'unicité — la seule des cinq couches touchées par cette story (rendu, normalisation, recherche, `is_no_op_change`, snapshot d'audit) à n'avoir reçu ni garde ni décision explicite.

| | Finding | Trace | Pourquoi hors périmètre |
|---|---|---|---|
| **D1** | La table `contacts` ne déclare **aucune collation** — l'une des deux seules du dépôt. La garantie « la casse ne distingue pas », écrite au manuel, dépend donc du défaut de la base à sa création : sous une collation UCA, `CLI-É1` et `CLI-E1` se percutent en 409 ; sous `general_ci`, ils coexistent. Même code, comportements opposés selon l'installation. Et `client_number_uniqueness_is_case_insensitive` passe sous les deux — il donne une confiance qu'il ne mesure pas. | **#295** | L'omission est **antérieure** à la 16-3b (migration d'avril). Fermer demande un `MODIFY COLUMN … COLLATE`, donc le garde-fou **P3** au complet : bump `kesh_version_min_required` **et** bump Cargo de tout le workspace, avec rejeu du gate **runtime** (boot + import de sauvegarde). Hors périmètre d'une story de champ. |
| **D2** | Un caractère invisible **encastré** traverse jusqu'à l'index d'unicité : `CLI-1` et `CLI‹ZWSP›-1` coexistent, **strictement identiques** à l'écran, dans la liste et sur le PDF. La passe 2 a délibérément conservé les valeurs mixtes — sans quoi la garde mangerait du contenu réel mal collé — et l'a verrouillé par un test. | **#294** *(commentaire)* | Rattaché à l'issue qui porte déjà la normalisation NFC des champs de contact : c'est le même problème — *deux valeurs distinctes pour la base, identiques pour l'œil* — sous une autre forme. Les traiter ensemble vaut mieux que deux correctifs séparés ; la solution propre (forme canonique de comparaison) recouvre aussi **#295**. |

**Propriétaire** : Project Lead. **Remédiation** : story dédiée à planifier, de préférence unique pour D1 et D2 — une colonne de comparaison canonique fermerait d'un coup la casse, les accents et la composition Unicode. À revoir au triage de la rétrospective de l'Epic 16.

## Change Log

**2026-08-11 — `bmad-code-review`, PASSE 4 (Sonnet, deux lentilles) — BOUCLE CONVERGÉE.**

| Lentille | CRIT | HIGH | MED | LOW |
|---|---|---|---|---|
| Edge Case Hunter | 0 | 0 | 0 | 0 |
| Acceptance Auditor | 0 | 0 | 0 | 0 |

Critère d'arrêt de la § *Review Iteration Rule* atteint : plus rien au-dessus de `LOW`, et pas un seul `LOW` neuf. Plafond de 8 passes jamais approché.

**Ce qui rend ce zéro crédible**, et le distingue d'un rapport vide : les deux lentilles ont **re-vérifié une par une** les corrections annoncées par la passe 3, avec la contrepartie dans le code — compteurs recomptés à 60 depuis la source, doc-comment rattaché à `draw_invoice_section`, clarification effectivement présente dans `docs/migrations-idempotence-audit.md`, fichier de migration **non retouché** (conforme à P8), justification d'`is_invisible` corrigée, nom de test réel. C'est précisément ce que la passe 3 avait trouvé faux ; cette fois les déclarations tiennent.

L'Acceptance Auditor a également contrôlé que les deux MEDIUM laissés ouverts **n'avaient pas été refermés furtivement** : la migration ne déclare toujours aucune `COLLATE`, et le test de casse ne couvre toujours pas un caractère accentué sous collation UCA.

**Trend de la boucle** — décroissance monotone, et la **nature** des findings s'est déplacée à chaque passe :

| Passe | Modèles | CRIT | HIGH | MED | Où vivaient les défauts |
|---|---|---|---|---|---|
| 1 | Opus ×3 | 0 | 1 *(réfuté au grep)* | 10 | tests muets ou absents |
| 2 | Sonnet ×3 | 0 | **1** | 1 *(réfuté au recompte)* | symptôme propagé à moitié |
| 3 | Haiku + Opus ×2 | 0 | 0 | **5** | **les déclarations**, pas le code |
| 4 | Sonnet ×2 | 0 | 0 | **0** | — |

**Différentiel E2E, suites complètes des deux côtés** : `main` 180 verts / 37 rouges, branche 180 / 39. Les 2 écarts ont été **rejoués** : `product-revenue-account:349` passe au rejeu (son échec était un timeout de `login()`, pas une assertion métier) et `sidebar-navigation:71` est la **KF #287**, documentée comme changeant de camp d'un run à l'autre. Les 2 tests neufs de la story passent. Contrôle d'arithmétique : `180 + 2 neufs + 0 réparés − 2 cassés = 180` — le découpage des logs est fiable, le différentiel est **nul**.

**2026-08-11 — `bmad-code-review`, PASSE 3 (Haiku 4.5 sur la lentille aveugle, Opus 5 sur les deux autres).**

| Lentille | CRIT | HIGH | MED | LOW |
|---|---|---|---|---|
| Blind Hunter (Haiku) | 0 | 0 | 0 | 0 |
| Edge Case Hunter (Opus) | 0 | 0 | 2 | 4 |
| Acceptance Auditor (Opus) | 0 | 0 | 3 | 6 |

⚠️ **Le « 0 finding » de Haiku n'est pas compté comme un signal de convergence** — c'est la 4ᵉ fois sur cet Epic, et le sondage de ses rapports vides a jusqu'ici trouvé un défaut réel à chaque fois. Les deux lentilles Opus ont d'ailleurs rendu 5 MEDIUM sur le même diff.

**Les trois MEDIUM de l'Acceptance Auditor portent tous sur ce que les artefacts DÉCLARENT d'eux-mêmes, pas sur le code livré.** C'est le mode d'échec que cette story nomme le pire du processus, déplacé du code vers le compte rendu :

1. **Le symptôme « compteur de fenêtre périmé » avait TROIS résidus**, quand la passe 1 déclarait l'avoir « grepé sur tout le dépôt, aucun résidu ». Le grep cherchait « 25 dernières » ; le fichier écrit « 25 **restantes** » (l. 112 et 221) et « 24 → 25 » (l. 120). **La déclaration était fausse.** Corrigé, cette fois par un grep sur `\b(25|59)\b` dans tout le fichier — les occurrences restantes sont des références historiques légitimes. Généalogie de `N` prolongée (`25 → 26`), que T1 exigeait et qui manquait.
2. **La passe 2 n'avait aucune entrée ici**, alors qu'elle a modifié `normalize_optional`, helper **partagé** avec `email`, `phone`, `default_payment_terms` et l'e-mail de compte (`users.rs:116`). Lu seul — comme le liront la passe suivante et la rétrospective —, ce fichier ne disait pas qu'un comportement hors périmètre nominal avait changé. Réparé ci-dessous.
3. **Une clarification annoncée « déplacée hors du fichier » ne l'avait été nulle part.** L'en-tête de la migration dit « numéro de client **de l'émetteur** », ce qui décrit la 16-3a ; le numéro porte sur le **destinataire**. Le fichier étant appliqué, P8 interdit de le retoucher — la clarification vit désormais dans `docs/migrations-idempotence-audit.md`, comme annoncé.

**LOW corrigés** : le doc-comment de `draw_invoice_section` était devenu **orphelin** (l'extraction de `build_meta_lines` s'était insérée entre lui et sa fonction — rustdoc l'attribuait au mauvais symbole ; deux passes avaient lu ce hunk sans le voir) ; la justification de la duplication d'`is_invisible` invoquait des « crates sans dépendance commune », ce qui est **faux** (`kesh-api/Cargo.toml:12` dépend de `kesh-qrbill`) ; et le story file citait un test `client_number_is_searchable` qui **n'existe pas** (le vrai est `contacts_are_searchable_by_client_number`).

**Deux MEDIUM de l'Edge Case Hunter restent OUVERTS et demandent un arbitrage du Project Lead** — ils portent sur la couche de **comparaison** d'unicité, seule des cinq couches touchées à n'avoir reçu ni `is_invisible` ni décision explicite :

- **M1 — la table `contacts` ne déclare aucune collation** (l'une des deux seules du dépôt dans ce cas, sur soixante migrations). La garantie « la casse ne distingue pas », écrite au manuel, repose donc sur le défaut de la base à sa création. Sous une collation UCA (`utf8mb4_unicode_ci`, ou `uca1400_ai_ci` sur MariaDB 11.x), les `_ci` sont **accent-insensibles** : `CLI-É1` et `CLI-E1` se percutent en 409 sur une installation et coexistent sur une autre. Le test `client_number_uniqueness_is_case_insensitive` passe sous les deux — il donne une confiance qu'il ne mesure pas. **Fermer demande un `MODIFY COLUMN … COLLATE`, donc le garde-fou P3** (bump `min_required` + bump Cargo). Hors périmètre d'une story de champ.
- **M2 — un caractère invisible ENCASTRÉ traverse jusqu'à l'index d'unicité.** La passe 2 a délibérément conservé les valeurs mixtes (et l'a verrouillé par un test) ; la conséquence est que `CLI-1` et `CLI‹ZWSP›-1` peuvent coexister, **strictement identiques à l'écran, dans la liste et sur le PDF**. Fermer suppose de décider si l'on **filtre** les invisibles à la saisie ou si l'on stocke une forme canonique de comparaison — un arbitrage de spec, pas de revue.

**2026-08-11 — `bmad-code-review`, PASSE 2 (Sonnet, trois lentilles).**

| Lentille | CRIT | HIGH | MED | LOW |
|---|---|---|---|---|
| Blind Hunter | 0 | 0 | 0 | 2 |
| Edge Case Hunter | 0 | **1** | 0 | 0 |
| Acceptance Auditor | 0 | 0 | 1 | 1 |

**HIGH — le symptôme de la passe 1 n'avait été propagé qu'à MOITIÉ.** La garde de vacuité avait été posée côté **rendu** (`pdf.rs`) et pas côté **normalisation d'entrée**. `normalize_optional` ne faisait que `trim()`, et `White_Space` n'inclut pas les caractères de largeur nulle : un numéro collé depuis un tableur avec un `U+200B` paraît vide à l'écran, est stocké comme une valeur ordinaire, et la **deuxième** fiche « vide » se prend un 409 sur une valeur que l'utilisateur ne peut ni voir ni effacer — exactement ce qu'`empty_client_number_is_stored_as_null_and_never_collides` promet d'empêcher. Corrigé, avec les **deux** bords testés (invisible seul → `None` ; invisible **mêlé** à du visible → inchangé, sinon la garde mangerait du contenu réel) et la mutation jouée.

⚠️ **Ce correctif touche un helper partagé, et il faut le dire.** `normalize_optional` sert aussi `email`, `phone`, `default_payment_terms` (`contacts.rs`) et l'e-mail de compte (`users.rs:116`, donc `POST /setup/admin`). Une valeur intégralement invisible y rendait un **400** de format ; elle rend désormais `None`, soit un effacement accepté. L'Acceptance Auditor de la passe 3 a arbitré ce point comme une **correction légitime et non une dérive** — le défaut était celui du helper, dont le nom promet `None` sur une valeur vide —, mais relève que ces appelants ne sont couverts par aucun test et que deux helpers homonymes (`products.rs:156`, `invoices.rs:369`) portent la même faiblesse sans index unique derrière. À traiter avec **#294**, qui porte déjà la normalisation des champs de contact.

**MEDIUM réfuté au recompte.** La lentille mesurait l'écart à `main` jusqu'à `HEAD`, revue comprise, et imputait aux tâches de dev le test ajouté par la passe 1. Recompté aux trois états : de `main` au commit de dev, `pdf.rs` gagne bien **+6** tests et l'ensemble **11**. L'ambiguïté de périmètre qui a produit l'erreur est levée dans la Completion Note.

**INCIDENT — la suite E2E de la branche ne démarrait plus, et le gate ne pouvait pas le voir.** Le correctif de passe 1 avait reformulé l'en-tête de la migration `20260810000001`, ce qui change son **checksum sqlx** : le backend refuse alors de démarrer sur toute base l'ayant déjà appliquée, **y compris la base de dev**. Le correctif a été annulé — la migration est bit-à-bit celle de son commit d'origine — et le garde-fou **P8** codifié dans CLAUDE.md. Les tests `#[sqlx::test]` recréant une base neuve, le gate backend était vert à 2158/2158 pendant que le démarrage réel était cassé : seul un boot contre une base **persistante** révèle ce défaut.

**2026-08-11 — `bmad-code-review`, PASSE 1 (Opus 5, trois lentilles en parallèle).**

Gate backend complet joué **avant** les patches, sur l'état livré : **2158/2158**, 4 ignorés, 0 échec.

| Lentille | CRIT | HIGH | MED | LOW |
|---|---|---|---|---|
| Blind Hunter (diff seul, sans spec) | 0 | 1 | 7 | 7 |
| Edge Case Hunter (diff + dépôt) | 0 | 0 | 2 | 2 |
| Acceptance Auditor (diff + spec) | 0 | 0 | 1 | 7 |

**Le HIGH est tombé au grep ground-truth.** « Un second appelant de `PUT /contacts/{id}` effacerait le numéro » — la lentille le donnait elle-même pour indécidable à l'aveugle. Vérifié : un seul émetteur (`contacts.api.ts:50`), et surtout `contacts.types.ts:108` documente que **tout** champ absent d'un `PUT` est effacé. C'est le contrat établi de l'endpoint, pour tous ses champs optionnels, pas un défaut introduit ici — et il est déjà tracé en **#278**.

**Un MEDIUM réfuté par une autre lentille.** Le Blind Hunter soupçonnait la colonne générée de casser l'import `.keshbackup` ; l'Acceptance Auditor a vérifié que `non_generated_columns` (`backup.rs:100`) filtre `EXTRA NOT LIKE '%GENERATED%'` — la colonne sort du backup toute seule. C'est l'intérêt d'avoir une lentille aveugle **et** une lentille avec accès au dépôt.

**Les deux MEDIUM de l'Edge Case Hunter sont les trouvailles de la passe**, et toutes deux portent sur un **symptôme non propagé** :

1. **`build_meta_lines` testait la nullité, pas la vacuité** — alors que le site **jumeau** du bloc gauche, posé par la 16-3a, porte ce filtre avec un commentaire décrivant exactement ce mode d'échec. Et le chemin est atteignable par l'API publique : `str::trim` suit la propriété Unicode `White_Space`, qui **n'inclut pas** les caractères de largeur nulle — `U+200B`, `U+FEFF`, `U+2060` traversent `normalize_optional` et arrivent au PDF. Une facture portait alors « N° client: » suivi de rien, en consommant un `META_LINE_STEP` qui décalait la date, la référence d'origine et l'échéance. Corrigé par un prédicat `is_invisible` et un test à six valeurs, **mutation jouée** (`is_invisible` neutralisé → le test tombe sur le cas ZWSP).
2. **La justification qui exclut les autres lignes de la troncature reposait sur une prémisse fausse.** Le doc-comment affirmait que le numéro de facture était « borné par le schéma de numérotation » ; ce schéma est un champ **libre** de *Paramètres → Facturation*, rendu jusqu'à `MAX_RENDERED_LEN = 64` (`invoice_format.rs:25-27`) — soit **plus** que les 50 du numéro de client. Le débordement est réel mais **antérieur** à cette story, et le borner à 32 couperait « Réf. facture d'origine: … » sur tout avoir français. Justification réécrite avec le vrai motif, test dé-figé de la fausse prémisse, et défaut tracé en **#293**.

**Cinq MEDIUM du Blind Hunter portaient sur des tests absents ou muets**, tous confirmés au grep et corrigés :

- `empty_client_number_…` lisait le corps **sans vérifier le statut** — un `400` de validation aurait rendu le test vert en ayant cessé de mesurer ;
- `client_number_conflict_renders_409_with_its_own_code` n'assertait **jamais** la chaîne du code, que le frontend consomme littéralement — l'assertion vit désormais dans `errors.rs`, seul endroit où le corps est lisible, et le test d'origine porte un nom honnête ;
- **aucun test multi-société** ne distinguait `UNIQUE (company_id, client_number_uniq)` d'un `UNIQUE` global — ajouté, avec extraction d'un helper `create_company_with_user` ;
- **aucun test de longueur** ne couvrait la borne des 50 caractères — ajouté, aux deux bords (51 refusé, 50 accepté) ;
- **compteurs de migration périmés à quatre endroits** de `migrations_upgrade_path.rs` (59 et 25 au lieu de 60 et 26). C'est le seul MEDIUM que l'Acceptance Auditor avait vu aussi. Ironie utile : ce fichier documente aux lignes 61-67 que la même dérive y a déjà été corrigée en passe 5 de la 16-1a. Symptôme grepé sur tout le dépôt après correction : aucun résidu.

**Deux LOW tracés plutôt que corrigés** : pas de normalisation NFC sur les champs du contact (**#294** — corriger toucherait `email`, `phone` et `ide_number`, hors périmètre), et l'ambiguïté d'un numéro recyclé visible en double quand « Inclure archivés » est coché (documenté dans le manuel utilisateur, comportement voulu).

**Deux LOW de l'Acceptance Auditor étaient déjà consignés** dans « Ce que l'implémentation a appris » (périmètre d'AC6-bis, forme SQL de D2-bis) — dismiss.

**2026-08-10 — BOUCLE `validate` ARRÊTÉE À LA PASSE 6 — arbitrage du Project Lead, dérogation motivée au critère d'arrêt.**

Le critère de la § *Review Iteration Rule* (« uniquement des `LOW` ») **n'est pas atteint** : la passe 6 a rendu 1 HIGH et 5 MEDIUM, tous remédiés. Le plafond de 8 passes n'est pas non plus atteint. **L'arrêt est donc une dérogation, et voici ce qui la motive.**

**Le rendement des passes est devenu négatif en information neuve.** Sur les six passes, la source des findings s'est déplacée :

| Passe | Findings > LOW | Dont défauts **introduits par une passe antérieure** |
|---|---|---|
| 1 | 3 | — |
| 2 | 9 | 0 |
| 3 | 1 | 0 |
| 4 | 10 | **2** (D2-bis et AC6, tous deux patchés en P2) |
| 5 | 4 | **2** (propagation P4 non faite, chiffre P5 faux) |
| 6 | 6 | **1** (borne de troncature posée en P2) |

Les passes 5 et 6 ont surtout réparé les réparations. C'est le constat déjà mesuré sur le lot 16-1 (« au moins 12 findings sur 28 corrigent un artefact produit par une passe antérieure ») et la § *Propagation post-patch* du `CLAUDE.md` le nomme explicitement comme mode d'échec récurrent du processus.

**Trois signaux convergents plaident pour l'arrêt** : (a) la lentille de synthèse de la passe 6, seule à qui un **verdict** ait été demandé, conclut que le socle est exact et recommande de ne pas enchaîner ; (b) toutes les ancres re-contrôlées à la passe 6 tiennent, sur une quarantaine de vérifications indépendantes ; (c) les deux dernières lentilles Haiku ont rendu « 0 finding » — signal faible pris isolément, mais cohérent avec le reste.

**Ce que l'arrêt ne prétend PAS.** Il ne déclare pas la spec exempte de défauts. Il constate que la **prochaine** passe a plus de chances de corriger un patch de la sixième que de trouver un défaut d'origine. Le filet suivant est la boucle `bmad-code-review` après implémentation, qui travaillera sur du **code exécutable** — donc sur des faits que la revue de spec ne peut pas atteindre.

**Trend complet** : `1H/2M/2L` → `4H/5M/2L` → `1M` → `1C/4H/5M/3L` → `2H/2M/3L` → `1H/5M/3L`. Rotation Opus → Sonnet → Haiku → Opus → Sonnet → Haiku+Opus, 11 lentilles en contexte frais. **Deux dérogations arbitrées par Guy** : garde-fou de splitting (passe 4) et critère d'arrêt (passe 6).

**Bilan de la boucle** : 9 critères et 13 tâches au départ, **13 critères et 16 tâches** à l'arrivée — une décision renversée (D2-bis), deux modes d'échec **silencieux** fermés que la spécification initiale ne voyait pas (`is_no_op_change`, qui perdait la donnée ; l'hydratation d'`openEdit`, qui l'effaçait à chaque édition), un critère intestable rendu testable (AC6), et un débordement hors page évité (AC6-bis).

**2026-08-10 — Passe 6 de `bmad-create-story validate`** (**Haiku** — audit mécanique des ajouts récents — et **Opus** — jugement de synthèse, contextes frais). **1 HIGH, 5 MEDIUM, 3 LOW**, tous vérifiés au ground-truth et remédiés. **Boucle NON convergée — passe 7 due.**

⚠️ **La lentille Haiku a rendu « 0 finding » et déclaré la story prête. Le sondage de son rapport a montré qu'elle se trompait, pour la SECONDE fois de cette boucle.** Elle annonçait 14 champs à `contact_snapshot_json` (il y en a **16**) et n'avait vu **qu'une** définition d'`apply_migrations_up_to` (il y en a **deux**, `tests/common/mod.rs:50` et `tests/migrations_upgrade_path.rs:30`). Ce sondage a surtout révélé un manque de la story elle-même — voir MEDIUM-1. Le constat du sprint-status vaut donc une troisième fois : *un « 0 finding » de Haiku n'est pas une preuve de convergence*.

**HIGH — la borne de troncature d'AC6-bis portait sur la valeur, alors qu'elle doit porter sur la LIGNE COMPLÈTE.** Le patron du bloc gauche tronque la ligne **formatée** : `truncate_display(&format!("{}: {}", i18n.get(key), v), IDENTITY_MAX_CHARS)` (`pdf.rs:271`). Les 32 caractères sont un budget de **largeur** et couvrent donc le libellé. Écrire `truncate_display(&client_number, 32)` produit une ligne d'environ 43 caractères ≈ 92 mm depuis `x = 120`, soit `x ≈ 212` sur une page de **210 mm** — du texte **hors feuille**. Gravité particulière : c'est une spécification dont **l'exécution fidèle recrée le défaut qu'elle prévient**, et l'AC établit lui-même que le test de calibrage ne peut pas le voir. Conséquence désormais écrite : la valeur ne dispose que d'une vingtaine de caractères une fois le libellé décompté.

**MEDIUM-1 — T1 taisait un fait dur : `upgrade_path_preserves_data` VA ÉCHOUER.** `migrations_upgrade_path.rs:89-94` porte `assert_eq!(total, 59, …)`, et le message de `apply_migrations_up_to` (l. 30-47) explique que cet échec est **son rôle** : il force à décider si la fenêtre `total − 25` s'élargit ou si la frontière reste à 34. T1 disait seulement « inspecter chaque site ». Elle porte désormais le relevé complet — deux définitions de la fonction, la généalogie en commentaire à prolonger, l'arbitrage à poser — et le constat que les deux suites de backfill résolvent **par version** (`migrations_before(<version>, <nom>)`), donc insensibles à l'ajout.

**MEDIUM-2 — AC8 n'avait aucune preuve pour le libellé PDF en de-CH / it-CH / en-CH.** `loader.rs:130-135` charge d'abord **toutes les clés `fr-CH` comme base de repli** des autres locales : une clé seulement française rend le libellé **français sur un PDF allemand**, en silence, et `DEFAULT_EN` n'est **jamais atteint en production**. La preuve d'appariement positionnel ne voit pas ce cas — elle teste un autre mécanisme. Un PDF de-CH aurait imprimé « N° client » avec tous les tests au vert : la KF #283 appliquée à l'artefact qui donne son titre à la story. Preuve étendue aux **deux** clés.

**MEDIUM-3 — T7-bis ne visait qu'un des deux sites `email LIKE`** (l. **174** et **181** ; la première branche sert quand le terme échappé est vide). N'en traiter qu'un compile et passe les tests, mais **cesse silencieusement de chercher le numéro** quand le terme n'est fait que d'opérateurs FULLTEXT.

**MEDIUM-4 — T12 laissait un arbitrage ouvert dans un document destiné à être déclaré prêt** (« export CSV, optionnel, à trancher »). **Tranché : exclu**, avec le motif que la story fournissait déjà — cet export est *déjà partiel*. T12 disparaît, la justification passe en « ce que la story ne doit PAS faire ». Une tâche de moins, et une classe entière de findings « avez-vous voulu… ? » fermée d'avance.

**MEDIUM-5 — la colonne « N° client » dans la liste des contacts est un ajout d'interface non arbitré** : le bénéfice annoncé par le « so that » est déjà servi par la **recherche** seule. Coût réel et disproportionné (le `colspan` câblé à deux endroits, un E2E, une clé sur 4 locales). **Soumis à l'arbitrage du Project Lead** — voir la question posée en fin de passe.

**Les 3 LOW, tous de forme** : environ 13 % de la partie prescriptive était de l'**archéologie** — des avertissements expliquant des tests qu'il ne faut *pas* écrire —, concentrée précisément sur **AC6**, le critère le plus difficile à exécuter ; quatre paragraphes y ont été **élagués** et la leçon des trois erreurs de calcul déplacée en Dev Notes comme règle sèche. Les ancres `+page.svelte:` ne donnaient **jamais** le chemin, or le groupe de routes entre parenthèses le rend non devinable — **cinq** sites complétés en `frontend/src/routes/(app)/contacts/+page.svelte`. Et « le DTO d'entrée » était au singulier pour **deux** structs (`CreateContactRequest` l. 70, `UpdateContactRequest` l. 108).

**Verdict de la lentille de synthèse** : socle **exact** — toutes les ancres re-contrôlées tiennent —, décisions argumentées, ordonnancement implémentable. Elle recommande de **ne pas** enchaîner une septième passe une fois ces points traités : le rendement décroît, et le mode d'échec dominant sur ce document est désormais la remédiation elle-même.

**2026-08-10 — Passe 5 de `bmad-create-story validate`** (**Sonnet**, contexte frais, 2 lentilles — audit des remédiations, implémentabilité littérale). **2 HIGH, 2 MEDIUM, 3 LOW**, tous vérifiés au ground-truth et remédiés. **Boucle NON convergée — passe 6 due.**

La passe visait délibérément le mode d'échec mesuré comme dominant : **les patches des passes antérieures**. Elle l'a confirmé — les deux HIGH sont des défauts **introduits par les remédiations des passes 2 et 4**.

**HIGH-1 — la correction du HIGH-2 de la passe 4 n'avait pas été propagée à sa propre copie.** La passe 4 avait remplacé, dans AC6, l'assertion inversée « ordonnées identiques » par `y_none − y_some == 4.5`. Mais **T11 portait encore le libellé fautif** — et T11 est justement la liste que `bmad-dev-story` suit le plus directement. Le patch qui corrigeait un défaut de remédiation a lui-même souffert d'un défaut de propagation. C'est le § *Propagation post-patch* du `CLAUDE.md` appliqué à lui-même.

**HIGH-2 — la formulation de repli d'AC6 contenait une TROISIÈME erreur de calcul sur le même bloc.** Elle proposait « en `None`, ordonnée échéance exactement 265,5 ». Recalcul depuis la chaîne de décréments (`pdf.rs:287-339`) sur le fixture par défaut (`origin_reference: None`) : `277` → `270` (n° facture) → **`265,5` (date)** → `−4,5` → **`261,0` (échéance)**. **`265,5` est la position de la date.** Un développeur suivant cette variante aurait écrit une assertion qui **échoue sur du code correct** — le mode d'échec que cet AC dénonce. La variante a été **retirée** plutôt que corrigée : même juste, une ordonnée absolue dépend du fixture, alors que le delta est invariant.

⚠️ **Trois erreurs de calcul sur ce seul bloc PDF** (`252` vs `256,5` ; le décompte des décréments de `my` ; `265,5` vs `261,0`), toutes du même type : un chiffre posé sans être dérivé du code. La consigne est désormais écrite dans AC6 — *ne jamais écrire une ordonnée sans la recalculer depuis `pdf.rs:287-339`*.

**MEDIUM-1 — une preuve citée qui ne reproduisait pas son résultat.** D2-bis et le Change Log citaient `grep -rniE "unarchive|restore_contact|reactivate"` → 0 ligne. Sans périmètre de fichiers, cette commande rend **84** lignes : `accounts`, `projects` et `reconciliation_rules` ont bien des routes de réactivation. La conclusion (aucune pour les *contacts*) tient, mais une preuve qui ne se reproduit pas n'est pas une preuve. Commande bornée aux trois fichiers concernés, **aux deux sites** où elle figurait.

**MEDIUM-2 — AC6-bis laissait deviner sa constante, et son test ne peut pas la valider.** La valeur est désormais **donnée** — `70 / (9 × 0,677 × 0,3528) ≈ 32` caractères, par la méthode de calibrage d'`IDENTITY_MAX_CHARS` — avec l'avertissement que le test de calibrage prouve la **cohérence** de la troncature, jamais qu'elle **tient dans la largeur** : une constante trop généreuse y passerait sans être vue.

**Les 3 LOW** : `colspan="6"` est câblé à **deux** endroits (l. 537 *chargement*, l. 540 *liste vide*), n'en corriger qu'un laisserait 6 colonnes pour 7 ; la **position d'insertion** de la ligne dans le bloc n'était pas fixée et aucun test ne la contraint — désormais « entre N° facture et Date » ; et `Contact` lui-même n'a pas de `Default`, ce qui ajoute au moins 4 fixtures au rayon d'impact annoncé.

**Quatre soupçons INFIRMÉS, ce qui a autant de valeur** : un `ALTER TABLE` **peut** ajouter en une instruction une colonne générée référençant une colonne créée par la même instruction — `accounts_role_postable.sql:74-105` le fait exactement ainsi, le renvoi de T1 suffit donc ; la recherche est construite par `QueryBuilder`, l'ajout d'une branche `LIKE` est trivial et sans ordre de `bind` à respecter ; `archive()` (`contacts.rs:524-529`) existe déjà, le test du 4ᵉ cas d'AC1 s'écrit sans rien construire ; et l'extraction de la fonction pure de T9 est **mécanique** — les quatre lignes du bloc partagent police et idiome, seul le titre en diffère et reste hors extraction.

**2026-08-10 — Passe 4 de `bmad-create-story validate`** (**Opus 5**, contexte frais, lentille « remise en cause des décisions »). **3 HIGH, 2 MEDIUM, 2 LOW**, tous vérifiés au ground-truth et remédiés. **Boucle NON convergée — passe 5 due.**

**C'est la première passe à contester une DÉCISION, et elle en renverse une.** Les passes 1 à 3 avaient corrigé des ancres, des décomptes, des clauses de preuve et des sites oubliés — aucune n'avait interrogé D1-D5. Le mandat de celle-ci était exclusivement là.

**HIGH-1 — D2-bis reposait sur une échappatoire qui n'existe pas, et le dépôt avait déjà jugé ce cas.** La décision « verrou permanent » se justifiait par « vider le champ sur l'archive libère le numéro ». Or **un contact archivé n'est pas modifiable** : `repositories/contacts.rs:421-426` rend `IllegalStateTransition("impossible de modifier un contact archivé")`, doublé du `WHERE … AND active = TRUE` de l'`UPDATE` (l. 457) ; et **aucune route de désarchivage n'existe** (`grep -rniE "unarchive|restore_contact|reactivate" crates/kesh-api/src/routes/contacts.rs crates/kesh-db/src/repositories/contacts.rs crates/kesh-db/src/entities/contact.rs` → 0 ligne ; ⚠️ **le périmètre de fichiers est indispensable** — la même commande lancée sur `crates/` entier rend **84** lignes, car `accounts`, `projects` et `reconciliation_rules` ont bien, eux, des routes de réactivation). Le numéro était donc perdu **à vie**, déblocable seulement par un `UPDATE` SQL manuel. Surtout, `20260722000001_accounts_role_postable.sql:35-38` avait déjà tranché le cas jumeau **par écrit** : « *le `active AND` n'est PAS cosmétique : sans lui, un compte archivé squatterait son rôle à vie… 409 permanent causé par un compte mort* ». **D2-bis est renversée** : unicité partielle par colonne `GENERATED … VIRTUAL`, troisième application d'un patron éprouvé deux fois.

**HIGH-2 — la clause de preuve d'AC6 assertait la signature du mutant qu'elle prétendait tuer.** La passe 2 avait imposé de tester « l'ordonnée de la ligne échéance **identique** entre `Some` et `None` ». C'est **l'inverse** : avec un code correct, `Some` est 4,5 mm plus bas ; c'est la **mutation** (`my -= 4.5` hors du `if let`) qui rend les deux égales. Suivie à la lettre, l'assertion aurait forcé à écrire un test rouge sur du bon code, puis à **implémenter la mutation** pour le verdir. Remplacée par `y_none − y_some == 4.5`, ou l'ordonnée absolue en `None` — la vraie parade de 16-3a mesure l'absence contre un **absolu**, jamais deux générations l'une contre l'autre. ⚠️ **Ce défaut a été introduit par la remédiation de la passe 2** : la remédiation reste la première source de défauts, comme le lot 16-1 l'avait mesuré sur 7 passes.

**HIGH-3 — une SIXIÈME liste, et c'est la seule qui perde la donnée.** `is_no_op_change` (`repositories/contacts.rs:377-396`) énumère 18 champs à la main et **court-circuite l'écriture** (l. 439-442 : `rollback` + `Ok(before)`). Sans `client_number`, **le geste central de la story** — ouvrir une fiche pour lui attribuer son numéro, sans rien changer d'autre — est classé « aucun changement » : pas d'`UPDATE`, pas d'audit, et l'API rend **`200 OK` avec l'ancienne valeur**. L'utilisateur voit son numéro disparaître au rechargement, sans erreur. Nouvel **AC2-ter** + **T3-ter**, avec le contrôle de `version` pour distinguer une écriture réelle d'un `Ok(before)`.

**MEDIUM-1 — le raisonnement de D2-bis était inversé.** Il invoquait D5 (« la valeur est résolue à la génération ») pour justifier le verrou. Vérifié : le PDF n'est **jamais stocké** (aucun blob dans les migrations) et est régénéré à chaque téléchargement comme à chaque envoi d'e-mail ; une facture désigne son contact par **clé étrangère**, le système ne résout jamais numéro → contact. Libérer un numéro ne crée donc **aucune** ambiguïté interne. D5 était un argument **contre** le verrou.

**MEDIUM-2 — la moitié du « so that » n'avait aucun critère.** La story promet « retrouver un contact depuis une facture papier ». La recherche ne couvre que `name` (FULLTEXT) et `email` (`LIKE`) ; la liste affiche l'IDE mais pas le numéro. Nouvel **AC10** + **T7-bis**, avec la bonne technique donnée par le dépôt lui-même : `OR client_number LIKE ?` **à côté** d'`email`, pas dans l'index FULLTEXT — les séparateurs cassent les tokens (`contacts.rs:162-165`).

**LOW** — l'unicité est **insensible à la casse** (collation `_ci` héritée, aucune déclarée sur `contacts`) : souhaitable, mais à décider et à tester, d'où le 3ᵉ cas d'AC1 ; et le 409 n'était pas diagnosticable quand le détenteur pouvait être un contact archivé donc invisible — **ce point disparaît avec le renversement de D2-bis**.

**Seconde lentille de la passe 4 (complétude) — 1 CRITICAL, 2 HIGH, 3 MEDIUM, 1 LOW.** Son CRITICAL est **le même que HIGH-1 ci-dessus** : deux lentilles indépendantes ont convergé sur l'échappatoire inexistante de D2-bis, ce qui en confirme la réalité. Trois findings neufs :

- **HIGH — le chemin d'ÉDITION efface le champ, et AC5 ne pouvait pas le voir.** `PUT /contacts/{id}` est un **full-replace** : `frontend/src/routes/(app)/contacts/+page.svelte:336-341` envoie le même `payload` pour créer et pour modifier, et `openEdit` (l. 240-258) hydrate le formulaire **champ par champ**, sur dix-huit lignes. Ajouter le champ au payload sans ajouter sa ligne d'hydratation ne casse **aucune compilation** — et modifier un simple téléphone **efface le numéro de client**. AC5 ne testait que la création : il serait resté **vert** sous cette mutation. Seconde preuve ajoutée (« modifier un champ sans rapport, le numéro survit »).
- **MEDIUM — AC4 créait un code d'erreur que personne n'affiche.** Le frontend branche les codes **un par un** (`frontend/src/routes/(app)/contacts/+page.svelte:349-358`) et **retombe sur `err.message` brut** hors des cas connus. Le doublon d'IDE a son message traduit ; le nôtre n'en avait aucun. **T7-ter** ajoute la branche et la clé sur les 4 locales.
- **MEDIUM — T13 ne visait qu'une section de manuel sur quatre sites.** Manquaient la liste des champs de création (`user-manual.tex:510-520`, la page que lira l'utilisateur), la phrase sur la recherche (l. 531 — qui **surestime déjà** le code), `docs/search-patterns.md:116` et `README.md:213`, puisque cette story **ferme #151**. T13 les énumère désormais.

**Et une liste d'angles ouverts puis rendus vides** — portes d'entrée d'un contact (il n'y en a qu'une), fixtures CI, import d'une sauvegarde antérieure, rappels, `api-external.md`, site web — consignée dans les Dev Notes pour que les passes suivantes ne refassent pas le trajet. Un défaut de doc **préexistant** a été relevé au passage, hors périmètre : `user-manual.tex:533-535` annonce un import CSV de contacts qui n'existe pas.

⚠️ **GARDE-FOU DE SPLITTING FORMELLEMENT DÉCLENCHÉ — arbitrage Project Lead requis.** Sévérité maximale par passe : **P1 HIGH → P2 HIGH → P3 MEDIUM → P4 CRITICAL**. La § *Règle de splitting préventif* définit comme non-convergence réelle une passe `N+1` de sévérité **égale ou supérieure** à `N` ; la remontée `MEDIUM → CRITICAL` la déclenche sans ambiguïté.

**Analyse pour l'arbitrage — ce qui plaide contre le split.** Le CRITICAL ne porte sur aucune difficulté intrinsèque de la story : il porte sur **D2-bis, décision introduite par la remédiation de la passe 2 et jamais reconfrontée au code depuis**. C'est un patch resté **sur parole** pendant deux passes, pas un symptôme de saturation contextuelle. Le même diagnostic vaut pour le HIGH sur AC6, dont l'assertion inversée a elle aussi été **introduite en passe 2**. Deux des quatre HIGH de cette passe sont donc des **défauts de remédiation**, ce que le lot 16-1 avait déjà mesuré sur 7 passes (« au moins 12 findings sur 28 corrigent un artefact produit par une passe antérieure »).

Le précédent applicable est la **16-2b**, où le garde-fou s'était déclenché et où la dérogation, arbitrée puis **validée par le résultat**, avait vu le compteur tomber. Et la 16-3a elle-même, dont la dérogation a été maintenue aux passes 2, 6 et 7 avant de converger à zéro en 8 passes.

✅ **ARBITRAGE — dérogation accordée par Guy (Project Lead), 2026-08-10.** La boucle continue sans split. Le motif retenu est celui de l'analyse ci-dessus : deux des quatre HIGH sont des **défauts de remédiation** et non de largeur de story, et le découpage naturel (backend / frontend) serait de surcroît coupé en travers par **AC10**, dont la recherche traverse les deux moitiés. La passe 5 cible donc en priorité **l'audit des patches des passes antérieures**, mode d'échec désormais mesuré comme dominant sur cette story.

**Trois décisions confirmées, avec des arguments plus forts que les miens.** **D1** (saisie manuelle) : le dépôt a bien des séquences par société, mais clées `(company_id, fiscal_year_id)` avec garantie « sans trou » consommée en transaction de validation — aucune de ces propriétés n'a de sens ici ; et surtout le « so that » appelle une **référence imposée par le client**, qu'aucune séquence ne peut produire. **D3** (bloc droit) : `pdf.rs:342` pose le bloc destinataire à 55 mm du bord, soit **la fenêtre de l'enveloppe** — y insérer une mention non postale serait fautif, le bloc métadonnées à `meta_x = 120` est hors fenêtre. Motif matériel que la story n'avait pas su nommer. **D5** (pas de dénormalisation) : l'hypothèse d'une incohérence est **réfutée** — `invoice_pdf_service.rs:287-288` résout déjà `debtor_name` et `debtor_address_lines` à la volée, et `entities/invoice.rs` ne porte aucun champ de destinataire hors `contact_id`. Dénormaliser le seul numéro en aurait fait le **seul** attribut figé du destinataire : *c'est cela* qui aurait été l'incohérence.

**2026-08-10 — Passe 3 de `bmad-create-story validate`** (**Haiku 4.5**, contexte frais, 2 lentilles — cohérence interne, faisabilité technique). **0 CRITICAL, 0 HIGH, 1 MEDIUM, 0 LOW**, remédié. **Boucle NON convergée — passe 4 due.**

⚠️ **Les DEUX lentilles ont rendu « 0 finding ». Le MEDIUM a été trouvé par l'orchestrateur en vérifiant leurs rapports.** C'est très exactement l'avertissement inscrit au sprint-status après le lot 16-1 — *« un 0 finding de Haiku n'est PAS une preuve de convergence »*, constaté alors sur 4 rapports vides. Le cas se reproduit, et sous une forme instructive : la lentille « faisabilité » **avait vu** `contact_snapshot_json` et l'a écarté d'une ligne — « partiel par design, non mentionné dans la story, donc non critique ». C'est le classement qui était faux, pas l'observation.

**Le MEDIUM — une cinquième liste, non-SQL, que la story ne voyait pas.** `contact_snapshot_json` (`repositories/contacts.rs:39-58`) énumère à la main les champs versés à l'audit log. La story revendiquait l'exhaustivité de « quatre listes de colonnes » — vrai pour le **SQL**, et c'est d'ailleurs ce que la passe 2 avait confirmé. Mais cette cinquième liste porte déjà `ideNumber`, et `client_number` lui est **fonctionnellement jumeau** : un identifiant du contact, unique par société. L'omettre laissait l'audit aveugle à sa modification, **sans rien casser ni faire échouer la compilation** — le mode d'échec silencieux que toute cette story s'emploie à fermer ailleurs. Nouvel **AC2-bis** + **T3-bis**.

**Ce que la passe a confirmé, et qui a de la valeur** : la propagation des patches des passes 1 et 2 est **complète** — les corrections `422 → 409`, `ContactJson → ContactResponse`, `252 → 256,5` et « conditions de paiement » ont été retrouvées à **tous** leurs sites, Change Log compris. La matrice AC ↔ tâches ne présentait aucun trou ni aucune tâche orpheline (avant l'ajout d'AC2-bis, qui en a créé un, comblé par T3-bis). Et le calcul de marge a été **recompté une troisième fois, depuis le code**, par une lentille indépendante : `277 − 7 − 3 × 4,5 = 256,5`, marge 89,5 mm — le chiffre rectifié tient.

⚠️ **Garde-fou de splitting — état.** Sévérité maximale par passe : **P1 HIGH → P2 HIGH → P3 MEDIUM**. Le critère de non-convergence (« passe N+1 de sévérité **égale ou supérieure** ») s'est donc formellement déclenché en **P2**. Il est **résolu en P3**, qui décroît. C'est le profil que l'amendement de la rétro Epic 14 décrit comme « une revue qui travaille » — les passes tardives trouvant des défauts *réels et décroissants* — et non celui d'une story trop large. Aucun split n'est demandé ; le fait est consigné pour que l'arbitrage reste traçable.

**2026-08-10 — Passe 2 de `bmad-create-story validate`** (**Sonnet**, contexte frais, 3 lentilles indépendantes — BlindHunter, EdgeCaseHunter, AcceptanceAuditor). **4 HIGH, 5 MEDIUM, 2 LOW**, tous vérifiés au ground-truth par l'orchestrateur et tous remédiés. **Boucle NON convergée — passe 3 due.**

**La passe valide son propre coût : aucun des 4 HIGH n'était à portée de l'auteur**, et la passe 1 — conduite dans le contexte de rédaction — n'en avait vu aucun.

**HIGH-1 — AC6 était un critère qu'aucun test ne pouvait faire échouer.** Sa seule preuve était le delta de taille. Or `pdf.rs:1165-1168` documente que cette technique est **aveugle au décalage vertical**, et précise : « **Mesuré** — une première version de ce test comparait deux générations entre elles et restait **verte** sous la mutation ». **16-3a a déjà payé ce test muet.** Sa parade — mesurer au seuil de la garde haute, où 2 mm changent un verdict — n'est **pas disponible à droite** : la garde ne surveille que `y`, et ces mêmes Dev Notes établissent qu'il ne faut pas en créer une. AC6 impose désormais l'**extraction d'une fonction pure** et une assertion de **position** (ordonnée de la ligne « échéance » identique entre `Some` et `None`), seul montage qui tue la mutation « `my -= 4.5` sorti du `if let` ».

**HIGH-2 — la largeur avait été négligée au profit de la hauteur.** Le bloc droit fait **70 mm** (`meta_x = 120` → `PAGE_W − 20 = 190`), soit **moins** que les 100 mm du bloc gauche qui imposent déjà `IDENTITY_MAX_CHARS = 46`. Et `client_number` est un champ **libre de 50 caractères saisi par l'utilisateur**, là où `origin_reference` — le patron invoqué par D3 — est un numéro système court. Nouvel **AC6-bis** : troncature dédiée au bloc droit.

**HIGH-3 — `''` n'est pas `NULL`, et D2 protégeait le cas majoritaire sur un invariant qui ne vaut que pour `NULL` littéral.** Deux contacts soumis avec `client_number: ""` se percutent. La parade était déjà dans le dépôt, inutilisée par la story : `normalize_optional()` (`contacts.rs:259`), branchée sur `email`, `phone` et `default_payment_terms`. AC3 l'impose et exige le test des deux `""`.

**HIGH-4 — le sort d'un contact archivé n'était pas tranché** (`grep -ciE "archiv|active"` sur la story → **0**). Le dépôt dispose pourtant d'un patron d'unicité partielle éprouvé **deux fois** (colonne `GENERATED … VIRTUAL` nulle si `active = FALSE`). Nouveau **D2-bis** : verrou permanent **assumé**, avec sa raison — la valeur étant résolue à la génération (D5), libérer un numéro le ferait désigner deux entités selon l'époque, cassant le rapprochement même que la story vise. ⚠️ **Cette décision a été RENVERSÉE en passe 4** : son échappatoire n'existait pas (un contact archivé n'est pas modifiable) et son raisonnement était inversé (D5 est un argument *contre* le verrou). Voir l'entrée de la passe 4 — D2-bis prescrit désormais l'unicité **partielle**.

**Les 5 MEDIUM** : les DTO étaient nommés `ContactJson`/`ContactChanges`, vocabulaire recopié de 16-3a alors que `contacts` utilise `ContactResponse`/`ContactUpdate` (**convergence de deux lentilles**) ; AC8 n'avait aucune preuve pour la moitié **frontend** de l'i18n, exactement la zone que la KF #283 documente, le loader repliant **silencieusement** vers le FR ; T3 sous-dimensionnait le rayon — **23** fichiers construisent `NewContact` en littéral sans `derive(Default)`, contre 6 pour `Company` en 16-3a ; et l'énumération du calcul de marge citait « conditions de paiement » dans le bloc droit alors que `payment_terms` est dessiné à **gauche** (`ty -= 8.0`, l. 555).

⚠️ **Le chiffre de ce calcul était faux, et il l'était DEUX FOIS.** Les notes annonçaient `my = 252` au pire cas ; une lentille de revue, en le recalculant indépendamment, **est retombée sur 252** — en comptant quatre décréments de 4,5 là où le code n'en porte que trois. Le relevé exhaustif des décréments (`pdf.rs:295, 303, 317, 327`) donne `277 − 7 − 3 × 4,5 = ` **256,5**. Deux analyses, la même erreur, parce qu'aucune n'avait énuméré **depuis le code**. C'est le mode d'échec que le `CLAUDE.md` décrit pour les compteurs de migrations : *relire une valeur n'est pas recompter sa source*. La conclusion est inchangée et la marge plus large qu'annoncée.

**Ce que la passe a confirmé exact** : les quatre listes de colonnes de `contacts.rs` sont bien **exhaustives** (les autres `SELECT` interpolent `{COLUMNS}`) ; `map_contact_error` est câblé dans **les deux** handlers réels, pas seulement ses tests ; `InvoicePdfData` n'a que **deux** sites de construction, tous deux avec le `Contact` en portée ; **`build_credit_note_pdf_data` est déjà une fonction pure** — le refactor que 16-3a avait dû faire est acquis ; les compteurs de migrations (59 = 54 + 5 + 0) recomptés indépendamment ; le site positionnel P6 porte déjà son `assert_eq!` fail-loud.

**2026-08-10 — Passe 1 de `bmad-create-story validate`** (**Opus 5**). **1 HIGH, 2 MEDIUM, 2 LOW**, tous remédiés. **Boucle NON convergée — passe 2 due.**

⚠️ **Limite assumée de cette passe, et elle est structurelle** : elle a été conduite dans le **même contexte que la rédaction**, par le **même modèle**. La § *Review Iteration Rule* exige contexte frais et modèle différent, précisément pour contourner le biais d'auteur. Cette passe a donc porté sur ce qu'un auteur peut encore vérifier contre le code — **ancres, décomptes, sites oubliés, incohérences avec les précédents du dépôt** — et **pas** sur les angles morts de conception. La passe 2 doit tourner ailleurs, et ne doit pas traiter celle-ci comme une passe adversariale pleine.

**Le HIGH est une incohérence d'API que le précédent de la MÊME TABLE contredisait.** AC4 annonçait un **422**. Or l'unicité d'IDE — même table, même nature de violation, unicité par société — rend **409 CONFLICT** (`AppError::IdeAlreadyExists` → `errors.rs:1208`, code `IDE_ALREADY_EXISTS`). Deux contraintes jumelles rendant deux codes HTTP différents auraient été une incohérence livrée, et durable. Corrigé en 409 / `CLIENT_NUMBER_ALREADY_EXISTS`.

**Les deux MEDIUM sont des réinventions évitées de justesse** :

1. **Le mapping d'erreur existe déjà.** `map_contact_error` (`contacts.rs:461`) intercepte `DbError::UniqueConstraintViolation` et remappe par **nom de contrainte** — avec la raison documentée : le format du message MariaDB varie entre versions. La story disait « chercher le patron avant d'en écrire un nouveau », ce qui laissait la porte ouverte ; elle **nomme** désormais le helper, le fichier et la ligne, et prescrit une branche de plus.
2. **La sur-capture n'était pas couverte.** Le helper matche par `contains` : avec deux contraintes, rien ne garantit qu'aucune ne capture l'autre. Le dépôt a **déjà** ce test pour l'IDE (`contacts.rs:765`) — la story le fait calquer.

**Les deux LOW ferment des faux findings futurs** : `backup.rs` construit ses colonnes **dynamiquement** depuis `information_schema` (l. 92-145), donc la colonne y entre sans une ligne de code — une passe qui réclamerait sa mise à jour se tromperait ; et T2 porte désormais l'état de départ mesuré (59 fichiers = 59 lignes, `54 + 5 + 0`) avec le rappel de **recompter** plutôt que d'incrémenter.

**Ce que la passe a vérifié et confirmé** : les quatre ancres citées (`pdf.rs:315-324` conditionnel avec `my -= 4.5` **à l'intérieur** du `if let`, `contacts.sql:23`, `csv_tables.rs:314`, `user-manual.tex:615`) sont exactes. Et **D2 a été éprouvée sur le moteur réel** plutôt que sur la réputation de MariaDB : trois `NULL` acceptés, doublon non-`NULL` refusé en `1062`.

**2026-08-10 — Création de la story** (Opus 5). Spécifiée après le merge de 16-3a (PR #290), conformément à la note de planification du sprint-status (« indépendante de 16-3a : peut partir séparément. À spécifier après 16-3a »).

Relevé effectué sur le code, pas sur la mémoire : le champ n'existe nulle part (`grep` sur `client_number|customer_number|numero_client|reference_client` → zéro occurrence) ; `contacts` porte **quatre** listes de colonnes à maintenir, dont **`FIND_BY_ID_SQL` qui duplique `COLUMNS`** ; l'export CSV des contacts en porte deux de plus, appariées positionnellement, et **déjà partielles**.

⚠️ Une intuition a été **vérifiée puis écartée** : celle d'un besoin de garde de capacité symétrique pour le bloc droit. Le calcul (`ty = 167` contre `my ≥ ` **256,5** ` ` au pire cas) donne **89,5 mm** de marge, soit une vingtaine de lignes — la garde de 16-3a est bien asymétrique, mais une ligne de plus ne l'approche pas. La story l'inscrit pour qu'une passe de revue ne réclame pas une branche inatteignable.

*(Chiffres rectifiés en passe 2 : cette entrée annonçait `252` et `85 mm`, en comptant une ligne « conditions de paiement » qui appartient à la colonne **gauche**. Voir l'entrée de la passe 2 — l'erreur a été reproduite à l'identique par une lentille de revue indépendante.)*
