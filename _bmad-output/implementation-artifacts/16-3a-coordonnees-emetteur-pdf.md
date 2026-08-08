# Story 16.3a : Coordonnées de l'émetteur sur le PDF de facture

## Status

review

## Story

**As a** indépendant ou fiduciaire qui envoie ses factures à des clients,
**I want** que mes **coordonnées de contact** — téléphone, e-mail, site web — figurent sur le PDF sous mon adresse,
**so that** un client qui a une question sur la facture puisse me joindre **depuis la facture elle-même**, sans avoir à chercher ailleurs.

Issue : **#151** (moitié « émetteur »). Sous-story de l'Epic 16 « Facturation avancée », cible **v0.9.0**.

## Contexte

L'issue #151 constate que le PDF « semble incomplet comparé à des factures commerciales reçues, **particulièrement sur le bloc gauche (émetteur)** ». Le relevé du code confirme : le bloc émetteur ne rend que **le nom, l'adresse et le numéro IDE** (`pdf.rs`, `draw_invoice_section`).

**Trois des quatre champs demandés par #151 n'existent nulle part**, ce qui fait de cette story bien plus qu'une retouche de mise en page. Relevé sur le schéma réel :

| Champ | État |
|---|---|
| Récapitulatif TVA | ✅ **déjà livré** (PR #267) — `vat_lines` + `recap_reserve` dans `pdf.rs`. **Hors périmètre.** |
| E-mail émetteur | ⚠️ colonne `companies.email` **existe**, le PDF **ne la rend pas** |
| Téléphone émetteur | ❌ **aucune colonne** sur `companies` |
| Site web émetteur | ❌ **aucune colonne** sur `companies` |
| Numéro de client | ❌ aucune colonne sur `contacts` — **story 16-3b** |

**Périmètre arbitré par Guy le 2026-08-05** : découpage **par entité**, imposé par la § *Règle de splitting préventif* — 7 modules recensés pour un seuil de 5. **16-3a** traite l'émetteur (`companies`), **16-3b** le numéro de client (`contacts`). Les deux sont **indépendantes** et peuvent partir séparément — contrairement à 16-2a/16-2b, aucune ne livre de donnée que l'autre seule afficherait.

## Décisions

### D1 — Le budget vertical du haut de page est de 110 mm, et **rien ne le surveille**

C'est le risque central de cette story, et il est **silencieux**. La chaîne verticale de `draw_invoice_section` :

```
y = PAGE_H - 20                     départ du bloc émetteur
  - 5                               nom (14 pt gras)
  - 4 × N                           N lignes d'adresse (9 pt)
  - 6                               IDE, si présent
y = y.min(PAGE_H - 55.0)            ⚠️ PLAFOND, pas plancher : si y est DÉJÀ plus bas, il y reste
  - 5                               libellé « Destinataire »
  - 4.5                             nom du débiteur
  - 4 × M                           M lignes d'adresse du débiteur
                                    puis : ty = PAGE_H - 130.0   ← CONSTANTE indépendante
```

**Le tableau des lignes démarre à une ordonnée fixe.** Il n'est jamais repoussé par ce qui précède. Allonger le bloc émetteur de 3 lignes (~12 mm) rapproche donc le bloc destinataire du tableau **sans qu'aucune garde ne le détecte** : la seule garde existante (`TooManyLines`) surveille le **plancher QR**, en bas de page, et ne voit rien de ce qui se passe en haut.

**Décision** : la story **doit** ajouter une garde symétrique en haut — le bloc destinataire ne doit pas descendre sous le début du tableau — **et** la prouver par un test de pire cas. Il est interdit de se contenter de constater que le cas nominal tient.

⚠️ **Ne PAS « corriger » en déplaçant `ty`** : cette constante aligne le tableau avec la zone QR plus bas. La toucher déplacerait tout le document.

### D2 — Les trois champs sont **facultatifs**, et une valeur absente ne laisse **aucune ligne vide**

Aucune donnée existante ne les porte : toute société déjà créée aura `NULL` sur les trois. Le rendu doit donc suivre le patron **déjà employé pour l'IDE** (`if let Some(ide) = &inv.creditor_ide`) — une ligne n'est dessinée que si la valeur est présente, et le curseur ne descend pas sinon.

**Conséquence à ne pas manquer** : c'est aussi ce qui **limite** le risque de D1 — une société sans coordonnées produit exactement le PDF d'aujourd'hui, à l'octet près pour le bloc haut.

### D3 — Le rendu se fait aux **DEUX** sites de construction, avoir compris

`InvoicePdfData` est construit à **deux endroits**, et les oublier serait invisible en revue de la facture :

- `crates/kesh-api/src/routes/invoice_pdf_service.rs:279` — la facture ;
- `crates/kesh-api/src/routes/credit_notes.rs:277` — l'**avoir**.

Les deux lisent déjà `company.ide_number`. **Un avoir sans coordonnées alors que la facture en porte serait un défaut visible par le client**, et aucun test de facture ne l'attraperait. Le grep de propagation à exécuter est `grep -rn "creditor_ide" crates/kesh-api/src/`.

### D4 — Saisie sur le patron **20-3b2** : édition inline dans les réglages + route dédiée

⚠️ **Il n'existe AUCUNE route de mise à jour générale de la société.** `crates/kesh-api/src/routes/companies.rs` n'expose que **deux** fonctions publiques : `update_company_email` (`:86`) et `get_current` (`:149`). Le seul autre chemin d'écriture est `update_company_coordinates` (`onboarding.rs:983`), qui **n'a qu'un seul appelant** — l'étape 5→6 de l'onboarding — et dont le commentaire prévient qu'un second appelant imposerait d'extraire un paramètre `reset_stub`, puisqu'elle pose `is_stub = FALSE` **inconditionnellement**.

**Et l'écran de réglages n'édite pas ces champs** : nom, type, adresse et IDE y sont rendus en **lecture seule** (`<dd>`). Un seul champ y est éditable — **l'e-mail**, en édition *inline* avec un bouton « Modifier » (`settings/+page.svelte:175`), servi par sa route dédiée. C'est la **Story 20-3b2**.

**Décision : reproduire ce patron.** Téléphone et site web deviennent éditables *inline* dans les réglages, servis par une route dédiée, exactement comme l'e-mail. Ne **pas** créer de route de mise à jour générale — ce serait une refonte hors périmètre — et ne **pas** réutiliser `update_company_coordinates`, dont le `is_stub = FALSE` inconditionnel et l'appelant unique sont des invariants documentés.

⚠️ **Sans cette décision, la story serait inutilisable par son premier utilisateur.** L'onboarding ne se rejoue pas : sur une instance **déjà installée** — celle de Guy en dogfooding — les champs existeraient en base et sur le PDF sans qu'aucun écran ne permette de les renseigner. Une fonctionnalité livrée que personne ne peut activer.

### D5 — Validation : longueur bornée, et **rien de plus**

Les trois champs sont du texte libre affiché sur un document. La validation se limite à une **longueur maximale** cohérente avec la largeur disponible du bloc gauche (le bloc droit des métadonnées commence à `meta_x = 120.0`, la marge gauche est à `20.0` — soit **100 mm** de large).

**Précédent de longueur du dépôt** : `contacts.phone` est `VARCHAR(50)` (`20260414000001_contacts.sql:13`) et `companies.email` `VARCHAR(320)`. S'y aligner plutôt que de dériver une longueur des millimètres.

**Ne PAS valider le format** du téléphone (les formats internationaux sont innombrables) ni du site web (un utilisateur écrira aussi bien `example.ch` que `https://example.ch`). Un rejet de saisie sur un champ purement décoratif coûterait plus qu'il ne rapporte.

⚠️ **L'e-mail, lui, est déjà validé** par la route dédiée existante — ne pas dupliquer cette validation.

## Acceptance Criteria

**AC1 — Migration.** Deux colonnes nullables sur `companies` (téléphone, site web). DDL pur, **aucune écriture de données**. Non-breaking : **pas** de bump de `kesh_version_min_required` ni de version Cargo.

**AC2 — Garde-fous de migration.** Ligne ajoutée à `docs/migrations-idempotence-audit.md` **à l'intérieur du tableau**, et les compteurs **recomptés depuis la source** — les deux sites du total et les trois compteurs de partition, dont la somme doit égaler le total. Le total passe de **58** à **59**. Garde-fou **P6** : `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` et inspection de chaque site — **DEUX** sites couplés de `migrations_upgrade_path.rs` doivent être bumpés **ensemble** : `assert_eq!(total, 58, …)` (`:89-93`, macro sur plusieurs lignes) → **59**, ET `let n_before_upgrade_window = total - 24;` (`:142`) → **`- 25`**, de sorte que la **frontière reste à 34**. ⚠️ Bumper le premier seul fait glisser la fenêtre testée de 34 à 35 **sans qu'aucune assertion ne le détecte** — le test continue de passer en mesurant moins. Le fichier documente lui-même cette règle de bump conjoint (`:104-119`), et 16-2a l'a appliquée (23 → 24). Garde-fou **P7** : ⚠️ la migration n'écrivant aucune donnée, elle ne relève **NI du registre `POST_RESTORE_BACKFILLS` NI des exemptions** — `every_data_backfill_migration_is_triaged` (`post_restore.rs:705`) fait `continue` sur toute migration dont aucun statement n'écrit, elle n'est donc **jamais sélectionnée**. **Ne rien y inscrire.** Reprendre mot pour mot la justification du précédent immédiat, la migration de 16-2a (`docs/migrations-idempotence-audit.md:80`). ⚠️ **Ne PAS recopier le marqueur `Hors fenêtre`** des exemptions voisines : il ferait échouer l'`assert_eq!(checked, 4)` d'`exemptions_claiming_out_of_window_really_are_out_of_window`.

**AC3 — API et les six listes de colonnes.** Les deux champs sont lus et écrits par une **route dédiée**, sur le patron de `update_company_email` (D4), avec verrou optimiste. `PUT /companies/current/email` **ne change ni de contrat ni de comportement** — mais son corps est **nécessairement** étendu, puisqu'il construit un `CompanyUpdate` (`companies.rs:106`) auquel T3 ajoute deux champs : le compilateur l'impose. `update_company_coordinates` n'est pas touchée (elle n'est pas un site `CompanyUpdate` ; seule sa constante `COMPANY_SELECT_FOR_UPDATE` change, liste n° 5 de T2). ⚠️ **Les SIX listes de colonnes `companies` écrites à la main sont mises à jour** (énumérées en T2) — une omission produit un `ColumnNotFound` **à l'exécution**, invisible à la compilation et au type-check.

