# Story 1.12 : Page de gestion des utilisateurs

Status: done

## Story

As a **administrateur**,
I want **gérer les utilisateurs depuis l'interface web**,
so that **je puisse créer, modifier, désactiver des comptes et réinitialiser des mots de passe sans passer par l'API directement**.

### Contexte

L'Epic 1 a livré le CRUD utilisateurs côté API (Story 1.7) et le RBAC middleware (Story 1.8), mais aucune interface frontend n'a été créée pour y accéder. Cette story comble ce gap pour que l'objectif de l'epic — *l'administrateur peut gérer les utilisateurs* — soit atteint du point de vue utilisateur.

### Décisions de conception

- **Accès restreint aux admins** : la page `/users` n'est visible dans le sidebar que pour les utilisateurs avec le rôle `Admin`. Les rôles `Comptable` et `Consultation` ne voient pas l'entrée de menu.
- **Composants existants** : utiliser les composants shadcn-svelte déjà installés (Table, Dialog, Button, Input, Select, Tooltip, Sonner pour les toasts).
- **API client** : utiliser `apiClient` (Story 1.11) pour toutes les requêtes API — refresh token et erreurs structurées gérés automatiquement.
- **Pagination** : la liste utilise la pagination API existante (`?limit=50&offset=0`). Pour le MVP, pagination simple (précédent/suivant).
- **Verrouillage optimiste** : les opérations de modification envoient le champ `version` et gèrent l'erreur 409 avec un message explicite invitant à recharger.
- **Pas de suppression** : conformément à la Story 1.7, seule la désactivation est possible (soft-disable, historique préservé).
- **Messages d'erreur** : affichage via Sonner (toast). Codes d'erreur connus (`CANNOT_DISABLE_SELF`, `CANNOT_DISABLE_LAST_ADMIN`, `RESOURCE_CONFLICT`, `VALIDATION_ERROR`) traduits en messages français lisibles.

## Acceptance Criteria (AC)

