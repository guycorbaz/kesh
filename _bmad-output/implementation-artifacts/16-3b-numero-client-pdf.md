# Story 16.3b : Numéro de client sur le PDF de facture

## Status

ready-for-dev

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

1. **L'échappatoire invoquée n'existe pas.** Elle disait « vider le champ sur l'archive libère le numéro ». Or **un contact archivé n'est pas modifiable** : `repositories/contacts.rs:421-426` rend `IllegalStateTransition("impossible de modifier un contact archivé")`, doublé par le `WHERE … AND active = TRUE` de l'`UPDATE` (l. 457) ; et il n'existe **aucune route de désarchivage** (`grep -rniE "unarchive|restore_contact|reactivate"` → 0 ligne). Le numéro aurait donc été perdu **à vie**, déblocable seulement par un `UPDATE` SQL manuel en base — ce qui n'est pas une échappatoire, c'est une intervention hors produit.
2. **Le dépôt avait déjà rendu ce jugement, par écrit, sur le cas jumeau.** `20260722000001_accounts_role_postable.sql:35-38` : « *Le `active AND` n'est PAS cosmétique : sans lui, un compte archivé squatterait son rôle singleton à vie et son remplaçant actif ne pourrait jamais le recevoir (409 permanent causé par un compte mort).* » La situation des contacts est **strictement plus fermée**, puisqu'ils n'ont même pas de réactivation.

**Et le précédent `uq_contacts_company_ide` ne sauve pas le verrou** : un IDE est un identifiant **attribué par l'État**, jamais réattribuable à une autre entité — le verrou permanent y est sémantiquement juste. Un numéro de client est une **étiquette interne**, recyclable par nature.

**Enfin, le raisonnement par D5 était inversé.** Il prétendait que libérer un numéro le ferait « désigner deux entités selon l'époque ». Le système ne résout **jamais** numéro → contact : une facture désigne son contact par clé étrangère (`entities/invoice.rs`), et le PDF imprime le numéro *courant* de ce contact. Il n'y a donc aucune ambiguïté interne. La seule ambiguïté possible est sur le **papier déjà envoyé** — exactement l'artefact que D5 laisse déjà dériver pour le nom et l'adresse. D5 est un argument **contre** le verrou, pas pour.

### D3 — Rendu dans le bloc **métadonnées (droite)**, sous le numéro de facture

*Pourquoi* : c'est ce que prescrit la note de planification au sprint-status (« bloc métadonnées du PDF »), et c'est la place usuelle sur une facture commerciale — le numéro de client appartient aux références du document, pas à l'adresse du destinataire. Le patron à copier est `origin_reference` (`pdf.rs:315-324`) : ligne conditionnelle qui ne descend le curseur `my` que si elle est dessinée.

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
`ContactResponse` (`crates/kesh-api/src/routes/contacts.rs:151`) et son `impl From<Contact> for ContactResponse` (l. 186), plus `NewContact` (`crates/kesh-db/src/entities/contact.rs:213`), `ContactUpdate` (l. 250) et le DTO d'entrée portent le champ.

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

⚠️ **Le chemin d'édition efface le champ si une seule ligne est oubliée, et rien ne le signale.** `PUT /contacts/{id}` est un **full-replace** : `+page.svelte:336-341` envoie le **même `payload`** pour la création et l'édition, et `openEdit` (l. 240-258) hydrate le formulaire **champ par champ**, sur dix-huit lignes. Ajouter `clientNumber` au payload **sans** ajouter `formClientNumber = c.clientNumber ?? ''` à `openEdit` ne casse **aucune** compilation (le champ TS est optionnel) — et toute modification d'un contact, même sans rapport, **efface son numéro**. La preuve 1 passe au vert sous cette mutation, puisqu'elle ne parcourt que la création.

