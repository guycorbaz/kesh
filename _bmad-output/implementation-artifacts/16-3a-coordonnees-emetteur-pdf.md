# Story 16.3a : Coordonnées de l'émetteur sur le PDF de facture

## Status

done

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

`bmad-code-review` **passe 4** — 2026-08-07, **Sonnet** (rotation Sonnet → Haiku → Opus → Sonnet), trois lentilles, diff aplati. **1 HIGH, 1 MEDIUM, 3 LOW.** Sévérité maximale **égale** à la passe 3 (`1H` → `1H`). **Boucle NON convergée — passe 5 due.**

⚠️ **Bloc reconstitué en passe 6, depuis le commit `d3883617`.** Il manquait : les blocs enchaînaient les passes 1, 2, 3 puis **5**, si bien que le HIGH de la passe 4 n'apparaissait dans aucun record, que l'en-tête de la passe 5 (« sévérité en DÉCROISSANCE HIGH → MEDIUM ») renvoyait à un HIGH introuvable dans le document, et que l'**égalité** de sévérité `passe 3 = passe 4 = 1 HIGH` — le critère même de la § *Règle de splitting préventif* — n'était lisible que dans `git log`. Quatrième occurrence du symptôme de propagation sur cette story, cette fois sur le récapitulatif des passes elles-mêmes. ⚠️ **Le message de commit n'étiquette pas les findings un à un** : le trend `1H/1M/3L` en vient, mais l'affectation de la sévérité à chacun des deux findings ci-dessous n'y figure pas et n'est **pas** reconstituée ici — la détail par finding est perdu.

**Les deux findings nommés visent les patches des passes précédentes, pas le code de la story :**

