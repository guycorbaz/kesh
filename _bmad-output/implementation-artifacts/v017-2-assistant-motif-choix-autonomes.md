# Release 0.17.0 — registre des choix autonomes de la Story v017-2

Registre propre à la story (`docs/agents-orchestration.md` § 1), à verser **par union** au registre de la release au
merge. Fiche : `v017-2-assistant-motif.md`. Numérotation `C-v017-2-k`. Issue : #809 (P3), en `refs` seulement ; #644 et
#710 citées, non traitées. Les numéros de ligne sont ceux de `c5101b07` (tête de la candidate 0.16.1-rc1). Chaque choix :
contexte, option retenue, alternatives écartées, réversibilité.

---

### C-v017-2-1 — Périmètre : l'assistant entier, inventaire clos de ses `catch` ; pas #644

- **Contexte** : #809 nomme `submitCoordinates` et ajoute « le même motif se répète aux autres étapes ». #644 vise une
  autre forme (des `catch` qui n'affichent **que** les `ApiError`, rien pour le reste), répartie sur une vingtaine
  d'écrans de l'application. L'inventaire des `catch` de l'assistant
  (`grep -rn "catch" frontend/src/routes/onboarding frontend/src/lib/features/onboarding`, tests exclus) en rend
  **douze** : sept `catch {` nus (`+page.svelte:39, 48, 57, 65, 73, 81, 126`), deux qui affichent déjà le motif
  (`:189, :208`), deux qui passent par `reportFinalizeError` (`:200, :219`), un silence dans le chargement
  (`+layout.ts:33`) — plus un rejet relancé, qui n'affiche rien par construction (`onboarding.svelte.ts:163`).
- **Retenu** : traiter **les onze sites d'affichage de `+page.svelte`** (les sept nus, les deux du compte bancaire
  harmonisés, les deux de la finalisation via `reportFinalizeError`) ; déclarer `onboarding.svelte.ts:163` (relance,
  conforme) et `+layout.ts:33` (silence, hors périmètre — C-v017-2-7). L'assistant est un seul écran, le défaut y est
  le même partout, et c'est l'écran d'une installation neuve, celui que Guy rejouera pour la production.
- **Écartées** : (a) le seul `submitCoordinates` — laisse six étapes muettes pour la même cause, contre la lettre de
  #809 ; (b) #644 entier — vingt sites d'autres écrans, décision site par site non mécanique (cf. le corps de #644),
  hors du budget d'une story de correction.
- **Réversible** : oui.

### C-v017-2-2 — La règle d'affichage : `errorMessageOf` reçoit un repli optionnel

- **Contexte** : `errorMessageOf` (`frontend/src/lib/shared/utils/error-message.ts:43`) rend le message d'une `ApiError`
  non blanche, sinon « Erreur inattendue. », et journalise toute valeur qui n'est pas une `ApiError`. Les deux sites du
  compte bancaire (`+page.svelte:190-194`, `:209-213`) écrivent à la main une variante qui garde un message **propre à
  l'étape** à la place du générique, sans écarter un message blanc ni journaliser.
- **Retenu** : `errorMessageOf(err: unknown, repli?: string): string` — si `repli` est fourni, il remplace le générique
  dans les règles 2 (message vide ou blanc) et 3 (valeur qui n'est pas une `ApiError`, toujours journalisée) ; sans
  `repli`, comportement inchangé pour les 23 fichiers qui l'appellent (`grep -rln "errorMessageOf(" frontend/src`, hors le module et les `.test.ts`). Les onze sites de l'assistant l'appellent avec
  le message de leur étape. Le message d'une `ApiError` est affiché **quel que soit son statut** (400 de validation,
  403, 409, réseau `status: 0`, 500 « Erreur interne ») : règle de la 28-2, déjà celle du compte bancaire.
- **Écartées** : (a) `errorMessageOf(err)` nu — perd le contexte de l'étape pour une panne imprévue ; (b) un utilitaire
  local à la page — une troisième écriture de la même règle (DRY) ; (c) n'afficher le motif que pour `status < 500` —
  règle nouvelle que rien ne demande, et un 500 rend déjà un message traduit.
