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

### D2 — Unicité **par société**, via `UNIQUE (company_id, client_number)`

*Pourquoi* : un numéro de client qui désigne deux contacts ne remplit pas sa fonction — il sert précisément à identifier. MariaDB autorise **plusieurs `NULL`** dans un index `UNIQUE`, donc les contacts sans numéro (la majorité au départ) ne se gênent pas entre eux. Le précédent est dans la **même table** : `uq_contacts_company_ide` (`20260414000001_contacts.sql:23`).

⚠️ La violation d'unicité doit remonter une **erreur métier** (422 avec un code parlant), pas un 500 opaque sur l'erreur SQLx `1062`. Voir AC4.

### D3 — Rendu dans le bloc **métadonnées (droite)**, sous le numéro de facture

*Pourquoi* : c'est ce que prescrit la note de planification au sprint-status (« bloc métadonnées du PDF »), et c'est la place usuelle sur une facture commerciale — le numéro de client appartient aux références du document, pas à l'adresse du destinataire. Le patron à copier est `origin_reference` (`pdf.rs:315-324`) : ligne conditionnelle qui ne descend le curseur `my` que si elle est dessinée.

### D4 — Les **avoirs** le portent aussi

*Pourquoi* : symétrie avec 16-3a, dont le manuel utilisateur dit « Les avoirs portent les mêmes coordonnées, à l'identique ». Un avoir est adressé au même client et sert le même rapprochement. `draw_invoice_section` est partagée par les deux documents — le champ suit donc automatiquement **dès lors que le site de construction de la donnée le renseigne** (voir le piège en Dev Notes).

### D5 — La valeur vient du **contact destinataire** de la facture

Résolue à la génération du PDF, comme le nom et l'adresse du débiteur. Pas de copie dénormalisée sur `invoices` : le numéro de client est un attribut du **contact**, et un changement doit se refléter sur les PDF régénérés.

## Acceptance Criteria

**AC1 — La colonne existe et est unique par société.**
Migration `crates/kesh-db/migrations/<date>_contacts_client_number.sql` : `ALTER TABLE contacts ADD COLUMN client_number VARCHAR(50) NULL, ADD CONSTRAINT uq_contacts_company_client_number UNIQUE (company_id, client_number);`
*Preuve* : test repository qui insère deux contacts de la même société avec le même `client_number` et vérifie le rejet, **plus** un test qui insère deux contacts avec `client_number = NULL` et vérifie qu'ils passent tous les deux (l'invariant MariaDB dont dépend D2 — s'il tombait, la moitié du parc deviendrait insaisissable).

**AC2 — Le champ traverse le repository sans se perdre.**
Les **quatre** listes de colonnes de `crates/kesh-db/src/repositories/contacts.rs` portent le champ : `COLUMNS` (l. 28-31), `FIND_BY_ID_SQL` (l. 33-36), l'`INSERT` (l. ~200) **et ses placeholders `VALUES`**, l'`UPDATE` (l. ~451).
*Preuve* : test d'aller-retour `create` → `find_by_id` → `update` → `find_by_id` qui vérifie la valeur à chaque étape. ⚠️ Un test qui n'utilise que `list` passerait alors même que `FIND_BY_ID_SQL` aurait été oublié — voir Dev Notes, « le piège qui coûterait le plus cher ».

**AC3 — Le champ est saisissable et relisible depuis l'API.**
`ContactJson` et son `impl From<Contact>` (`crates/kesh-api/src/routes/contacts.rs`), `NewContact`, `ContactChanges` et le DTO d'entrée portent le champ.
*Preuve* : test E2E API `POST /contacts` avec `clientNumber` → `GET /contacts/{id}` rend la même valeur. **Aucun compilateur ne vérifie cette couture** : omettre la ligne dans `From<Contact>` compile, stocke, et rend `null` pour toujours. C'est le HIGH de la passe 3 de 16-3a, sur la struct jumelle `CompanyJson`.

**AC4 — Un doublon rend une erreur métier, pas un 500.**
`POST`/`PUT` avec un `client_number` déjà pris dans la même société → **422** avec un code stable (p. ex. `CLIENT_NUMBER_ALREADY_USED`), pas une remontée brute de l'erreur SQLx `1062`.
*Preuve* : test E2E API qui crée le doublon et assert **le code HTTP et le code d'erreur**. Un test qui n'assert que « ce n'est pas 200 » laisserait passer un 500.

**AC5 — La fiche contact permet de le saisir.**
Champ dans l'écran contact du frontend, avec libellé traduit.
*Preuve* : test E2E Playwright qui saisit un numéro, enregistre, recharge la page et le relit. ⚠️ Un E2E est le **seul** test qui vérifie qu'une valeur traverse réellement la frontière HTTP — Vitest teste la construction du payload, les tests Rust la validation, et ni l'un ni l'autre ne voit une clé qui disparaît entre les deux.