**AC4 — Rendu PDF, les trois champs.** Téléphone, e-mail et site web sont rendus sous l'IDE dans le bloc émetteur, **chacun précédé de son libellé traduit**, et **uniquement s'ils sont renseignés** (D2). L'e-mail vient de la colonne existante.

**AC5 — L'avoir aussi, et le test doit être AU NIVEAU `kesh-api`.** Le rendu est identique sur le PDF d'avoir (D3), et **un test doit le prouver sur l'avoir spécifiquement**.

⚠️ **Le niveau n'est pas libre.** Facture et avoir partagent la **même** fonction de rendu (`draw_invoice_section`, appelée une seule fois par chaque générateur) : la seule divergence possible est au **site de construction**, `credit_notes.rs:277`. Un test posé dans `kesh-qrbill/src/pdf.rs` resterait donc **vert** sous la mutation prescrite — AC5 serait déclaré satisfait par un test muet, sur le défaut même que D3 qualifie de « piège qui coûterait le plus cher ». ⚠️ **Interdit d'hériter d'une fixture de facture par `..base`** : le nouveau champ serait hérité et le test passerait **sans exercer aucun code d'avoir**.

**AC6 — La garde de capacité haute existe et elle est prouvée.** Le bloc destinataire ne peut pas chevaucher le tableau. Un test monte le **pire cas** — adresse émetteur longue **et** les trois coordonnées renseignées **et** adresse destinataire longue — **et ce cas DOIT franchir le seuil**, l'issue attendue étant le **refus explicite**.

⚠️ **« Soit correctement rendu » ne suffit pas, et rendrait l'AC auto-annulant.** Le budget réel est large — le destinataire démarre au plus haut à `PAGE_H − 55` et le tableau à `PAGE_H − 130`, soit **75 mm** pour un bloc qui en consomme une trentaine. Un cas généreux mais sous le seuil se rend correctement, coche l'AC à la lettre, et rend la mutation « retirer la garde → le test rougit » **insatisfiable**. L'AC et la campagne de mutation ne sont conjointement satisfaisables que si le cas franchit réellement le seuil. ⚠️ **Un test qui se contente du cas nominal ne satisfait pas cet AC** : le cas nominal tient déjà aujourd'hui, il ne mesure rien.

**AC7 — i18n, et son tableau JUMEAU.** Trois nouvelles clés déclarées dans `I18N_KEYS` (`types.rs:202`) **ET dans `DEFAULT_EN` (`:233`), dans le MÊME ORDRE et en même nombre**, puis traduites dans **les 4 locales**.

⚠️ **L'i18n de cette story ne s'arrête PAS aux trois clés du PDF.** Le patron 20-3b2 qu'AC8 impose de reproduire a livré **trois clés de plus × 4 locales**, hors `I18N_KEYS` : libellé de champ, texte d'aide, et **message de validation** (`error-company-email-invalid`, consommé côté Rust). D5 imposant une validation de longueur, il en faut au moins un équivalent. ⚠️ Le `msg('clé', 'repli français')` du Svelte rend un manque **silencieux en français** — c'est ainsi que la KF **#283** s'est constituée.

⚠️ **Omettre `DEFAULT_EN` ne dégrade pas le rendu : ça le fait PANIQUER.** `QrBillI18n::get` résout son repli par **indexation positionnelle** — `I18N_KEYS.iter().position(...)` puis `DEFAULT_EN[idx]` (`:187-188`), **sans borne-check applicatif**. Une clé présente dans le premier et absente du second sort de l'index et panique, **en debug comme en release**. Et le piège est amplifié par les tests : le chemin de production peuple toutes les clés, mais la fixture interne de `pdf.rs` utilise `QrBillI18n::default()` — un `HashMap` **vide** (`:935`) — et `tests/golden_test.rs:98` un `QrBillI18n::new(HashMap::new())` tout aussi vide. Les deux forcent **chaque** appel à traverser `DEFAULT_EN`. Le test de pire cas exigé par AC6 est donc le premier à paniquer. ⚠️ Le dépôt a une KF ouverte sur des clés absentes des locales non françaises (**#283**) — ne pas l'aggraver.

**AC8 — Frontend, ÉDITABLE sur une instance déjà installée.** Les deux champs sont éditables *inline* dans les réglages, sur le patron de l'e-mail (`settings/+page.svelte:175`), avec un texte d'aide disant qu'ils apparaîtront sur les factures. ⚠️ **L'onboarding ne se rejoue pas** : une saisie qui n'existerait qu'à l'onboarding rendrait la fonctionnalité inaccessible à toute instance existante — dont celle en production. C'est le volet qui conditionne l'utilité de toute la story.

⚠️ **CLAUSE DE PREUVE — un test d'ALLER-RETOUR est exigé** : écrire une valeur par la route, la relire par `GET /companies/current`, et vérifier qu'elle arrive **jusqu'à l'écran**. Sans lui, l'omission d'un des deux miroirs `CompanyJson` (cf. T7) passe tous les gates : la valeur est stockée, rendue sur le PDF, et invisible dans les réglages. AC8 était le seul critère « utilité » de cette story sans clause de preuve — contrairement à AC5 et AC6.

**AC9 — Documentation.** Manuel utilisateur mis à jour et **PDF régénéré** (`make fr`). CHANGELOG. README vérifié — l'Epic 16 reste « 🚧 En cours » tant que 16-3b n'est pas livrée, donc **probablement aucun changement**, mais la vérification se **trace** même si la conclusion est « rien à changer ».

**AC10 — Gate.** Backend + frontend + suite E2E, sur l'**état final**, exit 0, **verdict lu dans le log**. La story touchant `crates/kesh-db/migrations/`, le ciblage est **interdit**, y compris entre deux passes de revue.

## Tasks / Subtasks

- [x] **T1 — Migration** (AC1, AC2) — deux colonnes nullables ; ligne d'audit **dans le tableau** ; compteurs recomptés `58 → 59` aux **deux** sites du total ; partition recomptée.

  ⚠️ **DEUX sites couplés dans `migrations_upgrade_path.rs`, à bumper ENSEMBLE** : `assert_eq!(total, 58, …)` (`:89-93`) → **59**, ET `let n_before_upgrade_window = total - 24;` (`:142`) → **`- 25`**. La frontière doit rester à **34** (59 − 25). Bumper le premier seul fait glisser la fenêtre testée à 35 : **le test continue de passer en mesurant moins**, et rien ne le signale.

  ⚠️ **Triage P7 : NE RIEN INSCRIRE**, ni au registre `POST_RESTORE_BACKFILLS` ni aux `EXEMPT_MIGRATIONS`. Une migration DDL pure n'est jamais sélectionnée par le détecteur (`post_restore.rs:705`, `continue` si aucun statement n'écrit). Reprendre la justification du précédent immédiat, `docs/migrations-idempotence-audit.md:80`.