- **Réversible** : oui (paramètre optionnel).

### C-v017-2-3 — `reportFinalizeError` : la branche `'VALIDATION'` est morte ; intégrée

- **Contexte** : `+page.svelte:153` compare `err.code === 'VALIDATION'`, alors que le serveur rend `VALIDATION_ERROR`
  pour toute `AppError::Validation` (`crates/kesh-api/src/errors.rs:1538-1540` :
  `build_response(StatusCode::BAD_REQUEST, "VALIDATION_ERROR", &msg)`). Aucun autre site du dépôt ne produit le code
  `VALIDATION` (`grep -rn "'VALIDATION'"` : ce seul site). Le refus de la finalisation « comptes 1100/3000 manquants »
  (`routes/onboarding.rs:871-876`) tombe donc sur « Finalisation échouée — réessayez. » : **le même défaut que #809**,
  dans l'assistant.
- **Retenu** : l'intégrer (mécanique, même classe, même fichier) : `ONBOARDING_STEP_ALREADY_COMPLETED` garde son
  message propre ; tout le reste passe par `errorMessageOf(err, <message de finalisation>)`.
- **Écartée** : corriger le seul littéral en `'VALIDATION_ERROR'` — garde une seconde écriture de la règle.
- **Réversible** : oui.

### C-v017-2-4 — Le champ Pays : une indication, sans sélecteur, sans `maxlength`, sans contrôle client

- **Contexte** : #809 demande « un sélecteur, ou une aide qui dit code à deux lettres ». Le champ est pré-rempli `CH`
  (`+page.svelte:94`) avec un `placeholder="CH"` (`:358`) et aucun libellé d'aide. Le serveur ne vérifie que la **forme**
  (deux lettres ASCII, `crates/kesh-api/src/address_input.rs:66-71`) ; `kesh-qrbill` vérifie, lui, la liste ISO
  (`crates/kesh-qrbill/src/validation.rs:258-271`, `is_iso_3166_alpha2` privée, `:276`).
- **Retenu** : une indication au libellé, sur le modèle de l'IDE (`:363-366`) : clé `onboarding-field-country-hint`
  aux quatre locales (fr « code ISO à deux lettres, ex. CH, FR, DE »), et **pas** de `maxlength`. Avec le motif du
  serveur désormais affiché, une saisie en toutes lettres est refusée **et expliquée**.
- **Écartées** :
  - `maxlength={2}` (le choix des contacts, `contacts/+page.svelte:995`, et de `SupplierCreatePanel.svelte:210`) :
    « Suisse » devient « Su », que le serveur **accepte** (deux lettres) — un refus bruyant échangé contre une donnée
    fausse muette. C'est précisément le défaut voisin de C-v017-2-8.
  - un sélecteur (`<select>` des 249 codes, noms par `Intl.DisplayNames`) : composant neuf, liste à tenir, et il
    divergerait des contacts et des fournisseurs ; sa vraie place est avec la validation ISO côté serveur
    (C-v017-2-8), pour les trois écrans à la fois.
  - un contrôle côté client (`/^[A-Za-z]{2}$/`) : seconde écriture d'une règle du serveur, qui n'attrape pas plus que
    lui.
- **Réversible** : oui.
- *Amendé le 2026-10-30, validation P1 (F4) : l'indication se place **sous** le champ (`<p id="coord-country-hint">`,
  `aria-describedby`), non dans le libellé — la cellule du pays occupe un tiers de la grille, et le texte allemand y
  ferait passer le libellé sur plusieurs lignes. Capture de-CH ajoutée.*

### C-v017-2-5 — Le motif s'affiche en français pour un utilisateur DE/IT/EN ; renvoyé à #710

- **Contexte** : les refus de l'assistant sont des `AppError::Validation("…")` écrits en français dans les routes
  (`address_input.rs:61, 68, 90` ; `routes/onboarding.rs:163, 202, 267, 378, 423, 527, 538, 594, 601, 607, 873`) et rendus
  tels quels. #710 (P4) suit la classe entière.