- **Le `.gitignore` de la passe 3 cassait plus qu'il ne réparait.** `.claude/` sans slash de tête matche **aussi la racine**, où 1714 fichiers sont légitimement versionnés (les 118 skills BMAD). Toute skill nouvellement installée serait devenue invisible à `git add`, sans que rien ne l'explique. Règle resserrée aux seuls sous-arbres réellement pollués, avec le pourquoi écrit dans le fichier. *(La passe 6 a constaté que ce resserrement n'avait jamais été propagé à la racine — cf. le patch `.gitignore` de la passe 6.)*
- **Le Dev Agent Record affirmait encore 2122 tests, « +6 » et « aucun test frontend »** — faux depuis trois passes. Rectifié avec le tableau d'évolution passe par passe (2122 → 2123 → 2125 → 2126) et la composition recomptée depuis la source.

**Gate de la passe 4** — complet et vert sur l'état final, verdict lu dans le log : `2126 tests run: 2126 passed (3 slow), 4 skipped — exit 0`. Le code n'avait pas bougé depuis son lancement ; seuls le `.gitignore` et le story file ont été touchés après, sans effet sur la compilation.

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

⚠️ **CE GATE N'EST PAS DÉCLARÉ VERT, et le mot est pesé** : aucun run complet **de la passe 5** n'est allé au bout sans échec. *(Portée précisée en passe 6 : en absolu la phrase serait fausse — le commit `d3883617`, passe 4, porte « 2126 tests run: 2126 passed, 4 skipped — exit 0 ».)* L'unique échec est `reconciliation_e2e::post_reject_marks_transactions_as_manually_reviewed` (**409 au lieu de 200**, `reconciliation_e2e.rs:1428`), **imputé au flake KF-038 / issue #228** sur quatre éléments concordants — il passe **3/3** rejoué seul, **25/25** avec toute sa suite, la branche ne touche **aucun** fichier de réconciliation ni bancaire (`git diff --name-only main...HEAD | grep -iE "reconcil|bank"` → vide), et le `CLAUDE.md` documente cette famille comme flake sous contention MariaDB. Un échec non reproductible reste un échec **observé** : il est consigné, pas effacé.

⚠️ **LE RUN 1 A ÉCHOUÉ SUR UN DÉFAUT D'ENVIRONNEMENT, PAS SUR LE CODE — ET C'EST LE GATE INTERROMPU QUI L'AVAIT CAUSÉ.** `test_check_constraint_rejects_debit_and_credit_same_line` échouait en `InactiveOrInvalidAccounts`, **déterministe** (8 ms, seul). Cause : le compte `1000 Caisse CI` était resté `postable = FALSE` **en base de développement** — le helper `set_postable` (`journal_entries.rs:3063`) bascule cette colonne en SQL direct et **confie la restauration à l'appelant**, lequel n'a jamais eu la main quand le gate de la passe 5 a été tué. Le test suivant qui prend « les deux premiers comptes actifs » tombait dessus. Vérifié avant correction : ce compte n'avait **ni parent ni sous-compte** (sa non-postabilité n'était donc pas la conséquence légitime de la règle `is_postable`), le schéma le veut `DEFAULT TRUE`, et il était le **seul** compte actif non-postable de toute la base. Restauré, puis `PASS (1443/2126)` dans le run complet — pas seulement en isolation.

**Ce que cet incident enseigne, et qui dépasse la story** : ces tests-là ne sont pas des `#[sqlx::test]` sur base éphémère mais des `#[tokio::test]` sur la **base de développement partagée**. Un gate interrompu n'y laisse pas seulement un verdict inachevé — il peut y laisser un **piège armé**, qui fera rougir le run suivant sur un test sans aucun rapport avec ce qu'on mesurait. Ce mode d'échec est invisible en revue de diff, au même titre que les garde-fous **P6** et **P7** : il ne naît ni du code ni de la spec, mais de l'état d'un environnement partagé. Un contrôle de propreté de la base en pré-vol du gate fermerait la fenêtre — arbitrage à porter au Project Lead, hors périmètre de cette story.

---

`bmad-code-review` **passe 6** — 2026-08-08, **Opus 5**, trois lentilles, diff aplati `main...HEAD` (38 fichiers, 2651 lignes). **1 HIGH, 8 MEDIUM, 8 LOW retenus** ; **4 findings réfutés au ground-truth**, 3 différés.

⚠️ **GARDE-FOU DE SPLITTING FORMELLEMENT DÉCLENCHÉ** — la sévérité maximale **augmente** (passe 5 : 0 HIGH / 1 MEDIUM retenu → passe 6 : **1 HIGH**), ce que la § *Règle de splitting préventif* définit comme une non-convergence réelle. **Arbitrage requis** ; analyse au bas de ce bloc.

**Décision — TRANCHÉE PAR GUY le 2026-08-08 : option (a), corriger LES DEUX ROUTES dans cette story.**

Le périmètre de 16-3a s'étend donc à `update_company_email` (Story 20-3b1), route existante et scellée. Motif de l'arbitrage : le défaut produit un **500 sur le parc installé antérieur à v0.5.0**, et la story est précisément celle qui invite ce parc à ouvrir l'écran concerné — laisser la route jumelle cassée aurait créé une asymétrie que rien ne justifie, et différer aurait laissé en production un défaut que cette story rend **plus atteignable**. La correction porte sur le site partagé, pas sur chaque route.

- [x] [Review][Decision] **Une société dont les colonnes d'adresse structurées sont vides part en 500 sur le nouvel écran** — `edge`, **MEDIUM**. `update_company_contact_details` reconstruit l'adresse par `company.structured_address()` (`companies.rs:221`), le repository écrit `addr.combined()` en full-replace (`repositories/companies.rs:197`), et `combined()` rend `""` quand les quatre composants sont vides (`entities/address.rs:34-46`). La contrainte `chk_companies_address_nonempty` (`20260404000001_initial_schema.sql:20`) rejette alors l'`UPDATE` → **500**. Vérifié : **aucune** migration ne backfille `companies` (`grep -l "UPDATE companies" migrations/*.sql` → vide), donc toute société créée **avant le 2026-07-05** (migration `structured_addresses`, v0.5.0) garde ses quatre colonnes à `''`. `is_no_op_change` ne protège pas — `phone`/`website` diffèrent, l'`UPDATE` part. Aucune fixture ne reproduit le cas : `test_fixtures.rs:85-89` peuple toujours les colonnes structurées. ⚠️ **Le défaut est HÉRITÉ de `update_company_email` (Story 20-3b1, même ligne `companies.rs:119`) — il n'est pas introduit ici**, mais il est reproduit sur une seconde route, et c'est celle-là que le CHANGELOG et le manuel invitent tout le parc existant à utiliser. **Trois issues possibles** : (a) corriger les deux routes dans cette story — élargit le périmètre à du code existant ; (b) ouvrir une issue GitHub et différer — conforme à la § *Issue Tracking Rule*, le défaut étant hors du flux normal de cette story ; (c) ne corriger que la route neuve — asymétrie difficile à justifier.

**Patches**

- [x] [Review][Patch] **Le site de construction FACTURE n'est asserté par AUCUN test — seul l'avoir l'est** — `edge`, **HIGH** [`crates/kesh-api/src/routes/invoice_pdf_service.rs:284`]. Vérifié au ground-truth : `grep -rn "creditor_phone" crates/` rend 4 sites, et la **seule** assertion est `credit_notes.rs:464` (l'avoir). Écrire `creditor_phone: None` à la ligne 284 ne fait rougir aucun test — le struct literal reste complet donc le compilateur se tait, les tests `kesh-qrbill` posent leurs propres valeurs, et `invoice_pdf_e2e.rs:228-235` ne contrôle que `status`, `content-type`, `%PDF-1.` et `bytes.len() > 1_000`. Conséquence : **toutes les factures sortiraient sans coordonnées pendant que les avoirs en porteraient**. C'est exactement la dissymétrie que la story nomme « le piège qui coûterait le plus cher » — le garde-fou existe, mais du seul côté du document **secondaire**.
- [x] [Review][Patch] **Le message d'erreur conseille une action qui ne libère aucune hauteur** — `blind`, **MEDIUM** [`crates/kesh-i18n/locales/*/messages.ftl`, 4 locales + `errors.rs`]. Le message dit « **Raccourcissez** le téléphone, l'e-mail ou le site web ». Or le débordement est **vertical** et chaque coordonnée coûte `y -= 4.0` (`pdf.rs:271`) **quelle que soit sa longueur** — la valeur est de toute façon tronquée à l'affichage (`pdf.rs:265`). L'utilisateur qui remplace `+41 21 123 45 67` par `+41211234567` obtient **exactement le même refus**, en boucle. Les seules actions efficaces sont de **vider** un champ ou de réduire le **nombre de lignes** de l'adresse. Propager aux 4 locales, au repli de `errors.rs`, au `user-manual.tex` et au `CHANGELOG.md`. Le message dit aussi « l'en-tête de **la facture** » alors que la garde s'applique aussi à l'avoir (`credit_notes.rs:341` passe par le même `map_qrbill_error`).
- [x] [Review][Patch] **La chaîne `INVOICE_PDF_HEADER_OVERFLOW` n'est vérifiée nulle part de bout en bout** — `edge`, **MEDIUM** [`errors.rs:1369` ↔ `invoices/[id]/+page.svelte:557`]. Les deux littéraux coïncident aujourd'hui — aucun défaut d'état. Ce qui manque, c'est ce qui les **tient** ensemble : le jumeau `INVOICE_TOO_MANY_LINES_FOR_PDF` a son test HTTP (`invoice_pdf_e2e.rs:335-352`), l'en-tête n'en a aucun — son seul test s'arrête à `matches!(err, QrBillError::HeaderOverflow(_))` (`pdf.rs:1210`) sans traverser `map_qrbill_error`, `IntoResponse` ni HTTP. Si la chaîne dérive, `PDF_ERROR_KEYS` retombe sur `'invoice-pdf-error-generic'` et les 4 traductions redeviennent mortes — **le HIGH de la passe 3 rejoué un cran plus bas**, gate vert.
- [x] [Review][Patch] **Sept nombres périmés dans le fichier anti-dérive canonique du dépôt** — `auditor`, **MEDIUM** [`crates/kesh-db/tests/migrations_upgrade_path.rs:38,40,57,58,106,113,115-116`]. Les deux sites **exécutables** sont correctement bumpés (`assert_eq!(total, 59)`, `n_before_upgrade_window = total - 25`), mais la documentation en vis-à-vis annonce toujours `total - 24`, `total == 58`, « les 24 restantes » et « le laisser à 23 ». Le précédent immédiat — la 16-2a — maintenait ces six sites ; 16-3a n'en a maintenu aucun. Ce fichier porte une section « DÉRIVE DOCUMENTAIRE CONSTATÉE » et la consigne « les nombres se recomptent, ils ne se relisent pas » : il affirme désormais une fenêtre de 24 quand le code en applique 25. *(`:216` est antérieur à cette branche — pas une régression de 16-3a.)*
- [x] [Review][Patch] **La passe 4 est absente du story file, alors qu'elle a remonté un HIGH** — `auditor`, **MEDIUM** [ce fichier, `:193`→`:247`]. Les blocs enchaînent passe 1, 2, 3 puis **5**. Le commit `d3883617` documente pourtant le trend `…→ passe 4 1H/1M/3L` et un HIGH qui visait une régression du patch de la passe 3. Conséquences : le HIGH n'apparaît dans aucun record ; l'en-tête de la passe 5 (« sévérité en DÉCROISSANCE HIGH → MEDIUM ») se réfère à un HIGH que le document ne contient pas ; et le critère de non-convergence (`passe 3 = 1H`, `passe 4 = 1H`, donc **égalité**) n'est lisible que dans `git log`. **Quatrième occurrence du symptôme « le récapitulatif n'est pas regrepé »** sur cette story — cette fois sur le récapitulatif des passes elles-mêmes.
- [x] [Review][Patch] **Le patch anti-masques du bac à sable n'a pas été propagé à la racine** — `auditor`, **MEDIUM** [`.gitignore`]. Le `.gitignore` livré ne vise que `docs/.claude/`, `docs/.mcp.json`, `docs/manual/.claude/`, `docs/manual/.mcp.json`. Vérifié : `git check-ignore -v .mcp.json .gitmodules .claude/settings.json .bashrc` ne rend **rien** — aucun n'est ignoré, et ce sont des fichiers de **0 octet**. Un `git add -A` à la racine reproduit l'incident de la passe 1 sur des chemins plus sensibles, dont `.gitmodules` (0 octet, **que git lit**). Le raisonnement de la passe 4 — ne pas écrire `.claude/` nu, la racine portant 1714 fichiers versionnés — est juste et interdit la règle générale, mais il ne dispensait pas d'énumérer les entrées réellement polluées.
- [x] [Review][Patch] **L'assertion de compilation ne garantit que la LONGUEUR, jamais la POSITION que son doc-comment promet** — `blind`, **MEDIUM** [`crates/kesh-qrbill/src/types.rs:238,252-253`]. Le commentaire énonce « Toute clé ajoutée ici DOIT l'être à la **MÊME POSITION** dans `DEFAULT_EN` » ; l'assertion ne compare que `I18N_KEYS.len() == DEFAULT_EN.len()`. Insérer une clé au **milieu** de l'un et ailleurs dans l'autre garde les longueurs égales, passe `cargo build`, et décale silencieusement le repli de toutes les entrées suivantes (`get` résout par `position()` puis `DEFAULT_EN[idx]`, `types.rs:193-194`). L'assertion élimine bien la panique d'index — le vrai gain — mais le doc-comment lui prête une garantie qu'elle n'a pas. Aggravé par les fixtures à `HashMap` vide, qui traversent toutes `DEFAULT_EN`.
- [x] [Review][Patch] **Aucun test n'exerce le seul écran de saisie de la story** — `blind`, **MEDIUM** [`frontend/src/routes/(app)/settings/+page.svelte`]. Le Vitest ajouté teste l'enveloppe HTTP (`settings.api.test.ts` mocke `apiClient`), pas le composant. Rien n'exerce `startContactEdit`, le mapping vide → `null` (`phoneValue.trim() || null`), la branche `OPTIMISTIC_LOCK_CONFLICT`, ni le gating `{#if isAdmin}`. Aucun `.spec.ts` Playwright n'existe alors que **tous les `data-testid` ont été posés** — des points d'ancrage créés pour un test qui n'existe pas. Supprimer le `.trim()`, inverser les deux `<Input>` ou retirer le `{#if isAdmin}` laisse la suite verte.
- [x] [Review][Patch] **Contradiction sur le verdict du gate, introduite par le bloc que je viens d'écrire** — `auditor`, **LOW** [ce fichier, bloc « Gate de la passe 5 »]. La phrase « aucun run complet n'est allé au bout sans échec » est vraie **de la passe 5** mais fausse en absolu : le commit `d3883617` (passe 4) porte « 2126 tests run: 2126 passed, 4 skipped — exit 0 ». Scoper explicitement la phrase à la passe 5.
- [x] [Review][Patch] **Trois commentaires de calibration faux ou périmés** — `blind+auditor`, **LOW** [`crates/kesh-qrbill/src/pdf.rs:183-189,253-255`]. (a) « une chaîne de 46 `W` occuperait **124 mm** » : la méthode du commentaire lui-même donne 46 × 0,944 × 9 pt × 0,35278 = **137,9 mm** (les deux autres lignes du tableau se vérifient exactement). (b) Le commentaire de la boucle affirme encore « ~**50** caractères à 9 pt » quand le doc-comment de `IDENTITY_MAX_CHARS` conclut « **D'où 46 et non 50** » — résidu de la recalibration de la passe 3. (c) Le doc-comment invoque la « **raison sociale** » pour justifier la borne, alors que le nom de l'émetteur est dessiné à 14 pt **hors** du dispositif de troncature.
- [x] [Review][Patch] **Le repli codé en dur perd l'information actionnable de la traduction** — `blind`, **LOW** [`crates/kesh-api/src/errors.rs`]. Le fallback dit « Le numéro de téléphone de la société est trop long. » là où la version FTL précise « (50 caractères au plus) ». Même écart pour le site web.
- [x] [Review][Patch] **L'ordonnée atteinte, seule donnée de diagnostic du refus, est jetée** — `blind+edge`, **LOW** [`invoice_pdf_service.rs:352`]. `HeaderOverflow(f32)` documente « ordonnée atteinte, en mm » ; le mapping l'écarte par `_` et `AppError::InvoicePdfHeaderOverflow` ne porte aucune charge, sans `tracing`. Le jumeau `InvoiceTooManyLinesForPdf(usize)`, lui, conserve son compte. Un exploitant ne sait ni de combien le document dépassait, ni si la cause est l'émetteur ou le destinataire — alors que le message rendu, lui, énumère les deux hypothèses.
- [x] [Review][Patch] **Le test d'aller-retour n'assert la nullité de départ que du téléphone** — `blind`, **LOW** [`crates/kesh-api/tests/companies_e2e.rs`]. Le montage vérifie `current["company"]["phone"].is_null()` mais jamais le site web, alors que l'assertion finale porte sur les deux. Si `create_test_company` posait un `website`, ce volet cesserait de mesurer l'écriture — le risque exact que le commentaire du test dit vouloir écarter.
- [x] [Review][Patch] **Le test de la borne exacte n'assert que le code HTTP** — `edge`, **LOW** [`crates/kesh-api/tests/companies_e2e.rs:1364-1394`]. Seule assertion : `assert_eq!(resp.status(), 200)`. Une régression de `normalize_contact_field` rendant `Ok(None)` au lieu d'`Err` pour une valeur pile à la borne resterait verte — le 200 est là, la valeur silencieusement perdue. Correctif d'une ligne : asserter la valeur dans le `CompanyJson` de la réponse.
- [x] [Review][Patch] **Deux écarts de périmètre et de forme** — `auditor`, **LOW**. (a) Le commit `26fd6d51` (rectificatif d'en-tête de l'Epic 19) est sur cette branche sans rapport avec #151 — **c'est moi qui l'y ai versé**, et il rend faux le recompte « 33 fichiers » de la File List, qui donne désormais 34. Quatre-vingt-dixième illustration du symptôme de propagation. (b) AC2 demandait de reprendre « mot pour mot » la justification du précédent immédiat ; la ligne livrée abandonne la clause « non-breaking → pas de bump `min_required` ni de version Cargo ». Sans conséquence fonctionnelle, le commentaire SQL la portant intégralement.
- [x] [Review][Patch] **Le formulaire de coordonnées réutilise les clés i18n du formulaire e-mail** — `blind`, **LOW** [`settings/+page.svelte`]. `settings-company-email-conflict` et son jumeau `-reload-failed` sont génériques aujourd'hui, donc corrects à l'affichage — mais toute reformulation pour le contexte e-mail s'affichera à tort sur le bloc téléphone/site web, sans qu'aucun test ne le voie.

**Différés**

- [x] [Review][Defer] **La mesure par delta d'octets repose sur une monotonie que rien ne garantit** — `blind`, LOW — différé : le choix est documenté et justifié (le dépôt ne compare jamais le contenu d'un PDF, cf. § *Comment tester un PDF*), la date est figée par `fixed_date()`. Fragilité signalée, pas défaut : un bump de `printpdf` changeant la compression en ferait un faux négatif silencieux.
- [x] [Review][Defer] **La garde transforme en 400 la régénération d'un PDF déjà émis** — `blind`, LOW — différé : effet de bord assumé et documenté au CHANGELOG. La marge est confortable (~15 lignes d'adresse destinataire pour franchir le seuil), mais le seul remède offert consiste à modifier la fiche contact, ce qui change le destinataire imprimé d'une facture déjà envoyée.
- [x] [Review][Defer] **AC4 « chacun précédé de son libellé traduit » n'a pas de preuve directe** — `auditor`, LOW — différé : retirer `i18n.get(key)` laisserait le test de delta vert. Atténuation structurelle vérifiée (`build_i18n` itère `I18N_KEYS` en entier, les 3 clés existent traduites ×4, l'invariant de longueur verrouille le repli) et la § *Comment tester un PDF* écarte l'inspection de contenu. La clause n'est tenue que **par construction** — c'est consigné, pas corrigé.

