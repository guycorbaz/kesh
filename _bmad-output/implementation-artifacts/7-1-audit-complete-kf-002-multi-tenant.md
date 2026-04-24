---
spec: "7-1-audit-complete-kf-002-multi-tenant"
story_id: 7.1
epic: 7
story_num: 1
title: "KF-002: Audit complet multi-tenant scoping refactor"
status: "ready-for-dev"
last_updated: 2026-04-24
stepsCompleted:
  - spec-created
---

# Story 7-1: Audit Complet Multi-Tenant Scoping Refactor (KF-002)

## 📋 Vue d'ensemble

**Objectif:** Auditer complètement l'implémentation multi-tenant de Story 6-2 pour identifier et documenter tout cas où le scoping d'accès par tenant pourrait être incomplet ou défaillant. Cet audit couvre tous les endpoints API, les requêtes SQL, les migrations et la logique métier.

**Valeur métier:** Assurer que l'isolation multi-tenant est complète et robuste avant la mise en production. Identifier toute brèche de sécurité IDOR potentielle.

**Priorité:** HIGH — Bloquante avant v0.1 (production release)

---

## 👤 User Stories & Acceptance Criteria

### AC 1: Audit complet des routes API
**Étant donné** que Story 6-2 a refactorisé les routes API pour le multi-tenant,
**Je veux** un audit exhaustif qui vérifie que chaque endpoint vérifie l'accès du tenant,
**Afin que** aucune brèche IDOR n'existe.

**Critères d'acceptation:**
- Tous les endpoints GET, POST, PUT, DELETE dans `src/routes/` sont listés
- Pour chaque endpoint:
  - ✅ Vérifie que `req.tenant_id` est extrait correctement
  - ✅ Vérifie que les requêtes SQL filtrent par tenant
  - ✅ Vérifie que la réponse ne fuit aucune donnée d'un autre tenant
  - ✅ Documente le pattern utilisé (ex: "WHERE company_id = req.tenant_id")
- Générer un rapport structuré CSV/JSON listant chaque endpoint et son status de scoping

### AC 2: Audit des requêtes SQL et migrations
**Étant donné** que le schéma MariaDB a été modifié en Story 6-2,
**Je veux** vérifier que toutes les requêtes SQL et migrations maintiennent le scoping par tenant,
**Afin que** aucune requête n'expose accidentellement de données cross-tenant.

**Critères d'acceptation:**
- Scanner les fichiers `*.sql` et `src/repositories/*.rs` pour les requêtes
- Pour chaque requête SELECT/UPDATE/DELETE:
  - ✅ Vérifie la présence d'un filtre WHERE sur le tenant (company_id, user_id ou parent)
  - ✅ Identifie les requêtes sans filtre tenant explicite
  - ✅ Documente le owner tenant pour les jointures multi-table
- Lister toutes les migrations (`migrations/*.sql`) et vérifier les constraints de tenant
- Générer un rapport détaillant les risques identifiés

### AC 3: Audit de la logique métier côté backend
**Étant donné** que Rust backend implémente de la logique métier complexe,
**Je veux** un audit des patterns de vérification d'accès dans les routes,
**Afin que** les modifications futures ne contournent pas le scoping.

**Critères d'acceptation:**
- Analyser les patterns de vérification d'accès dans `src/routes/` et `src/handlers/`
- Identifier et documenter les patterns établis (ex: middleware tenant, queries avec WHERE tenant)
- Lister les cas où vérifications manuelles sont requises vs. middleware automatique
- Créer une documentation interne: "Multi-Tenant Scoping Verification Patterns"
- Identifier les opportunités d'automation (ajouter du middleware pour centraliser les vérifications)

### AC 4: Audit de la logique métier côté frontend
**Étant donné** que Svelte frontend accède aux données via API,
**Je veux** vérifier que le frontend ne stocke/n'affiche que des données du tenant courant,
**Afin que** aucune fuite de donnée n'existe dans le navigateur.

**Critères d'acceptation:**
- Analyser comment les données sont fetchées et stockées (stores Svelte, localStorage, state)
- Vérifier que chaque requête fetch() inclut le tenant correct
- Vérifier que les réponses API sont filtrées par tenant avant affichage
- Générer un rapport: "Frontend Tenant Isolation Audit"

### AC 5: Documentation de l'audit et recommendations
**Étant donné** que l'audit identifiera des gaps potentiels,
**Je veux** un rapport exhaustif avec recommendations pour corrections,
**Afin que** les future stories puissent remédier aux problèmes.

**Critères d'acceptation:**
- Créer un rapport `KF-002-AUDIT-REPORT.md` structuré:
  - Résumé exécutif (findings hauts niveaux)
  - Endpoints à risque (si identifiés)
  - Requêtes SQL à risque (si identifiées)
  - Patterns recommandés (best practices)
  - Recommendations pour refactoring
- Classer les findings par sévérité (CRITICAL, HIGH, MEDIUM, LOW)
- Générer ou mettre à jour une issue GitHub pour chaque finding CRITICAL/HIGH
- Mettre à jour le statut de KF-002 dans le suivi des issues (closes ou crée subtasks)