- **Retenu** : afficher le motif tel quel ; un motif en français vaut mieux qu'un message traduit qui ne dit rien.
  Aucune traduction de refus serveur dans cette story.
- **Écartée** : traduire les refus de l'assistant ici — un morceau de #710, sans sa garde.
- **Réversible** : oui.

### C-v017-2-6 — `ONBOARDING_STEP_ALREADY_COMPLETED` aux autres étapes : le message du serveur, sans rechargement

- **Contexte** : une étape rejouée (deux onglets, état périmé) reçoit 400 `ONBOARDING_STEP_ALREADY_COMPLETED`, message
  traduit « Cette étape de configuration a déjà été complétée » (`errors.rs:1723-1730`). Seule la finalisation a un
  message propre (« … Veuillez recharger la page. »).
- **Retenu** : la règle générale s'applique (le message du serveur s'affiche) ; pas de rechargement automatique de
  l'état.
- **Écartée** : recharger l'état (`fetchState`) sur ce code — comportement nouveau, hors de #809.
- **Réversible** : oui.

### C-v017-2-7 — Hors périmètre : le silence du chargement (`+layout.ts:33`) et `AdminRestorePanel`

- **Contexte** : `+layout.ts:33-38` journalise l'échec de `fetchState` et laisse la page s'afficher ; l'état restant
  `stepCompleted: 0` (valeur initiale, `onboarding.svelte.ts:15-20`), l'assistant montre le choix de la langue quelle
  que soit l'étape réelle. `AdminRestorePanel.svelte:84` (rendu par `/onboarding/restore`) est nommé par #644.
- **Retenu** : ni l'un ni l'autre ; le premier demande un état d'erreur et une reprise (non mécanique) → **issue à
  ouvrir** (P3) ; le second reste à #644 (composant partagé avec `/admin/restore`).
- **Réversible** : oui.
- *Annoté le 2026-10-30, validation P1 (R7) : la ligne `AdminRestorePanel.svelte:84` est celle que cite #644 ; à
  `c5101b07`, les `catch` du panneau sont `:157` (affiche `importErrorMessage(err)`) et `:181` (silence voulu). Issue du
  chargement ouverte : **#818** (P3). Défaut voisin relevé en P1 (R3), même traitement : la finalisation refusée laisse
  un écran vide (étape 7 sans branche, `+layout.ts:30`) → **#821** (P3), limite déclarée en AC-4.*

### C-v017-2-8 — Défaut voisin : le pays n'est contrôlé qu'à la forme ; issue à ouvrir, non intégré

- **Contexte** : `address_input.rs:66-71` accepte tout couple de lettres (`SU`, `XX`) ; `kesh-qrbill` le refusera à la
  génération d'une QR-facture (`validation.rs:266-270`). Les champs des contacts et du fournisseur tronquent à deux
  caractères (`maxlength`), si bien que « Suisse » y devient `SU`, accepté en silence. Aucun écran ne modifie l'adresse
  de la société après l'assistant (`frontend/src/lib/features/settings/settings.api.ts` : courriel, coordonnées de
  contact, verrou des livres).
- **Retenu** : **issue à ouvrir** (bug, P2 proposé : une QR-facture refusée plus tard pour une donnée acceptée à la
  saisie) — valider contre la liste ISO au serveur (rendre `is_iso_3166_alpha2` publique ou la déplacer), retirer la
  troncature muette des deux formulaires, envisager un sélecteur commun. Non intégré : règle métier partagée par trois
  écrans et deux crates, et une adresse existante invalide deviendrait inenregistrable (à examiner avec les données à
  garder de la 0.16.1-rc1).
- **Réversible** : —
- *Annoté le 2026-10-30, validation P1 : issue ouverte, **#817** (P2).*

### C-v017-2-9 — Garde de non-régression : lexicale et locale à l'assistant