**Réfutés au ground-truth — 4, tous documentés pour que la passe 7 ne les rejoue pas**

- **« L'avoir n'est pas branché sur le mapping d'erreur »** (`blind`, MEDIUM) — **réfuté** : `credit_notes.rs:341` appelle `map_qrbill_error`, la fonction **partagée** définie en `invoice_pdf_service.rs:323`. L'avoir hérite du 400. Le Blind Hunter avait honnêtement marqué le point « non vérifiable depuis le diff » et fourni la commande de contrôle ; elle le réfute. *(Le volet « le message dit facture » survit, fusionné dans le patch du message.)*
- **« La route n'a aucun contrôle de rôle »** (`blind`, MEDIUM) — **réfuté** : la route (`lib.rs:273`) est déclarée dans `admin_routes` (`lib.rs:182`), dont le `route_layer` (`lib.rs:283`) applique `require_admin_role`. L'admin-only est porté par le **routeur**, comme le commentaire l'affirmait. Convergent avec la vérification indépendante de l'`edge`.
- **« Le `.gitignore` ne désindexe pas les fichiers déjà suivis »** (`blind`, LOW) — **réfuté** : `git ls-files docs | grep -E "\.claude|\.mcp\.json"` rend **vide**. Le `git rm --cached` a bien eu lieu. *(À ne pas confondre avec le MEDIUM sur la **racine**, lui bien réel.)*
- **« La ligne tronquée dépasse d'un caractère le budget »** (`blind`, LOW) — **réfuté** : `truncate_display` (`pdf.rs:954-962`) fait `take(max_chars - 1)` **puis** pousse l'ellipse — la chaîne rendue fait exactement `max_chars`, pas `max_chars + 1`. L'ellipse est incluse dans la borne.