**AC6 — Le PDF affiche le numéro quand il existe, et rien quand il n'existe pas.**
`InvoicePdfData` porte `debtor_client_number: Option<String>` ; `draw_invoice_section` dessine la ligne sous le numéro de facture **uniquement** si `Some`, et ne descend `my` que dans ce cas.
*Preuve* : **deux** cas de test, `Some` et `None`, comparés par **delta de taille du PDF** — le contenu textuel est hex-encodé dans les opérateurs `Tj`, un `grep` du texte échouerait de toute façon (technique établie par 16-3a, AC4). Le cas `None` doit rendre un PDF **de taille identique** à la baseline sans le champ.

**AC7 — L'avoir le porte aussi.**
*Preuve* : test au **site de construction de la donnée**, pas dans `pdf.rs`. Facture et avoir partagent `draw_invoice_section` : un test posé dans `pdf.rs` ne peut **structurellement pas** discriminer les deux, et resterait vert sous la mutation « ne pas renseigner le champ pour l'avoir ». ⚠️ Interdit d'hériter d'une fixture de facture par `..base` — c'est ce qui avait rendu le test d'AC5 de 16-3a muet.

**AC8 — L'i18n est complète sur les 4 locales.**
Clé `invoice-pdf-client-number` ajoutée à `I18N_KEYS` **et à la même position dans `DEFAULT_EN`** (`crates/kesh-qrbill/src/types.rs`), plus les 4 locales `fr-CH`/`de-CH`/`it-CH`/`en-CH` pour le libellé du PDF **et** celui de la fiche contact.
*Preuve* : l'assertion de compilation existante couvre les longueurs ; l'**appariement positionnel** ne l'est pas — le vérifier par un test qui résout la clé et compare au libellé attendu. Ne pas aggraver la KF #283 (57 clés déjà absentes des locales non-françaises).

**AC9 — Les garde-fous de migration sont honorés.**
Ligne ajoutée au tableau de `docs/migrations-idempotence-audit.md` **à sa place chronologique dans le tableau**, avec les **deux** sites du total et la partition recomptés **depuis le tableau** (`ls crates/kesh-db/migrations/*.sql | wc -l` doit égaler `grep -c '^| \`20' docs/migrations-idempotence-audit.md`). DDL pur → **ni** registre `POST_RESTORE_BACKFILLS` **ni** `EXEMPT_MIGRATIONS` (P7). `ADD COLUMN` nullable → **pas** de bump `kesh_version_min_required` ni de version Cargo (P1/P2).
*Preuve* : le test `every_data_backfill_migration_is_triaged` ne doit jamais sélectionner cette migration.

## Tasks / Subtasks

- [ ] **T1 — Migration** (AC1, AC9). Créer le `.sql` (`ADD COLUMN` + `ADD CONSTRAINT UNIQUE`), avec l'en-tête de commentaire du dépôt : rôle, longueur justifiée, statut non-breaking, et la mention explicite « DDL pur, ni registre ni exemption ». Puis `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` et **inspecter chaque site** (P6 — un test qui indexe les migrations par position change de sens à chaque ajout).
- [ ] **T2 — Audit d'idempotence** (AC9). Ligne dans le tableau, **recompter** les deux sites du total et les trois partitions. ⚠️ Les compteurs de partition ne valent pas le total ; les aligner dessus casserait l'invariant qu'ils servent à tenir.
- [ ] **T3 — Entité et repository** (AC1, AC2). `Contact`, `NewContact`, `ContactChanges` (`crates/kesh-db/src/entities/contact.rs`), puis **les quatre** listes de `repositories/contacts.rs`. Vérifier que le nombre de `?` de l'`INSERT` suit la liste de colonnes. `reconciliation.rs:201` réutilise `COLUMNS` — rien à y faire, mais le vérifier plutôt que le supposer.
- [ ] **T4 — Tests repository** (AC1, AC2). Aller-retour `create`/`find_by_id`/`update`/`find_by_id` ; unicité rejetée ; **deux `NULL` acceptés**.
- [ ] **T5 — Route et DTO** (AC3, AC4). `ContactJson` + `impl From<Contact>` + DTO d'entrée ; mapper l'erreur `1062` vers un 422 à code stable. Chercher le patron de mapping déjà utilisé pour `uq_contacts_company_ide` avant d'en écrire un nouveau.
- [ ] **T6 — Tests API** (AC3, AC4). Aller-retour `POST`/`GET` ; doublon → 422 + code d'erreur asserté.
- [ ] **T7 — Frontend** (AC5, AC8). Type TS, champ de la fiche contact, libellé sur les 4 locales. Respecter `lint-i18n-ownership`.
- [ ] **T8 — E2E fiche contact** (AC5). Saisie → enregistrement → rechargement → relecture.
- [ ] **T9 — PDF** (AC6, AC8). `InvoicePdfData.debtor_client_number`, rendu conditionnel calqué sur `origin_reference` (`pdf.rs:315-324`), clé i18n à la **même position** dans `I18N_KEYS` et `DEFAULT_EN`.
- [ ] **T10 — Service de génération** (AC6, AC7, D5). Renseigner le champ depuis le contact destinataire dans `invoice_pdf_service.rs` **et** au site de construction de l'avoir. C'est le site que la mutation doit tuer.
- [ ] **T11 — Tests PDF** (AC6, AC7). `Some`/`None` par delta de taille ; avoir testé au site de construction, sans `..base` d'une fixture de facture.
- [ ] **T12 — Export CSV** (optionnel, à trancher). `serialize_contacts_csv` (`crates/kesh-api/src/exports/csv_tables.rs:314`) a **deux listes appariées positionnellement** (en-têtes puis valeurs). Elle est **déjà partielle** — ni `first_name`, ni `address_street`, ni `language` n'y figurent —, donc l'omission est défendable. Si le champ est ajouté, **les deux** listes le sont, sous peine de décalage silencieux de toutes les colonnes suivantes.
- [ ] **T13 — Documentation** (règle de synchronisation). Manuel utilisateur FR : la section « Vos coordonnées sur la facture » (`docs/manual/fr/user-manual.tex:615`) a un pendant à écrire côté client. Régénérer le PDF. CHANGELOG.