- **Retenu** : un test Vitest qui lit `frontend/src/routes/onboarding/+page.svelte` (commentaires masqués) et échoue
  sur tout `catch {` (sans liaison) et sur tout `catch (x)` dont le bloc n'appelle ni `errorMessageOf(` ni
  `reportFinalizeError(` ; plancher : onze `catch` recueillis. Les tests de comportement par étape restent la preuve ;
  la garde empêche qu'une étape ajoutée reprenne la forme nue.
- **Écartée** : une règle globale dans `erreurs-en-texte.test.ts` — c'est le « règle décidable propre » qu'attend #644.
- **Réversible** : oui.

### C-v017-2-10 — E2E : une spec dédiée, pas la réparation d'`onboarding-path-b`

- **Contexte** : `onboarding-path-b.spec.ts` remplit `#coord-address` (`:53, :73, :106`), champ disparu avec #213 ; ses
  tests figurent aux échecs attendus (KF-029, #97 ; `docs/testing.md:396-397`).
- **Retenu** : une spec neuve `frontend/tests/e2e/onboarding-motif.spec.ts`, preset `fresh`, qui parcourt l'assistant
  jusqu'aux coordonnées par l'interface, saisit « Suisse », voit le motif, corrige en `CH` et atteint l'étape bancaire.
  KF-029 reste ouverte, non touchée.
- **Écartée** : réparer `onboarding-path-b` — #97 a d'autres causes (bannière, redirections).
- **Réversible** : oui.
- *Amendé le 2026-10-30, validation P1 (F1, consigne de l'orchestrateur) : l'état est amené à l'étape 5 **par l'API**
  (`authedApiContext`), seul l'écran des coordonnées se joue par l'interface, par sélecteurs d'`id` — la garde
  `e2e-selecteurs-traduits.test.ts` refuserait des sélecteurs de texte dans un fichier neuf, et trois boutons de
  l'assistant n'ont pas de `data-testid`. Aucun `data-testid` ajouté.*

### C-v017-2-11 — Le contrat du refus se fixe aussi au serveur

- **Retenu** : un test d'intégration `coordinates_rejects_country_name` dans `crates/kesh-api/tests/onboarding_path_b_e2e.rs`
  (pays « Suisse » → 400 `VALIDATION_ERROR`, message contenant « code ISO à 2 lettres », étape restée à 5). Aucun code
  serveur ne change ; le test fixe ce que l'écran affiche.
- **Réversible** : oui.

### C-v017-2-12 — Aucune migration, aucune écriture sur les données existantes

- **Contexte** : contrainte du Project Lead du 2026-10-30 (`kesh-gate-logs/prompts/v017-donnees-a-garder.md`).
- **Retenu** : la story ne touche que le frontend, un test serveur, les locales et le manuel. Sur un refus, le serveur
  n'écrit rien : la validation (`routes/onboarding.rs:526-541`) précède l'ouverture de la transaction (`:545`). Aucune
  table, aucune migration, aucun bump.
- *Amendé le 2026-10-30, validation P1 (R1) : « le serveur n'écrit rien » vaut pour toutes les étapes **sauf la
  démonstration** — `kesh-seed/src/lib.rs:162-172` : les quatre premières validations de `seed-demo` commitent avant la
  dernière transaction, et le rejeu rend 500 (#538). Défaut préexistant ; la story ne change aucune écriture.*

### C-v017-2-13 — Le littéral de S1 devient une clé (validation P1, F10)

- **Contexte** : la fiche gardait `'Erreur lors du choix de langue'` en dur, « faute de catalogue avant le choix de la
  langue ». Or `msg(clé, repli)` retombe sur son repli quand le catalogue manque : l'argument ne tenait pas, et c'était
  le dernier message en dur de l'écran.
- **Retenu** : clé `onboarding-language-error` aux quatre locales, repli égal à la valeur fr-CH ; mutation M11 (le test
  exige l'appel d'`i18nMsg` avec la clé).
- **Écartée** : garder le littéral en corrigeant seulement la justification.
- **Réversible** : oui.
