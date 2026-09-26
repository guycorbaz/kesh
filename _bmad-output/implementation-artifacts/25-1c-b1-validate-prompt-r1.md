# Prompt — revalidation R1 de `bmad-create-story validate`, Story 25-1c-b1 (réouverte)

*Versionné le 2026-09-15. Une lentille en contexte frais (Opus), orthogonale à la passe 7 (Sonnet). Première
passe de la boucle rouverte : l'écran consomme désormais un vocabulaire traduit par la route.*

Tu es un **valideur adversarial** en contexte frais. Ton objet est la fiche
`_bmad-output/implementation-artifacts/25-1c-b1-journal-audit-ecran.md`, dépôt `/home/gcorbaz/devel/kesh`,
branche `story/25-1c-b-journal-audit-ecran`. Le contrat qu'elle consomme est
`25-1c-a-journal-audit-route.md` (même branche, rouvert lui aussi).

**Base** : `git diff 5760bab7 HEAD -- _bmad-output/implementation-artifacts/25-1c-b1-journal-audit-ecran.md`.
Nomme-la dans ton rapport. La section « Réouverture » du Change Log liste les choix de spécification **à
contester**.

⛔ **Rien ne se croit sur parole.** ⚠️ **Ne conteste PAS les arbitrages du Project Lead** (actions et types
traduits à l'écran, filtre strict, export non audité, découpage b1/b2).

## Les axes, et tu déclareras lesquels tu as exercés

1. **Le contrat** : chaque champ, route, forme et numéro d'AC que la b1 attribue à la 25-1c-a existe-t-il
   **exactement** dans la 25-1c-a rouverte (DTO de son AC 8, vocabulaire de son AC 17, refus de son AC 6,
   plafond de son AC 11) ?
2. **Les `<select>` natifs (AC 5)** : sont-ils réellement le patron de `contacts/+page.svelte` (style,
   liaison, synchronisation d'URL) ? Le composant `Select` de bits-ui est-il **vraiment** impilotable par
   valeur en Playwright, ou une forme existante le permet-elle ? La garde `e2e-selecteurs-traduits.test.ts`
   refuse-t-elle **effectivement** `getByRole('option', { name })` — lis-la. Un `<select>` natif pose-t-il un
   problème d'accessibilité ou de cohérence visuelle que la fiche tait ?
3. **`toSelectOptions` (AC 3)** : d'où le frontend tire-t-il sa locale (`i18n.svelte.ts`) ? Le tri
   `Intl.Collator(locale, { sensitivity: 'base' })` range-t-il « Écriture créée » avant « Facture validée »
   en `fr-CH` sous jsdom/Node — **exécute-le** ? L'ajout d'un code hors vocabulaire est-il cohérent avec la
   synchronisation d'URL et le bouton « Réinitialiser » ?
4. **Les gardes i18n (AC 10-11), par l'exécution** : la page prescrite laisse-t-elle vraiment
   `MOTIFS_DYNAMIQUES`, `sitesGabarit`, `CANDIDATES_ATTENDUES` inchangés ? Les clés `audit-log-*` de l'écran,
   dans une feature `audit-log`, passent-elles `lint-i18n-ownership` alors que le backend possède des clés
   au même préfixe ? Construis une **copie jetable** dans le scratchpad pour observer ce que rendent les
   extracteurs sur une page synthétique conforme.
5. **Les tests et l'E2E (AC 12-13)** : pour chaque assertion neuve, construis l'état où elle **devrait
   rougir**. La preuve de traduction du scénario 1 (« cellule non vide et différente du code ») est-elle
   discriminante — que se passe-t-il si la route renvoie un libellé égal au code pour un code inconnu ?
   `selectOption({ value })` sur un `<select>` avec `data-testid` : patron réel ?
6. **Les mutations (AC 14)** : chacune produit-elle un échec d'**assertion**, sur le test annoncé ?
7. **Résidus de l'ancienne conception** dans le corps (`entity-type-label`, `29`, `companyId`, « société
   indéterminée », `sitesGabarit` 11, clés `audit-log-entity-*` créées ici) ; renvois internes ; tâches.

## Ce que tu rends

- **Les findings**, chacun avec sévérité (`CRITICAL` / `HIGH` / `MEDIUM` / `LOW`), l'endroit exact, **la
  commande ou l'extrait qui l'établit**, et le correctif.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Ne qualifie jamais de
  « vérifié » ce que tu n'as pas exécuté.

## Interdits

⛔ **N'écris aucun fichier du dépôt et n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante** — nommément `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`,
`scripts/install-hooks.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, tout
`git commit`/`push`/`checkout`/`switch`/`add`/`stash`/`reset`/`rebase`, `npm install`, `npm run build`,
`npm run test:e2e`, `sqlx migrate`, `cargo test`/`cargo nextest`. Autorisés : lecture, `grep`,
`git show`/`git diff`, `npm run check`, `npx vitest run` en lecture, et une copie jetable **dans le
scratchpad** `/tmp/claude-1000/-home-gcorbaz-devel-kesh/cb9f9ce3-4808-472f-93d0-698c43110e0e/scratchpad/r1-b1/`.
⚠️ `grep` est ici `ugrep`, qui refuse les motifs trop complexes : utilise Python pour les extractions à
contexte.
