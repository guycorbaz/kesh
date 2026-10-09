# Prompt — revue de code P2 ciblée, Story 15-1a-ii

*Versionné le 2026-10-09. Passe ciblée (CLAUDE.md § « La passe ciblée ») : une seule lentille, contexte frais, en lecture
seule, braquée sur le seul commit de code de la remédiation P1. Confiée à Opus plutôt qu'à Haiku : la remédiation change
la forme d'une réponse d'API (`details` du 409 `ENTRY_LETTERED`) et touche deux crates.*

Worktree `/home/gcorbaz/devel/kesh-15-1a-ii`, branche `story/15-1a-ii-gardes-du-lettrage` (rebasée sur `1ae3963e`).
**Objet** : `git show 581040aa` (diff unique ; ouvre les fichiers à `HEAD` autour des hunks). Rapports remédiés :
`/home/gcorbaz/devel/kesh-gate-logs/15-1a-ii-review-p1-{B,E,A}.md` (P1 Sonnet ×3 : 1 MEDIUM, 15 LOW). Choix C-15-1a-ii-7 à 11
(`_bmad-output/implementation-artifacts/epic-15-choix-autonomes.md`). Fiche : `15-1a-ii-gardes-du-lettrage.md`.

⚠️ Tu relis du CODE : un défaut est ce que le code fait de faux, ou un test qui ne prouve pas ce qu'il dit. Ce que la
fiche prescrit à une AUTRE story (15-1a2, 15-1c) n'est pas un manque. Toute affirmation de présence ou d'absence : la
sortie d'un `grep -nF`, copiée.

## Lentille unique — chasseur de régressions de la remédiation P1

1. **`details` du 409 `ENTRY_LETTERED`** devenu `{ letteringCode }` (C-15-1a-ii-7, écart assumé à la lettre d'AC8) et la
   fonction commune `refusal_409` (`crates/kesh-api/src/errors.rs`) : les AUTRES 409 qui passent désormais par
   `refusal_409` gardent-ils exactement leur forme d'avant (`documentId`, `documentNumber`, message, clé i18n) ? Un client
   (`frontend/src`, `docs/api-external.md`, tests E2E) lit-il encore `documentNumber` sur ce code ? Les trois sites
   d'`api-external.md` et le CHANGELOG disent-ils la forme réelle ?
2. **Garde de longueur avant le `zip`** origine/miroir (`journal_entries.rs`, R6) : l'`Invariant` est-il levé AVANT toute
   écriture de la marque (pas de groupe à moitié posé) ? Le message et le code d'erreur sont-ils ceux des autres
   `Invariant` ?
3. **`lettering_guard` et `Lecture` privées, `_company_id` retiré** (C-15-1a-ii-8) : un appelant hors du module en
   dépendait-il (tests d'intégration, autre crate) ? La requête reste-t-elle correcte sans scoping par société, et le
   doc-comment dit-il pourquoi c'est sûr ?
4. **Test étendu** `reverse_in_tx_disparait_avec_le_rollback_de_l_appelant` : prouve-t-il ce qu'il dit (marque et audit
   visibles DANS la transaction, absents APRÈS le rollback) ? Les trois mutations M-A1-1..3 du journal
   (`/home/gcorbaz/devel/kesh-gate-logs/15-1a-ii-review-p1-mutations.log`) mordent-elles réellement ?
5. **Doc-comments** neufs (en-tête de `reverse_in_tx_inner`, Pattern 5 `/reverse`, « quatre cycles ») : exacts au code ?

## Ce que tu rends

Findings numérotés (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve**, correction proposée. ⛔ **Liste des axes
exercés ET non exercés** — un « 0 » sans elle ne compte pas. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/15-1a-ii-review-p2-ciblee.md` ; dernier message : chemin, bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh`), `make`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`, `sqlx`,
`cargo`, `npm`, `npx`, `docker`, `gh` en écriture, SQL. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`.