**AC6 — Le PDF affiche le numéro quand il existe, et rien quand il n'existe pas.**
`InvoicePdfData` porte `debtor_client_number: Option<String>` ; `draw_invoice_section` dessine la ligne sous le numéro de facture **uniquement** si `Some`, et ne descend `my` que dans ce cas — `my -= 4.5` **à l'intérieur** du `if let`, comme `origin_reference` (`pdf.rs:315-324`).

*Preuve (présence)* : deux cas, `Some` et `None`, comparés par **delta de taille du PDF** — le contenu textuel est hex-encodé dans les opérateurs `Tj`, un `grep` du texte échouerait de toute façon.

⚠️ **Le delta de taille NE PEUT PAS prouver la conditionnalité du décrément, et le dépôt l'a déjà payé.** `pdf.rs:1165-1168` le dit noir sur blanc : « un décalage vertical de 2 mm déplace le texte sans changer sa longueur, donc le PDF pèse le même nombre d'octets. **Mesuré** — une première version de ce test comparait deux générations entre elles et restait **verte** sous la mutation. » La parade de 16-3a fut de mesurer au **seuil de la garde haute**, où 2 mm changent un verdict (`no_identity_line_costs_no_vertical_space`, l. 1182). **Cette parade n'est pas disponible ici** : la garde ne surveille que `y`, la colonne gauche, et les Dev Notes de cette story établissent qu'il ne faut pas en créer une à droite (89,5 mm de marge ⇒ branche inatteignable).

*Preuve (conditionnalité) — le refactor est donc obligatoire*, comme AC5 de 16-3a a imposé le sien : extraire la construction du bloc métadonnées en **fonction pure** rendant la séquence des lignes et leur ordonnée (`Vec<(String, f32)>`), et tester **la position**.

⚠️ **L'assertion doit être `y_none − y_some == 4.5`, PAS « les deux ordonnées sont identiques ».** Une version antérieure de cet AC écrivait « identique », et c'était **exactement le verdict du mutant** : avec un code **correct**, la ligne « échéance » est 4,5 mm **plus basse** dans le cas `Some` ; c'est la mutation — `my -= 4.5` sorti du `if let`, donc appliqué dans les deux cas — qui rend les deux ordonnées **égales**. Suivie à la lettre, cette assertion aurait forcé l'implémenteur à écrire un test rouge sur du bon code, puis à sortir le décrément du `if let` pour le verdir : **à implémenter la mutation qu'elle prétendait tuer.**

Deux formulations correctes, au choix — la seconde transpose la vraie parade de 16-3a (`no_identity_line_costs_no_vertical_space`, `pdf.rs:1182`), qui ne compare pas deux générations entre elles mais mesure l'absence contre un **absolu** :

- `assert_eq!(y_none - y_some, 4.5)` — le delta *doit* valoir 4,5 ; la mutation le met à 0 ;
- ou, en `None`, ordonnée « échéance » **exactement 265,5** (valeur pré-story, codée en dur).

Sans ce montage, AC6 est un critère dont aucun test ne peut échouer, c'est-à-dire un test muet — le mode d'échec le plus documenté de ce dépôt.

**AC6-bis — Un numéro trop long est tronqué, pas débordé.**
Le bloc droit dispose de **70 mm** (`meta_x = 120.0` à `PAGE_W − 20.0 = 190.0`, cf. `hline` l. 428) — **moins** que les 100 mm du bloc gauche, qui imposent déjà `IDENTITY_MAX_CHARS = 46`. Or `client_number` est un champ **libre de 50 caractères saisi par l'utilisateur**, contrairement à `origin_reference` qui est un numéro système court : le patron de D3 ne couvre donc pas ce risque.
*Preuve* : constante de troncature **dédiée au bloc droit**, calibrée sur 70 mm — donc **strictement inférieure** à `IDENTITY_MAX_CHARS` —, et test de calibrage calqué sur celui du bloc gauche. Tronquer, jamais refuser : refuser une facture pour un champ décoratif serait disproportionné (c'est déjà l'arbitrage de 16-3a, `pdf.rs:266-267`).