1. **Navigation conditionnelle** — Given rôle `Admin`, When affichage du sidebar, Then une entrée "Utilisateurs" est visible dans le groupe Paramètres. Given rôle `Comptable` ou `Consultation`, Then l'entrée n'apparaît pas.
2. **Liste des utilisateurs** — Given rôle `Admin`, When navigation vers `/users`, Then un tableau affiche les colonnes : Nom d'utilisateur, Rôle, Statut (Actif/Désactivé), Créé le. Pagination précédent/suivant si > 50 utilisateurs.
3. **Création d'utilisateur** — Given rôle `Admin`, When clic sur "Nouvel utilisateur", Then un Dialog s'ouvre avec les champs : nom d'utilisateur, mot de passe, confirmation mot de passe, rôle (Select: Admin/Comptable/Consultation). Validation côté client : username non vide, password ≥ 12 caractères, confirmation identique. On submit → `POST /api/v1/users`, toast succès, tableau rafraîchi.
4. **Modification de rôle et statut** — Given rôle `Admin`, When clic sur un utilisateur dans le tableau, Then un Dialog affiche les détails avec possibilité de modifier le rôle (Select) et le statut actif (checkbox). On submit → `PUT /api/v1/users/{id}` avec `version`. Toast succès ou erreur 409 (conflit de version → "Les données ont changé, veuillez recharger").
5. **Désactivation** — Given rôle `Admin`, When clic sur le bouton désactiver d'un utilisateur, Then confirmation demandée (Dialog). On confirm → `PUT /api/v1/users/{id}/disable`. Gestion des erreurs : `CANNOT_DISABLE_SELF` → "Vous ne pouvez pas désactiver votre propre compte", `CANNOT_DISABLE_LAST_ADMIN` → "Impossible de désactiver le dernier administrateur".
6. **Réinitialisation de mot de passe** — Given rôle `Admin`, When clic sur "Réinitialiser le mot de passe" d'un utilisateur, Then un Dialog demande le nouveau mot de passe + confirmation. Validation : ≥ 12 caractères, confirmation identique. On submit → `PUT /api/v1/users/{id}/reset-password`. Toast succès.
7. **Auth guard** — Given rôle non-Admin, When navigation directe vers `/users`, Then redirection vers `/` (page d'accueil).
8. **Accessibilité** — And tous les Dialogs sont accessibles au clavier (Tab, Escape pour fermer), les actions destructives (désactivation) ont un focus explicite sur le bouton d'annulation, les toasts sont annoncés par les lecteurs d'écran (aria-live via Sonner).
9. **Indicateur utilisateur courant** — Given la liste des utilisateurs, Then l'utilisateur connecté est visuellement identifié (badge "Vous" ou similaire) et le bouton désactiver est absent pour sa propre ligne.

## Tasks / Subtasks

### T1 — Route et auth guard (AC: #7)
- [x] T1.1 Créer `frontend/src/routes/(app)/users/+page.svelte`
- [x] T1.2 Créer `frontend/src/routes/(app)/users/+page.ts` avec guard : si `authState.currentUser?.role !== 'Admin'`, redirect vers `/`

### T2 — Navigation conditionnelle sidebar (AC: #1)
- [x] T2.1 Modifier `frontend/src/routes/(app)/+layout.svelte` : ajouter l'entrée "Utilisateurs" (`/users`) dans le groupe Administration, conditionnée par `authState.currentUser?.role === 'Admin'`
- [x] T2.2 Vérifier que l'entrée n'apparaît pas pour les rôles Comptable et Consultation

### T3 — Tableau des utilisateurs (AC: #2, #9)
- [x] T3.1 Implémenter le chargement de la liste via `apiClient.get<UserListResponse>('/api/v1/users?limit=50&offset=0')`
- [x] T3.2 Afficher le tableau avec les composants Table (shadcn-svelte) : colonnes Username, Rôle, Statut, Créé le
- [x] T3.3 Badge "Vous" sur la ligne de l'utilisateur connecté
- [x] T3.4 Boutons d'action par ligne : Modifier, Réinitialiser mot de passe, Désactiver (absent pour soi-même)
- [x] T3.5 Pagination précédent/suivant

### T4 — Dialog création utilisateur (AC: #3)
- [x] T4.1 Dialog avec formulaire : username (Input), password (Input type=password), confirmation (Input type=password), rôle (Select)
- [x] T4.2 Validation côté client : username non vide (après trim), password ≥ 12 caractères, confirmation === password
- [x] T4.3 Appel `apiClient.post('/api/v1/users', { username, password, role })` + toast succès + refresh tableau
- [x] T4.4 Gestion erreur `RESOURCE_CONFLICT` (username déjà pris) → message explicite

### T5 — Dialog modification utilisateur (AC: #4)
- [x] T5.1 Dialog avec formulaire : rôle (Select), actif (checkbox), version (hidden)
- [x] T5.2 Appel `apiClient.put('/api/v1/users/{id}', { role, active, version })` + toast succès + refresh tableau
- [x] T5.3 Gestion erreur 409 `OPTIMISTIC_LOCK_CONFLICT` → toast "Les données ont été modifiées. Rechargez la page."

### T6 — Dialog désactivation (AC: #5)
- [x] T6.1 Dialog de confirmation avec message explicite ("Désactiver l'utilisateur X ? Ses sessions seront révoquées.")
- [x] T6.2 Focus par défaut sur le bouton Annuler (action destructive) — Dialog shadcn-svelte focus automatiquement le premier élément focusable, le bouton Annuler précède Désactiver
- [x] T6.3 Appel `apiClient.put('/api/v1/users/{id}/disable')` + toast succès + refresh tableau
- [x] T6.4 Gestion erreurs `CANNOT_DISABLE_SELF`, `CANNOT_DISABLE_LAST_ADMIN` → messages français

### T7 — Dialog réinitialisation mot de passe (AC: #6)
- [x] T7.1 Dialog avec formulaire : nouveau mot de passe (Input type=password), confirmation
- [x] T7.2 Validation côté client : ≥ 12 caractères, confirmation identique
- [x] T7.3 Appel `apiClient.put('/api/v1/users/{id}/reset-password', { newPassword })` + toast succès

### T8 — Accessibilité (AC: #8)
- [x] T8.1 Vérifier navigation clavier dans tous les Dialogs (Tab, Shift+Tab, Escape) — shadcn-svelte/Bits UI gère nativement
- [x] T8.2 aria-describedby sur les champs de formulaire avec erreurs de validation — role="alert" sur les messages d'erreur
- [x] T8.3 Toasts Sonner avec aria-live (déjà natif — vérifié, Toaster monté dans +layout.svelte racine)

### T9 — Tests (AC: #1-#9)
- [x] T9.1 Tests Vitest : auth guard redirect non-admin (3 tests : Comptable, Consultation, Admin)
- [x] T9.2 Tests Vitest : sidebar conditionnel (3 tests : rôle Admin, Comptable, Consultation)
- [x] T9.3 Tests E2E Playwright : parcours complet CRUD utilisateurs (create, list, validation)
- [x] T9.4 Tests E2E Playwright : erreurs (self-disable bouton absent, validation mot de passe)
- [x] T9.5 Test axe-core sur la page `/users`

## Dev Notes

### Endpoints API disponibles (Story 1.7)

| Méthode | URL | Body | Réponse |
|---------|-----|------|---------|
| GET | `/api/v1/users?limit=N&offset=N` | — | `{ items: [UserResponse], total, offset, limit }` |
| GET | `/api/v1/users/{id}` | — | `UserResponse` |
| POST | `/api/v1/users` | `{ username, password, role }` | `UserResponse` (201) |
| PUT | `/api/v1/users/{id}` | `{ role, active, version }` | `UserResponse` |
| PUT | `/api/v1/users/{id}/disable` | — | `UserResponse` |
| PUT | `/api/v1/users/{id}/reset-password` | `{ newPassword }` | `UserResponse` |

### Types frontend à créer

```typescript
interface UserResponse {
  id: number;
  username: string;
  role: 'Admin' | 'Comptable' | 'Consultation';
  active: boolean;
  version: number;
  createdAt: string; // ISO 8601
  updatedAt: string;
}

interface UserListResponse {
  items: UserResponse[];
  total: number;
  offset: number;
  limit: number;
}
```

### Composants shadcn-svelte disponibles

Button, Input, Select, Table, Dialog, Sonner (toasts), Tooltip, DropdownMenu, Separator

### Politique de mot de passe

Configurable via `KESH_PASSWORD_MIN_LENGTH` (défaut 12, borne [8, 128]). La validation côté client utilise le défaut 12 (pas d'endpoint pour récupérer la config serveur — acceptable MVP).

## Dev Agent Record

### Implementation Plan
- Page unique `/users` avec 4 dialogs inline (Create, Edit, Disable, Reset Password)
- Types `UserResponse`/`UserListResponse` dans `$lib/shared/types/user.ts`
- Sidebar conditionnel via `$derived(authState.currentUser?.role === 'Admin')` dans le layout
- Auth guard dans `+page.ts` : redirect 302 vers `/` si non-Admin
- Tous les appels API via `apiClient` (Story 1.11) — refresh token et erreurs structurées gérés automatiquement

### Completion Notes
- ✅ T1 : Route `/users` + auth guard redirect non-Admin
- ✅ T2 : Section "Administration" dans le sidebar, visible uniquement pour Admin
- ✅ T3 : Tableau avec Table shadcn, badge "Vous", boutons d'action contextuels, pagination
- ✅ T4 : Dialog création avec validation client (username trim, password ≥ 12, confirmation)
- ✅ T5 : Dialog modification avec Select rôle + checkbox actif + verrouillage optimiste (version)
- ✅ T6 : Dialog désactivation avec confirmation, gestion CANNOT_DISABLE_SELF/LAST_ADMIN
- ✅ T7 : Dialog reset password avec validation confirmation
- ✅ T8 : Accessibilité — role="alert" sur erreurs, aria-label sur boutons, keyboard nav native Bits UI
- ✅ T9 : 6 tests Vitest (auth guard + sidebar conditionnel) + 6 tests E2E Playwright (CRUD + erreurs + axe-core)

## File List

### New Files
- `frontend/src/routes/(app)/users/+page.svelte` — Page principale gestion utilisateurs (tableau + 4 dialogs)
- `frontend/src/routes/(app)/users/+page.ts` — Auth guard Admin-only
- `frontend/src/lib/shared/types/user.ts` — Types UserResponse, UserListResponse, Role
- `frontend/src/routes/(app)/users/users-page.test.ts` — Tests Vitest auth guard + sidebar conditionnel
- `frontend/tests/e2e/users.spec.ts` — Tests E2E Playwright CRUD utilisateurs + axe-core

### Modified Files
- `frontend/src/routes/(app)/+layout.svelte` — Ajout section "Administration" conditionnelle dans sidebar

## Change Log

| Date | Passe | Modèle | Findings | Patches |
|------|-------|--------|----------|---------|
| 2026-04-08 | Implémentation | Opus 4.6 | — | T1-T9 complètes, 43 tests Vitest passent (6 nouveaux), build OK |
| 2026-04-08 | Code review passe 1 | Sonnet 4.6 (3 agents parallèles) | 7 patch, 1 bad-spec, 2 defer, 8 rejetés | P1: guard null submitReset, P2: else branch catch (5 handlers), P3: aria-describedby inputs↔erreurs, P4: autofocus Annuler dialog désactivation, P5: $effect loadUsers, P6: guard nextPage, P7: fallback formatDate |
| 2026-04-08 | Code review passe 2 | Haiku 4.5 (2 agents parallèles) | 3 patch, 2 defer, 6 rejetés | P8: loadUsers() manquant après reset password, P9: aria-describedby sur champs password create+reset, P10: erreur non-API dans resetError au lieu de toast |
