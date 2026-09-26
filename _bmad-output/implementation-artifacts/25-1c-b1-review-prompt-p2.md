# Prompt — passe 2 de `bmad-code-review`, Story 25-1c-b1

*Versionné le 2026-09-26. Trois lentilles en contexte frais (Haiku 4.5) — cycle Sonnet → Haiku → Opus.
Diffs APLATIS (règle Haiku du `CLAUDE.md`).*

⛔ **Premier suspect : la remédiation de la passe 1** (entrée « review P1 » du Change Log et section
« Review Findings » de la fiche) — jeton de requête dans `loadList`, identifiant vidé à tout
changement de type, plage de dates contrôlée dans la page, « Précédent » dans l'état vide, `FakeURL`,
`goto` bouclé dans le test de page. Diff de la remédiation seule :
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/ca5ce2e2-67a3-4eeb-817f-c2de35620a1c/scratchpad/25-1c-b1-remediation-p1.diff`.
Reporté, **à ne pas re-signaler** : #469 (refus de validation de la route non traduits).

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-1c-b1-journal-audit-ecran`, tête `576464df`.
**Diff revu** : `git diff main..HEAD` sur le code (PDF et `_bmad-output/` exclus), enregistré dans
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/ca5ce2e2-67a3-4eeb-817f-c2de35620a1c/scratchpad/25-1c-b1-code-p2.diff`.
Fiche : `_bmad-output/implementation-artifacts/25-1c-b1-journal-audit-ecran.md` — AC 1 à 14, arbitrages
du Project Lead (**ne pas les contester**, en contester la mise en œuvre), Dev Agent Record. Contrat de la
route consommée : `crates/kesh-api/src/routes/audit_log.rs` (Story 25-1c-a, mergée).

Objet : l'**écran de consultation du journal d'audit** (#378) — feature `frontend/src/lib/features/audit-log/`,
page `frontend/src/routes/(app)/audit-log/`, module partagé `frontend/src/lib/shared/utils/download.ts`,
`i18nLocale()` dans le store i18n, entrée de menu, 26 clés ×4, gardes i18n, spec E2E. ⚠️ Les manuels et
le vocabulaire des catalogues sont la **25-1c-b2**, hors de cette revue.

## Lentille 1 — Blind Hunter (diff SEUL)

Revue adversariale générale : logique, réactivité Svelte 5 (`$state`, `$derived`, `$effect` — boucle
possible entre l'effet qui écrit l'URL et `onMount` qui la lit ?), gestion d'erreurs, tests qui ne
prouvent rien ou passent à vide, incohérences code / commentaires / textes, clés i18n (quatre locales,
replis **mot pour mot** le FTL fr-CH, vocabulaire arbitré), sélecteurs E2E par libellé.

## Lentille 2 — Edge Case Hunter (diff + dépôt)

Chaque branche du code neuf, en lisant l'appelé et l'appelant. Priorités :
1. **La page** : ordre des effets au montage (l'effet d'URL s'exécute-t-il avant que `onMount` ait lu
   l'URL, et l'écrase-t-il ?), double chargement, courses entre deux `loadList` (un filtre changé pendant
   un chargement — la réponse la plus ancienne peut-elle écraser la plus récente ?), `entityIdValue` lié à
   un `<input type="number">` (nombre ou chaîne ? `null` quand on efface ?), pagination aux bornes,
   `details` à `null` / chaîne / tableau, `createdAt` invalide.
2. **Le contrat** : chaque paramètre envoyé à `/api/v1/audit-log`, `/vocabulary`, `/export.csv` contre
   `ListAuditLogQuery` (noms, valeurs par défaut, `offset`/`limit` obligatoires ou non) ; chaque champ lu
   contre `AuditLogEntryResponse`. Une réponse 400 de la route (plage inversée, date hors bornes, action
   > 64 caractères, `entityId` sans type) est-elle atteignable depuis l'écran, et que voit l'utilisateur ?
3. **`i18nLocale()`** : qui d'autre charge les messages (`loadI18nMessages`) — la locale peut-elle rester
   `'fr-CH'` sur une installation allemande (écran ouvert avant le chargement) ?
4. **`download.ts`** : strictement identique à l'original d'`exports.api.ts` ? Les commentaires
   modifiés disent-ils vrai ?
5. **Les gardes i18n** : les compteurs modifiés, recomptés depuis la source ; `FAMILLES_RESOLUES`.
6. **L'E2E** : chaque scénario peut-il passer à vide (isolation par type et identifiant, attente de la
   liste avant l'assertion, fuseau `Europe/Zurich`, jour UTC) ?

## Lentille 3 — Acceptance Auditor (diff + fiche)

Chaque AC (1 à 14) contre le diff, sous-point par sous-point. Les décomptes du Dev Agent Record
**recomptés** depuis la source : 27 clés ×4, total 165 par catalogue, `sitesTotal` 1709 → 1740 (+31),
vitest +40 (et sa ventilation), E2E +2, 17 mutations (juge, en lisant le code et les tests, que chacune
aurait bien rougi). Les `data-testid` de l'AC 8 tous présents ? Aucun sélecteur par libellé ? Les trois
commentaires de l'AC 9 réécrits et `reports.api.ts` intact ? Les termes du glossaire
(`docs/i18n-glossaire.md`) respectés dans les quatre langues ?

## Ce que tu rends

- Findings : sévérité (CRITICAL / HIGH / MEDIUM / LOW), `fichier:ligne`, **preuve** (code lu, commande
  et résultat), correction. Pour tout CRITICAL ou HIGH affirmant qu'une chose est absente ou présente :
  la commande `grep -nF` exécutée et son résultat.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.**

## Interdits

⛔ N'écris aucun fichier du dépôt, n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante — `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, tout `git commit`/`push`/`add`/`stash`/`reset`/
`rebase`/`checkout`/`switch`/`worktree`, `sqlx migrate`, `cargo test`/`cargo nextest`, `npm run`,
`npx vitest`, `npx playwright`. Lecture, `grep`, `git log`/`show`/`diff` et `cargo check` sont autorisés.