**AC7 — L'avoir le porte aussi.**
*Preuve* : test au **site de construction de la donnée**, pas dans `pdf.rs`. Facture et avoir partagent `draw_invoice_section` : un test posé dans `pdf.rs` ne peut **structurellement pas** discriminer les deux, et resterait vert sous la mutation « ne pas renseigner le champ pour l'avoir ». ⚠️ Interdit d'hériter d'une fixture de facture par `..base` — c'est ce qui avait rendu le test d'AC5 de 16-3a muet.

**AC8 — L'i18n est complète sur les 4 locales.**
Clé `invoice-pdf-client-number` ajoutée à `I18N_KEYS` **et à la même position dans `DEFAULT_EN`** (`crates/kesh-qrbill/src/types.rs`), plus les 4 locales `fr-CH`/`de-CH`/`it-CH`/`en-CH` pour le libellé du PDF **et** celui de la fiche contact.
*Preuve, côté PDF* : l'assertion de compilation existante (`types.rs:264`) couvre les **longueurs** ; l'**appariement positionnel** ne l'est pas — le vérifier par un test qui résout la clé et compare au libellé attendu (mutation à tuer : décaler une entrée de `DEFAULT_EN` d'un cran ; les longueurs restent égales, le test runtime doit rougir).

*Preuve, côté fiche contact — sans quoi AC8 ne couvre que la moitié du sujet* : un test qui résout la clé du libellé **dans les 4 locales** et échoue si l'une retombe sur le repli français. Le loader `kesh-i18n` **replie silencieusement** une clé absente vers le FR (`loader.rs`, `format_missing_key_in_de_falls_back_to_fr`) — c'est un comportement voulu, mais il rend l'absence **invisible** : c'est le mécanisme même de la KF #283 (57 clés déjà manquantes en de-CH / it-CH / en-CH). AC8 avertissait de « ne pas l'aggraver » sans fournir aucune preuve exécutable pour la partie exactement visée ; l'avertissement reposait donc sur la seule discipline manuelle.

**AC9 — Les garde-fous de migration sont honorés.**
Ligne ajoutée au tableau de `docs/migrations-idempotence-audit.md` **à sa place chronologique dans le tableau**, avec les **deux** sites du total et la partition recomptés **depuis le tableau** (`ls crates/kesh-db/migrations/*.sql | wc -l` doit égaler `grep -c '^| \`20' docs/migrations-idempotence-audit.md`). DDL pur → **ni** registre `POST_RESTORE_BACKFILLS` **ni** `EXEMPT_MIGRATIONS` (P7). `ADD COLUMN` nullable → **pas** de bump `kesh_version_min_required` ni de version Cargo (P1/P2).
*Preuve* : le test `every_data_backfill_migration_is_triaged` ne doit jamais sélectionner cette migration.

**AC10 — Le numéro est cherchable, et visible dans la liste.**
La recherche de contacts accepte le numéro de client, et la liste affiche une colonne « N° client ».

*Pourquoi c'est un critère et non un confort* : le « so that » de cette story promet **deux** bénéfices, et le second est « que je puisse moi-même **retrouver un contact depuis une facture papier** ». Sans recherche ni colonne, un utilisateur tenant une facture portant « N° client : CLI-2026-00042 » n'a **littéralement aucun moyen** de remonter au contact. Livrer une story dont la moitié du bénéfice annoncé n'est ni spécifiée ni testée est le plus mauvais des choix disponibles.

*Le coût est faible, et le dépôt donne la bonne technique.* La recherche actuelle (`repositories/contacts.rs:170-186`) couvre `name` par `MATCH … AGAINST` (FULLTEXT) et `email` par `LIKE`. Le commentaire de la l. 162-165 explique **pourquoi** `email` est resté en `LIKE` : ses séparateurs cassent les tokens FULLTEXT. Un `CLI-2026-00042` subirait exactement le même sort — donc une branche `OR client_number LIKE ?` à côté de celle d'`email`, et non un ajout à l'index FULLTEXT.