- [x] **T2 — Entité et les SIX listes de colonnes** (AC3) — champs sur `Company`, **puis les six listes de colonnes écrites à la main**, relevées et vérifiées :
  1. `crates/kesh-db/src/repositories/companies.rs:17` — `FIND_BY_ID_SQL`
  2. `crates/kesh-db/src/repositories/companies.rs:22` — `LIST_SQL`
  3. `crates/kesh-api/src/routes/onboarding.rs:688` — `SELECT` en ligne
  4. `crates/kesh-api/src/routes/onboarding.rs:853` — `SELECT` en ligne
  5. `crates/kesh-api/src/routes/onboarding.rs:908` — `COMPANY_SELECT_FOR_UPDATE`
  6. `crates/kesh-seed/src/lib.rs:96-97` — `SELECT` **en ligne** (l'appel est en `:96`, la liste de colonnes en `:97`), propre au seed de démonstration

  ⚠️ **Une omission ne casse pas la compilation : elle produit un `ColumnNotFound` À L'EXÉCUTION**, et met en 500 la route concernée. C'est le piège de `FIND_BY_ID_SCOPED_SQL` de la story 16-2a, ici **multiplié par six**.

  ⚠️ **Le 6ᵉ site n'est PAS du code mort** : `seed_demo` est appelé en production par la route de démonstration de l'onboarding (`onboarding.rs:191`, exposée en `POST` via `lib.rs:713`). L'oublier casse la création de démo, un chemin qu'aucun test de facturation n'emprunte.

  Commande de contrôle : `grep -rn "query_as::<_, Company>\|query_as::<_, kesh_db::entities::Company>" crates/` rend **11** sites — chacun doit être rattaché à l'une des six listes.
- [x] **T3 — Route dédiée** (AC3, D4) — sur le patron de `update_company_email` (`companies.rs:86`) : verrou optimiste, validation de longueur (D5). **Ne pas** toucher `PUT /companies/current/email`, **ne pas** réutiliser `update_company_coordinates` (son `is_stub = FALSE` inconditionnel et son appelant unique sont des invariants documentés).

  ⚠️ **Ce que « le patron de `update_company_email` » implique exactement** : cette route ne fait **pas** une `UPDATE` ciblée. Elle **reconstruit un `CompanyUpdate` complet** depuis l'état courant (`companies.rs:106-117`) puis appelle `companies::update`, qui exécute un **full-replace** sur toutes les colonnes. Suivre ce patron impose donc d'étendre **trois** sites supplémentaires : la struct `CompanyUpdate`, la liste de colonnes de son `UPDATE` (`repositories/companies.rs:179-187`) et `is_no_op_change` (`:123-133`, énumération à la main).

  *Atténuation* : les 8 sites qui construisent `CompanyUpdate { ... }` sont vérifiés **par le compilateur** — l'omission échoue au build, pas en silence. C'est la seule des listes de cette story qui soit protégée ainsi.
- [x] **T4 — Rendu PDF** (AC4, AC5) — champs sur `InvoicePdfData`, rendu conditionnel sur le patron de l'IDE, **aux deux sites de construction** (D3).
- [x] **T5 — Garde de capacité haute** (AC6) — garde + test de pire cas. C'est la tâche à risque de la story ; la traiter **avant** le frontend.
- [x] **T6 — i18n et son tableau JUMEAU** (AC7) — 3 clés × 4 locales, déclarées dans `I18N_KEYS` (`types.rs:202`) **ET dans `DEFAULT_EN` (`:233`), même ordre, même nombre**. ⚠️ Omettre le second ne dégrade pas le rendu : il le fait **paniquer** (indexation positionnelle sans borne-check, `:187-188`).
- [x] **T7 — Frontend et les DEUX miroirs de DTO** (AC8) — édition *inline* sur le patron de l'e-mail, texte d'aide, **et les clés i18n du frontend** — libellé, aide et **message de validation**, ces trois-là étant décrites dans **AC7** et non dans T6, qui ne couvre que le couple `I18N_KEYS`/`DEFAULT_EN` du PDF.

  ⚠️ **`CompanyJson` est un miroir écrit à la main, en DEUX exemplaires, qu'AUCUN compilateur ne vérifie contre `Company`** :
  1. `crates/kesh-api/src/routes/companies.rs:28-47` — la struct Rust ;
  2. `crates/kesh-api/src/routes/companies.rs:49-68` — son `impl From<Company>`, énuméré **champ par champ** ;
  3. `frontend/src/lib/features/settings/settings.types.ts` — l'interface TypeScript ;
  4. `frontend/src/lib/features/settings/settings.api.ts` — le client ;
  5. `crates/kesh-api/src/lib.rs` (voisinage de `:266`) — l'enregistrement de la route.

  **Omettre le `From` ne casse RIEN à la compilation** : la base stocke, le PDF affiche, et l'écran de réglages affiche `—` **pour toujours**. `npm run check` ne le voit pas davantage — il valide le type TypeScript contre lui-même, jamais contre le backend. C'est la même forme de piège que les six listes SQL de T2, sur la seule couture qu'elles ne couvrent pas.
- [x] **T8 — Documentation** (AC9) — manuel + PDF régénéré, CHANGELOG, vérification README **tracée**.
- [x] **T9 — Gate complet** (AC10) — état final, exit 0, verdict lu dans le log.

## Dérogation à la règle de splitting préventif

*(Garde-fou déclenché en passe 2 de `validate` — **dérogation arbitrée par Guy le 2026-08-05**.)*

**Le critère est formellement rempli** : la sévérité maximale a **augmenté** entre deux passes (passe 1 `HIGH` → passe 2 `CRITICAL`), ce que la § *Règle de splitting préventif* définit comme une non-convergence réelle.

**Justification de la dérogation — ce que les `CRITICAL` étaient réellement.** Les quatre findings de la passe 2 sont **des résidus de la remédiation de la passe 1**, et **aucun ne porte sur une décision de conception** : le document se contredisait parce que j'avais corrigé les **critères d'acceptation** sans propager aux **tâches** qui les exécutent — « ne rien inscrire » contre « inscrire une exemption », « six listes » dans un corps de tâche contre « CINQ » dans son titre. Un découpage n'aurait ni évité ni corrigé ces incohérences : elles naissent d'un geste de patch incomplet, pas d'une surface trop large.

**Ce qui soutient l'analyse, en fait vérifiables :**

- **Les cinq décisions D1–D5 n'ont pas bougé depuis la création de la story.** Les deux passes n'ont contesté aucune d'elles ; elles ont corrigé des **décomptes**, des **ancres** et des **contradictions internes**.
- **La story tient sur un seul mental-model** : une entité (`companies`), une migration, un bloc du PDF. Le découpage par entité a **déjà eu lieu** — 16-3b porte `contacts` séparément.
- **Le seul découpage encore possible serait par couche**, et il produirait le défaut que 16-2 a payé : une première moitié livrant des colonnes que rien n'affiche, donc invérifiable seule et contrainte à une PR groupée.

**Risque accepté et sa condition de sortie.** Le risque est qu'une largeur réelle se cache derrière ces résidus et ne se révèle qu'en revue de code. **Condition de sortie explicite** : si la **passe 3** remonte encore un finding `CRITICAL` ou `HIGH`, la dérogation tombe et le split par couche devient la réponse — la démonstration serait alors faite par la mesure, non par le pronostic.

**Précédent** : story **16-2b**, où le garde-fou s'était déclenché sur un compteur `MEDIUM` stagnant, où la dérogation avait été arbitrée de la même façon, et où le **résultat l'a validée** — le compteur est tombé à **zéro** à la passe suivante.

### Prolongation d'une passe — arbitrage du 2026-08-06

**La condition de sortie ci-dessus a été ATTEINTE en passe 3** (1 `HIGH` : le septième miroir, `CompanyJson`). La dérogation aurait donc dû tomber. **Guy l'a prolongée d'exactement une passe**, sur trois faits établis par la mesure et non par le pronostic :

1. **La trajectoire est décroissante** — `4H/1M` → `3C/1M` → `1H/4M`.
2. **Aucun finding ≥ MEDIUM des trois passes n'a contesté une décision D1–D5.** Les cinq ont été reconfrontées au code à chaque passe et tiennent à la ligne près.
3. **Le `HIGH` n'est pas un symptôme de largeur** : `CompanyJson` est à la **charnière API↔frontend**, qu'un découpage par couche placerait sur la **couture** — l'endroit le moins surveillé des deux moitiés. Le split l'aggraverait au lieu de le fermer.

⚠️ **NOUVELLE CONDITION DE SORTIE, et elle est la dernière** : si la **passe 4** remonte encore un finding `CRITICAL` ou `HIGH`, le split est **démontré nécessaire par deux mesures consécutives** et devient inconditionnel — sans nouvel arbitrage. Si elle converge (rien au-dessus de `LOW`), la dérogation est validée par le résultat, comme elle l'a été pour 16-2b, et la story passe au développement.

### Review Findings

`bmad-code-review` **passe 1** — 2026-08-07, **Sonnet** (modèle ≠ implémentation), trois lentilles, diff aplati `main...HEAD`. **1 HIGH, 3 MEDIUM, 2 LOW**, tous confirmés au ground-truth. **Boucle NON convergée — passe 2 due.**

⚠️ **Le HIGH et deux MEDIUM sont des CONVERGENCES de plusieurs lentilles**, ce qui leur donne du poids.

**Patches**

- [x] [Review][Patch] **(CORRIGÉ — `truncate_display` appliqué au bloc identité, `IDENTITY_MAX_CHARS = 50`, calibré comme les 45 caractères / 90 mm de la colonne description ; test + mutation 6, rayon 1)** **La garde de capacité ne surveillait que le VERTICAL — une seule ligne trop longue débordait sur le bloc de droite, en silence** — `edge+blind+auditor`, **HIGH**. `grep -cF "text_width" pdf.rs` → **0**. Un site web de 255 caractères — la borne que la story fixe elle-même — ou un e-mail de 320 (`VARCHAR(320)`, **jamais borné à la saisie**) s'imprimait par-dessus « Facture / n° / date » (`meta_x = 120`), voire hors page (`PAGE_W = 210`), **rendu en 200**. Le dépôt possédait déjà `truncate_display` (`:903`), employé pour les descriptions de ligne mais **pas** pour l'identité. ⚠️ **Aucun des trois tests d'origine n'exerçait cet axe** : tous empilaient des *lignes*, aucun n'employait une valeur *unique* longue.
- [x] [Review][Patch] **(CORRIGÉ aux TROIS sites — repli Rust, CHANGELOG, manuel + PDF régénéré)** **Le message de refus accusait les seules coordonnées de la société, alors que l'adresse du DESTINATAIRE peut être seule en cause** — `edge+auditor`, **MEDIUM**. La garde réagit à une ordonnée qui cumule adresse émetteur, coordonnées **et** adresse destinataire. Une société sans aucune coordonnée déclenche le refus via une adresse client de quinze lignes — et l'utilisateur était renvoyé raccourcir des champs **vides** dans les réglages.
- [x] [Review][Patch] **(CORRIGÉ — clé ajoutée aux 4 locales)** **`settings-company-contact-saved` n'existait dans AUCUNE locale** — `blind+auditor`, **MEDIUM**. Un admin en DE/IT/EN voyait le message de succès **en français**. C'est exactement le mécanisme que la story cite comme cause de la **KF #283** — le `msg('clé', 'repli français')` rend le manque silencieux — et `lint-i18n-ownership` ne l'attrape pas : il contrôle le **périmètre** des clés, jamais leur **existence**.
- [x] [Review][Patch] **(CORRIGÉ — second `<p>` d'aide ajouté sous le champ)** **`settings-company-website-help` traduite ×4 mais jamais affichée** — `edge+auditor`, **MEDIUM**. Le champ site web n'avait aucun texte d'aide ; seule l'aide « téléphone » s'affichait, sous les deux champs.
- [x] [Review][Patch] **(CORRIGÉ — et la migration rejouée sur la base de gate, son checksum ayant changé)** **Le commentaire de la migration affirmait que 255 caractères tiennent sur 100 mm — faux d'un facteur quatre** — `blind`, **LOW**. Et ce n'est pas une coquette : **c'est cette affirmation qui avait laissé passer l'absence de troncature**. Le commentaire distingue désormais longueur de **stockage** et longueur d'**affichage**.
- [x] [Review][Patch] **(CORRIGÉ — aligné sur le Rust)** **Le commentaire du DTO TypeScript disait encore que `version` ne sert qu'à la route e-mail** — `blind`, **LOW**.

**Consigné, non retenu**

- **Aucun test frontend pour l'édition inline** — `blind`, **LOW**. Le flux (édition / annulation / conflit de verrou / visibilité Admin) n'est exercé par aucun test Vitest ni E2E. **Non retenu** : la story ne l'exigeait pas, et AC8 couvre le chemin qui compte — l'aller-retour HTTP, seul niveau qui voit le DTO. À verser à un lot « couverture frontend » (cf. KF #126, déjà ouverte sur ce sujet).

---

`bmad-code-review` **passe 2** — 2026-08-07, **Haiku 4.5** (rotation Opus → Sonnet → Haiku), trois lentilles, diff aplati. **0 HIGH, 3 MEDIUM, 3 LOW.** Sévérité maximale en **décroissance** (HIGH → MEDIUM) : le garde-fou de splitting n'est pas déclenché. **Boucle NON convergée — passe 3 due.**

**Deux lentilles sur trois rendent 0 finding au-dessus de LOW** : l'Acceptance Auditor après avoir vérifié une à une les six corrections de la passe 1 et recompté les invariants ; le Blind Hunter après contrôle des zones patchées.

**Patches**

- [x] [Review][Patch] **(CORRIGÉ — test `each_route_preserves_the_fields_it_does_not_touch`, mutation 7 : rayon 1)** **Le full-replace n'était testé dans AUCUNE des deux directions** — `edge`, **MEDIUM**. Les deux routes `PUT /companies/current/*` reportent les champs qu'elles ne modifient pas, sinon `companies::update` les efface. Le Dev Agent Record **nommait ce piège** — « modifier son e-mail aurait effacé le téléphone » — et le code le fermait, mais **aucun test ne le gardait**. Si le report disparaissait, éditer son téléphone effacerait l'adresse de réponse des factures, en `200`, avec bump de `version` et une entrée d'audit normale. Le test ferme les deux sens.
- [x] [Review][Patch] **(CORRIGÉ — test `overlong_contact_details_are_rejected_by_the_api`)** **La borne de longueur n'était vérifiée que par le `maxlength` du navigateur** — `edge`, **MEDIUM**. Aucun test n'exerçait le refus côté API. Un appel direct ignore le `maxlength`, et MariaDB tronquerait en silence.

**Reclassé, non retenu**

- **Le budget de troncature est partagé entre le libellé i18n et la valeur** — `edge`, **MEDIUM → LOW, non retenu**. Le libellé fait 3 à 6 caractères selon la locale et le champ (`Web`, `Tél.`, `E-mail`, `Phone`), donc la valeur affichée varie d'autant. **Ce n'est pas un défaut, c'est l'invariant** : ce qui doit tenir dans les ~100 mm est la **ligne rendue**, pas la valeur seule. Un libellé plus long laisse mécaniquement moins de place — le contraire serait le bug. Corriger en réservant un budget fixe à la valeur **casserait** la garantie de largeur.
- **La clé de conflit optimiste est partagée avec le formulaire e-mail** — `blind`, **LOW**. `settings-company-email-conflict` est réutilisée par le formulaire des coordonnées. Le message est exact et l'utilisateur ne voit aucune incohérence ; créer une clé jumelle au texte identique ajouterait deux traductions par locale sans rien apporter.
- **Aucun test frontend de l'édition inline** — `edge`, **LOW**, déjà consigné en passe 1. Couvert au niveau qui compte par l'aller-retour HTTP d'AC8. À verser au lot « couverture frontend » (**KF #126**).

---

`bmad-code-review` **passe 3** — 2026-08-07, **Opus 5** (rotation Sonnet → Haiku → Opus complète), trois lentilles, diff aplati. **1 HIGH, 4 MEDIUM, 5 LOW.** **Boucle NON convergée — passe 4 due.**

**Le HIGH est le défaut le plus subtil de la story, et rien ne pouvait le voir sauf une lentille qui suit le chemin jusqu'à l'écran.**

- [x] [Review][Patch] **(CORRIGÉ — variant `AppError::InvoicePdfHeaderOverflow` dédié, code `INVOICE_PDF_HEADER_OVERFLOW` enregistré dans la liste blanche du frontend)** **Le message de refus n'atteignait JAMAIS l'utilisateur sur l'écran facture** — `edge`, **HIGH**. `HeaderOverflow` était mappé sur `AppError::Validation`, dont le code `VALIDATION_ERROR` **ne figure pas** dans `PDF_ERROR_KEYS` (`invoices/[id]/+page.svelte:551-557`) : la clé retombait sur `invoice-pdf-error-generic` — « Erreur lors du téléchargement du PDF. » — et `i18nMsg` privilégiant la valeur FTL, le message soigné était **jeté**. Les **quatre traductions étaient mortes sur ce chemin**, pendant que le manuel **et** le CHANGELOG promettaient « un message explicite ». Asymétrie révélatrice : l'écran **avoir** affiche `err.message` et recevait donc le bon message — la facture non. Le patron correct était à trois lignes : `TooManyLines` a son variant dédié et son code propre.
- [x] [Review][Patch] **(CORRIGÉ — `git rm --cached` des 15 chemins + règles `.gitignore`)** **Quinze fichiers VIDES d'environnement avaient été commités** — `blind+auditor`, **MEDIUM**. `docs/.claude/*`, `docs/manual/.claude/*`, `.mcp.json` — les masques de bind-mount du bac à sable, entrés par un `git add -- docs/` trop large **au commit de la passe 1**. Mergés, `docs/manual/.claude/skills` et `hooks` deviendraient des **fichiers de 0 octet là où un répertoire est attendu**, et un `settings.json` vide écraserait la configuration sur tout clone frais. ⚠️ Le `.gitignore` ne portait **aucune** règle `.claude` — rien n'empêchait la récidive.
- [x] [Review][Patch] **(CORRIGÉ — 50 → 46, calculé sur les métriques AFM Helvetica)** **`IDENTITY_MAX_CHARS` compte des CARACTÈRES, pas une largeur — la troncature elle-même débordait** — `edge`, **MEDIUM**. Helvetica est proportionnelle : à 9 pt, 50 caractères font 77,8 mm en minuscules mais **107,5 mm en capitales**, soit 7 mm **au-delà** du bloc de droite — après troncature. Le calibrage d'origine (« 2 mm par caractère ») était une **fausse précision**. 46 tient les capitales moyennes (98,9 mm). ⚠️ Ce que la borne ne couvre pas — 46 `W` feraient 124 mm — est désormais **écrit**, plutôt que laissé croire.
- [x] [Review][Patch] **(CORRIGÉ — deux tests Vitest sur `settings.api.test.ts`)** **Aucun test ne couvrait le chemin API du frontend** — `edge`, **MEDIUM**. Le seul lien entre l'écran et la route est un littéral de chemin. Une faute de frappe, un renommage côté Rust : `npm run check` valide le TypeScript contre lui-même, les tests Rust construisent leur propre URL — **aucun gate ne le verrait**, et le défaut n'apparaîtrait qu'en 404 sur « Enregistrer ». Le jumeau e-mail avait ce test ; celui-ci ne l'avait pas.
- [x] [Review][Patch] **(ÉPINGLÉ par `an_omitted_field_clears_it_just_like_null` + doc-comment)** **Une clé ABSENTE du payload efface la valeur, comme `null`** — `edge+blind`, **MEDIUM**. `#[serde(default)]` rend l'absence indistinguable de `null`, et l'écriture est un full-replace : n'envoyer qu'un des deux champs efface l'autre, en `200`. Le frontend envoie toujours les deux, ce qui borne le risque aux clients API — mais le doc-comment ne mentionnait que « `null`/vide ». Comportement **hérité du patron e-mail**, donc épinglé et documenté plutôt que changé.
- [x] [Review][Patch] **(CORRIGÉ)** **Cinq LOW** — l'invariant `I18N_KEYS.len() == DEFAULT_EN.len()`, documenté comme fatal, n'était tenu par **rien** : désormais une `const _: () = assert!(…)` qui échoue au `cargo build` et non au premier PDF. Le doc-comment de la route de l'avoir avait été capté par le helper extrait en passe 1 — rendu à son handler. Le commentaire « marge de 2 mm » décrivait mal la géométrie (l'écart réel au seuil est de 6 mm, `y` étant la position libre suivante). Une chaîne vide arrivée par un autre chemin qu'à la route produisait une **ligne orpheline** consommant 4 mm — le rendu teste désormais la vacuité, pas la nullité.

**Consigné, non retenu**

- **Le nom et l'adresse de l'émetteur ne sont pas tronqués non plus** — `auditor`, **LOW**. Même axe que le HIGH de la passe 1, mais sur des champs **antérieurs à cette story** : aucune régression introduite, et le manuel ne parle que des coordonnées. À verser à une issue distincte si l'on veut fermer l'axe entièrement.
- **Les caractères hors Windows-1252 sont supprimés en silence** par `printpdf` sur les polices intégrées — `edge`, **LOW**. Exposition bornée : accents, apostrophes typographiques et le `…` de la troncature sont tous dans cp1252. Effet de bord favorable — la troncature compte **avant** la suppression, donc elle reste conservatrice.

---

`bmad-code-review` **passe 5** — 2026-08-07, **Haiku 4.5**, trois lentilles, diff aplati. **0 HIGH, 2 MEDIUM, 4 LOW** — dont **un MEDIUM reclassé** et un seul retenu. **Sévérité maximale en DÉCROISSANCE** (HIGH → MEDIUM). Le **Blind Hunter rend 0 finding**.

**Patches**

- [x] [Review][Patch] **(CORRIGÉ — la borne est désormais testée des DEUX CÔTÉS)** **50 et 255 caractères n'étaient testés qu'au-delà (51, 256), jamais à la valeur exacte** — `edge+auditor`, **LOW relevé DEUX fois** (passes 4 et 5). Un `>` changé en `>=` refuserait des valeurs parfaitement légales avec un message « trop long » **faux**, et la suite resterait verte. ⚠️ **Le test a échoué à sa première exécution — et il avait raison** : mon montage réutilisait la même `version` d'une itération à l'autre, alors qu'une acceptation la bumpe ; le second tour rendait `409`, que j'aurais pris pour un rejet de longueur. La version est relue à chaque tour.
- [x] [Review][Patch] **(CORRIGÉ — recomptée depuis la source : 33 fichiers)** **La File List sous-estimait de 9 fichiers** — `auditor`, **MEDIUM**. Elle annonçait 28 et ignorait tout ce que les passes de revue avaient ajouté : `errors.rs`, `settings.api.test.ts`, `invoices/[id]/+page.svelte`, `.gitignore`. **Troisième occurrence du même symptôme** sur cette story — après le décompte des tests et celui des clés i18n. La § *Propagation post-patch* décrit exactement ce mode d'échec : corriger le site signalé sans regreper le récapitulatif qu'on rend faux.

**Reclassé, non retenu**

- **La troncature n'est testée que sur un champ** (`creditor_website`) — `edge`, **MEDIUM → non retenu**, et la raison est structurelle : `truncate_display` n'est appelé **qu'une seule fois**, dans une **boucle unique** qui traite les quatre champs (`grep -c "truncate_display"` dans la boucle → **1**). Une asymétrie entre champs est donc **impossible par construction** — la produire exigerait de sortir un champ de la boucle, ce qui n'est pas une mutation réaliste. Le même argument vaut pour les trois LOW voisins : l'IDE non testé isolément, la troncature non testée sur l'avoir (dont le rendu est **partagé**, `draw_invoice_section` appelée une fois par générateur), et la couverture multi-locale (le budget partagé libellé/valeur ayant déjà été reclassé en passe 2).

**Gate de la passe 5 — exécuté intégralement le 2026-08-08, APRÈS le redémarrage qui l'avait interrompu.** Il remplace la mention « GATE NON TERMINÉ, NE PAS LE PRÉSUMER VERT » du commit `d102977f`.

| Étape | Verdict |
|---|---|
| `cargo fmt --all -- --check` | **0** |
| `cargo clippy --workspace --all-targets -- -D warnings` | **0** |
| `cargo build --workspace --all-targets` | **0** |
| `cargo nextest run --workspace --no-fail-fast` | **2125 / 2126**, 4 skipped, 56 min — **1 échec**, cf. ci-dessous |
| `npm run check` | **0** (4880 fichiers, 27 warnings préexistants) |
| `npm run lint-i18n-ownership` | **0** |
| `npm run test:unit` | **0** — **512 / 512**, 63 fichiers |
| `npm run build` | **0** |

⚠️ **CE GATE N'EST PAS DÉCLARÉ VERT, et le mot est pesé** : aucun run complet n'est allé au bout sans échec. L'unique échec est `reconciliation_e2e::post_reject_marks_transactions_as_manually_reviewed` (**409 au lieu de 200**, `reconciliation_e2e.rs:1428`), **imputé au flake KF-038 / issue #228** sur quatre éléments concordants — il passe **3/3** rejoué seul, **25/25** avec toute sa suite, la branche ne touche **aucun** fichier de réconciliation ni bancaire (`git diff --name-only main...HEAD | grep -iE "reconcil|bank"` → vide), et le `CLAUDE.md` documente cette famille comme flake sous contention MariaDB. Un échec non reproductible reste un échec **observé** : il est consigné, pas effacé.

⚠️ **LE RUN 1 A ÉCHOUÉ SUR UN DÉFAUT D'ENVIRONNEMENT, PAS SUR LE CODE — ET C'EST LE GATE INTERROMPU QUI L'AVAIT CAUSÉ.** `test_check_constraint_rejects_debit_and_credit_same_line` échouait en `InactiveOrInvalidAccounts`, **déterministe** (8 ms, seul). Cause : le compte `1000 Caisse CI` était resté `postable = FALSE` **en base de développement** — le helper `set_postable` (`journal_entries.rs:3063`) bascule cette colonne en SQL direct et **confie la restauration à l'appelant**, lequel n'a jamais eu la main quand le gate de la passe 5 a été tué. Le test suivant qui prend « les deux premiers comptes actifs » tombait dessus. Vérifié avant correction : ce compte n'avait **ni parent ni sous-compte** (sa non-postabilité n'était donc pas la conséquence légitime de la règle `is_postable`), le schéma le veut `DEFAULT TRUE`, et il était le **seul** compte actif non-postable de toute la base. Restauré, puis `PASS (1443/2126)` dans le run complet — pas seulement en isolation.

**Ce que cet incident enseigne, et qui dépasse la story** : ces tests-là ne sont pas des `#[sqlx::test]` sur base éphémère mais des `#[tokio::test]` sur la **base de développement partagée**. Un gate interrompu n'y laisse pas seulement un verdict inachevé — il peut y laisser un **piège armé**, qui fera rougir le run suivant sur un test sans aucun rapport avec ce qu'on mesurait. Ce mode d'échec est invisible en revue de diff, au même titre que les garde-fous **P6** et **P7** : il ne naît ni du code ni de la spec, mais de l'état d'un environnement partagé. Un contrôle de propreté de la base en pré-vol du gate fermerait la fenêtre — arbitrage à porter au Project Lead, hors périmètre de cette story.

## Dev Notes

### Ce que cette story ne doit PAS faire

- **Ne pas réimplémenter le récapitulatif TVA** — il est livré (PR #267). L'issue #151 le demandait à une époque où il n'existait pas.
- **Ne pas traiter le numéro de client** — c'est 16-3b.
- **Ne pas déplacer `ty = PAGE_H - 130.0`** (D1) — cette constante aligne le tableau avec la zone QR.
- **Ne pas changer le contrat ni le comportement de `PUT /companies/current/email`** (D4) — elle sert le flux d'envoi d'e-mail de l'Epic 20. *(Son corps, lui, est nécessairement étendu au nouveau champ de `CompanyUpdate` — le compilateur l'impose, cf. T3.)*
- **Ne pas valider le format** du téléphone ni du site web (D5).
- **Ne pas ajouter de route dédiée par champ.**

### Le piège qui coûterait le plus cher

**Oublier le second site de construction.** Une revue qui ne regarde que la facture ne verra jamais que l'avoir sort sans coordonnées. Le grep de propagation est `grep -rn "creditor_ide" crates/kesh-api/src/` — il rend **deux** sites, et c'est ce nombre qui doit être retrouvé après le patch.

### Comment tester un PDF dans ce dépôt — lire AVANT d'écrire T4/T5

⚠️ **Le dépôt ne compare JAMAIS le contenu textuel d'un PDF**, et c'est une décision prise avec Guy : *« Plan C : on ne compare pas l'octet exact du PDF »* (`crates/kesh-qrbill/tests/golden_test.rs:1-7`), `printpdf` introduisant un aléa dans le trailer `/ID`. Les tests existants ne vérifient que `bytes.starts_with(b"%PDF-1.")` et la **stabilité de taille**.

⚠️ **Et un `grep` naïf du texte échouerait de toute façon** : le contenu est **hex-encodé** dans les opérateurs `Tj` — « Robert Schneider SA » y apparaît comme `<526F62657274205363686E6569646572205341>`, jamais en ASCII.

**Les trois AC se testent donc à des niveaux différents, et il faut choisir le bon :**

- **AC6** — par le `Result` : `Err(QrBillError::…)`. Aucun contenu à inspecter, c'est le plus simple des trois. *(Bonus vérifié : `map_qrbill_error` est un `match` **exhaustif sans bras `_`** (`invoice_pdf_service.rs:316-344`), réutilisé par l'avoir — un nouveau variant d'erreur serait donc forcé par le compilateur sur les deux générateurs.)*
- **AC4** — par **delta de taille d'octets**, sur le précédent direct `pdf_size_stable_with_fixed_date`.
- **AC5** — ⚠️ **exige un petit refactor, et c'est la seule tâche qui en demande un.** La facture a déjà une fonction pure extraite, `build_qrbill_inputs` (`invoice_pdf_service.rs:162`) — privée, non-`async`, prenant des entités par référence, donc **testable sans base**. L'avoir n'a pas d'équivalent : il construit son `InvoicePdfData` **en ligne dans le handler `async`** (`credit_notes.rs:267`). **Reproduire le patron `build_qrbill_inputs` côté avoir** est ce qui rend AC5 testable au niveau `kesh-api`, comme il l'exige.

### Conventions de test

Le dépôt attend qu'une story démontre que ses tests **discriminent**, pas seulement qu'ils passent : une **campagne de mutation** est la norme (cf. 16-2a et 16-2b). Pour cette story, au minimum :

- retirer le rendu d'un des trois champs → le test correspondant doit rougir, **et lui seul** ;
- retirer la garde de capacité haute → le test de pire cas d'AC6 doit rougir ;
- retirer le rendu sur l'**avoir** en le laissant sur la facture → le test d'AC5 doit rougir, et **aucun test de facture** ne doit bouger.

⚠️ **Commiter AVANT toute campagne de mutation.** `git checkout` ne restaure que ce que l'index connaît ; sur du travail non commité il **détruit** — quatre tests ont été perdus ainsi en 16-2a. À défaut, sauvegarder par copie explicite.

### References

- Issue **#151** — la source. Sa liste de champs date d'avant la livraison du récap TVA.
- **16-3b** — numéro de client, l'autre moitié. Indépendante : les deux peuvent partir séparément.
- `crates/kesh-qrbill/src/pdf.rs` — `draw_invoice_section`, bloc émetteur et chaîne verticale (D1).
- `crates/kesh-qrbill/src/types.rs` — `InvoicePdfData` et `I18N_KEYS`.
- `crates/kesh-api/src/routes/invoice_pdf_service.rs:279` et `credit_notes.rs:277` — les **deux** sites de construction (D3).
- `crates/kesh-api/src/routes/companies.rs` — route générale et route dédiée à l'e-mail (D4).
- **KF #283** — clés i18n absentes des locales non françaises ; ne pas aggraver (AC7).

## Dev Agent Record

### Agent Model Used

Opus 5 (`bmad-dev-story`, 2026-08-06).

### Ce que l'implémentation a appris, et qui corrige la spec

**La spec surestimait D3, et sous-estimait D2.**

- **D3 — « le piège qui coûterait le plus cher » n'existe pas.** Ajouter les trois champs à `InvoicePdfData` rend l'oubli du second site de construction **impossible** : le compilateur exige les deux. Le grep de propagation prescrit était inutile — le type l'avait déjà fait.
- **D2 portait le vrai risque, et il s'est réalisé.** En écrivant le rendu, une respiration de 2 mm posée **inconditionnellement** après le bloc d'identité aurait décalé le PDF de **toute société sans IDE ni coordonnées** — exactement ce que D2 interdit. Aucun test existant ne l'aurait vu. Corrigé en conditionnant la respiration à ce qu'au moins une ligne d'identité ait été dessinée : IDE seul = 4 + 2 = **6 mm**, l'ancien pas exactement ; rien = **0 mm**.

**Le piège réel était ailleurs et la spec ne le nommait pas** : `update_company_email` reconstruit un `CompanyUpdate` **complet** et `companies::update` est un *full-replace*. Sans report explicite, **modifier son e-mail aurait effacé le téléphone et le site web**. Le compilateur force à ouvrir le fichier ; il ne dit pas quoi y écrire.

### La campagne de mutation : 5 sur 5 — après correction de DEUX tests qui ne mesuraient rien

| Mutation | 1er jet | Après correction |
|---|---|---|
| 1 — neutraliser la garde haute | ✅ tuée, rayon 1 | — |
| 2 — retirer le rendu du téléphone | ❌ **survécue** | ✅ rayon 1 |
| 3 — respiration inconditionnelle | ❌ **survécue** | ✅ rayon 1 |
| 4 — retirer les coordonnées du seul avoir | ✅ tuée, rayon 1 | — |
| 5 — oublier les champs dans l'`impl From` du DTO | ✅ tuée, rayon **2** | — |

**Les deux survivantes sont l'enseignement de cette story.** Les trois tests étaient **verts** ; deux ne prouvaient rien.

- **Mutation 2** — « les trois coordonnées ensemble > aucune » reste vrai avec **deux** champs rendus sur trois. Corrigé par un cas **par champ**, chacun comparé au même témoin.
- **Mutation 3** — un décalage de 2 mm **déplace le texte sans changer sa taille en octets** : aucun delta ne pouvait l'attraper, et le test comparait deux générations entre elles, ce qui ne mesure que le **déterminisme**. Il fallait mesurer là où 2 mm changent un **verdict** : au seuil de la garde haute. Calibrage — 8 lignes d'adresse émetteur (pour passer sous le plafond `min`, sinon l'écart est masqué) et 15 côté destinataire : `y = 170,5` qui passe contre `168,5` qui refuse.

La **mutation 5** rend `2` et c'est le bon nombre : les deux tests d'aller-retour lisent le DTO. Elle verrouille le `HIGH` de la passe 3 de `validate` — sans ces tests, une valeur **stockée** et **rendue sur le PDF** resterait **invisible** dans l'écran de réglages, tous gates au vert.

### Deux incidents de méthode, consignés

1. **Un timeout de commande a laissé le fichier de production MUTÉ sur disque** (mutation 4, premier essai). Vérifié et restauré immédiatement. **La restauration doit être dans la MÊME commande que la mutation** — jamais dans une commande suivante, qui peut ne jamais s'exécuter. Les mutations suivantes ont été lancées en arrière-plan, restauration incluse.
2. **La base de gate `kesh_gate2` était en retard d'une migration** (58 contre 59). Attrapé par le **contrôle de cinq secondes** avant de lancer une heure de tests — le même symptôme avait coûté 38 minutes la veille sur `kesh_gate`. `cargo sqlx migrate run` avant le gate.

### Debug Log References

- Gate backend : `scratchpad/gate-16-3a.log` — base `kesh_gate2`.
- Gate frontend : `scratchpad/gate-front-16-3a.log`.

### Completion Notes List

**T9 — GATE.** ⚠️ **Cette section a été RECTIFIÉE en passe 4 de revue** : elle affirmait encore les chiffres du gate initial (`2122`, « +6 tests », « aucun test frontend ») alors que trois passes de revue avaient depuis ajouté des tests. C'est exactement ce que la § *Test Locally First* interdit — un record que les passes suivantes lisent pour argent comptant. Le symptôme est celui de la § *Propagation post-patch* : chaque passe a corrigé le site signalé sans regreper le récapitulatif de composition qu'elle rendait faux.

**Gate initial (T9, avant la boucle de revue)** : `2122 tests run: 2122 passed`, 4 skipped, exit 0, DB `kesh_gate2`.

**Évolution au fil de la boucle**, chaque chiffre lu dans son log :

| Étape | Total | Delta |
|---|---|---|
| Gate T9 initial | **2122** | +6 (les tests de l'implémentation) |
| Après passe 1 | **2123** | +1 — test de troncature |
| Après passe 2 | **2125** | +2 — full-replace bidirectionnel, borne API |
| Après passe 3 | **2126** | +1 — clé omise = effacement |

**Composition finale, recomptée depuis la source** (`git diff main | grep -cE "^\+\s*#\[(sqlx::)?test"`) : **10 tests Rust**, non 6 —

- `kesh-qrbill/src/pdf.rs` : **4** (chaque coordonnée rendue isolément, aucune ligne d'identité = aucun espace, en-tête débordant refusé, valeur trop longue tronquée) ;
- `kesh-api/src/routes/credit_notes.rs` : **1** (l'avoir porte les coordonnées) ;
- `kesh-api/tests/companies_e2e.rs` : **5** (aller-retour, effacement, full-replace bidirectionnel, borne de longueur, clé omise).

Plus **2 tests frontend** (`settings.api.test.ts`) — le Record affirmait « cette story n'introduit ni warning ni test frontend », ce qui est **faux depuis la passe 3**.

**Gate frontend** : `check` 0 erreur sur 4880 fichiers, `lint-i18n-ownership` PASS, `build` 0, unitaires **512/512** (et non 510 : les 2 tests du client API s'y ajoutent).

⚠️ **La suite E2E Playwright n'a PAS été exécutée.** Sa baseline est rouge et démontrée telle, et cette story n'ajoute aucun scénario. Elle reste à jouer avant le push. Ce Record ne déclare que ce qui a tourné.

### File List

*(Recomptée depuis `git diff --name-only main`, pas de mémoire — **33 fichiers** hors story files.)*

⚠️ **Ce décompte a été faux deux fois** : annoncé 28 à l'implémentation, il ignorait les fichiers ajoutés par les passes de revue — `errors.rs` (variant dédié, passe 3), `settings.api.test.ts` (2 tests, passe 3), `invoices/[id]/+page.svelte` (liste blanche, passe 3), `.gitignore` (passe 3 puis 4). C'est le symptôme de la § *Propagation post-patch* : chaque passe corrigeait son site sans regreper le récapitulatif qu'elle rendait faux. Recompté depuis la source en passe 5.

**Nouveau**

- `crates/kesh-db/migrations/20260806000001_companies_phone_website.sql`

**Base de données** — `entities/company.rs` (`Company` + `CompanyUpdate`), `repositories/companies.rs` (2 listes de colonnes, `is_no_op_change`, `UPDATE` + binds), `tests/companies_repository.rs` (6 sites), `tests/migrations_upgrade_path.rs` (**double** bump + historique).

**API** — `routes/companies.rs` (route dédiée, `CompanyJson`, report à l'identique dans `update_company_email`), `lib.rs` (enregistrement), `routes/onboarding.rs` (3 listes de colonnes), `routes/credit_notes.rs` (**`build_credit_note_pdf_data` extraite** + module de tests), `routes/invoice_pdf_service.rs` (construction facture + mapping `HeaderOverflow`), `routes/invoice_email.rs` et `exports/metadata.rs` (fixtures), `tests/companies_e2e.rs` (**5** tests). Plus, ajoutés en revue : `errors.rs` (variant `InvoicePdfHeaderOverflow`), et côté frontend `settings.api.test.ts` (**2** tests) et `invoices/[id]/+page.svelte` (code d'erreur dans la liste blanche).

**PDF** — `kesh-qrbill/src/types.rs` (`InvoicePdfData`, `I18N_KEYS`, `DEFAULT_EN`, `HeaderOverflow`), `src/pdf.rs` (rendu conditionnel, troncature de largeur, garde haute, **4** tests), `tests/golden_test.rs` (fixture).

**Seed** — `kesh-seed/src/lib.rs` (6ᵉ liste de colonnes + coordonnées de démonstration).

**i18n** — les 4 locales, **11** clés chacune (recomptées : les 3 du PDF, les 4 des réglages, le message de succès, et les 3 messages d'erreur).

**Documentation** — `docs/migrations-idempotence-audit.md` (ligne + compteurs recomptés), `docs/manual/fr/user-manual.tex` + `.pdf` (55 pages), `CHANGELOG.md`, `README.md`.

## Change Log

**2026-08-06 — Passe 4 de `bmad-create-story validate` — ✅ BOUCLE CONVERGÉE, DÉROGATION VALIDÉE PAR LE RÉSULTAT** (**Sonnet**, contexte frais). **0 CRITICAL, 0 HIGH, 2 LOW** — critère d'arrêt atteint.

**Trend des quatre passes : `4H/1M` → `3C/1M` → `1H/4M/4L` → `0/0/2L`.** La condition de sortie inscrite en passe 3 est satisfaite **dans le sens favorable** : le split n'a pas lieu d'être, et c'est la **mesure** qui le dit, non le pronostic. Rotation complète Opus → Sonnet → Haiku → Opus → Sonnet ; plafond de 8 passes jamais approché.

**Les 2 LOW ont été intégrés plutôt que seulement consignés**, le premier étant trop actionnable pour être perdu :

- **Aucune technique n'était indiquée pour tester le contenu d'un PDF**, alors que le seul précédent du dépôt les **écarte explicitement** : *« Plan C : on ne compare pas l'octet exact du PDF »*. Et un `grep` du texte échouerait de toute façon — le contenu est **hex-encodé** dans les opérateurs `Tj`. La story porte désormais la technique **par AC** : AC6 par le `Result`, AC4 par delta de taille, et **AC5 par un refactor** — la facture a une fonction pure extraite (`build_qrbill_inputs`), l'avoir construit sa donnée **en ligne dans un handler `async`**, donc intestable en l'état. C'est le seul refactor que la story demande, et il conditionne la testabilité d'AC5.
- **Deux imprécisions d'ancrage** corrigées : le site 6 de T2 (`:96-97`, l'appel et la liste de colonnes étant sur deux lignes) et le renvoi de T7 vers T6, qui ne couvre que le couple `I18N_KEYS`/`DEFAULT_EN` du PDF — les clés du frontend sont décrites dans AC7.

**Ce que les quatre passes n'ont jamais contesté** : les cinq décisions **D1–D5**. Elles ont été reconfrontées au code à chaque passe et tiennent à la ligne près. Les 20 findings ont porté sur des **décomptes**, des **ancres**, des **clauses de preuve** et des **contradictions internes** — jamais sur la conception. C'est ce qui justifiait la dérogation, et c'est ce que le résultat confirme.

**2026-08-05 — Passe 3 de `bmad-create-story validate`** (**Opus 5**, rotation Opus → Sonnet → Haiku → Opus complète, contexte frais). **1 HIGH, 4 MEDIUM, 4 LOW**, tous confirmés au ground-truth et remédiés. ⚠️ **La condition de sortie de la dérogation est ATTEINTE** — arbitrage requis, analyse ci-dessous.

**Le HIGH est un septième miroir, sur la seule couture que la story ne surveillait pas.** `CompanyJson` existe en **deux** exemplaires écrits à la main — la struct Rust et son `impl From<Company>` énuméré champ par champ (`companies.rs:28-68`), et l'interface TypeScript (`settings.types.ts`) — dont **aucun compilateur ne vérifie la correspondance** avec l'entité. Omettre le `From` ne casse ni `cargo build` ni `npm run check` : la base stocke, le PDF affiche, et **l'écran de réglages affiche `—` pour toujours**. C'est la forme exacte du piège que T2 ferme six fois pour le SQL, laissée ouverte une septième — et elle frappe AC8, que la story désigne elle-même comme « le volet qui conditionne l'utilité de toute la story ». AC8 était par ailleurs le **seul critère d'utilité sans clause de preuve** ; il en porte désormais une, un test d'aller-retour.

**Les quatre MEDIUM portent tous sur la PRÉCISION des clauses de preuve, aucun sur une décision** :

- **AC5 ne disait pas à quel niveau tester l'avoir** — et le site le plus naturel, `pdf.rs`, ne peut **structurellement pas** discriminer : facture et avoir partagent la même fonction de rendu, la seule divergence possible étant au site de construction. Un test posé là serait resté vert sous la mutation prescrite, déclarant AC5 satisfait par un test muet, sur le défaut que D3 qualifie de « piège qui coûterait le plus cher ». L'héritage `..base` d'une fixture de facture est désormais explicitement interdit.
- **AC6 admettait « soit correctement rendu »**, ce qui rendait l'AC auto-annulant : un cas généreux mais sous le seuil coche le critère à la lettre et rend la mutation « retirer la garde » **insatisfiable**. Le cas de test doit désormais **franchir** le seuil.
- **AC3 interdisait de modifier `PUT /companies/current/email`** pendant que T3 imposait une extension de `CompanyUpdate` que le compilateur **force** dans le corps de cette route. Contradiction levée en distinguant *contrat* et *corps*.
- **AC7 bornait l'i18n aux trois clés du PDF**, alors que le patron 20-3b2 en a livré trois de plus hors `I18N_KEYS`, dont un message de validation — l'aggravation exacte de la KF #283 que la story demande d'éviter.

**Analyse pour l'arbitrage — ce que la mesure dit, et ce qu'elle ne dit pas.** La trajectoire est **décroissante** : `4H/1M` → `3C/1M` → `1H/4M`. **Aucun des findings ≥ MEDIUM des trois passes n'a contesté une décision D1–D5** ; les cinq ont été reconfrontées au code à chaque passe et tiennent à la ligne près. Et surtout : **le HIGH n'est pas un symptôme de largeur**. `CompanyJson` est à la charnière API↔frontend — un découpage par couche le placerait précisément **sur la couture**, c'est-à-dire à l'endroit le moins surveillé des deux moitiés. Le remède qu'il appelle est de l'**énoncer**, ce qui est fait, non de le partager en deux.

**2026-08-05 — Passe 2 de `bmad-create-story validate`** (**Haiku 4.5**, contexte frais). **3 CRITICAL, 1 MEDIUM**, tous **confirmés au ground-truth** et remédiés. **Boucle NON convergée — passe 3 due.**

⚠️ **GARDE-FOU DE SPLITTING FORMELLEMENT DÉCLENCHÉ** — la sévérité maximale a **augmenté** (passe 1 `HIGH` → passe 2 `CRITICAL`), ce que la § *Règle de splitting préventif* définit comme une non-convergence réelle. **Arbitrage requis** ; l'analyse est au paragraphe suivant.

**LES QUATRE FINDINGS SONT DES RÉSIDUS DE LA REMÉDIATION DE LA PASSE 1, ET AUCUN NE PORTE SUR LA CONCEPTION.** J'avais corrigé les **critères d'acceptation** sans propager aux **tâches** qui les exécutent, laissant le document se contredire :

1. **T1 prescrivait encore « triage P7 en exemption »** quand AC2, corrigé, disait « ne rien y inscrire ». Un dev suivant T1 aurait créé l'exemption et fait échouer un compteur codé en dur.
2. **T1 ne nommait qu'un des deux sites couplés au total** — le second, `n_before_upgrade_window`, n'avait été ajouté qu'à AC2. Or c'est T1 qu'on exécute.
3. **AC3 et le titre de T2 annonçaient « CINQ »** listes de colonnes quand le corps de T2, corrigé, en énumérait **six**.
4. **T6 ne mentionnait pas `DEFAULT_EN`**, ajouté au seul AC7.

**C'est le mode d'échec que la § *Propagation post-patch* décrit exactement** : un patch appliqué au site signalé, jamais grepé sur le reste du document. La contre-mesure a été appliquée cette fois — `grep -F` de « CINQ », « cinq listes » et « en exemption » sur tout le fichier après correction : **zéro occurrence résiduelle**.

**Analyse pour l'arbitrage.** La régression de sévérité est **réelle mais trompeuse** : ces `CRITICAL` sont des **incohérences internes que j'ai introduites en passe 1**, non des signes que la story serait trop large. Un split ne les aurait ni évitées ni corrigées — la story tient sur un seul mental-model (une entité, une migration, un bloc de PDF), et ses **décisions** n'ont pas bougé depuis la création. Le précédent applicable est **16-2b**, où le garde-fou s'était déclenché et où la dérogation, arbitrée puis **validée par le résultat**, avait vu le compteur tomber à zéro à la passe suivante.

**2026-08-05 — Passe 1 de `bmad-create-story validate`** (**Sonnet**, modèle ≠ celui qui a rédigé la story, contexte frais). **4 HIGH, 1 MEDIUM**, tous **confirmés au ground-truth** avant patch et tous remédiés. **Boucle NON convergée — passe 2 due.**

**Les quatre HIGH visaient tous des affirmations de la story, pas le code.** C'est le résultat attendu d'une validation : ce qui est faux dans un document se paie à l'implémentation.

1. **Le décompte des listes de colonnes était FAUX — six, pas cinq.** `crates/kesh-seed/src/lib.rs:96` porte une sixième liste, écrite en ligne, qu'aucune des cinq constantes ne couvre. Et **ce n'est pas du code mort** : `seed_demo` est appelé en production par la route de démonstration de l'onboarding. La commande de contrôle que la story prescrivait rend **11** sites, pas 10 — le chiffre était vérifiable et n'avait pas été vérifié.
2. **L'AC sur P7 était FAUX, et contredisait le précédent immédiat.** Il prescrivait d'inscrire la migration aux exemptions ; or `every_data_backfill_migration_is_triaged` fait `continue` sur toute migration sans écriture — une migration DDL pure ne relève **ni** du registre **ni** des exemptions. La ligne d'audit de 16-2a, **juste au-dessus** dans le même tableau, l'énonce déjà correctement. S'y ajoute un piège : recopier le marqueur `Hors fenêtre` de la voisine ferait échouer un compteur codé en dur.
3. **Un seul des deux sites couplés au total était nommé.** `migrations_upgrade_path.rs` porte `assert_eq!(total, 58)` **et** `n_before_upgrade_window = total - 24`. Bumper le premier seul fait glisser la fenêtre testée de 34 à 35 — **le test continue de passer en mesurant moins**, et rien ne le signale. Le fichier documente lui-même cette règle de bump conjoint.
4. **`I18N_KEYS` a un tableau jumeau que la story ne nommait pas.** `QrBillI18n::get` résout son repli par **indexation positionnelle** dans `DEFAULT_EN`, **sans borne-check** : trois clés ajoutées d'un seul côté font **paniquer** le rendu, en debug comme en release. Aggravé par les fixtures : toutes celles de `pdf.rs` utilisent un `HashMap` vide, donc **forcent** ce chemin — le test de pire cas exigé par AC6 aurait été le premier à tomber, sur un mode d'échec sans rapport avec ce qu'il mesure.

**Le MEDIUM porte sur une formule trop elliptique** : « reproduire le patron de `update_company_email` » cache que cette route ne fait pas une `UPDATE` ciblée mais **reconstruit un `CompanyUpdate` complet** en full-replace — soit trois sites de plus à étendre. Sévérité contenue parce que le compilateur les vérifie ; c'est la seule des listes de cette story à être protégée ainsi.

**2026-08-05 — Story créée** (`bmad-create-story`, Opus 5), née du découpage de **16-3** arbitré par Guy le même jour : 7 modules recensés contre un seuil de 5, la § *Règle de splitting préventif* imposait de scinder **avant** de lancer la spécification. Découpage **par entité** — 16-3a l'émetteur (`companies`), 16-3b le numéro de client (`contacts`) — préféré à un découpage par couche, qui aurait fait livrer à la première des colonnes que rien n'afficherait, défaut exact qui avait imposé la PR groupée de 16-2a/16-2b.

**Le relevé du schéma réel a changé la nature de la story.** #151 se lit comme une correction de mise en page ; en fait **trois des quatre champs n'existent nulle part en base**, et le quatrième — le récapitulatif TVA — **est déjà livré** depuis la PR #267. La story porte donc une migration, une route, un formulaire et un rendu, non une retouche.

**Le risque central est une garde manquante, pas un champ manquant** (D1) : le tableau des lignes démarre à une **ordonnée constante** que rien ne repousse, et la seule garde de capacité du fichier surveille le **plancher QR**, en bas. Allonger le bloc émetteur rapproche le destinataire du tableau **en silence**. D'où l'AC6, qui exige une garde symétrique **et** un test de pire cas — le cas nominal tenant déjà, il ne mesurerait rien.