## Dev Notes

### Ce que cette story ne doit PAS faire

- **Pas de numérotation automatique**, pas de séquence, pas de backfill du parc existant (D1). Les contacts existants restent à `NULL`, et c'est l'état normal.
- **Pas de copie du numéro sur `invoices`** (D5). Le PDF le résout depuis le contact.
- **Pas de garde de capacité symétrique** — voir ci-dessous, ce serait du code mort.

### Le piège qui coûterait le plus cher

**`FIND_BY_ID_SQL` (l. 33-36) duplique `COLUMNS` (l. 28-31) mot pour mot.** Les deux listes sont identiques aujourd'hui, mais ce sont **deux chaînes distinctes écrites à la main**. Ajouter le champ à `COLUMNS` seul compile, passe les tests de `list` (qui utilisent `COLUMNS`), et rend `find_by_id` **silencieusement amnésique** : la fiche contact affiche un champ vide alors que la base contient la valeur.

C'est pour cela qu'AC2 exige un aller-retour **par `find_by_id`**, et non par `list`.

### Ce que j'ai vérifié plutôt que supposé — la garde de capacité haute

16-3a a posé une garde `if y < ty + 2.0 { return Err(HeaderOverflow) }` (`pdf.rs:382`). On pourrait croire qu'ajouter une ligne au bloc droit appelle une garde symétrique. **Le calcul dit non** :

- `ty = PAGE_H - 130.0 = 297 - 130 = **167**`
- `my` part de `PAGE_H - 20.0 = 277`, puis `-7` (titre), puis `-4.5` par ligne. Au **pire cas actuel** — numéro, date, référence d'origine, échéance, conditions de paiement — `my` vaut **252**.

La colonne droite dispose donc de **85 mm**, soit **dix-huit lignes** de marge. Une ligne de plus la porte à 247,5. La garde est bien **asymétrique** (elle ne surveille que `y`, la colonne gauche), et c'est un fait à connaître — mais il est **sans conséquence ici**. Ajouter une garde sur `my` produirait une branche que rien ne peut atteindre, donc intestable et non couverte : exactement le genre de code que les passes de revue suivantes signaleront à juste titre.

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

**2026-08-10 — Création de la story** (Opus 5). Spécifiée après le merge de 16-3a (PR #290), conformément à la note de planification du sprint-status (« indépendante de 16-3a : peut partir séparément. À spécifier après 16-3a »).

Relevé effectué sur le code, pas sur la mémoire : le champ n'existe nulle part (`grep` sur `client_number|customer_number|numero_client|reference_client` → zéro occurrence) ; `contacts` porte **quatre** listes de colonnes à maintenir, dont **`FIND_BY_ID_SQL` qui duplique `COLUMNS`** ; l'export CSV des contacts en porte deux de plus, appariées positionnellement, et **déjà partielles**.

⚠️ Une intuition a été **vérifiée puis écartée** : celle d'un besoin de garde de capacité symétrique pour le bloc droit. Le calcul (`ty = 167` contre `my ≥ 252` au pire cas) donne 85 mm de marge, soit dix-huit lignes — la garde de 16-3a est bien asymétrique, mais une ligne de plus ne l'approche pas. La story l'inscrit pour qu'une passe de revue ne réclame pas une branche inatteignable.