---

## 🔧 Contexte Technique

### Story Précédente: Story 6-2 (Multi-Tenant Scoping Refactor)
Story 6-2 a implémenté:
- Extraction de `tenant_id` (company_id) via middleware Axum
- Refactoring des routes pour utiliser `req.tenant_id`
- Refactoring des requêtes SQL pour filtrer par tenant
- Ajout de `company_id` comme foreign key aux tables pertinentes
- Migrations pour backfill et ajouter constraints

**Patterns établis par Story 6-2:**
1. **Backend routes:** Pattern `{company_id}/resource-id` pour paths
2. **Queries SQL:** `WHERE company_id = ? AND ...` systématiquement
3. **Errors:** `AppError::Forbidden` si accès refusé
4. **Schema:** FK constraints pour maintenir l'intégrité

### Issues Connues
- **KF-002:** (This story) Audit complet du scoping multi-tenant
- **KF-003 à KF-010:** Autres dettes techniques (TVA config, indexes, etc.)

### Architecture Multi-Tenant
- **Tenant ID:** Mappé à `company_id` dans la base de données
- **Extraction:** Via middleware Axum `req.tenant_id` (provient de JWT token ou session)
- **Isolation:** Niveau requête SQL (WHERE filters)
- **Fallback:** `AppError::Forbidden` sur accès cross-tenant

---

## 📁 Fichiers Impactés

### À Auditer (Pas de modifications):
- `src/routes/` — Tous les endpoints API
- `src/handlers/` — Logique métier backend
- `src/repositories/` — Requêtes SQL
- `migrations/` — Migrations de schéma
- `frontend/src/routes/` — Pages Svelte
- `frontend/src/lib/stores/` — État frontend

### À Créer/Modifier:
- `KF-002-AUDIT-REPORT.md` — Rapport d'audit principal
- `docs/MULTI-TENANT-SCOPING-PATTERNS.md` — Documentation des patterns
- `scripts/audit-multi-tenant-scoping.js` ou `.rs` — Script d'audit automatisé (optionnel)
- GitHub issues — Subtasks pour corrections (si findings CRITICAL/HIGH)

---

## ✅ Checklist de Validation

- [ ] Tous les endpoints GET/POST/PUT/DELETE listés et vérifiés
- [ ] Rapport CSV/JSON des endpoints avec status de scoping
- [ ] Toutes les requêtes SQL auditées pour filtres tenant
- [ ] Rapport détaillé des requêtes à risque
- [ ] Documentation patterns multi-tenant créée
- [ ] Audit frontend documenté
- [ ] KF-002-AUDIT-REPORT.md généré et reviewé
- [ ] GitHub issues créées pour findings CRITICAL/HIGH
- [ ] Status KF-002 mis à jour (closes ou subtasks créées)

---

## 🚀 Étapes Implémentation

### T1: Audit des Routes API
1. Lister tous les fichiers dans `src/routes/`
2. Pour chaque route, extraire:
   - Nom de l'endpoint (ex: GET /companies/{company_id}/invoices)
   - Vérification tenant (regex/pattern match)
   - Status scoping (✅ PASS, ⚠️ MANUAL CHECK, ❌ FAIL)
3. Générer rapport CSV: endpoints.csv

### T2: Audit des Requêtes SQL
1. Scanner `src/repositories/*.rs` et migrations
2. Pour chaque SELECT/UPDATE/DELETE:
   - Extraire la requête SQL
   - Vérifier: WHERE clause + tenant filter
   - Documenter: owner tenant, jointures
3. Générer rapport: sql-audit.md

### T3: Audit Logique Métier Backend
1. Analyser handlers pour patterns de vérification d'accès
2. Documenter patterns établis (middleware vs. manual)
3. Créer doc: MULTI-TENANT-SCOPING-PATTERNS.md
4. Identifier opportunités d'automation

### T4: Audit Frontend & Stores
1. Analyser Svelte pages pour fetch() et data handling
2. Vérifier que réponses API sont filtrées par tenant
3. Documenter patterns frontend
4. Générer: FRONTEND-TENANT-AUDIT.md

### T5: Génération Rapport Final & Remédiation
1. Compiler tous les audits en KF-002-AUDIT-REPORT.md
2. Classer findings par sévérité
3. Créer GitHub issues pour CRITICAL/HIGH
4. Fermer KF-002 ou créer subtasks
5. Code review du rapport

---

## 📝 Notes Dev

- **Timezone:** Toutes les audits sont point-in-time (2026-04-24)
- **Scope:** Audit COMPLET — pas de raccourcis. L'objectif est la confiance zéro pour v0.1
- **Automation:** Écrire des scripts d'audit réutilisables pour futures audits
- **Documentation:** Prioriser la clarté pour futurs devs — ce rapport devient la source de vérité multi-tenant

---

**Status:** Ready for dev — Story created 2026-04-24
