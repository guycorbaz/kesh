# Prompt — revue de code P1, Story 25-1c-b2 (textes du journal d'audit)

*Versionné le 2026-09-26. **Une lentille** (Sonnet), contexte frais. Story de documentation : le
code de production touché se limite à une valeur de clé FTL (×3), un repli Svelte et le sous-titre
italien de l'écran.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-1c-b1-journal-audit-ecran`. Diff à relire :
`git diff ac0a2fc9 cd865a15` (commit de dev de la b2). Fiche :
`_bmad-output/implementation-artifacts/25-1c-b2-journal-audit-textes.md` (AC, inventaire des sites,
Dev Agent Record). L'écran décrit est celui de la b1 (même branche) :
`frontend/src/routes/(app)/audit-log/+page.svelte` et `frontend/src/lib/features/audit-log/`, la
route `crates/kesh-api/src/routes/` (module du journal d'audit).

## Axes — tous obligatoires

1. **Le manuel dit-il vrai ?** Chaque phrase neuve ou réécrite de `docs/manual/fr/user-manual.tex` et
   `admin-manual.tex` (sous-section « Consulter le journal d'audit », note de la numérotation,
   `:1802-1803` de l'admin, rôles) se contrôle **contre le code** : rôles autorisés, jours UTC, plage
   inversée, champ numéro, export (plafond, nom de fichier), ce qui est journalisé ou non, la
   dévalidation, la restauration d'une sauvegarde. Une promesse que le code ne tient pas est au moins
   MEDIUM.
2. **Le PDF, pas seulement le `.tex`** : aplatis les trois PDF
   (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`, vers
   `/tmp/claude-1000/-home-gcorbaz-devel-kesh/ca5ce2e2-67a3-4eeb-817f-c2de35620a1c/scratchpad/`),
   cherche les `??`, vérifie que le PDF porte le texte du `.tex` (régénéré après la dernière retouche).
3. **Vocabulaire** : rejoue l'inventaire de l'AC 3 **depuis le symptôme** (`piste d.audit`,
   `piste de contr[ôo]le`, `audit trail`, `Prüfpfad`, `traccia`, `pista di`), sur tout le dépôt
   visible de l'utilisateur (manuels, FTL ×4, Svelte, README, website). Tout résidu non listé comme
   légitime par la fiche est un finding. Contrôle aussi accords et articles (« le journal »).
4. **i18n** : les valeurs modifiées dans les 4 catalogues — sens, registre du glossaire
   (`docs/i18n-glossaire.md` § Registre : it = 2ᵉ personne du singulier, de = *Sie*, fr = vous),
   cohérence avec le terme du glossaire ; le repli Svelte égal à la valeur fr.
5. **README / CHANGELOG** : ce qui est annoncé pour v0.12.1 est-il livré sur cette branche, et rien de
   publié (v0.12.0) n'a-t-il été réécrit ?
6. **Dev Agent Record** : chaque décompte et chaque affirmation de gate est-il cohérent avec la source
   (recompte, ne relis pas) ?

## Ce que tu rends

- **Findings** : sévérité (CRITICAL / HIGH / MEDIUM / LOW), fichier:ligne, **preuve** (commande et
  résultat, code lu), correction proposée.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un « 0 finding » sans
  cette liste ne compte pas.

## Interdits

⛔ N'écris aucun fichier du dépôt ; n'exécute aucune commande qui écrit dans le dépôt ou dans une
base — `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, `latexmk`, tout `git commit`/`push`/`add`/
`stash`/`reset`/`rebase`/`checkout`/`switch`/`worktree`, `sqlx migrate`, `cargo test`/`nextest`,
`npm run`, `npx playwright`. Autorisés : lecture, `grep`, `git log`/`show`/`diff`, `gh issue view`,
`pdftotext` vers le scratchpad.