**Gate de la passe 6 — ⚠️ INCOMPLET, ET LE MOT EST PESÉ.** *(✅ **LEVÉ le 2026-08-09** : le gate complet a été exécuté sur cet état — backend `2131/2131`, frontend `512/512`, suite E2E jouée en entier. Le constat ci-dessous décrit l'état **au moment de la passe 6** et est conservé tel quel ; le verdict qui fait foi est au Change Log du 2026-08-09.)* MariaDB n'était pas démarré au moment d'appliquer les patches (port 3306 non écouté). Ont réellement tourné, et sont verts :

| Ce qui a tourné | Verdict |
|---|---|
| `cargo fmt --all -- --check` (workspace) | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo test -p kesh-qrbill` | 57/57 |
| `cargo test -p kesh-i18n` | 21/21 |
| `cargo test -p kesh-api --lib` | 296 passed, **9 failed** |
| `npm run check` (frontend) | 0 erreur, 27 warnings préexistants |
| `npm run lint-i18n-ownership` + `npm run test:unit` | exit 0 |

Les **9 échecs** sont tous `auth::bootstrap::tests::*` et paniquent dans `sqlx-core-0.8.6/src/testing/mod.rs:226` — l'ouverture de la base de test. Aucun fichier `auth`/`bootstrap` n'est touché par la branche (`git diff --name-only main...HEAD` → vide sur ce motif) : c'est l'absence de MariaDB, pas une régression.

**N'ONT PAS TOURNÉ, et ne doivent donc pas être présumés verts** : toutes les suites `#[sqlx::test]` — dont `companies_e2e`, `companies_repository`, `invoice_pdf_e2e` et `migrations_upgrade_path`, c'est-à-dire **précisément celles que cette passe modifie** — ainsi que la suite Playwright, dont la spec `company-contact-details.spec.ts` **écrite à cette passe n'a jamais été exécutée une seule fois**. Un test jamais lancé ne rougit pas, il se tait. ⚠️ La § *Migration breaking policy* (P6/P7) interdit le ciblage dès qu'un patch touche `crates/kesh-db/` — ce qui est le cas ici (`migrations_upgrade_path.rs`) : **le gate complet est dû avant tout commit de clôture de boucle et avant tout push.**

*(✅ **Dette levée le 2026-08-09.** Les suites `#[sqlx::test]` ont toutes tourné — `2131/2131`, 4 skipped — et `company-contact-details.spec.ts` a été exécutée pour la première fois : **5/5**. Elle n'était donc pas muette.)*

**Analyse pour l'arbitrage du garde-fou de splitting.** La sévérité augmente (0 HIGH → 1 HIGH), ce qui déclenche formellement le second critère. **Ce que le HIGH est, et ce qu'il n'est pas** : c'est un **trou de couverture sur un site unique de six lignes** (`invoice_pdf_service.rs:284-286`), symétrique d'un test qui existe déjà et fonctionne pour l'avoir. Le remède tient en un test, sur le modèle exact de `credit_note_pdf_carries_the_issuer_contact_details`. Ce n'est **pas** le symptôme d'une story qui excéderait un mental-model : aucun des trois lentilles n'a pris en défaut une décision **D1–D5**, et l'Acceptance Auditor a reconfronté les cinq au code en les confirmant toutes. Un découpage par couche placerait précisément ce site **sur la couture** API↔PDF, c'est-à-dire à l'endroit le moins surveillé des deux moitiés — le même argument qui avait fondé la dérogation en passe 3 de `validate`. **Ce que la mesure dit aussi**, et qui va dans l'autre sens : cinq des neuf findings ≥ MEDIUM portent sur des **récapitulatifs devenus faux** (nombres de migrations, passes manquantes, `.gitignore` non propagé, commentaires de calibration, File List) — le symptôme de propagation post-patch est ici **structurel et répété**, non résiduel. Il ne se soigne pas par un split mais par un `grep` systématique du symptôme avant chaque passe.

---

`bmad-code-review` **passe 7** — 2026-08-09, **Sonnet** (rotation, la passe 6 était Opus 5), trois lentilles, diff aplati `main...HEAD` restreint au code et à la doc (35 fichiers, 2968 lignes ; `_bmad-output/` chargé comme **spec**, non comme contenu révisé). **1 HIGH, 4 MEDIUM retenus** ; **4 findings réfutés au ground-truth**, 2 différés.

⚠️ **GARDE-FOU DE SPLITTING À NOUVEAU DÉCLENCHÉ** — sévérité maximale **égale** à la passe 6 (`1 HIGH` → `1 HIGH`), ce que la § *Règle de splitting préventif* définit comme une non-convergence réelle. **Arbitrage requis** ; analyse au bas de ce bloc.

**Le HIGH n'est pas dans le code que la story a écrit : il est dans un consommateur qu'elle a oublié de prévenir.**

- [x] [Review][Patch] **(CORRIGÉ — variante déclarée au bras métier, `classify_render_error` EXTRAITE du handler pour être testable, test + mutation tuée, rayon 1)** **Le lot de rappels classe le refus d'en-tête en panne d'infrastructure — mauvais code d'erreur ET log mensonger** — `edge`, **HIGH** [`crates/kesh-api/src/routes/invoice_email.rs:1105-1119`]. Vérifié au ground-truth : `grep -nF "InvoicePdfHeaderOverflow" invoice_email.rs` rend **zéro**. Le `match` de la boucle per-facture énumère `InvoiceNotValidated`, `Database(NotFound)`, puis `InvoiceNotPdfReady(_) | InvoiceTooManyLinesForPdf(_)` — et fait tomber tout le reste dans `other => BatchItemError::infra(...)`. La variante `AppError::InvoicePdfHeaderOverflow`, **ajoutée par cette story en passe 3**, n'y a jamais été propagée. Conséquences : le client reçoit `error_code: "DATABASE_ERROR"` pour une facture dont rien n'est cassé, sans aucun moyen de savoir qu'il faut vider une coordonnée ou raccourcir l'adresse du destinataire ; et chaque occurrence — **déterministe**, fonction des seules données — écrit un `tracing::error!` « erreur d'infrastructure », là où `infra()` est documentée pour « pool mort, IO ». C'est exactement ce que la § *Pattern batch — FailedProposal per-proposal* du `CLAUDE.md` classe en **erreur métier per-proposal**. Le chemin unitaire (`send-email`) propage l'`AppError` et rend le bon code : la dissymétrie ne tient qu'à ce `match`. Le patron est donné par le **jumeau exact** `InvoiceTooManyLinesForPdf`, qui a lui aussi son variant et son code HTTP dédiés et se voit **groupé** ici sous `INVOICE_NOT_PDF_READY`.
- [x] [Review][Patch] **(CORRIGÉ — recompté depuis la source : 16, avec la répartition par fichier et la mention de la passe d'origine de chacun)** **La « Composition finale » du Dev Agent Record sous-compte les tests Rust de 5** — `auditor`, **MEDIUM** [ce fichier, `:459`]. Elle annonce **10 tests Rust** en citant sa commande de contrôle ; exécutée telle quelle, celle-ci rend **15**. Manquent les trois fichiers ajoutés par les patches de la passe 6 : `invoice_pdf_service.rs` (**2** — précisément les tests qui remédient le HIGH de la passe 6), `invoice_pdf_e2e.rs` (**1**) et `companies_repository.rs` (**2** de plus que les « 6 sites » comptés). ⚠️ Ces mêmes 5 tests sont **correctement comptés une ligne plus haut**, au tableau d'évolution du gate (`2126 → 2131, +5`) : le même bloc, écrit dans la même passe, les compte et les oublie.
- [x] [Review][Patch] **(CORRIGÉ — `invoice_pdf_e2e.rs` inscrit, `companies_repository.rs` rectifié, total recompté à 36)** **La File List omet `invoice_pdf_e2e.rs` et sous-compte `companies_repository.rs`** — `auditor`, **MEDIUM** [ce fichier, `:471-500`]. Vérifié : `invoice_pdf_e2e` n'apparaît qu'aux lignes 304, 306 et 348 — toutes dans des blocs de revue, **jamais dans la File List** —, alors que le Change Log du 2026-08-09 le nomme parmi les suites que la passe modifie. Et `companies_repository.rs` y est annoncé « 6 sites » sans refléter les 2 tests qu'y a ajoutés la Decision MEDIUM « société pré-#213 part en 500 ».
- [x] [Review][Decision] **(CORRIGÉ — whitelist `PDF_ERROR_KEYS` posée sur l'écran avoir, alignée sur la fiche facture ; `i18nMsg` retombe sur `err.message` pour tout code absent, donc aucun message n'est perdu.)** **(TRANCHÉE PAR GUY le 2026-08-09 : option (a), corriger sur l'écran avoir dans cette story.** Même arbitrage que celui rendu le 2026-08-08 sur la route jumelle, et pour le même motif : la story est celle qui rend ce chemin d'erreur atteignable, et laisser la moitié « avoir » afficher la mauvaise langue créerait une asymétrie que rien ne justifie — d'autant que le CHANGELOG promet l'identique.**)** **Le refus d'en-tête s'affiche sur l'écran AVOIR dans la langue du serveur, pas dans celle de l'utilisateur** — `edge`, **MEDIUM** [`frontend/src/routes/(app)/credit-notes/[id]/+page.svelte:99`]. Vérifié : cette page n'a **aucune** liste blanche (`grep -nF "PDF_ERROR_KEYS"` → rien) et affiche `err.message` tel quel — c'est-à-dire le message traduit **côté serveur**, dans la locale d'instance figée au démarrage, quelle que soit la langue d'interface. La page **facture** a reçu en passe 3 une entrée dédiée qui re-traduit le code côté client (`INVOICE_PDF_HEADER_OVERFLOW`, `:557`), précisément pour éviter ce défaut. Le CHANGELOG de la story affirme « les avoirs les portent également, **à l'identique** » : le refus l'est côté backend, sa **traduction** ne l'est pas. **La correction sort du périmètre de la branche** — ce fichier n'est pas touché par la story —, d'où l'arbitrage : (a) corriger ici, comme la passe 6 l'a fait pour la route jumelle sur décision de Guy ; (b) ouvrir une issue et différer ; (c) laisser tel quel, l'utilisateur voyant un message exact mais dans la mauvaise langue.

**Différés**

- [x] [Review][Defer] **Aucun test ne discrimine une permutation libellé↔valeur sur le bloc identité** — `blind`, **MEDIUM → différé** [`crates/kesh-qrbill/src/pdf.rs`, `tests/golden_test.rs:64-66`]. Vérifié : le golden met les **trois champs à `None`**, donc ne couvre aucun cas renseigné ; et les tests de rendu ne mesurent qu'un **delta d'octets**. Rendre le site web sous le libellé « Tél. » produirait un document de **taille identique** — tous les tests resteraient verts, et un client appellerait une URL. **Différé parce que c'est la même famille que le différé assumé de la passe 6** (« AC4 n'a pas de preuve directe, la clause n'est tenue que par construction ») : la § *Comment tester un PDF dans ce dépôt* écarte l'inspection du contenu, le texte étant hex-encodé dans les opérateurs `Tj`. Le remède réel est un élargissement du golden à un cas renseigné — à traiter comme un lot, pas au détour de cette story.
- [x] [Review][Defer] **Les caractères de contrôle ne sont pas filtrés dans `phone` / `website`** — `blind`, **LOW → différé** [`crates/kesh-api/src/routes/companies.rs`, `normalize_contact_field`]. Seul `trim()` est appliqué ; un `\n` interne est stocké tel quel et passé au rendu. **Différé parce que D5 tranche explicitement** — « validation : longueur bornée, **et rien de plus** » — et que la saisie est réservée aux admins. Y toucher serait revenir sur une décision, pas corriger un défaut.

**Réfutés au ground-truth — 4**

- **« L'avoir n'est pas branché sur le mapping d'erreur »** (`blind`, MEDIUM) — **réfuté pour la DEUXIÈME fois** : `grep -nF "map_qrbill_error" credit_notes.rs` rend `:27` (import) et `:341` (appel). La passe 6 l'avait déjà réfuté et **explicitement documenté pour que la passe 7 ne le rejoue pas** ; la lentille aveugle l'a néanmoins soulevé — honnêtement, en le marquant « doute » et en fournissant la commande de contrôle. Preuve que la consigne de non-rejeu ne protège pas une lentille qui, par construction, ne lit que le diff.
- **« Un appelant tiers de `companies::update` verrait son effacement d'adresse silencieusement annulé »** (`blind`, LOW/MEDIUM) — **réfuté** : `grep -rnF "companies::update("` rend **deux** sites, `companies.rs:133` et `:232` — les deux routes de la story. L'appelant hypothétique n'existe pas. Convergent avec la vérification indépendante de l'`edge`.
- **« Un site `SELECT ... FROM companies` a pu être oublié »** (`blind`, LOW) — **réfuté** : les seuls sites qui projettent une **liste de colonnes** vers `Company` sont les 6 déjà traités ; tous les autres lisent des scalaires (`COUNT`, `id`, `name`, `country`, `is_stub`). Convergent avec l'`edge`.
- **« Écart `maxlength` HTML (UTF-16) vs `chars().count()` (points de code) »** (`blind`, LOW) — **écarté comme bruit** : l'écart ne se manifeste que sur des caractères hors BMP dans un numéro de téléphone ou une URL, et joue dans le sens **prudent** (le navigateur est plus strict que l'API).

**Analyse pour l'arbitrage du garde-fou de splitting.** La sévérité n'augmente pas mais **ne décroît pas** — `1 HIGH` en passe 6, `1 HIGH` en passe 7 —, ce qui coche formellement le critère. **Ce que le HIGH est** : un consommateur non prévenu. La story a créé un variant d'erreur en passe 3 et ne l'a propagé qu'aux deux chemins qu'elle regardait ; un troisième, le lot de rappels, l'attrape par son bras `other`. **Ce n'est pas un symptôme de largeur** — c'est le symptôme, encore, de la § *Propagation post-patch* : un patch appliqué au site signalé sans greper le symptôme sur le dépôt. Un découpage par couche n'y changerait rien, `invoice_email.rs` étant dans la même couche que le site corrigé. **Ce que la mesure dit dans l'autre sens** : sur les 5 findings retenus, **3 portent encore sur des récapitulatifs devenus faux** — après que la passe 6 en ait déjà corrigé 5 de la même famille. Le taux ne baisse pas, et aucune des sept passes n'a jamais pris en défaut une décision **D1–D5**.

---

`bmad-code-review` **passe 8** — 2026-08-09, **Haiku 4.5** (rotation Sonnet → Haiku → Opus bouclée une troisième fois), trois lentilles, diff aplati `main...HEAD` (36 fichiers, 3144 lignes). **✅ BOUCLE CONVERGÉE — 0 finding retenu au-dessus de LOW.** Critère d'arrêt de la § *Review Iteration Rule* atteint **par convergence**, non par épuisement du plafond de 8 passes — ce qui n'allait pas de soi, la passe 8 étant la dernière autorisée.

**Deux lentilles sur trois rendent zéro.** L'Acceptance Auditor après avoir **recompté** — et non relu — les tests (16), les fichiers (36), les migrations (59) et les compteurs de partition ; l'Edge Case Hunter après avoir parcouru les chemins un à un, dont les deux correctifs de la passe 7, l'ordre des 18 `bind()` de l'`UPDATE` et les deux sens du *full-replace*.

⚠️ **Le Blind Hunter a rendu 1 CRITICAL et 1 HIGH. Les DEUX sont des hallucinations, réfutées au ground-truth — et leur cause commune mérite d'être écrite.**

- **« Les trois clés manquent à `I18N_KEYS` »** (CRITICAL) — **réfuté** : `types.rs:243` porte `"invoice-pdf-phone"`. Argument dirimant que l'agent n'a pas vu : l'assertion `const _: () = assert!(I18N_KEYS.len() == DEFAULT_EN.len())` (`types.rs:265`) **ferait échouer `cargo build`** en cas d'écart, or le gate est vert.
- **« Le struct `InvoicePdfData` n'a pas les trois champs »** (HIGH) — **réfuté** : `types.rs:132-134`. Là encore, le manque serait une **erreur de compilation**, pas un défaut runtime.
- **« Perte d'écriture d'adresse sous concurrence »** (MEDIUM) — **réfuté** : l'`UPDATE` est `WHERE id = ? AND version = ?` (`repositories/companies.rs:218-219`). Une écriture concurrente bump `version`, la seconde ne matche aucune ligne et part en conflit optimiste.

**La cause commune** : l'agent cherchait `DEFAULT_EN` « aux lignes 2371-2386 de `pdf.rs` », alors que ces définitions vivent dans `types.rs` autour de la ligne 265. Il a confondu les **offsets du fichier de patch** avec les **numéros de ligne des fichiers source**. ⚠️ **C'est la pathologie d'indexation documentée pour Haiku au `CLAUDE.md`, mais sous une forme que la mitigation prescrite ne couvre pas** : le diff aplati protège de la confusion multi-commit, **pas** de la confusion patch↔source. Les deux findings les plus graves de la passe naissent de cette seule erreur, et la discipline du `grep -nF` les a arrêtés tous les deux. Le seul LOW rendu — les clés de conflit du bloc coordonnées jugées « génériques » — porte sur des clés **dédiées créées en passe 6** ; non retenu.

**Trend des huit passes : `1H/3M/2L` → `0H/3M/3L` → `1H/4M/5L` → `1H/1M/3L` → `0H/2M/4L` → `1H/8M/8L` → `1H/4M` → `0`.** Rotation Sonnet → Haiku → Opus parcourue **trois fois** entièrement.

**Le garde-fou de splitting, déclenché aux passes 6 et 7, est validé par le résultat — comme il l'avait été pour la 16-2b.** Il s'était déclenché deux fois de suite (sévérité croissante en 6, puis égale en 7), et la dérogation a été maintenue à chaque fois sur le même argument : aucune des huit passes n'a **jamais** pris en défaut une décision **D1–D5**, et les deux HIGH portaient sur des **consommateurs oubliés** (`invoice_pdf_service.rs`, puis `invoice_email.rs`), c'est-à-dire sur le symptôme de la § *Propagation post-patch* — qu'un découpage n'aurait pas soigné, les sites concernés étant dans la même couche que le code corrigé. La passe 8 à zéro confirme l'arbitrage par la mesure, non par le pronostic.

**Ce que ces huit passes auront coûté et appris, en une phrase** : sur 60 findings retenus, la **remédiation elle-même** est la première source de défauts — récapitulatifs devenus faux, variants non propagés, tests qui ne discriminent rien — et c'est le `grep` du symptôme, jamais la relecture du site corrigé, qui les attrape.

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
| Après passe 6 — **gate de clôture, complet** | **2131** | +5 — les tests des patches de la passe 6 |

**Composition finale, recomptée depuis la source** (`git diff main | grep -cE "^\+\s*#\[(sqlx::)?test"`) : **16 tests Rust**.

⚠️ **Ce décompte a été faux DEUX fois de plus, et pour la même raison à chaque fois.** Il annonçait 6 à l'implémentation, puis **10** — et la passe 7 a exécuté la commande citée juste au-dessus pour lire **15**. Les trois fichiers manquants étaient tous nés des patches de la **passe 6**, c'est-à-dire de la passe qui écrivait ce paragraphe. Pire : ces mêmes tests étaient **correctement comptés une ligne plus haut**, au tableau d'évolution du gate (`2126 → 2131, +5`). Le même bloc, écrit dans la même passe, les comptait et les oubliait. Le 16ᵉ est celui de la passe 7.

- `kesh-api/tests/companies_e2e.rs` : **5** (aller-retour, effacement, full-replace bidirectionnel, borne de longueur, clé omise) ;
- `kesh-qrbill/src/pdf.rs` : **4** (chaque coordonnée rendue isolément, aucune ligne d'identité = aucun espace, en-tête débordant refusé, valeur trop longue tronquée) ;
- `kesh-db/tests/companies_repository.rs` : **2** (adresse préservée quand les colonnes structurées sont vides, adresse recomposée quand elles sont remplies) — *passe 6* ;
- `kesh-api/src/routes/invoice_pdf_service.rs` : **2** (la facture porte les coordonnées, elle omet celles que la société n'a pas) — *passe 6, remédiation de son HIGH* ;
- `kesh-api/tests/invoice_pdf_e2e.rs` : **1** (l'en-tête débordant rend 400 avec son code propre) — *passe 6* ;
- `kesh-api/src/routes/credit_notes.rs` : **1** (l'avoir porte les coordonnées) ;
- `kesh-api/src/routes/invoice_email.rs` : **1** (un refus de rendu n'est pas déguisé en panne) — *passe 7, remédiation de son HIGH*.

Plus **2 tests frontend** (`settings.api.test.ts`) — le Record affirmait « cette story n'introduit ni warning ni test frontend », ce qui est **faux depuis la passe 3**.

**Gate frontend** : `check` 0 erreur sur 4880 fichiers, `lint-i18n-ownership` PASS, `build` 0, unitaires **512/512** (et non 510 : les 2 tests du client API s'y ajoutent).

⚠️ **RECTIFIÉ le 2026-08-09 — cette section affirmait deux choses désormais fausses.** Elle disait « la suite E2E Playwright n'a PAS été exécutée » : elle l'a été, intégralement, et le détail est au Change Log du 2026-08-09. Elle disait aussi « cette story n'ajoute aucun scénario » : c'était vrai à l'implémentation, **plus depuis la passe 6**, qui a écrit `company-contact-details.spec.ts` (5 scénarios). Enfin, « sa baseline est rouge et démontrée telle » n'était adossée à **aucun chiffre** — elle en a un maintenant, mesuré sur `main` : **30 échecs** sur les 15 fichiers concernés.

### File List

*(Recomptée depuis `git diff --name-only main`, pas de mémoire — **36 fichiers** hors artefacts `_bmad-output/`. Le 36ᵉ est l'écran avoir, ajouté en passe 7 ; `invoice_email.rs`, également touché par cette passe, était déjà au périmètre depuis l'implémentation.)*

⚠️ **L'assiette du décompte est désormais explicite, parce que sa seule ambiguïté a produit deux nombres.** « Hors story files » ne disait pas si les autres artefacts BMAD comptaient : la passe 5 annonçait 33 en excluant tout `_bmad-output/`, et la passe 6 a lu 34 en y comptant le commit d'en-tête de l'Epic 19 (`26fd6d51`), qui vit précisément dans `_bmad-output/planning-artifacts/`. Le compte retenu **exclut tout `_bmad-output/`** et se reproduit par :

```sh
{ git diff --name-only main...HEAD; git diff --name-only HEAD; \
  git ls-files --others --exclude-standard; } | sort -u | grep -vc "_bmad-output/"
```

⚠️ **Ce décompte a été faux deux fois** : annoncé 28 à l'implémentation, il ignorait les fichiers ajoutés par les passes de revue — `errors.rs` (variant dédié, passe 3), `settings.api.test.ts` (2 tests, passe 3), `invoices/[id]/+page.svelte` (liste blanche, passe 3), `.gitignore` (passe 3 puis 4). C'est le symptôme de la § *Propagation post-patch* : chaque passe corrigeait son site sans regreper le récapitulatif qu'elle rendait faux. Recompté depuis la source en passe 5.

**Nouveau**

- `crates/kesh-db/migrations/20260806000001_companies_phone_website.sql`
- `frontend/tests/e2e/company-contact-details.spec.ts` — 5 scénarios Playwright sur l'écran de réglages *(passe 6 : les `data-testid` étaient posés depuis l'implémentation, sans aucun test pour s'y ancrer)*.

**Base de données** — `entities/company.rs` (`Company` + `CompanyUpdate`), `repositories/companies.rs` (2 listes de colonnes, `is_no_op_change`, `UPDATE` + binds, garde de préservation d'adresse), `tests/companies_repository.rs` (6 sites d'ajustement **plus 2 tests** ajoutés en passe 6), `tests/migrations_upgrade_path.rs` (**double** bump + historique).

**API** — `routes/companies.rs` (route dédiée, `CompanyJson`, report à l'identique dans `update_company_email`), `lib.rs` (enregistrement), `routes/onboarding.rs` (3 listes de colonnes), `routes/credit_notes.rs` (**`build_credit_note_pdf_data` extraite** + module de tests), `routes/invoice_pdf_service.rs` (construction facture + mapping `HeaderOverflow`), `routes/invoice_email.rs` (fixtures ; puis en passe 7 l'extraction de `classify_render_error` + **1** test), `exports/metadata.rs` (fixtures), `tests/companies_e2e.rs` (**5** tests), `tests/invoice_pdf_e2e.rs` (**1** test, passe 6). Plus, ajoutés en revue : `errors.rs` (variant `InvoicePdfHeaderOverflow`), et côté frontend `settings.api.test.ts` (**2** tests), `invoices/[id]/+page.svelte` (code d'erreur dans la liste blanche) et `credit-notes/[id]/+page.svelte` (whitelist de remappage, passe 7).

**PDF** — `kesh-qrbill/src/types.rs` (`InvoicePdfData`, `I18N_KEYS`, `DEFAULT_EN`, `HeaderOverflow`), `src/pdf.rs` (rendu conditionnel, troncature de largeur, garde haute, **4** tests), `tests/golden_test.rs` (fixture).

**Seed** — `kesh-seed/src/lib.rs` (6ᵉ liste de colonnes + coordonnées de démonstration).

**i18n** — les 4 locales, **13** clés chacune (recomptées depuis le diff : les 3 du PDF, les 4 des réglages, le message de succès, les 3 messages d'erreur, et les **2 clés de conflit de version** dédiées au bloc coordonnées, ajoutées en passe 6 — jusque-là il empruntait celles du bloc e-mail).

**Documentation** — `docs/migrations-idempotence-audit.md` (ligne + compteurs recomptés), `docs/manual/fr/user-manual.tex` + `.pdf` (55 pages), `CHANGELOG.md`, `README.md`.

## Change Log

**2026-08-10 — GATE DE CLÔTURE DE LA BOUCLE (passes 7 et 8) — backend et frontend VERTS, ET LES 3 ÉCHECS E2E LAISSÉS « NON ATTRIBUÉS » LE 2026-08-09 SONT ATTRIBUÉS.** Les patches des passes 7 et 8 touchaient du code exécutable et n'avaient eu qu'un gate ciblé ; ce gate-ci est complet. Chaque verdict est **lu dans son log**, jamais dans un code de sortie d'enveloppe.

| Gate | Verdict | Mesure |
|---|---|---|
| `fmt` + `clippy` workspace | ✅ vert | via `scripts/test-fast.sh` |
| Backend complet (nextest) | ✅ vert | `Summary [3885.743s] 2132 tests run: 2132 passed, 4 skipped` — 64,8 min, DB dev `kesh` |
| Frontend | ✅ vert | `check` 0 erreur / 27 warnings préexistants, `lint-i18n-ownership` PASS, **512/512** sur 63 fichiers, `build` OK |
| E2E — **suite complète, branche** | `rc=1` | **178 passed / 39 failed / 19 skipped**, 236 tests, 13,7 min |
| E2E — **suite complète, `main`** | `rc=1` | **172 passed / 40 failed / 19 skipped**, 231 tests, 14,7 min |
| **Différentiel branche ↔ `main`** | ✅ **nul** | 39 échecs **strictement communs** ; **0 échec propre à la branche** ; 1 échec propre à `main` |

**La comparaison à armes égales qui manquait au gate du 2026-08-09 a été faite** : suite **complète** des deux côtés, montages identiques, sur deux ports et deux bases distincts (`:3000`/`kesh_e2e` pour la branche, `:3001`/`kesh_e2e_main` pour `main` depuis un worktree sur `ddbea7c1`). C'est ce que la passe précédente désignait comme « seule comparaison réellement à armes égales », et elle tranche.

**L'arithmétique boucle exactement** : `172` (passed sur `main`) `+ 5` (les tests neufs de `company-contact-details.spec.ts`, tous verts) `+ 1` (`sidebar-navigation.spec:71`, qui échoue sur `main` et **passe** sur la branche) `= 178`. Aucun test ne se perd en route.

**Les trois échecs « non attribués » sont désormais attribués, et aucun n'est imputable à la branche.**

- **`invoices.spec:387` et `:411` — KF réelle, dépendante de L'HEURE DE LA JOURNÉE.** Le helper `recordManualReminderViaApi` poste `sentAt = <jour>T12:00:00` construit depuis `new Date().toISOString()`, donc en **UTC** ; et `crates/kesh-api/src/routes/dunning_reminders.rs:261` refuse tout `sent_at` futur (`if body.sent_at > Utc::now().naive_utc()` → 422). **Toute exécution avant 12:00 UTC échoue mécaniquement, toute exécution après passe.** C'est l'explication complète du symptôme qui avait résisté : « échoue en suite complète, passe en rejeu de la sélection » — le rejeu avait simplement lieu plus tard dans la journée. Tracé en issue GitHub.
- **`sidebar-navigation.spec:71`** échoue **sur `main`** et **passe sur la branche**.
- ⚠️ **L'hypothèse de la passe précédente est RÉFUTÉE.** Elle supposait que `company-contact-details.spec.ts`, insérée en position 47-51, polluait ses voisines via la DB partagée (dette `D-6-4-A`). Or les trois échouent **aussi sur `main`**, où cette spec **n'existe pas**. Elle avait été explicitement donnée pour « non testée » ; elle l'est désormais, et elle est fausse. La dette `D-6-4-A` reste réelle, mais elle n'explique pas ces trois-là.

⚠️ **CE QUE CE GATE N'ÉTABLIT PAS, et le mot est pesé : la suite E2E locale N'EST PAS VERTE.** 39 échecs subsistent des **deux** côtés. Répartition par cause, relevée sur le log de la branche : **20** de type `localStorage` inaccessible / « JWT introuvable post-login », **5** dus à l'absence de vars SMTP dans ce montage (les specs le disent elles-mêmes : « backend démarré sans SMTP factice ? »), **2** la KF d'heure ci-dessus, **12** timeouts et assertions vraisemblablement en cascade. Ce sont des défauts d'environnement et de dette préexistante, **identiques sur `main`** — mais le story file ne doit affirmer que ce qui a tourné : ce gate établit **l'absence de régression**, pas une suite verte. Le gate du 2026-08-09 employait déjà cette prudence (« E2E jouée, 0 régression »).

**Piège d'environnement rencontré, et écarté en cinq secondes** : la DB dev `kesh` — celle qu'utilisent les tests lib `kesh-db` — était **en retard de la 59ᵉ migration**, celle de cette branche. Contrôlé au `cargo sqlx migrate info` **avant** d'engager l'heure de tests, appliqué, puis gate lancé. C'est le troisième relevé de ce même symptôme sur cette story (après `kesh_gate2` et `kesh_e2e`) ; le contrôle préalable reste le meilleur geste du dépôt.

**2026-08-09 — GATE DE CLÔTURE DE LA PASSE 6 — backend et frontend VERTS, E2E exécutée pour la première fois, aucune régression imputable à la branche.** Le gate de la passe 6 était déclaré « ⚠️ INCOMPLET, ET LE MOT EST PESÉ » ; il est désormais complet, et chaque verdict ci-dessous est **lu dans son log**, jamais dans un code de sortie d'enveloppe — le piège `script ; echo "RC=$?"`, qui rend le code du `echo`, s'est présenté **deux fois** dans cette session et aurait annoncé « exit 0 » sur un run E2E à `rc=1`.

| Gate | Verdict | Mesure |
|---|---|---|
| Backend complet | ✅ vert | `fmt`, `build`, `clippy` à `rc=0` ; `Summary [3682.694s] 2131 tests run: 2131 passed, 4 skipped` — 61 min, DB `kesh_gate2` |
| Frontend | ✅ vert | `check`, `lint-i18n-ownership`, `test:unit`, `build` à `rc=0` ; **512/512** sur 63 fichiers |
| E2E — run 1, suite complète (49 specs) | `rc=1` | **177 passed / 40 failed / 19 skipped**, 13,7 min |
| E2E — run 2, montage SMTP | ✅ vert | `invoice-send-email` 4/4, `reminders` 10/10, `dunning-roundtrip` 1/1 |
| Baseline `main`, 15 fichiers | — | **53 passed / 30 failed / 5 skipped** |
| Mêmes 15 fichiers, **branche** | — | **53 passed / 30 failed / 5 skipped** — identique *fichier par fichier* |

**`company-contact-details.spec.ts` passe 5/5 à son tout premier passage.** La passe 6 l'avait écrite sans jamais l'exécuter, et le notait : « un test jamais lancé ne rougit pas, il se tait ». Elle a parlé.

**Les 40 échecs du run 1 se décomposent, et aucun n'est imputable au code de la branche.**

- **7 étaient de mon montage, pas du dépôt.** J'avais démarré un backend **sans SMTP** là où `docs/testing.md` prescrit **deux runs séquentiels** avec des configurations opposées ; les specs échouaient sur `GET /_test/sent-emails` en le disant explicitement. Relancées avec le `MockMailer` (`smtpConfigured:true`), les trois specs passent intégralement. **C'est une faute de lecture de la doc du dépôt, exactement celle que le `CLAUDE.md` impute à la story 16-4** — chercher la recette existante *avant* de diagnostiquer.
- **30 préexistent à la branche**, et c'est **mesuré, non supposé** : la même sélection rejouée sur `main` avec un montage identique rend le même total et la même répartition par fichier. La mention « baseline rouge » du Dev Agent Record n'était jusqu'ici adossée à aucun chiffre.
- **3 restent non attribués, et le mot est choisi** — `invoices.spec:387`, `invoices.spec:411`, `sidebar-navigation.spec:71`. Ils échouent dans la **suite complète** sur la branche, et **passent** aussi bien sur `main` que **sur la branche** dès qu'on rejoue la même sélection de 15 fichiers. Le code de la branche est donc hors de cause — c'est le **voisinage d'exécution** qui les fait tomber, ce que rend possible la dette documentée `D-6-4-A` (« pas de reset entre tests d'une même DB partagée »). L'hypothèse naturelle est que `company-contact-details.spec.ts`, insérée en position 47-51, pollue des specs jouées bien plus loin (114-115 et 222) — **mais elle n'est pas testée**, et je ne la donne donc pas pour établie. La départager demande de rejouer la **suite complète sur `main`**, seule comparaison réellement à armes égales.

⚠️ **Deux pannes d'environnement rencontrées en chemin, consignées parce qu'elles font prendre une absence d'exécution pour un échec de code.** (a) L'utilisateur MariaDB `kesh` avait perdu tout privilège sur `kesh_gate2` **et** le droit de créer les bases éphémères de `#[sqlx::test]` — le gate ne pouvait pas démarrer, avec un `1044 Access denied` qui ne ressemble à rien de connu. Restauré au plus étroit (`kesh_gate2`, `kesh_gate`, motif `\_sqlx\_test%`), sans toucher aux autres bases du conteneur. (b) `kesh_e2e` était **en retard d'une migration** — la 59ᵉ, celle de cette branche. Même symptôme que celui déjà consigné au Dev Agent Record pour `kesh_gate2` ; le contrôle de cinq secondes avant de lancer une heure de tests reste le meilleur geste du dépôt. Enfin, la baseline `main` a exigé une base **dédiée** (`kesh_e2e_main`) : sqlx refuse de démarrer sur une base portant une migration que le binaire ne résout pas.

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
