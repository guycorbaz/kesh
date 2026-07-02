# Epic 19 — Comptabilité analytique par projet

> **Statut** : design (à valider par Guy avant kickoff). Rédigé 2026-07-01.
> **Type** : feature transversale (touche > 5 modules) → découpée en story-zéro + rollout (règle de splitting préventif, CLAUDE.md).

## 1. Objectif métier (mots de Guy)

Deux usages concrets, un même besoin : **rattacher chaque opération comptable à un projet** pour l'analyser isolément.

1. **Rénovations de bâtiments → déductions fiscales.** « Je peux déduire des impôts les frais de rénovation ; si je peux clairement identifier **toutes les dépenses d'un projet**, ça simplifie ma tâche et j'évite d'oublier des déductions. » → besoin : **capter toutes les dépenses d'un projet, sans en oublier**, avec une liste + un total exportables pour la déclaration.
2. **Investissements à rendement régulier → analyse de rendement.** « Certains projets d'investissement ont des rendements réguliers ; je veux analyser le rendement de ces projets, d'où le besoin du **coût total du projet même s'il est découpé en sous-projets**. » → besoin : **coût total investi** (avec agrégation des sous-projets) + **revenus** → **rendement**.

## 2. Décisions de design (validées Guy 2026-07-01)

| # | Décision | Choix retenu | Implication |
|---|----------|--------------|-------------|
| **D1 — Granularité** | Où affecter le projet ? | **Hybride** : au **document** (facture, facture fournisseur = tout le doc sur un projet) + à la **ligne** pour les écritures manuelles. | Le `project_id` document se **propage** aux lignes d'écriture générées. Les écritures manuelles portent le projet ligne par ligne. |
| **D2 — Hiérarchie** | Sous-projets ? | **Deux niveaux** : projet → sous-projets. | `parent_id` auto-référent, contraint à 1 seul niveau (un sous-projet ne peut pas avoir de sous-projet). Le coût/rendement d'un projet **agrège ses sous-projets** (rollup). |
| **D3 — Périmètre comptable** | Quels comptes taguables ? | **Tous** : charges + produits + **bilan (actifs)**. | Indispensable au rendement : le coût d'un investissement est souvent porté à l'**actif** (immobilisation), pas en charge. « Coût investi » = lignes de comptes d'actif/charge taguées. |
| **D4 — Horizon** | Période d'analyse ? | **Les deux** : par exercice **ET** cumulé depuis l'origine (multi-années). | Les rapports acceptent un filtre exercice **ou** « depuis le début du projet ». Le cumulé traverse les clôtures d'exercice. |

**Principe d'architecture** : puisque **tout** finit en écriture comptable (partie double), la **source de vérité analytique est `project_id` sur `journal_entry_lines`**. Les documents (factures, factures fournisseurs) ne font que **stamper** ce `project_id` sur les lignes qu'ils génèrent. Les rapports lisent uniquement les lignes d'écriture → cohérence garantie, une seule source.

## 3. Modèle de données

### 3.1 Table `projects` (nouvelle)
```
projects
  id            BIGINT PK
  company_id    BIGINT NOT NULL FK companies      -- multi-tenant scoping
  parent_id     BIGINT NULL FK projects           -- NULL = projet racine ; sinon sous-projet
  code          VARCHAR(32) NOT NULL              -- court, ex. "RENOV-CHALET"
  name          VARCHAR(150) NOT NULL
  description   TEXT NULL
  status        ENUM('active','archived') NOT NULL DEFAULT 'active'
  start_date    DATE NULL
  end_date      DATE NULL
  created_at / updated_at
  UNIQUE (company_id, code)
  INDEX (company_id, parent_id)
```
- **Contrainte 2 niveaux (D2)** : à la création/édition, si `parent_id` est fourni, le parent doit être racine (`parent.parent_id IS NULL`). Vérifié en code (repo) + message d'erreur clair. Un projet racine avec des sous-projets ne peut pas devenir lui-même sous-projet.
- **Archivage** (pas de suppression si des écritures y sont rattachées) : `status='archived'` masque le projet des sélecteurs mais conserve l'historique et les rapports.

### 3.2 Dimension sur les lignes d'écriture
```
ALTER TABLE journal_entry_lines ADD COLUMN project_id BIGINT NULL FK projects;  -- non-breaking (ADD COLUMN nullable)
INDEX (project_id)
```
- **Nullable** = optionnel (D1) : aucune écriture existante n'est impactée, aucune saisie n'est forcée. Migration **non-breaking** (pas de bump `kesh_version_min_required`, cf. Migration breaking policy).

### 3.3 `project_id` document-level (propagation)
```
ALTER TABLE supplier_invoices ADD COLUMN project_id BIGINT NULL FK projects;
ALTER TABLE invoices          ADD COLUMN project_id BIGINT NULL FK projects;
```
- À la validation/comptabilisation du document, le `project_id` est **recopié sur toutes les lignes** de l'écriture générée (charge, TVA, contrepartie…). Les rapports filtrant par **type de compte**, les lignes de contrepartie (fournisseur, banque, TVA) taguées sont sans effet sur « dépenses » / « produits » / « coût investi ».