*Preuve* : test API — recherche par numéro exact rend le contact ; recherche par fragment le rend aussi. Test E2E — la colonne est visible dans la liste.

⚠️ Ne pas oublier le **placeholder** du champ de recherche (`+page.svelte:441`, « Rechercher par nom ou email… ») : le laisser tel quel ferait mentir l'interface sur ce qu'elle sait faire, sur les 4 locales.

## Tasks / Subtasks

- [ ] **T1 — Migration** (AC1, AC9). Créer le `.sql` : `ADD COLUMN client_number`, **`ADD COLUMN client_number_uniq … GENERATED ALWAYS AS (IF(active, client_number, NULL)) VIRTUAL`**, puis `ADD CONSTRAINT … UNIQUE (company_id, client_number_uniq)`. En-tête de commentaire du dépôt : rôle, longueur justifiée, statut non-breaking, mention « DDL pur, ni registre ni exemption », **et le renvoi aux deux précédents du patron** (`reconciliation_rules`, `accounts_role_postable`) avec le pré-requis MariaDB ≥ 10.6. Puis `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` et **inspecter chaque site** (P6 — un test qui indexe les migrations par position change de sens à chaque ajout).
- [ ] **T2 — Audit d'idempotence** (AC9). Ligne dans le tableau, **recompter** les deux sites du total et les trois partitions. État de départ mesuré le 2026-08-10 : **59** fichiers `.sql` = 59 lignes de tableau = en-tête = total, partitionnés en `54 tracked-by-sqlx + 5 yes + 0 no`. Cette migration porte l'ensemble à **60** et `tracked-by-sqlx` à **55** — mais **recompter depuis le tableau**, ne pas incrémenter ces chiffres de confiance : c'est exactement le geste qui avait laissé dériver le compteur de 7 unités jusqu'à la Story 16-1a. ⚠️ Les compteurs de partition ne valent pas le total ; les aligner dessus casserait l'invariant qu'ils servent à tenir.
- [ ] **T3 — Entité et repository** (AC1, AC2). `Contact`, `NewContact` (`entities/contact.rs:213`), `ContactUpdate` (l. 250) — **pas** `ContactChanges`, qui n'existe pas —, puis **les quatre** listes de `repositories/contacts.rs`. Vérifier que le nombre de `?` de l'`INSERT` suit la liste de colonnes. `reconciliation.rs:201` réutilise `COLUMNS` — rien à y faire, mais le vérifier plutôt que le supposer.
- [ ] **T3-bis — Audit** (AC2-bis). Ajouter le champ à `contact_snapshot_json` (`repositories/contacts.rs:39-58`) — **cinquième liste de champs écrite à la main**, non-SQL, que les quatre listes de T3 ne couvrent pas.
- [ ] **T3-ter — Détection de non-changement** (AC2-ter). Ajouter le champ à `is_no_op_change` (`repositories/contacts.rs:377-396`) — **sixième liste**. Sans elle, modifier le seul numéro est classé no-op : rollback, `200 OK`, donnée perdue en silence.
- [ ] **T4 — Tests repository** (AC1, AC2, AC2-bis, AC2-ter). Aller-retour `create`/`find_by_id`/`update`/`find_by_id` ; unicité rejetée **entre contacts actifs** ; **deux `NULL` acceptés** ; **numéro d'un contact archivé réattribuable à un actif** (l'invariant de D2-bis) ; trace d'audit ; et `update` du **seul** `client_number` → nouvelle valeur **et** `version` incrémentée.
- [ ] **T5 — Route et DTO** (AC3, AC4). `ContactResponse` (`contacts.rs:151`) + `impl From<Contact> for ContactResponse` (l. 186) + DTO d'entrée, **avec `normalize_optional()`** sur le champ (l. 259) comme pour `email`/`phone`. Pour l'erreur : **étendre `map_contact_error` (`contacts.rs:461`)** d'une branche, ajouter la variante `AppError::ClientNumberAlreadyExists` et sa ligne dans le `match` de `errors.rs` (409 / `CLIENT_NUMBER_ALREADY_EXISTS`). Ne **pas** écrire un second helper : le repository rend déjà `DbError::UniqueConstraintViolation`, tout le chemin existe.
- [ ] **T6 — Tests API** (AC3, AC4). Aller-retour `POST`/`GET` ; doublon → **409** + code d'erreur asserté ; **non-sur-capture** entre les deux contraintes (calquer `contacts.rs:765`).
- [ ] **T7 — Frontend** (AC4, AC5, AC8, AC10). Type TS (`contacts.types.ts`, interface `ContactResponse`), champ de la fiche contact, **hydratation dans `openEdit` (l. 240-258) — la ligne dont l'oubli efface le champ à chaque édition**, **colonne « N° client » dans la liste** (+ `colspan` du tableau, câblé en dur l. 540), **placeholder de recherche corrigé** (l. 441), libellés sur les 4 locales. Respecter `lint-i18n-ownership`.
- [ ] **T7-ter — Message du 409 à l'écran** (AC4). Le frontend branche les codes **un par un** (`+page.svelte:349-358`) : `OPTIMISTIC_LOCK_CONFLICT`, puis `IDE_ALREADY_EXISTS` → `contact-error-ide-duplicate`, **sinon `err.message` brut du backend**. Sans branche dédiée, l'utilisateur qui saisit un doublon reçoit un message non maîtrisé et non traduit, là où le doublon d'IDE a droit au sien. Ajouter la branche `CLIENT_NUMBER_ALREADY_EXISTS` et la clé `contact-error-client-number-duplicate` **dans les 4 locales** — le domaine `contact-*` est aujourd'hui à 59 clés dans chacune, sans dérive : ne pas l'ouvrir.
- [ ] **T7-bis — Recherche par numéro** (AC10). Branche `OR client_number LIKE ?` dans `repositories/contacts.rs:170-186`, **à côté** de celle d'`email` et **pas** dans l'index FULLTEXT — le commentaire l. 162-165 explique pourquoi les séparateurs cassent les tokens.
- [ ] **T8 — E2E fiche contact** (AC5). Saisie → enregistrement → rechargement → relecture. ⚠️ Le fichier **DOIT** être nommé `*.spec.ts` : `playwright.config.ts:35` filtre sur `testMatch: /(.+\.)?spec\.[jt]s/`, et un `*.test.ts` posé dans `tests/e2e/` est **silencieusement ignoré** — il ne rougit jamais, il se tait.
- [ ] **T9 — PDF** (AC6, AC6-bis, AC8). `InvoicePdfData.debtor_client_number`, rendu conditionnel calqué sur `origin_reference` (`pdf.rs:315-324`), **extraction de la fonction pure** de construction du bloc métadonnées (sans elle, AC6 est intestable), **constante de troncature dédiée au bloc droit** (70 mm, donc < `IDENTITY_MAX_CHARS`), clé i18n à la **même position** dans `I18N_KEYS` et `DEFAULT_EN`.
- [ ] **T10 — Service de génération** (AC6, AC7, D5). Renseigner le champ depuis le contact destinataire dans `invoice_pdf_service.rs` **et** au site de construction de l'avoir. C'est le site que la mutation doit tuer.
- [ ] **T11 — Tests PDF** (AC6, AC6-bis, AC7). Présence par delta de taille ; **conditionnalité par assertion de position** sur la fonction pure extraite en T9 (l'ordonnée de la ligne « échéance » identique entre `Some` et `None`) ; troncature calibrée ; avoir testé au site de construction, sans `..base` d'une fixture de facture.
- [ ] **T12 — Export CSV** (optionnel, à trancher). `serialize_contacts_csv` (`crates/kesh-api/src/exports/csv_tables.rs:314`) a **deux listes appariées positionnellement** (en-têtes puis valeurs). Elle est **déjà partielle** — ni `first_name`, ni `address_street`, ni `language` n'y figurent —, donc l'omission est défendable. Si le champ est ajouté, **les deux** listes le sont, sous peine de décalage silencieux de toutes les colonnes suivantes.
- [ ] **T13 — Documentation** (règle de synchronisation). Les sites sont **énumérés** plutôt que laissés à l'appréciation :
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

## Change Log

**2026-08-10 — Passe 4 de `bmad-create-story validate`** (**Opus 5**, contexte frais, lentille « remise en cause des décisions »). **3 HIGH, 2 MEDIUM, 2 LOW**, tous vérifiés au ground-truth et remédiés. **Boucle NON convergée — passe 5 due.**

**C'est la première passe à contester une DÉCISION, et elle en renverse une.** Les passes 1 à 3 avaient corrigé des ancres, des décomptes, des clauses de preuve et des sites oubliés — aucune n'avait interrogé D1-D5. Le mandat de celle-ci était exclusivement là.

**HIGH-1 — D2-bis reposait sur une échappatoire qui n'existe pas, et le dépôt avait déjà jugé ce cas.** La décision « verrou permanent » se justifiait par « vider le champ sur l'archive libère le numéro ». Or **un contact archivé n'est pas modifiable** : `repositories/contacts.rs:421-426` rend `IllegalStateTransition("impossible de modifier un contact archivé")`, doublé du `WHERE … AND active = TRUE` de l'`UPDATE` (l. 457) ; et **aucune route de désarchivage n'existe** (`grep -rniE "unarchive|restore_contact|reactivate"` → 0 ligne). Le numéro était donc perdu **à vie**, déblocable seulement par un `UPDATE` SQL manuel. Surtout, `20260722000001_accounts_role_postable.sql:35-38` avait déjà tranché le cas jumeau **par écrit** : « *le `active AND` n'est PAS cosmétique : sans lui, un compte archivé squatterait son rôle à vie… 409 permanent causé par un compte mort* ». **D2-bis est renversée** : unicité partielle par colonne `GENERATED … VIRTUAL`, troisième application d'un patron éprouvé deux fois.

**HIGH-2 — la clause de preuve d'AC6 assertait la signature du mutant qu'elle prétendait tuer.** La passe 2 avait imposé de tester « l'ordonnée de la ligne échéance **identique** entre `Some` et `None` ». C'est **l'inverse** : avec un code correct, `Some` est 4,5 mm plus bas ; c'est la **mutation** (`my -= 4.5` hors du `if let`) qui rend les deux égales. Suivie à la lettre, l'assertion aurait forcé à écrire un test rouge sur du bon code, puis à **implémenter la mutation** pour le verdir. Remplacée par `y_none − y_some == 4.5`, ou l'ordonnée absolue en `None` — la vraie parade de 16-3a mesure l'absence contre un **absolu**, jamais deux générations l'une contre l'autre. ⚠️ **Ce défaut a été introduit par la remédiation de la passe 2** : la remédiation reste la première source de défauts, comme le lot 16-1 l'avait mesuré sur 7 passes.

**HIGH-3 — une SIXIÈME liste, et c'est la seule qui perde la donnée.** `is_no_op_change` (`repositories/contacts.rs:377-396`) énumère 18 champs à la main et **court-circuite l'écriture** (l. 439-442 : `rollback` + `Ok(before)`). Sans `client_number`, **le geste central de la story** — ouvrir une fiche pour lui attribuer son numéro, sans rien changer d'autre — est classé « aucun changement » : pas d'`UPDATE`, pas d'audit, et l'API rend **`200 OK` avec l'ancienne valeur**. L'utilisateur voit son numéro disparaître au rechargement, sans erreur. Nouvel **AC2-ter** + **T3-ter**, avec le contrôle de `version` pour distinguer une écriture réelle d'un `Ok(before)`.

**MEDIUM-1 — le raisonnement de D2-bis était inversé.** Il invoquait D5 (« la valeur est résolue à la génération ») pour justifier le verrou. Vérifié : le PDF n'est **jamais stocké** (aucun blob dans les migrations) et est régénéré à chaque téléchargement comme à chaque envoi d'e-mail ; une facture désigne son contact par **clé étrangère**, le système ne résout jamais numéro → contact. Libérer un numéro ne crée donc **aucune** ambiguïté interne. D5 était un argument **contre** le verrou.

**MEDIUM-2 — la moitié du « so that » n'avait aucun critère.** La story promet « retrouver un contact depuis une facture papier ». La recherche ne couvre que `name` (FULLTEXT) et `email` (`LIKE`) ; la liste affiche l'IDE mais pas le numéro. Nouvel **AC10** + **T7-bis**, avec la bonne technique donnée par le dépôt lui-même : `OR client_number LIKE ?` **à côté** d'`email`, pas dans l'index FULLTEXT — les séparateurs cassent les tokens (`contacts.rs:162-165`).

**LOW** — l'unicité est **insensible à la casse** (collation `_ci` héritée, aucune déclarée sur `contacts`) : souhaitable, mais à décider et à tester, d'où le 3ᵉ cas d'AC1 ; et le 409 n'était pas diagnosticable quand le détenteur pouvait être un contact archivé donc invisible — **ce point disparaît avec le renversement de D2-bis**.

**Seconde lentille de la passe 4 (complétude) — 1 CRITICAL, 2 HIGH, 3 MEDIUM, 1 LOW.** Son CRITICAL est **le même que HIGH-1 ci-dessus** : deux lentilles indépendantes ont convergé sur l'échappatoire inexistante de D2-bis, ce qui en confirme la réalité. Trois findings neufs :

- **HIGH — le chemin d'ÉDITION efface le champ, et AC5 ne pouvait pas le voir.** `PUT /contacts/{id}` est un **full-replace** : `+page.svelte:336-341` envoie le même `payload` pour créer et pour modifier, et `openEdit` (l. 240-258) hydrate le formulaire **champ par champ**, sur dix-huit lignes. Ajouter le champ au payload sans ajouter sa ligne d'hydratation ne casse **aucune compilation** — et modifier un simple téléphone **efface le numéro de client**. AC5 ne testait que la création : il serait resté **vert** sous cette mutation. Seconde preuve ajoutée (« modifier un champ sans rapport, le numéro survit »).
- **MEDIUM — AC4 créait un code d'erreur que personne n'affiche.** Le frontend branche les codes **un par un** (`+page.svelte:349-358`) et **retombe sur `err.message` brut** hors des cas connus. Le doublon d'IDE a son message traduit ; le nôtre n'en avait aucun. **T7-ter** ajoute la branche et la clé sur les 4 locales.
- **MEDIUM — T13 ne visait qu'une section de manuel sur quatre sites.** Manquaient la liste des champs de création (`user-manual.tex:510-520`, la page que lira l'utilisateur), la phrase sur la recherche (l. 531 — qui **surestime déjà** le code), `docs/search-patterns.md:116` et `README.md:213`, puisque cette story **ferme #151**. T13 les énumère désormais.

**Et une liste d'angles ouverts puis rendus vides** — portes d'entrée d'un contact (il n'y en a qu'une), fixtures CI, import d'une sauvegarde antérieure, rappels, `api-external.md`, site web — consignée dans les Dev Notes pour que les passes suivantes ne refassent pas le trajet. Un défaut de doc **préexistant** a été relevé au passage, hors périmètre : `user-manual.tex:533-535` annonce un import CSV de contacts qui n'existe pas.

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