## 4. Modèle de reporting (le vrai livrable)

Deux rapports, tous deux avec **rollup sous-projets** (D2) et **filtre exercice OU cumulé-depuis-origine** (D4), exportables **PDF + CSV** :

1. **Dépenses par projet** (usage rénovation/fiscal) : pour un projet (et ses sous-projets), liste des lignes de **comptes de charge** (+ éventuellement TVA non récupérable) taguées, groupées par sous-projet et par compte, avec **total**. Drill-down jusqu'à l'écriture. → « toutes les dépenses du projet, sans en oublier ».
2. **Rendement par projet** (usage investissement) : pour un projet (+ sous-projets) — **Coût investi** (Σ lignes de comptes d'actif + charge taguées), **Revenus** (Σ lignes de comptes de produit taguées), **Résultat net** (produits − charges), et **Rendement %** (revenus / coût investi). Vue par exercice et cumulée.

*(Compte de résultat analytique complet — bilan par projet — possible en extension, mais v1 se concentre sur ces deux rapports directement liés aux besoins exprimés.)*

## 5. Découpage en stories (story-zéro + rollout)

Ordre de dépendance strict. Chaque story est livrable et testable en isolation (sauf 19-1 qui pose le socle).

| Story | Titre | Contenu | Dépend de |
|-------|-------|---------|-----------|
| **19-1** | **Socle : entité Projet + dimension** *(story-zéro)* | Migration `projects` + `project_id` sur `journal_entry_lines` (non-breaking) ; entité + repo CRUD (hiérarchie 2 niveaux, archivage) ; API `/api/v1/projects` (CRUD, Comptable+) ; page **Administration → Projets** (liste/création/édition/archivage, arbre 2 niveaux) ; intégration export/backup (table + colonne dans `.keshbackup` + `TABLES_TO_TRUNCATE` + manifeste). | — |
| **19-2** | **Tagging des écritures manuelles** | Sélecteur de projet **par ligne** dans le formulaire d'écriture ; backend accepte `project_id` par ligne + validation (projet de la company, actif) ; affichage du projet dans le détail/journal. | 19-1 |
| **19-3** | **Tagging des factures fournisseurs** *(rénovations)* | `project_id` document sur le formulaire de facture fournisseur (+ import répertoire + scan QR héritent le champ) → propagation aux lignes d'écriture d'achat à la comptabilisation. | 19-1 |
| **19-4** | **Tagging des factures de vente** *(revenus d'investissement)* | `project_id` document sur la facture client → propagation aux lignes à la validation. | 19-1 |
| **19-5** | **Tagging depuis la banque / réconciliation** | Affectation d'un projet lors de la création d'écriture depuis une transaction bancaire ; option « projet par défaut » sur une règle d'affectation. | 19-1 (idéalement après 19-2) |
| **19-6** | **Rapports analytiques** *(le payoff)* | Rapport **Dépenses par projet** + rapport **Rendement par projet**, avec rollup sous-projets + filtre exercice/cumulé + drill-down + export PDF/CSV. Menu **Mensuel → Rapports** (ou section dédiée). | 19-1..19-5 (données à agréger) |
| **19-7** | **Doc & clôture** | CHANGELOG, README (fonctionnalité + roadmap), manuels admin/user (nouveau chapitre analytique), site web. Rétro epic. | 19-1..19-6 |

> **Pourquoi ce découpage** : 19-1 pose *le pattern* (dimension sur la ligne + propagation document) sur le socle ; 19-2..19-5 sont des **rollouts mécaniques** du même pattern sur chaque flux de saisie (revue file-by-file plutôt qu'en passes adversariales lourdes) ; 19-6 est la valeur métier. On peut livrer **19-1 → 19-3 → 19-6** en premier (chemin minimal « rénovations déductibles ») puis 19-4/19-5 pour compléter le rendement, si tu veux un résultat utile plus vite.

## 6. Périmètre v1 / non-goals

- **v1** : dimension unique « projet » (mono-axe), optionnelle, 2 niveaux, sur tous les flux, + 2 rapports (dépenses, rendement).
- **Hors v1** (extensions possibles) : multi-axes (projet + centre de coût), budgets par projet, refacturation inter-projets, répartition automatique (clés de ventilation), compte de résultat/bilan analytique complet, amortissements par projet.

## 7. Points à confirmer avant kickoff

1. **Nommage** : « Projet » convient-il (vs « Affaire », « Chantier », « Centre de coût ») ? Le terme apparaîtra dans l'UI et les rapports.
2. **TVA récupérable dans les dépenses** : pour les rénovations déductibles, on compte le **HT** (TVA récupérable exclue) ou le **TTC** ? *(Proposition : afficher le HT comme coût, la TVA récupérable étant neutre — mais pour du privé non assujetti, le TTC est la vraie dépense. À trancher selon ton cas.)*
3. **Chemin de livraison** : tout l'epic d'un coup, ou le chemin minimal **19-1 → 19-3 → 19-6** (rénovations) d'abord ?
